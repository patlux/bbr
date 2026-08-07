//! Lifecycle contract tests: the pure state machine (reported → superseded,
//! recovery requested → recovered/failed, dismissed), the per-web-view
//! tracker's duplicate/stale/mismatched report and callback rejection, and
//! (behind the `serde` feature) serializability of the state wire type.
//! Requestable-action legality and the exhaustive transition table live in
//! the sibling `tests_lifecycle_table` module.

use crate::web_errors::{
    apply_event, Incident, IncidentCategory, IncidentContext, IncidentId, IncidentSubReason,
    Lifecycle, LifecycleError, LifecycleEvent, LifecycleState, NavigationReason, RecoveryAction,
    TlsReason,
};
use crate::{ProfileId, WebViewId};

use RecoveryAction::{GoBack, InspectCertificate, Reload, Restore, Retry};

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

/// A freshly reported incident starts in `Reported` and exposes its id,
/// incident, and no in-flight action.
#[test]
fn report_creates_reported_state() {
    let lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    assert_eq!(lc.id(), IncidentId::new(ProfileId(1), WebViewId(2), 1));
    assert_eq!(
        lc.incident(),
        &incident_for(IncidentSubReason::Navigation(
            NavigationReason::ServerNotFound
        ))
    );
    assert_eq!(lc.state(), LifecycleState::Reported);
    assert_eq!(lc.state().action(), None);
}

/// Retry and reload are valid recovery requests for a navigation incident
/// and complete successfully when the callback matches the request.
#[test]
fn retry_and_reload_recoveries_succeed() {
    for action in [Retry, Reload] {
        let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
            NavigationReason::ServerNotFound,
        )));
        lc.request_recovery(action).expect("legal for navigation");
        assert_eq!(
            lc.state(),
            LifecycleState::RecoveryRequested { action },
            "request {action}"
        );
        assert_eq!(lc.state().action(), Some(action));
        lc.recovered(lc.id(), action)
            .expect("completion matches request");
        assert_eq!(lc.state(), LifecycleState::Recovered, "request {action}");
    }
}

/// A failed recovery lands in `RecoveryFailed`: no further recovery request
/// is accepted on the same incident (fresh attempts are new incidents), and
/// the incident may then be dismissed.
#[test]
fn failed_recovery_is_terminal_until_dismissed() {
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::RequestTimedOut,
    )));
    lc.request_recovery(Retry).unwrap();
    lc.recovery_failed(lc.id(), Retry).unwrap();
    assert_eq!(lc.state(), LifecycleState::RecoveryFailed);
    assert_eq!(lc.state().action(), None);

    assert_eq!(
        lc.request_recovery(Reload),
        Err(LifecycleError::InvalidState {
            state: LifecycleState::RecoveryFailed,
            event: LifecycleEvent::RequestRecovery { action: Reload },
        })
    );
    lc.dismiss()
        .expect("dismiss is legal after a failed recovery");
    assert_eq!(lc.state(), LifecycleState::Dismissed);
}

/// Dismissal is legal from every active state: reported, recovery in
/// flight, and recovery failed.
#[test]
fn dismiss_from_active_states_is_legal() {
    // Reported: fresh report.
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    lc.dismiss().expect("dismiss from reported");
    assert_eq!(lc.state(), LifecycleState::Dismissed);

    // Recovery in flight: report + request.
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    lc.request_recovery(Retry).unwrap();
    lc.dismiss().expect("dismiss from recovery requested");
    assert_eq!(lc.state(), LifecycleState::Dismissed);

    // Recovery failed: report + request + failure.
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    lc.request_recovery(Retry).unwrap();
    lc.recovery_failed(lc.id(), Retry).unwrap();
    lc.dismiss().expect("dismiss from recovery failed");
    assert_eq!(lc.state(), LifecycleState::Dismissed);
}

/// Terminal states (`Recovered`, `Superseded`, `Dismissed`) accept no event
/// at all: every event is rejected with `InvalidState`.
#[test]
fn terminal_states_accept_no_events() {
    let events = [
        LifecycleEvent::Supersede,
        LifecycleEvent::RequestRecovery { action: Retry },
        LifecycleEvent::RecoverySucceeded { action: Retry },
        LifecycleEvent::RecoveryFailed { action: Retry },
        LifecycleEvent::Dismiss,
    ];
    for state in [
        LifecycleState::Recovered,
        LifecycleState::Superseded,
        LifecycleState::Dismissed,
    ] {
        for event in events {
            assert_eq!(
                apply_event(state, event, IncidentCategory::Navigation),
                Err(LifecycleError::InvalidState { state, event }),
                "{state} must reject {event}"
            );
        }
    }
}

