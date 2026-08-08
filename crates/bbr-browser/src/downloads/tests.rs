//! Contract tests for the portable download controller (no display server,
//! no filesystem, no wall clock). Every L6 event/rejection path, ordering,
//! cancel races, retry linkage, reveal eligibility, and the action sink.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use bbr_web::downloads::{
    CancelRejection, DestinationDecision, DownloadAction, DownloadEvent, DownloadFailure,
    DownloadId, DownloadProgress, DownloadState, EventRejection, RetryRejection, StartRejection,
    StartRequest,
};

use super::{ApplyOutcome, DownloadController, DownloadError};

fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

/// A controller with a deterministic advancing clock.
fn controller() -> (DownloadController, Rc<Cell<u64>>) {
    let clock = Rc::new(Cell::new(1_000));
    let tick = clock.clone();
    (DownloadController::with_clock(move || tick.get()), clock)
}

/// A controller with a recorded action sink.
fn wired() -> (DownloadController, Rc<RefCell<Vec<DownloadAction>>>) {
    let (mut controller, _) = controller();
    let actions: Rc<RefCell<Vec<DownloadAction>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded = actions.clone();
    controller.set_action_sink(Box::new(move |action| recorded.borrow_mut().push(action)));
    (controller, actions)
}

fn suggested() -> DestinationDecision {
    DestinationDecision::Suggested("file.bin".into())
}

/// Drive a fresh start to `Running { progress }` with a resolved path.
fn run_attempt(controller: &mut DownloadController, url: &str) -> DownloadId {
    let attempt = controller.start(url, suggested()).unwrap();
    assert_eq!(
        controller.apply(DownloadEvent::DestinationResolved {
            id: attempt,
            destination: format!("/Downloads/{url}"),
        }),
        ApplyOutcome::Applied
    );
    assert_eq!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(10, Some(100)).unwrap(),
        }),
        ApplyOutcome::Applied
    );
    attempt
}

/// Drive a fresh start to a fully-received success (legal L6 transition:
/// `Succeeded` requires every declared byte).
fn succeed_attempt(controller: &mut DownloadController, url: &str) -> DownloadId {
    let attempt = run_attempt(controller, url);
    assert_eq!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(100, Some(100)).unwrap(),
        }),
        ApplyOutcome::Applied
    );
    assert_eq!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Applied
    );
    attempt
}

// --- Start / validation / actions
// ---------------------------------------------------------------------------

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
fn retry_starts_a_linked_fresh_attempt_only_after_failure() {
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
    // ...and on success and cancellation.
    let ok = controller
        .start("https://example.com/h", suggested())
        .unwrap();
    controller.apply(DownloadEvent::Succeeded { id: ok });
    assert!(matches!(
        controller.retry(ok),
        Err(DownloadError::Retry(RetryRejection::NotRetryable { .. }))
    ));
    let cancelled = controller
        .start("https://example.com/i", suggested())
        .unwrap();
    controller.apply(DownloadEvent::Cancelled { id: cancelled });
    assert!(matches!(
        controller.retry(cancelled),
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
fn cancelled_attempts_are_never_retried_by_the_ui() {
    // The L6 contract forbids evaluate_retry on Cancelled; the controller
    // surfaces that as NotRetryable, so the panel never offers Retry there.
    let (mut controller, _) = wired();
    let attempt = controller
        .start("https://example.com/f", suggested())
        .unwrap();
    controller.apply(DownloadEvent::Cancelled { id: attempt });
    assert!(matches!(
        controller.retry(attempt),
        Err(DownloadError::Retry(RetryRejection::NotRetryable { .. }))
    ));
}

// --- Deterministic ordering (E2E row index contract)
// ---------------------------------------------------------------------------

#[test]
fn active_rows_are_ascending_by_id() {
    let (mut controller, _) = wired();
    let a = run_attempt(&mut controller, "https://example.com/a");
    let b = succeed_attempt(&mut controller, "https://example.com/b");
    let c = run_attempt(&mut controller, "https://example.com/c");
    assert_eq!(controller.active_ids(), [a, c]);
    assert_eq!(controller.rows(), [a, c, b]);
}

#[test]
fn history_rows_are_most_recent_terminal_first() {
    let (mut controller, clock) = controller();
    let a = run_attempt(&mut controller, "a");
    let b = succeed_attempt(&mut controller, "b");
    let c = run_attempt(&mut controller, "c");

    // Terminal in order b, a, c — history must be c, a, b.
    clock.set(2_000);
    let _ = b;
    clock.set(3_000);
    controller.apply(DownloadEvent::Failed {
        id: a,
        failure: DownloadFailure::Network,
    });
    clock.set(4_000);
    controller.apply(DownloadEvent::Cancelled { id: c });

    assert_eq!(controller.history_ids(), [c, a, b]);
    assert!(controller.active_ids().is_empty());
    assert_eq!(controller.rows(), [c, a, b]);
}

#[test]
fn active_and_history_interleave_in_panel_order() {
    let (mut controller, _) = wired();
    let a = run_attempt(&mut controller, "a");
    let b = succeed_attempt(&mut controller, "b");
    let c = run_attempt(&mut controller, "c");
    assert_eq!(controller.rows(), [a, c, b]);
}

// --- Reveal eligibility and record access
// ---------------------------------------------------------------------------

#[test]
fn reveal_candidate_requires_a_succeeded_record_with_a_path() {
    let (mut controller, _) = wired();
    // A live attempt has no reveal candidate.
    let live = run_attempt(&mut controller, "live");
    assert_eq!(controller.reveal_candidate(live), None);

    // Succeeded with a resolved path is eligible.
    let done = run_attempt(&mut controller, "done");
    controller.apply(DownloadEvent::Progress {
        id: done,
        progress: DownloadProgress::new(100, Some(100)).unwrap(),
    });
    controller.apply(DownloadEvent::Succeeded { id: done });
    assert_eq!(controller.reveal_candidate(done), Some("/Downloads/done"));

    // Failed is not a reveal candidate (Retry is its action).
    let failed = run_attempt(&mut controller, "failed");
    controller.apply(DownloadEvent::Failed {
        id: failed,
        failure: DownloadFailure::Network,
    });
    assert_eq!(controller.reveal_candidate(failed), None);

    // Unknown ids have none.
    assert_eq!(controller.reveal_candidate(id(99)), None);
}

// --- Action sink edge cases
// ---------------------------------------------------------------------------

#[test]
fn actions_without_a_sink_are_dropped_silently() {
    let (mut controller, _) = controller();
    let attempt = controller
        .start("https://example.com/f", suggested())
        .unwrap();
    assert_eq!(controller.record(attempt).unwrap().id, attempt);
    controller.cancel(attempt).unwrap();
    assert!(!controller.record(attempt).unwrap().state.is_terminal());
}
