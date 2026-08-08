//! The macOS download adapter facade: owns the pure mapping core
//! ([`DownloadAdapterCore`]) and the native strategy
//! ([`DownloadStrategy`]), and routes native signals into L6
//! [`DownloadEvent`]s for the composition's sink.
//!
//! One adapter instance serves the whole window (downloads outlive tab
//! switches), unlike the per-view find adapter. The composition mints attempt
//! ids through the portable download controller and routes its
//! [`DownloadAction`]s here: `Start` → [`Self::start`] /
//! [`Self::adopt`], `Cancel` → [`Self::cancel`]. Native terminal signals
//! release the underlying WKDownload resources via
//! [`DownloadStrategy::finish`] after the event reaches the sink.
//!
//! The signal router is installed once; every native callback (destination
//! decision, progress KVO, finish/fail) flows through
//! [`DownloadAdapterCore::signal`], which mirrors the L6 machine so rejected
//! or late events never reach the composition. The strategy is `Rc`-shared
//! because its async completion blocks need it after the facade call
//! returns; contract tests inject a deterministic fake strategy and drive
//! signals directly without a display server.

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2_web_kit::{WKDownload, WKWebView};

use bbr_web::downloads::{
    CancelRejection, DownloadAction, DownloadEvent, DownloadId, StartRequest,
};

use crate::downloads_mapping::{AdoptRejection, DownloadAdapterCore, NativeSignal};
use crate::downloads_strategy::{DownloadSignalRouter, DownloadStrategy, RevealError};
use crate::downloads_web_kit::WebKitDownloadStrategy;

/// The production adapter type, bound to the real `WKDownload` lifecycle.
pub type NativeDownloadAdapter = WebKitDownloadAdapter<WebKitDownloadStrategy>;

/// The composition-facing event sink. Replaced by the composition whenever
/// the controller rebinds; late completions after a teardown become no-ops.
pub type DownloadEventSink = Rc<RefCell<Option<Box<dyn FnMut(DownloadEvent)>>>>;

/// The live `WKWebView` the download strategy dispatches `Start` requests on
/// (the active session's view). Portable callers (the app composition and
/// the testkit) hold this opaque slot and refresh it before each start or
/// retry; only the macOS adapter interprets the payload, so no AppKit/WebKit
/// type ever crosses into a portable crate.
#[derive(Clone, Default)]
pub struct DownloadWebViewSlot(Rc<RefCell<Option<Retained<WKWebView>>>>);

impl DownloadWebViewSlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the live view (called right before a start/retry with the
    /// active session's view; `None` clears the slot).
    pub fn set(&self, view: Option<Retained<WKWebView>>) {
        *self.0.borrow_mut() = view;
    }

    /// The currently captured view, if any.
    pub fn get(&self) -> Option<Retained<WKWebView>> {
        self.0.borrow().clone()
    }
}

/// Why [`WebKitDownloadAdapter::start`] / [`WebKitDownloadAdapter::adopt`]
/// rejected the attempt before touching the native layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterError {
    /// The id is already bound; attempt ids are minted once by the
    /// controller.
    DuplicateAttempt(DownloadId),
}

impl From<AdoptRejection> for AdapterError {
    fn from(value: AdoptRejection) -> Self {
        match value {
            AdoptRejection::Duplicate { id } => AdapterError::DuplicateAttempt(id),
        }
    }
}

/// Adapter over a native download strategy (main-thread only). Generic over
/// the strategy so contract tests can inject a deterministic fake.
pub struct WebKitDownloadAdapter<S: DownloadStrategy> {
    core: Rc<RefCell<DownloadAdapterCore>>,
    strategy: Rc<S>,
    router: DownloadSignalRouter,
    sink: DownloadEventSink,
}

