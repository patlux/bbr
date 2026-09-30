//! HTTP authentication requests (Basic, Digest, NTLM; origin and proxy).
//!
//! The navigation delegate maps WebKit's challenge onto the portable
//! protection space and asks the app, which presents the credential sheet.
//! WebKit's completion handler waits in the shared table until the app
//! answers through [`bbr_web::WebEngine::answer_http_auth`]; a view that
//! goes away answers without a credential (WebKit expects every handler to
//! be called). Everything else — server trust, Negotiate, HTML forms,
//! mechanisms BBR does not map — keeps WebKit's default handling.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2_foundation::{
    NSString, NSURLAuthenticationChallenge, NSURLCredential, NSURLCredentialPersistence,
    NSURLProtectionSpace, NSURLSessionAuthChallengeDisposition,
};

use bbr_web::auth_challenges::{AuthAuthority, AuthScheme, HttpScheme, ProtectionSpace};
use bbr_web::http_auth::HttpAuthAnswer;
use bbr_web::{WebEvent, WebViewId};

type Handler = RcBlock<dyn Fn(NSURLSessionAuthChallengeDisposition, *mut NSURLCredential)>;

/// The open requests, shared by every delegate and the engine.
#[derive(Default)]
pub struct HttpAuthTable {
    next: u64,
    open: HashMap<u64, (WebViewId, Handler)>,
}

pub type HttpAuthRequests = Rc<RefCell<HttpAuthTable>>;

impl HttpAuthTable {
    /// Keep the handler; return the request's id.
    pub(crate) fn register(&mut self, view: WebViewId, handler: Handler) -> u64 {
        self.next += 1;
        self.open.insert(self.next, (view, handler));
        self.next
    }

    /// Answer one request (unknown ids are no-ops).
    pub(crate) fn answer(&mut self, request: u64, answer: &HttpAuthAnswer) {
        let Some((_, handler)) = self.open.remove(&request) else {
            return;
        };
        match answer {
            HttpAuthAnswer::Credential(credential) => {
                // `ForSession`: WebKit keeps it in the profile's own network
                // session, so later loads of the space sign in without
                // asking again (Safari's behaviour); another profile's data
                // store never sees it.
                let credential = NSURLCredential::credentialWithUser_password_persistence(
                    &NSString::from_str(credential.user()),
                    &NSString::from_str(credential.password()),
                    NSURLCredentialPersistence::ForSession,
                );
                handler.call((
                    NSURLSessionAuthChallengeDisposition::UseCredential,
                    Retained::as_ptr(&credential).cast_mut(),
                ));
            }
            // `UseCredential` with no credential continues without one: the
            // server's own answer loads.
            HttpAuthAnswer::WithoutCredential => handler.call((
                NSURLSessionAuthChallengeDisposition::UseCredential,
                std::ptr::null_mut(),
            )),
        }
    }

    /// Answer every open request of a view that goes away.
    pub(crate) fn cancel_view(&mut self, view: WebViewId) {
        let requests: Vec<u64> = self
            .open
            .iter()
            .filter(|(_, (owner, _))| *owner == view)
            .map(|(request, _)| *request)
            .collect();
        for request in requests {
            self.answer(request, &HttpAuthAnswer::WithoutCredential);
        }
    }
}

/// Keep the handler and describe the request when BBR answers this
/// challenge; `None` leaves it to WebKit's default handling.
pub(crate) fn request(
    challenge: &NSURLAuthenticationChallenge,
    view: WebViewId,
    table: &HttpAuthRequests,
    handler: Handler,
) -> Option<WebEvent> {
    let space = challenge.protectionSpace();
    let scheme = scheme_of(&space.authenticationMethod())?;
    let space = protection_space(&space)?;
    let previous_failures = u32::try_from(challenge.previousFailureCount()).unwrap_or(0);
    let request = table.borrow_mut().register(view, handler);
    Some(WebEvent::HttpAuthRequested {
        view,
        request,
        space,
        scheme,
        previous_failures,
    })
}

/// The schemes the credential sheet answers.
fn scheme_of(method: &NSString) -> Option<AuthScheme> {
    // SAFETY: static NSStrings exported by Foundation.
    let (default, basic, digest, ntlm) = unsafe {
        (
            objc2_foundation::NSURLAuthenticationMethodDefault,
            objc2_foundation::NSURLAuthenticationMethodHTTPBasic,
            objc2_foundation::NSURLAuthenticationMethodHTTPDigest,
            objc2_foundation::NSURLAuthenticationMethodNTLM,
        )
    };
    // `Default` is HTTP Basic for an HTTP protection space.
    if method.isEqualToString(basic) || method.isEqualToString(default) {
        Some(AuthScheme::Basic)
    } else if method.isEqualToString(digest) {
        Some(AuthScheme::Digest)
    } else if method.isEqualToString(ntlm) {
        Some(AuthScheme::Ntlm)
    } else {
        None
    }
}

