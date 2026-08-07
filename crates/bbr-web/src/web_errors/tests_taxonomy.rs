//! Taxonomy contract tests: incident identity, closed category/subreason
//! sets, definition validation (including contradictory pairs), severity and
//! retryability derivation, privacy-safe context, display/debug behavior,
//! and (behind the `serde` feature) serializability of every wire type.

use crate::web_errors::{
    validate_definition, CrashReason, DefinitionError, Incident, IncidentCategory, IncidentContext,
    IncidentId, IncidentSubReason, NavigationReason, OfflineReason, Retryability, SeverityLevel,
    TlsReason,
};
use crate::{ProfileId, WebViewId};

/// Every closed subreason, flattened for exhaustive matrix/derivation tests.
fn all_subreasons() -> Vec<IncidentSubReason> {
    let mut all = Vec::new();
    for reason in TlsReason::ALL {
        all.push(IncidentSubReason::Tls(reason));
    }
    for reason in NavigationReason::ALL {
        all.push(IncidentSubReason::Navigation(reason));
    }
    for reason in OfflineReason::ALL {
        all.push(IncidentSubReason::Offline(reason));
    }
    for reason in CrashReason::ALL {
        all.push(IncidentSubReason::Crash(reason));
    }
    all
}

fn incident(sequence: u64, category: IncidentCategory, reason: IncidentSubReason) -> Incident {
    Incident::new(
        IncidentId::new(ProfileId(1), WebViewId(2), sequence),
        category,
        reason,
        IncidentContext::from_url("https://example.com/a?q=1"),
    )
    .expect("consistent definition")
}

/// The closed category set is stable: exactly the four contract categories,
/// each with a distinct diagnostic string.
#[test]
fn categories_are_closed() {
    assert_eq!(IncidentCategory::ALL.len(), 4);
    let displays: Vec<String> = IncidentCategory::ALL
        .iter()
        .map(ToString::to_string)
        .collect();
    let unique: std::collections::HashSet<_> = displays.iter().collect();
    assert_eq!(unique.len(), 4, "category display strings must be unique");
}

/// The subreason sets are closed and total per category: every subreason
/// derives exactly the category that owns it, and the union covers each
/// category at least once.
#[test]
fn subreason_sets_are_closed_and_categorized() {
    assert_eq!(TlsReason::ALL.len(), 6);
    assert_eq!(NavigationReason::ALL.len(), 5);
    assert_eq!(OfflineReason::ALL.len(), 3);
    assert_eq!(CrashReason::ALL.len(), 2);

    let mut per_category: std::collections::BTreeMap<IncidentCategory, usize> =
        std::collections::BTreeMap::new();
    for reason in all_subreasons() {
        *per_category.entry(reason.category()).or_default() += 1;
    }
    assert_eq!(
        per_category,
        std::collections::BTreeMap::from([
            (IncidentCategory::Tls, 6),
            (IncidentCategory::Navigation, 5),
            (IncidentCategory::Offline, 3),
            (IncidentCategory::ProcessCrash, 2),
        ])
    );
}

/// Incident identity is stable, ordered, and distinguishes every axis:
/// profile, web view, and per-web-view sequence.
#[test]
fn incident_identity_is_ordered_and_stable() {
    use std::collections::{BTreeMap, HashSet};

    let a = IncidentId::new(ProfileId(1), WebViewId(2), 3);
    assert_eq!(a, IncidentId::new(ProfileId(1), WebViewId(2), 3));
    assert_ne!(a, IncidentId::new(ProfileId(1), WebViewId(2), 4));
    assert_ne!(a, IncidentId::new(ProfileId(1), WebViewId(9), 3));
    assert_ne!(a, IncidentId::new(ProfileId(9), WebViewId(2), 3));

    // Ordered collections key exactly like ProfileId/WebViewId (perf harness).
    let mut latest: BTreeMap<IncidentId, u64> = BTreeMap::new();
    latest.insert(IncidentId::new(ProfileId(1), WebViewId(2), 1), 1);
    latest.insert(IncidentId::new(ProfileId(1), WebViewId(2), 2), 2);
    assert_eq!(latest.iter().next_back().unwrap().1, &2);

    let mut seen = HashSet::new();
    seen.insert(a);
    assert!(seen.contains(&a));
}

