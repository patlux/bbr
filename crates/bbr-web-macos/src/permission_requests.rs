//! Blueprint D11: WebKit's camera/microphone permission requests and the
//! pages' live capture state.
//!
//! `webView:requestMediaCapturePermissionForOrigin:…` hands the delegate a
//! decision handler; the table keeps a copy under a fresh request id until
//! the app answers through [`bbr_web::WebEngine::decide_permission`].
//! Destroying a view denies its open requests (WebKit expects every handler
//! to be called). The `cameraCaptureState` / `microphoneCaptureState` KVO
//! reports the in-use state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;

use block2::RcBlock;
use objc2::ClassType;
use objc2_foundation::{
    NSKeyValueObservingOptions, NSObject, NSObjectNSKeyValueObserverRegistration, NSString,
};
use objc2_web_kit::{WKMediaCaptureState, WKPermissionDecision, WKWebView};

use bbr_web::WebViewId;

type Handler = RcBlock<dyn Fn(WKPermissionDecision)>;

/// The open requests, shared by every delegate and the engine.
#[derive(Default)]
pub struct PermissionTable {
    next: u64,
    pending: HashMap<u64, (WebViewId, Handler)>,
}

pub type PermissionRequests = Rc<RefCell<PermissionTable>>;

impl PermissionTable {
    /// Keep `handler` and return the request's id.
    pub(crate) fn register(&mut self, view: WebViewId, handler: Handler) -> u64 {
        self.next += 1;
        self.pending.insert(self.next, (view, handler));
        self.next
    }

    /// Answer one request (unknown ids are no-ops).
    pub(crate) fn decide(&mut self, request: u64, allow: bool) {
        if let Some((_, handler)) = self.pending.remove(&request) {
            handler.call((if allow {
                WKPermissionDecision::Grant
            } else {
                WKPermissionDecision::Deny
            },));
        }
    }

    /// Deny every open request of a view that goes away.
    pub(crate) fn deny_view(&mut self, view: WebViewId) {
        let requests: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, (owner, _))| *owner == view)
            .map(|(request, _)| *request)
            .collect();
        for request in requests {
            self.decide(request, false);
        }
    }
}

/// `scheme://host[:port]` of a WebKit security origin.
pub(crate) fn origin_string(protocol: &str, host: &str, port: isize) -> String {
    if port > 0 {
        format!("{protocol}://{host}:{port}")
    } else {
        format!("{protocol}://{host}")
    }
}

const CAMERA_KEY_PATH: &str = "cameraCaptureState";
const MICROPHONE_KEY_PATH: &str = "microphoneCaptureState";

static CAPTURE_KVO_CONTEXT: u8 = 0;

pub(crate) fn capture_kvo_context() -> *mut c_void {
    (&CAPTURE_KVO_CONTEXT as *const u8).cast_mut().cast()
}

/// Observe the view's capture state with `observer` (the delegate).
///
/// # Safety
/// Main thread, a live `web_view`; pair with [`unobserve_capture`] before
/// the view is released (the observer must outlive the observation).
pub(crate) unsafe fn observe_capture(web_view: &WKWebView, observer: &NSObject) {
    let web_obj = web_view.as_super().as_super().as_super();
    for key in [CAMERA_KEY_PATH, MICROPHONE_KEY_PATH] {
        let key = NSString::from_str(key);
        unsafe {
            web_obj.addObserver_forKeyPath_options_context(
                observer,
                &key,
                NSKeyValueObservingOptions::New,
                capture_kvo_context(),
            );
        }
    }
}

/// Remove the observation registered by [`observe_capture`].
///
/// # Safety
/// Same preconditions as [`observe_capture`], paired one-to-one.
pub(crate) unsafe fn unobserve_capture(web_view: &WKWebView, observer: &NSObject) {
    let web_obj = web_view.as_super().as_super().as_super();
    for key in [CAMERA_KEY_PATH, MICROPHONE_KEY_PATH] {
        let key = NSString::from_str(key);
        unsafe {
            web_obj.removeObserver_forKeyPath_context(observer, &key, capture_kvo_context());
        }
    }
}

/// The view's (camera, microphone) in-use state (active or muted).
pub(crate) fn capture_state(web_view: &WKWebView) -> (bool, bool) {
    // SAFETY: main thread (KVO and delegate callbacks run on it).
    let (camera, microphone) = unsafe {
        (
            web_view.cameraCaptureState(),
            web_view.microphoneCaptureState(),
        )
    };
    (
        camera != WKMediaCaptureState::None,
        microphone != WKMediaCaptureState::None,
    )
}

#[cfg(test)]
mod tests {
    use super::origin_string;

    #[test]
    fn origins_carry_a_port_only_when_webkit_names_one() {
        assert_eq!(
            origin_string("https", "meet.example.com", 0),
            "https://meet.example.com"
        );
        assert_eq!(
            origin_string("http", "127.0.0.1", 8080),
            "http://127.0.0.1:8080"
        );
    }
}
