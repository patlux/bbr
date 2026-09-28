//! # bbr-web — platform-neutral browser contracts
//!
//! This crate defines the seam between the GPUI chrome (`bbr-app`) and a
//! native web engine (`bbr-web-macos`). It contains **no** framework imports,
//! no GPUI, no Objective-C, and no `unsafe` code — it is the only crate that
//! later portable crates (`bbr-domain`, `bbr-browser`, `bbr-ui`, ...) are
//! allowed to depend on.
//!
//! ## Coordinate systems (important)
//!
//! [`WebRect`] uses the **host layout coordinate system**: origin at the
//! **top-left** of the window content area, units in **points** (logical
//! pixels, matching GPUI's `Bounds<Pixels>` and AppKit points). AppKit frames
//! use a bottom-left origin, so the engine adapter flips the rect with
//! [`flip_rect`] before calling `setFrame:`. The math is kept here so it can
//! be unit-tested without any macOS machinery.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

pub mod auth_challenges;
pub mod content_gesture;
pub mod external_urls;
pub mod favicon;
pub mod file_panels;
pub mod find_in_page;
pub mod media;
pub mod new_window;
pub mod permissions;
pub mod site_settings;
pub mod web_errors;
pub use content_gesture::{ContentGestureCommit, ContentGestureDirection, ContentGesturePhase};
pub use external_urls::{
    classify, decide_routing, ExternalUrlCoordinator, ExternalUrlError, ExternalUrlOutcome,
    ExternalUrlRequest, ExternalUrlRequestId, RoutingDecision, UrlClass,
};
pub use favicon::{choose_best_candidate, FaviconCandidate, FaviconContent};
pub use new_window::{
    decide, decision_for_intent, default_decision, peek_eligible, NewWindowDecision,
    NewWindowDelivery, NewWindowDisposition, NewWindowFeatures, NewWindowInput, NewWindowIntent,
    NewWindowOutcome, NewWindowRequest, NewWindowSession, NewWindowTarget,
};

/// Stable identifier of a browsing profile. Maps 1:1 to a persistent
/// [`WKWebsiteDataStore`] on macOS.
///
/// [`WKWebsiteDataStore`]: https://developer.apple.com/documentation/webkit/wkwebsitedatastore
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileId(pub u64);

/// Stable identifier of a live web view / tab content slot.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebViewId(pub u64);

/// A rectangle in **host layout coordinates**: top-left origin, points.
///
/// `y` grows downwards (GPUI / web layout convention).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WebRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WebRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// The native safe-titlebar layout guide (macOS; packet
/// `titlebar-navigation-controls.md`). Plain presentation data in host
/// layout coordinates (top-left origin, points): the titlebar strip height
/// and the traffic-light region the BBR-owned titlebar row must clear.
/// AppKit details stay in the macOS composition edge; the portable UI
/// renders only what this guide reports.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeTitlebarGuide {
    /// Titlebar strip height in points (window frame minus the native
    /// content layout rect). Zero on platforms without a native titlebar
    /// (the portable fallback renders no strip).
    pub height: f32,
    /// The traffic-light region in window coordinates (top-left origin,
    /// points): the close button's frame. The sidebar toggle must start
    /// after its right edge; never hardcoded traffic-light coordinates.
    pub traffic_lights: WebRect,
}

impl SafeTitlebarGuide {
    /// The portable fallback when no native titlebar exists (non-macOS or
    /// before the window is attached): a zero-height strip with an empty
    /// traffic-light region.
    pub fn none() -> Self {
        Self {
            height: 0.0,
            traffic_lights: WebRect::default(),
        }
    }
}

impl fmt::Display for WebRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:.1}, {:.1}) {}x{}",
            self.x, self.y, self.width, self.height
        )
    }
}

/// Flip a top-left-origin rect into an AppKit bottom-left-origin frame.
///
/// `container_height` is the window content area height in points. The result
/// keeps `x`/`width`/`height` unchanged (AppKit and host layout agree on
/// horizontal direction and on distances), and only the vertical origin is
/// mirrored: `y_appkit = container_height - (y_host + height)`.
pub fn flip_rect(rect: WebRect, container_height: f32) -> WebRect {
    debug_assert!(
        container_height.is_finite() && container_height >= 0.0,
        "container height must be a finite non-negative number, got {container_height}"
    );
    WebRect {
        x: rect.x,
        y: container_height - (rect.y + rect.height),
        width: rect.width,
        height: rect.height,
    }
}

