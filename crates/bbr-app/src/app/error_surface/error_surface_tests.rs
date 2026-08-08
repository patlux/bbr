//! Unit tests for the error-surface presentation state (task packet
//! `error-surface-contract`): kept in their own file so the implementation
//! module stays under the ~500-line budget (same split as the contract
//! crate's `web_errors` test modules).

use super::*;
use bbr_web::web_errors::{CrashReason, NavigationReason, OfflineReason, TlsReason};

fn context(url: &str) -> IncidentContext {
    IncidentContext::from_url(url)
}

#[test]
fn present_allocates_monotonic_ids_per_view() {
    let mut state = ErrorSurfaceState::default();
    let first = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
            context("https://example.com/"),
            Some("https://example.com/".into()),
        )
        .expect("first incident presents");
    assert_eq!(first.incident.id.sequence, 1);
    let second = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
            context("https://example.com/"),
            Some("https://example.com/".into()),
        )
        .expect("superseding incident presents");
    assert_eq!(second.incident.id.sequence, 2);
    assert_eq!(second.state, LifecycleState::Reported);
    // A different view starts its own sequence at 1.
    let other = state
        .present(
            ProfileId(1),
            WebViewId(8),
            IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
            IncidentContext::default(),
            None,
        )
        .expect("second view presents");
    assert_eq!(other.incident.id.sequence, 1);
}

#[test]
fn surface_spec_is_derived_from_the_classified_incident() {
    let mut state = ErrorSurfaceState::default();
    let spec = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Navigation(NavigationReason::ServerNotFound),
            context("https://user:pass@Example.com:8443/path?q=1"),
            Some("https://user:pass@Example.com:8443/path?q=1".into()),
        )
        .expect("presents");
    assert_eq!(spec.incident.category, IncidentCategory::Navigation);
    assert_eq!(
        spec.incident.reason,
        IncidentSubReason::Navigation(NavigationReason::ServerNotFound)
    );
    // Privacy-safe display: reduced origin only, userinfo stripped.
    assert_eq!(spec.origin.as_deref(), Some("https://example.com:8443"));
    assert_eq!(spec.action, RecoveryAction::Retry);
    assert_eq!(spec.action_label, "Try again");
    assert_eq!(spec.state, LifecycleState::Reported);
    // The full URL stays the app-side retry target.
    assert_eq!(
        state.retry_url(WebViewId(7)).as_deref(),
        Some("https://user:pass@Example.com:8443/path?q=1")
    );
}

#[test]
fn tls_and_offline_specs_carry_the_category_tone() {
    let mut state = ErrorSurfaceState::default();
    let tls = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Tls(TlsReason::CertificateUntrusted),
            context("https://example.com/"),
            None,
        )
        .expect("tls presents");
    assert_eq!(tls.incident.category, IncidentCategory::Tls);
    assert!(tls.title.contains("securely"));
    assert!(!tls.copy.to_lowercase().contains("bypass"));
    assert_eq!(tls.action, RecoveryAction::Retry);

    let offline = state
        .present(
            ProfileId(1),
            WebViewId(8),
            IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
            IncidentContext::default(),
            None,
        )
        .expect("offline presents");
    assert_eq!(offline.incident.category, IncidentCategory::Offline);
    assert!(offline.title.contains("offline"));
    assert_eq!(offline.origin, None);
}

#[test]
fn crash_class_is_classified_but_owned_by_the_design_panel() {
    let mut state = ErrorSurfaceState::default();
    let spec = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Crash(CrashReason::WebContentCrashed),
            IncidentContext::default(),
            None,
        )
        .expect("crash presents");
    assert_eq!(spec.incident.category, IncidentCategory::ProcessCrash);
    assert_eq!(spec.action, RecoveryAction::Restore);
    assert!(is_legal_action(
        IncidentCategory::ProcessCrash,
        RecoveryAction::Restore
    ));
    assert!(!is_legal_action(
        IncidentCategory::ProcessCrash,
        RecoveryAction::Retry
    ));
    // The contract never exposes a TLS bypass for any category.
    assert!(!is_legal_action(
        IncidentCategory::Tls,
        RecoveryAction::Restore
    ));
}

#[test]
fn recovery_request_is_validated_by_the_contract() {
    let mut state = ErrorSurfaceState::default();
    state.present(
        ProfileId(1),
        WebViewId(7),
        IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
        context("https://example.com/"),
        None,
    );
    // Retry is legal for navigation; GoBack is legal too, Restore is not.
    assert_eq!(
        state.request_recovery(WebViewId(7), RecoveryAction::Retry),
        Some(RecoveryAction::Retry)
    );
    // A second request on the same incident is rejected (a fresh attempt
    // is a new incident with a higher sequence).
    assert_eq!(
        state.request_recovery(WebViewId(7), RecoveryAction::Reload),
        None
    );
    assert!(matches!(
        state.by_view.get(&WebViewId(7)).map(|l| l.state()),
        Some(LifecycleState::RecoveryRequested { .. })
    ));
}

#[test]
fn committed_navigation_recovers_and_dismiss_clears_identity() {
    let mut state = ErrorSurfaceState::default();
    state.present(
        ProfileId(1),
        WebViewId(7),
        IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
        context("https://example.com/"),
        None,
    );
    state.mark_hidden(WebViewId(7));
    let action = state
        .request_recovery(WebViewId(7), RecoveryAction::Retry)
        .expect("retry accepted");
    assert_eq!(
        state.recovery_in_flight(WebViewId(7)),
        Some(RecoveryAction::Retry)
    );
    assert!(state.recovered(WebViewId(7), action));
    // Recovered clears the surface; the hidden marker returns so the
    // caller can restore the native view (identity untouched).
    assert!(state.spec_for(WebViewId(7)).is_none());
    assert!(state.take_hidden(WebViewId(7)));

    // Dismiss path: present again, dismiss, surface clears.
    state.present(
        ProfileId(1),
        WebViewId(7),
        IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
        context("https://example.com/"),
        None,
    );
    state.mark_hidden(WebViewId(7));
    assert!(state.dismiss(WebViewId(7)));
    assert!(state.spec_for(WebViewId(7)).is_none());
    assert!(state.take_hidden(WebViewId(7)));
}

#[test]
fn unknown_view_recovery_and_dismiss_are_noops() {
    let mut state = ErrorSurfaceState::default();
    assert_eq!(
        state.request_recovery(WebViewId(99), RecoveryAction::Retry),
        None
    );
    assert!(!state.dismiss(WebViewId(99)));
    assert!(!state.recovered(WebViewId(99), RecoveryAction::Retry));
    assert!(!state.clear_view(WebViewId(99)));
}

#[test]
fn forget_view_drops_all_presentation_state() {
    let mut state = ErrorSurfaceState::default();
    state.present(
        ProfileId(1),
        WebViewId(7),
        IncidentSubReason::Navigation(NavigationReason::ConnectionRefused),
        context("https://example.com/"),
        Some("https://example.com/".into()),
    );
    state.mark_hidden(WebViewId(7));
    state.forget_view(WebViewId(7));
    assert!(state.spec_for(WebViewId(7)).is_none());
    assert!(!state.is_hidden(WebViewId(7)));
    assert_eq!(state.retry_url(WebViewId(7)), None);
    // A fresh incident on the same view restarts its sequence at 1
    // (the view id was released with the closed tab).
    let fresh = state
        .present(
            ProfileId(1),
            WebViewId(7),
            IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
            IncidentContext::default(),
            None,
        )
        .expect("fresh incident presents");
    assert_eq!(fresh.incident.id.sequence, 1);
}
