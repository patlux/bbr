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
use crate::new_window::{extract_context, NewWindowCoordinator};

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
pub type EventSink = Rc<RefCell<Option<Box<dyn FnMut(WebEvent)>>>>;

/// How the delegate reports classified web-error incidents back to the
/// engine (task packet `error-surface-contract`): the native adapter maps
/// WebKit failures onto the merged contract's closed taxonomy, and the app
/// owns the incident-id allocation on top of that classification.
pub type IncidentSink = Rc<RefCell<Option<Box<dyn FnMut(ClassifiedIncident)>>>>;

/// Shared engine-owned delegate handles, cloned into every navigation
/// delegate (task packets `new-window` and `real-favicon-pipeline.md`).
/// Bundled so the delegate constructor stays within the crate's argument
/// budget.
pub struct DelegateHandles {
    /// Shared productive new-window request coordinator.
    pub new_window: NewWindowCoordinator,
    /// Real favicon acquisition host: started after each navigation commit;
    /// emits `FaviconChanged` observations through the shared sink.
    pub favicon_acquisition: crate::favicon_acquisition::FaviconAcquisitionHost,
    /// Blueprint D11: the open camera/microphone requests.
    pub permissions: crate::permission_requests::PermissionRequests,
    /// Blueprint D12: the open client-certificate requests.
    pub client_certificates: crate::client_identities::ClientCertificateRequests,
    /// The open HTTP authentication requests.
    pub http_auth: crate::http_auth::HttpAuthRequests,
}

