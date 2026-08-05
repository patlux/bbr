//! The concrete [`WebEngine`] implementation for macOS: owns one WKWebView
//! (Phase 1), the profile → data-store map, and the host bridge (geometry +
//! focus). All trait methods must run on the main thread; a debug assertion
//! enforces it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSString, NSURLRequest, NSURL, NSUUID};
use objc2_web_kit::{WKWebView, WKWebViewConfiguration, WKWebsiteDataStore};

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebRect, WebViewId};

use crate::delegate::{self, WebViewDelegate};
use crate::host::HostBridge;

/// Options that change engine behaviour; populated by the composition root.
#[derive(Debug, Clone, Default)]
pub struct EngineOptions {
    /// Use a non-persistent `WKWebsiteDataStore` per profile (E2E hermeticity).
    /// When false, each profile maps to a deterministic persistent
    /// identifier-backed store.
    pub non_persistent: bool,
}

/// The macOS WebKit engine.
pub struct WebEngineWk {
    host: HostBridge,
    options: EngineOptions,

    /// The id of the Phase-1 web view (the active tab's content slot).
    web_view_id: WebViewId,
    /// Phase 1: exactly one live web view.
    web_view: Option<Retained<WKWebView>>,
    /// The delegate must outlive the WKWebView (WebKit does not retain it).
    delegate: Option<Retained<WebViewDelegate>>,
    /// Shared with the delegate; replaced by [`WebEngine::set_events`].
    sink: delegate::EventSink,

    /// Profile → website data store cache.
    stores: HashMap<ProfileId, Retained<WKWebsiteDataStore>>,

    /// The last slot rect published by GPUI layout; used to frame the view
    /// at creation and to avoid redundant `setFrame:` calls.
    last_rect: WebRect,
    /// True once `attach_slot` published a non-trivial rect.
    rect_published: bool,
}

impl WebEngineWk {
    /// Attach the engine to the GPUI root view.
    ///
    /// # Safety
    ///
    /// `ns_view` must be a valid, still-alive `NSView*` (the GPUI root view
    /// captured from `window.window_handle()` on the main thread). The
    /// pointer is retained; the accompanying `MainThreadMarker` is the
    /// compile-time proof that we are on the main thread.
    pub unsafe fn attach(
        mtm: MainThreadMarker,
        ns_view: *mut std::ffi::c_void,
        web_view_id: WebViewId,
        options: EngineOptions,
    ) -> Self {
        let mut host = HostBridge::new();
        unsafe { host.attach_root_view(mtm, ns_view) };

        WebEngineWk {
            host,
            options,
            web_view_id,
            web_view: None,
            delegate: None,
            sink: Rc::new(RefCell::new(None)),
            stores: HashMap::new(),
            last_rect: WebRect::default(),
            rect_published: false,
        }
    }

    /// True when a live WKWebView is hosted (diagnostics/E2E snapshot).
    pub fn web_view_hosted(&self) -> bool {
        self.web_view.is_some()
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebEngineWk used off the main thread — all WKWebView/AppKit calls must run on the main thread"
        );
    }

    fn data_store(&mut self, profile: ProfileId) -> Retained<WKWebsiteDataStore> {
        if let Some(store) = self.stores.get(&profile) {
            return store.clone();
        }
        let Some(mtm) = MainThreadMarker::new() else {
            panic!("data_store called off main thread");
        };
        let store = if self.options.non_persistent {
            unsafe { WKWebsiteDataStore::nonPersistentDataStore(mtm) }
        } else {
            // Deterministic identifier-backed persistent store per profile
            // (mirrors the Swift reference's `getDataStore(for:)`).
            let uuid = NSUUID::new();
            unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&uuid, mtm) }
        };
        self.stores.insert(profile, store.clone());
        store
    }

    fn current_frame(&self) -> objc2_foundation::NSRect {
        self.host.frame_rect(self.last_rect)
    }

    /// Load a URL into the web view via `loadRequest:`.
    fn load_url(&self, url: &str) {
        let Some(web_view) = self.web_view.as_deref() else {
            return;
        };
        let ns_url = NSString::from_str(url);
        let Some(url_obj) = NSURL::URLWithString(&ns_url) else {
            log::warn!("cannot parse URL: {url}");
            return;
        };
        let request = NSURLRequest::requestWithURL(&url_obj);
        unsafe {
            web_view.loadRequest(&request);
        }
    }

    /// Push navigation state for the active view (used after actions whose
    /// completion isn't surfaced by a delegate callback).
    fn emit_state_from_view(&self) {
        let Some(web_view) = self.web_view.as_deref() else {
            return;
        };
        let id = self.web_view_id;
        let url = unsafe { web_view.URL() }
            .and_then(|u| u.absoluteString())
            .map(|s| s.to_string());
        let title = unsafe { web_view.title() }.map(|s| s.to_string());
        let can_back = unsafe { web_view.canGoBack() };
        let can_forward = unsafe { web_view.canGoForward() };
        if let Some(sink) = self.sink.borrow_mut().as_mut() {
            if let Some(url) = url {
                sink(WebEvent::UrlChanged(id, url));
            }
            if let Some(title) = title {
                sink(WebEvent::TitleChanged(id, title));
            }
            sink(WebEvent::CanGoBackChanged(id, can_back));
            sink(WebEvent::CanGoForwardChanged(id, can_forward));
            sink(WebEvent::LoadingChanged(id, false));
        }
    }
}

