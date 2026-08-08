use super::support::*;
use crate::downloads::*;

/// Cancellation is legal exactly while waiting or running— approval-only,
/// no optimistic transition; terminal attempts are rejected explicitly.
#[test]
fn cancel_is_legal_only_before_terminal() {
    for (name, state) in states() {
        let result = evaluate_cancel(&state);
        match name {
            "Waiting-None" | "Waiting-Some" | "Running" => {
                assert_eq!(result, Ok(()), "{name}")
            }
            other => assert!(
                matches!(result, Err(CancelRejection::AlreadyTerminal { .. })),
                "{other} must reject cancellation"
            ),
        }
    }
}

/// The engine's `Cancelled` confirmation is accepted from any non-terminal
/// state and rejected once terminal (covered by the matrix, asserted here for
/// the payload shape).
#[test]
fn cancelled_confirmation_lands_terminal() {
    let running = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(10, Some(100)),
    };
    assert_eq!(
        apply_event(id(1), &running, DownloadEvent::Cancelled { id: id(1) })
            .expect("running attempts confirm cancellation"),
        DownloadState::Cancelled
    );
}

/// Retry is legal exactly for `Failed` (the Retry action); Restart is legal
/// exactly for `Cancelled` (the settled post-MVP decision that a cancelled
/// history row may be restarted as a fresh linked attempt). Every other
/// state — live or terminal — is rejected with the offending state
/// reported.
#[test]
fn retry_is_legal_only_for_failed_and_restart_only_for_cancelled() {
    for (name, state) in states() {
        let result = evaluate_retry(id(1), &state, &retry_request());
        match name {
            "Failed" | "Cancelled" => assert_eq!(result, Ok(())),
            other => assert!(
                matches!(result, Err(RetryRejection::NotRetryable { .. })),
                "{other} must reject retry"
            ),
        }
    }
}

/// A cancelled attempt is restartable through the same machine path as a
/// failed retry: a fresh id, the cancelled attempt linked via `retry_of`,
/// and the caller sends the request verbatim.
#[test]
fn restart_accepts_fresh_linked_attempt_from_cancelled() {
    let cancelled = DownloadState::Cancelled;
    let request = retry_request();
    evaluate_retry(id(1), &cancelled, &request).expect("cancelled attempts are restartable");
    assert_eq!(request.id, id(2));
    assert_eq!(request.retry_of, Some(id(1)));
    assert_ne!(request.id, request.retry_of.unwrap());
}

/// Retry rejection variants: a missing link, a reused attempt id, and
/// ill-formed request content each have a precise reason.
#[test]
fn retry_rejection_variants() {
    let failed = DownloadState::Failed { failure: FAILURE };

    let mut missing_link = retry_request();
    missing_link.retry_of = None;
    assert_eq!(
        evaluate_retry(id(1), &failed, &missing_link),
        Err(RetryRejection::MissingRetryLink)
    );

    // A retry link naming a different attempt than the failed one is a
    // mismatched link (link identity is checked before id reuse).
    let mut mismatched_link = retry_request();
    mismatched_link.retry_of = Some(id(2));
    assert_eq!(
        evaluate_retry(id(1), &failed, &mismatched_link),
        Err(RetryRejection::MismatchedRetryLink {
            failed_attempt: id(1),
            request_retry_of: id(2)
        })
    );

    // A genuine id reuse: the link names the failed attempt but the new id
    // equals it; a retry must be a fresh attempt with a distinct id.
    let mut same_attempt = retry_request();
    same_attempt.retry_of = Some(id(1));
    same_attempt.id = id(1);
    assert_eq!(
        evaluate_retry(id(1), &failed, &same_attempt),
        Err(RetryRejection::SameAttempt {
            previous_attempt: id(1),
            new_id: id(1)
        })
    );

    let mut empty_url = retry_request();
    empty_url.url = String::new();
    assert_eq!(
        evaluate_retry(id(1), &failed, &empty_url),
        Err(RetryRejection::EmptyUrl)
    );

    let mut empty_explicit = retry_request();
    empty_explicit.destination = DestinationDecision::Explicit(String::new());
    assert_eq!(
        evaluate_retry(id(1), &failed, &empty_explicit),
        Err(RetryRejection::EmptyExplicitDestination)
    );
}

