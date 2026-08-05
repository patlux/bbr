//! Objective-C delegate classes for the WKWebView.
//!
//! All protocol methods in `WKNavigationDelegate`/`WKUIDelegate` are
//! `#[optional]`, so we only implement the ones BBR needs for Phase 1's
//! observable navigation state. Nothing here holds a strong reference back
//! to the engine: delegates carry a `WebViewId` and a shared event sink
//! (`Rc<RefCell<Option<Box<dyn FnMut(WebEvent)>>>>`) that the engine owns.
//! WKWebView does **not** retain its delegates, so the engine must keep the
//! [`Retained<WebViewDelegate>`] alive for the view's lifetime.

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{define_class, msg_send, ClassType, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{
    NSKeyValueObservingOptions, NSObject, NSObjectNSKeyValueObserverRegistration, NSObjectProtocol,
    NSString,
};
use objc2_web_kit::{WKNavigation, WKNavigationDelegate, WKUIDelegate, WKWebView};

use bbr_web::{WebEvent, WebViewId};

/// The `title` key path observed on the WKWebView (see [`WebViewDelegate::new`]).
const TITLE_KEY_PATH: &str = "title";

/// Opaque KVO context for the `title` observation. Never dereferenced; used
/// only to scope the delegate's `observeValueForKeyPath:ofObject:change:context:`
/// callback to the observation we registered.
static TITLE_KVO_CONTEXT: u8 = 0;

fn title_kvo_context() -> *mut c_void {
    (&TITLE_KVO_CONTEXT as *const u8).cast_mut().cast()
}

/// Register KVO observation of the WKWebView `title` property with `observer`
/// (the navigation delegate). Registering with `Initial` yields one immediate
/// callback with the current title, so the shell learns the startup state even
/// when the page has already begun loading.
///
/// # Safety
///
/// Must be called on the main thread with a live `web_view` and a retained
/// `observer` (the delegate). The observer is not retained by KVO, so it must
/// outlive `web_view`.
pub(crate) unsafe fn observe_title(web_view: &WKWebView, observer: &NSObject) {
    let key = NSString::from_str(TITLE_KEY_PATH);
    let web_obj = web_view.as_super().as_super().as_super(); // &NSObject
    unsafe {
        web_obj.addObserver_forKeyPath_options_context(
            observer,
            &key,
            NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Initial,
            title_kvo_context(),
        );
    }
}

/// Remove the KVO `title` observation registered by [`observe_title`]. Must be
/// called (with the same observer and context) before `web_view` is deallocated.
///
/// # Safety
///
/// Same preconditions as [`observe_title`]; must pair one-to-one with a prior
/// successful [`observe_title`] call on the same view/observer.
pub(crate) unsafe fn unobserve_title(web_view: &WKWebView, observer: &NSObject) {
    let key = NSString::from_str(TITLE_KEY_PATH);
    let web_obj = web_view.as_super().as_super().as_super(); // &NSObject
    unsafe {
        web_obj.removeObserver_forKeyPath_context(observer, &key, title_kvo_context());
    }
}

/// How the delegate reports navigation observations back to the engine.
pub(crate) type EventSink = Rc<RefCell<Option<Box<dyn FnMut(WebEvent)>>>>;

/// Instance variables for the navigation delegate.
pub(crate) struct WebViewDelegateIvars {
    /// The web view this delegate belongs to (used to key events).
    pub(crate) web_view_id: WebViewId,
    /// Shared with the engine; updated by `set_events`.
    pub(crate) sink: EventSink,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = WebViewDelegateIvars]
    #[name = "BBRWebViewDelegate"]
    pub(crate) struct WebViewDelegate;

    unsafe impl NSObjectProtocol for WebViewDelegate {}

    impl WebViewDelegate {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value_for_key_path(
            &self,
            _key_path: Option<&NSString>,
            object: Option<&AnyObject>,
            _change: Option<&AnyObject>,
            context: *mut c_void,
        ) {
            // Scope to our own `title` observation; ignore everyone else's.
            if context != title_kvo_context() {
                return;
            }
            // WebKit pushes the `title` property asynchronously, so it may
            // not be populated at `didFinishNavigation`. Re-emit the full
            // navigation state whenever the title actually lands.
            if let Some(web_view) = object.and_then(|o| o.downcast_ref::<WKWebView>()) {
                self.emit_state(web_view);
            }
        }
    }

    unsafe impl WKNavigationDelegate for WebViewDelegate {
        #[unsafe(method(webView:didStartProvisionalNavigation:))]
        fn did_start_provisional_navigation(
            &self,
            web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
        ) {
            self.emit_loading(true);
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didCommitNavigation:))]
        fn did_commit_navigation(&self, web_view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        fn did_finish_navigation(&self, web_view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.emit_loading(false);
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
        fn did_fail_provisional_navigation(
            &self,
            _web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.emit_loading(false);
            self.emit(WebEvent::NavigationFailed(
                self.id(),
                self.error_message(error),
            ));
        }

        #[unsafe(method(webView:didFailNavigation:withError:))]
        fn did_fail_navigation(
            &self,
            _web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.emit_loading(false);
            self.emit(WebEvent::NavigationFailed(
                self.id(),
                self.error_message(error),
            ));
        }

        #[unsafe(method(webViewWebContentProcessDidTerminate:))]
        fn web_content_process_did_terminate(&self, _web_view: &WKWebView) {
            self.emit(WebEvent::PageCrashed(self.id()));
        }
    }

    unsafe impl WKUIDelegate for WebViewDelegate {
        #[unsafe(method_id(webView:createWebViewWithConfiguration:forNavigationAction:windowFeatures:))]
        fn create_web_view(
            &self,
            _web_view: &WKWebView,
            _configuration: &objc2_web_kit::WKWebViewConfiguration,
            _action: &objc2_web_kit::WKNavigationAction,
            _features: &objc2_web_kit::WKWindowFeatures,
        ) -> Option<Retained<WKWebView>> {
            // Phase 1: suppress new-window popups so test navigation is fully
            // deterministic. Wired popups land in a later phase.
            None
        }
    }
);

