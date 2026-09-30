//! The concrete [`WebEngine`] implementation for macOS.
//!
//! The engine owns an ID-addressed registry of native sessions. Each session
//! keeps its WKWebView, retained delegate, profile identity, and visibility
//! together so create/show/action/focus/destroy cannot accidentally target a
//! different tab. All methods run on the AppKit main thread.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSColor, NSView};
use objc2_web_kit::{WKWebView, WKWebViewConfiguration, WKWebsiteDataStore};

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebRect, WebViewId};

use crate::content_gesture::{ContentGestureHost, ContentGestureSink};
use crate::delegate::{self, WebViewDelegate};
use crate::error_mapping::ClassifiedIncident;
use crate::host::HostBridge;
use crate::native_session::{self, NativeWebSession};
use crate::navigation_cover::NavigationCover;
use crate::new_window::NewWindowCoordinator;

/// Options that change engine behaviour; populated by the composition root.
#[derive(Debug, Clone, Default)]
pub struct EngineOptions {
    /// Use a non-persistent `WKWebsiteDataStore` per profile (E2E hermeticity).
    /// When false, each profile maps to a deterministic persistent store id.
    pub non_persistent: bool,
}

/// The macOS WebKit engine.
pub struct WebEngineWk {
    host: HostBridge,
    options: EngineOptions,
    sessions: HashMap<WebViewId, NativeWebSession>,
    /// Shared with every delegate; replaced by [`WebEngine::set_events`].
    sink: delegate::EventSink,
    /// Shared with every delegate; replaced by [`WebEngineWk::set_incidents`].
    incident_sink: delegate::IncidentSink,
    /// Shared synchronous portable policy and app-delivery seam for every
    /// WKUI delegate.
    new_window: NewWindowCoordinator,
    /// Profile → website data store cache. Stores outlive individual views.
    stores: HashMap<ProfileId, Retained<WKWebsiteDataStore>>,
    /// Per-view geometry cache: the last host-layout rect published for each
    /// live view. An unchanged rect for a view produces no repeated
    /// `setFrame:` call; destroying a view removes its cached rect; unknown
    /// ids are no-ops.
    geometry: HashMap<WebViewId, WebRect>,
    /// The one native context-menu bridge (task packet
    /// `bbr-context-menus-over-web`): observes right-clicks over the live
    /// web views, classifies WebKit's own menu (page vs element), and
    /// requests the BBR menu through the portal for page-owned clicks.
    menu_bridge: Option<Box<crate::menu_adapter::NativeContextMenuBridge>>,
    /// The one native content-gesture bridge (task packet
    /// `web-content-navigation-gestures`): observes scroll-wheel events over
    /// the live web views and reports at most one logical
    /// `ContentGestureCommit` per committed horizontal gesture. A pure
    /// observer — native scrolling/selection/zoom are never disabled.
    content_gesture: ContentGestureHost,
    /// Per-view bounded liveness probes (renderer-isolation lane): the
    /// client-world `evaluateJavaScript` probe plus the per-view deadline
    /// timer. Shares the engine's event sink, so `UnresponsiveChanged`
    /// observations flow through the same transport as every other event.
    pub(crate) probe: crate::responsiveness::RendererProbeHost,
    /// Real favicon acquisition (task packet `real-favicon-pipeline.md`):
    /// after each navigation commit the isolated content-world collector
    /// reads the page's icon candidates and the bounded profile-bound
    /// download emits `FaviconChanged` observations through the shared
    /// engine sink. Silent on any failure (the §7.2 monogram stays).
    pub(crate) favicon_acquisition: crate::favicon_acquisition::FaviconAcquisitionHost,
    /// Blueprint D11: the open camera/microphone requests.
    permissions: crate::permission_requests::PermissionRequests,
    /// Blueprint D12: the open client-certificate requests.
    client_certificates: crate::client_identities::ClientCertificateRequests,
    /// Blueprint D24: the per-view media probes.
    media: crate::media_adapter::MediaHost,
    /// Blueprint D19: the further windows' roots and each session's window.
    roots: crate::window_roots::WindowRoots,
    /// The content-gesture sink every window's bridge shares.
    gesture_sink: ContentGestureSink,
}

