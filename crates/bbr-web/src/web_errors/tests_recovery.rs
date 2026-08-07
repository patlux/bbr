//! Recovery-action contract tests: the category-legal action matrix for
//! TLS/navigation/offline/process crash, rejection of illegal combinations
//! (especially any TLS bypass), exhaustive validate/derive consistency, and
//! (behind the `serde` feature) serializability of the wire types.

use crate::web_errors::{
    legal_actions_for_category, validate_action, ActionRejection, Incident, IncidentCategory,
    IncidentContext, IncidentId, IncidentSubReason, RecoveryAction,
};
use crate::{ProfileId, WebViewId};

use RecoveryAction::{Dismiss, GoBack, InspectCertificate, OpenSettings, Reload, Restore, Retry};

fn incident_for(category: IncidentCategory, reason: IncidentSubReason) -> Incident {
    Incident::new(
        IncidentId::new(ProfileId(1), WebViewId(2), 1),
        category,
        reason,
        IncidentContext::new(None),
    )
    .expect("consistent definition")
}

/// The TLS matrix: retry, go back, inspect certificate, dismiss. Nothing that
/// could bypass or downgrade the certificate check, and no reload/settings/
/// restore.
#[test]
fn matrix_tls_is_closed_and_bypass_free() {
    let legal = legal_actions_for_category(IncidentCategory::Tls);
    assert_eq!(legal, vec![Retry, GoBack, InspectCertificate, Dismiss]);
    assert!(!legal.contains(&Reload));
    assert!(!legal.contains(&OpenSettings));
    assert!(!legal.contains(&Restore));
}

/// The navigation matrix: retry, reload, go back, dismiss.
#[test]
fn matrix_navigation_is_closed() {
    assert_eq!(
        legal_actions_for_category(IncidentCategory::Navigation),
        vec![Retry, Reload, GoBack, Dismiss]
    );
    assert!(!legal_actions_for_category(IncidentCategory::Navigation).contains(&OpenSettings));
}

/// The offline matrix adds open-settings (network settings) to retry, reload,
/// go back, dismiss.
#[test]
fn matrix_offline_is_closed() {
    assert_eq!(
        legal_actions_for_category(IncidentCategory::Offline),
        vec![Retry, Reload, GoBack, OpenSettings, Dismiss]
    );
}

/// A process crash is not an ordinary navigation retry: only restore and
/// dismiss are exposed.
#[test]
fn matrix_process_crash_is_closed() {
    assert_eq!(
        legal_actions_for_category(IncidentCategory::ProcessCrash),
        vec![Restore, Dismiss]
    );
    assert!(!legal_actions_for_category(IncidentCategory::ProcessCrash).contains(&Retry));
    assert!(!legal_actions_for_category(IncidentCategory::ProcessCrash).contains(&Reload));
}

/// TLS bypass is unrepresentable: the closed action set contains no bypass/
/// downgrade action, and inspect-certificate is the only certificate-related
/// action (exposed only for TLS).
#[test]
fn tls_bypass_is_unrepresentable() {
    for action in RecoveryAction::ALL {
        let label = action.to_string();
        assert!(
            !label.contains("bypass") && !label.contains("ignore") && !label.contains("exception"),
            "a bypass-like action must not exist, found {label}"
        );
    }
    for category in IncidentCategory::ALL {
        let legal = legal_actions_for_category(category);
        if category == IncidentCategory::Tls {
            assert!(legal.contains(&InspectCertificate));
        } else {
            assert!(
                !legal.contains(&InspectCertificate),
                "certificate inspection is legal only for TLS, got {category}"
            );
        }
    }
}

/// Exhaustive matrix: for every closed subreason, the derived legal set
/// matches the category matrix, and validate/derive agree on every action.
#[test]
fn exhaustive_matrix_validate_derive_agreement() {
    let mut checked = 0usize;
    for reason in all_subreasons() {
        let category = reason.category();
        let expected = legal_actions_for_category(category);
        assert_eq!(reason.legal_actions(), expected, "{reason:?}");

        for action in RecoveryAction::ALL {
            match validate_action(category, action) {
                Ok(()) => {
                    assert!(
                        expected.contains(&action),
                        "{category} accepted {action} outside its matrix"
                    );
                }
                Err(ActionRejection::NotLegalForCategory {
                    action: got_action,
                    category: got_category,
                }) => {
                    assert!(
                        !expected.contains(&action),
                        "{category} rejected {action} although it is in the matrix"
                    );
                    assert_eq!(got_action, action);
                    assert_eq!(got_category, category);
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, all_subreasons().len() * RecoveryAction::ALL.len());
}

/// Concrete illegal combinations are rejected with the exact offending
/// action and category (security-relevant examples).
#[test]
fn validate_action_rejects_illegal_combinations() {
    use IncidentCategory::{Navigation, Offline, ProcessCrash, Tls};

    let cases = [
        (Tls, Restore),
        (Tls, Reload),
        (Tls, OpenSettings),
        (Navigation, OpenSettings),
        (Navigation, Restore),
        (Navigation, InspectCertificate),
        (Offline, Restore),
        (Offline, InspectCertificate),
        (ProcessCrash, Retry),
        (ProcessCrash, Reload),
        (ProcessCrash, GoBack),
        (ProcessCrash, OpenSettings),
        (ProcessCrash, InspectCertificate),
    ];
    for (category, action) in cases {
        let err = validate_action(category, action).expect_err("must be rejected");
        assert_eq!(
            err,
            ActionRejection::NotLegalForCategory { action, category }
        );
        assert_eq!(
            err.to_string(),
            format!("action {action} is not legal for category {category}")
        );
    }
}

/// An incident exposes exactly the category-legal actions and never more;
/// `can_recover_with` agrees with the pure validation.
#[test]
fn incident_exposes_only_category_legal_actions() {
    for reason in all_subreasons() {
        let inc = incident_for(reason.category(), reason);
        assert_eq!(
            inc.legal_actions(),
            legal_actions_for_category(reason.category())
        );
        for action in RecoveryAction::ALL {
            assert_eq!(
                inc.can_recover_with(action),
                validate_action(reason.category(), action).is_ok(),
                "{reason:?} / {action}"
            );
        }
    }
}

/// The recovery display strings are stable diagnostic labels.
#[test]
fn recovery_display_is_stable() {
    let labels: Vec<String> = RecoveryAction::ALL
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        labels,
        vec![
            "retry",
            "reload",
            "go back",
            "open settings",
            "inspect certificate",
            "restore",
            "dismiss",
        ]
    );
}

fn all_subreasons() -> Vec<IncidentSubReason> {
    use crate::web_errors::{CrashReason, NavigationReason, OfflineReason, TlsReason};
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

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    /// Every recovery wire type derives stable serde shapes behind the
    /// feature. `ActionRejection` is a validation rejection (like the
    /// downloads rejections) and is intentionally not a wire type.
    #[test]
    fn recovery_wire_types_are_serde_serializable() {
        fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
        assert_serde::<RecoveryAction>();
    }
}
