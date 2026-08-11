//! Renderer-liveness probe adapter (renderer-isolation lane).
//!
//! The portable monitor ([`bbr_browser::responsiveness`]) owns the policy;
//! this module owns the **macOS probe mechanics** for one live view: a
//! bounded, nonblocking liveness probe executed through the public
//! client-content-world `evaluateJavaScript` seam
//! ([`WKContentWorld::defaultClientWorld`] — the same client world the find
//! adapter uses, so page scripts can neither observe nor collide with the
//! probe), plus a per-view bounded deadline [`NSTimer`]. The expression is
//! the trivial `1 + 1` class constant — it requires the renderer's JS main
//! thread to run at all and never reads page data.
//!
//! ## Semantics
//!
//! - Arming one probe bumps the per-view **generation** and starts one
//!   in-flight probe + one deadline timer.
//! - A completing probe (any `evaluateJavaScript` completion — a result or a
//!   JS error both prove the renderer's main thread ran) emits
//!   `UnresponsiveChanged(view, false)` and invalidates the deadline timer.
//! - Deadline expiry emits `UnresponsiveChanged(view, true)` (fail-closed);
//!   a later completion from the same probe is dropped (stale).
//! - Completions/timeouts for unknown, destroyed, or re-registered views are
//!   dropped (stale-id discipline mirrors the portable
//!   `ControllerEvent::UnknownWebView` path); a completion carrying a stale
//!   generation can never mutate a newer probe.
//! - All of this runs on the AppKit main thread; nothing ever waits on the
//!   renderer.
//!
//! The [`WebEngine::probe_responsiveness`] trait method is implemented on
//! [`crate::webview::WebEngineWk`] here so the 530-line `webview.rs` is
//! never grown by this lane.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_foundation::{NSError, NSString, NSTimer};
use objc2_web_kit::{WKContentWorld, WKWebView};

use bbr_web::{WebEvent, WebViewId};

use crate::delegate::EventSink;

/// How long one probe may stay in flight before the view is declared
/// unresponsive (fail-closed). Mirrors the portable
/// `ResponsivenessConfig::default().probe_deadline` (2 s): the adapter's
/// timer is the primary detection path and the portable monitor enforces
/// the same deadline as a belt-and-braces layer.
pub(crate) const PROBE_DEADLINE: Duration = Duration::from_millis(2000);

/// The trivial client-world expression the probe evaluates: `1 + 1` has no
/// page-observable side effect, never reads page data, and requires the
/// renderer's JS main thread to run at all (a busy-loop renderer cannot
/// answer it inside its deadline).
const PROBE_EXPRESSION: &str = "1 + 1";

/// One live view's probe record.
#[derive(Default)]
struct ProbeRecord {
    /// Incremented on every arm; a late answer from a previous probe (or a
    /// re-registered session) carries an older generation and is dropped.
    generation: u64,
    in_flight: bool,
    /// The armed deadline timer; invalidated when the probe completes and
    /// dropped when the view is destroyed.
    timer: Option<Retained<NSTimer>>,
}

/// The probe host's shared per-view state. `Rc<RefCell<..>>` so the
/// `evaluateJavaScript` completion block and the `NSTimer` block (both
/// delivered on the main thread) can mutate it; the engine keeps the host
/// alive for the engine's lifetime.
#[derive(Default)]
struct ProbeInner {
    records: HashMap<WebViewId, ProbeRecord>,
}

/// Per-view bounded liveness probe state, shared with the engine. The host
/// shares the engine's event sink, so `UnresponsiveChanged` observations
/// flow through the exact transport every other `WebEvent` uses.
#[derive(Clone)]
pub(crate) struct RendererProbeHost {
    inner: Rc<RefCell<ProbeInner>>,
    sink: EventSink,
}