impl WebEngineWk {
    /// Attach the engine to the GPUI root view.
    ///
    /// # Safety
    ///
    /// `ns_view` must be a valid, still-alive `NSView*` captured from the GPUI
    /// window on the main thread. The bridge retains it.
    pub unsafe fn attach(
        mtm: MainThreadMarker,
        ns_view: *mut std::ffi::c_void,
        options: EngineOptions,
    ) -> Self {
        let mut host = HostBridge::new();
        unsafe { host.attach_root_view(mtm, ns_view) };
        let sink = Rc::new(RefCell::new(None));
        let menu_bridge = unsafe {
            crate::menu_adapter::NativeContextMenuBridge::attach(mtm, ns_view, sink.clone())
        };
        let content_gesture_reporter: ContentGestureSink = Rc::new(RefCell::new(None));
        let gesture_sink = content_gesture_reporter.clone();
        let content_gesture = ContentGestureHost::attach(mtm, ns_view, content_gesture_reporter);
        let probe = crate::responsiveness::RendererProbeHost::new(sink.clone());
        let favicon_acquisition =
            crate::favicon_acquisition::FaviconAcquisitionHost::new(sink.clone());
        let media = crate::media_adapter::MediaHost::new(sink.clone());
        Self {
            host,
            options,
            sessions: HashMap::new(),
            // The event sink is created before the menu bridge because the
            // bridge shares it; the incident sink is independent.
            sink,
            incident_sink: Rc::new(RefCell::new(None)),
            new_window: NewWindowCoordinator::new(),
            stores: HashMap::new(),
            geometry: HashMap::new(),
            menu_bridge,
            content_gesture,
            probe,
            favicon_acquisition,
            media,
            roots: Default::default(),
            gesture_sink,
            permissions: Default::default(),
            client_certificates: Default::default(),
        }
    }

    /// Blueprint D19: attach a further browser window's GPUI root view as
    /// window `window` (the app's window key).
    ///
    /// # Safety
    ///
    /// `ns_view` must be that window's live GPUI root `NSView*`, captured
    /// on the main thread.
    pub unsafe fn attach_window(
        &mut self,
        mtm: MainThreadMarker,
        window: u64,
        ns_view: *mut std::ffi::c_void,
    ) {
        unsafe {
            self.roots.attach(
                mtm,
                window,
                ns_view,
                self.sink.clone(),
                self.gesture_sink.clone(),
            )
        };
    }

    /// Blueprint D19: forget a closed window's root (its sessions moved or
    /// were hidden first).
    pub fn detach_window(&mut self, window: u64) {
        self.roots.detach(window);
    }

    /// Blueprint D19: the window whose root hosts `view` (0: the first
    /// window's).
    pub fn window_of_view(&self, view: WebViewId) -> u64 {
        self.roots.window_of(view)
    }

    /// Blueprint D19: the window whose chrome `focus_host_chrome` serves.
    pub fn set_key_window(&mut self, window: u64) {
        self.roots.key_window = window;
    }

    /// The host of `window` (the first window's for an unknown one).
    fn host_of_window(&self, window: u64) -> &HostBridge {
        self.roots
            .root(window)
            .map_or(&self.host, |root| &root.host)
    }

    /// The host of the window `id` lives in.
    fn host_of_view(&self, id: WebViewId) -> &HostBridge {
        self.host_of_window(self.roots.window_of(id))
    }

