//! Real favicon acquisition adapter (task packet `real-favicon-pipeline.md`).
//!
//! The macOS half of the acquisition contract: after a navigation finishes,
//! an **isolated content-world script** (BBR's own
//! [`WKContentWorld::defaultClientWorld`] — the same isolation the find and
//! content-probe adapters use, so page scripts can neither observe nor
//! collide with the collector) reads the committed DOM's
//! `<link rel~="icon">` candidates in document order. One best candidate is
//! chosen by the deterministic portable rule
//! ([`bbr_web::choose_best_candidate`]); `/favicon.ico` derived from the
//! committed URL is the final fallback when the page declares no icon.
//!
//! The chosen candidate downloads through a **session bound to the owning
//! profile's data store** (see [`download`]): the profile's
//! [`WKHTTPCookieStore`] seeds a private [`NSHTTPCookieStorage`] that backs
//! a bounded ephemeral [`NSURLSession`], so the request carries exactly that
//! profile's cookies and two profiles can never share a fetch, a cache
//! entry, or a session.
//!
//! ## Bounds and silent failure
//!
//! - **Size:** the response's declared length and the accumulated payload
//!   are both capped at [`bounds::FAVICON_MAX_BYTES`]; an over-bound
//!   download is cancelled.
//! - **Deadline:** the session's request and resource timeouts cap one
//!   download at [`bounds::FAVICON_DOWNLOAD_DEADLINE`] (bounded; never a
//!   hang).
//! - **Concurrency:** at most [`bounds::FAVICON_MAX_IN_FLIGHT`] downloads
//!   run concurrently across all live views; further requests wait in a
//!   bounded FIFO queue ([`bounds::FAVICON_QUEUE_CAP`]).
//! - **Silent failure:** any failure — a page with no resolvable candidate,
//!   a non-2xx response, an over-bound payload, a timeout, a destroyed or
//!   re-registered view — emits **no** event, so the §7.2 monogram stays
//!   the deterministic fallback and there is never a retry storm.
//!
//! Only a successful bounded download emits
//! [`WebEvent::FaviconChanged`] with [`FaviconContent::Image`], landing in
//! the owning profile's bounded host-keyed store on the portable side.
//!
//! ## Thread model
//!
//! The host is main-thread-only (`Rc`-based, like the event sink it
//! shares). The bounded download delegate must implement
//! `NSURLSessionDataDelegate`, whose protocol requires `Send + Sync`, so
//! the delegate is an `AnyThread` class holding only `Send + Sync` ivars
//! (a numeric [`HostRef`] plus `Mutex`/`AtomicBool` state). The delegate's
//! terminal path resolves the host through a main-thread registry keyed by
//! [`HostRef`]; the entry exists exactly while the host is alive (it is
//! removed in the host's `Drop`), so a terminal callback that outlives the
//! engine finds no host and stays silent — the documented failure mode.
//!
//! ## Module layout
//!
//! The acquisition surface is split so every file stays under the ~500-line
//! budget and each concern owns one file: [`bounds`] (the bounded constants,
//! the collector script, the fallback/parsing rules), this module (the
//! host, the per-view records, the bounded FIFO queue, the main-thread
//! registry), [`download`] (the profile-bound download start), and
//! [`delegate`] (the bounded download delegate).

mod bounds;
mod delegate;
mod download;

pub(crate) use bounds::{
    candidates_script, fallback_icon_url, parse_candidates, FAVICON_MAX_IN_FLIGHT,
    FAVICON_QUEUE_CAP, FAVICON_TARGET_SIZE,
};

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_foundation::{NSError, NSString, NSURLSession};
use objc2_web_kit::{WKContentWorld, WKWebView};

use bbr_web::{choose_best_candidate, FaviconCandidate, FaviconContent, WebEvent, WebViewId};

use crate::delegate::EventSink;

// Next host registry id (monotonic; never reused within a process run).
static NEXT_HOST_ID: AtomicU64 = AtomicU64::new(1);

