//! Objective-C `WKDownloadDelegate` + KVO progress observer.
//!
//! WebKit does **not** retain the download delegate (the `WKDownload.delegate`
//! property is weak), so the strategy keeps every [`DownloadDelegate`]
//! retained until its terminal signal. The delegate also retains the
//! `WKDownload` it observes (for `-cancel`) and its `NSProgress` (to
//! unregister KVO on finish). Everything runs on the AppKit main thread.
//!
//! KVO context: one static context scopes the `completedUnitCount` /
//! `totalUnitCount` observations to this delegate's own download progress, so
//! unrelated `observeValueForKeyPath:` callbacks are ignored. The observed
//! `NSProgress` is `Send`/`Sync` (thread-safe), but the delegate itself is
//! main-thread-only and WebKit drives the transfer on the main thread.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{
    NSData, NSFileManager, NSHTTPURLResponse, NSObject, NSObjectProtocol, NSProgress, NSString,
    NSURLResponse, NSURL,
};
use objc2_web_kit::{WKDownload, WKDownloadDelegate};

use bbr_web::downloads::DownloadId;

use crate::downloads_destination::{uniquify_name, DestinationPlan};
use crate::downloads_mapping::NativeSignal;
use crate::downloads_strategy::DownloadSignalRouter;

/// The `completedUnitCount` key path observed on the download's NSProgress.
pub(crate) const PROGRESS_KEY: &str = "completedUnitCount";
/// The `totalUnitCount` key path observed on the download's NSProgress
/// (unknown → known transitions).
pub(crate) const TOTAL_KEY: &str = "totalUnitCount";

/// Opaque KVO context for the download progress observation. Never
/// dereferenced; scopes the callback to our own observations.
static DOWNLOAD_PROGRESS_KVO_CONTEXT: u8 = 0;

pub(crate) fn progress_kvo_context() -> *mut c_void {
    (&DOWNLOAD_PROGRESS_KVO_CONTEXT as *const u8)
        .cast_mut()
        .cast()
}

/// Instance variables for the download delegate.
pub(crate) struct DownloadDelegateIvars {
    /// The L6 attempt id this native download belongs to.
    pub(crate) id: DownloadId,
    /// The resolved destination plan (verbatim or suggested name).
    pub(crate) plan: DestinationPlan,
    /// Default folder for `Suggest` plans (`None` = system downloads dir).
    pub(crate) folder: Option<String>,
    /// Shared signal router; the facade installed it.
    pub(crate) router: DownloadSignalRouter,
    /// Retained WKDownload (cancel + progress source); WebKit's own reference
    /// is weak, so the adapter's delegate keeps the download alive.
    pub(crate) download: Retained<WKDownload>,
    /// Retained NSProgress for KVO unregistration on the terminal callback.
    pub(crate) progress: Retained<NSProgress>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = DownloadDelegateIvars]
    #[name = "BBRDownloadDelegate"]
    pub(crate) struct DownloadDelegate;

    unsafe impl NSObjectProtocol for DownloadDelegate {}

    impl DownloadDelegate {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value_for_key_path(
            &self,
            _key_path: Option<&NSString>,
            object: Option<&AnyObject>,
            _change: Option<&AnyObject>,
            context: *mut c_void,
        ) {
            // Scope to our own progress observations; ignore everyone else's.
            if context != progress_kvo_context() {
                return;
            }
            let Some(progress) = object.and_then(|o| o.downcast_ref::<NSProgress>()) else {
                return;
            };
            let (received, total) = read_progress(progress);
            self.report(NativeSignal::Progress {
                id: self.ivars().id,
                received,
                total,
            });
        }
    }

    unsafe impl WKDownloadDelegate for DownloadDelegate {
        #[unsafe(method(download:decideDestinationUsingResponse:suggestedFilename:completionHandler:))]
        fn decide_destination(
            &self,
            _download: &WKDownload,
            response: &NSURLResponse,
            _suggested_filename: &NSString,
            completion_handler: &block2::DynBlock<dyn Fn(*mut NSURL)>,
        ) {
            let id = self.ivars().id;
            // The response declares the total length (`-1` while unknown);
            // this seeds the L6 total, which is authoritative over the
            // NSProgress total reported later.
            let expected = response.expectedContentLength();
            let expected_total = (expected >= 0).then_some(expected as u64);
            // An HTTP error status (≥ 400) is a network-layer failure per the
            // L6 contract (`DownloadFailure::Network` names HTTP status).
            // `WKDownload` does not fail on HTTP status — it would download
            // the error body and report success — so the destination is
            // refused (nil cancels the transfer) and the failure is
            // classified explicitly.
            if let Some(status) = response
                .downcast_ref::<NSHTTPURLResponse>()
                .map(|http| http.statusCode())
            {
                if status >= 400 {
                    completion_handler.call((std::ptr::null_mut(),));
                    self.report(NativeSignal::HttpStatus {
                        id,
                        status: status as u16,
                    });
                    return;
                }
            }
            let plan = self.ivars().plan.clone();
            let folder = self.ivars().folder.clone();

            match resolve_destination(&plan, folder.as_deref()) {
                Some(path) => {
                    let ns_path = NSString::from_str(&path);
                    let url = NSURL::fileURLWithPath_relativeToURL(&ns_path, None);
                    // The completion block synchronously copies the URL; the
                    // local retains it for the duration of the call. Passing
                    // nil would cancel the download.
                    let ptr: *mut NSURL = (&*url) as *const NSURL as *mut NSURL;
                    completion_handler.call((ptr,));
                    self.report(NativeSignal::DestinationResolved {
                        id,
                        path,
                        expected_total,
                    });
                }
                None => {
                    // nil cancels the transfer; the core classifies this as
                    // `Failed { NoDestination }` and drops the later error.
                    completion_handler.call((std::ptr::null_mut(),));
                    self.report(NativeSignal::DestinationUnavailable { id });
                }
            }
        }

        #[unsafe(method(downloadDidFinish:))]
        fn did_finish(&self, _download: &WKDownload) {
            let (received, total) = read_progress(&self.ivars().progress);
            self.report(NativeSignal::Finished {
                id: self.ivars().id,
                received,
                total,
            });
        }

        #[unsafe(method(download:didFailWithError:resumeData:))]
        fn did_fail(
            &self,
            _download: &WKDownload,
            error: &objc2_foundation::NSError,
            _resume_data: Option<&NSData>,
        ) {
            self.report(NativeSignal::Failed {
                id: self.ivars().id,
                domain: error.domain().to_string(),
                code: error.code() as i64,
            });
        }
    }
);

