//! Native WebKit → portable web-error classification (task packet
//! `error-surface-contract`, adapter-mapping boundary).
//!
//! This module translates a native `NSError` (domain + code) into the
//! merged contract's closed taxonomy (`crates/bbr-web/src/web_errors`). It
//! is the **only** place native error codes are interpreted: the contract
//! deliberately contains none. The mapping is pure and unit-testable (no
//! AppKit objects beyond the plain `(domain, code)` pair), and the engine
//! delegate feeds it the `NSError` it receives from WebKit.
//!
//! ## Honesty rules
//!
//! - Every code maps to exactly one contract `(category, subreason)` pair
//!   or to `None`. `None` means "not a navigable incident" (user
//!   cancellation, JavaScript exceptions, unknown codes) — the surface
//!   never fabricates a cause for a code it does not know.
//! - TLS failures never map to a bypassable action: the contract's closed
//!   action set has no bypass variant, and this module only produces
//!   subreasons the contract already knows.
//! - HTTP error **statuses** (4xx/5xx with a delivered body) do **not** fail
//!   a WebKit navigation — the engine renders the server's error page, so
//!   there is no `NSError` to classify. This module therefore never invents
//!   an incident for a status the engine accepted. The HTTP-status-class
//!   failures WebKit *does* fail (an endless redirect chain →
//!   `NSURLErrorHTTPTooManyRedirects`, a truncated error response →
//!   `NSURLErrorNetworkConnectionLost`/`NSURLErrorBadServerResponse`) map
//!   into the navigation category.
//! - The failing URL never leaves this crate as a raw payload: the
//!   contract's privacy-safe [`IncidentContext`] carries only the reduced
//!   origin, and the full URL only travels as the app-side retry target.

#![forbid(unsafe_code)]

use bbr_web::web_errors::{
    CrashReason, IncidentContext, IncidentSubReason, NavigationReason, OfflineReason, TlsReason,
};
use bbr_web::{ProfileId, WebViewId};

/// The classified payload one native failure produces. The app-side error
/// surface owns the [`bbr_web::web_errors::IncidentId`] allocation (the
/// contract's "controller allocates increasing values" rule), so this type
/// deliberately carries no incident id or sequence — only the classification
/// and the identity the incident is scoped to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedIncident {
    /// The profile that owns the failing web view (incident scope).
    pub profile: ProfileId,
    /// The web view that failed (incident scope).
    pub web_view: WebViewId,
    /// The closed classification the native code maps to. `category` is
    /// derived from this subreason by the contract.
    pub reason: IncidentSubReason,
    /// Privacy-safe display context (reduced origin or none).
    pub context: IncidentContext,
    /// The full failing URL (engine-reported), the app's retry target.
    /// `None` when the engine reported none (e.g. a process termination).
    pub requested_url: Option<String>,
}

/// What a native error mapped to: the closed subreason plus a privacy-safe
/// context. `category` is always derived from the subreason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapOutcome {
    pub reason: IncidentSubReason,
}

impl MapOutcome {
    pub const fn new(reason: IncidentSubReason) -> Self {
        Self { reason }
    }
}

/// The `NSURLErrorDomain` codes this adapter understands, as signed i64
/// (the `NSError.code` value). Constants kept local so the mapping table is
/// reviewable in one place and independent of objc2 re-exports.
pub mod ns_url_error {
    pub const TIMED_OUT: i64 = -1001;
    pub const UNSUPPORTED_URL: i64 = -1002;
    pub const CANNOT_FIND_HOST: i64 = -1003;
    pub const CANNOT_CONNECT_TO_HOST: i64 = -1004;
    pub const NETWORK_CONNECTION_LOST: i64 = -1005;
    pub const HTTP_TOO_MANY_REDIRECTS: i64 = -1007;
    pub const NOT_CONNECTED_TO_INTERNET: i64 = -1009;
    pub const BAD_SERVER_RESPONSE: i64 = -1011;
    pub const SECURE_CONNECTION_FAILED: i64 = -1200;
    pub const SERVER_CERTIFICATE_HAS_BAD_DATE: i64 = -1201;
    pub const SERVER_CERTIFICATE_UNTRUSTED: i64 = -1202;
    pub const SERVER_CERTIFICATE_HAS_UNKNOWN_ROOT: i64 = -1203;
    pub const SERVER_CERTIFICATE_NOT_YET_VALID: i64 = -1204;
    pub const SERVER_CERTIFICATE_REVOKED: i64 = -1205;
    pub const CLIENT_CERTIFICATE_REJECTED: i64 = -1206;
}

