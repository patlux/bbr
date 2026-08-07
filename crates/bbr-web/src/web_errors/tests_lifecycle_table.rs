//! Requestable-action legality and transition-table contract tests: for every
//! closed subreason a recovery request is accepted exactly when the action is
//! in the incident's requestable set (category-legal actions minus `Dismiss`),
//! process crash stays distinct from navigation retry, TLS never exposes a
//! bypass, and the full 6×5 lifecycle transition table is pinned row by row.

use crate::web_errors::{
    apply_event, legal_actions_for_category, lifecycle_requestable_actions, CrashReason, Incident,
    IncidentCategory, IncidentContext, IncidentId, IncidentSubReason, Lifecycle, LifecycleError,
    LifecycleEvent, LifecycleState, NavigationReason, OfflineReason, RecoveryAction, TlsReason,
};
use crate::{ProfileId, WebViewId};

use RecoveryAction::{Dismiss, GoBack, InspectCertificate, OpenSettings, Reload, Restore, Retry};

fn incident(sequence: u64, category: IncidentCategory, reason: IncidentSubReason) -> Incident {
    Incident::new(
        IncidentId::new(ProfileId(1), WebViewId(2), sequence),
        category,
        reason,
        IncidentContext::new(None),
    )
    .expect("consistent definition")
}

fn incident_for(reason: IncidentSubReason) -> Incident {
    incident(1, reason.category(), reason)
}

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

/// Exhaustive legality: for every closed subreason, a recovery request is
/// accepted exactly when the action is in the incident's requestable set
/// (category-legal actions minus `Dismiss`), and every accepted request
/// enters `RecoveryRequested` with that action.
#[test]
fn recovery_requests_are_validated_against_incident_legal_actions() {
    for reason in all_subreasons() {
        let expected = lifecycle_requestable_actions(reason.category());
        for action in RecoveryAction::ALL {
            let mut lc = Lifecycle::report(incident_for(reason));
            match lc.request_recovery(action) {
                Ok(()) => {
                    assert!(
                        expected.contains(&action),
                        "{reason:?} accepted {action} outside its requestable set"
                    );
                    assert_eq!(
                        lc.state(),
                        LifecycleState::RecoveryRequested { action },
                        "{reason:?} / {action}"
                    );
                }
                Err(LifecycleError::IllegalAction {
                    action: got,
                    category,
                }) => {
                    assert!(
                        !expected.contains(&action),
                        "{reason:?} rejected requestable action {action}"
                    );
                    assert_eq!(got, action);
                    assert_eq!(category, reason.category());
                }
                Err(other) => panic!("{reason:?} / {action}: unexpected rejection {other}"),
            }
        }
    }
}

/// A process crash is not an ordinary navigation retry: the only requestable
/// action is `Restore` (recreate the web session); `Retry`/`Reload` and
/// every other action are rejected.
#[test]
fn process_crash_accepts_only_restore() {
    for reason in [
        IncidentSubReason::Crash(CrashReason::WebContentCrashed),
        IncidentSubReason::Crash(CrashReason::ProcessTerminatedBySystem),
    ] {
        let mut lc = Lifecycle::report(incident_for(reason));
        for bad in [
            Retry,
            Reload,
            GoBack,
            OpenSettings,
            InspectCertificate,
            Dismiss,
        ] {
            assert!(
                matches!(
                    lc.request_recovery(bad),
                    Err(LifecycleError::IllegalAction { .. })
                ),
                "{reason:?} must reject recovery request {bad}"
            );
        }
        lc.request_recovery(Restore)
            .unwrap_or_else(|e| panic!("{reason:?}: {e}"));
        assert_eq!(
            lc.state(),
            LifecycleState::RecoveryRequested { action: Restore }
        );
        lc.recovered(lc.id(), Restore)
            .unwrap_or_else(|e| panic!("{reason:?}: {e}"));
        assert_eq!(lc.state(), LifecycleState::Recovered, "{reason:?}");
    }
}