impl WebViewDelegate {
    /// Create a new delegate for `web_view_id`.
    pub(crate) fn new(
        mtm: MainThreadMarker,
        web_view_id: WebViewId,
        sink: EventSink,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WebViewDelegateIvars { web_view_id, sink });
        // SAFETY: `init` is the NSObject initializer; we are on the main thread.
        unsafe { msg_send![super(this), init] }
    }

    fn id(&self) -> WebViewId {
        self.ivars().web_view_id
    }

    fn emit(&self, event: WebEvent) {
        if let Some(sink) = self.ivars().sink.borrow_mut().as_mut() {
            sink(event);
        }
    }

    fn emit_loading(&self, loading: bool) {
        self.emit(WebEvent::LoadingChanged(self.id(), loading));
    }

    /// Emit url/title/canGoBack/canGoForward from the current view state.
    fn emit_state(&self, web_view: &WKWebView) {
        let id = self.id();
        // SAFETY: main thread, valid view.
        let url = unsafe { web_view.URL() }
            .and_then(|u| u.absoluteString())
            .map(|s| s.to_string());
        let title = unsafe { web_view.title() }.map(|s| s.to_string());
        if let Some(url) = url {
            self.emit(WebEvent::UrlChanged(id, url));
        }
        if let Some(title) = title {
            self.emit(WebEvent::TitleChanged(id, title));
        }
        // It is safe to read these on the main thread.
        let can_back = unsafe { web_view.canGoBack() };
        let can_forward = unsafe { web_view.canGoForward() };
        self.emit(WebEvent::CanGoBackChanged(id, can_back));
        self.emit(WebEvent::CanGoForwardChanged(id, can_forward));
    }

    fn error_message(&self, error: &objc2_foundation::NSError) -> String {
        let desc = error.localizedDescription().to_string();
        let code = error.code();
        if desc.is_empty() {
            format!("webkit error code {code}")
        } else {
            desc
        }
    }
}
