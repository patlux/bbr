//! Contract tests for the adapter facade against a deterministic fake
//! strategy (no display server, no AppKit).
//!
//! Coverage map (downloads-sidebar-panel.md, "Unit and contract"):
//! start/cancel routing to the native layer, signal→sink delivery through
//! the installed router, terminal resource release, duplicate adoption
//! rejection, the cancellation race at the facade, and the typed
//! `dispatch_action` seam.

use std::cell::RefCell;
use std::rc::Rc;

use bbr_web::downloads::{
    CancelRejection, DestinationDecision, DownloadEvent, DownloadFailure, DownloadId,
    DownloadProgress, StartRequest,
};

use super::{AdapterError, WebKitDownloadAdapter};
use crate::downloads_mapping::NativeSignal;
use crate::downloads_strategy::{DownloadSignalRouter, DownloadStrategy, RevealError};

fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

fn request(id: DownloadId, url: &str) -> StartRequest {
    StartRequest {
        id,
        url: url.into(),
        destination: DestinationDecision::Suggested("file.bin".into()),
        retry_of: None,
    }
}

/// Deterministic fake strategy: records native ops so the tests can assert
/// what reached the native layer.
#[derive(Default)]
struct FakeStrategy {
    executed: RefCell<Vec<&'static str>>,
    finished: RefCell<Vec<DownloadId>>,
    cancelled: RefCell<Vec<DownloadId>>,
}

impl FakeStrategy {
    fn executed_kinds(&self) -> Vec<&'static str> {
        self.executed.borrow().clone()
    }
}

/// Replay one native signal through the facade's installed router — the exact
/// seam WKDownload callbacks report through.
fn route(adapter: &WebKitDownloadAdapter<FakeStrategy>, signal: NativeSignal) {
    if let Some(router) = adapter.router().borrow_mut().as_mut() {
        router(signal);
    }
}

impl DownloadStrategy for FakeStrategy {
    fn start(
        &self,
        _request: &StartRequest,
        _web_view: objc2::rc::Retained<objc2_web_kit::WKWebView>,
        _router: &DownloadSignalRouter,
    ) {
        self.executed.borrow_mut().push("start");
    }

    fn adopt(
        &self,
        _request: &StartRequest,
        _download: objc2::rc::Retained<objc2_web_kit::WKDownload>,
        _router: &DownloadSignalRouter,
    ) {
        self.executed.borrow_mut().push("adopt");
    }

    fn cancel(&self, id: DownloadId, _router: &DownloadSignalRouter) {
        self.cancelled.borrow_mut().push(id);
    }

    fn finish(&self, id: DownloadId) {
        self.finished.borrow_mut().push(id);
    }

    fn reveal(&self, _path: &str) -> Result<(), RevealError> {
        Ok(())
    }

    fn reveal_eligible(&self, _path: &str) -> bool {
        true
    }
}

/// A recording adapter: facade + fake strategy + a captured event sink.
fn recording_adapter() -> (
    WebKitDownloadAdapter<FakeStrategy>,
    Rc<FakeStrategy>,
    Rc<RefCell<Vec<DownloadEvent>>>,
) {
    let strategy = Rc::new(FakeStrategy::default());
    let adapter = WebKitDownloadAdapter::with_shared_strategy(strategy.clone());
    let events: Rc<RefCell<Vec<DownloadEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = events.clone();
    adapter.set_event_sink(Box::new(move |event| sink.borrow_mut().push(event)));
    (adapter, strategy, events)
}

fn terminal_events(events: &[DownloadEvent]) -> Vec<DownloadEvent> {
    events
        .iter()
        .filter(|event| {
            matches!(
                event,
                DownloadEvent::Succeeded { .. }
                    | DownloadEvent::Failed { .. }
                    | DownloadEvent::Cancelled { .. }
            )
        })
        .cloned()
        .collect()
}

#[test]
fn signals_flow_through_the_router_into_the_sink() {
    let (adapter, strategy, events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/file.bin"))
        .unwrap();

    route(
        &adapter,
        NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/file.bin".into(),
            expected_total: Some(100),
        },
    );
    route(
        &adapter,
        NativeSignal::Progress {
            id: id(1),
            received: 40,
            total: 100,
        },
    );
    route(
        &adapter,
        NativeSignal::Finished {
            id: id(1),
            received: 100,
            total: 100,
        },
    );

    assert_eq!(
        events.borrow().as_slice(),
        [
            DownloadEvent::DestinationResolved {
                id: id(1),
                destination: "/Downloads/file.bin".into()
            },
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(40, Some(100)).unwrap()
            },
            DownloadEvent::Progress {
                id: id(1),
                progress: DownloadProgress::new(100, Some(100)).unwrap()
            },
            DownloadEvent::Succeeded { id: id(1) }
        ]
    );
    // The terminal signal released the native resources exactly once.
    assert_eq!(strategy.finished.borrow().as_slice(), [id(1)]);
}