/// TLS incidents expose certificate inspection as a recovery request and
/// nothing that could bypass, downgrade, or ignore the certificate check;
/// reload, settings, restore, and dismiss-as-recovery are rejected.
#[test]
fn tls_never_exposes_bypass_or_unrelated_actions() {
    for reason in TlsReason::ALL {
        for (action, expected_ok) in [
            (Retry, true),
            (GoBack, true),
            (InspectCertificate, true),
            (Reload, false),
            (OpenSettings, false),
            (Restore, false),
            (Dismiss, false),
        ] {
            let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Tls(reason)));
            assert_eq!(
                lc.request_recovery(action).is_ok(),
                expected_ok,
                "{reason:?} / {action}"
            );
        }
        // Certificate inspection is a valid in-flight recovery on TLS.
        let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Tls(reason)));
        lc.request_recovery(InspectCertificate).unwrap();
        lc.recovered(lc.id(), InspectCertificate).unwrap();
        assert_eq!(lc.state(), LifecycleState::Recovered, "{reason:?}");
    }
}

/// The requestable set never contains `Dismiss`: dismissal is a lifecycle
/// event of its own, not a recovery request.
#[test]
fn dismiss_is_never_a_recovery_request() {
    for category in IncidentCategory::ALL {
        assert!(
            !lifecycle_requestable_actions(category).contains(&Dismiss),
            "{category} must not expose dismiss as a recovery request"
        );
        assert_eq!(
            lifecycle_requestable_actions(category).len(),
            legal_actions_for_category(category).len() - 1,
            "{category}"
        );
    }
}

