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
pub mod delegate;

#[cfg(target_os = "macos")]
pub mod host;

#[cfg(target_os = "macos")]
pub mod portal;

#[cfg(target_os = "macos")]
mod portal_deferred;

#[cfg(target_os = "macos")]
pub mod portal_frame;

#[cfg(target_os = "macos")]
pub mod webview;

#[cfg(target_os = "macos")]
pub use portal::NativePortal;

#[cfg(target_os = "macos")]
pub use webview::{EngineOptions, WebEngineWk};