// The main-thread registry of live acquisition hosts, keyed by [`HostRef`].
//
// # Soundness
//
// Entries are weak references to heap-stable shared host state. They are
// inserted when a host is created and removed when the last host clone drops,
// both on the main thread. The download delegate's terminal path (main queue)
// upgrades the weak reference before touching anything: a successful upgrade
// keeps the host alive for the complete callback, while a failed upgrade means
// the engine was torn down and the result stays silent. No raw pointer crosses
// an asynchronous boundary.
thread_local! {
    static ACQUISITION_HOSTS: RefCell<HashMap<u64, Weak<HostShared>>> =
        RefCell::new(HashMap::new());
}

/// A `Send + Sync` handle to the main-thread acquisition host, carried by
/// the download delegate's ivars. Resolved through [`ACQUISITION_HOSTS`] on
/// the main thread.
#[derive(Clone, Copy)]
pub(crate) struct HostRef(u64);

impl HostRef {
    /// Resolve the live host, if any. `None` when the host is gone (the
    /// engine was torn down) — the caller then stays silent.
    fn host(self) -> Option<FaviconAcquisitionHost> {
        let shared = ACQUISITION_HOSTS.with(|map| {
            let mut map = map.borrow_mut();
            let shared = map.get(&self.0).and_then(Weak::upgrade);
            if shared.is_none() {
                map.remove(&self.0);
            }
            shared
        })?;
        Some(FaviconAcquisitionHost {
            shared,
            host_ref: self,
        })
    }
}

/// One acquisition record for a live view: the current generation (stale
/// completions are dropped) and whether an acquisition is in flight (a
/// repeated navigation commit is dropped while one is active).
#[derive(Default)]
struct ViewRecord {
    generation: u64,
    active: bool,
}

/// A download waiting for a concurrency slot, or in the async cookie-seed
/// window.
struct PendingDownload {
    view: WebViewId,
    generation: u64,
    url: String,
    /// Retained so the profile's data-store cookie authority stays
    /// resolvable even if the session that committed the page is destroyed
    /// mid-download (the bounded queue caps the retention).
    web_view: Retained<WKWebView>,
}

/// The shared acquisition state (main-thread only).
#[derive(Default)]
struct AcquisitionInner {
    records: HashMap<WebViewId, ViewRecord>,
    /// Live download sessions keyed by view (destroyed views are
    /// invalidated through this map).
    sessions: HashMap<WebViewId, Retained<NSURLSession>>,
    /// Global in-flight download count (bounded).
    in_flight: usize,
    /// Bounded FIFO wait queue.
    queue: VecDeque<PendingDownload>,
}

/// Heap-stable state shared by the engine-owned host and its bounded async
/// clones. The registry stores only a weak reference to this value.
struct HostShared {
    inner: RefCell<AcquisitionInner>,
    sink: EventSink,
}

/// The acquisition host: one per engine, shared with every navigation
/// delegate, feeding the shared engine event sink. Main-thread only.
pub struct FaviconAcquisitionHost {
    shared: Rc<HostShared>,
    /// The registry id handed to download delegates.
    host_ref: HostRef,
}

impl FaviconAcquisitionHost {
    /// A host feeding the shared engine event sink. Public so the
    /// delegate lifecycle contract tests can construct the shared
    /// [`crate::delegate::DelegateHandles`] bundle.
    pub fn new(sink: EventSink) -> Self {
        let host_ref = HostRef(NEXT_HOST_ID.fetch_add(1, Ordering::Relaxed));
        let shared = Rc::new(HostShared {
            inner: RefCell::new(AcquisitionInner::default()),
            sink,
        });
        ACQUISITION_HOSTS.with(|map| {
            map.borrow_mut().insert(host_ref.0, Rc::downgrade(&shared));
        });
        Self { shared, host_ref }
    }

    /// Forget a destroyed/recreated view: its record, queued downloads, and
    /// live download session are dropped or invalidated, so a late
    /// completion can never mutate a newer session (the invalidation lets
    /// the download delegate's terminal path release its concurrency slot).
    pub(crate) fn forget(&self, view: WebViewId) {
        let mut inner = self.shared.inner.borrow_mut();
        inner.records.remove(&view);
        inner.queue.retain(|pending| pending.view != view);
        if let Some(session) = inner.sessions.remove(&view) {
            // SAFETY: main thread; the session is retained and alive.
            session.invalidateAndCancel();
        }
    }

