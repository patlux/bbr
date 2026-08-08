//! The production download strategy: binds real `WKDownload`s, drives
//! `WKDownloadDelegate` callbacks and `NSProgress` KVO through the shared
//! signal router, and reveals files in Finder. Main-thread only.
//!
//! ## Lifecycle (mirrors the L6 contract)
//!
//! - **Start:** `startDownloadUsingRequest:completionHandler:` on the given
//!   web view hands us a `WKDownload`; the strategy binds it (delegate,
//!   progress KVO) and keeps it retained — WebKit's `delegate` property is
//!   weak and the completion-block reference is +0.
//! - **Destination:** WebKit calls the **required**
//!   `decideDestinationUsingResponse:suggestedFilename:completionHandler:`
//!   before any bytes flow; the delegate resolves the L6 destination plan
//!   ([`crate::downloads_delegate::resolve_destination`]) and reports
//!   `DestinationResolved`/`DestinationUnavailable`.
//! - **Progress:** KVO on `completedUnitCount`/`totalUnitCount` of the
//!   download's `NSProgress` reports byte snapshots.
//! - **Terminal:** `downloadDidFinish:` and
//!   `download:didFailWithError:resumeData:` are the one-shot terminal
//!   callbacks; the router forwards them and the facade releases the native
//!   resources after routing. Cancellation is `-cancel` with a real
//!   completion block: `WKDownload` does not deliver `didFailWithError:` for
//!   cancellations, so the block routes the engine's confirmation
//!   (`NativeSignal::Cancelled`) itself.
//!
//! ## Safety
//!
//! Every method re-asserts the main thread. The delegate, WKDownload and
//! NSProgress are retained by the strategy (`HashMap`), and a finished
//! delegate is moved into `pending_release` so it is never deallocated while
//! its own Objective-C method is still on the stack; the queue drains on the
//! next strategy entry. The router closure is `Rc<RefCell<...>>`, never the
//! strategy itself, so a signal arriving after the adapter was dropped is
//! harmless.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, MainThreadMarker};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{
    NSArray, NSData, NSFileManager, NSKeyValueObservingOptions,
    NSObjectNSKeyValueObserverRegistration, NSProgressReporting, NSString, NSURLRequest, NSURL,
};
use objc2_web_kit::{WKDownload, WKWebView};

use bbr_web::downloads::{DownloadFailure, DownloadId, StartRequest};

use crate::downloads_delegate::{
    progress_kvo_context, DownloadDelegate, DownloadDelegateIvars, PROGRESS_KEY, TOTAL_KEY,
};
use crate::downloads_destination::destination_plan;
use crate::downloads_mapping::NativeSignal;
use crate::downloads_strategy::{DownloadSignalRouter, DownloadStrategy, RevealError};

/// Options that change download adapter behaviour; populated by the
/// composition root.
#[derive(Debug, Clone, Default)]
pub struct DownloadAdapterOptions {
    /// Default folder for `Suggested` destinations; `None` = the system
    /// downloads directory. The v1 destination policy (silent suggested
    /// folder vs. a per-download save panel) is DESIGN §17.9 — still open —
    /// so the folder source stays composition-configurable and no save panel
    /// is built here.
    pub download_folder: Option<String>,
}

/// Shared native state the strategy owns and hands to async blocks.
#[derive(Clone)]
struct StrategyState {
    options: DownloadAdapterOptions,
    /// id → live delegate. WebKit does not retain the download delegate, so
    /// the adapter keeps every delegate alive until its terminal signal.
    downloads: Rc<RefCell<HashMap<DownloadId, Retained<DownloadDelegate>>>>,
    /// Delegates whose terminal callback already ran; released on the next
    /// strategy entry (never mid-callback).
    pending_release: Rc<RefCell<Vec<Retained<DownloadDelegate>>>>,
}

/// The production strategy bound to the pinned `WKDownload` lifecycle.
pub struct WebKitDownloadStrategy {
    state: StrategyState,
}

impl WebKitDownloadStrategy {
    /// Create the strategy. `options` carries the composition's download
    /// folder policy (destination policy per DESIGN §17.9).
    pub fn new(mtm: MainThreadMarker, options: DownloadAdapterOptions) -> Self {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitDownloadStrategy constructed off the main thread"
        );
        let _ = mtm;
        Self {
            state: StrategyState {
                options,
                downloads: Rc::new(RefCell::new(HashMap::new())),
                pending_release: Rc::new(RefCell::new(Vec::new())),
            },
        }
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitDownloadStrategy used off the main thread"
        );
    }
}

impl DownloadStrategy for WebKitDownloadStrategy {
    fn start(
        &self,
        request: &StartRequest,
        web_view: Retained<WKWebView>,
        router: &DownloadSignalRouter,
    ) {
        self.assert_main_thread();
        let ns_url = NSString::from_str(&request.url);
        let Some(url_obj) = NSURL::URLWithString(&ns_url) else {
            log::warn!(
                "download {}: cannot parse URL, engine refuses to start",
                request.id.0
            );
            if let Some(router) = router.borrow_mut().as_mut() {
                router(NativeSignal::Rejected {
                    id: request.id,
                    failure: DownloadFailure::EngineRejected,
                });
            }
            return;
        };
        let ns_request = NSURLRequest::requestWithURL(&url_obj);
        let state = self.state.clone();
        let request = request.clone();
        let router = router.clone();
        // SAFETY: main thread; the block only touches the cloned state and
        // router; WebKit hands us a live WKDownload (+0, retained here).
        let block = block2::RcBlock::new(move |download: NonNull<WKDownload>| {
            // SAFETY: WebKit hands us a live WKDownload (+0); retain it for
            // the adapter's lifetime (the delegate property is weak).
            let Some(download) = (unsafe { Retained::retain(download.as_ptr()) }) else {
                log::warn!(
                    "download {}: WebKit handed us a null WKDownload",
                    request.id.0
                );
                return;
            };
            bind_download(&state, &request, download, &router);
        });
        unsafe {
            web_view.startDownloadUsingRequest_completionHandler(&ns_request, &block);
        }
    }

