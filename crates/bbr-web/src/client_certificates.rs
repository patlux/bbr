//! Blueprint D12 (board M-11): a site asks for a client certificate.
//!
//! The engine lists the Keychain identities that match the server's
//! accepted issuers as plain metadata — the certificate's name, its issuer,
//! its expiry and a stable fingerprint — and keeps the identities
//! themselves. No key material and no certificate bytes ever leave the
//! engine; the app answers with a fingerprint, no certificate, or a cancel.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One Keychain identity the site could accept (metadata only).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIdentityInfo {
    /// SHA-256 of the certificate, lowercase hex: stable across launches,
    /// so a remembered choice can find the identity again.
    pub fingerprint: String,
    /// The certificate's subject summary ("Patrick Wozniak").
    pub name: String,
    /// The issuer's common name ("Corp Issuing CA 2").
    pub issuer: String,
    /// The end of the validity period, seconds since the Unix epoch.
    pub expires_unix: Option<i64>,
}

impl ClientIdentityInfo {
    /// Whether the certificate has expired at `now_unix`.
    pub fn expired(&self, now_unix: i64) -> bool {
        self.expires_unix.is_some_and(|expires| expires <= now_unix)
    }
}

/// How the app answers a client-certificate request.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientCertificateAnswer {
    /// Identify with the identity of this fingerprint.
    Identity(String),
    /// Continue without a certificate ("Don't Send").
    NoCertificate,
    /// Cancel the request.
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiry_is_judged_against_now() {
        let info = ClientIdentityInfo {
            fingerprint: "ab".into(),
            name: "Patrick".into(),
            issuer: "Corp CA".into(),
            expires_unix: Some(100),
        };
        assert!(!info.expired(99));
        assert!(info.expired(100));
        let open = ClientIdentityInfo {
            expires_unix: None,
            ..info
        };
        assert!(!open.expired(i64::MAX));
    }
}
