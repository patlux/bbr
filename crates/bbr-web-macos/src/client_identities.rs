//! Blueprint D12 (board M-11): client-certificate requests.
//!
//! When a server asks for a client certificate, the navigation delegate
//! looks up the Keychain identities issued by one of the server's accepted
//! issuers and hands the app their metadata — subject, issuer, expiry and a
//! SHA-256 fingerprint of the certificate — never key material. The
//! identities and WebKit's completion handler wait in the shared table
//! until the app answers; destroying the view cancels the request.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{msg_send, ClassType};
use objc2_core_foundation::{CFArray, CFDictionary, CFRetained, CFString, CFType};
use objc2_foundation::{
    NSArray, NSData, NSDictionary, NSNumber, NSString, NSURLCredential, NSURLCredentialPersistence,
    NSURLSessionAuthChallengeDisposition,
};
use objc2_security::{SecCertificate, SecIdentity};

use bbr_web::client_certificates::{ClientCertificateAnswer, ClientIdentityInfo};
use bbr_web::WebViewId;

type Handler = RcBlock<dyn Fn(NSURLSessionAuthChallengeDisposition, *mut NSURLCredential)>;

/// Seconds between the Core Foundation epoch (2001) and the Unix epoch.
const CF_EPOCH_OFFSET: f64 = 978_307_200.0;

struct OpenRequest {
    view: WebViewId,
    handler: Handler,
    identities: Vec<(String, CFRetained<SecIdentity>)>,
}

/// The open requests, shared by every delegate and the engine.
#[derive(Default)]
pub struct ClientCertificateTable {
    next: u64,
    open: HashMap<u64, OpenRequest>,
}

pub type ClientCertificateRequests = Rc<RefCell<ClientCertificateTable>>;

impl ClientCertificateTable {
    /// Keep the handler and the identities; return the request's id.
    pub(crate) fn register(
        &mut self,
        view: WebViewId,
        handler: Handler,
        identities: Vec<(String, CFRetained<SecIdentity>)>,
    ) -> u64 {
        self.next += 1;
        self.open.insert(
            self.next,
            OpenRequest {
                view,
                handler,
                identities,
            },
        );
        self.next
    }

    /// Answer one request (unknown ids are no-ops).
    pub(crate) fn answer(&mut self, request: u64, answer: &ClientCertificateAnswer) {
        let Some(open) = self.open.remove(&request) else {
            return;
        };
        match answer {
            ClientCertificateAnswer::Identity(fingerprint) => {
                match open
                    .identities
                    .iter()
                    .find(|(candidate, _)| candidate == fingerprint)
                {
                    Some((_, identity)) => {
                        let credential = credential_for(identity);
                        open.handler.call((
                            NSURLSessionAuthChallengeDisposition::UseCredential,
                            Retained::as_ptr(&credential).cast_mut(),
                        ));
                    }
                    // An unknown fingerprint never sends anything.
                    None => open.handler.call((
                        NSURLSessionAuthChallengeDisposition::CancelAuthenticationChallenge,
                        std::ptr::null_mut(),
                    )),
                }
            }
            // `UseCredential` with no credential continues without one.
            ClientCertificateAnswer::NoCertificate => open.handler.call((
                NSURLSessionAuthChallengeDisposition::UseCredential,
                std::ptr::null_mut(),
            )),
            ClientCertificateAnswer::Cancel => open.handler.call((
                NSURLSessionAuthChallengeDisposition::CancelAuthenticationChallenge,
                std::ptr::null_mut(),
            )),
        }
    }

    /// Cancel every open request of a view that goes away.
    pub(crate) fn cancel_view(&mut self, view: WebViewId) {
        let requests: Vec<u64> = self
            .open
            .iter()
            .filter(|(_, open)| open.view == view)
            .map(|(request, _)| *request)
            .collect();
        for request in requests {
            self.answer(request, &ClientCertificateAnswer::Cancel);
        }
    }
}

/// The session credential for `identity` (no intermediate certificates:
/// the server's chain is its own concern).
fn credential_for(identity: &SecIdentity) -> Retained<NSURLCredential> {
    let certificates: Option<&NSArray> = None;
    // SAFETY: `credentialWithIdentity:certificates:persistence:` takes a
    // live SecIdentityRef, an optional array and a persistence value; it
    // returns an autoreleased credential that `Retained` keeps.
    unsafe {
        msg_send![
            NSURLCredential::class(),
            credentialWithIdentity: identity,
            certificates: certificates,
            persistence: NSURLCredentialPersistence::ForSession
        ]
    }
}