    fn adopt(
        &self,
        request: &StartRequest,
        download: Retained<WKDownload>,
        router: &DownloadSignalRouter,
    ) {
        self.assert_main_thread();
        bind_download(&self.state, request, download, router);
    }

    fn cancel(&self, id: DownloadId, router: &DownloadSignalRouter) {
        self.assert_main_thread();
        if let Some(delegate) = self.state.downloads.borrow().get(&id) {
            // SAFETY: main thread; a real completion block finalizes the
            // cancellation. `WKDownload` does not deliver
            // `download:didFailWithError:resumeData:` for cancellations —
            // the `cancel:` completion handler IS the engine's terminal
            // confirmation — so the block reports `NativeSignal::Cancelled`
            // through the shared router (the same path every other native
            // signal takes), keeping the L6 request/confirmation cycle
            // deterministic. v1 discards the resume data (retry
            // re-downloads from scratch).
            let router = router.clone();
            let block = block2::RcBlock::new(move |resume_data: *mut NSData| {
                let _ = resume_data; // v1 discards resume data.
                if let Some(router) = router.borrow_mut().as_mut() {
                    router(NativeSignal::Cancelled { id });
                }
            });
            unsafe {
                delegate.download().cancel(Some(&block));
            }
        }
    }

    fn finish(&self, id: DownloadId) {
        self.assert_main_thread();
        if let Some(delegate) = self.state.downloads.borrow_mut().remove(&id) {
            // Unregister KVO before releasing the observer.
            unsafe {
                let progress = delegate.progress().as_super();
                let observer = delegate.as_super();
                progress.removeObserver_forKeyPath_context(
                    observer,
                    &NSString::from_str(PROGRESS_KEY),
                    progress_kvo_context(),
                );
                progress.removeObserver_forKeyPath_context(
                    observer,
                    &NSString::from_str(TOTAL_KEY),
                    progress_kvo_context(),
                );
            }
            self.state.pending_release.borrow_mut().push(delegate);
        }
    }

    fn reveal(&self, path: &str) -> Result<(), RevealError> {
        self.assert_main_thread();
        let ns_path = NSString::from_str(path);
        // SAFETY: main thread; `path` is a valid file path string; the
        // is-directory out-param may be null.
        let exists = unsafe {
            NSFileManager::defaultManager()
                .fileExistsAtPath_isDirectory(&ns_path, std::ptr::null_mut())
        };
        if !exists {
            return Err(RevealError::MissingFile {
                path: path.to_owned(),
            });
        }
        let url = NSURL::fileURLWithPath_relativeToURL(&ns_path, None);
        let urls = NSArray::from_retained_slice(&[url]);
        NSWorkspace::sharedWorkspace().activateFileViewerSelectingURLs(&urls);
        Ok(())
    }

    fn reveal_eligible(&self, path: &str) -> bool {
        self.assert_main_thread();
        let ns_path = NSString::from_str(path);
        // SAFETY: main thread; `path` is a valid file path string; the
        // is-directory out-param may be null.
        unsafe {
            NSFileManager::defaultManager()
                .fileExistsAtPath_isDirectory(&ns_path, std::ptr::null_mut())
        }
    }
}

/// Bind one WKDownload to the adapter: create the retained delegate, install
/// it (WebKit's property is weak), observe progress, and register the id.
fn bind_download(
    state: &StrategyState,
    request: &StartRequest,
    download: Retained<WKDownload>,
    router: &DownloadSignalRouter,
) {
    // Sweep deferred releases from prior terminal callbacks.
    state.pending_release.borrow_mut().clear();
    let mtm = MainThreadMarker::new().expect("bind_download on the main thread");
    let id = request.id;
    let plan = destination_plan(request);
    let folder = state.options.download_folder.clone();
    // SAFETY: WKDownload conforms to NSProgressReporting; the progress exists
    // from download creation.
    let progress = NSProgressReporting::progress(&*download);
    // SAFETY: main thread; ivars are plain Rust values; the download and
    // progress are retained by the delegate.
    let delegate = unsafe {
        DownloadDelegate::new(
            mtm,
            DownloadDelegateIvars {
                id,
                plan,
                folder,
                router: router.clone(),
                download: download.clone(),
                progress: progress.clone(),
            },
        )
    };
    // SAFETY: main thread; `delegate` outlives the download (retained here).
    unsafe {
        download.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    }
    // Observe byte progress and total transitions (unknown → known).
    // SAFETY: main thread; observer (the delegate) is retained by the
    // strategy and outlives the observed progress; context is our own static.
    unsafe {
        let progress_obj = progress.as_super();
        let observer = delegate.as_super();
        progress_obj.addObserver_forKeyPath_options_context(
            observer,
            &NSString::from_str(PROGRESS_KEY),
            NSKeyValueObservingOptions::New,
            progress_kvo_context(),
        );
        progress_obj.addObserver_forKeyPath_options_context(
            observer,
            &NSString::from_str(TOTAL_KEY),
            NSKeyValueObservingOptions::New,
            progress_kvo_context(),
        );
    }
    state.downloads.borrow_mut().insert(id, delegate);
}