/// Every subreason is consistent with its own derived category: the pure
/// definition validation accepts the true pairing.
#[test]
fn definition_matches_category_is_accepted() {
    for reason in all_subreasons() {
        validate_definition(reason.category(), &reason)
            .unwrap_or_else(|err| panic!("{reason:?} must match its category, got {err}"));
    }
}

/// Contradictory definitions are rejected: for every (category, subreason)
/// pair, validation succeeds exactly when the category is the one the
/// subreason belongs to.
#[test]
fn contradictory_definitions_are_rejected() {
    let mut accepted = 0usize;
    for category in IncidentCategory::ALL {
        for reason in all_subreasons() {
            let expected_ok = category == reason.category();
            match validate_definition(category, &reason) {
                Ok(()) => {
                    assert!(expected_ok, "{category} must reject {reason:?}");
                    accepted += 1;
                }
                Err(DefinitionError::CategoryMismatch {
                    category: got_category,
                    reason: got_reason,
                }) => {
                    assert!(!expected_ok, "{category} must accept {reason:?}");
                    assert_eq!(got_category, category);
                    assert_eq!(got_reason, reason);
                }
            }
        }
    }
    assert_eq!(accepted, all_subreasons().len());
}

/// `Incident::new` enforces the same rule as the pure validation: a
/// category/reason contradiction cannot produce an incident.
#[test]
fn incident_new_rejects_contradictory_definitions() {
    let reason = IncidentSubReason::Tls(TlsReason::CertificateRevoked);
    let err = Incident::new(
        IncidentId::new(ProfileId(1), WebViewId(2), 1),
        IncidentCategory::Offline,
        reason,
        IncidentContext::new(None),
    )
    .expect_err("must be rejected");
    assert_eq!(
        err,
        DefinitionError::CategoryMismatch {
            category: IncidentCategory::Offline,
            reason
        }
    );
    assert_eq!(
        err.to_string(),
        "category offline does not match reason tls: certificate revoked"
    );
}

/// Severity is derived per subreason and is stable (the exact table).
#[test]
fn severity_derivation_is_stable() {
    use IncidentSubReason::*;
    use SeverityLevel::*;
    let table = [
        (Tls(TlsReason::CertificateExpired), High),
        (Tls(TlsReason::CertificateNotYetValid), Medium),
        (Tls(TlsReason::CertificateUntrusted), High),
        (Tls(TlsReason::CertificateRevoked), Critical),
        (Tls(TlsReason::HostnameMismatch), Critical),
        (Tls(TlsReason::HandshakeFailed), High),
        (Navigation(NavigationReason::UnsupportedScheme), Low),
        (Navigation(NavigationReason::ServerNotFound), Medium),
        (Navigation(NavigationReason::ConnectionRefused), Medium),
        (Navigation(NavigationReason::RequestTimedOut), Medium),
        (Navigation(NavigationReason::RedirectLoop), High),
        (Offline(OfflineReason::NoNetworkConnection), Medium),
        (Offline(OfflineReason::ProxyUnreachable), Medium),
        (Offline(OfflineReason::NetworkUnreachable), High),
        (Crash(CrashReason::WebContentCrashed), High),
        (Crash(CrashReason::ProcessTerminatedBySystem), High),
    ];
    assert_eq!(table.len(), all_subreasons().len());
    for (reason, expected) in table {
        assert_eq!(reason.severity(), expected, "{reason:?}");
        assert_eq!(incident(1, reason.category(), reason).severity(), expected);
    }
    // Severity is ordered low → critical.
    assert!(Low < Medium && Medium < High && High < Critical);
}

