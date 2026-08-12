//! The bounded download delegate (task packet `real-favicon-pipeline.md`).
//!
//! Enforces the size bounds (declared length and accumulated bytes), the
//! status bound (2xx only), and the deadline (the session's own timeouts),
//! then emits the acquired bytes or stays silent. All callbacks arrive on
//! the main thread (the session's delegate queue is the main queue); the
//! class is `AnyThread` because the protocol requires `Send + Sync`, and
//! its ivars are chosen to satisfy that.

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use objc2::{define_class, AnyThread, ClassType, DefinedClass};
use objc2_foundation::{
    NSData, NSError, NSHTTPURLResponse, NSObject, NSObjectProtocol, NSURLResponse, NSURLSession,
    NSURLSessionDataDelegate, NSURLSessionDataTask, NSURLSessionDelegate,
    NSURLSessionResponseDisposition, NSURLSessionTaskDelegate,
};

use bbr_web::WebViewId;

use super::bounds::FAVICON_MAX_BYTES;
use super::HostRef;

/// Atomically claim the exactly-once terminal path. Kept separate from the
/// Objective-C callback so duplicate callback delivery has a pure regression
/// test and can never release one concurrency slot twice.
fn claim_terminal(terminated: &AtomicBool) -> bool {
    !terminated.swap(true, Ordering::Relaxed)
}

/// Instance variables for one bounded download (all `Send + Sync`: the
/// `NSURLSessionDataDelegate` protocol requires it; the host is reached
/// through the numeric [`HostRef`] registry). `pub` so the generated class
/// satisfies the ivar-visibility requirement; the fields stay crate-private
/// (constructed by the download-start module).
pub struct FaviconDownloadIvars {
    pub(crate) view: WebViewId,
    pub(crate) generation: u64,
    pub(crate) host: HostRef,
    /// Accumulated payload bytes (bounded by [`FAVICON_MAX_BYTES`]).
    pub(crate) bytes: Mutex<Vec<u8>>,
    /// The terminal path ran exactly once (task cancellation also lands
    /// here via `didCompleteWithError`).
    pub(crate) terminated: AtomicBool,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = AnyThread]
    #[ivars = FaviconDownloadIvars]
    #[name = "BBRFaviconDownloadDelegate"]
    pub struct FaviconDownloadDelegate;

    unsafe impl NSObjectProtocol for FaviconDownloadDelegate {}

    unsafe impl NSURLSessionDelegate for FaviconDownloadDelegate {}

    unsafe impl NSURLSessionTaskDelegate for FaviconDownloadDelegate {
        // `URLSession:task:didCompleteWithError:` is declared on
        // `NSURLSessionTaskDelegate` (inherited by the data-delegate
        // protocol), so it must be registered against this protocol block —
        // the class-registration debug check verifies the selector against
        // the declaring protocol's method list.
        #[unsafe(method(URLSession:task:didCompleteWithError:))]
        fn url_session_task_did_complete_with_error(
            &self,
            _session: &NSURLSession,
            _task: &NSURLSessionDataTask,
            error: Option<&NSError>,
        ) {
            if self.ivars().terminated.load(Ordering::Relaxed) {
                return;
            }
            if error.is_some() {
                // A cancelled (over-bound), failed, or timed-out download is
                // silent.
                self.terminate(None);
                return;
            }
            let bytes = self
                .ivars()
                .bytes
                .lock()
                .expect("favicon bytes lock")
                .clone();
            if bytes.is_empty() {
                self.terminate(None);
                return;
            }
            self.terminate(Some(bytes));
        }
    }

    unsafe impl NSURLSessionDataDelegate for FaviconDownloadDelegate {
        #[unsafe(method(URLSession:dataTask:didReceiveResponse:completionHandler:))]
        fn url_session_data_task_did_receive_response_completion_handler(
            &self,
            _session: &NSURLSession,
            _task: &NSURLSessionDataTask,
            response: &NSURLResponse,
            completion_handler: &block2::DynBlock<dyn Fn(NSURLSessionResponseDisposition)>,
        ) {
            if self.ivars().terminated.load(Ordering::Relaxed) {
                completion_handler.call((NSURLSessionResponseDisposition::Cancel,));
                return;
            }
            // Only 2xx responses carry a usable favicon; a declared length
            // beyond the bound is rejected up front.
            let status_ok = response
                .as_super()
                .as_super()
                .downcast_ref::<NSHTTPURLResponse>()
                .is_none_or(|http| (200..300).contains(&http.statusCode()));
            let expected = response.expectedContentLength();
            if !status_ok || expected > FAVICON_MAX_BYTES as i64 {
                self.terminate(None);
                completion_handler.call((NSURLSessionResponseDisposition::Cancel,));
                return;
            }
            completion_handler.call((NSURLSessionResponseDisposition::Allow,));
        }

        #[unsafe(method(URLSession:dataTask:didReceiveData:))]
        fn url_session_data_task_did_receive_data(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionDataTask,
            data: &NSData,
        ) {
            if self.ivars().terminated.load(Ordering::Relaxed) {
                return;
            }
            let length = data.length();
            if length == 0 {
                return;
            }
            let mut bytes = self.ivars().bytes.lock().expect("favicon bytes lock");
            if bytes.len().saturating_add(length) > FAVICON_MAX_BYTES {
                drop(bytes);
                // Over-bound: cancel the task; `didCompleteWithError` then
                // lands and the terminal path releases the slot silently.
                task.cancel();
                return;
            }
            // SAFETY: `data` is alive on the main thread; `buffer` has
            // exactly `length` bytes.
            let mut buffer = vec![0u8; length];
            unsafe {
                data.getBytes_length(
                    NonNull::new(buffer.as_mut_ptr() as *mut std::ffi::c_void).unwrap(),
                    length,
                );
            }
            bytes.extend_from_slice(&buffer);
        }
    }
);

impl FaviconDownloadDelegate {
    /// The exactly-once terminal path: emit the acquired bytes (when the
    /// view's acquisition is still current), release the concurrency slot,
    /// and start the next queued download. Resolves the host through the
    /// [`HostRef`] registry; a host that is already gone (engine torn down)
    /// means the observation is silently dropped.
    fn terminate(&self, bytes: Option<Vec<u8>>) {
        if !claim_terminal(&self.ivars().terminated) {
            return;
        }
        let view = self.ivars().view;
        let generation = self.ivars().generation;
        if let Some(host) = self.ivars().host.host() {
            host.complete_download(view, generation, bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_path_can_be_claimed_exactly_once() {
        let terminated = AtomicBool::new(false);
        assert!(claim_terminal(&terminated));
        assert!(!claim_terminal(&terminated));
        assert!(!claim_terminal(&terminated));
    }
}