/// Instance variables for the navigation delegate.
pub struct WebViewDelegateIvars {
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
    /// Shared productive new-window request coordinator.
    pub(crate) new_window: NewWindowCoordinator,
    /// Real favicon acquisition host (task packet
    /// `real-favicon-pipeline.md`): started after each navigation commit;
    /// emits `FaviconChanged` observations through the shared sink.
    pub(crate) favicon_acquisition: crate::favicon_acquisition::FaviconAcquisitionHost,
    /// Blueprint D11: the open camera/microphone requests.
    pub(crate) permissions: crate::permission_requests::PermissionRequests,
    /// Blueprint D12: the open client-certificate requests.
    pub(crate) client_certificates: crate::client_identities::ClientCertificateRequests,
    /// The open HTTP authentication requests.
    pub(crate) http_auth: crate::http_auth::HttpAuthRequests,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = WebViewDelegateIvars]
    #[name = "BBRWebViewDelegate"]
    pub struct WebViewDelegate;

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
            // Blueprint D11: the page's camera/microphone capture state.
            if context == crate::permission_requests::capture_kvo_context() {
                if let Some(web_view) = object.and_then(|o| o.downcast_ref::<WKWebView>()) {
                    let (camera, microphone) =
                        crate::permission_requests::capture_state(web_view);
                    self.emit(WebEvent::MediaCaptureChanged {
                        view: self.id(),
                        camera,
                        microphone,
                    });
                }
                return;
            }
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
            // The initial document of a session committed: the controller
            // marks the session's root settled, so a later Back-at-root
            // content gesture may run the opener-return fallback (N1).
            self.emit(WebEvent::DidCommitNavigation(self.id()));
            self.emit_state(web_view);
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        fn did_finish_navigation(&self, web_view: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit_loading(false);
            self.emit_state(web_view);
            // Real favicon acquisition (task packet
            // `real-favicon-pipeline.md`): the committed document's
            // declared icon candidates are collected in the isolated client
            // content world and the best candidate downloads through the
            // owning profile's data store session. Any failure stays
            // silent — the §7.2 monogram keeps rendering. The acquisition
            // is one per view and asynchronous; a result arrives as a
            // `FaviconChanged` event.
            if let Some(mtm) = MainThreadMarker::new() {
                self.ivars()
                    .favicon_acquisition
                    .start(mtm, self.id(), web_view);
            }
        }

        #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
        fn did_fail_provisional_navigation(
            &self,
            web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.did_fail(web_view, error);
        }

        #[unsafe(method(webView:didFailNavigation:withError:))]
        fn did_fail_navigation(
            &self,
            web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
            error: &objc2_foundation::NSError,
        ) {
            self.did_fail(web_view, error);
        }

        // Blueprint D13: the main frame may only go to schemes BBR renders;
        // any other (`mailto:`, `zoom:`…) is cancelled and handed to the app,
        // which asks before another app opens it. Subframes never trigger
        // it (their other-scheme loads are cancelled quietly).
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        fn decide_policy_for_navigation_action(
            &self,
            _web_view: &WKWebView,
            action: &objc2_web_kit::WKNavigationAction,
            decision_handler: &block2::DynBlock<dyn Fn(objc2_web_kit::WKNavigationActionPolicy)>,
        ) {
            use objc2_web_kit::{WKNavigationActionPolicy as Policy, WKNavigationType};
            // SAFETY: WebKit delivered a live action for the call.
            let (input, button) = unsafe {
                (
                    crate::new_window::classify_input(
                        action.modifierFlags().0,
                        action.buttonNumber(),
                    ),
                    action.buttonNumber(),
                )
            };
            // SAFETY: WebKit delivered a live action for the call.
            let (url, main_frame, link) = unsafe {
                let request = action.request();
                let url = request
                    .URL()
                    .and_then(|url| url.absoluteString())
                    .map(|url| url.to_string())
                    .unwrap_or_default();
                let main_frame = action
                    .targetFrame()
                    .is_none_or(|frame| frame.isMainFrame());
                (url, main_frame, action.navigationType() == WKNavigationType::LinkActivated)
            };
            // A ⌘-click or middle click on any link asks for a background
            // tab after this one — the same request a link that opens a new
            // window makes (`createWebView…` below), so it takes the same
            // coordinator; the page itself stays.
            if link
                && input == bbr_web::new_window::NewWindowInput::Background
                && crate::external_schemes::renders_in_browser(&url)
            {
                decision_handler.call((Policy::Cancel,));
                let context = crate::new_window::NewWindowContext {
                    source: self.id(),
                    source_profile: self.ivars().profile,
                    url,
                    user_initiated: crate::new_window::user_activation(link, button),
                    input,
                    features: Default::default(),
                };
                let _ = self.ivars().new_window.handle(&context);
                return;
            }
            if crate::external_schemes::renders_in_browser(&url) {
                decision_handler.call((Policy::Allow,));
                return;
            }
            decision_handler.call((Policy::Cancel,));
            if main_frame {
                self.emit(WebEvent::ExternalSchemeRequested {
                    view: self.id(),
                    url,
                    user_initiated: link,
                });
            }
        }

        // Blueprint D12: a server asks for a client certificate. The
        // matching Keychain identities (metadata only) go to the app, which
        // answers through the engine. HTTP Basic/Digest/NTLM (origin or
        // proxy) go to the credential sheet (`http_auth`); every other
        // challenge — server trust above all — keeps WebKit's default
        // handling.
        #[unsafe(method(webView:didReceiveAuthenticationChallenge:completionHandler:))]
        fn did_receive_authentication_challenge(
            &self,
            _web_view: &WKWebView,
            challenge: &objc2_foundation::NSURLAuthenticationChallenge,
            completion_handler: &block2::DynBlock<
                dyn Fn(
                    objc2_foundation::NSURLSessionAuthChallengeDisposition,
                    *mut objc2_foundation::NSURLCredential,
                ),
            >,
        ) {
            use objc2_foundation::NSURLSessionAuthChallengeDisposition as Disposition;
            // WebKit's challenge is a forwarding proxy; message the real one.
            let challenge = crate::auth_challenge::real_challenge(challenge);
            let space = challenge.protectionSpace();
            // SAFETY: a static NSString exported by Foundation.
            let client_certificate =
                unsafe { objc2_foundation::NSURLAuthenticationMethodClientCertificate };
            if space.authenticationMethod().to_string() != client_certificate.to_string() {
                let event = crate::http_auth::request(
                    &challenge,
                    self.id(),
                    &self.ivars().http_auth,
                    completion_handler.copy(),
                );
                match event {
                    Some(event) => self.emit(event),
                    None => completion_handler
                        .call((Disposition::PerformDefaultHandling, std::ptr::null_mut())),
                }
                return;
            }
            let (infos, identities) =
                crate::client_identities::identities_for_challenge(space.distinguishedNames());
            if infos.is_empty() {
                // Nothing in the Keychain matches: continue without one.
                completion_handler.call((Disposition::UseCredential, std::ptr::null_mut()));
                return;
            }
            let host = match space.port() {
                0 | 443 => space.host().to_string(),
                port => format!("{}:{port}", space.host()),
            };
            let request = self.ivars().client_certificates.borrow_mut().register(
                self.id(),
                completion_handler.copy(),
                identities,
            );
            self.emit(WebEvent::ClientCertificateRequested {
                view: self.id(),
                request,
                host,
                identities: infos,
            });
        }

        #[unsafe(method(webViewWebContentProcessDidTerminate:))]
        fn web_content_process_did_terminate(&self, _web_view: &WKWebView) {
            self.ivars().navigation_cover.setHidden(true);
            self.emit_termination();
        }
    }

    unsafe impl WKUIDelegate for WebViewDelegate {
        #[unsafe(method_id(webView:createWebViewWithConfiguration:forNavigationAction:windowFeatures:))]
        fn create_web_view(
            &self,
            _web_view: &WKWebView,
            _configuration: &objc2_web_kit::WKWebViewConfiguration,
            action: &objc2_web_kit::WKNavigationAction,
            features: &objc2_web_kit::WKWindowFeatures,
        ) -> Option<Retained<WKWebView>> {
            // SAFETY: WebKit delivered live action/features objects to this
            // main-thread delegate call. The coordinator applies portable
            // policy and publishes a plain delivery to the app/controller.
            let context = unsafe {
                extract_context(
                    action,
                    features,
                    self.ivars().web_view_id,
                    self.ivars().profile,
                )
            };
            let _ = self.ivars().new_window.handle(&context);
            // Controller/session ownership stays outside WebKit: never return
            // a detached or untracked WKWebView from this delegate.
            None
        }

        // JS-dialog boundary (renderer-isolation lane; interaction contract
        // dialog rows): alert/confirm/prompt are auto-dismissed
        // deterministically in v1 — the chrome never blocks on a page
        // dialog and no dialog state survives close/recreate. WebKit
        // documents that omitting these methods behaves as if the user
        // dismissed the panel, but implementing them makes the dismissal
        // deterministic and explicit (a busy/hung renderer can never leave
        // a dialog open over the BBR chrome). There is deliberately NO
        // beforeunload hook: the pinned SDK exposes no public
        // `WKUIDelegate` beforeunload method (the only
        // `runBeforeUnloadConfirmPanel…` binding is the deprecated legacy
        // `WebUIDelegate`, which this project does not use), so chrome-
        // initiated close destroys the WKWebView immediately and never runs
        // page scripts.
        // Blueprint D11: a page asks for the camera and/or microphone. The
        // handler waits in the shared table until the app answers (a
        // remembered decision, or the prompt); destroying the view denies.
        #[unsafe(method(webView:requestMediaCapturePermissionForOrigin:initiatedByFrame:type:decisionHandler:))]
        fn request_media_capture_permission(
            &self,
            _web_view: &WKWebView,
            origin: &objc2_web_kit::WKSecurityOrigin,
            _frame: &objc2_web_kit::WKFrameInfo,
            capture: objc2_web_kit::WKMediaCaptureType,
            decision_handler: &block2::DynBlock<dyn Fn(objc2_web_kit::WKPermissionDecision)>,
        ) {
            use bbr_web::permissions::PermissionCapability;
            use objc2_web_kit::WKMediaCaptureType;
            // SAFETY: WebKit delivered a live origin for the call.
            let origin = unsafe {
                crate::permission_requests::origin_string(
                    &origin.protocol().to_string(),
                    &origin.host().to_string(),
                    origin.port(),
                )
            };
            let capabilities = match capture {
                WKMediaCaptureType::Camera => vec![PermissionCapability::Camera],
                WKMediaCaptureType::Microphone => vec![PermissionCapability::Microphone],
                _ => vec![PermissionCapability::Camera, PermissionCapability::Microphone],
            };
            let request = self
                .ivars()
                .permissions
                .borrow_mut()
                .register(self.id(), decision_handler.copy());
            self.emit(WebEvent::PermissionRequested {
                view: self.id(),
                request,
                origin,
                capabilities,
            });
        }

        #[unsafe(method(webView:runJavaScriptAlertPanelWithMessage:initiatedByFrame:completionHandler:))]
        fn run_javascript_alert_panel(
            &self,
            _web_view: &WKWebView,
            _message: &NSString,
            _frame: &objc2_web_kit::WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn()>,
        ) {
            // Auto-dismiss: answer the completion immediately with no user
            // input (never a blocking wait, never a presentation).
            completion_handler.call(());
        }

        #[unsafe(method(webView:runJavaScriptConfirmPanelWithMessage:initiatedByFrame:completionHandler:))]
        fn run_javascript_confirm_panel(
            &self,
            _web_view: &WKWebView,
            _message: &NSString,
            _frame: &objc2_web_kit::WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn(objc2::runtime::Bool)>,
        ) {
            // Auto-dismiss as Cancel (`NO`): a page `confirm()` never
            // yields a destructive surprise and never blocks the chrome.
            completion_handler.call((objc2::runtime::Bool::NO,));
        }

        #[unsafe(method(webView:runJavaScriptTextInputPanelWithPrompt:defaultText:initiatedByFrame:completionHandler:))]
        fn run_javascript_text_input_panel(
            &self,
            _web_view: &WKWebView,
            _prompt: &NSString,
            _default_text: Option<&NSString>,
            _frame: &objc2_web_kit::WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn(*mut NSString)>,
        ) {
            // Auto-dismiss as Cancel (`nil`): a page `prompt()` never
            // blocks the chrome and never captures input.
            completion_handler.call((std::ptr::null_mut(),));
        }
    }
);