/// Retryability is derived per subreason and is stable (the exact table).
/// Critically, certificate identity failures are `Never` retryable — a retry
/// cannot succeed without a bypass, and bypass is not part of the contract.
#[test]
fn retryability_derivation_is_stable() {
    use IncidentSubReason::*;
    use Retryability::*;
    let table = [
        (Tls(TlsReason::CertificateExpired), Deferred),
        (Tls(TlsReason::CertificateNotYetValid), Deferred),
        (Tls(TlsReason::CertificateUntrusted), Never),
        (Tls(TlsReason::CertificateRevoked), Never),
        (Tls(TlsReason::HostnameMismatch), Never),
        (Tls(TlsReason::HandshakeFailed), Safe),
        (Navigation(NavigationReason::UnsupportedScheme), Never),
        (Navigation(NavigationReason::ServerNotFound), Safe),
        (Navigation(NavigationReason::ConnectionRefused), Deferred),
        (Navigation(NavigationReason::RequestTimedOut), Safe),
        (Navigation(NavigationReason::RedirectLoop), Never),
        (Offline(OfflineReason::NoNetworkConnection), Deferred),
        (Offline(OfflineReason::ProxyUnreachable), Deferred),
        (Offline(OfflineReason::NetworkUnreachable), Deferred),
        (Crash(CrashReason::WebContentCrashed), Never),
        (Crash(CrashReason::ProcessTerminatedBySystem), Never),
    ];
    assert_eq!(table.len(), all_subreasons().len());
    for (reason, expected) in table {
        assert_eq!(reason.retryability(), expected, "{reason:?}");
        assert_eq!(
            incident(1, reason.category(), reason).retryability(),
            expected
        );
    }
}

/// The context derivation is privacy-safe by construction: path, query,
/// fragment, URL userinfo, and the payload of authority-less scheme URLs
/// never survive, and the origin is lower-cased.
#[test]
fn context_from_url_is_privacy_safe() {
    let cases = [
        (
            "https://example.com/a/b?q=1#frag",
            Some("https://example.com"),
        ),
        (
            "https://user:secret@example.com:8443/path",
            Some("https://example.com:8443"),
        ),
        ("HTTP://EXAMPLE.com/Path", Some("http://example.com")),
        ("https://[::1]:8080/x", Some("https://[::1]:8080")),
        ("about:blank", Some("about:blank")),
        ("ABOUT:BLANK", Some("about:blank")),
        ("javascript:alert('secret')", Some("javascript:")),
        ("JAVASCRIPT:alert('secret')", Some("javascript:")),
        ("mailto:user@example.com", Some("mailto:")),
        ("data:text/html,<script>alert(1)</script>", Some("data:")),
        ("tel:+1-555-0100", Some("tel:")),
        ("about:config", Some("about:")),
        ("example.com/path", Some("example.com")),
        ("   ", None),
        ("", None),
    ];
    for (url, expected) in cases {
        let ctx = IncidentContext::from_url(url);
        assert_eq!(ctx.origin(), expected, "from_url({url:?})");
    }
}

/// Direct construction sanitizes input the same way: trimmed, empty → `None`.
#[test]
fn context_new_sanitizes_origin() {
    assert_eq!(IncidentContext::new(None).origin(), None);
    assert_eq!(IncidentContext::new(Some("   ")).origin(), None);
    assert_eq!(
        IncidentContext::new(Some("  https://example.com  ")).origin(),
        Some("https://example.com")
    );
    assert_eq!(IncidentContext::default().origin(), None);
}