/// The `NSURLErrorDomain` constant, as produced by `NSError.domain`.
pub const NS_URL_ERROR_DOMAIN: &str = "NSURLErrorDomain";

/// The failure URL key WebKit places in `NSError.userInfo`.
pub const FAILING_URL_KEY: &str = "NSErrorFailingURLStringKey";

/// The legacy failure-URL key WebKit still uses for some failure classes
/// (value is an `NSURL` or an `NSString`, unlike the String key above).
pub const FAILING_NSURL_KEY: &str = "NSErrorFailingURLKey";

/// Classify a native navigation error into the closed contract taxonomy.
///
/// `domain` and `code` come straight from the `NSError` WebKit delivered.
/// Returns `None` for codes that are not navigable incidents (user
/// cancellation, unknown codes, non-URL domains) so the surface never
/// guesses a cause. The failing URL is carried separately by [`classify`].
pub fn map_error(domain: &str, code: i64) -> Option<MapOutcome> {
    if domain != NS_URL_ERROR_DOMAIN {
        return None;
    }
    let reason: IncidentSubReason = match code {
        ns_url_error::TIMED_OUT => IncidentSubReason::Navigation(NavigationReason::RequestTimedOut),
        ns_url_error::UNSUPPORTED_URL => {
            IncidentSubReason::Navigation(NavigationReason::UnsupportedScheme)
        }
        ns_url_error::CANNOT_FIND_HOST => {
            IncidentSubReason::Navigation(NavigationReason::ServerNotFound)
        }
        ns_url_error::CANNOT_CONNECT_TO_HOST => {
            IncidentSubReason::Navigation(NavigationReason::ConnectionRefused)
        }
        ns_url_error::HTTP_TOO_MANY_REDIRECTS => {
            IncidentSubReason::Navigation(NavigationReason::RedirectLoop)
        }
        // The server answered an HTTP error status and then broke the
        // response (truncated body, malformed HTTP): an HTTP-level
        // navigation failure. Connection lost is a transient transport
        // failure of the same class.
        ns_url_error::NETWORK_CONNECTION_LOST | ns_url_error::BAD_SERVER_RESPONSE => {
            IncidentSubReason::Navigation(NavigationReason::RequestTimedOut)
        }
        ns_url_error::NOT_CONNECTED_TO_INTERNET => {
            IncidentSubReason::Offline(OfflineReason::NoNetworkConnection)
        }
        // TLS failures: the handshake or certificate chain failed. The
        // certificate-specific codes map to their exact subreasons; the
        // generic handshake failure stays generic — never a bypass.
        ns_url_error::SECURE_CONNECTION_FAILED => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::HandshakeFailed,
            )));
        }
        ns_url_error::SERVER_CERTIFICATE_HAS_BAD_DATE => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::CertificateExpired,
            )));
        }
        ns_url_error::SERVER_CERTIFICATE_UNTRUSTED
        | ns_url_error::SERVER_CERTIFICATE_HAS_UNKNOWN_ROOT => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::CertificateUntrusted,
            )));
        }
        ns_url_error::SERVER_CERTIFICATE_NOT_YET_VALID => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::CertificateNotYetValid,
            )));
        }
        ns_url_error::SERVER_CERTIFICATE_REVOKED => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::CertificateRevoked,
            )));
        }
        // Client-certificate rejection is a server-side identity failure of
        // the TLS handshake; the closed taxonomy's handshake class is the
        // honest bucket (there is no bypass variant anywhere).
        ns_url_error::CLIENT_CERTIFICATE_REJECTED => {
            return Some(MapOutcome::new(IncidentSubReason::Tls(
                TlsReason::HandshakeFailed,
            )));
        }
        // -1006 ResourceUnavailable and every other code stay unclassified:
        // the surface never guesses a cause it cannot name.
        _ => return None,
    };
    Some(MapOutcome::new(reason))
}

