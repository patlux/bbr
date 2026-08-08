//! Contract tests for the pure native↔L6 mapping (no display server, no
//! AppKit, no filesystem).
//!
//! Coverage map (downloads-sidebar-panel.md, "Unit and contract"):
//! destination decisions, progress mapping with known/unknown totals,
//! zero-byte success, failure classification, cancellation (requested vs
//! engine-internal), first-terminal-event-wins, monotonicity/total
//! stability deflection, destination-before-progress, and the
//! success-short-of-total network fallback.

use bbr_web::downloads::{
    DestinationDecision, DownloadEvent, DownloadFailure, DownloadId, DownloadProgress, StartRequest,
};

use super::{
    classify_failure, AdoptRejection, DownloadAdapterCore, FailureKind, NativeSignal,
    NSURL_ERROR_CANCELLED, NSURL_ERROR_DOMAIN, WEBKIT_ERROR_DOMAIN,
};

fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

fn suggested_start(url: &str, name: &str) -> StartRequest {
    StartRequest {
        id: id(1),
        url: url.into(),
        destination: DestinationDecision::Suggested(name.into()),
        retry_of: None,
    }
}

// ---------------------------------------------------------------------------
// Failure classification
// ---------------------------------------------------------------------------

#[test]
fn cancellation_requires_a_user_request() {
    assert_eq!(
        classify_failure(NSURL_ERROR_DOMAIN, NSURL_ERROR_CANCELLED, true),
        FailureKind::Cancelled
    );
    // Engine-internal cancel without a request is a network failure; the L6
    // `Cancelled` state is reserved for deliberate user cancellation.
    assert_eq!(
        classify_failure(NSURL_ERROR_DOMAIN, NSURL_ERROR_CANCELLED, false),
        FailureKind::Failed(DownloadFailure::Network)
    );
}

#[test]
fn webkit_errors_are_engine_rejections() {
    assert_eq!(
        classify_failure(WEBKIT_ERROR_DOMAIN, 3, false),
        FailureKind::Failed(DownloadFailure::EngineRejected)
    );
}

#[test]
fn url_network_codes_are_network_failures() {
    for code in [-1001, -1003, -1005, -1009, -1011] {
        assert_eq!(
            classify_failure(NSURL_ERROR_DOMAIN, code, false),
            FailureKind::Failed(DownloadFailure::Network),
            "code {code}"
        );
    }
}

#[test]
fn url_file_codes_are_destination_failures() {
    for code in [-1100, -1102, -1104] {
        assert_eq!(
            classify_failure(NSURL_ERROR_DOMAIN, code, false),
            FailureKind::Failed(DownloadFailure::Destination),
            "code {code}"
        );
    }
}

#[test]
fn unknown_domains_and_codes_are_unknown() {
    assert_eq!(
        classify_failure("com.example.Weird", 42, false),
        FailureKind::Failed(DownloadFailure::Unknown)
    );
}

// ---------------------------------------------------------------------------
// Known-length lifecycle: destination → progress → success
// ---------------------------------------------------------------------------

#[test]
fn known_length_download_maps_to_destination_resolved_progress_success() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/file.bin", "file.bin"))
        .unwrap();

    assert_eq!(
        core.signal(NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/file.bin".into(),
            expected_total: Some(100),
        }),
        [DownloadEvent::DestinationResolved {
            id: id(1),
            destination: "/Downloads/file.bin".into()
        }]
    );
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 40,
            total: 100
        }),
        [DownloadEvent::Progress {
            id: id(1),
            progress: DownloadProgress::new(40, Some(100)).unwrap()
        }]
    );
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 100,
            total: 100
        }),
        [
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(100, Some(100)).unwrap()
            },
            DownloadEvent::Succeeded { id: id(1) }
        ]
    );
    // Late signals after the terminal event are dropped (terminal wins).
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 100,
            total: 100
        }),
        []
    );
}

#[test]
fn zero_byte_download_succeeds_with_a_full_bar() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/empty", "empty"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/empty".into(),
        expected_total: Some(0),
    });
    // The final snapshot reports 0/0; fraction() of (0, Some(0)) is 1.0.
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 0,
            total: 0
        }),
        [
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(0, Some(0)).unwrap()
            },
            DownloadEvent::Succeeded { id: id(1) }
        ]
    );
}

#[test]
fn unknown_length_uses_the_ns_progress_total_when_it_becomes_known() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/stream", "stream"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/stream".into(),
        expected_total: None,
    });
    // Unknown: NSProgress total is 0.
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 10,
            total: 0
        }),
        [DownloadEvent::Progress {
            id: id(1),
            progress: DownloadProgress::new(10, None).unwrap()
        }]
    );
    // Total becomes known: determinate replaces indeterminate (L6).
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 50,
            total: 100
        }),
        [DownloadEvent::Progress {
            id: id(1),
            progress: DownloadProgress::new(50, Some(100)).unwrap()
        }]
    );
}

// ---------------------------------------------------------------------------
// Invariant deflection (rejected events never leak out)
// ---------------------------------------------------------------------------

#[test]
fn regressing_progress_is_dropped() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(10),
    });
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 5,
            total: 10
        })
        .len(),
        1
    );
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 3,
            total: 10
        }),
        []
    );
}