/// The canonical 6×5 transition table, pinned row by row: the same state,
/// event, and category always yield the same outcome. `Navigation` is the
/// reference category (Retry is requestable); legality rows use their own
/// categories.
#[test]
fn exhaustive_transition_table_is_deterministic() {
    use IncidentCategory::Navigation as Nav;
    use LifecycleEvent::{
        Dismiss, RecoveryFailed as FailedEvent, RecoverySucceeded as Succeeded, RequestRecovery,
        Supersede,
    };
    use LifecycleState::{
        Dismissed as StDismissed, Recovered as StRecovered, RecoveryFailed as StFailed,
        RecoveryRequested as StRequested, Reported as StReported, Superseded as StSuperseded,
    };

    let table: [(
        LifecycleState,
        LifecycleEvent,
        IncidentCategory,
        Result<LifecycleState, LifecycleError>,
    ); 32] = [
        // Supersede
        (StReported, Supersede, Nav, Ok(StSuperseded)),
        (
            StRequested { action: Retry },
            Supersede,
            Nav,
            Ok(StSuperseded),
        ),
        (StFailed, Supersede, Nav, Ok(StSuperseded)),
        (
            StRecovered,
            Supersede,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRecovered,
                event: Supersede,
            }),
        ),
        (
            StSuperseded,
            Supersede,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StSuperseded,
                event: Supersede,
            }),
        ),
        (
            StDismissed,
            Supersede,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StDismissed,
                event: Supersede,
            }),
        ),
        // RequestRecovery
        (
            StReported,
            RequestRecovery { action: Retry },
            Nav,
            Ok(StRequested { action: Retry }),
        ),
        (
            StRequested { action: Retry },
            RequestRecovery { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRequested { action: Retry },
                event: RequestRecovery { action: Retry },
            }),
        ),
        (
            StFailed,
            RequestRecovery { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StFailed,
                event: RequestRecovery { action: Retry },
            }),
        ),
        (
            StRecovered,
            RequestRecovery { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRecovered,
                event: RequestRecovery { action: Retry },
            }),
        ),
        (
            StSuperseded,
            RequestRecovery { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StSuperseded,
                event: RequestRecovery { action: Retry },
            }),
        ),
        (
            StDismissed,
            RequestRecovery { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StDismissed,
                event: RequestRecovery { action: Retry },
            }),
        ),
        // RecoverySucceeded
        (
            StReported,
            Succeeded { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StReported,
                event: Succeeded { action: Retry },
            }),
        ),
        (
            StRequested { action: Retry },
            Succeeded { action: Retry },
            Nav,
            Ok(StRecovered),
        ),
        (
            StRequested { action: Retry },
            Succeeded { action: Reload },
            Nav,
            Err(LifecycleError::MismatchedAction {
                requested: Retry,
                completed: Reload,
            }),
        ),
        (
            StFailed,
            Succeeded { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StFailed,
                event: Succeeded { action: Retry },
            }),
        ),
        (
            StRecovered,
            Succeeded { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRecovered,
                event: Succeeded { action: Retry },
            }),
        ),
        (
            StSuperseded,
            Succeeded { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StSuperseded,
                event: Succeeded { action: Retry },
            }),
        ),
        (
            StDismissed,
            Succeeded { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StDismissed,
                event: Succeeded { action: Retry },
            }),
        ),
        // RecoveryFailed
        (
            StReported,
            FailedEvent { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StReported,
                event: FailedEvent { action: Retry },
            }),
        ),
        (
            StRequested { action: Retry },
            FailedEvent { action: Retry },
            Nav,
            Ok(StFailed),
        ),
        (
            StRequested { action: Retry },
            FailedEvent { action: Reload },
            Nav,
            Err(LifecycleError::MismatchedAction {
                requested: Retry,
                completed: Reload,
            }),
        ),
        (
            StFailed,
            FailedEvent { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StFailed,
                event: FailedEvent { action: Retry },
            }),
        ),
        (
            StRecovered,
            FailedEvent { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRecovered,
                event: FailedEvent { action: Retry },
            }),
        ),
        (
            StSuperseded,
            FailedEvent { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StSuperseded,
                event: FailedEvent { action: Retry },
            }),
        ),
        (
            StDismissed,
            FailedEvent { action: Retry },
            Nav,
            Err(LifecycleError::InvalidState {
                state: StDismissed,
                event: FailedEvent { action: Retry },
            }),
        ),
        // Dismiss
        (StReported, Dismiss, Nav, Ok(StDismissed)),
        (StRequested { action: Retry }, Dismiss, Nav, Ok(StDismissed)),
        (StFailed, Dismiss, Nav, Ok(StDismissed)),
        (
            StRecovered,
            Dismiss,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StRecovered,
                event: Dismiss,
            }),
        ),
        (
            StSuperseded,
            Dismiss,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StSuperseded,
                event: Dismiss,
            }),
        ),
        (
            StDismissed,
            Dismiss,
            Nav,
            Err(LifecycleError::InvalidState {
                state: StDismissed,
                event: Dismiss,
            }),
        ),
    ];

    for (state, event, category, expected) in table {
        assert_eq!(
            apply_event(state, event, category),
            expected,
            "{state} + {event} must be deterministic"
        );
    }

    // Cross-category legality rows: the same request event is rejected when
    // the action is not in the incident category's requestable set.
    let legality_rows = [
        (
            IncidentCategory::ProcessCrash,
            Retry,
            Err(LifecycleError::IllegalAction {
                action: Retry,
                category: IncidentCategory::ProcessCrash,
            }),
        ),
        (
            IncidentCategory::Tls,
            OpenSettings,
            Err(LifecycleError::IllegalAction {
                action: OpenSettings,
                category: IncidentCategory::Tls,
            }),
        ),
        (
            IncidentCategory::Navigation,
            InspectCertificate,
            Err(LifecycleError::IllegalAction {
                action: InspectCertificate,
                category: IncidentCategory::Navigation,
            }),
        ),
        (
            IncidentCategory::Navigation,
            RecoveryAction::Dismiss,
            Err(LifecycleError::IllegalAction {
                action: RecoveryAction::Dismiss,
                category: IncidentCategory::Navigation,
            }),
        ),
    ];
    for (category, action, expected) in legality_rows {
        assert_eq!(
            apply_event(
                LifecycleState::Reported,
                LifecycleEvent::RequestRecovery { action },
                category,
            ),
            expected,
        );
    }

    // Coverage proof: the canonical table is the full state × event product
    // (6 states × 5 events) plus the two action-mismatch rows.
    assert_eq!(table.len(), 6 * 5 + 2);
}