/// A successful retry keeps the caller's fresh id, names the previous
/// attempt, and stays a well-formed start the caller can send verbatim.
#[test]
fn retry_accepts_fresh_linked_attempt() {
    let failed = DownloadState::Failed { failure: FAILURE };
    let request = retry_request();
    evaluate_retry(id(1), &failed, &request).expect("failed attempts are retryable");
    assert_eq!(request.id, id(2));
    assert_eq!(request.retry_of, Some(id(1)));
    assert_ne!(request.id, request.retry_of.unwrap());
}

/// Accessor truth table: the resolved destination and progress are observable
/// exactly where the machine holds them.
#[test]
fn accessor_truth_table() {
    for (name, state) in states() {
        match name {
            "Waiting-None" => {
                assert_eq!(state.resolved_destination(), None);
                assert_eq!(state.progress(), None);
            }
            "Waiting-Some" => {
                assert_eq!(state.resolved_destination(), Some(DEST));
                assert_eq!(state.progress(), None);
            }
            "Running" => {
                assert_eq!(state.resolved_destination(), Some(DEST));
                assert_eq!(state.progress(), Some(&full_progress()));
            }
            "Succeeded" => {
                assert_eq!(state.resolved_destination(), None);
                assert_eq!(state.progress(), None);
                assert!(state.is_terminal());
                assert!(!state.is_retryable());
            }
            "Cancelled" => {
                assert_eq!(state.resolved_destination(), None);
                assert_eq!(state.progress(), None);
                assert!(state.is_terminal());
                assert!(
                    state.is_retryable(),
                    "cancelled attempts are restartable (settled decision)"
                );
            }
            "Failed" => {
                assert_eq!(state.resolved_destination(), None);
                assert_eq!(state.progress(), None);
                assert!(state.is_terminal());
                assert!(state.is_retryable());
            }
            other => panic!("unexpected state {other}"),
        }
    }
}

/// A full happy-path lifecycle and its failure → retry counterpart, walking
/// the machine exactly as the future controller will: dispatch by id over a
/// per-id state, recording the optimistic initial state after each start.
#[test]
fn full_lifecycle_happy_path_and_failure_retry() {
    // Happy path: resolve, transfer in monotonic steps, succeed.
    let mut state = DownloadState::Waiting { destination: None };
    state = apply_event(
        id(1),
        &state,
        DownloadEvent::DestinationResolved {
            id: id(1),
            destination: DEST.into(),
        },
    )
    .expect("destination decision");
    state = apply_event(
        id(1),
        &state,
        DownloadEvent::Progress {
            id: id(1),
            progress: progress(0, Some(4)),
        },
    )
    .expect("transfer starts");
    for received in 1..=4 {
        state = apply_event(
            id(1),
            &state,
            DownloadEvent::Progress {
                id: id(1),
                progress: progress(received, Some(4)),
            },
        )
        .expect("monotonic progress");
    }
    state = apply_event(id(1), &state, DownloadEvent::Succeeded { id: id(1) }).expect("completion");
    assert_eq!(state, DownloadState::Succeeded);
    assert!(
        evaluate_retry(id(1), &state, &retry_request()).is_err(),
        "no retry after success"
    );

    // Failure path: fail, then retry as a fresh linked attempt that succeeds.
    let failed = DownloadState::Failed { failure: FAILURE };
    let request = retry_request();
    evaluate_retry(id(1), &failed, &request).expect("failed attempts are retryable");
    let mut retried = DownloadState::Waiting { destination: None };
    retried = apply_event(
        id(2),
        &retried,
        DownloadEvent::DestinationResolved {
            id: id(2),
            destination: "/tmp/report (1).pdf".into(),
        },
    )
    .expect("retry destination resolution");
    retried = apply_event(
        id(2),
        &retried,
        DownloadEvent::Progress {
            id: id(2),
            progress: progress(2, None),
        },
    )
    .expect("retry transfer");
    retried = apply_event(id(2), &retried, DownloadEvent::Succeeded { id: id(2) })
        .expect("unknown-length retry succeeds");
    assert_eq!(retried, DownloadState::Succeeded);
}