#[test]
fn a_known_total_never_reverts_or_changes() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(10),
    });
    core.signal(NativeSignal::Progress {
        id: id(1),
        received: 5,
        total: 10,
    });
    // The response-declared total is authoritative; a conflicting NSProgress
    // total is deflected by the machine.
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 7,
            total: 99
        }),
        [DownloadEvent::Progress {
            id: id(1),
            progress: DownloadProgress::new(7, Some(10)).unwrap()
        }]
    );
}

#[test]
fn progress_before_destination_is_dropped() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    assert_eq!(
        core.signal(NativeSignal::Progress {
            id: id(1),
            received: 1,
            total: 10
        }),
        []
    );
}

#[test]
fn success_short_of_a_declared_total_becomes_a_network_failure() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(100),
    });
    core.signal(NativeSignal::Progress {
        id: id(1),
        received: 40,
        total: 100,
    });
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 99,
            total: 100
        }),
        [
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(99, Some(100)).unwrap()
            },
            // Succeeded is rejected (99 < 100) and converted per L6.
            DownloadEvent::Failed {
                id: id(1),
                failure: DownloadFailure::Network
            }
        ]
    );
}

// ---------------------------------------------------------------------------
// Cancellation and the first-terminal-event-wins race
// ---------------------------------------------------------------------------

#[test]
fn user_cancel_confirms_as_cancelled() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(100),
    });
    core.signal(NativeSignal::Progress {
        id: id(1),
        received: 10,
        total: 100,
    });

    core.cancel_requested(id(1)).unwrap();
    assert_eq!(
        core.signal(NativeSignal::Failed {
            id: id(1),
            domain: NSURL_ERROR_DOMAIN.into(),
            code: NSURL_ERROR_CANCELLED,
        }),
        [DownloadEvent::Cancelled { id: id(1) }]
    );
}

#[test]
fn cancel_is_rejected_after_a_terminal_state() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    // Zero-byte success: resolve the destination, then finish (0/0).
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(0),
    });
    core.signal(NativeSignal::Finished {
        id: id(1),
        received: 0,
        total: 0,
    });
    assert!(core.cancel_requested(id(1)).is_err());
}

#[test]
fn success_racing_with_cancellation_wins_first() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(10),
    });
    core.cancel_requested(id(1)).unwrap();

    // The transfer finished before the cancel took effect.
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 10,
            total: 10
        }),
        [
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(10, Some(10)).unwrap()
            },
            DownloadEvent::Succeeded { id: id(1) }
        ]
    );
    // The cancel confirmation that lands later is dropped: success won.
    assert_eq!(
        core.signal(NativeSignal::Failed {
            id: id(1),
            domain: NSURL_ERROR_DOMAIN.into(),
            code: NSURL_ERROR_CANCELLED,
        }),
        []
    );
}

#[test]
fn cancellation_racing_with_success_wins_first() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    core.signal(NativeSignal::DestinationResolved {
        id: id(1),
        path: "/Downloads/f".into(),
        expected_total: Some(10),
    });
    core.cancel_requested(id(1)).unwrap();

    // The cancel confirmation lands first.
    assert_eq!(
        core.signal(NativeSignal::Failed {
            id: id(1),
            domain: NSURL_ERROR_DOMAIN.into(),
            code: NSURL_ERROR_CANCELLED,
        }),
        [DownloadEvent::Cancelled { id: id(1) }]
    );
    // A success that then lands (racing callback) is dropped.
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 10,
            total: 10
        }),
        []
    );
}

// ---------------------------------------------------------------------------
// Miscellaneous mapping edges
// ---------------------------------------------------------------------------

#[test]
fn duplicate_adoption_is_rejected() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    assert_eq!(
        core.adopt(&suggested_start("https://example.com/f", "f")),
        Err(AdoptRejection::Duplicate { id: id(1) })
    );
}

#[test]
fn signals_for_unknown_attempts_are_dropped() {
    let mut core = DownloadAdapterCore::new();
    assert!(!core.is_known(id(9)));
    assert_eq!(
        core.signal(NativeSignal::DestinationResolved {
            id: id(9),
            path: "/tmp/x".into(),
            expected_total: None,
        }),
        []
    );
}

#[test]
fn http_error_status_fails_as_network_and_is_terminal() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    // L6 `DownloadFailure::Network` names HTTP status explicitly; the 500
    // must never be reported as success (WKDownload would otherwise download
    // the error body and call `downloadDidFinish:`).
    assert_eq!(
        core.signal(NativeSignal::HttpStatus {
            id: id(1),
            status: 500,
        }),
        [DownloadEvent::Failed {
            id: id(1),
            failure: DownloadFailure::Network,
        }]
    );
    // First terminal event wins: a late finish (the error body the engine
    // downloaded anyway) is dropped.
    assert_eq!(
        core.signal(NativeSignal::Finished {
            id: id(1),
            received: 4,
            total: 4,
        }),
        []
    );
}

#[test]
fn unavailable_destination_fails_with_no_destination() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    assert_eq!(
        core.signal(NativeSignal::DestinationUnavailable { id: id(1) }),
        [DownloadEvent::Failed {
            id: id(1),
            failure: DownloadFailure::NoDestination
        }]
    );
}

#[test]
fn engine_rejection_maps_through() {
    let mut core = DownloadAdapterCore::new();
    core.adopt(&suggested_start("https://example.com/f", "f"))
        .unwrap();
    assert_eq!(
        core.signal(NativeSignal::Rejected {
            id: id(1),
            failure: DownloadFailure::EngineRejected,
        }),
        [DownloadEvent::Failed {
            id: id(1),
            failure: DownloadFailure::EngineRejected
        }]
    );
}
