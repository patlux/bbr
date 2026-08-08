//! Burst deduplication contract tests: the tick-window boundary, first
//! anchoring, privacy-safe key semantics, capacity eviction, and the absence
//! of any wall clock.

use crate::external_urls::{
    classify, BurstConfig, BurstConfigError, BurstTracker, DedupVerdict, ExternalUrlRequestId,
    WebUrl,
};

const WINDOW: u64 = 3;
const CAPACITY: usize = 8;

fn tracker() -> BurstTracker {
    BurstTracker::new(BurstConfig::new(WINDOW, CAPACITY).expect("valid config"))
}

fn id(n: u64) -> ExternalUrlRequestId {
    ExternalUrlRequestId(n)
}

fn web(url: &str) -> WebUrl {
    match classify(url) {
        crate::external_urls::UrlClass::Web(url) => url,
        other => panic!("expected web URL for {url:?}, got {other:?}"),
    }
}

/// The same URL is a duplicate while `tick - first_seen_tick <= window_ticks`
/// (boundary inclusive) and fresh once the window elapses.
#[test]
fn window_boundary_is_inclusive_and_then_expires() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://a.example/x"), id(1)),
        DedupVerdict::Fresh
    );

    // 3 - 0 = 3 <= 3: still inside the window.
    assert_eq!(
        tracker.observe(3, &web("https://a.example/x"), id(2)),
        DedupVerdict::Duplicate {
            original: id(1),
            observations: 2,
        }
    );
    // 4 - 0 = 4 > 3: expired, fresh again.
    assert_eq!(
        tracker.observe(4, &web("https://a.example/x"), id(3)),
        DedupVerdict::Fresh
    );
}

/// A duplicate never refreshes the window: it is anchored to the first
/// observation, so a stream of identical arrivals cannot extend the burst.
#[test]
fn duplicates_do_not_refresh_the_window() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://a.example/x"), id(1)),
        DedupVerdict::Fresh
    );
    assert!(matches!(
        tracker.observe(1, &web("https://a.example/x"), id(2)),
        DedupVerdict::Duplicate {
            original: ExternalUrlRequestId(1),
            observations: 2
        }
    ));
    assert!(matches!(
        tracker.observe(2, &web("https://a.example/x"), id(3)),
        DedupVerdict::Duplicate {
            original: ExternalUrlRequestId(1),
            observations: 3
        }
    ));
    assert!(matches!(
        tracker.observe(3, &web("https://a.example/x"), id(4)),
        DedupVerdict::Duplicate {
            original: ExternalUrlRequestId(1),
            observations: 4
        }
    ));
    // Window anchored at 0: expired at tick 4 regardless of the repeats.
    assert_eq!(
        tracker.observe(4, &web("https://a.example/x"), id(5)),
        DedupVerdict::Fresh
    );
}

/// Distinct paths on the same origin do not dedupe: they are different URLs.
#[test]
fn distinct_paths_do_not_dedupe() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://a.example/one"), id(1)),
        DedupVerdict::Fresh
    );
    assert_eq!(
        tracker.observe(1, &web("https://a.example/two"), id(2)),
        DedupVerdict::Fresh
    );
}

/// Any byte difference in the remainder — including the fragment — means a
/// different URL and does not dedupe.
#[test]
fn distinct_fragments_do_not_dedupe() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://a.example/x#one"), id(1)),
        DedupVerdict::Fresh
    );
    assert_eq!(
        tracker.observe(1, &web("https://a.example/x#two"), id(2)),
        DedupVerdict::Fresh
    );
}

/// Scheme/host case differences collapse: the key uses the reduced (lower
/// case) origin and the byte-exact remainder, so identical targets dedupe.
#[test]
fn case_variant_host_collapses_to_same_key() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://EXAMPLE.com/x"), id(1)),
        DedupVerdict::Fresh
    );
    assert!(matches!(
        tracker.observe(1, &web("https://example.com/x"), id(2)),
        DedupVerdict::Duplicate {
            original: ExternalUrlRequestId(1),
            observations: 2
        }
    ));
}

/// An explicit default port written out does not collapse onto the
/// port-less form: the reduction keeps the port as written (matching the
/// sibling web-error rule).
#[test]
fn port_variant_does_not_collapse() {
    let mut tracker = tracker();
    assert_eq!(
        tracker.observe(0, &web("https://a.example/x"), id(1)),
        DedupVerdict::Fresh
    );
    assert_eq!(
        tracker.observe(1, &web("https://a.example:443/x"), id(2)),
        DedupVerdict::Fresh
    );
}

/// Capacity eviction bounds the remembered set: with `max_remembered` full,
/// the oldest entry is evicted and its URL becomes fresh again.
#[test]
fn capacity_eviction_evicts_the_oldest() {
    let mut tracker = BurstTracker::new(BurstConfig::new(100, 1).expect("valid config"));
    assert_eq!(
        tracker.observe(0, &web("https://a.example/one"), id(1)),
        DedupVerdict::Fresh
    );
    // Evicts the first URL to make room.
    assert_eq!(
        tracker.observe(1, &web("https://a.example/two"), id(2)),
        DedupVerdict::Fresh
    );
    // The evicted URL is fresh again, even inside its original window.
    assert_eq!(
        tracker.observe(2, &web("https://a.example/one"), id(3)),
        DedupVerdict::Fresh
    );
}

/// A zero-width window and a zero-capacity tracker are rejected up front.
#[test]
fn invalid_configs_are_rejected() {
    assert_eq!(
        BurstConfig::new(0, 1).unwrap_err(),
        BurstConfigError::ZeroWindowTicks
    );
    assert_eq!(
        BurstConfig::new(1, 0).unwrap_err(),
        BurstConfigError::ZeroCapacity
    );
}

/// Every verdict is a pure function of the explicit tick sequence: no wall
/// clock is consulted, so the same sequence always yields the same verdicts.
#[test]
fn verdicts_are_deterministic_in_ticks_only() {
    let run = |tracker: &mut BurstTracker| {
        vec![
            tracker.observe(0, &web("https://a.example/x"), id(1)),
            tracker.observe(1, &web("https://a.example/y"), id(2)),
            tracker.observe(2, &web("https://a.example/x"), id(3)),
            tracker.observe(3, &web("https://b.example/x"), id(4)),
            tracker.observe(4, &web("https://a.example/x"), id(5)),
        ]
    };
    let first = run(&mut tracker());
    let second = run(&mut tracker());
    assert_eq!(first, second);
}