/// Commands the engine can execute on a single web view.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum WebAction {
    /// Navigate to a URL. The engine uses the URL exactly as given (apply
    /// [`normalize_url`] in the caller if you want scheme defaulting).
    Load {
        url: String,
    },
    Back,
    Forward,
    Reload,
    Stop,
    /// Copy the view's current URL to the system clipboard (the BBR context
    /// menu's Copy Link entry; the engine owns the platform pasteboard).
    CopyLink,
    /// Jump `offset` entries through the session history in one step
    /// (negative = back, positive = forward; blueprint D14's history
    /// menu). An offset outside the list is a no-op.
    GoToHistory {
        offset: i32,
    },
}

/// One session-history entry as the engine reports it (blueprint D14).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub url: String,
    /// The page title at the time of the visit (may be empty).
    pub title: String,
}

/// A live view's session history around the current entry: `back` is
/// ordered nearest first (offset −1, −2, …), `forward` nearest first
/// (offset +1, +2, …).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionHistory {
    pub back: Vec<HistoryEntry>,
    pub forward: Vec<HistoryEntry>,
}

/// Events emitted by the native engine, keyed by [`WebViewId`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum WebEvent {
    UrlChanged(WebViewId, String),
    TitleChanged(WebViewId, String),
    LoadingChanged(WebViewId, bool),
    CanGoBackChanged(WebViewId, bool),
    CanGoForwardChanged(WebViewId, bool),
    /// The initial document of this session committed in WebKit (the
    /// `didCommitNavigation:` delegate callback). The controller uses it to
    /// decide whether a Back-at-root gesture may return to a child tab's
    /// opener: only a committed initial document (or a settled failure/
    /// crash state) is a settled root, never an ambiguous provisional load.
    DidCommitNavigation(WebViewId),
    PageCrashed(WebViewId),
    NavigationFailed(WebViewId, String),
    /// The session's renderer stopped answering liveness probes
    /// (`true`) or answers again (`false`). A sibling of [`WebEvent::PageCrashed`]:
    /// `PageCrashed` stays the crash authority (the session is destroyed and
    /// the tab marked discarded), while `Unresponsive` keeps the session and
    /// tab alive under the bounded recovery surface. `true` is fail-closed —
    /// a probe that could not complete inside its bounded deadline — and
    /// `false` is a completing probe (generation-checked by the adapter so a
    /// late answer from a previous probe can never clear a newer timeout).
    UnresponsiveChanged(WebViewId, bool),
    /// A right-click over a web view was classified BBR-owned (page
    /// background, not a WebKit interactive element): the context menu
    /// bridge requests the BBR menu at `point` (window content
    /// coordinates, top-left origin, points — the same space the portal
    /// placement math uses). The app presents the menu through the one
    /// `NativeAboveWeb` portal host.
    ContextMenuRequested(WebViewId, WebRect),
    /// A per-page favicon acquisition observation (task packet
    /// `real-favicon-pipeline.md`): the typed [`FaviconContent`] for the
    /// committed page of one live view. The macOS adapter emits the
    /// resolved [`FaviconContent::Image`] bytes after a bounded,
    /// profile-isolated download; any failure is silent (no event), so the
    /// §7.2 monogram stays the deterministic fallback. The
    /// [`FaviconContent::Candidates`] shape exists on the contract so
    /// candidate-URL observation is representable and fake-engine tests can
    /// prove the boundary without a display server.
    FaviconChanged(WebViewId, FaviconContent),
}

