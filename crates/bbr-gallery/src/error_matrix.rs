//! The Error/Crash matrix authority (packet
//! `component-gallery-browser-components.md`, Group 7): **one declaration
//! source** for every valid §11.8 error-family row and every rejected
//! combination. The gallery stories render exactly the valid families
//! through the production `bbr_ui::surfaces::error` surface, and the
//! fixture builder refuses (`None`) any case the validity authority
//! rejects — so the story, the fixture builder, the matrix table, and the
//! tests cannot drift. The rejected reasons also live here. The §11.8
//! glass panes (crash panel / empty pane) are the sibling authority
//! [`crate::error_panel_matrix`]; together the two files are the one
//! Error/Crash matrix declaration.
//!
//! Matrix rows come straight from the packet's Error/Crash row: TLS
//! (expired / not-yet-valid / untrusted / revoked / hostname-mismatch /
//! handshake), Navigation (server-not-found / refused / timed-out /
//! redirect-loop / unsupported-scheme), and Offline (no-network /
//! proxy-unreachable / network-unreachable) — each with the calm
//! title/copy the production surface renders (the exact strings of the
//! app-owned `copy_for` at extraction time, byte-identical) and the
//! contract-legal action (`Retry` → "Try again"); origin shown vs omitted.
//! Rejected: a TLS action other than Retry (the closed contract action set
//! has no bypass variant) and raw platform error codes in any copy.

use bbr_ui::surfaces::error::ErrorSurfaceModel;
use gpui::SharedString;

/// The production Retry action label (the app's
/// `action_label(RecoveryAction::Retry)`): every TLS / navigation /
/// offline family renders this primary action. The matrix pins it so a
/// bypass-like action label can never become a valid TLS family.
pub const RETRY_ACTION_LABEL: &str = "Try again";
/// The production Dismiss label (the surface's secondary action).
pub const DISMISS_LABEL: &str = "Dismiss";

/// The §11.8 presentation family of an error case (the closed contract
/// categories, presentation view only — the incident lifecycle and the
/// category → action policy stay app-owned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorFamily {
    Tls,
    Navigation,
    Offline,
}

impl ErrorFamily {
    /// The story's meta label for the family.
    pub const fn label(self) -> &'static str {
        match self {
            ErrorFamily::Tls => "TLS",
            ErrorFamily::Navigation => "navigation",
            ErrorFamily::Offline => "offline",
        }
    }
}

/// One matrix row: a family label, the presentation family, the calm
/// title/copy the production surface renders, the readable failing origin
/// (shown vs omitted), and the production action/dismiss labels. The
/// matrix-only fields (`claims_tls_bypass`, `claims_raw_code`) exist
/// solely to name the rejected combinations in the same case shape; they
/// are never mapped into `ErrorSurfaceModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCase {
    pub label: &'static str,
    pub family: ErrorFamily,
    /// Calm, conventional title (never a raw platform code).
    pub title: &'static str,
    /// Calm explanation: what failed and what the user can do.
    pub copy: &'static str,
    /// The readable failing origin the surface shows, or `None` (the URL
    /// line is omitted rather than guessed).
    pub origin: Option<&'static str>,
    /// The primary action label (the production Retry label for every
    /// TLS/navigation/offline family).
    pub action_label: &'static str,
    /// The secondary action label (Dismiss — a lifecycle event).
    pub dismiss_label: &'static str,
    /// Matrix-only: this case claims a TLS bypass action (the contract
    /// action set has no bypass variant; TLS maps to Retry only).
    pub claims_tls_bypass: bool,
    /// Matrix-only: this case claims raw platform error codes in the copy
    /// (the surface copy is calm and conventional — raw codes never
    /// appear as the primary message).
    pub claims_raw_code: bool,
}

impl ErrorCase {
    /// The valid-family constructor: the six production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        family: ErrorFamily,
        title: &'static str,
        copy: &'static str,
        origin: Option<&'static str>,
    ) -> Self {
        ErrorCase {
            label,
            family,
            title,
            copy,
            origin,
            action_label: RETRY_ACTION_LABEL,
            dismiss_label: DISMISS_LABEL,
            claims_tls_bypass: false,
            claims_raw_code: false,
        }
    }
}