/// The portable protection space of an HTTP(S) origin or proxy; `None`
/// for anything else (FTP, SOCKS) or a space the contract rejects.
fn protection_space(space: &NSURLProtectionSpace) -> Option<ProtectionSpace> {
    // SAFETY: static NSStrings exported by Foundation.
    let (http, https, http_proxy, https_proxy) = unsafe {
        (
            objc2_foundation::NSURLProtectionSpaceHTTP,
            objc2_foundation::NSURLProtectionSpaceHTTPS,
            objc2_foundation::NSURLProtectionSpaceHTTPProxy,
            objc2_foundation::NSURLProtectionSpaceHTTPSProxy,
        )
    };
    let (kind, authority) = if space.isProxy() {
        (space.proxyType()?, AuthAuthority::Proxy)
    } else {
        (space.protocol()?, AuthAuthority::Origin)
    };
    let scheme = if kind.isEqualToString(http) || kind.isEqualToString(http_proxy) {
        HttpScheme::Http
    } else if kind.isEqualToString(https) || kind.isEqualToString(https_proxy) {
        HttpScheme::Https
    } else {
        return None;
    };
    let port = u16::try_from(space.port()).ok()?;
    let realm = space.realm().map(|realm| realm.to_string());
    ProtectionSpace::new(
        scheme,
        &space.host().to_string(),
        port,
        realm.as_deref().filter(|realm| !realm.trim().is_empty()),
        authority,
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::AnyThread;

    fn space(protocol: &NSString, method: &NSString) -> Retained<NSURLProtectionSpace> {
        NSURLProtectionSpace::initWithHost_port_protocol_realm_authenticationMethod(
            NSURLProtectionSpace::alloc(),
            &NSString::from_str("Intranet.Example"),
            8080,
            Some(protocol),
            Some(&NSString::from_str("Finance")),
            Some(method),
        )
    }

    #[test]
    fn basic_digest_and_ntlm_map_and_other_methods_stay_default() {
        // SAFETY: static NSStrings exported by Foundation.
        unsafe {
            use objc2_foundation::*;
            assert_eq!(
                scheme_of(NSURLAuthenticationMethodHTTPBasic),
                Some(AuthScheme::Basic)
            );
            assert_eq!(
                scheme_of(NSURLAuthenticationMethodDefault),
                Some(AuthScheme::Basic)
            );
            assert_eq!(
                scheme_of(NSURLAuthenticationMethodHTTPDigest),
                Some(AuthScheme::Digest)
            );
            assert_eq!(
                scheme_of(NSURLAuthenticationMethodNTLM),
                Some(AuthScheme::Ntlm)
            );
            for other in [
                NSURLAuthenticationMethodServerTrust,
                NSURLAuthenticationMethodNegotiate,
                NSURLAuthenticationMethodHTMLForm,
                NSURLAuthenticationMethodClientCertificate,
            ] {
                assert_eq!(scheme_of(other), None, "{other}");
            }
        }
    }

    #[test]
    fn an_origin_space_maps_scheme_host_port_and_realm() {
        // SAFETY: static NSStrings exported by Foundation.
        let (https, basic) = unsafe {
            (
                objc2_foundation::NSURLProtectionSpaceHTTPS,
                objc2_foundation::NSURLAuthenticationMethodHTTPBasic,
            )
        };
        let mapped = protection_space(&space(https, basic)).expect("an https origin");
        assert_eq!(mapped.scheme(), HttpScheme::Https);
        assert_eq!(mapped.host().as_str(), "intranet.example");
        assert_eq!(mapped.port(), 8080);
        assert_eq!(mapped.realm().map(|realm| realm.as_str()), Some("Finance"));
        assert!(!mapped.is_proxy());
    }

    #[test]
    fn a_proxy_space_is_a_proxy_authority_and_ftp_is_not_mapped() {
        // SAFETY: static NSStrings exported by Foundation.
        let (http_proxy, basic) = unsafe {
            (
                objc2_foundation::NSURLProtectionSpaceHTTPProxy,
                objc2_foundation::NSURLAuthenticationMethodHTTPBasic,
            )
        };
        let ftp = NSString::from_str("ftp");
        let proxy = NSURLProtectionSpace::initWithProxyHost_port_type_realm_authenticationMethod(
            NSURLProtectionSpace::alloc(),
            &NSString::from_str("proxy.example"),
            3128,
            Some(http_proxy),
            None,
            Some(basic),
        );
        let mapped = protection_space(&proxy).expect("an http proxy");
        assert!(mapped.is_proxy());
        assert_eq!(mapped.scheme(), HttpScheme::Http);
        assert_eq!(mapped.port(), 3128);
        assert_eq!(protection_space(&space(&ftp, basic)), None);
    }
}
