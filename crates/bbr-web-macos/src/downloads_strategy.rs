//! The native download strategy seam: what the adapter needs from a concrete
//! `WKDownload` binding layer, and how native signals flow back.
//!
//! The pattern mirrors [`crate::find_strategy`]: the facade is generic over
//! [`DownloadStrategy`], so contract tests inject a deterministic fake while
//! the production type is [`crate::downloads_web_kit::WebKitDownloadStrategy`].
//! The strategy executes native operations synchronously and reports
//! asynchronous [`NativeSignal`]s through the shared
//! [`DownloadSignalRouter`]; the router was installed by the facade and
//! routes every signal through [`crate::downloads_mapping::DownloadAdapterCore`]
//! before the composition-facing sink sees an event.

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2_web_kit::{WKDownload, WKWebView};

use bbr_web::downloads::{DownloadId, StartRequest};

use crate::downloads_mapping::NativeSignal;

/// Where the native layer reports signals for every attempt. A clone is kept
/// by the strategy and its delegates; the facade installs the real router.
pub type DownloadSignalRouter = Rc<RefCell<Option<Box<dyn FnMut(NativeSignal)>>>>;

/// Why Reveal-in-Finder failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevealError {
    /// The resolved path does not exist on disk (file was moved or deleted).
    MissingFile { path: String },
    /// The native reveal service could not be reached.
    Unavailable,
}

/// The native operations the download adapter drives, keyed by
/// [`DownloadId`]. Main-thread only; the concrete strategy re-asserts that.
pub trait DownloadStrategy {
    /// Start a fresh download for `request` on `web_view` (programmatic
    /// start — the path E2E fixtures and retries use). The strategy binds the
    /// resulting `WKDownload` and reports signals through `router`.
    fn start(
        &self,
        request: &StartRequest,
        web_view: Retained<WKWebView>,
        router: &DownloadSignalRouter,
    );

    /// Bind a `WKDownload` the engine created through navigation
    /// (`webView:navigationResponse:didBecomeDownload:` handoff). Same
    /// binding as [`Self::start`] without issuing a new request.
    fn adopt(
        &self,
        request: &StartRequest,
        download: Retained<WKDownload>,
        router: &DownloadSignalRouter,
    );

    /// Cancel the download for `id` (the engine confirms via a terminal
    /// signal reported through `router`; no optimistic state change here).
    /// For `WKDownload` the confirmation is the `cancel:` completion block
    /// (resume data) — `didFailWithError:` is not delivered for
    /// cancellations — so the concrete strategy must route that completion
    /// as [`NativeSignal::Cancelled`].
    fn cancel(&self, id: DownloadId, router: &DownloadSignalRouter);

    /// Release native resources for `id` after its terminal signal. Safe to
    /// call more than once (unknown ids are no-ops).
    fn finish(&self, id: DownloadId);

    /// Reveal `path` in Finder. Typed failure when the file is missing.
    fn reveal(&self, path: &str) -> Result<(), RevealError>;

    /// Whether `path` currently exists on disk — the Reveal-in-Finder
    /// eligibility check (DESIGN.md §11.12: enabled only when the terminal
    /// record's resolved path exists; §10.8 `disabled` with a reason
    /// otherwise). Pure query; never opens Finder.
    fn reveal_eligible(&self, path: &str) -> bool;
}
