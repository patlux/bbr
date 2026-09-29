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

#[cfg(target_os = "macos")]
pub mod responsiveness;

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
pub use external_urls_launch_services::{request_default_browser, DefaultBrowserError};

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
