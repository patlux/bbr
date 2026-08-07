//! Downloads contract tests: identifier ordering, destination decisions,
//! invariant-holding byte progress, start-request validation, wire
//! action/event shapes, and (behind the `serde` feature) serializability.

use crate::downloads::{
    validate_start, DestinationDecision, DownloadAction, DownloadEvent, DownloadFailure,
    DownloadId, DownloadProgress, InvalidProgress, StartRejection, StartRequest,
};

fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

/// A canonical well-formed start request for a fresh download.
fn fresh_start() -> StartRequest {
    StartRequest {
        id: id(1),
        url: "https://example.com/report.pdf".into(),
        destination: DestinationDecision::Suggested("report.pdf".into()),
        retry_of: None,
    }
}

/// Identifiers are plain ordered values usable as `BTreeMap`/`BTreeSet` keys
/// and `HashSet` elements, mirroring the `ProfileId`/`WebViewId` guard in
/// `lib.rs` (the perf and E2E harnesses key on ordered collections).
#[test]
fn download_ids_are_ordered_and_hashable() {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    let mut attempts = BTreeMap::new();
    attempts.insert(id(1), "first");
    attempts.insert(id(2), "retry");
    assert_eq!(attempts.get(&id(2)), Some(&"retry"));

    let mut done = BTreeSet::new();
    done.insert(id(3));
    assert!(done.contains(&id(3)));
    assert!(!done.contains(&id(4)));

    let mut seen = HashSet::new();
    seen.insert(id(9));
    assert!(seen.contains(&id(9)));
    assert_eq!(id(1), id(1));
    assert_ne!(id(1), id(2));
    assert!(id(1) < id(2));
}

/// The destination decision distinguishes a verbatim caller choice from a
/// policy-resolved suggestion, and never coerces one into the other.
#[test]
fn destination_decision_preserves_kind() {
    let explicit = DestinationDecision::Explicit("/tmp/saved.pdf".into());
    let suggested = DestinationDecision::Suggested("report.pdf".into());
    assert_eq!(
        explicit,
        DestinationDecision::Explicit("/tmp/saved.pdf".into())
    );
    assert_eq!(
        suggested,
        DestinationDecision::Suggested("report.pdf".into())
    );
    assert_ne!(explicit, suggested);
}

/// `DownloadProgress` construction accepts every value whose received count
/// does not exceed a known total, including zero-byte and full progress.
/// (The test module is a child of `downloads`, so the private fields are
/// directly constructible for exact comparisons.)
#[test]
fn progress_construction_accepts_valid_values() {
    let mk = |received: u64, total: Option<u64>| DownloadProgress { received, total };
    assert_eq!(DownloadProgress::new(0, None), Ok(mk(0, None)));
    assert_eq!(DownloadProgress::new(0, Some(0)), Ok(mk(0, Some(0))));
    assert_eq!(DownloadProgress::new(5, Some(10)), Ok(mk(5, Some(10))));
    assert_eq!(DownloadProgress::new(10, Some(10)), Ok(mk(10, Some(10))));
    assert_eq!(
        DownloadProgress::new(u64::MAX, Some(u64::MAX)),
        Ok(mk(u64::MAX, Some(u64::MAX)))
    );
}

/// A received count above a known total is rejected with the offending value;
/// invalid progress is unrepresentable rather than silently clamped.
#[test]
fn progress_construction_rejects_received_above_total() {
    assert_eq!(
        DownloadProgress::new(11, Some(10)),
        Err(InvalidProgress {
            received: 11,
            total: Some(10)
        })
    );
    assert_eq!(
        DownloadProgress::new(1, Some(0)),
        Err(InvalidProgress {
            received: 1,
            total: Some(0)
        })
    );
}

/// Accessors and derived quantities stay consistent: `received`/`total`
/// round-trip, `fraction()` is `None` exactly while the total is unknown and
/// otherwise always in `[0.0, 1.0]` (never `NaN`), and `is_complete()`
/// requires a fully received known total.
#[test]
fn progress_accessors_fraction_and_completeness() {
    let unknown = DownloadProgress::new(42, None).unwrap();
    assert_eq!(unknown.received(), 42);
    assert_eq!(unknown.total(), None);
    assert_eq!(unknown.fraction(), None);
    assert!(!unknown.is_complete());

    let zero_byte = DownloadProgress::new(0, Some(0)).unwrap();
    assert_eq!(zero_byte.fraction(), Some(1.0));
    assert!(zero_byte.is_complete());

    let partial = DownloadProgress::new(5, Some(10)).unwrap();
    assert_eq!(partial.fraction(), Some(0.5));
    assert!(!partial.is_complete());

    let full = DownloadProgress::new(10, Some(10)).unwrap();
    assert_eq!(full.fraction(), Some(1.0));
    assert!(full.is_complete());

    let half = DownloadProgress::new(50, Some(100)).unwrap();
    assert_eq!(half.received(), 50);
    assert_eq!(half.total(), Some(100));
}