impl<S: DownloadStrategy + 'static> WebKitDownloadAdapter<S> {
    /// Adapter over an injected strategy; contract tests inject a fake.
    pub fn with_strategy(strategy: S) -> Self {
        Self::with_shared_strategy(Rc::new(strategy))
    }

    /// Adapter over a shared strategy handle (tests keep inspecting the same
    /// instance the facade drives).
    pub fn with_shared_strategy(strategy: Rc<S>) -> Self {
        let core = Rc::new(RefCell::new(DownloadAdapterCore::new()));
        let sink: DownloadEventSink = Rc::new(RefCell::new(None));
        let router = install_router(core.clone(), sink.clone(), strategy.clone());
        Self {
            core,
            strategy,
            router,
            sink,
        }
    }

    /// Install the composition-facing event sink (the controller's `apply`).
    pub fn set_event_sink(&self, sink: Box<dyn FnMut(DownloadEvent)>) {
        *self.sink.borrow_mut() = Some(sink);
    }

    /// Start a fresh download for `request` on `web_view` (programmatic path
    /// — E2E fixtures and retries). Registers the attempt in the core before
    /// the native transfer begins, so no signal can precede the record.
    pub fn start(
        &self,
        request: StartRequest,
        web_view: Retained<WKWebView>,
    ) -> Result<(), AdapterError> {
        self.prepare(&request)?;
        self.strategy.start(&request, web_view, &self.router);
        Ok(())
    }

    /// Adopt a WKDownload the engine created through navigation
    /// (`webView:navigationResponse:didBecomeDownload:` handoff). Same
    /// lifecycle as [`Self::start`] without issuing a new request.
    pub fn adopt(
        &self,
        request: StartRequest,
        download: Retained<WKDownload>,
    ) -> Result<(), AdapterError> {
        self.prepare(&request)?;
        self.strategy.adopt(&request, download, &self.router);
        Ok(())
    }

    /// Register an attempt in the mapping core before its native transfer
    /// begins. Called internally by [`Self::start`] / [`Self::adopt`]; the
    /// composition and contract tests use it to establish the record before
    /// routing native signals, which keeps the adapter contract testable
    /// without a display server.
    pub fn prepare(&self, request: &StartRequest) -> Result<(), AdapterError> {
        self.core.borrow_mut().adopt(request)?;
        Ok(())
    }

    /// Request cancellation of `id`: records the request (L6
    /// [`CancelRejection`] for terminal attempts) and asks the native side
    /// to cancel. The engine confirms with a terminal event — no optimistic
    /// state change.
    pub fn cancel(&self, id: DownloadId) -> Result<(), CancelRejection> {
        self.core.borrow_mut().cancel_requested(id)?;
        self.strategy.cancel(id, &self.router);
        Ok(())
    }

    /// Whether the adapter has a record for `id` (diagnostics/tests).
    pub fn is_known(&self, id: DownloadId) -> bool {
        self.core.borrow().is_known(id)
    }

    /// Reveal `path` in Finder (typed failure when the file is missing).
    pub fn reveal(&self, path: &str) -> Result<(), RevealError> {
        self.strategy.reveal(path)
    }

    /// Whether `path` exists on disk — the Reveal-in-Finder eligibility
    /// check (DESIGN.md §11.12; pure query, never opens Finder).
    pub fn reveal_eligible(&self, path: &str) -> bool {
        self.strategy.reveal_eligible(path)
    }

    /// The underlying strategy (test access).
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// The installed signal router — the exact seam native callbacks report
    /// through (test access; routes through the core and releases terminal
    /// resources via the strategy).
    pub fn router(&self) -> &DownloadSignalRouter {
        &self.router
    }
}

impl<S: DownloadStrategy> WebKitDownloadAdapter<S> {
    /// Route one native signal (test access; production signals arrive via
    /// the router).
    pub fn route_signal(&self, signal: NativeSignal) -> Vec<DownloadEvent> {
        self.core.borrow_mut().signal(signal)
    }
}

fn install_router<S: DownloadStrategy + 'static>(
    core: Rc<RefCell<DownloadAdapterCore>>,
    sink: DownloadEventSink,
    strategy: Rc<S>,
) -> DownloadSignalRouter {
    let router: DownloadSignalRouter =
        Rc::new(RefCell::new(Some(Box::new(move |signal: NativeSignal| {
            let id = signal.id();
            let events = core.borrow_mut().signal(signal);
            // The native transfer concluded only if this signal moved the
            // machine to a terminal state (the first terminal event wins; a
            // late terminal signal after the machine is terminal produces no
            // events and must not release twice).
            let reached_terminal = events.iter().any(|event| {
                matches!(
                    event,
                    DownloadEvent::Succeeded { .. }
                        | DownloadEvent::Failed { .. }
                        | DownloadEvent::Cancelled { .. }
                )
            });
            for event in events {
                if let Some(sink) = sink.borrow_mut().as_mut() {
                    sink(event);
                }
            }
            if reached_terminal {
                strategy.finish(id);
            }
        }))));
    router
}

/// A composition helper mapping a portable [`DownloadAction`] onto the
/// adapter. The `Start` target web view comes from `slot` (the app refreshes
/// the slot with the active session's view before each start/retry); a
/// `Start` without a captured view is a typed dispatch error.
pub fn dispatch_action<S: DownloadStrategy + 'static>(
    adapter: &WebKitDownloadAdapter<S>,
    action: DownloadAction,
    slot: &DownloadWebViewSlot,
) -> Result<(), AdapterDispatchError> {
    match action {
        DownloadAction::Start(request) => {
            let Some(web_view) = slot.get() else {
                return Err(AdapterDispatchError::NoWebView(request.id));
            };
            adapter.start(request, web_view)?;
        }
        DownloadAction::Cancel { id } => {
            adapter.cancel(id)?;
        }
    }
    Ok(())
}

/// Why [`dispatch_action`] could not be carried out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterDispatchError {
    /// A `Start` arrived without a target web view (no active session).
    NoWebView(DownloadId),
    /// The adapter rejected the attempt (duplicate id).
    Adapter(AdapterError),
    /// The cancel was rejected (terminal attempt).
    Cancel(CancelRejection),
}

impl From<AdapterError> for AdapterDispatchError {
    fn from(value: AdapterError) -> Self {
        AdapterDispatchError::Adapter(value)
    }
}

impl From<CancelRejection> for AdapterDispatchError {
    fn from(value: CancelRejection) -> Self {
        AdapterDispatchError::Cancel(value)
    }
}

#[cfg(test)]
mod tests;
