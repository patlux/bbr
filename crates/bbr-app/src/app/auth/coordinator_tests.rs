//! Unit tests for the app-side authentication coordinator (queue item 10).
//!
//! Split from `coordinator.rs` (which owns the core) so both files stay
//! under the ~500-line budget. The tests prove the identity invariant (a
//! challenge belongs to its originating web view), supersession can never
//! become credential use, cancel fails the request and never retries, the
//! redaction invariant (credentials appear in no Debug/wire shape), and
//! the deterministic failure of unpresentable challenge kinds. As a child
//! module of `coordinator` they access the coordinator's private records,
//! state, and store accessors.
use super::*;
use bbr_web::auth_challenges::AuthScheme;

fn record_basic(coordinator: &mut AuthCoordinator, view: u64, host: &str) -> ChallengeId {
    coordinator
        .record_challenge(
            bbr_web::ProfileId(1),
            bbr_web::WebViewId(view),
            basic_space(host, Some("Finance VPN")),
            ChallengeKind::HttpAuth(AuthScheme::Basic),
            true,
        )
        .expect("challenge record")
}

#[test]
fn sheet_answers_only_the_presented_challenge_of_its_own_view() {
    let mut coordinator = AuthCoordinator::new();
    let first = record_basic(&mut coordinator, 1, "intranet.example");
    let second = record_basic(&mut coordinator, 2, "proxy.example");
    coordinator.present_next().expect("present first");
    assert_eq!(coordinator.presented(), Some(first));

    // The response is keyed by the presented challenge id; the record
    // for that id carries the originating web view. The second
    // challenge (another tab) is still Pending — the contract rejects
    // any response to it (`NotRequested`), so it can never be answered
    // on behalf of its tab.
    let second_state = coordinator.state_of(second).cloned().expect("second state");
    assert_eq!(second_state, AuthState::Pending);
    let challenge = coordinator
        .records
        .iter()
        .find(|record| record.challenge.id() == second)
        .map(|record| record.challenge.clone())
        .expect("second challenge");
    let rejected = apply_event(
        &challenge,
        &second_state,
        AuthEvent::Respond {
            id: second,
            response: AuthResponse::UseCredential(CredentialHandle::new(9)),
        },
    );
    assert!(
        matches!(
            rejected,
            Err(bbr_web::auth_challenges::EventRejection::NotRequested { .. })
        ),
        "an awaiting challenge from another tab cannot be answered"
    );

    assert_eq!(coordinator.submit(), ChallengeDecision::Resolved);
    assert_eq!(
        coordinator.presented_web_view(),
        Some(bbr_web::WebViewId(1)),
        "the resolved response belongs to the presented challenge's own view"
    );
    // The other tab's challenge is untouched.
    assert_eq!(
        coordinator.state_of(second),
        Some(&AuthState::Pending),
        "the other tab's challenge stays pending and unanswered"
    );
}

#[test]
fn a_new_challenge_supersedes_the_presented_one_and_can_never_become_credential_use() {
    let mut coordinator = AuthCoordinator::new();
    let first = record_basic(&mut coordinator, 1, "intranet.example");
    let second = record_basic(&mut coordinator, 2, "other.example");
    coordinator.present_next().expect("present first");
    assert_eq!(coordinator.presented(), Some(first));
    coordinator.present_next().expect("present second");
    assert_eq!(coordinator.presented(), Some(second));
    // The first challenge was superseded by the second (terminal).
    assert_eq!(
        coordinator.state_of(first),
        Some(&AuthState::Superseded { by: second }),
        "the replaced challenge records the successor id"
    );
    // Supersession can never become credential use: the superseded
    // challenge is terminal and rejects every further event, including
    // a legal-looking credential response.
    let challenge = coordinator
        .records
        .iter()
        .find(|record| record.challenge.id() == first)
        .map(|record| record.challenge.clone())
        .expect("first challenge");
    let state = coordinator.state_of(first).cloned().expect("first state");
    let late = apply_event(
        &challenge,
        &state,
        AuthEvent::Respond {
            id: first,
            response: AuthResponse::UseCredential(CredentialHandle::new(9)),
        },
    );
    assert!(
        matches!(
            late,
            Err(bbr_web::auth_challenges::EventRejection::AlreadyTerminal { .. })
        ),
        "superseded challenges reject late responses whole"
    );
    assert_eq!(coordinator.submit(), ChallengeDecision::Resolved);
    assert_eq!(
        coordinator.presented_web_view(),
        Some(bbr_web::WebViewId(2))
    );
}

