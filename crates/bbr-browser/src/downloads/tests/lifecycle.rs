//! Machine-fidelity contract tests (task packet `downloads-sidebar-panel`):
//! start/validation/actions, event application and terminal accounting, the
//! cancellation race (request/confirmation, first terminal event wins), and
//! retry/restart linkage.

use bbr_web::downloads::{
    CancelRejection, DestinationDecision, DownloadAction, DownloadEvent, DownloadFailure,
    DownloadProgress, DownloadState, EventRejection, RetryRejection, StartRejection, StartRequest,
};

use super::{controller, id, run_attempt, suggested, wired};
use crate::downloads::{ApplyOutcome, DownloadError};

#[test]
fn start_records_the_optimistic_waiting_state_and_emits_start() {
    let (mut controller, actions) = wired();
    let attempt = controller
        .start("https://example.com/f.bin", suggested())
        .unwrap();
    assert_eq!(attempt, id(0));
    assert_eq!(controller.next_id(), 1);
    assert_eq!(
        actions.borrow().as_slice(),
        [DownloadAction::Start(StartRequest {
            id: id(0),
            url: "https://example.com/f.bin".into(),
            destination: suggested(),
            retry_of: None,
        })]
    );
    let record = controller.record(attempt).unwrap();
    assert_eq!(record.filename, "file.bin");
    assert_eq!(record.state, DownloadState::Waiting { destination: None });
    assert_eq!(record.attempt, 1);
    assert_eq!(record.retry_of, None);
}

#[test]
fn start_rejects_malformed_requests_without_recording() {
    let (mut controller, _) = wired();
    assert_eq!(
        controller.start("", suggested()),
        Err(DownloadError::Start(StartRejection::EmptyUrl))
    );
    assert_eq!(
        controller.start(
            "https://example.com/f",
            DestinationDecision::Explicit("".into())
        ),
        Err(DownloadError::Start(
            StartRejection::EmptyExplicitDestination
        ))
    );
    assert!(controller.is_empty());
}

#[test]
fn filename_derivation_covers_all_decisions() {
    let (mut controller, _) = wired();
    let explicit = controller
        .start(
            "https://example.com/f",
            DestinationDecision::Explicit("/tmp/saved.pdf".into()),
        )
        .unwrap();
    assert_eq!(controller.record(explicit).unwrap().filename, "saved.pdf");

    let url_name = controller
        .start(
            "https://example.com/a/b/report.pdf",
            DestinationDecision::Suggested("".into()),
        )
        .unwrap();
    assert_eq!(controller.record(url_name).unwrap().filename, "report.pdf");

    let fallback = controller
        .start(
            "https://example.com/",
            DestinationDecision::Suggested("".into()),
        )
        .unwrap();
    assert_eq!(controller.record(fallback).unwrap().filename, "download");
}

// --- Event application and terminal accounting
// ---------------------------------------------------------------------------

#[test]
fn destination_resolution_records_the_resolved_path() {
    let (mut controller, _) = wired();
    let attempt = controller
        .start("https://example.com/f", suggested())
        .unwrap();
    assert_eq!(
        controller.apply(DownloadEvent::DestinationResolved {
            id: attempt,
            destination: "/Downloads/f".into(),
        }),
        ApplyOutcome::Applied
    );
    assert_eq!(
        controller.record(attempt).unwrap().resolved_path.as_deref(),
        Some("/Downloads/f")
    );
}

#[test]
fn progress_is_monotonic_and_total_stable() {
    let (mut controller, _) = wired();
    let attempt = run_attempt(&mut controller, "f");
    // Regression is rejected; the record keeps the accepted progress.
    assert_eq!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(5, Some(100)).unwrap(),
        }),
        ApplyOutcome::Rejected(EventRejection::ProgressRegressed {
            previous: DownloadProgress::new(10, Some(100)).unwrap(),
            event: DownloadProgress::new(5, Some(100)).unwrap(),
        })
    );
    // A changed total is rejected.
    assert!(matches!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(11, Some(200)).unwrap(),
        }),
        ApplyOutcome::Rejected(EventRejection::TotalChanged { .. })
    ));
}

#[test]
fn zero_byte_download_succeeds_with_full_progress() {
    let (mut controller, clock) = controller();
    let attempt = controller
        .start("https://example.com/empty", suggested())
        .unwrap();
    controller.apply(DownloadEvent::DestinationResolved {
        id: attempt,
        destination: "/Downloads/empty".into(),
    });
    controller.apply(DownloadEvent::Progress {
        id: attempt,
        progress: DownloadProgress::new(0, Some(0)).unwrap(),
    });
    clock.set(2_000);
    assert_eq!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Applied
    );
    let record = controller.record(attempt).unwrap();
    assert_eq!(record.state, DownloadState::Succeeded);
    assert_eq!(record.terminal_at, Some(2_000));
    assert_eq!(record.terminal_seq, Some(1));
}

#[test]
fn success_short_of_a_declared_total_is_rejected() {
    let (mut controller, _) = wired();
    let attempt = run_attempt(&mut controller, "f");
    assert!(matches!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Rejected(EventRejection::SuccessByteMismatch { .. })
    ));
}

#[test]
fn unknown_attempt_events_are_reported_unknown() {
    let (mut controller, _) = wired();
    assert_eq!(
        controller.apply(DownloadEvent::Succeeded { id: id(42) }),
        ApplyOutcome::Unknown(id(42))
    );
}

#[test]
fn terminal_events_are_immutable_and_first_wins() {
    let (mut controller, _) = wired();
    let attempt = run_attempt(&mut controller, "f");
    controller.apply(DownloadEvent::Cancelled { id: attempt });
    // A second terminal event (even success) is rejected: cancel won first.
    assert!(matches!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Rejected(EventRejection::AlreadyTerminal { .. })
    ));
}