/// Supersession is legal from every active state and lands on `Superseded`.
#[test]
fn supersede_from_active_states_marks_superseded() {
    for state in [
        LifecycleState::Reported,
        LifecycleState::RecoveryRequested { action: Retry },
        LifecycleState::RecoveryFailed,
    ] {
        assert_eq!(
            apply_event(
                state,
                LifecycleEvent::Supersede,
                IncidentCategory::Navigation
            ),
            Ok(LifecycleState::Superseded),
            "{state}"
        );
    }
}

/// The tracker advances to a newer incident on the same web view; the old
/// incident's active state is superseded, and the new incident starts
/// reported with the old in-flight recovery gone.
#[test]
fn supersede_via_tracker_replaces_incident() {
    let mut a = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    a.request_recovery(Retry).unwrap();
    assert_eq!(
        a.state(),
        LifecycleState::RecoveryRequested { action: Retry }
    );

    let b = a
        .supersede_with(incident(
            2,
            IncidentCategory::Tls,
            IncidentSubReason::Tls(TlsReason::CertificateExpired),
        ))
        .expect("newer incident supersedes");
    assert_eq!(b.id().sequence, 2);
    assert_eq!(b.state(), LifecycleState::Reported);
    assert_eq!(b.state().action(), None);

    // The old incident's final state is derived by the pure transition.
    assert_eq!(
        apply_event(
            LifecycleState::RecoveryRequested { action: Retry },
            LifecycleEvent::Supersede,
            IncidentCategory::Navigation,
        ),
        Ok(LifecycleState::Superseded)
    );
}

/// Reporting the same incident twice is a duplicate and is rejected before
/// any state change.
#[test]
fn duplicate_report_is_rejected() {
    let a = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    let err = a
        .supersede_with(incident_for(IncidentSubReason::Navigation(
            NavigationReason::ServerNotFound,
        )))
        .expect_err("duplicate id must be rejected");
    assert_eq!(err, LifecycleError::DuplicateIncident { id: a.id() });
    assert_eq!(
        err.to_string(),
        "incident profile 1 web-view 2 incident 1 was already reported"
    );
}

/// An older sequence on the same web view is stale and cannot mutate the
/// tracked incident.
#[test]
fn stale_report_is_rejected() {
    let a = Lifecycle::report(incident(
        3,
        IncidentCategory::Navigation,
        IncidentSubReason::Navigation(NavigationReason::ServerNotFound),
    ));
    let stale = incident(
        2,
        IncidentCategory::Navigation,
        IncidentSubReason::Navigation(NavigationReason::ServerNotFound),
    );
    let err = a
        .supersede_with(stale.clone())
        .expect_err("older sequence must be rejected");
    assert_eq!(
        err,
        LifecycleError::StaleIncident {
            current: a.id(),
            received: stale.id,
        }
    );
}

/// A report for a different profile or web view does not belong to this
/// tracker and is rejected as mismatched.
#[test]
fn wrong_scope_report_is_rejected() {
    let a = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    let foreign = Incident::new(
        IncidentId::new(ProfileId(1), WebViewId(9), 2),
        IncidentCategory::Navigation,
        IncidentSubReason::Navigation(NavigationReason::ServerNotFound),
        IncidentContext::new(None),
    )
    .expect("consistent definition");
    let err = a
        .supersede_with(foreign.clone())
        .expect_err("wrong web view");
    assert_eq!(
        err,
        LifecycleError::MismatchedCallback {
            tracked: a.id(),
            received: foreign.id,
        }
    );
}

/// A completion without an in-flight recovery is a stale completion and is
/// rejected before any state change.
#[test]
fn stale_completion_without_request_is_rejected() {
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    assert_eq!(
        lc.recovered(lc.id(), Retry),
        Err(LifecycleError::InvalidState {
            state: LifecycleState::Reported,
            event: LifecycleEvent::RecoverySucceeded { action: Retry },
        })
    );
    assert_eq!(lc.state(), LifecycleState::Reported);
    assert_eq!(
        lc.recovery_failed(lc.id(), Retry),
        Err(LifecycleError::InvalidState {
            state: LifecycleState::Reported,
            event: LifecycleEvent::RecoveryFailed { action: Retry },
        })
    );
}

