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
use objc2::{AnyThread, ClassType, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSString, NSURLRequest, NSURL, NSUUID};
use objc2_web_kit::{WKWebView, WKWebViewConfiguration, WKWebsiteDataStore};

use bbr_web::{ProfileId, WebAction, WebEngine, WebEvent, WebRect, WebViewId};

use crate::delegate::{self, WebViewDelegate};
use crate::host::HostBridge;

/// Options that change engine behaviour; populated by the composition root.
#[derive(Debug, Clone, Default)]
pub struct EngineOptions {
    /// Use a non-persistent `WKWebsiteDataStore` per profile (E2E hermeticity).
    /// When false, each profile maps to a deterministic persistent store id.
    pub non_persistent: bool,
}

/// All native objects and state belonging to one live web session.
struct NativeWebSession {
    profile: ProfileId,
    web_view: Retained<WKWebView>,
    /// WebKit does not retain navigation/UI delegates.
    delegate: Retained<WebViewDelegate>,
    visible: bool,
}

impl NativeWebSession {
    /// Remove KVO before releasing the observed view, then detach it from AppKit.
    fn teardown(self) {
        unsafe {
            delegate::unobserve_title(&self.web_view, self.delegate.as_super());
        }
        self.web_view.removeFromSuperview();
    }
}

/// The macOS WebKit engine.
pub struct WebEngineWk {
    host: HostBridge,
    options: EngineOptions,
    sessions: HashMap<WebViewId, NativeWebSession>,
    /// Shared with every delegate; replaced by [`WebEngine::set_events`].
    sink: delegate::EventSink,
    /// Profile → website data store cache. Stores outlive individual views.
    stores: HashMap<ProfileId, Retained<WKWebsiteDataStore>>,
    /// Last GPUI slot geometry, applied to every live session.
    last_rect: WebRect,
    rect_published: bool,
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
        Self {
            host,
            options,
            sessions: HashMap::new(),
            sink: Rc::new(RefCell::new(None)),
            stores: HashMap::new(),
            last_rect: WebRect::default(),
            rect_published: false,
        }
    }

    /// True when at least one live WKWebView is hosted.
    pub fn web_view_hosted(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Profile identity of a live session (native integration diagnostics).
    pub fn session_profile(&self, id: WebViewId) -> Option<ProfileId> {
        self.sessions.get(&id).map(|session| session.profile)
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebEngineWk used off the main thread"
        );
    }

    fn data_store(&mut self, profile: ProfileId) -> Retained<WKWebsiteDataStore> {
        if let Some(store) = self.stores.get(&profile) {
            return store.clone();
        }
        let mtm = MainThreadMarker::new().expect("data_store called off main thread");
        let store = if self.options.non_persistent {
            unsafe { WKWebsiteDataStore::nonPersistentDataStore(mtm) }
        } else {
            let uuid_string = NSString::from_str(&profile_store_uuid(profile));
            let uuid = NSUUID::initWithUUIDString(NSUUID::alloc(), &uuid_string)
                .expect("generated profile UUID is valid");
            unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&uuid, mtm) }
        };
        self.stores.insert(profile, store.clone());
        store
    }

    fn current_frame(&self) -> objc2_foundation::NSRect {
        self.host.frame_rect(self.last_rect)
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

    fn load_url(web_view: &WKWebView, url: &str) {
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

    fn emit_state(&self, id: WebViewId) {
        let Some(session) = self.session(id) else {
            return;
        };
        let web_view = &session.web_view;
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
            return;
        }
        self.rect_published = rect.width.max(rect.height) > 0.0;
        self.last_rect = rect;
        let frame = self.current_frame();
        for session in self.sessions.values() {
            session.web_view.setFrame(frame);
        }
    }

    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
        self.assert_main_thread();
        if self.sessions.contains_key(&id) {
            log::error!("duplicate web view id {id:?}");
            return;
        }
        let mtm = MainThreadMarker::new().expect("create_web_view called off main thread");
        let store = self.data_store(profile);
        let config = unsafe { WKWebViewConfiguration::new(mtm) };
        unsafe {
            config.setWebsiteDataStore(&store);
        }
        let web_view = unsafe {
            WKWebView::initWithFrame_configuration(
                WKWebView::alloc(mtm),
                self.current_frame(),
                &config,
            )
        };
        let delegate = WebViewDelegate::new(mtm, id, self.sink.clone());
        unsafe {
            web_view.setNavigationDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            web_view.setUIDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            delegate::observe_title(&web_view, delegate.as_super());
        }
        web_view.setHidden(true);
        if let Some(root) = self.host.root_view() {
            root.addSubview(&web_view);
        }
        if !initial_url.is_empty() {
            Self::load_url(&web_view, initial_url);
        }
        self.sessions.insert(
            id,
            NativeWebSession {
                profile,
                web_view,
                delegate,
                visible: false,
            },
        );
    }

    fn destroy_web_view(&mut self, id: WebViewId) {
        self.assert_main_thread();
        if let Some(session) = self.sessions.remove(&id) {
            session.teardown();
        } else {
            log::warn!("destroy requested for unknown web view {id:?}");
        }
    }

    fn show(&mut self, id: WebViewId, visible: bool) {
        self.assert_main_thread();
        let Some(session) = self.session_mut(id) else {
            return;
        };
        session.web_view.setHidden(!visible);
        session.visible = visible;
        if !visible {
            let mtm = MainThreadMarker::new().expect("main thread");
            self.host.focus_chrome(mtm);
        }
    }

    fn action(&mut self, id: WebViewId, action: WebAction) {
        self.assert_main_thread();
        let Some(session) = self.session(id) else {
            return;
        };
        let web_view = &session.web_view;
        match action {
            WebAction::Load { url } => Self::load_url(web_view, &url),
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
                self.emit_state(id);
            },
        }
    }

    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>) {
        self.assert_main_thread();
        *self.sink.borrow_mut() = Some(sink);
    }

    fn focus_native_web(&mut self, id: WebViewId) {
        self.assert_main_thread();
        let Some(session) = self.session(id) else {
            return;
        };
        let mtm = MainThreadMarker::new().expect("main thread");
        self.host.focus_web_view(mtm, &session.web_view);
    }

    fn focus_host_chrome(&mut self) {
        self.assert_main_thread();
        let mtm = MainThreadMarker::new().expect("main thread");
        self.host.focus_chrome(mtm);
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

/// Stable UUID namespace for a persistent WebKit data store.
fn profile_store_uuid(profile: ProfileId) -> String {
    let upper = (profile.0 >> 48) as u16;
    let lower = profile.0 & 0x0000_ffff_ffff_ffff;
    format!("42425200-0000-4000-{upper:04x}-{lower:012x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_store_uuid_is_stable_and_unique() {
        assert_eq!(
            profile_store_uuid(ProfileId(1)),
            "42425200-0000-4000-0000-000000000001"
        );
        assert_eq!(
            profile_store_uuid(ProfileId(u64::MAX)),
            "42425200-0000-4000-ffff-ffffffffffff"
        );
        assert_ne!(
            profile_store_uuid(ProfileId(1)),
            profile_store_uuid(ProfileId(2))
        );
    }
}