/// Sweep every boundary around the total: `fraction()` is finite, in
/// `[0.0, 1.0]`, and monotonically non-decreasing with `received`.
#[test]
fn progress_fraction_is_bounded_and_monotonic() {
    let mut previous = -1.0_f64;
    for received in 0..=100_u64 {
        let p = DownloadProgress::new(received, Some(100)).unwrap();
        let f = p.fraction().expect("total known");
        assert!(f.is_finite(), "fraction must never be NaN/infinite");
        assert!((0.0..=1.0).contains(&f));
        assert!(f >= previous, "fraction must be monotonic in received");
        previous = f;
    }
    assert_eq!(
        DownloadProgress::new(100, Some(100)).unwrap().fraction(),
        Some(1.0)
    );
}

/// A well-formed fresh start passes; empty URLs and empty explicit
/// destinations are rejected, while an empty suggested destination is legal
/// (the engine derives the filename from the URL).
#[test]
fn validate_start_wellformedness() {
    assert_eq!(validate_start(&fresh_start()), Ok(()));

    let mut empty_url = fresh_start();
    empty_url.url = String::new();
    assert_eq!(validate_start(&empty_url), Err(StartRejection::EmptyUrl));

    let mut empty_explicit = fresh_start();
    empty_explicit.destination = DestinationDecision::Explicit(String::new());
    assert_eq!(
        validate_start(&empty_explicit),
        Err(StartRejection::EmptyExplicitDestination)
    );

    let mut empty_suggested = fresh_start();
    empty_suggested.destination = DestinationDecision::Suggested(String::new());
    assert_eq!(validate_start(&empty_suggested), Ok(()));
}

/// A retry-shaped request is well-formed as a start too; linkage legality is
/// `evaluate_retry`'s concern (covered in the state suite).
#[test]
fn validate_start_accepts_retry_shape() {
    let retry = StartRequest {
        id: id(2),
        url: "https://example.com/report.pdf".into(),
        destination: DestinationDecision::Suggested("report.pdf".into()),
        retry_of: Some(id(1)),
    };
    assert_eq!(validate_start(&retry), Ok(()));
}

/// Wire actions carry the start request (including the retry link) and the
/// cancel target; wire events carry the attempt id plus their payload.
#[test]
fn wire_action_and_event_shapes() {
    let start = fresh_start();
    assert_eq!(
        DownloadAction::Start(start.clone()),
        DownloadAction::Start(start.clone())
    );
    assert_ne!(
        DownloadAction::Start(start.clone()),
        DownloadAction::Cancel { id: id(1) }
    );

    let progress = DownloadProgress::new(7, Some(10)).unwrap();
    let events = [
        DownloadEvent::DestinationResolved {
            id: id(1),
            destination: "/tmp/report.pdf".into(),
        },
        DownloadEvent::Progress {
            id: id(1),
            progress,
        },
        DownloadEvent::Succeeded { id: id(1) },
        DownloadEvent::Failed {
            id: id(1),
            failure: DownloadFailure::Network,
        },
        DownloadEvent::Cancelled { id: id(1) },
    ];
    assert_eq!(events[1].clone(), events[1]);
    assert_ne!(events[0], events[1]);
    assert_ne!(events[2], events[4]);

    let retried = DownloadAction::Start(StartRequest {
        id: id(2),
        url: start.url.clone(),
        destination: start.destination.clone(),
        retry_of: Some(id(1)),
    });
    assert_ne!(DownloadAction::Start(start), retried);
}

/// With the `serde` feature every wire and state type derives
/// `Serialize`/`Deserialize`, so a later E2E protocol can move downloads
/// events and snapshots across the testkit socket. The compile-time bound is
/// the proof; the actual serialization round-trip lives where a format
/// exists (the testkit wire).
#[cfg(feature = "serde")]
#[test]
fn wire_types_are_serde_serializable() {
    fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
    assert_serde::<DownloadId>();
    assert_serde::<DestinationDecision>();
    assert_serde::<DownloadProgress>();
    assert_serde::<DownloadFailure>();
    assert_serde::<crate::downloads::DownloadState>();
    assert_serde::<StartRequest>();
    assert_serde::<DownloadAction>();
    assert_serde::<DownloadEvent>();
}