/// A completion for the superseded incident arriving at the new tracker is
/// stale and rejected as a mismatched callback.
#[test]
fn stale_completion_after_supersession_is_rejected() {
    let mut a = Lifecycle::report(incident_for(IncidentSubReason::Navigation(
        NavigationReason::ServerNotFound,
    )));
    a.request_recovery(Retry).unwrap();
    let mut b = a
        .supersede_with(incident(
            2,
            IncidentCategory::Tls,
            IncidentSubReason::Tls(TlsReason::CertificateExpired),
        ))
        .unwrap();

    let old_id = IncidentId::new(ProfileId(1), WebViewId(2), 1);
    assert_eq!(
        b.recovered(old_id, Retry),
        Err(LifecycleError::MismatchedCallback {
            tracked: b.id(),
            received: old_id,
        })
    );
    assert_eq!(b.state(), LifecycleState::Reported);
}

/// A completion naming an incident the tracker does not own is mismatched
/// and rejected.
#[test]
fn mismatched_callback_is_rejected() {
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Tls(
        TlsReason::CertificateExpired,
    )));
    lc.request_recovery(Retry).unwrap();
    let stranger = IncidentId::new(ProfileId(1), WebViewId(2), 99);
    assert_eq!(
        lc.recovered(stranger, Retry),
        Err(LifecycleError::MismatchedCallback {
            tracked: lc.id(),
            received: stranger,
        })
    );
    assert_eq!(
        lc.state(),
        LifecycleState::RecoveryRequested { action: Retry }
    );
}

/// A completion carrying an action that was not requested is rejected as a
/// mismatched action, for both success and failure callbacks.
#[test]
fn mismatched_action_is_rejected() {
    let mut lc = Lifecycle::report(incident_for(IncidentSubReason::Tls(
        TlsReason::CertificateExpired,
    )));
    lc.request_recovery(Retry).unwrap();
    assert_eq!(
        lc.recovered(lc.id(), InspectCertificate),
        Err(LifecycleError::MismatchedAction {
            requested: Retry,
            completed: InspectCertificate,
        })
    );
    assert_eq!(
        lc.recovery_failed(lc.id(), GoBack),
        Err(LifecycleError::MismatchedAction {
            requested: Retry,
            completed: GoBack,
        })
    );
    assert_eq!(
        lc.state(),
        LifecycleState::RecoveryRequested { action: Retry }
    );
}

/// Display and error strings are stable diagnostic labels, not UI copy.
#[test]
fn display_strings_are_stable() {
    assert_eq!(LifecycleState::Reported.to_string(), "reported");
    assert_eq!(LifecycleState::Superseded.to_string(), "superseded");
    assert_eq!(
        LifecycleState::RecoveryRequested { action: Retry }.to_string(),
        "recovery requested (retry)"
    );
    assert_eq!(LifecycleState::Recovered.to_string(), "recovered");
    assert_eq!(
        LifecycleState::RecoveryFailed.to_string(),
        "recovery failed"
    );
    assert_eq!(LifecycleState::Dismissed.to_string(), "dismissed");

    assert_eq!(LifecycleEvent::Supersede.to_string(), "supersede");
    assert_eq!(
        LifecycleEvent::RequestRecovery { action: Reload }.to_string(),
        "request recovery (reload)"
    );
    assert_eq!(
        LifecycleEvent::RecoverySucceeded { action: Retry }.to_string(),
        "recovery succeeded (retry)"
    );
    assert_eq!(
        LifecycleEvent::RecoveryFailed { action: Retry }.to_string(),
        "recovery failed (retry)"
    );
    assert_eq!(LifecycleEvent::Dismiss.to_string(), "dismiss");

    assert_eq!(
        LifecycleError::MismatchedAction {
            requested: Retry,
            completed: Reload,
        }
        .to_string(),
        "completed action reload does not match requested action retry"
    );
    assert_eq!(
        LifecycleError::InvalidState {
            state: LifecycleState::Reported,
            event: LifecycleEvent::Dismiss,
        }
        .to_string(),
        "cannot apply dismiss from state reported"
    );
    assert_eq!(
        LifecycleError::IllegalAction {
            action: Restore,
            category: IncidentCategory::Tls,
        }
        .to_string(),
        "action restore is not a legal recovery request for category tls"
    );

    let lc = Lifecycle::report(incident(
        3,
        IncidentCategory::Tls,
        IncidentSubReason::Tls(TlsReason::CertificateExpired),
    ));
    assert_eq!(
        lc.to_string(),
        "profile 1 web-view 2 incident 3: tls: certificate expired (unknown origin) [reported]"
    );
}

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    /// The lifecycle state is a wire type behind the feature (the E2E
    /// protocol serializes snapshots). Events and errors are controller
    /// input/rejections, like the downloads rejections, and are deliberately
    /// not serialized.
    #[test]
    fn lifecycle_state_is_serde_serializable() {
        fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
        assert_serde::<LifecycleState>();
    }
}
