//! macOS adapter seam for new-window requests.
//!
//! This is the only macOS code that knows how a `target="_blank"` /
//! `window.open` request reaches the portable contract:
//!
//! 1. The `WKUIDelegate::createWebViewWithConfiguration:…` method (wired to
//!    [`extract_context`] in a later serial integration) snapshots the
//!    `WKNavigationAction`/`WKWindowFeatures` it receives into a plain
//!    [`NewWindowContext`].
//! 2. [`NewWindowCoordinator::handle`] reports the portable
//!    [`NewWindowRequest`] to orchestration and maps the returned
//!    [`NewWindowDecision`] to the deterministic [`NewWindowOutcome`].
//! 3. The delegate executes the outcome and, for [`NewWindowOutcome::Blocked`],
//!    the refusal is reported through the coordinator's blocked-request
//!    channel so the user's gesture is never silent.
//!
//! The seam never auto-navigates and never silently discards a user gesture:
//! with no handler installed it deterministically blocks (the Phase-1
//! "popups suppressed" behaviour), but the block is reported instead of
//! vanishing. Profile isolation is structural: the only session this seam can
//! request is a fresh context in the source profile's website data store.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use bbr_web::new_window::{
    decide, NewWindowDecision, NewWindowDelivery, NewWindowFeatures, NewWindowOutcome,
    NewWindowRequest, NewWindowTarget,
};
use bbr_web::{ProfileId, WebViewId};

/// Main-thread callback that reports a blocked request for surfacing.
type NewWindowBlockedReporter = dyn FnMut(NewWindowRequest);
/// Main-thread callback that delivers every decided request to the app.
type NewWindowDeliveryReporter = dyn FnMut(NewWindowDelivery);

/// Plain-value snapshot of a WebKit new-window call — everything a
/// `createWebViewWithConfiguration:…` delegate method extracts before deciding.
#[derive(Debug, Clone, PartialEq)]
pub struct NewWindowContext {
    /// The session that issued the request.
    pub source: WebViewId,
    /// The profile of the source session, resolved from the engine's session
    /// registry by the caller. Created sessions must use this store.
    pub source_profile: ProfileId,
    /// The target URL exactly as requested by the page.
    pub url: String,
    /// WebKit's user-gesture report.
    pub user_initiated: bool,
    /// WebKit window-features metadata (sizes/toolbars).
    pub features: NewWindowFeatures,
}

impl NewWindowContext {
    /// Build the portable request the decision handler receives.
    ///
    /// This delegate method is only invoked for a fresh browsing context, so
    /// the requested target is always [`NewWindowTarget::NewWindow`].
    pub fn to_request(&self) -> NewWindowRequest {
        NewWindowRequest {
            source: self.source,
            source_profile: self.source_profile,
            url: self.url.clone(),
            user_initiated: self.user_initiated,
            target: NewWindowTarget::NewWindow,
            features: self.features,
        }
    }
}

/// Synchronous request/decision channel plus the deterministic block fallback.
///
/// Main-thread only: the WebKit delegate invokes [`handle`] synchronously
/// inside the UI-delegate call, and the closures it runs have the same
/// main-thread-only shape as the engine's `set_events` sink.
///
/// [`handle`]: NewWindowCoordinator::handle
#[derive(Clone)]
pub struct NewWindowCoordinator {
    handler: Rc<RefCell<Option<Box<bbr_web::new_window::NewWindowHandler>>>>,
    blocked: Rc<RefCell<Option<Box<NewWindowBlockedReporter>>>>,
    delivery: Rc<RefCell<Option<Box<NewWindowDeliveryReporter>>>>,
}

impl NewWindowCoordinator {
    /// A coordinator with no handler installed: every request blocks and is
    /// reported through the blocked-request channel.
    pub fn new() -> Self {
        Self {
            handler: Rc::new(RefCell::new(None)),
            blocked: Rc::new(RefCell::new(None)),
            delivery: Rc::new(RefCell::new(None)),
        }
    }

    /// Install the portable decision handler, replacing any previous one.
    pub fn set_handler(&mut self, handler: Box<bbr_web::new_window::NewWindowHandler>) {
        *self.handler.borrow_mut() = Some(handler);
    }

    /// Install the blocked-request reporter invoked for every
    /// [`NewWindowOutcome::Blocked`], replacing any previous one.
    pub fn set_blocked_reporter(&mut self, reporter: Box<NewWindowBlockedReporter>) {
        *self.blocked.borrow_mut() = Some(reporter);
    }

    /// Install the app delivery sink invoked once for every decided request.
    /// The sink executes the plain outcome through the controller; the native
    /// delegate still returns `None` and never creates an untracked view.
    pub fn set_delivery_reporter(&mut self, reporter: Box<NewWindowDeliveryReporter>) {
        *self.delivery.borrow_mut() = Some(reporter);
    }