    /// Start acquisition for `web_view` after a navigation commit: collect
    /// the page's declared candidates in the isolated client content world,
    /// then download the best candidate through the profile-bound session.
    /// One acquisition per live view; a repeated commit while one is in
    /// flight is dropped (silent). All steps are asynchronous and
    /// main-thread; a result arrives as `WebEvent::FaviconChanged` with
    /// `FaviconContent::Image` only on success.
    pub(crate) fn start(&self, mtm: MainThreadMarker, view: WebViewId, web_view: &WKWebView) {
        let mut inner = self.shared.inner.borrow_mut();
        let record = inner.records.entry(view).or_default();
        if record.active {
            return;
        }
        record.generation += 1;
        record.active = true;
        let generation = record.generation;
        drop(inner);

        let host = self.clone();
        let ns_script = NSString::from_str(candidates_script());
        // BBR's own client content world — never the page's main content
        // world: the collector reads the shared DOM but page scripts can
        // neither observe nor collide with it (the same isolation the find
        // and content-probe adapters use).
        let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
        // A retained clone keeps the view alive for the bounded lifetime of
        // the pending completion (a concurrent `destroy_web_view` can never
        // leave the block holding a deallocated view).
        let view_for_block: Retained<WKWebView> = web_view.into();
        let completion = RcBlock::new(move |result: *mut AnyObject, error: *mut NSError| {
            host.on_candidates(mtm, view, generation, &view_for_block, result, error);
        });
        // SAFETY: main thread; `web_view` is retained and alive for the
        // duration of this call; `world` is the process-wide default client
        // world (retained by WebKit); the completion block only touches the
        // host and a retained clone of the view.
        unsafe {
            web_view.evaluateJavaScript_inFrame_inContentWorld_completionHandler(
                &ns_script,
                None,
                &world,
                Some(&completion),
            );
        }
    }

    /// The collector completion: parse the candidates, apply the
    /// `/favicon.ico` fallback, choose the best candidate deterministically,
    /// and enqueue the bounded download. Any failure (JS error, malformed
    /// payload, no resolvable candidate) releases the view silently.
    fn on_candidates(
        &self,
        mtm: MainThreadMarker,
        view: WebViewId,
        generation: u64,
        web_view: &WKWebView,
        result: *mut AnyObject,
        error: *mut NSError,
    ) {
        if !error.is_null() || result.is_null() {
            log::warn!("favicon: candidate collection failed for view {view:?}");
            self.release_view(view, generation);
            return;
        }
        // SAFETY: `result` is the JS return value (an NSString for our
        // string-returning script) or null; downcast_ref verifies the class
        // before dereferencing.
        let Some(object) = (unsafe { result.as_ref() }) else {
            self.release_view(view, generation);
            return;
        };
        let Some(ns_string) = object.downcast_ref::<NSString>() else {
            log::warn!("favicon: candidate collection returned a non-string value");
            self.release_view(view, generation);
            return;
        };
        let mut candidates = parse_candidates(&ns_string.to_string());
        // The deterministic `/favicon.ico` final fallback: the page declared
        // no icon, so derive the conventional location from the committed
        // URL (never guessed from the DOM).
        if candidates.is_empty() {
            let committed = (unsafe { web_view.URL() })
                .and_then(|url| url.absoluteString())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if let Some(fallback) = fallback_icon_url(&committed) {
                candidates.push(FaviconCandidate {
                    url: fallback,
                    size: None,
                });
            }
        }
        let Some(best) = choose_best_candidate(&candidates, FAVICON_TARGET_SIZE) else {
            self.release_view(view, generation);
            return;
        };
        self.enqueue(PendingDownload {
            view,
            generation,
            url: candidates[best].url.clone(),
            web_view: web_view.into(),
        });
        let _ = mtm;
    }

    /// Release a view's in-flight acquisition (the async candidate path
    /// failed or the view became stale). Generation-checked: a stale
    /// completion never mutates a newer acquisition.
    fn release_view(&self, view: WebViewId, generation: u64) {
        let mut inner = self.shared.inner.borrow_mut();
        let Some(record) = inner.records.get_mut(&view) else {
            return;
        };
        if record.generation != generation || !record.active {
            return;
        }
        record.active = false;
    }

