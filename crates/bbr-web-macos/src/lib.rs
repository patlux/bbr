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
pub mod find_adapter;

#[cfg(target_os = "macos")]
pub mod find_mapping;

#[cfg(target_os = "macos")]
pub mod find_strategy;

#[cfg(target_os = "macos")]
pub mod find_web_kit;

#[cfg(target_os = "macos")]
pub mod host;

#[cfg(target_os = "macos")]
pub mod new_window;

#[cfg(target_os = "macos")]
pub mod portal;

#[cfg(target_os = "macos")]
mod portal_deferred;

#[cfg(target_os = "macos")]
pub mod portal_frame;

#[cfg(target_os = "macos")]
pub mod webview;

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
pub use new_window::{NewWindowContext, NewWindowCoordinator};

#[cfg(target_os = "macos")]
pub use portal::NativePortal;

#[cfg(target_os = "macos")]
pub use webview::{EngineOptions, WebEngineWk};
