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
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_web_kit::{WKNavigation, WKNavigationDelegate, WKUIDelegate, WKWebView};

use bbr_web::{WebEvent, WebViewId};

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