impl WebViewDelegate {
    /// Create a new delegate for `web_view_id` in `profile`.
    pub fn new(
        mtm: MainThreadMarker,
        web_view_id: WebViewId,
        profile: ProfileId,
        sink: EventSink,
        incident_sink: IncidentSink,
        navigation_cover: Retained<NSView>,
        handles: DelegateHandles,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WebViewDelegateIvars {
            web_view_id,
            profile,
            sink,
            incident_sink,
            navigation_cover,
            new_window: handles.new_window,
            favicon_acquisition: handles.favicon_acquisition,
            permissions: handles.permissions,
            client_certificates: handles.client_certificates,
            http_auth: handles.http_auth,
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

    /// Shared navigation-failure handling. A `WKErrorDomain` +
    /// `WKErrorWebContentProcessTerminated` error is the documented error
    /// form of a dead content process: it routes to the same portable
    /// termination authority as `webViewWebContentProcessDidTerminate:` (a
    /// `PageCrashed` event plus the classified `ProcessCrash` incident)
    /// instead of emitting a misleading `NavigationFailed` or inventing a
    /// navigation cause.
    fn did_fail(&self, web_view: &WKWebView, error: &objc2_foundation::NSError) {
        self.ivars().navigation_cover.setHidden(true);
        self.emit_loading(false);
        if is_web_content_process_termination(error) {
            self.emit_termination();
            return;
        }
        self.emit(WebEvent::NavigationFailed(
            self.id(),
            self.error_message(error),
        ));
        self.emit_incident(error, web_view);
    }

    /// Emit the portable termination signal for the bound session: a
    /// `PageCrashed` event plus the classified `ProcessCrash` incident (the
    /// contract's only class for a dead web process; the surface's recovery
    /// — Restore, recreate the session — is derived from that
    /// classification).
    fn emit_termination(&self) {
        self.emit(WebEvent::PageCrashed(self.id()));
        if let Some(sink) = self.ivars().incident_sink.borrow_mut().as_mut() {
            sink(classify(
                self.ivars().profile,
                self.id(),
                map_termination(),
                None,
            ));
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

/// Is this the documented WebKit error form of a terminated content
/// process (`WKErrorDomain` + `WKErrorWebContentProcessTerminated`)?
fn is_web_content_process_termination(error: &objc2_foundation::NSError) -> bool {
    error.domain().to_string() == crate::error_mapping::WK_ERROR_DOMAIN
        && error.code() as i64 == crate::error_mapping::WK_ERROR_WEB_CONTENT_PROCESS_TERMINATED
}
