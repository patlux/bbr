//! Closed challenge taxonomy: supported HTTP schemes, the challenge kinds the
//! contract distinguishes, and the typed set of unsupported challenges.
//!
//! The taxonomy is deliberately closed. A challenge is exactly one of:
//! HTTP authentication (origin or proxy, per the protection space's
//! authority), TLS client-certificate selection, or a typed unsupported
//! challenge. Unsupported challenges are distinct types — never opaque native
//! payloads, raw error codes, or secret material.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A closed set of HTTP authentication schemes the contract maps onto adapter
/// support. Anything else is an [`UnsupportedChallenge`], never an opaque
/// native payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthScheme {
    /// HTTP Basic (RFC 7617).
    Basic,
    /// HTTP Digest (RFC 7616).
    Digest,
    /// Windows NTLM challenge/response.
    Ntlm,
    /// Negotiate (SPNEGO/Kerberos).
    Negotiate,
}

impl AuthScheme {
    /// All supported schemes, in stable order. Used by the closed-set tests
    /// and by adapters that enumerate mapping coverage.
    pub const ALL: [AuthScheme; 4] = [
        AuthScheme::Basic,
        AuthScheme::Digest,
        AuthScheme::Ntlm,
        AuthScheme::Negotiate,
    ];

    /// The canonical scheme token for diagnostics and adapter mapping
    /// (mirrors the WebKit `NSURLAuthenticationMethodHTTP*` suffix).
    pub const fn as_str(self) -> &'static str {
        match self {
            AuthScheme::Basic => "Basic",
            AuthScheme::Digest => "Digest",
            AuthScheme::Ntlm => "NTLM",
            AuthScheme::Negotiate => "Negotiate",
        }
    }
}

/// A challenge the authentication authority cannot answer with a credential
/// or client identity. Typed so callers can surface a precise explanation;
/// none of the variants carries native payload or secret material.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnsupportedChallenge {
    /// Form-based HTML authentication (WebKit
    /// `NSURLAuthenticationMethodHTMLForm`). Not part of the supported HTTP
    /// scheme set.
    HtmlForm,
    /// TLS server-trust evaluation. Owned by the web-error contract; the
    /// authentication authority never bypasses or answers trust challenges.
    ServerTrust,
    /// Any other mechanism the adapter could not map. `mechanism` is a short
    /// diagnostic label (for example `"KerberosV4"`), not a native payload.
    Other {
        /// Diagnostic-only mechanism label.
        mechanism: String,
    },
}

/// The closed set of challenge kinds the contract distinguishes.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChallengeKind {
    /// HTTP authentication against the authority named by the protection
    /// space — origin or proxy ([`crate::auth_challenges::AuthAuthority`]).
    /// The scheme is one of the closed [`AuthScheme`] set.
    HttpAuth(AuthScheme),
    /// TLS client-certificate selection. Legal only on an `https`
    /// protection space; construction enforces this.
    ClientCertificate,
    /// A typed unsupported challenge; no credential or client identity can
    /// answer it.
    Unsupported(UnsupportedChallenge),
}