#[test]
fn cancel_fails_the_request_and_never_retries() {
    let mut coordinator = AuthCoordinator::new();
    let id = record_basic(&mut coordinator, 1, "intranet.example");
    coordinator.present_next().expect("present");
    assert_eq!(coordinator.cancel(), ChallengeDecision::Cancelled);
    assert_eq!(
        coordinator.state_of(id),
        Some(&AuthState::Cancelled),
        "cancel moves the challenge to the terminal cancelled state"
    );
    // The glue closes the sheet after the decision (submit/cancel both
    // call `close_sheet`); the terminal challenge can never be
    // re-presented.
    coordinator.close_sheet();
    assert_eq!(coordinator.presented(), None, "no sheet after cancel");
    // No silent retry: no new challenge was created, nothing is awaiting,
    // and the terminal challenge rejects any later credential response.
    let challenge = coordinator
        .records
        .iter()
        .find(|record| record.challenge.id() == id)
        .map(|record| record.challenge.clone())
        .expect("challenge");
    let state = coordinator.state_of(id).cloned().expect("state");
    let late = apply_event(
        &challenge,
        &state,
        AuthEvent::Respond {
            id,
            response: AuthResponse::UseCredential(CredentialHandle::new(9)),
        },
    );
    assert!(
        matches!(
            late,
            Err(bbr_web::auth_challenges::EventRejection::AlreadyTerminal { .. })
        ),
        "cancelled challenges can never become credential use"
    );
    assert!(coordinator.snapshot().awaiting.is_empty());
}

#[test]
fn submit_resolves_with_the_redacted_handle_and_snapshot_never_holds_credentials() {
    let mut coordinator = AuthCoordinator::new();
    let id = record_basic(&mut coordinator, 1, "intranet.example");
    coordinator.present_next().expect("present");
    coordinator.sheet().type_char("a");
    coordinator.sheet().type_char("l");
    coordinator.sheet().type_char("i");
    coordinator.sheet().type_char("c");
    coordinator.sheet().type_char("e");
    coordinator.sheet().focus_next();
    for ch in ["s", "3", "c", "r", "3", "t"] {
        coordinator.sheet().type_char(ch);
    }
    assert_eq!(coordinator.submit(), ChallengeDecision::Resolved);
    assert_eq!(
        coordinator.state_of(id).and_then(|s| match s {
            AuthState::Resolved { response } => Some(response.clone()),
            _ => None,
        }),
        Some(AuthResponse::UseCredential(CredentialHandle::new(1))),
        "the accepted response carries the opaque handle only"
    );
    // The snapshot is redacted by construction: serialize every shape
    // the coordinator exposes and prove the typed values never appear.
    coordinator.close_sheet();
    let snapshot = coordinator.snapshot();
    let debug = format!("{snapshot:?}");
    for secret in ["alice", "s3cr3t", "s3cret"] {
        assert!(
            !debug.contains(secret),
            "snapshot Debug must not contain {secret}: {debug}"
        );
    }
    let surface_debug = format!("{:?}", coordinator);
    assert!(!surface_debug.contains("alice") && !surface_debug.contains("s3cr3t"));
}

#[test]
fn submit_stores_the_typed_credential_only_under_the_opaque_handle_key() {
    let mut coordinator = AuthCoordinator::new();
    record_basic(&mut coordinator, 1, "intranet.example");
    coordinator.present_next().expect("present");
    for ch in ["a", "l", "i", "c", "e"] {
        coordinator.sheet().type_char(ch);
    }
    coordinator.sheet().focus_next();
    for ch in ["s", "3", "c", "r", "3", "t"] {
        coordinator.sheet().type_char(ch);
    }
    assert_eq!(coordinator.submit(), ChallengeDecision::Resolved);
    // The opaque handle the contract response carries is the only key;
    // the raw values exist nowhere else (snapshot Debug already proven
    // redacted). The future adapter resolves the handle here.
    assert_eq!(coordinator.stored(1), Some(("alice", "s3cr3t")));
    assert_eq!(coordinator.stored(2), None);
    // After close the sheet values are gone; the store keeps the
    // handle-keyed credential until the adapter consumes it.
    coordinator.close_sheet();
    assert_eq!(coordinator.stored(1), Some(("alice", "s3cr3t")));
}