impl WebEngine for WebEngineWk {
    fn attach_slot(&mut self, rect: WebRect, _content_height: f32) {
        self.assert_main_thread();
        if self.rect_published && self.last_rect == rect {
            return; // no layout change → no AppKit traffic (no polling loop)
        }
        self.rect_published = rect.width.max(rect.height) > 0.0;
        self.last_rect = rect;
        if let Some(web_view) = self.web_view.as_deref() {
            let frame = self.current_frame();
            web_view.setFrame(frame);
        }
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        self.assert_main_thread();
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };

        let store = self.data_store(profile);

        // SAFETY: WebKit configuration creation is confined to the AppKit
        // main thread and uses the framework's designated initializer.
        let config = unsafe { WKWebViewConfiguration::new(mtm) };
        unsafe {
            config.setWebsiteDataStore(&store);
        }

        let frame = self.current_frame();
        let web_view = unsafe {
            WKWebView::initWithFrame_configuration(WKWebView::alloc(mtm), frame, &config)
        };

        let delegate = WebViewDelegate::new(mtm, id, self.sink.clone());
        unsafe {
            web_view.setNavigationDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            web_view.setUIDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }
        // Observe `title`: WebKit pushes it asynchronously (after didFinish),
        // so the snapshot at didFinish may be empty. The KVO callback re-emits
        // full navigation state when the title actually lands.
        unsafe {
            delegate::observe_title(&web_view, delegate.as_super());
        }

        if let Some(root) = self.host.root_view() {
            root.addSubview(&web_view);
        }

        self.web_view = Some(web_view);
        self.delegate = Some(delegate);

        if !initial_url.is_empty() {
            self.load_url(initial_url);
        }
    }

    fn destroy_web_view(&mut self, _id: WebViewId) {
        self.assert_main_thread();
        if let Some(web_view) = self.web_view.take() {
            if let Some(delegate) = self.delegate.as_deref() {
                // Remove the KVO observer before the view is deallocated
                // (leaving it registered would raise NSKVODeallocateException).
                unsafe {
                    delegate::unobserve_title(&web_view, delegate.as_super());
                }
            }
            web_view.removeFromSuperview();
        }
        self.delegate = None;
        // NOTE: the store stays cached; profiles are immutable during Phase 1.
    }

    fn show(&mut self, id: WebViewId, visible: bool) {
        self.assert_main_thread();
        if let Some(web_view) = self.web_view.as_deref() {
            web_view.setHidden(!visible);
            if !visible {
                // Dropping keyboard focus when hidden (the view must not keep
                // it after being removed from the flow).
                let mtm = MainThreadMarker::new().expect("main thread");
                self.host.focus_chrome(mtm);
            }
        }
        let _ = id;
    }

    fn action(&mut self, id: WebViewId, action: WebAction) {
        self.assert_main_thread();
        let Some(web_view) = self.web_view.as_deref() else {
            return;
        };
        match action {
            WebAction::Load { url } => self.load_url(&url),
            WebAction::Back => unsafe {
                if web_view.canGoBack() {
                    web_view.goBack();
                }
            },
            WebAction::Forward => unsafe {
                if web_view.canGoForward() {
                    web_view.goForward();
                }
            },
            WebAction::Reload => unsafe {
                web_view.reload();
            },
            WebAction::Stop => unsafe {
                web_view.stopLoading();
                self.emit_state_from_view();
            },
        }
        let _ = id;
    }

    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>) {
        self.assert_main_thread();
        *self.sink.borrow_mut() = Some(sink);
    }

    fn focus_native_web(&mut self) {
        self.assert_main_thread();
        if let Some(web_view) = self.web_view.as_deref() {
            let mtm = MainThreadMarker::new().expect("main thread");
            self.host.focus_web_view(mtm, web_view);
        }
    }

    fn focus_host_chrome(&mut self) {
        self.assert_main_thread();
        let mtm = MainThreadMarker::new().expect("main thread");
        self.host.focus_chrome(mtm);
    }
}
