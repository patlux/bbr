use super::support::*;
use crate::downloads::*;

/// Exhaustive state × event matrix: every one of the 6 states crossed with
/// every one of the 5 events yields exactly the hand-derived outcome, so no
/// legal transition is missing and no illegal one slips through.
#[test]
fn exhaustive_state_event_matrix() {
    for (state_name, state) in states() {
        for (event_name, event) in events() {
            let expected = expected_outcome(state_name, event_name);
            let actual = classify(apply_event(id(1), &state, event));
            assert_eq!(actual, expected, "state {state_name} × event {event_name}");
        }
    }
}

/// The destination resolution lands the exact reported path, and a second
/// resolution is rejected with the conflicting path so the caller can route
/// it (never silently overwriting the first decision).
#[test]
fn destination_resolution_exact_payload_and_rejection() {
    let waiting = DownloadState::Waiting { destination: None };
    let resolved = apply_event(
        id(1),
        &waiting,
        DownloadEvent::DestinationResolved {
            id: id(1),
            destination: DEST.into(),
        },
    )
    .expect("resolution is legal before bytes");
    assert_eq!(
        resolved,
        DownloadState::Waiting {
            destination: Some(DEST.into())
        }
    );

    let err = apply_event(
        id(1),
        &resolved,
        DownloadEvent::DestinationResolved {
            id: id(1),
            destination: "/tmp/other.pdf".into(),
        },
    )
    .expect_err("a second resolution contradicts the first");
    assert_eq!(
        err,
        EventRejection::AlreadyResolved {
            destination: "/tmp/other.pdf".into()
        }
    );
}

/// Progress carries the resolved destination into `Running`, and advancing
/// progress keeps the destination while the byte count moves forward.
#[test]
fn progress_advances_monotonically_and_carries_destination() {
    let waiting = DownloadState::Waiting {
        destination: Some(DEST.into()),
    };
    let running = apply_event(
        id(1),
        &waiting,
        DownloadEvent::Progress {
            id: id(1),
            progress: progress(10, Some(100)),
        },
    )
    .expect("bytes flow after destination");
    let running = apply_event(
        id(1),
        &running,
        DownloadEvent::Progress {
            id: id(1),
            progress: progress(20, Some(100)),
        },
    )
    .expect("forward progress is legal");
    match running {
        DownloadState::Running {
            destination,
            progress,
        } => {
            assert_eq!(destination, DEST);
            assert_eq!(progress.received(), 20);
            assert_eq!(progress.total(), Some(100));
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

/// A progress event that reports fewer bytes than the previous one is
/// rejected with both values reported; progress is monotonic non-decreasing.
#[test]
fn progress_regression_is_rejected() {
    let running = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(10, Some(100)),
    };
    let err = apply_event(
        id(1),
        &running,
        DownloadEvent::Progress {
            id: id(1),
            progress: progress(9, Some(100)),
        },
    )
    .expect_err("regressed progress contradicts monotonicity");
    assert_eq!(
        err,
        EventRejection::ProgressRegressed {
            previous: progress(10, Some(100)),
            event: progress(9, Some(100)),
        }
    );
}

/// Equal progress is a legal no-op (monotonic non-decreasing, and idempotent
/// re-delivery must not kill the machine).
#[test]
fn equal_progress_is_a_noop() {
    let running = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(10, Some(100)),
    };
    let result = apply_event(
        id(1),
        &running,
        DownloadEvent::Progress {
            id: id(1),
            progress: progress(10, Some(100)),
        },
    )
    .expect("equal progress is monotonic");
    assert_eq!(result, running);
}

/// A known total is stable: it never changes to another value and never
/// reverts to unknown, while an unknown total may become known.
#[test]
fn total_is_stable_once_known() {
    let running = |received: u64, total: Option<u64>| DownloadState::Running {
        destination: DEST.into(),
        progress: progress(received, total),
    };
    let apply = |state: &DownloadState, received: u64, total: Option<u64>| {
        apply_event(
            id(1),
            state,
            DownloadEvent::Progress {
                id: id(1),
                progress: progress(received, total),
            },
        )
    };

    let changed =
        apply(&running(10, Some(100)), 20, Some(50)).expect_err("a known total must not change");
    assert_eq!(
        changed,
        EventRejection::TotalChanged {
            previous: Some(100),
            event: Some(50)
        }
    );

    let reverted = apply(&running(10, Some(100)), 20, None)
        .expect_err("a known total must not revert to unknown");
    assert_eq!(
        reverted,
        EventRejection::TotalChanged {
            previous: Some(100),
            event: None
        }
    );

    // Unknown → known is legal and keeps monotonic bytes.
    let became_known =
        apply(&running(10, None), 15, Some(100)).expect("the total may appear once bytes flow");
    assert_eq!(became_known.progress(), Some(&progress(15, Some(100))));

    // Unknown → unknown stays legal.
    let still_unknown = apply(&running(10, None), 20, None).expect("unknown total continues");
    assert_eq!(still_unknown.progress(), Some(&progress(20, None)));
}

/// `Succeeded` while a known total is not fully received is rejected with the
/// accounting mismatch; complete or unknown-length transfers may succeed.
#[test]
fn success_accounts_for_declared_total() {
    let short = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(50, Some(100)),
    };
    let err = apply_event(id(1), &short, DownloadEvent::Succeeded { id: id(1) })
        .expect_err("a partial declared total cannot succeed");
    assert_eq!(
        err,
        EventRejection::SuccessByteMismatch {
            received: 50,
            total: Some(100)
        }
    );

    let complete = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(100, Some(100)),
    };
    assert_eq!(
        apply_event(id(1), &complete, DownloadEvent::Succeeded { id: id(1) })
            .expect("a fully received declared total succeeds"),
        DownloadState::Succeeded
    );

    let unknown = DownloadState::Running {
        destination: DEST.into(),
        progress: progress(5, None),
    };
    assert_eq!(
        apply_event(id(1), &unknown, DownloadEvent::Succeeded { id: id(1) })
            .expect("unknown-length downloads may succeed"),
        DownloadState::Succeeded
    );
}