    /// True when at least one live WKWebView is hosted.
    pub fn web_view_hosted(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Profile identity of a live session (native integration diagnostics).
    pub fn session_profile(&self, id: WebViewId) -> Option<ProfileId> {
        self.sessions.get(&id).map(|session| session.profile)
    }

    /// The native content-card clip of `id`'s WKWebView (DESIGN §2): its
    /// backing layer's corner radius, `None` without a live session.
    pub fn web_view_corner_radius(&self, id: WebViewId) -> Option<f64> {
        self.assert_main_thread();
        let session = self.sessions.get(&id)?;
        crate::card_clip::corner_radius(&session.web_view)
    }

    /// Whether the dark native cover is currently masking WebKit's
    /// provisional-navigation plane for `id`.
    pub fn navigation_cover_visible(&self, id: WebViewId) -> bool {
        self.assert_main_thread();
        self.sessions
            .get(&id)
            .is_some_and(|session| !session.navigation_cover.isHidden())
    }

    /// The index of `id`'s WKWebView among the host root view's subviews
    /// (AppKit draws subviews front-to-back, so a higher index is above a
    /// lower one). This is the truthful native z-order evidence for the Peek
    /// preview: while a preview is open its index must exceed its covered
    /// origin's index. `None` for an unknown id or a detached root.
    pub fn subview_index(&self, id: WebViewId) -> Option<usize> {
        self.assert_main_thread();
        let session = self.sessions.get(&id)?;
        let root = self.host_of_view(id).root_view()?;
        let target = objc2::rc::Retained::as_ptr(&session.web_view) as *const std::ffi::c_void;
        // `subviews()` returns the live children of the retained root view,
        // which includes `session.web_view`, and the array is not mutated
        // while we read it. The `NSEnumerator` feature is not enabled in
        // this workspace, so the array is read by index.
        let subviews = root.subviews();
        let count = subviews.count();
        for index in 0..count {
            // SAFETY: `index < count` on the live, unmutated array.
            let item = unsafe { subviews.objectAtIndex_unchecked(index) };
            let item_ptr = item as *const _ as *const std::ffi::c_void;
            if item_ptr == target {
                return Some(index);
            }
        }
        None
    }

    /// A retained clone of the live `WKWebView` for `id` — the find
    /// adapter's bind point. Never creates, moves, or reselects a session:
    /// the caller binds one adapter per active view and the adapter only
    /// scripts the existing view. `None` for an unknown/destroyed id.
    pub fn web_view(&self, id: WebViewId) -> Option<Retained<WKWebView>> {
        self.assert_main_thread();
        self.sessions
            .get(&id)
            .map(|session| session.web_view.clone())
    }

    /// The browser window's content rect in GPUI host coordinates (points,
    /// top-left origin) — see [`crate::native_session::window_content_rect`].
    /// `None` when no live session is hosted (no window exists yet).
    pub fn window_content_rect(&self, id: WebViewId) -> Option<[f64; 4]> {
        self.assert_main_thread();
        let web_view = self.web_view(id)?;
        native_session::window_content_rect(&web_view)
    }

    /// The live safe-titlebar layout guide (packet
    /// `titlebar-navigation-controls`): the titlebar strip height and the
    /// traffic-light region the BBR-owned titlebar row must clear, read
    /// from the real window — never hardcoded traffic-light coordinates.
    /// `None` before the root view has a window (non-macOS or startup).
    pub fn safe_titlebar_guide(&self) -> Option<bbr_web::SafeTitlebarGuide> {
        self.assert_main_thread();
        let root = self.host.root_view()?;
        let mtm = MainThreadMarker::new()?;
        crate::titlebar_geometry::safe_titlebar_guide(mtm, root)
    }

    /// Hand all subsequent classified web-error incidents to `sink`
    /// (task packet `error-surface-contract`). Replaces any previous sink.
    /// Inherent to the concrete engine (not part of the portable trait): the
    /// app wires it once at composition, like the find/downloads adapters.
    pub fn set_incidents(&mut self, sink: Box<dyn FnMut(ClassifiedIncident)>) {
        self.assert_main_thread();
        *self.incident_sink.borrow_mut() = Some(sink);
    }

    /// Install portable new-window policy for every existing and future
    /// delegate. Replaces any previous handler.
    pub fn set_new_window_handler(&mut self, handler: Box<bbr_web::new_window::NewWindowHandler>) {
        self.assert_main_thread();
        self.new_window.set_handler(handler);
    }

    /// Install the app delivery sink for every decided new-window request.
    pub fn set_new_window_delivery(
        &mut self,
        reporter: Box<dyn FnMut(bbr_web::new_window::NewWindowDelivery)>,
    ) {
        self.assert_main_thread();
        self.new_window.set_delivery_reporter(reporter);
    }

    /// Install the app sink for every committed logical content gesture
    /// (exactly one per committed horizontal trackpad swipe over a web
    /// view). Replaces any previous reporter.
    pub fn set_content_gesture_reporter(
        &mut self,
        reporter: Box<dyn FnMut(bbr_web::ContentGestureCommit)>,
    ) {
        self.assert_main_thread();
        self.content_gesture.set_reporter(reporter);
    }

    /// Narrow diagnostics of the native content-gesture bridge (task
    /// packet `web-content-navigation-gestures`): monitor installation,
    /// gestures over web views, commits reported, cancels, and vertical-
    /// dominant abandonments. `None` when no bridge is attached.
    pub fn content_gesture_diagnostics(
        &self,
    ) -> Option<crate::content_gesture::ContentGestureNativeDiagnostics> {
        self.assert_main_thread();
        self.content_gesture.diagnostics()
    }

    /// Install the blocked-request diagnostic sink.
    pub fn set_new_window_blocked_reporter(
        &mut self,
        reporter: Box<dyn FnMut(bbr_web::new_window::NewWindowRequest)>,
    ) {
        self.assert_main_thread();
        self.new_window.set_blocked_reporter(reporter);
    }

    /// Narrow diagnostics of the one native context-menu bridge (task
    /// packet `bbr-context-menus-over-web`): right-clicks observed over
    /// web views and the page-vs-element ownership classification counters.
    /// `None` when no bridge is attached (non-macOS or attach failure).
    pub fn context_menu_diagnostics(
        &self,
    ) -> Option<crate::menu_adapter::ContextMenuNativeDiagnostics> {
        self.assert_main_thread();
        let mtm = MainThreadMarker::new()?;
        self.menu_bridge
            .as_ref()
            .map(|bridge| bridge.diagnostics(mtm))
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebEngineWk used off the main thread"
        );
    }