impl DownloadDelegate {
    /// Create a retained delegate bound to `download`'s lifecycle.
    ///
    /// # Safety
    ///
    /// Main thread; `download` must be a live WKDownload whose progress is
    /// readable (WebKit creates the progress at download creation).
    pub(crate) unsafe fn new(
        mtm: MainThreadMarker,
        ivars: DownloadDelegateIvars,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ivars);
        // SAFETY: `init` is the NSObject initializer; we are on the main thread.
        msg_send![super(this), init]
    }

    /// The observed NSProgress (KVO unregistration + byte snapshots).
    pub(crate) fn progress(&self) -> &NSProgress {
        &self.ivars().progress
    }

    /// The retained WKDownload (cancel).
    pub(crate) fn download(&self) -> &WKDownload {
        &self.ivars().download
    }

    fn report(&self, signal: NativeSignal) {
        if let Some(router) = self.ivars().router.borrow_mut().as_mut() {
            router(signal);
        }
    }
}

/// Read the current byte snapshot from an NSProgress. A non-positive
/// `totalUnitCount` means the length is unknown (the core prefers the
/// response-declared total and treats `0` as unknown).
fn read_progress(progress: &NSProgress) -> (u64, u64) {
    let received = progress.completedUnitCount().max(0) as u64;
    let total = progress.totalUnitCount().max(0) as u64;
    (received, total)
}

/// Resolve a [`DestinationPlan`] into a concrete path string (the native
/// half of the L6 destination policy).
///
/// - [`DestinationPlan::Verbatim`]: the path exactly as chosen — no folder,
///   no uniquification (L6 `Explicit` semantics).
/// - [`DestinationPlan::Suggest`]: the filename inside `folder` (or the
///   system downloads directory), uniquified macOS-style ("name 2.ext") when
///   a file already exists. `None` when the folder cannot be established —
///   the caller then cancels the download (nil destination).
pub(crate) fn resolve_destination(plan: &DestinationPlan, folder: Option<&str>) -> Option<String> {
    match plan {
        DestinationPlan::Verbatim(path) => Some(path.clone()),
        DestinationPlan::Suggest { filename } => {
            let folder = match folder {
                Some(folder) => folder.to_owned(),
                None => system_downloads_folder()?,
            };
            ensure_folder_exists(&folder)?;
            let exists = |name: &str| {
                let path = format!("{folder}/{name}");
                let ns_path = NSString::from_str(&path);
                // SAFETY: main thread; `path` is a valid file path string.
                unsafe {
                    NSFileManager::defaultManager()
                        .fileExistsAtPath_isDirectory(&ns_path, std::ptr::null_mut())
                }
            };
            let name = uniquify_name(filename, &exists);
            Some(format!("{folder}/{name}"))
        }
    }
}

/// The system downloads directory (first `NSDownloadsDirectory` URL in the
/// user domain), as a path string.
fn system_downloads_folder() -> Option<String> {
    let manager = NSFileManager::defaultManager();
    let urls = manager.URLsForDirectory_inDomains(
        objc2_foundation::NSSearchPathDirectory::DownloadsDirectory,
        objc2_foundation::NSSearchPathDomainMask::UserDomainMask,
    );
    urls.firstObject()
        .and_then(|url| url.path())
        .map(|path| path.to_string())
}

/// Ensure `folder` exists, creating intermediate directories as needed.
/// `None` when creation fails (the download is then cancelled with
/// `NoDestination`).
fn ensure_folder_exists(folder: &str) -> Option<()> {
    let manager = NSFileManager::defaultManager();
    let ns_path = NSString::from_str(folder);
    // SAFETY: main thread; valid path; `is_directory` out-param may be null.
    let exists = unsafe { manager.fileExistsAtPath_isDirectory(&ns_path, std::ptr::null_mut()) };
    if exists {
        return Some(());
    }
    let url = NSURL::fileURLWithPath_relativeToURL(&ns_path, None);
    // SAFETY: main thread; valid URL; no attributes.
    unsafe {
        manager
            .createDirectoryAtURL_withIntermediateDirectories_attributes_error(&url, true, None)
            .ok()
    }
}
