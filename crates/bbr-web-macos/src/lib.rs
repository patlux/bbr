//! # bbr-web-macos — native WebKit bridge (macOS only)
//!
//! All Objective-C / AppKit / WebKit code lives here and only here. The rule
//! of this crate:
//!
//! 1. **Everything runs on the main thread.** `WKWebView` and the GPUI mac
//!    platform are main-thread-only. Every public entry point re-asserts this
//!    with `MainThreadMarker::new()`; a `None` means a bug (a panic in debug
//!    builds).
//! 2. **Unsafe is contained.** The raw GPUI root-view pointer is captured
//!    once ([`host::HostBridge`]) and only ever dereferenced on the main
//!    thread while the owning window is alive.
//! 3. **Portable code does not import this crate.** [`bbr_web`] defines the
//!    contract; `bbr-web-macos` implements it.
//!
//! On non-macOS targets this crate compiles to an empty library so the
//! workspace still resolves and evaluates.

/// Live Increase Contrast / Reduce Motion options (task packet
/// `v2-system-prefs`): public `NSWorkspace` reads plus the change
/// notification; the app's appearance authority owns the theme decision.
#[cfg(target_os = "macos")]
pub mod accessibility_display;

#[cfg(target_os = "macos")]
pub mod content_gesture;

#[cfg(target_os = "macos")]
mod gesture_machine;

#[cfg(target_os = "macos")]
pub mod content_probe;

#[cfg(target_os = "macos")]
pub mod delegate;

#[cfg(target_os = "macos")]
pub mod downloads_adapter;

#[cfg(target_os = "macos")]
pub mod external_urls_adapter;

#[cfg(target_os = "macos")]
pub mod external_urls_launch_services;

#[cfg(target_os = "macos")]
pub mod downloads_delegate;

#[cfg(target_os = "macos")]
pub mod downloads_destination;

#[cfg(target_os = "macos")]
pub mod downloads_mapping;

#[cfg(target_os = "macos")]
pub mod downloads_strategy;

#[cfg(target_os = "macos")]
pub mod downloads_web_kit;

#[cfg(target_os = "macos")]
pub mod error_mapping;

#[cfg(target_os = "macos")]
pub mod find_adapter;

#[cfg(target_os = "macos")]
pub mod find_mapping;

#[cfg(target_os = "macos")]
pub mod find_strategy;

#[cfg(target_os = "macos")]
pub mod find_web_kit;

/// Real favicon acquisition adapter (task packet
/// `real-favicon-pipeline.md`): the isolated content-world collector, the
/// deterministic best-candidate selection, and the bounded profile-bound
/// download that feeds [`WebEvent::FaviconChanged`] observations.
#[cfg(target_os = "macos")]
pub mod favicon_acquisition;

#[cfg(target_os = "macos")]
pub mod host;

/// The pure WebKit page-menu ownership classification (task packet
/// `bbr-context-menus-over-web`): given the item titles of the menu WebKit
/// constructed, is it the generic page menu (BBR-owned) or an element menu
/// (WebKit-owned)? Deterministic and conservative; unit-tested.
mod menu_classify;

#[cfg(target_os = "macos")]
pub mod menu_adapter;

#[cfg(target_os = "macos")]
mod navigation_cover;

/// Blueprint D15: Safari's bookmarks for the first-run import.
#[cfg(target_os = "macos")]
pub mod plist_bookmarks;

/// Blueprint D13: links to schemes BBR does not render.
#[cfg(target_os = "macos")]
pub mod external_schemes;

/// Blueprint D12: client-certificate requests and Keychain identities.
#[cfg(target_os = "macos")]
pub mod client_identities;

/// Blueprint D11: WebKit's camera/microphone requests and capture state.
#[cfg(target_os = "macos")]
pub mod permission_requests;

/// Blueprint D24: the media probes behind the sidebar media card.
#[cfg(target_os = "macos")]
mod media_adapter;

/// Blueprint D10: the local time zone's current offset from UTC in
/// seconds (the History page's day boundaries and times).
#[cfg(target_os = "macos")]
pub fn local_utc_offset_seconds() -> i64 {
    objc2_foundation::NSTimeZone::localTimeZone().secondsFromGMT() as i64
}

#[cfg(target_os = "macos")]
mod card_clip;

mod native_session;

#[cfg(target_os = "macos")]
mod session_history;

#[cfg(target_os = "macos")]
pub mod new_window;

#[cfg(target_os = "macos")]
pub mod portal;

#[cfg(target_os = "macos")]
mod portal_deferred;

#[cfg(target_os = "macos")]
mod portal_lifecycle;

#[cfg(target_os = "macos")]
pub mod portal_frame;

#[cfg(target_os = "macos")]
pub mod portal_hit;

/// Blueprint D06/D10: profile website-data removal.
#[cfg(target_os = "macos")]
mod profile_data;

/// Blueprint D02: the floating rail's pointer observer.
#[cfg(target_os = "macos")]
pub mod pointer_observer;

#[cfg(target_os = "macos")]
pub mod responsiveness;

/// Blueprint D02: the `NSTextInputClient` test driver (testkit only).
#[cfg(all(target_os = "macos", feature = "testkit"))]
pub mod text_input_client;

#[cfg(target_os = "macos")]
pub mod titlebar_geometry;

#[cfg(target_os = "macos")]
pub mod window_server;

#[cfg(target_os = "macos")]
pub mod webview;

#[cfg(target_os = "macos")]
pub mod zoom_adapter;

#[cfg(target_os = "macos")]
pub use downloads_adapter::{
    dispatch_action, AdapterDispatchError, AdapterError, DownloadEventSink, WebKitDownloadAdapter,
};

#[cfg(target_os = "macos")]
pub use external_urls_adapter::{ExternalUrlAdapterCore, DECLARED_URL_SCHEMES};

#[cfg(target_os = "macos")]
pub use external_urls_launch_services::{
    is_default_browser, request_default_browser, DefaultBrowserError,
};

#[cfg(target_os = "macos")]
pub use downloads_destination::{destination_plan, uniquify_name, DestinationPlan};

#[cfg(target_os = "macos")]
pub use downloads_mapping::{DownloadAdapterCore, NativeSignal};

#[cfg(target_os = "macos")]
pub use downloads_strategy::{DownloadSignalRouter, DownloadStrategy, RevealError};

#[cfg(target_os = "macos")]
pub use downloads_web_kit::{DownloadAdapterOptions, WebKitDownloadStrategy};

#[cfg(target_os = "macos")]
pub use find_adapter::WebKitFindAdapter;

#[cfg(target_os = "macos")]
pub use find_web_kit::WebKitFindStrategy;

#[cfg(target_os = "macos")]
pub use menu_adapter::{
    classify_menu, ContextMenuNativeDiagnostics, MenuOwnership, NativeContextMenuBridge,
};

#[cfg(target_os = "macos")]
pub use content_gesture::{ContentGestureNativeDiagnostics, NativeContentGestureBridge};

#[cfg(target_os = "macos")]
pub use new_window::{classify_input, NewWindowContext, NewWindowCoordinator};

#[cfg(target_os = "macos")]
pub use portal::NativePortal;

#[cfg(target_os = "macos")]
pub use webview::{EngineOptions, WebEngineWk};

#[cfg(target_os = "macos")]
pub use zoom_adapter::{page_zoom_factor, WebKitZoomAdapter};