/// Every valid §11.8 error family — the single declaration the story
/// renders. The copy strings are the production `copy_for` strings
/// byte-identical at extraction time; the app's own surface tests pin the
/// same copy shape ("securely", "offline", no "bypass"). Origin presence
/// follows the production mapping: TLS/navigation incidents carry the
/// readable origin; offline incidents (commonly reported without a failing
/// URL) omit it. The two trailing families pin the "origin omitted"
/// variants of TLS and navigation so the dimension is explicit.
#[rustfmt::skip]
pub const ERROR_VALID: &[ErrorCase] = &[
    // TLS — the contract maps TLS to Retry only (no bypass).
    ErrorCase::new(
        "TLS — certificate expired",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The site's security certificate has expired, so the connection wasn't made. Contact the site owner or try again later.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "TLS — certificate not yet valid",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The site's security certificate isn't valid yet, so the connection wasn't made. Contact the site owner or try again later.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "TLS — certificate untrusted",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The site's security certificate isn't trusted, so the connection wasn't made. Contact the site owner.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "TLS — certificate revoked",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The site's security certificate has been revoked, so the connection wasn't made. Contact the site owner.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "TLS — hostname mismatch",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The site's security certificate doesn't match this address, so the connection wasn't made. Check the address and try again.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "TLS — handshake failed",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The secure connection to this site failed. Try again in a moment.",
        Some("https://example.com"),
    ),
    // Navigation.
    ErrorCase::new(
        "Navigation — server not found",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "The site's server couldn't be found. Check the address and try again.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "Navigation — connection refused",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "The site refused the connection. It may be temporarily unavailable — try again.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "Navigation — request timed out",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "The site took too long to respond. Try again.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "Navigation — redirect loop",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "The page redirected too many times — a problem with the site. Try again or check the address.",
        Some("https://example.com"),
    ),
    ErrorCase::new(
        "Navigation — unsupported scheme",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "This address uses a scheme this browser can't open.",
        Some("https://example.com"),
    ),
    // Offline — reported without a failing URL, so the origin line is
    // omitted.
    ErrorCase::new(
        "Offline — no network connection",
        ErrorFamily::Offline,
        "You're offline",
        "There's no network connection right now. Check your connection and try again.",
        None,
    ),
    ErrorCase::new(
        "Offline — proxy unreachable",
        ErrorFamily::Offline,
        "You're offline",
        "There's no network connection right now. Check your connection and try again.",
        None,
    ),
    ErrorCase::new(
        "Offline — network unreachable",
        ErrorFamily::Offline,
        "You're offline",
        "The network can't reach this site right now. Check your connection and try again.",
        None,
    ),
    // The explicit "origin omitted" dimension variants for TLS and
    // navigation (the same copy, without the URL line).
    ErrorCase::new(
        "TLS — handshake failed (origin omitted)",
        ErrorFamily::Tls,
        "This page can't be loaded securely",
        "The secure connection to this site failed. Try again in a moment.",
        None,
    ),
    ErrorCase::new(
        "Navigation — refused (origin omitted)",
        ErrorFamily::Navigation,
        "This page can't be loaded",
        "The site refused the connection. It may be temporarily unavailable — try again.",
        None,
    ),
];

/// A rejected combination with the exact DESIGN/contract reason (the
/// matrix "rejected" column). Same `ErrorCase` shape as the valid rows, so
/// one validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedError {
    pub label: &'static str,
    pub case: ErrorCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column for the error surface. Every entry must
/// fail [`error_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const ERROR_REJECTED: &[RejectedError] = &[
    RejectedError {
        label: "TLS bypass action",
        case: ErrorCase {
            label: "TLS bypass action",
            family: ErrorFamily::Tls,
            title: "This page can't be loaded securely",
            copy: "The site's security certificate isn't trusted, so the connection wasn't made. Contact the site owner.",
            origin: Some("https://example.com"),
            action_label: "Proceed anyway",
            dismiss_label: DISMISS_LABEL,
            claims_tls_bypass: true,
            claims_raw_code: false,
        },
        reason: "the closed contract action set has no bypass variant — the app's ErrorSurfaceSpec/IncidentCategory maps TLS to Retry only (error-surface-contract matrix); a bypass is never a legal TLS action",
    },
    RejectedError {
        label: "raw platform error code in copy",
        case: ErrorCase {
            label: "raw platform error code in copy",
            family: ErrorFamily::Offline,
            title: "You're offline",
            copy: "Network error (NSURLErrorDomain -1009).",
            origin: None,
            action_label: RETRY_ACTION_LABEL,
            dismiss_label: DISMISS_LABEL,
            claims_tls_bypass: false,
            claims_raw_code: true,
        },
        reason: "the surface copy is calm and conventional — raw platform error codes never appear as the primary message",
    },
];

/// The one error-case validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected family.
pub fn error_case_valid(case: &ErrorCase) -> bool {
    // TLS maps to Retry only: a TLS family with any other action label is
    // an impossible combination (the closed action set has no bypass).
    if case.family == ErrorFamily::Tls && case.action_label != RETRY_ACTION_LABEL {
        return false;
    }
    if case.claims_tls_bypass {
        return false;
    }
    if case.claims_raw_code {
        return false;
    }
    true
}