/// Display and debug behavior are stable diagnostic strings — technical
/// labels, not UI copy.
#[test]
fn display_and_debug_behavior_is_stable() {
    use crate::web_errors::RecoveryAction;

    assert_eq!(IncidentCategory::Tls.to_string(), "tls");
    assert_eq!(IncidentCategory::Navigation.to_string(), "navigation");
    assert_eq!(IncidentCategory::Offline.to_string(), "offline");
    assert_eq!(IncidentCategory::ProcessCrash.to_string(), "process crash");

    assert_eq!(
        TlsReason::CertificateRevoked.to_string(),
        "certificate revoked"
    );
    assert_eq!(
        NavigationReason::ServerNotFound.to_string(),
        "server not found"
    );
    assert_eq!(
        OfflineReason::ProxyUnreachable.to_string(),
        "proxy unreachable"
    );
    assert_eq!(
        CrashReason::WebContentCrashed.to_string(),
        "web content crashed"
    );

    assert_eq!(
        IncidentSubReason::Tls(TlsReason::HostnameMismatch).to_string(),
        "tls: hostname mismatch"
    );
    assert_eq!(SeverityLevel::Critical.to_string(), "critical");
    assert_eq!(Retryability::Never.to_string(), "never");
    assert_eq!(
        RecoveryAction::InspectCertificate.to_string(),
        "inspect certificate"
    );

    let id = IncidentId::new(ProfileId(1), WebViewId(2), 3);
    assert_eq!(id.to_string(), "profile 1 web-view 2 incident 3");

    let inc = incident(
        3,
        IncidentCategory::Tls,
        IncidentSubReason::Tls(TlsReason::CertificateExpired),
    );
    assert_eq!(
        inc.to_string(),
        "profile 1 web-view 2 incident 3: tls: certificate expired (https://example.com)"
    );

    // Debug derives expose the stable variant names (used by logs/snapshots).
    let debug = format!("{inc:?}");
    assert!(debug.contains("IncidentId"));
    assert!(debug.contains("CertificateExpired"));
    assert!(debug.contains("Tls"));
}

/// Debug derive is present and stable on every wire type (the crate denies
/// `missing_debug_implementations`, this guards the value formatting).
#[test]
fn debug_formatting_is_available() {
    assert_eq!(
        format!("{:?}", IncidentCategory::ProcessCrash),
        "ProcessCrash"
    );
    assert_eq!(
        format!(
            "{:?}",
            IncidentSubReason::Crash(CrashReason::WebContentCrashed)
        ),
        "Crash(WebContentCrashed)"
    );
    let _ = format!(
        "{:?}",
        DefinitionError::CategoryMismatch {
            category: IncidentCategory::Tls,
            reason: IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
        }
    );
}

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    /// Every taxonomy wire type derives stable serde shapes behind the
    /// feature, matching the `assert_serde` contract pattern used by the
    /// downloads module.
    #[test]
    fn taxonomy_wire_types_are_serde_serializable() {
        fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
        assert_serde::<IncidentId>();
        assert_serde::<IncidentCategory>();
        assert_serde::<TlsReason>();
        assert_serde::<NavigationReason>();
        assert_serde::<OfflineReason>();
        assert_serde::<CrashReason>();
        assert_serde::<IncidentSubReason>();
        assert_serde::<SeverityLevel>();
        assert_serde::<Retryability>();
        assert_serde::<IncidentContext>();
        assert_serde::<Incident>();
    }

    /// The struct shape backing the derives stays stable: field names and
    /// their meaning are pinned here (a rename or field-type change shows up
    /// as a compile error), complementing the trait-bound check above.
    #[test]
    fn incident_shape_fields_are_stable() {
        let inc = incident(
            7,
            IncidentCategory::Tls,
            IncidentSubReason::Tls(TlsReason::HostnameMismatch),
        );
        // Field order and names are part of the derived serde shape.
        let Incident {
            id,
            category,
            reason,
            context,
        } = &inc;
        assert_eq!(id.sequence, 7);
        assert_eq!(*category, IncidentCategory::Tls);
        assert_eq!(*reason, IncidentSubReason::Tls(TlsReason::HostnameMismatch));
        assert_eq!(context.origin(), Some("https://example.com"));
    }
}