/// A zero-byte download succeeds directly from the resolved waiting state,
/// and never before a destination is decided.
#[test]
fn zero_byte_download_succeeds_from_resolved_waiting() {
    let waiting = DownloadState::Waiting {
        destination: Some(DEST.into()),
    };
    assert_eq!(
        apply_event(id(1), &waiting, DownloadEvent::Succeeded { id: id(1) })
            .expect("a decided destination is enough for a zero-byte success"),
        DownloadState::Succeeded
    );

    let unresolved = DownloadState::Waiting { destination: None };
    assert_eq!(
        apply_event(id(1), &unresolved, DownloadEvent::Succeeded { id: id(1) })
            .expect_err("success needs a decided destination"),
        EventRejection::UndecidedDestination
    );
}

/// Terminal states reject every further event and report the terminal state
/// itself, so the caller sees exactly what is already terminal.
#[test]
fn terminal_states_are_immutable_and_reported() {
    for (name, terminal) in states() {
        if !terminal.is_terminal() {
            continue;
        }
        for (event_name, event) in events() {
            let err = apply_event(id(1), &terminal, event.clone())
                .expect_err("terminal states reject every event");
            assert_eq!(
                err,
                EventRejection::AlreadyTerminal {
                    state: terminal.clone()
                },
                "{name} × {event_name}"
            );
        }
    }
}

/// An event carrying a foreign attempt id is rejected before any transition,
/// with both ids reported; a stale or misrouted event cannot mutate the wrong
/// attempt (contract invariant 8).
#[test]
fn event_with_foreign_id_is_rejected() {
    let waiting = DownloadState::Waiting { destination: None };
    let err = apply_event(
        id(1),
        &waiting,
        DownloadEvent::DestinationResolved {
            id: id(2),
            destination: DEST.into(),
        },
    )
    .expect_err("a foreign event id must not mutate this attempt");
    assert_eq!(
        err,
        EventRejection::MismatchedId {
            expected: id(1),
            event: id(2)
        }
    );

    // The check fires for every event kind, not just destination resolution.
    let err = apply_event(id(1), &waiting, DownloadEvent::Cancelled { id: id(9) })
        .expect_err("a foreign cancellation must not mutate this attempt");
    assert_eq!(
        err,
        EventRejection::MismatchedId {
            expected: id(1),
            event: id(9)
        }
    );
}

/// An engine cannot resolve an empty destination; the attempt stays
/// unresolved and unchanged so the caller can re-drive the decision
/// (contract invariant 9).
#[test]
fn empty_resolved_destination_is_rejected() {
    let waiting = DownloadState::Waiting { destination: None };
    let err = apply_event(
        id(1),
        &waiting,
        DownloadEvent::DestinationResolved {
            id: id(1),
            destination: String::new(),
        },
    )
    .expect_err("an empty destination cannot be resolved");
    assert_eq!(err, EventRejection::EmptyDestination);
    assert_eq!(waiting, DownloadState::Waiting { destination: None });
}