/// The seam between chrome and engine.
///
/// Implementations must be **main-thread only** — this trait is used from
/// GPUI's render/event loop, and the macOS WebKit bridge is not thread-safe.
/// The event sink is deliberately *not* `Send` (it is a `Box<dyn FnMut>`);
/// adding `+ Send` would be a lie because the concrete engine and its
/// delegate objects are `MainThreadOnly`.
pub trait WebEngine {
    /// Position the native web surface of one live [`WebViewId`].
    ///
    /// `rect` is in host layout coordinates (top-left origin, points); the
    /// implementation flips it for AppKit using `content_height` (window
    /// content height in points) via [`flip_rect`]. Geometry is addressed
    /// per view: an unknown id is a no-op, a destroyed id loses its cached
    /// geometry, and an unchanged rect produces no repeated native frame
    /// update. A newly created view keeps a zero/default frame until its
    /// keyed slot publishes.
    fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, content_height: f32);

    /// Clip the native surface of one live view to the seated content
    /// card's corner radius (DESIGN §2); `0` is full-bleed (native full
    /// screen). Clipping belongs to the platform's hosting layer — web
    /// content is never sampled or re-rendered — and an unknown id is a
    /// no-op. The default suits engines without a native surface.
    fn set_web_view_corner_radius(&mut self, _id: WebViewId, _radius: f32) {}

    /// Create a web view inside `profile` and kick off `initial_url`.
    fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str);

    /// Tear down and release a web view and its delegates.
    fn destroy_web_view(&mut self, id: WebViewId);

    /// Show or hide a web view (`visible == false` must also drop keyboard
    /// focus held by the view).
    fn show(&mut self, id: WebViewId, visible: bool);

    /// Raise `id` to the top of the native z-order above every other web
    /// view (the transient Peek preview above its warm covered origin).
    /// Unknown ids are no-ops; the implementation must keep the view's
    /// navigation cover (if any) above the view itself.
    fn raise_web_view(&mut self, id: WebViewId);

    /// Execute a navigation/user action on a web view.
    fn action(&mut self, id: WebViewId, action: WebAction);

    /// Refresh the adapter's bounded liveness probe for one live view
    /// (renderer-isolation lane): arms one bounded deadline probe whose
    /// result arrives as a [`WebEvent::UnresponsiveChanged`]. Main-thread
    /// only, never blocks, never waits on the renderer. Unknown or
    /// destroyed ids are no-ops. The adapter generation-checks completions
    /// so a late answer from a previous probe cannot clear a newer timeout.
    fn probe_responsiveness(&mut self, id: WebViewId);

    /// Hand all subsequent [`WebEvent`]s to `sink`. Replaces any previous sink.
    fn set_events(&mut self, sink: Box<dyn FnMut(WebEvent)>);

    /// Move the AppKit first responder to the **specific** web view
    /// (multi-session: the selected tab's view is the focus target, not
    /// "whichever view exists").
    fn focus_native_web(&mut self, id: WebViewId);

    /// Move the AppKit first responder back to the GPUI chrome host view.
    /// GPUI's own focus tree is managed by GPUI; this only fixes the native
    /// side so key events stop going to the web view.
    fn focus_host_chrome(&mut self);

    /// The session history of one live view (blueprint D14's Back/Forward
    /// history menu), read synchronously from the engine's own list. The
    /// default suits engines without a history list; unknown ids are
    /// `None`.
    fn session_history(&self, _id: WebViewId) -> Option<SessionHistory> {
        None
    }

    /// Number of currently live (hosted, not yet destroyed) web views.
    /// Used by the controller and E2E snapshots as native destruction
    /// evidence.
    fn live_session_count(&self) -> usize;
}

/// Default the scheme on user-typed URLs.
///
/// - trims whitespace
/// - passes through strings that already have a scheme (`https://…`,
///   `about:…`, `data:…`) or start with `//`
/// - otherwise prepends `https://`
pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let has_scheme = trimmed.find(':').is_some_and(|idx| {
        let valid_scheme = idx > 0
            && trimmed[..idx]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.');
        let suffix = &trimmed[idx + 1..];
        let looks_like_port = suffix
            .split(['/', '?', '#'])
            .next()
            .is_some_and(|port| !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()));
        valid_scheme && !looks_like_port
    });
    if has_scheme || trimmed.starts_with("//") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