/// Build the production `ErrorSurfaceModel` for a valid matrix family;
/// `None` for any rejected combination. The model carries exactly the
/// declared calm title/copy, the readable origin, and the production
/// action/dismiss labels the surface renders.
pub fn build_error_fixture(case: &ErrorCase) -> Option<ErrorSurfaceModel> {
    if !error_case_valid(case) {
        return None;
    }
    Some(ErrorSurfaceModel {
        title: case.title.into(),
        copy: case.copy.into(),
        origin: case.origin.map(SharedString::from),
        action_label: case.action_label.into(),
        dismiss_label: case.dismiss_label.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every declared valid family builds through the production model and
    /// every declared rejected combination fails fixture construction.
    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in ERROR_VALID {
            assert!(
                error_case_valid(case),
                "valid error family rejected: {}",
                case.label
            );
            let fixture = build_error_fixture(case)
                .unwrap_or_else(|| panic!("valid error family builds: {}", case.label));
            assert_eq!(fixture.title, case.title, "{}", case.label);
            assert_eq!(fixture.copy, case.copy, "{}", case.label);
            assert_eq!(fixture.origin.as_deref(), case.origin, "{}", case.label);
            assert_eq!(fixture.action_label, case.action_label, "{}", case.label);
            assert_eq!(fixture.dismiss_label, case.dismiss_label, "{}", case.label);
            assert!(
                !raw_code_like(fixture.copy.as_ref()),
                "copy is calm: {}",
                case.label
            );
        }
        for rejected in ERROR_REJECTED {
            assert!(
                !error_case_valid(&rejected.case),
                "rejected error accepted: {}",
                rejected.label
            );
            assert!(
                build_error_fixture(&rejected.case).is_none(),
                "rejected error builds a fixture: {}",
                rejected.label
            );
        }
    }

    /// The matrix is the one stable declaration source: the story renders
    /// exactly these families in this order; reordering or removing a
    /// family fails here — the no-drift contract.
    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = ERROR_VALID.iter().map(|c| c.label).collect();
        assert_eq!(labels.len(), 16);
        assert_eq!(
            &labels[..6],
            &[
                "TLS — certificate expired",
                "TLS — certificate not yet valid",
                "TLS — certificate untrusted",
                "TLS — certificate revoked",
                "TLS — hostname mismatch",
                "TLS — handshake failed",
            ]
        );
        assert_eq!(
            &labels[6..11],
            &[
                "Navigation — server not found",
                "Navigation — connection refused",
                "Navigation — request timed out",
                "Navigation — redirect loop",
                "Navigation — unsupported scheme",
            ]
        );
        assert_eq!(
            &labels[11..14],
            &[
                "Offline — no network connection",
                "Offline — proxy unreachable",
                "Offline — network unreachable",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in ERROR_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    /// The matrix covers the required dimensions: all three closed
    /// categories, every production subreason, origin shown vs omitted, and
    /// the Retry-only action policy for TLS.
    #[test]
    fn matrix_covers_the_required_dimensions() {
        let families: BTreeSet<ErrorFamily> = ERROR_VALID.iter().map(|c| c.family).collect();
        assert_eq!(
            families,
            BTreeSet::from([
                ErrorFamily::Tls,
                ErrorFamily::Navigation,
                ErrorFamily::Offline
            ])
        );
        // Origin shown and omitted are both covered, with the explicit
        // dimension variants pinned.
        assert!(
            ERROR_VALID.iter().any(|c| c.origin.is_some()),
            "origin shown is covered"
        );
        assert!(
            ERROR_VALID.iter().any(|c| c.origin.is_none()),
            "origin omitted is covered"
        );
        assert!(
            ERROR_VALID
                .iter()
                .any(|c| c.family == ErrorFamily::Tls && c.origin.is_none()),
            "a TLS family pins origin omitted"
        );
        assert!(
            ERROR_VALID
                .iter()
                .any(|c| c.family == ErrorFamily::Navigation && c.origin.is_none()),
            "a navigation family pins origin omitted"
        );
        // The Retry-only policy: every TLS family renders the production
        // Retry label (never a bypass).
        for case in ERROR_VALID.iter().filter(|c| c.family == ErrorFamily::Tls) {
            assert_eq!(
                case.action_label, RETRY_ACTION_LABEL,
                "TLS family {} carries the Retry action",
                case.label
            );
            assert!(
                !case.copy.to_lowercase().contains("bypass"),
                "TLS copy never suggests a bypass: {}",
                case.label
            );
        }
    }

    /// The rejected rules cover the full matrix rejected column.
    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = ERROR_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec!["TLS bypass action", "raw platform error code in copy"]
        );
        for rejected in ERROR_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }

    /// The copy never carries a raw platform-code shape (a plain pattern
    /// check mirroring the surface's calm-copy contract).
    #[test]
    fn no_valid_copy_contains_a_raw_platform_code() {
        for case in ERROR_VALID {
            assert!(
                !raw_code_like(case.copy) && !raw_code_like(case.title),
                "{} renders calm copy, no raw platform code",
                case.label
            );
        }
    }

    fn raw_code_like(text: &str) -> bool {
        text.contains("NSURLError")
            || text.contains("WKError")
            || text.contains("CFNetwork")
            || text.contains("error -")
            || text.contains("Code ")
    }
}
