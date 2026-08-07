//! Focused lifecycle behavior tests: the request/response cycle, cancel and
//! supersede termination, the never-credential-use invariants, and the
//! mismatched/early/duplicate/illegal/stale callback rejections with exact
//! payloads.

use super::support::*;
use crate::auth_challenges::{
    apply_event, AuthEvent, AuthFailure, AuthResponse, AuthState, ChallengeKind, EventRejection,
    ResponseRejection,
};

/// Request → response cycle: presenting a pending challenge is legal exactly
/// once, and a legal credential response resolves the challenge while keeping
/// the exact response readable by the adapter.
#[test]
fn request_and_response_cycle_resolves_with_the_exact_response() {
    let challenge = basic_challenge();
    let requested = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect("presenting a pending challenge is legal");
    assert_eq!(requested, AuthState::ResponseRequested);
    assert!(!requested.is_terminal());

    let resolved = apply_event(
        &challenge,
        &requested,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseCredential(handle(42)),
        },
    )
    .expect("a legal credential response resolves the challenge");
    assert_eq!(
        resolved,
        AuthState::Resolved {
            response: AuthResponse::UseCredential(handle(42))
        }
    );
    assert_eq!(
        resolved.resolved_response(),
        Some(&AuthResponse::UseCredential(handle(42)))
    );
    assert!(resolved.is_terminal());
}

/// Cancellation terminates from any pre-terminal state: the authority's
/// `Cancel` response on a requested challenge, and the engine's `Cancel`
/// event on a pending challenge, both land in `Cancelled`.
#[test]
fn cancel_response_and_cancel_event_both_terminate() {
    let challenge = basic_challenge();

    // Authority responds Cancel on a requested challenge.
    let requested = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect("request");
    let cancelled = apply_event(
        &challenge,
        &requested,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::Cancel,
        },
    )
    .expect("cancel response is legal");
    assert_eq!(cancelled, AuthState::Cancelled);

    // Engine cancellation before presentation.
    let cancelled_early = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::Cancel { id: id(1) },
    )
    .expect("engine cancel on pending is legal");
    assert_eq!(cancelled_early, AuthState::Cancelled);
}

/// Supersession records the successor id and becomes terminal; a later
/// response — even a legal-looking credential — can never revive the
/// superseded challenge or become credential use.
#[test]
fn supersede_records_successor_and_blocks_later_use() {
    let challenge = basic_challenge();
    let requested = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect("request");
    let superseded = apply_event(
        &challenge,
        &requested,
        AuthEvent::Supersede {
            id: id(1),
            by: id(2),
        },
    )
    .expect("supersession is legal while awaiting a response");
    assert_eq!(superseded, AuthState::Superseded { by: id(2) });
    assert!(superseded.is_terminal());

    let err = apply_event(
        &challenge,
        &superseded,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseCredential(handle(42)),
        },
    )
    .expect_err("a response after supersession must reject whole");
    assert_eq!(
        err,
        EventRejection::AlreadyTerminal {
            state: superseded.clone()
        }
    );
}

/// A challenge cannot supersede itself: `Supersede { by: <own id> }` is
/// rejected from `Pending` and `ResponseRequested` and never produces the
/// terminal `Superseded` state, while a genuinely newer successor id stays
/// legal from both states.
#[test]
fn self_supersede_is_rejected() {
    let challenge = basic_challenge();
    for state in [AuthState::Pending, AuthState::ResponseRequested] {
        let err = apply_event(
            &challenge,
            &state,
            AuthEvent::Supersede {
                id: id(1),
                by: id(1),
            },
        )
        .expect_err("a challenge cannot supersede itself");
        assert_eq!(err, EventRejection::SelfSupersede { id: id(1) });
    }

    // A distinct successor id remains legal and stays terminal.
    for state in [AuthState::Pending, AuthState::ResponseRequested] {
        let superseded = apply_event(
            &challenge,
            &state,
            AuthEvent::Supersede {
                id: id(1),
                by: id(2),
            },
        )
        .expect("a distinct successor id is legal");
        assert_eq!(superseded, AuthState::Superseded { by: id(2) });
        assert!(superseded.is_terminal());
    }
}