pub mod downloads;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_rect_basic() {
        // A rect in the top-left of a 1000pt-tall window maps to the
        // bottom-left of the AppKit coordinate space, vertically mirrored.
        let flipped = flip_rect(WebRect::new(50.0, 10.0, 300.0, 200.0), 1000.0);
        assert_eq!(flipped.x, 50.0);
        assert_eq!(flipped.y, 1000.0 - (10.0 + 200.0)); // 790.0
        assert_eq!(flipped.width, 300.0);
        assert_eq!(flipped.height, 200.0);
    }

    #[test]
    fn flip_rect_full_content_bottom_aligned() {
        // A rect flush with the bottom of the host layout (y + height == h)
        // must land flush with the bottom of the content area (y_appkit == 0).
        let flipped = flip_rect(WebRect::new(0.0, 800.0, 400.0, 200.0), 1000.0);
        assert_eq!(flipped.y, 0.0);
    }

    #[test]
    fn flip_rect_center_is_an_involution() {
        let rect = WebRect::new(10.0, 20.0, 100.0, 50.0);
        let h = 600.0;
        assert_eq!(flip_rect(flip_rect(rect, h), h), rect);
    }

    #[test]
    fn flip_rect_degenerate_height_is_zero() {
        let flipped = flip_rect(WebRect::new(0.0, 0.0, 0.0, 0.0), 0.0);
        assert_eq!(flipped, WebRect::default());
    }

    #[test]
    fn normalize_url_adds_https_scheme() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(
            normalize_url("  localhost:8080  "),
            "https://localhost:8080"
        );
    }

    #[test]
    fn normalize_url_preserves_existing_scheme() {
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
        assert_eq!(
            normalize_url("https://example.com/a?b=c"),
            "https://example.com/a?b=c"
        );
        assert_eq!(normalize_url("about:blank"), "about:blank");
        assert_eq!(normalize_url("data:text/html,hi"), "data:text/html,hi");
        assert_eq!(normalize_url("//example.com"), "//example.com");
    }

    #[test]
    fn normalize_url_empty_is_empty() {
        assert_eq!(normalize_url("   "), "");
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn ids_and_events_are_plain_values() {
        let id = WebViewId(7);
        assert_eq!(id, WebViewId(7));
        assert_ne!(id, WebViewId(8));
        let _ = ProfileId(1);
        let ev = WebEvent::UrlChanged(WebViewId(1), "https://x".into());
        assert_eq!(ev.clone(), ev);
    }

    /// The perf harness keys its page-ready state on `WebViewId` in a
    /// `BTreeMap`/`BTreeSet`, so the identifiers must implement `Ord`. Guard
    /// the derived ordering so a future removal of the derive dies in the
    /// portable crate instead of as a late `testkit`-feature app compile error.
    #[test]
    fn identifiers_are_ordered_collection_keys() {
        use std::collections::{BTreeMap, BTreeSet};
        let mut seen = BTreeMap::new();
        seen.insert(WebViewId(1), "one".to_string());
        seen.insert(WebViewId(2), "two".to_string());
        assert_eq!(seen.get(&WebViewId(2)).map(String::as_str), Some("two"));

        let mut ready = BTreeSet::new();
        ready.insert(WebViewId(1));
        assert!(ready.contains(&WebViewId(1)));
        assert!(!ready.contains(&WebViewId(2)));

        let mut profiles = BTreeMap::new();
        profiles.insert(ProfileId(9), 1_usize);
        assert_eq!(profiles.get(&ProfileId(9)), Some(&1));
    }

    /// The contract must be object-safe so the app can hold
    /// `Rc<RefCell<dyn WebEngine>>`.
    #[test]
    fn web_engine_is_object_safe() {
        let mut engine: Box<dyn WebEngine> = Box::new(MockEngine::default());
        let _sink: Box<dyn FnMut(WebEvent)> = Box::new(|_| {});
        engine.set_events(_sink);
        engine.set_web_view_rect(WebViewId(7), WebRect::new(0.0, 0.0, 100.0, 80.0), 700.0);
        engine.probe_responsiveness(WebViewId(7));
        engine.focus_native_web(WebViewId(7));
        engine.focus_host_chrome();
        assert_eq!(engine.live_session_count(), 0);
    }

    /// A minimal mock used to prove the trait is usable without any platform.
    #[derive(Debug, Default)]
    struct MockEngine {
        logs: Vec<String>,
    }

    impl WebEngine for MockEngine {
        fn set_web_view_rect(&mut self, id: WebViewId, rect: WebRect, content_height: f32) {
            self.logs
                .push(format!("rect {id:?} {rect} in {content_height}pt"));
        }

        fn create_web_view(&mut self, id: WebViewId, profile: ProfileId, initial_url: &str) {
            self.logs
                .push(format!("create {id:?} in {profile:?} at {initial_url}"));
        }

        fn destroy_web_view(&mut self, id: WebViewId) {
            self.logs.push(format!("destroy {id:?}"));
        }

        fn show(&mut self, id: WebViewId, visible: bool) {
            self.logs.push(format!("show {id:?} = {visible}"));
        }

        fn raise_web_view(&mut self, id: WebViewId) {
            self.logs.push(format!("raise {id:?}"));
        }

        fn action(&mut self, id: WebViewId, action: WebAction) {
            self.logs.push(format!("action {id:?} {action:?}"));
        }

        fn probe_responsiveness(&mut self, id: WebViewId) {
            self.logs.push(format!("probe {id:?}"));
        }

        fn set_events(&mut self, _sink: Box<dyn FnMut(WebEvent)>) {}

        fn focus_native_web(&mut self, id: WebViewId) {
            self.logs.push(format!("focus web {id:?}"));
        }

        fn focus_host_chrome(&mut self) {
            self.logs.push("focus chrome".into());
        }

        fn live_session_count(&self) -> usize {
            0
        }
    }
}