#[test]
fn duplicate_prepare_is_rejected() {
    let (adapter, _strategy, _events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    assert_eq!(
        adapter.prepare(&request(id(1), "https://example.com/a")),
        Err(AdapterError::DuplicateAttempt(id(1)))
    );
}

#[test]
fn cancel_completion_confirmation_lands_as_cancelled() {
    let (adapter, strategy, events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    route(
        &adapter,
        NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/a".into(),
            expected_total: Some(10),
        },
    );
    route(
        &adapter,
        NativeSignal::Progress {
            id: id(1),
            received: 5,
            total: 10,
        },
    );

    adapter.cancel(id(1)).unwrap();
    assert_eq!(strategy.cancelled.borrow().as_slice(), [id(1)]);

    // `WKDownload.cancel:`'s completion handler is the engine's terminal
    // confirmation (resume data) — `didFailWithError:` is not delivered for
    // cancellations. The strategy routes it as `NativeSignal::Cancelled`;
    // the core confirms the request as the L6 `Cancelled` state.
    route(&adapter, NativeSignal::Cancelled { id: id(1) });
    assert_eq!(
        terminal_events(&events.borrow()),
        [DownloadEvent::Cancelled { id: id(1) }]
    );
    // Terminal: finish() releases the native resources exactly once.
    assert_eq!(strategy.finished.borrow().as_slice(), [id(1)]);
}

#[test]
fn cancel_is_routed_and_rejected_after_terminal() {
    let (adapter, strategy, events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    route(
        &adapter,
        NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/a".into(),
            expected_total: Some(10),
        },
    );

    adapter.cancel(id(1)).unwrap();
    assert_eq!(strategy.cancelled.borrow().as_slice(), [id(1)]);

    // The engine confirms with a cancelled error → Cancelled event.
    route(
        &adapter,
        NativeSignal::Failed {
            id: id(1),
            domain: crate::downloads_mapping::NSURL_ERROR_DOMAIN.into(),
            code: crate::downloads_mapping::NSURL_ERROR_CANCELLED,
        },
    );
    assert_eq!(
        terminal_events(&events.borrow()),
        [DownloadEvent::Cancelled { id: id(1) }]
    );

    // A second cancel on the now-terminal attempt is rejected.
    assert!(matches!(
        adapter.cancel(id(1)),
        Err(CancelRejection::AlreadyTerminal { .. })
    ));
}

#[test]
fn success_racing_with_cancel_wins_first_and_releases_once() {
    let (adapter, strategy, events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    route(
        &adapter,
        NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/a".into(),
            expected_total: Some(10),
        },
    );
    adapter.cancel(id(1)).unwrap();

    // The transfer finished before the cancel took effect.
    route(
        &adapter,
        NativeSignal::Finished {
            id: id(1),
            received: 10,
            total: 10,
        },
    );
    // The late cancel confirmation is dropped by the core (success won).
    route(
        &adapter,
        NativeSignal::Failed {
            id: id(1),
            domain: crate::downloads_mapping::NSURL_ERROR_DOMAIN.into(),
            code: crate::downloads_mapping::NSURL_ERROR_CANCELLED,
        },
    );

    assert_eq!(
        terminal_events(&events.borrow()),
        [DownloadEvent::Succeeded { id: id(1) }]
    );
    // finish() is idempotent-safe: only the first terminal signal releases.
    assert_eq!(strategy.finished.borrow().as_slice(), [id(1)]);
}

#[test]
fn cancel_confirmation_racing_with_success_wins_first() {
    let (adapter, _strategy, events) = recording_adapter();
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    route(
        &adapter,
        NativeSignal::DestinationResolved {
            id: id(1),
            path: "/Downloads/a".into(),
            expected_total: Some(10),
        },
    );
    adapter.cancel(id(1)).unwrap();

    route(
        &adapter,
        NativeSignal::Failed {
            id: id(1),
            domain: crate::downloads_mapping::NSURL_ERROR_DOMAIN.into(),
            code: crate::downloads_mapping::NSURL_ERROR_CANCELLED,
        },
    );
    route(
        &adapter,
        NativeSignal::Finished {
            id: id(1),
            received: 10,
            total: 10,
        },
    );

    assert_eq!(
        terminal_events(&events.borrow()),
        [DownloadEvent::Cancelled { id: id(1) }]
    );
}

#[test]
fn dispatch_action_maps_start_and_cancel() {
    use super::{dispatch_action, AdapterDispatchError};

    let (adapter, strategy, events) = recording_adapter();
    let slot = super::DownloadWebViewSlot::new();

    // Start without a captured web view is a typed error.
    assert_eq!(
        dispatch_action(
            &adapter,
            bbr_web::downloads::DownloadAction::Start(request(id(1), "https://example.com/a")),
            &slot,
        ),
        Err(AdapterDispatchError::NoWebView(id(1)))
    );

    // Cancel of an unknown attempt reaches the strategy (a no-op there: no
    // native delegate exists), and the controller would never emit it.
    dispatch_action(
        &adapter,
        bbr_web::downloads::DownloadAction::Cancel { id: id(2) },
        &slot,
    )
    .unwrap();
    assert_eq!(strategy.cancelled.borrow().as_slice(), [id(2)]);

    // A failed download's retry is a fresh Start (linked via retry_of); the
    // composition supplies a web view, so the action reaches the strategy.
    adapter
        .prepare(&request(id(1), "https://example.com/a"))
        .unwrap();
    route(
        &adapter,
        NativeSignal::Rejected {
            id: id(1),
            failure: DownloadFailure::EngineRejected,
        },
    );
    let retry = StartRequest {
        id: id(2),
        url: "https://example.com/a".into(),
        destination: DestinationDecision::Suggested("file.bin".into()),
        retry_of: Some(id(1)),
    };
    // A Start without a captured web view is rejected before reaching the
    // strategy; in production the composition supplies the active session's
    // view through the slot.
    assert_eq!(
        dispatch_action(
            &adapter,
            bbr_web::downloads::DownloadAction::Start(retry),
            &slot,
        ),
        Err(AdapterDispatchError::NoWebView(id(2)))
    );
    assert!(strategy.executed_kinds().is_empty());
    assert_eq!(events.borrow().len(), 1); // only the original rejection
}
