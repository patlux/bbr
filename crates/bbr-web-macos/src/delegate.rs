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
use objc2_app_kit::NSView;
use objc2_foundation::{
    NSKeyValueObservingOptions, NSObject, NSObjectNSKeyValueObserverRegistration, NSObjectProtocol,
    NSString, NSURL,
};
use objc2_web_kit::{WKNavigation, WKNavigationDelegate, WKUIDelegate, WKWebView};

use bbr_web::{ProfileId, WebEvent, WebViewId};

use crate::error_mapping::{classify, map_error, map_termination, ClassifiedIncident};

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

/// How the delegate reports classified web-error incidents back to the
/// engine (task packet `error-surface-contract`): the native adapter maps
/// WebKit failures onto the merged contract's closed taxonomy, and the app
/// owns the incident-id allocation on top of that classification.
pub(crate) type IncidentSink = Rc<RefCell<Option<Box<dyn FnMut(ClassifiedIncident)>>>>;

/// Instance variables for the navigation delegate.
pub(crate) struct WebViewDelegateIvars {
    /// The web view this delegate belongs to (used to key events).
    pub(crate) web_view_id: WebViewId,
    /// The profile this web view was created in (incident scope).
    pub(crate) profile: ProfileId,
    /// Shared with the engine; updated by `set_events`.
    pub(crate) sink: EventSink,
    /// Shared with the engine; updated by `set_incidents`.
    pub(crate) incident_sink: IncidentSink,
    /// Native loading cover above this view's WebKit plane.
    pub(crate) navigation_cover: Retained<NSView>,
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
            self.ivars().navigation_cover.setHidden(false);
            self.emit_loading(true);
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didCommitNavigation:))]
        fn did_commit_navigation(&self, web_view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        fn did_finish_navigation(&self, web_view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit_loading(false);
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
        fn did_fail_provisional_navigation(
            &self,
            web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit_loading(false);
            self.emit(WebEvent::NavigationFailed(
                self.id(),
                self.error_message(error),
            ));
            self.emit_incident(error, web_view);
        }

        #[unsafe(method(webView:didFailNavigation:withError:))]
        fn did_fail_navigation(
            &self,
            web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit_loading(false);
            self.emit(WebEvent::NavigationFailed(
                self.id(),
                self.error_message(error),
            ));
            self.emit_incident(error, web_view);
        }

        #[unsafe(method(webViewWebContentProcessDidTerminate:))]
        fn web_content_process_did_terminate(&self, _web_view: &WKWebView) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit(WebEvent::PageCrashed(self.id()));
            // The renderer-termination path is classified too: the contract's
            // only class for a dead web process is `ProcessCrash`, and the
            // surface's recovery (Restore — recreate the session) is derived
            // from that classification.
            if let Some(sink) = self.ivars().incident_sink.borrow_mut().as_mut() {
                sink(classify(
                    self.ivars().profile,
                    self.id(),
                    map_termination(),
                    None,
                ));
            }
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
    /// Create a new delegate for `web_view_id` in `profile`.
    pub(crate) fn new(
        mtm: MainThreadMarker,
        web_view_id: WebViewId,
        profile: ProfileId,
        sink: EventSink,
        incident_sink: IncidentSink,
        navigation_cover: Retained<NSView>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WebViewDelegateIvars {
            web_view_id,
            profile,
            sink,
            incident_sink,
            navigation_cover,
        });
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

    /// Classify a native `NSError` into the merged contract and emit the
    /// classified incident. Codes the taxonomy cannot name honestly are
    /// dropped (never a guessed cause); the raw `NavigationFailed` event
    /// still flows for legacy navigation-state consumers.
    fn emit_incident(&self, error: &objc2_foundation::NSError, web_view: &WKWebView) {
        let domain = error.domain().to_string();
        let code = error.code() as i64;
        // The failing URL rides in the error's userInfo. WebKit is not
        // consistent about the key or the value type: some failures carry
        // `NSErrorFailingURLStringKey` (an NSString), others carry
        // `NSErrorFailingURLKey` (an NSURL or NSString), and some (a TLS
        // handshake that never establishes) carry neither — the view's own
        // URL is then also uncommitted, so the incident honestly has no
        // retry target and no readable origin. Try every source in order
        // so the surface and the retry action use the real failing URL
        // whenever the engine reported one.
        let failing_url = error
            .userInfo()
            .objectForKey(&NSString::from_str(crate::error_mapping::FAILING_URL_KEY))
            .and_then(|value| value.downcast::<NSString>().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                error
                    .userInfo()
                    .objectForKey(&NSString::from_str(crate::error_mapping::FAILING_NSURL_KEY))
                    .and_then(|value| {
                        if let Some(url) = value
                            .clone()
                            .downcast::<NSURL>()
                            .ok()
                            .and_then(|url| url.absoluteString())
                        {
                            return Some(url.to_string());
                        }
                        value.downcast::<NSString>().ok().map(|s| s.to_string())
                    })
            })
            .or_else(|| {
                unsafe { web_view.URL() }
                    .and_then(|u| u.absoluteString())
                    .map(|s| s.to_string())
            });
        let Some(outcome) = map_error(&domain, code) else {
            return;
        };
        if let Some(sink) = self.ivars().incident_sink.borrow_mut().as_mut() {
            sink(classify(
                self.ivars().profile,
                self.id(),
                outcome,
                failing_url.as_deref(),
            ));
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
        let domain = error.domain().to_string();
        // Diagnostic evidence for logs/contract tests only; the UI surface
        // never renders this string (it presents the classified incident).
        if desc.is_empty() {
            format!("{domain} error {code}")
        } else {
            format!("{desc} ({domain} {code})")
        }
    }
}
