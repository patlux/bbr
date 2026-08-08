//! Coordinator contract tests: the full classify → dedupe → route pipeline,
//! guard order, error state preservation, rejection outcomes, and
//! panic-free pathological input.

use crate::external_urls::{
    classify, BurstConfig, ExternalUrlCoordinator, ExternalUrlError, ExternalUrlOutcome,
    ExternalUrlRequest, ExternalUrlRequestId, MalformedUrl, RejectionReason, RoutingDecision,
    RoutingInput, SpaceId, WindowId, WindowSnapshot,
};
use crate::ProfileId;

fn id(n: u64) -> ExternalUrlRequestId {
    ExternalUrlRequestId(n)
}

fn request(id: ExternalUrlRequestId, raw_url: &str) -> ExternalUrlRequest {
    ExternalUrlRequest {
        id,
        raw_url: raw_url.into(),
        activation: None,
    }
}

fn window(window: u64, profile: u64, space: u64) -> WindowSnapshot {
    WindowSnapshot {
        window: WindowId(window),
        profile: ProfileId(profile),
        space: SpaceId(space),
        accepts_external_urls: true,
    }
}

fn coordinator() -> ExternalUrlCoordinator {
    ExternalUrlCoordinator::new(BurstConfig::new(3, 8).expect("valid config"))
}

/// The store reuses the frontmost eligible window end to end.
#[test]
fn routes_web_url_into_eligible_window() {
    let mut store = coordinator();
    let windows = [window(1, 10, 100)];
    let input = RoutingInput {
        windows: &windows,
        default_profile: ProfileId(99),
        default_space: SpaceId(999),
    };
    let outcome = store
        .handle(request(id(1), "https://example.com/x"), 0, &input)
        .expect("valid request");
    assert_eq!(
        outcome,
        ExternalUrlOutcome::Routed {
            request: id(1),
            decision: RoutingDecision::ReuseWindow {
                window: WindowId(1),
                profile: ProfileId(10),
                space: SpaceId(100),
            },
        }
    );
}

/// With no eligible window the store falls back to the declared defaults.
#[test]
fn routes_new_window_when_none_eligible() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };
    let outcome = store
        .handle(request(id(1), "https://example.com"), 0, &input)
        .expect("valid request");
    assert_eq!(
        outcome,
        ExternalUrlOutcome::Routed {
            request: id(1),
            decision: RoutingDecision::NewWindow {
                profile: ProfileId(42),
                space: SpaceId(7),
            },
        }
    );
}

/// Dangerous, unsupported, and malformed URLs are rejected with a typed
/// reason — never routed, never deduplicated.
#[test]
fn rejects_non_web_urls() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };

    let outcome = store
        .handle(request(id(1), "javascript:alert(1)"), 0, &input)
        .expect("valid request");
    assert_eq!(
        outcome,
        ExternalUrlOutcome::Rejected {
            request: id(1),
            reason: RejectionReason::DangerousScheme {
                scheme: "javascript".into(),
            },
        }
    );

    let outcome = store
        .handle(request(id(2), "mailto:user@example.com"), 1, &input)
        .expect("valid request");
    assert_eq!(
        outcome,
        ExternalUrlOutcome::Rejected {
            request: id(2),
            reason: RejectionReason::UnsupportedScheme {
                scheme: "mailto".into(),
            },
        }
    );

    let outcome = store
        .handle(request(id(3), "example.com"), 2, &input)
        .expect("valid request");
    assert_eq!(
        outcome,
        ExternalUrlOutcome::Rejected {
            request: id(3),
            reason: RejectionReason::Malformed(MalformedUrl::MissingScheme),
        }
    );
}

/// The same URL twice inside the burst window routes once and deduplicates
/// the second time; once the window elapses it routes again.
#[test]
fn deduplicates_repeated_deliveries_within_burst() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };

    let first = store
        .handle(request(id(1), "https://example.com/x"), 0, &input)
        .expect("valid request");
    assert!(matches!(first, ExternalUrlOutcome::Routed { .. }));

    let second = store
        .handle(request(id(2), "https://example.com/x"), 1, &input)
        .expect("valid request");
    assert_eq!(
        second,
        ExternalUrlOutcome::Deduplicated {
            request: id(2),
            original: id(1),
            observations: 2,
        }
    );

    // Window is 3 ticks anchored at tick 0: fresh again at tick 4.
    let third = store
        .handle(request(id(3), "https://example.com/x"), 4, &input)
        .expect("valid request");
    assert!(matches!(third, ExternalUrlOutcome::Routed { .. }));
}