    /// A frame for a newly created view: zero until its keyed slot publishes
    /// geometry (the view starts hidden, so the default frame is never
    /// visible).
    fn initial_frame(&self) -> objc2_foundation::NSRect {
        self.host.frame_rect(WebRect::default())
    }

    fn session(&self, id: WebViewId) -> Option<&NativeWebSession> {
        let session = self.sessions.get(&id);
        if session.is_none() {
            log::warn!("unknown web view {id:?}");
        }
        session
    }

    fn session_mut(&mut self, id: WebViewId) -> Option<&mut NativeWebSession> {
        let session = self.sessions.get_mut(&id);
        if session.is_none() {
            log::warn!("unknown web view {id:?}");
        }
        session
    }
}

impl WebEngine for WebEngineWk {
    fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, _content_height: f32) {
        self.assert_main_thread();
        // Geometry is addressed per live view: unknown ids are no-ops, and
        // only that view's frame changes (never a sibling pane). An unchanged
        // rect is skipped, so an idle window makes no `setFrame:` traffic.
        if !self.sessions.contains_key(&id) {
            log::warn!("unknown web view {id:?}");
            return;
        }
        if self.geometry.get(&id) == Some(&rect) {
            return;
        }
        let frame = self.host_of_view(id).frame_rect(rect);
        self.geometry.insert(id, rect);
        if let Some(session) = self.sessions.get_mut(&id) {
            session.web_view.setFrame(frame);
            session.navigation_cover.setFrame(frame);
        }
    }

    fn set_web_view_corner_radius(&mut self, id: WebViewId, radius: f32) {
        self.assert_main_thread();
        // The content card clips the slot and its navigation cover alike,
        // so the cover never shows square corners over the chassis.
        if let Some(session) = self.sessions.get(&id) {
            crate::card_clip::clip_to_radius(&session.web_view, f64::from(radius));
            crate::card_clip::clip_to_radius(&session.navigation_cover, f64::from(radius));
        }
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        self.assert_main_thread();
        if self.sessions.contains_key(&id) {
            log::error!("duplicate web view id {id:?}");
            return;
        }
        let mtm = MainThreadMarker::new().expect("create_web_view called off main thread");
        let store = native_session::data_store_for_profile(
            mtm,
            profile,
            self.options.non_persistent,
            &mut self.stores,
        );
        let config = unsafe { WKWebViewConfiguration::new(mtm) };
        unsafe {
            config.setWebsiteDataStore(&store);
        }
        let web_view = unsafe {
            WKWebView::initWithFrame_configuration(
                WKWebView::alloc(mtm),
                self.initial_frame(),
                &config,
            )
        };
        // Match DESIGN's dark `material:glass` behind page content so a new
        // or navigating WKWebView never exposes WebKit's default white plane.
        let under_page_background = NSColor::colorWithSRGBRed_green_blue_alpha(
            0x15 as f64 / 255.0,
            0x17 as f64 / 255.0,
            0x19 as f64 / 255.0,
            1.0,
        );
        unsafe {
            web_view.setUnderPageBackgroundColor(Some(&under_page_background));
        }
        let navigation_cover: Retained<NSView> =
            NavigationCover::new(mtm, self.initial_frame()).into_super();
        let delegate = WebViewDelegate::new(
            mtm,
            id,
            profile,
            self.sink.clone(),
            self.incident_sink.clone(),
            navigation_cover.clone(),
            delegate::DelegateHandles {
                new_window: self.new_window.clone(),
                favicon_acquisition: self.favicon_acquisition.clone(),
                permissions: self.permissions.clone(),
                client_certificates: self.client_certificates.clone(),
            },
        );
        unsafe {
            web_view.setNavigationDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            web_view.setUIDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            delegate::observe_title(&web_view, delegate.as_super());
            crate::permission_requests::observe_capture(&web_view, delegate.as_super());
        }
        web_view.setHidden(true);
        navigation_cover.setHidden(true);
        if let Some(root) = self.host.root_view() {
            root.addSubview(&web_view);
            root.addSubview(&navigation_cover);
        }
        if !initial_url.is_empty() {
            native_session::load_url(&web_view, initial_url);
        }
        self.sessions.insert(
            id,
            NativeWebSession {
                profile,
                web_view: web_view.clone(),
                navigation_cover,
                delegate,
                visible: false,
            },
        );
        // The renderer-liveness probe host registers the fresh view (a stale
        // completion from a previous incarnation of the id is dropped by the
        // generation reset).
        self.probe.register(id);
        if let Some(bridge) = self.menu_bridge.as_ref() {
            bridge.register_web_view(mtm, id, &web_view);
        }
        self.content_gesture.register_web_view(mtm, id, &web_view);
    }

    fn destroy_web_view(&mut self, id: WebViewId) {
        self.assert_main_thread();
        self.geometry.remove(&id);
        // The liveness probe's record (and any armed deadline timer) is
        // dropped with the session: a late answer for the id cannot mutate
        // a newer session.
        self.probe.forget(id);
        // Any in-flight favicon acquisition (collector completion, queued
        // or live download) for the id is dropped or invalidated: a late
        // observation cannot mutate a newer session.
        self.favicon_acquisition.forget(id);
        // Blueprint D11: WebKit expects every permission handler answered.
        self.permissions.borrow_mut().deny_view(id);
        self.client_certificates.borrow_mut().cancel_view(id);
        self.media.forget(id);
        if let Some(session) = self.sessions.remove(&id) {
            session.teardown();
        } else {
            log::warn!("destroy requested for unknown web view {id:?}");
        }
        let window = self.roots.window_of(id);
        self.roots.session_window.remove(&id);
        if let Some(mtm) = MainThreadMarker::new() {
            match self.roots.root(window) {
                Some(root) => {
                    if let Some(bridge) = root.menu_bridge.as_ref() {
                        bridge.unregister_web_view(mtm, id);
                    }
                    root.content_gesture.unregister_web_view(mtm, id);
                }
                None => {
                    if let Some(bridge) = self.menu_bridge.as_ref() {
                        bridge.unregister_web_view(mtm, id);
                    }
                    self.content_gesture.unregister_web_view(mtm, id);
                }
            }
        }
    }

    fn show(&mut self, id: WebViewId, visible: bool) {
        self.assert_main_thread();
        let Some(session) = self.session_mut(id) else {
            return;
        };
        session.web_view.setHidden(!visible);
        session
            .navigation_cover
            .setHidden(!visible || !unsafe { session.web_view.isLoading() });
        session.visible = visible;
        if !visible {
            let mtm = MainThreadMarker::new().expect("main thread");
            self.host_of_view(id).focus_chrome(mtm);
        }
    }

    fn raise_web_view(&mut self, id: WebViewId) {
        self.assert_main_thread();
        let Some(session) = self.session(id) else {
            return;
        };
        let Some(root) = self.host_of_view(id).root_view() else {
            return;
        };
        // AppKit subviews draw front-to-back: re-adding the view moves it to
        // the end of the subview list, i.e. above every other web view. The
        // dark navigation cover is re-added after the view so it stays above
        // the view itself. Removing and re-adding within one main-thread
        // turn is deterministic and keeps the same retained objects. The
        // objc2 APIs are main-thread-safe on the retained views.
        session.web_view.removeFromSuperview();
        root.addSubview(&session.web_view);
        session.navigation_cover.removeFromSuperview();
        root.addSubview(&session.navigation_cover);
    }

    fn action(&mut self, id: WebViewId, action: WebAction) {
        self.assert_main_thread();
        let Some(session) = self.session(id) else {
            return;
        };
        let web_view = &session.web_view;
        match action {
            WebAction::Load { url } => native_session::load_url(web_view, &url),
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
                if let Some(session) = self.sessions.get(&id) {
                    session.navigation_cover.setHidden(true);
                }
                if let Some(session) = self.sessions.get(&id) {
                    native_session::emit_state(&self.sink, session, id);
                }
            },
            // The BBR context menu's Copy Link entry: the engine owns the
            // platform pasteboard, so the portable action is implemented
            // natively here (the view's current URL, never a caller-passed
            // string).
            WebAction::CopyLink => {
                if let Some(url) = unsafe { web_view.URL() }.and_then(|u| u.absoluteString()) {
                    crate::menu_adapter::copy_to_pasteboard(&url.to_string());
                }
            }
            WebAction::GoToHistory { offset } => crate::session_history::go(web_view, offset),
        }
    }

    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>) {
        self.assert_main_thread();
        *self.sink.borrow_mut() = Some(sink);
    }

    fn probe_responsiveness(&mut self, id: WebViewId) {
        // The probe mechanics live in the focused `responsiveness` module
        // (this file is a no-grow surface); unknown/destroyed ids are
        // no-ops there.
        crate::responsiveness::probe_web_view(self, id);
    }

    fn focus_native_web(&mut self, id: WebViewId) {
        self.assert_main_thread();
        let Some(session) = self.session(id) else {
            return;
        };
        let mtm = MainThreadMarker::new().expect("main thread");
        self.host_of_view(id).focus_web_view(mtm, &session.web_view);
    }

    fn focus_host_chrome(&mut self) {
        self.assert_main_thread();
        let mtm = MainThreadMarker::new().expect("main thread");
        self.host_of_window(self.roots.key_window).focus_chrome(mtm);
    }

    fn place_web_view(&mut self, id: WebViewId, window: u64) {
        self.assert_main_thread();
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let from = self.roots.window_of(id);
        if from == window || (window != 0 && self.roots.root(window).is_none()) {
            return;
        }
        let Some(session) = self.sessions.get(&id) else {
            return;
        };
        let Some(root) = self.host_of_window(window).root_view() else {
            return;
        };
        // Re-parent the page and its cover: the WKWebView keeps its
        // process, history and state.
        session.web_view.removeFromSuperview();
        root.addSubview(&session.web_view);
        session.navigation_cover.removeFromSuperview();
        root.addSubview(&session.navigation_cover);
        let web_view = session.web_view.clone();
        // The bridges of the old window let go; the new window's take it.
        match self.roots.root(from) {
            Some(old) => {
                if let Some(bridge) = old.menu_bridge.as_ref() {
                    bridge.unregister_web_view(mtm, id);
                }
                old.content_gesture.unregister_web_view(mtm, id);
            }
            None => {
                if let Some(bridge) = self.menu_bridge.as_ref() {
                    bridge.unregister_web_view(mtm, id);
                }
                self.content_gesture.unregister_web_view(mtm, id);
            }
        }
        match self.roots.root(window) {
            Some(new) => {
                if let Some(bridge) = new.menu_bridge.as_ref() {
                    bridge.register_web_view(mtm, id, &web_view);
                }
                new.content_gesture.register_web_view(mtm, id, &web_view);
            }
            None => {
                if let Some(bridge) = self.menu_bridge.as_ref() {
                    bridge.register_web_view(mtm, id, &web_view);
                }
                self.content_gesture.register_web_view(mtm, id, &web_view);
            }
        }
        if window == 0 {
            self.roots.session_window.remove(&id);
        } else {
            self.roots.session_window.insert(id, window);
        }
        // The next slot geometry flips against the new root.
        self.geometry.remove(&id);
    }

    fn session_history(&self, id: WebViewId) -> Option<bbr_web::SessionHistory> {
        self.assert_main_thread();
        self.session(id)
            .map(|session| crate::session_history::read(&session.web_view))
    }

    fn decide_permission(&mut self, request: u64, allow: bool) {
        self.permissions.borrow_mut().decide(request, allow);
    }

    fn probe_media(&mut self) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        for (id, session) in &self.sessions {
            self.media.probe(mtm, *id, &session.web_view);
        }
    }

    fn media_command(&mut self, view: WebViewId, command: bbr_web::media::MediaCommand) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        if let Some(session) = self.sessions.get(&view) {
            self.media.command(mtm, &session.web_view, command);
        }
    }

    fn external_app_name(&self, url: &str) -> Option<String> {
        crate::external_schemes::app_name_for(url)
    }

    fn open_external(&mut self, url: &str) -> bool {
        crate::external_schemes::open(url)
    }

    fn answer_client_certificate(
        &mut self,
        request: u64,
        answer: bbr_web::client_certificates::ClientCertificateAnswer,
    ) {
        self.client_certificates
            .borrow_mut()
            .answer(request, &answer);
    }

    fn clear_site_data(&mut self, profile: ProfileId, host: &str) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let store = native_session::data_store_for_profile(
            mtm,
            profile,
            self.options.non_persistent,
            &mut self.stores,
        );
        crate::profile_data::clear_site(&store, host);
    }

    fn delete_profile_store(&mut self, profile: ProfileId) {
        crate::profile_data::delete_store(profile, self.options.non_persistent, &mut self.stores);
    }

    fn clear_profile_data(
        &mut self,
        profile: ProfileId,
        kinds: bbr_web::WebsiteDataKinds,
        since_unix_seconds: Option<f64>,
    ) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let store = native_session::data_store_for_profile(
            mtm,
            profile,
            self.options.non_persistent,
            &mut self.stores,
        );
        crate::profile_data::clear_data(&store, kinds, since_unix_seconds);
    }

    fn live_session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Drop for WebEngineWk {
    fn drop(&mut self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "engine dropped off main thread"
        );
        for (_, session) in self.sessions.drain() {
            session.teardown();
        }
    }
}