// --- Cancellation race: request/confirmation, first terminal event wins
// ---------------------------------------------------------------------------

#[test]
fn cancel_is_request_only_and_confirmed_by_the_engine() {
    let (mut controller, actions) = wired();
    let attempt = run_attempt(&mut controller, "f");
    controller.cancel(attempt).unwrap();
    assert_eq!(
        actions.borrow().last(),
        Some(&DownloadAction::Cancel { id: attempt })
    );
    // No optimistic state change: the row stays active until the terminal
    // event lands.
    assert!(!controller.record(attempt).unwrap().state.is_terminal());
    assert_eq!(
        controller.apply(DownloadEvent::Cancelled { id: attempt }),
        ApplyOutcome::Applied
    );
    assert!(controller.record(attempt).unwrap().state.is_terminal());
}

#[test]
fn success_racing_with_cancel_wins_first() {
    let (mut controller, _) = wired();
    let attempt = run_attempt(&mut controller, "f");
    controller.cancel(attempt).unwrap();
    // The transfer finished before the cancel took effect.
    controller.apply(DownloadEvent::Progress {
        id: attempt,
        progress: DownloadProgress::new(100, Some(100)).unwrap(),
    });
    assert_eq!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Applied
    );
    // The cancel confirmation that lands later is rejected.
    assert!(matches!(
        controller.apply(DownloadEvent::Cancelled { id: attempt }),
        ApplyOutcome::Rejected(EventRejection::AlreadyTerminal { .. })
    ));
}

#[test]
fn cancel_confirmation_racing_with_success_wins_first() {
    let (mut controller, _) = wired();
    let attempt = run_attempt(&mut controller, "f");
    controller.cancel(attempt).unwrap();
    assert_eq!(
        controller.apply(DownloadEvent::Cancelled { id: attempt }),
        ApplyOutcome::Applied
    );
    assert!(matches!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Rejected(EventRejection::AlreadyTerminal { .. })
    ));
}

#[test]
fn cancel_is_rejected_after_terminal_and_for_unknown_ids() {
    let (mut controller, _) = wired();
    assert_eq!(
        controller.cancel(id(7)),
        Err(DownloadError::UnknownAttempt(id(7)))
    );
    let attempt = run_attempt(&mut controller, "f");
    controller.apply(DownloadEvent::Failed {
        id: attempt,
        failure: DownloadFailure::Network,
    });
    assert_eq!(
        controller.cancel(attempt),
        Err(DownloadError::Cancel(CancelRejection::AlreadyTerminal {
            state: DownloadState::Failed {
                failure: DownloadFailure::Network
            }
        }))
    );
}

// --- Retry linkage
// ---------------------------------------------------------------------------

#[test]
fn retry_starts_a_linked_fresh_attempt_only_after_terminal_failure() {
    let (mut controller, actions) = wired();
    let first = run_attempt(&mut controller, "f");
    controller.apply(DownloadEvent::Failed {
        id: first,
        failure: DownloadFailure::Network,
    });

    // Retry is illegal on a live attempt...
    let live = controller
        .start("https://example.com/g", suggested())
        .unwrap();
    assert_eq!(
        controller.retry(live),
        Err(DownloadError::Retry(RetryRejection::NotRetryable {
            state: DownloadState::Waiting { destination: None }
        }))
    );
    // ...and on success.
    let ok = controller
        .start("https://example.com/h", suggested())
        .unwrap();
    controller.apply(DownloadEvent::Succeeded { id: ok });
    assert!(matches!(
        controller.retry(ok),
        Err(DownloadError::Retry(RetryRejection::NotRetryable { .. }))
    ));

    // Failed is retryable: a fresh id linked via retry_of.
    let retried = controller.retry(first).unwrap();
    assert_ne!(retried, first);
    assert_eq!(controller.record(retried).unwrap().retry_of, Some(first));
    assert_eq!(controller.record(retried).unwrap().attempt, 2);
    assert_eq!(controller.record(first).unwrap().retries, [retried]);
    assert_eq!(
        actions.borrow().last(),
        Some(&DownloadAction::Start(StartRequest {
            id: retried,
            url: "f".into(),
            destination: suggested(),
            retry_of: Some(first),
        }))
    );
    assert_eq!(
        controller.record(retried).unwrap().state,
        DownloadState::Waiting { destination: None }
    );
}

#[test]
fn cancelled_attempts_are_restartable_as_fresh_linked_attempts() {
    // The settled post-MVP decision (DESIGN.md §17.8): a cancelled history
    // row offers Restart — a fresh attempt linked via `retry_of`, exactly
    // like a failed retry. The machine accepts `Cancelled` in evaluate_retry;
    // the controller surfaces the same fresh linked attempt.
    let (mut controller, actions) = wired();
    let cancelled = controller
        .start("https://example.com/f", suggested())
        .unwrap();
    controller.apply(DownloadEvent::Cancelled { id: cancelled });

    let restarted = controller.retry(cancelled).unwrap();
    assert_ne!(restarted, cancelled, "a fresh attempt id, never reused");
    assert_eq!(
        controller.record(restarted).unwrap().retry_of,
        Some(cancelled)
    );
    assert_eq!(controller.record(restarted).unwrap().attempt, 2);
    assert_eq!(controller.record(cancelled).unwrap().retries, [restarted]);
    assert_eq!(
        actions.borrow().last(),
        Some(&DownloadAction::Start(StartRequest {
            id: restarted,
            url: "https://example.com/f".into(),
            destination: suggested(),
            retry_of: Some(cancelled),
        }))
    );
    assert_eq!(
        controller.record(restarted).unwrap().state,
        DownloadState::Waiting { destination: None }
    );
}