impl RendererProbeHost {
    /// A host feeding the shared engine event sink.
    pub(crate) fn new(sink: EventSink) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ProbeInner::default())),
            sink,
        }
    }

    /// Register a fresh live view: probe record reset, so a stale
    /// completion from a previous incarnation of the id is dropped by the
    /// generation check.
    pub(crate) fn register(&self, view: WebViewId) {
        self.inner
            .borrow_mut()
            .records
            .insert(view, ProbeRecord::default());
    }

    /// Forget a destroyed/recreated view: its record (and any armed
    /// deadline timer) is dropped, so a late answer for the id cannot
    /// mutate a newer session.
    pub(crate) fn forget(&self, view: WebViewId) {
        let mut inner = self.inner.borrow_mut();
        if let Some(record) = inner.records.remove(&view) {
            if let Some(timer) = record.timer {
                // SAFETY: main thread; the timer is retained and alive.
                timer.invalidate();
            }
        }
    }

    /// Arm exactly one bounded probe for `view`: bump the generation,
    /// evaluate the trivial expression in the client content world, and
    /// start the per-view deadline timer. Never blocks; the outcome arrives
    /// as a `WebEvent::UnresponsiveChanged` through the sink.
    pub(crate) fn arm(&self, mtm: MainThreadMarker, view: WebViewId, web_view: &WKWebView) {
        let mut inner = self.inner.borrow_mut();
        let record = inner.records.entry(view).or_default();
        record.generation += 1;
        record.in_flight = true;
        let generation = record.generation;
        drop(inner);

        // SAFETY: main thread; `web_view` is retained and alive for the
        // duration of this call; `world` is the process-wide default client
        // world (retained by WebKit); the completion block only touches the
        // shared probe state and the event sink.
        let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
        let ns_script = NSString::from_str(PROBE_EXPRESSION);
        let answered_inner = self.inner.clone();
        let answered_sink = self.sink.clone();
        let completion = RcBlock::new(move |_result: *mut AnyObject, _error: *mut NSError| {
            // Any completion — a result or a JS error — proves the
            // renderer's JS main thread ran: the probe answered.
            complete_probe(&answered_inner, &answered_sink, view, generation, false);
        });
        // SAFETY: main thread; retained web view; the block is main-thread
        // delivered (WebKit guarantees it).
        unsafe {
            web_view.evaluateJavaScript_inFrame_inContentWorld_completionHandler(
                &ns_script,
                None,
                &world,
                Some(&completion),
            );
        }

        // The bounded deadline: the timer fires on the main run loop even
        // while the renderer's JS main thread is busy (the timer lives in
        // BBR's process, never in WebContent). Non-repeating; invalidated
        // early when the probe completes.
        let timeout_inner = self.inner.clone();
        let timeout_sink = self.sink.clone();
        let timer_block = RcBlock::new(move |_timer: NonNull<NSTimer>| {
            complete_probe(&timeout_inner, &timeout_sink, view, generation, true);
        });
        // SAFETY: main thread; the block is non-Send (main-thread delivery).
        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_repeats_block(
                PROBE_DEADLINE.as_secs_f64(),
                false,
                &timer_block,
            )
        };
        let mut inner = self.inner.borrow_mut();
        if let Some(record) = inner.records.get_mut(&view) {
            if record.generation == generation && record.in_flight {
                record.timer = Some(timer);
            }
        }
    }
}

/// Apply one probe outcome: generation- and in-flight-checked (a late
/// answer from a previous probe, a destroyed view, or a re-registered
/// session mutates nothing), then emit the typed observation through the
/// engine sink.
fn complete_probe(
    inner: &Rc<RefCell<ProbeInner>>,
    sink: &EventSink,
    view: WebViewId,
    generation: u64,
    timed_out: bool,
) {
    let mut inner = inner.borrow_mut();
    let Some(record) = inner.records.get_mut(&view) else {
        return;
    };
    if record.generation != generation || !record.in_flight {
        return;
    }
    record.in_flight = false;
    if let Some(timer) = record.timer.take() {
        // SAFETY: main thread; the timer is retained and alive.
        timer.invalidate();
    }
    if let Some(sink) = sink.borrow_mut().as_mut() {
        sink(WebEvent::UnresponsiveChanged(view, timed_out));
    }
}

/// Refresh the adapter's bounded liveness probe for one live view (the
/// [`WebEngine::probe_responsiveness`] implementation body): unknown or
/// destroyed ids are no-ops (the host simply has no record to arm against —
/// a fresh `register` at create time reset the generation). A free helper so
/// the 530-line `webview.rs` existing `impl WebEngine` block only carries a
/// one-line forwarding method.
pub(crate) fn probe_web_view(engine: &mut crate::webview::WebEngineWk, id: WebViewId) {
    let mtm = MainThreadMarker::new().expect("probe_responsiveness off the main thread");
    let Some(web_view) = engine.web_view(id) else {
        return;
    };
    engine.probe.arm(mtm, id, &web_view);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_expression_is_the_trivial_constant() {
        assert_eq!(PROBE_EXPRESSION, "1 + 1");
    }

    #[test]
    fn probe_deadline_is_bounded_and_matches_the_portable_constant() {
        // The adapter deadline mirrors the portable
        // `ResponsivenessConfig::default().probe_deadline` (2 s): the
        // adapter's timer is the primary detection path, the portable
        // monitor enforces the same deadline as belt-and-braces.
        assert_eq!(PROBE_DEADLINE, Duration::from_millis(2000));
    }

    #[test]
    fn stale_generation_and_unknown_views_are_dropped() {
        let sink: EventSink = Rc::new(RefCell::new(None));
        let host = RendererProbeHost::new(sink.clone());
        host.register(WebViewId(1));
        // A completion for a generation that was never armed mutates
        // nothing (no record mutation, no event).
        complete_probe(&host.inner, &host.sink, WebViewId(1), 99, false);
        assert!(!host.inner.borrow().records[&WebViewId(1)].in_flight);
        // A completion for an unknown view is dropped.
        complete_probe(&host.inner, &host.sink, WebViewId(42), 1, true);
        assert!(!host.inner.borrow().records.contains_key(&WebViewId(42)));
    }

    #[test]
    fn forgot_view_drops_its_record() {
        let sink: EventSink = Rc::new(RefCell::new(None));
        let host = RendererProbeHost::new(sink.clone());
        host.register(WebViewId(2));
        host.forget(WebViewId(2));
        assert!(!host.inner.borrow().records.contains_key(&WebViewId(2)));
    }
}
