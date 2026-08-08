//! History clear / delete / restore / export — the settled post-MVP
//! persistence seams (DESIGN.md §17.7/§17.8/§17.10).

use bbr_web::downloads::{DownloadEvent, DownloadFailure, DownloadProgress, DownloadState};

use super::{controller, id, run_attempt, succeed_attempt, suggested};
use crate::downloads::{history::RestoredTerminal, DownloadError, DownloadRecord};

/// The settled Clear action empties the controller history (terminal records
/// only) and returns the removed ids so the caller can delete the persisted
/// rows; live attempts survive.
#[test]
fn clear_history_removes_terminal_records_and_keeps_live_attempts() {
    let (mut controller, _) = controller();
    let succeeded = succeed_attempt(&mut controller, "s.bin");
    let failed = run_attempt(&mut controller, "f.bin");
    controller.apply(DownloadEvent::Failed {
        id: failed,
        failure: DownloadFailure::Network,
    });
    let live = run_attempt(&mut controller, "live.bin");

    let removed = controller.clear_history();
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&succeeded));
    assert!(removed.contains(&failed));
    assert!(!removed.contains(&live));
    assert!(controller.history_ids().is_empty());
    assert_eq!(controller.active_ids(), vec![live]);
    // The removed terminal rows are gone from the authority — they cannot
    // return from this controller.
    assert!(controller.record(succeeded).is_none());
    assert!(controller.record(failed).is_none());
}

/// The settled row Delete action removes exactly one terminal record and
/// rejects live attempts (a running transfer is never deleted from under
/// the engine) and unknown ids.
#[test]
fn delete_history_removes_one_terminal_record_only() {
    let (mut controller, _) = controller();
    let cancelled = run_attempt(&mut controller, "c.bin");
    controller.apply(DownloadEvent::Cancelled { id: cancelled });
    let other = succeed_attempt(&mut controller, "o.bin");
    let live = run_attempt(&mut controller, "live.bin");

    assert_eq!(controller.delete_history(cancelled), Ok(cancelled));
    assert!(controller.record(cancelled).is_none());
    assert!(controller.record(other).is_some(), "sibling rows survive");
    assert!(controller.record(live).is_some());

    // A live attempt is never deleted.
    assert_eq!(
        controller.delete_history(live),
        Err(DownloadError::DeleteNonTerminal { id: live })
    );
    assert!(controller.record(live).is_some());
    // Unknown ids are reported.
    assert_eq!(
        controller.delete_history(id(99)),
        Err(DownloadError::UnknownAttempt(id(99)))
    );
}

/// `terminal_history` exports exactly the terminal records in ascending
/// terminal order (oldest first — the persisted order); live attempts are
/// never exported (terminal-only persistence policy).
#[test]
fn terminal_history_exports_ascending_terminal_order_only() {
    let (mut controller, _) = controller();
    let first = run_attempt(&mut controller, "a.bin");
    controller.apply(DownloadEvent::Failed {
        id: first,
        failure: DownloadFailure::Network,
    });
    let second = succeed_attempt(&mut controller, "b.bin");
    let live = run_attempt(&mut controller, "live.bin");
    let third = run_attempt(&mut controller, "c.bin");
    controller.apply(DownloadEvent::Cancelled { id: third });

    let history = controller.terminal_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].id, first, "oldest terminal first");
    assert_eq!(history[1].id, second);
    assert_eq!(history[2].id, third, "most recent terminal last");
    assert!(history.iter().all(|r| r.state.is_terminal()));
    assert!(history.iter().all(|r| r.terminal_seq.is_some()));
    assert!(
        history.iter().all(|r| r.id != live),
        "live attempts are never persisted"
    );
}

/// `restore_terminal_history` restores exactly the terminal records with
/// deterministic ordering (History stays most-recent-terminal-first), drops
/// any current record (active attempts are never resurrected), and advances
/// `next_id` so fresh attempts never collide with restored identities.
#[test]
fn restore_terminal_history_restores_ordering_and_continues_ids() {
    let (mut controller, _) = controller();
    // A live pre-restore attempt and a terminal row: the restart drops the
    // live attempt (it died with the process) and keeps the terminal row.
    let _pre_live = run_attempt(&mut controller, "pre-live.bin");
    let restored_a = DownloadRecord::restored_terminal(RestoredTerminal {
        id: id(7),
        url: "https://example.com/a.bin".into(),
        filename: "a.bin".into(),
        state: DownloadState::Cancelled,
        resolved_path: None,
        final_bytes: Some(12),
        terminal_at: 1000,
        retry_of: None,
        attempt: 1,
    })
    .expect("terminal record");
    let restored_b = DownloadRecord::restored_terminal(RestoredTerminal {
        id: id(9),
        url: "https://example.com/b.bin".into(),
        filename: "b.bin".into(),
        state: DownloadState::Succeeded,
        resolved_path: Some("/Downloads/b.bin".into()),
        final_bytes: Some(4096),
        terminal_at: 2000,
        retry_of: None,
        attempt: 1,
    })
    .expect("terminal record");

    controller
        .restore_terminal_history(vec![restored_a, restored_b])
        .expect("restore");

    // Active attempts are never resurrected.
    assert!(controller.active_ids().is_empty());
    // History most-recent-terminal-first: restored_b (seq 2) before
    // restored_a (seq 1).
    assert_eq!(controller.history_ids(), vec![id(9), id(7)]);
    assert_eq!(controller.terminal_history()[0].id, id(7));
    assert_eq!(controller.terminal_history()[1].id, id(9));
    assert_eq!(controller.record(id(7)).unwrap().terminal_at, Some(1000));
    assert_eq!(controller.record(id(9)).unwrap().terminal_seq, Some(2));

    // Fresh attempts continue past the restored ids — never a collision.
    let fresh = controller
        .start("https://example.com/new.bin", suggested())
        .unwrap();
    assert_eq!(fresh, id(10), "next_id advanced past the max restored id");
    // The restored Cancelled row is restartable: a fresh linked attempt.
    let restarted = controller
        .retry(id(7))
        .expect("restored cancelled rows restart");
    assert_eq!(restarted, id(11));
    assert_eq!(controller.record(restarted).unwrap().retry_of, Some(id(7)));
    assert_eq!(controller.record(restarted).unwrap().attempt, 2);
    // The restored Succeeded row keeps its resolved path (Reveal source).
    assert_eq!(controller.reveal_candidate(id(9)), Some("/Downloads/b.bin"));
}

/// Restore rejects non-terminal input — active attempts are never
/// resurrected as running (the settled persistence contract).
#[test]
fn restore_rejects_non_terminal_records() {
    let (mut controller, _) = controller();
    let live = DownloadRecord {
        id: id(1),
        url: "https://example.com/live.bin".into(),
        filename: "live.bin".into(),
        destination: suggested(),
        state: DownloadState::Running {
            destination: "/Downloads/live.bin".into(),
            progress: DownloadProgress::new(1, Some(10)).unwrap(),
        },
        resolved_path: Some("/Downloads/live.bin".into()),
        started_at: 0,
        terminal_at: Some(0),
        final_bytes: None,
        terminal_seq: None,
        attempt: 1,
        retry_of: None,
        retries: Vec::new(),
    };
    assert_eq!(
        controller.restore_terminal_history(vec![live]),
        Err(DownloadError::RestoreNonTerminal { id: id(1) })
    );
    assert!(
        controller.terminal_history().is_empty(),
        "a rejected restore leaves the controller unchanged"
    );
}