/// Cancellation can never become credential use: every later event on a
/// cancelled challenge — including a legal-looking credential response — is
/// rejected whole and the state stays `Cancelled`.
#[test]
fn cancellation_never_becomes_credential_use() {
    let challenge = basic_challenge();
    let cancelled = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::Cancel { id: id(1) },
    )
    .expect("cancel");
    for event in [
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseCredential(handle(42)),
        },
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseClientIdentity(identity(9)),
        },
        AuthEvent::RequestResponse { id: id(1) },
        AuthEvent::Supersede {
            id: id(1),
            by: id(2),
        },
    ] {
        let err = apply_event(&challenge, &cancelled, event)
            .expect_err("a cancelled challenge accepts no further event");
        assert_eq!(
            err,
            EventRejection::AlreadyTerminal {
                state: cancelled.clone()
            }
        );
    }
}

/// Mismatched event ids reject whole and leave the state unchanged.
#[test]
fn mismatched_ids_reject_whole() {
    let challenge = basic_challenge();
    let err = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(99) },
    )
    .expect_err("an event for another challenge id must reject");
    assert_eq!(
        err,
        EventRejection::MismatchedId {
            expected: id(1),
            event: id(99)
        }
    );

    // Even a mismatched Cancel for the right challenge cannot terminate it.
    let err = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::Cancel { id: id(99) },
    )
    .expect_err("mismatched cancel must reject");
    assert_eq!(
        err,
        EventRejection::MismatchedId {
            expected: id(1),
            event: id(99)
        }
    );
}

/// A response before the challenge was presented is an early/mismatched
/// callback, never an implicit decision.
#[test]
fn early_response_before_request_rejects_whole() {
    let challenge = basic_challenge();
    let err = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::Continue,
        },
    )
    .expect_err("a response before RequestResponse must reject");
    assert_eq!(
        err,
        EventRejection::NotRequested {
            response: AuthResponse::Continue
        }
    );
}

/// Duplicate callbacks reject: a second request on an already-requested
/// challenge, and a second response on a resolved challenge, are both
/// rejected whole.
#[test]
fn duplicate_callbacks_reject_whole() {
    let challenge = basic_challenge();
    let requested = apply_event(
        &challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect("request");
    let err = apply_event(
        &challenge,
        &requested,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect_err("a second request is a duplicate callback");
    assert_eq!(err, EventRejection::AlreadyRequested);

    let resolved = apply_event(
        &challenge,
        &requested,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::Continue,
        },
    )
    .expect("legal response");
    let err = apply_event(
        &challenge,
        &resolved,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseCredential(handle(42)),
        },
    )
    .expect_err("a second response is a duplicate callback");
    assert_eq!(
        err,
        EventRejection::AlreadyTerminal {
            state: resolved.clone()
        }
    );
}

/// An illegal response (here: a credential on a client-certificate
/// challenge) is rejected whole with the underlying response rejection, and
/// the state stays `ResponseRequested`.
#[test]
fn illegal_response_is_rejected_and_state_unchanged() {
    let cert_challenge = challenge(ChallengeKind::ClientCertificate);
    let requested = apply_event(
        &cert_challenge,
        &AuthState::Pending,
        AuthEvent::RequestResponse { id: id(1) },
    )
    .expect("request");
    let err = apply_event(
        &cert_challenge,
        &requested,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseCredential(handle(42)),
        },
    )
    .expect_err("a credential cannot answer a client-certificate challenge");
    match err {
        EventRejection::IllegalResponse { rejection } => assert_eq!(
            rejection,
            ResponseRejection::CrossKindCredential {
                kind: ChallengeKind::ClientCertificate
            }
        ),
        other => panic!("expected IllegalResponse, got {other:?}"),
    }

    // The state is unchanged and a legal identity response still resolves.
    let resolved = apply_event(
        &cert_challenge,
        &requested,
        AuthEvent::Respond {
            id: id(1),
            response: AuthResponse::UseClientIdentity(identity(9)),
        },
    )
    .expect("a legal identity response resolves the challenge");
    assert!(matches!(resolved, AuthState::Resolved { .. }));
}

/// Native failure terminates from every pre-terminal state and reports the
/// exact closed reason.
#[test]
fn native_failure_terminates_from_every_preterminal_state() {
    let challenge = basic_challenge();
    for (name, state) in states().into_iter().take(2) {
        let failed = apply_event(
            &challenge,
            &state,
            AuthEvent::Failed {
                id: id(1),
                failure: AuthFailure::Connection,
            },
        )
        .expect("native failure is legal while not terminal");
        assert_eq!(
            failed,
            AuthState::Failed {
                failure: AuthFailure::Connection
            },
            "from {name}"
        );
        assert!(failed.is_terminal());
    }
}