/// Classify a web-content-process termination: the only contract class for
/// a renderer death is `ProcessCrash/WebContentCrashed` (the engine cannot
/// know whether the system killed the process, so the crashed class is the
/// honest one; recovery is the contract's `Restore` — recreate the session).
pub fn map_termination() -> MapOutcome {
    MapOutcome::new(IncidentSubReason::Crash(CrashReason::WebContentCrashed))
}

/// Assemble a [`ClassifiedIncident`] from a mapping outcome, the owning
/// identity, and the engine-reported failing URL. The context is derived
/// privacy-safely from the failing URL; a termination (no URL) yields an
/// unknown-origin context.
pub fn classify(
    profile: ProfileId,
    web_view: WebViewId,
    outcome: MapOutcome,
    failing_url: Option<&str>,
) -> ClassifiedIncident {
    let context = failing_url.map_or_else(IncidentContext::default, IncidentContext::from_url);
    ClassifiedIncident {
        profile,
        web_view,
        reason: outcome.reason,
        context,
        requested_url: failing_url.map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_web::web_errors::{
        CrashReason, IncidentCategory, NavigationReason, OfflineReason, TlsReason,
    };

    fn classified(domain: &str, code: i64) -> Option<(IncidentCategory, IncidentSubReason)> {
        map_error(domain, code).map(|outcome| (outcome.reason.category(), outcome.reason))
    }

    #[test]
    fn navigation_codes_map_to_their_exact_subreasons() {
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::TIMED_OUT),
            Some((
                IncidentCategory::Navigation,
                IncidentSubReason::Navigation(NavigationReason::RequestTimedOut)
            ))
        );
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::UNSUPPORTED_URL),
            Some((
                IncidentCategory::Navigation,
                IncidentSubReason::Navigation(NavigationReason::UnsupportedScheme)
            ))
        );
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::CANNOT_FIND_HOST),
            Some((
                IncidentCategory::Navigation,
                IncidentSubReason::Navigation(NavigationReason::ServerNotFound)
            ))
        );
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::CANNOT_CONNECT_TO_HOST),
            Some((
                IncidentCategory::Navigation,
                IncidentSubReason::Navigation(NavigationReason::ConnectionRefused)
            ))
        );
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::HTTP_TOO_MANY_REDIRECTS),
            Some((
                IncidentCategory::Navigation,
                IncidentSubReason::Navigation(NavigationReason::RedirectLoop)
            ))
        );
    }

    #[test]
    fn offline_code_maps_to_offline() {
        assert_eq!(
            classified(NS_URL_ERROR_DOMAIN, ns_url_error::NOT_CONNECTED_TO_INTERNET),
            Some((
                IncidentCategory::Offline,
                IncidentSubReason::Offline(OfflineReason::NoNetworkConnection)
            ))
        );
    }

    #[test]
    fn tls_codes_map_without_any_bypass() {
        let cases = [
            (
                ns_url_error::SECURE_CONNECTION_FAILED,
                TlsReason::HandshakeFailed,
            ),
            (
                ns_url_error::SERVER_CERTIFICATE_HAS_BAD_DATE,
                TlsReason::CertificateExpired,
            ),
            (
                ns_url_error::SERVER_CERTIFICATE_UNTRUSTED,
                TlsReason::CertificateUntrusted,
            ),
            (
                ns_url_error::SERVER_CERTIFICATE_HAS_UNKNOWN_ROOT,
                TlsReason::CertificateUntrusted,
            ),
            (
                ns_url_error::SERVER_CERTIFICATE_NOT_YET_VALID,
                TlsReason::CertificateNotYetValid,
            ),
            (
                ns_url_error::SERVER_CERTIFICATE_REVOKED,
                TlsReason::CertificateRevoked,
            ),
            (
                ns_url_error::CLIENT_CERTIFICATE_REJECTED,
                TlsReason::HandshakeFailed,
            ),
        ];
        for (code, expected) in cases {
            let mapped = classified(NS_URL_ERROR_DOMAIN, code);
            assert_eq!(
                mapped,
                Some((IncidentCategory::Tls, IncidentSubReason::Tls(expected))),
                "code {code}"
            );
        }
    }

    #[test]
    fn unknown_and_non_failure_codes_stay_unclassified() {
        // User cancellation is not an incident.
        assert_eq!(classified(NS_URL_ERROR_DOMAIN, -999), None);
        // Resource unavailable has no honest closed subreason.
        assert_eq!(classified(NS_URL_ERROR_DOMAIN, -1006), None);
        // Unknown codes never fabricate a cause.
        assert_eq!(classified(NS_URL_ERROR_DOMAIN, -12345), None);
        // Non-URL domains are ignored entirely.
        assert_eq!(classified("WebKitErrorDomain", 4), None);
        assert_eq!(classified("CocoaErrorDomain", 1), None);
    }

    #[test]
    fn truncated_http_error_response_is_an_http_level_navigation_failure() {
        // The engine fails a truncated/ill-formed HTTP error response; the
        // adapter classifies it as a transient navigation failure — never as
        // a fabricated category for a status the engine accepted.
        for code in [
            ns_url_error::NETWORK_CONNECTION_LOST,
            ns_url_error::BAD_SERVER_RESPONSE,
        ] {
            let mapped = classified(NS_URL_ERROR_DOMAIN, code);
            assert_eq!(
                mapped,
                Some((
                    IncidentCategory::Navigation,
                    IncidentSubReason::Navigation(NavigationReason::RequestTimedOut)
                ))
            );
        }
    }

    #[test]
    fn termination_maps_to_the_crash_class() {
        let outcome = map_termination();
        assert_eq!(
            outcome.reason,
            IncidentSubReason::Crash(CrashReason::WebContentCrashed)
        );
        assert_eq!(outcome.reason.category(), IncidentCategory::ProcessCrash);
    }

    #[test]
    fn classify_derives_privacy_safe_context_and_retry_url() {
        let incident = classify(
            ProfileId(1),
            WebViewId(2),
            map_error(NS_URL_ERROR_DOMAIN, ns_url_error::CANNOT_CONNECT_TO_HOST).expect("mapped"),
            Some("https://User:Pass@Example.com:8443/path?q=1#frag"),
        );
        assert_eq!(incident.profile, ProfileId(1));
        assert_eq!(incident.web_view, WebViewId(2));
        // Privacy-safe: origin only, lower-cased, userinfo stripped.
        assert_eq!(incident.context.origin(), Some("https://example.com:8443"));
        // The full failing URL is preserved as the app-side retry target.
        assert_eq!(
            incident.requested_url.as_deref(),
            Some("https://User:Pass@Example.com:8443/path?q=1#frag")
        );
        // A termination has no failing URL: unknown-origin context.
        let terminated = classify(ProfileId(1), WebViewId(2), map_termination(), None);
        assert_eq!(terminated.context.origin(), None);
        assert_eq!(terminated.requested_url, None);
    }

    #[test]
    fn classifiable_payload_is_plain_data() {
        let incident = classify(
            ProfileId(1),
            WebViewId(2),
            map_error(NS_URL_ERROR_DOMAIN, ns_url_error::HTTP_TOO_MANY_REDIRECTS).expect("mapped"),
            Some("http://loop.example/"),
        );
        assert_eq!(incident.clone(), incident);
    }
}