fn cf_as_ns(string: &CFString) -> &NSString {
    // SAFETY: CFString and NSString are toll-free bridged.
    unsafe { &*(string as *const CFString).cast::<NSString>() }
}

fn cf_dictionary_as_ns(dictionary: &CFDictionary) -> &NSDictionary<AnyObject, AnyObject> {
    // SAFETY: CFDictionary and NSDictionary are toll-free bridged.
    unsafe { &*(dictionary as *const CFDictionary).cast::<NSDictionary<AnyObject, AnyObject>>() }
}

fn lookup<'a>(
    dictionary: &'a NSDictionary<AnyObject, AnyObject>,
    key: &CFString,
) -> Option<&'a AnyObject> {
    let key: &AnyObject = cf_as_ns(key).as_ref();
    // SAFETY: a lookup in a live dictionary; the value lives as long as
    // the dictionary (borrowed, not retained).
    unsafe {
        let value: *mut AnyObject = msg_send![dictionary, objectForKey: key];
        value.as_ref()
    }
}

/// SHA-256 of `bytes`, lowercase hex (CommonCrypto, part of libSystem).
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    extern "C" {
        fn CC_SHA256(data: *const c_void, len: u32, md: *mut u8) -> *mut u8;
    }
    let mut digest = [0u8; 32];
    // SAFETY: CC_SHA256 reads `len` bytes and writes 32 into `digest`.
    unsafe {
        CC_SHA256(
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            digest.as_mut_ptr(),
        )
    };
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The metadata of one certificate.
pub(crate) fn certificate_info(certificate: &SecCertificate) -> ClientIdentityInfo {
    use objc2_security::{
        kSecOIDCommonName, kSecOIDOrganizationName, kSecOIDX509V1IssuerName,
        kSecOIDX509V1ValidityNotAfter, kSecPropertyKeyLabel, kSecPropertyKeyValue,
    };
    // SAFETY: a live certificate; the returned objects are owned.
    let data = unsafe { certificate.data() };
    let fingerprint = sha256_hex(data.to_vec().as_slice());
    let name = unsafe { certificate.subject_summary() }
        .map(|summary| summary.to_string())
        .unwrap_or_default();
    // SAFETY: statics exported by Security.framework.
    let (issuer_key, expiry_key, value_key, label_key, cn, org) = unsafe {
        (
            kSecOIDX509V1IssuerName,
            kSecOIDX509V1ValidityNotAfter,
            kSecPropertyKeyValue,
            kSecPropertyKeyLabel,
            kSecOIDCommonName,
            kSecOIDOrganizationName,
        )
    };
    let keys = CFArray::from_objects(&[issuer_key, expiry_key]);
    // SAFETY: a live certificate, a keys array of CFStrings, no error out.
    let values = unsafe { certificate.values(Some(keys.as_opaque()), std::ptr::null_mut()) };
    let mut issuer = String::new();
    let mut expires_unix = None;
    if let Some(values) = values {
        let values = cf_dictionary_as_ns(&values);
        let entry = |key: &CFString| {
            lookup(values, key)
                .and_then(|entry| entry.downcast_ref::<NSDictionary<AnyObject, AnyObject>>())
                .and_then(|entry| lookup(entry, value_key))
        };
        if let Some(parts) = entry(issuer_key).and_then(|v| v.downcast_ref::<NSArray<AnyObject>>())
        {
            let part = |oid: &CFString| {
                parts.iter().find_map(|part| {
                    let part = part.downcast_ref::<NSDictionary<AnyObject, AnyObject>>()?;
                    let label = lookup(part, label_key)?.downcast_ref::<NSString>()?;
                    if label.to_string() != cf_as_ns(oid).to_string() {
                        return None;
                    }
                    Some(
                        lookup(part, value_key)?
                            .downcast_ref::<NSString>()?
                            .to_string(),
                    )
                })
            };
            issuer = part(cn).or_else(|| part(org)).unwrap_or_default();
        }
        if let Some(seconds) = entry(expiry_key).and_then(|v| v.downcast_ref::<NSNumber>()) {
            expires_unix = Some((seconds.doubleValue() + CF_EPOCH_OFFSET) as i64);
        }
    }
    ClientIdentityInfo {
        fingerprint,
        name,
        issuer,
        expires_unix,
    }
}