#[test]
fn the_wire_snapshot_proves_the_last_decision_redacted() {
    let mut coordinator = AuthCoordinator::new();
    record_basic(&mut coordinator, 1, "intranet.example");
    coordinator.present_next().expect("present");
    // Fresh presentation: no decision evidence yet.
    assert_eq!(coordinator.snapshot().outcome, None);
    coordinator.sheet().type_char("u");
    coordinator.sheet().focus_next();
    coordinator.sheet().type_char("p");
    coordinator.submit();
    coordinator.close_sheet();
    // The redacted handle decision survives the sheet close: the wire
    // snapshot can prove the submit decision without any credential.
    assert_eq!(
        coordinator.snapshot().outcome.as_deref(),
        Some("credential:#1"),
        "the last decision is visible as the contract's redaction"
    );
    // Cancel resets the evidence to the cancelled redaction.
    record_basic(&mut coordinator, 2, "other.example");
    coordinator.present_next().expect("present second");
    assert_eq!(
        coordinator.snapshot().outcome,
        None,
        "fresh sheet clears evidence"
    );
    coordinator.cancel();
    coordinator.close_sheet();
    assert_eq!(coordinator.snapshot().outcome.as_deref(), Some("cancelled"));
}

#[test]
fn submit_and_cancel_are_noops_without_a_presented_sheet() {
    let mut coordinator = AuthCoordinator::new();
    assert_eq!(coordinator.submit(), ChallengeDecision::Noop);
    assert_eq!(coordinator.cancel(), ChallengeDecision::Noop);
    let id = record_basic(&mut coordinator, 1, "intranet.example");
    assert_eq!(coordinator.presented(), None);
    assert_eq!(coordinator.submit(), ChallengeDecision::Noop);
    assert_eq!(coordinator.state_of(id), Some(&AuthState::Pending));
}

#[test]
fn snapshot_reports_authority_metadata_and_masked_edit_state() {
    let mut coordinator = AuthCoordinator::new();
    record_basic(&mut coordinator, 1, "Intranet.EXAMPLE");
    coordinator.present_next().expect("present");
    coordinator.sheet().type_char("b");
    coordinator.sheet().focus_next();
    coordinator.sheet().type_char("x");
    let snap = coordinator.snapshot();
    assert!(snap.open);
    assert_eq!(snap.challenge, Some(1));
    assert_eq!(snap.web_view, Some(bbr_web::WebViewId(1)));
    // Host is normalized by the protection space (lowercased).
    assert_eq!(snap.host, "intranet.example");
    assert_eq!(snap.realm.as_deref(), Some("Finance VPN"));
    assert_eq!(snap.scheme, "Basic");
    assert_eq!(snap.authority, "origin");
    assert_eq!(snap.state, "response_requested");
    assert_eq!(snap.outcome, None);
    assert_eq!(snap.focused_field.as_deref(), Some("password"));
    assert!(snap.username_entered);
    assert!(snap.password_entered);
}

#[test]
fn non_http_challenge_is_failed_deterministically_at_present_time() {
    // The username/password sheet only answers HTTP authentication. A
    // client-certificate challenge (which needs identity selection, a
    // later adapter capability) is failed through the contract when the
    // authority tries to present it — a typed decision, never an opaque
    // escape.
    let mut coordinator = AuthCoordinator::new();
    let https = ProtectionSpace::new(
        bbr_web::auth_challenges::HttpScheme::Https,
        "bank.example",
        0,
        None,
        bbr_web::auth_challenges::AuthAuthority::Origin,
    )
    .expect("valid https space");
    coordinator
        .record_challenge(
            bbr_web::ProfileId(1),
            bbr_web::WebViewId(1),
            https,
            ChallengeKind::ClientCertificate,
            true,
        )
        .expect("client certificate record");
    assert_eq!(
        coordinator.present_next(),
        None,
        "no sheet for non-HTTP challenges"
    );
    let id = ChallengeId(1);
    let state = coordinator.state_of(id).cloned().expect("state");
    assert!(
        matches!(
            state,
            AuthState::Failed {
                failure: AuthFailure::EngineRejected
            }
        ),
        "unpresentable challenges fail deterministically"
    );
}