/// Deduplication applies only to accepted web URLs; rejections are
/// deterministic on their own and are reported every time.
#[test]
fn dedup_never_suppresses_rejections() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };
    for tick in 0..3 {
        let outcome = store
            .handle(request(id(tick + 1), "file:///etc/passwd"), tick, &input)
            .expect("valid request");
        assert!(matches!(outcome, ExternalUrlOutcome::Rejected { .. }));
    }
}

/// Request ids are never reused; a reused id is a contract violation, not an
/// outcome.
#[test]
fn duplicate_request_ids_are_rejected() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };
    store
        .handle(request(id(5), "https://example.com/x"), 0, &input)
        .expect("valid request");
    let error = store
        .handle(request(id(5), "https://example.com/y"), 1, &input)
        .expect_err("reused id must fail");
    assert_eq!(
        error,
        ExternalUrlError::DuplicateRequestId {
            id: id(5),
            last: id(5),
        }
    );
    // A lower id is equally a reuse of the sequence.
    let error = store
        .handle(request(id(2), "https://example.com/z"), 2, &input)
        .expect_err("lower id must fail");
    assert_eq!(
        error,
        ExternalUrlError::DuplicateRequestId {
            id: id(2),
            last: id(5),
        }
    );
}

/// Ticks must be non-decreasing; going backwards is a contract violation.
/// Equal ticks are allowed.
#[test]
fn non_monotonic_ticks_are_rejected_but_equal_ticks_pass() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };
    store
        .handle(request(id(1), "https://example.com/a"), 5, &input)
        .expect("valid request");
    // Equal tick: allowed.
    store
        .handle(request(id(2), "https://example.com/b"), 5, &input)
        .expect("equal tick is non-decreasing");
    // Backwards tick: rejected.
    let error = store
        .handle(request(id(3), "https://example.com/c"), 4, &input)
        .expect_err("backwards tick must fail");
    assert_eq!(
        error,
        ExternalUrlError::NonMonotonicTick { tick: 4, last: 5 }
    );
}

/// A discipline error leaves the store unchanged: a later valid request is
/// not poisoned by the failed call.
#[test]
fn discipline_errors_leave_the_store_unchanged() {
    let mut store = coordinator();
    let input = RoutingInput {
        windows: &[],
        default_profile: ProfileId(42),
        default_space: SpaceId(7),
    };
    store
        .handle(request(id(2), "https://example.com/x"), 0, &input)
        .expect("valid request");
    // Reused id fails...
    let _ = store.handle(request(id(2), "https://example.com/y"), 1, &input);
    // ...and the next fresh id still routes.
    let outcome = store
        .handle(request(id(3), "https://example.com/y"), 1, &input)
        .expect("fresh id must still route");
    assert!(matches!(outcome, ExternalUrlOutcome::Routed { .. }));
}

/// Pathological input never panics: every fallible path returns a typed
/// result.
#[test]
fn pathological_input_never_panics() {
    let corpus = [
        "",
        "   ",
        ":",
        "::",
        "https://",
        "https:///",
        "https://[::1",
        "https://host:999999999999999999999999999",
        "javascript:",
        "data:text/html,<script>x</script>",
        "https://exa mple.com",
        "\u{00a0}https://example.com",
        "https://host:\u{7f}",
        "https://user:pass@",
        "a",
        "https://example.com:8080:90",
    ];
    for (index, raw) in corpus.iter().enumerate() {
        let mut store = coordinator();
        let input = RoutingInput {
            windows: &[],
            default_profile: ProfileId(42),
            default_space: SpaceId(7),
        };
        let outcome = store.handle(request(id(index as u64 + 1), raw), index as u64, &input);
        assert!(outcome.is_ok(), "handling {raw:?} must not panic or fail");
        let _ = classify(raw);
    }
}
