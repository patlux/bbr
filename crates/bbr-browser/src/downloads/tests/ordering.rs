//! Deterministic ordering (the E2E row-index contract), Reveal-in-Finder
//! eligibility, and action-sink edge cases.

use bbr_web::downloads::{DownloadEvent, DownloadFailure, DownloadProgress};

use super::{controller, id, run_attempt, succeed_attempt, suggested, wired};

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