    /// Report `context` to orchestration and return the outcome to execute.
    ///
    /// - No handler → [`NewWindowOutcome::Blocked`]: the request is never
    ///   auto-navigated and the caller still receives the refusal to surface.
    /// - A blocked outcome always invokes the blocked-request reporter, so a
    ///   suppressed user gesture is never silent.
    pub fn handle(&self, context: &NewWindowContext) -> NewWindowOutcome {
        let request = context.to_request();
        let decision = match self.handler.borrow_mut().as_mut() {
            Some(handler) => handler(request.clone()),
            None => NewWindowDecision::Block,
        };
        let outcome = decide(&request, decision);
        if matches!(outcome, NewWindowOutcome::Blocked) {
            if let Some(reporter) = self.blocked.borrow_mut().as_mut() {
                reporter(request.clone());
            }
        }
        if let Some(reporter) = self.delivery.borrow_mut().as_mut() {
            reporter(NewWindowDelivery {
                request,
                outcome: outcome.clone(),
            });
        }
        outcome
    }
}

impl Default for NewWindowCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for NewWindowCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewWindowCoordinator")
            .field("has_handler", &self.handler.borrow().is_some())
            .field("has_blocked_reporter", &self.blocked.borrow().is_some())
            .field("has_delivery_reporter", &self.delivery.borrow().is_some())
            .finish()
    }
}

/// Extract the plain context from a WebKit new-window call.
///
/// `source`/`source_profile` are resolved by the caller from the engine's
/// session registry (the delegate knows its own `WebViewId`; the profile
/// lives in the engine).
///
/// # Safety
///
/// `action` and `features` must be the live objects WebKit delivers to a
/// `WKUIDelegate::createWebViewWithConfiguration:…` method, read on the main
/// thread.
#[cfg(target_os = "macos")]
pub unsafe fn extract_context(
    action: &objc2_web_kit::WKNavigationAction,
    features: &objc2_web_kit::WKWindowFeatures,
    source: WebViewId,
    source_profile: ProfileId,
) -> NewWindowContext {
    // SAFETY: main thread, live delegate objects (caller contract).
    let request = unsafe { action.request() };
    let url = request
        .URL()
        .and_then(|u| u.absoluteString())
        .map(|s| s.to_string())
        .unwrap_or_default();
    NewWindowContext {
        source,
        source_profile,
        url,
        user_initiated: extract_user_initiated(action),
        features: extract_features(features),
    }
}

/// Best-effort user-gesture classification from supported WebKit metadata.
///
/// `WKNavigationAction.isUserInitiated` is not declared by the installed
/// `objc2-web-kit 0.3.2` bindings and is not messaged here: an undocumented
/// selector could terminate the app with an unrecognized-selector exception.
/// `LinkActivated` is the supported navigation metadata that proves a direct
/// link activation; script-created `window.open` requests remain false unless
/// a later binding exposes a documented gesture signal.
///
/// # Safety
///
/// Live `action` on the main thread, as delivered by WebKit.
#[cfg(target_os = "macos")]
unsafe fn extract_user_initiated(action: &objc2_web_kit::WKNavigationAction) -> bool {
    // SAFETY: main thread, live object; this accessor is declared by WebKit.
    (unsafe { action.navigationType() }) == objc2_web_kit::WKNavigationType::LinkActivated
}

/// WebKit window-features metadata for a new-window call.
///
/// Every `WKWindowFeatures` attribute is *nil* when the page did not specify
/// it, which maps directly onto the portable `Option` fields. The `NSNumber`
/// conversions are safe bindings; only the accessors are unsafe (main thread).
///
/// # Safety
///
/// Live `features` on the main thread (as delivered by WebKit).
#[cfg(target_os = "macos")]
unsafe fn extract_features(features: &objc2_web_kit::WKWindowFeatures) -> NewWindowFeatures {
    // SAFETY: main thread, live object; accessors are main-thread-only.
    let x = unsafe { features.x() }.map(|n| n.doubleValue());
    // SAFETY: main thread, live object.
    let y = unsafe { features.y() }.map(|n| n.doubleValue());
    // SAFETY: main thread, live object.
    let width = unsafe { features.width() }.map(|n| n.doubleValue());
    // SAFETY: main thread, live object.
    let height = unsafe { features.height() }.map(|n| n.doubleValue());
    // SAFETY: main thread, live object.
    let menu_bar_visible = unsafe { features.menuBarVisibility() }.map(|n| n.boolValue());
    // SAFETY: main thread, live object.
    let status_bar_visible = unsafe { features.statusBarVisibility() }.map(|n| n.boolValue());
    // SAFETY: main thread, live object.
    let toolbar_visible = unsafe { features.toolbarsVisibility() }.map(|n| n.boolValue());
    // SAFETY: main thread, live object.
    let resizable = unsafe { features.allowsResizing() }.map(|n| n.boolValue());
    NewWindowFeatures {
        x,
        y,
        width,
        height,
        menu_bar_visible,
        status_bar_visible,
        toolbar_visible,
        resizable,
    }
}