    /// Enqueue a download under the bounded concurrency cap: start it
    /// immediately when a slot is free, else wait in the bounded FIFO queue
    /// (a full queue drops the request silently).
    fn enqueue(&self, pending: PendingDownload) {
        let mut inner = self.shared.inner.borrow_mut();
        let Some(record) = inner.records.get_mut(&pending.view) else {
            return;
        };
        if record.generation != pending.generation || !record.active {
            return;
        }
        if inner.in_flight < FAVICON_MAX_IN_FLIGHT {
            inner.in_flight += 1;
            drop(inner);
            download::begin_download(self, pending);
        } else if inner.queue.len() < FAVICON_QUEUE_CAP {
            inner.queue.push_back(pending);
        } else {
            // Full queue: drop silently (the §7.2 monogram stays).
            drop(inner);
            self.release_view(pending.view, pending.generation);
        }
    }

    /// Release one consumed concurrency slot and start the next queued
    /// download. Exactly one caller runs this per started download: the
    /// delegate's terminal path, or the cookie-seed drop path when no
    /// session was ever created.
    fn release_slot(&self) {
        let mut inner = self.shared.inner.borrow_mut();
        inner.in_flight = inner.in_flight.saturating_sub(1);
        if let Some(pending) = inner.queue.pop_front() {
            inner.in_flight += 1;
            drop(inner);
            download::begin_download(self, pending);
        }
    }

    /// The download delegate's terminal path: emit the acquired bytes when
    /// the view's acquisition is still current, then release the view and
    /// the concurrency slot and start the next queued download.
    fn complete_download(&self, view: WebViewId, generation: u64, bytes: Option<Vec<u8>>) {
        let current = {
            let inner = self.shared.inner.borrow();
            inner
                .records
                .get(&view)
                .is_some_and(|record| record.generation == generation && record.active)
        };
        if current {
            if let Some(bytes) = bytes {
                if let Some(sink) = self.shared.sink.borrow_mut().as_mut() {
                    sink(WebEvent::FaviconChanged(view, FaviconContent::Image(bytes)));
                }
            }
        }
        let mut inner = self.shared.inner.borrow_mut();
        if let Some(record) = inner.records.get_mut(&view) {
            if record.generation == generation {
                record.active = false;
            }
        }
        inner.sessions.remove(&view);
        drop(inner);
        self.release_slot();
    }
}

impl Clone for FaviconAcquisitionHost {
    fn clone(&self) -> Self {
        // A clone shares the same inner state and registry entry (the
        // original is the engine's copy; clones outlive it only while the
        // engine's copy is alive, since the registry entry is removed in
        // the *last* drop — the engine's copy is kept for the engine's
        // lifetime, so delegates never resolve a dead host).
        Self {
            shared: self.shared.clone(),
            host_ref: self.host_ref,
        }
    }
}

impl Drop for FaviconAcquisitionHost {
    fn drop(&mut self) {
        if Rc::strong_count(&self.shared) == 1 {
            ACQUISITION_HOSTS.with(|map| {
                map.borrow_mut().remove(&self.host_ref.0);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host() -> FaviconAcquisitionHost {
        FaviconAcquisitionHost::new(Rc::new(RefCell::new(None)))
    }

    #[test]
    fn short_lived_clone_cannot_remove_the_live_engine_entry() {
        let engine = host();
        let host_ref = engine.host_ref;
        let clone = engine.clone();
        drop(clone);
        let resolved = host_ref.host().expect("engine keeps registry live");
        assert!(Rc::ptr_eq(&resolved.shared, &engine.shared));
    }

    #[test]
    fn dropping_the_last_engine_host_makes_late_callbacks_silent() {
        let engine = host();
        let host_ref = engine.host_ref;
        drop(engine);
        assert!(host_ref.host().is_none());
        ACQUISITION_HOSTS.with(|map| assert!(!map.borrow().contains_key(&host_ref.0)));
    }

    #[test]
    fn successful_upgrade_keeps_state_alive_until_the_callback_finishes() {
        let engine = host();
        let host_ref = engine.host_ref;
        let callback_host = host_ref.host().expect("callback upgrades host");
        drop(engine);
        assert!(
            host_ref.host().is_some(),
            "the upgraded callback host keeps shared state alive"
        );
        drop(callback_host);
        assert!(host_ref.host().is_none());
    }
}