/// The Keychain identities issued by one of `issuers` (the server's
/// accepted DER-encoded distinguished names; empty accepts any).
pub(crate) fn matching_identities(
    issuers: Option<&NSArray<NSData>>,
) -> Vec<(ClientIdentityInfo, CFRetained<SecIdentity>)> {
    use objc2_security::{
        kSecClass, kSecClassIdentity, kSecMatchIssuers, kSecMatchLimit, kSecMatchLimitAll,
        kSecReturnRef, SecItemCopyMatching,
    };
    // SAFETY: statics exported by Security.framework.
    let (class_key, class_identity, limit_key, limit_all, return_ref, issuers_key) = unsafe {
        (
            kSecClass,
            kSecClassIdentity,
            kSecMatchLimit,
            kSecMatchLimitAll,
            kSecReturnRef,
            kSecMatchIssuers,
        )
    };
    let yes = NSNumber::new_bool(true);
    let mut keys: Vec<&NSString> = vec![
        cf_as_ns(class_key),
        cf_as_ns(limit_key),
        cf_as_ns(return_ref),
    ];
    let mut values: Vec<&AnyObject> = vec![
        cf_as_ns(class_identity).as_ref(),
        cf_as_ns(limit_all).as_ref(),
        yes.as_ref(),
    ];
    if let Some(issuers) = issuers.filter(|issuers| issuers.count() > 0) {
        keys.push(cf_as_ns(issuers_key));
        values.push(issuers.as_ref());
    }
    let query = NSDictionary::from_slices(&keys, &values);
    let mut result: *const CFType = std::ptr::null();
    // SAFETY: a valid query dictionary (bridged) and an out pointer.
    let status = unsafe {
        SecItemCopyMatching(
            &*(Retained::as_ptr(&query).cast::<CFDictionary>()),
            &mut result,
        )
    };
    let Some(result) = NonNull::new(result.cast_mut()).filter(|_| status == 0) else {
        return Vec::new();
    };
    // SAFETY: with kSecMatchLimitAll + kSecReturnRef the result is an owned
    // CFArray of SecIdentityRefs.
    let array: CFRetained<CFArray> = unsafe { CFRetained::from_raw(result.cast()) };
    let array = unsafe {
        &*(CFRetained::as_ptr(&array)
            .as_ptr()
            .cast::<NSArray<AnyObject>>())
    };
    array
        .iter()
        .filter_map(|item| {
            // SAFETY: every element is a SecIdentityRef.
            let identity: CFRetained<SecIdentity> =
                unsafe { CFRetained::retain(NonNull::from(&*item).cast::<SecIdentity>()) };
            let mut certificate: *mut SecCertificate = std::ptr::null_mut();
            // SAFETY: a live identity and an out pointer.
            let status = unsafe { identity.copy_certificate(NonNull::from(&mut certificate)) };
            let certificate = NonNull::new(certificate).filter(|_| status == 0)?;
            // SAFETY: the copy is owned by us.
            let certificate: CFRetained<SecCertificate> =
                unsafe { CFRetained::from_raw(certificate) };
            Some((certificate_info(&certificate), identity))
        })
        .collect()
}

/// Build the table entry and the event payload for one challenge.
pub(crate) fn identities_for_challenge(
    issuers: Option<Retained<NSArray<NSData>>>,
) -> (
    Vec<ClientIdentityInfo>,
    Vec<(String, CFRetained<SecIdentity>)>,
) {
    let found = matching_identities(issuers.as_deref());
    let infos = found.iter().map(|(info, _)| info.clone()).collect();
    let identities = found
        .into_iter()
        .map(|(info, identity)| (info.fingerprint, identity))
        .collect();
    (infos, identities)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A self-signed test certificate without a key (CN "BBR Test
    /// Identity", O "BBR Test", valid until 2036-09-26T17:33:47Z), DER,
    /// base64.
    const TEST_CERT_DER_B64: &str = include_str!("../tests/fixtures/client-identity.der.b64");

    fn decode_base64(text: &str) -> Vec<u8> {
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = Vec::new();
        let mut buffer = 0u32;
        let mut bits = 0;
        for byte in text
            .bytes()
            .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        {
            let value = alphabet.iter().position(|a| *a == byte).expect("base64") as u32;
            buffer = (buffer << 6) | value;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buffer >> bits) as u8);
            }
        }
        out
    }

    #[test]
    fn certificate_metadata_is_read_without_the_keychain() {
        let der = decode_base64(TEST_CERT_DER_B64);
        let data = objc2_core_foundation::CFData::from_bytes(&der);
        // SAFETY: valid DER bytes.
        let certificate = unsafe { SecCertificate::with_data(None, &data) }.expect("certificate");
        let info = certificate_info(&certificate);
        assert_eq!(info.name, "BBR Test Identity");
        assert_eq!(info.issuer, "BBR Test Identity");
        assert_eq!(info.expires_unix, Some(2_106_063_227));
        assert_eq!(info.fingerprint, sha256_hex(&der));
        assert_eq!(info.fingerprint.len(), 64);
    }

    #[test]
    fn sha256_matches_the_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
