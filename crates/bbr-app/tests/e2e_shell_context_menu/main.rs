//! Real-app BBR context-menu E2E (task packet `bbr-context-menus-over-web`,
//! queue item 12, part B).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the
//! context-menu wiring end to end through the typed testkit protocol:
//!
//! - the BBR page menu opens through the exact engine event transport the
//!   native bridge's `ContextMenuRequested` routes to, presented in the one
//!   `NativeAboveWeb` portal window (stable `0xC4` presentation, the §15
//!   backdrop/card/row ids, child attachment, frame sync, non-key/non-main
//!   policy);
//! - Back/Forward enablement is genuine: it mirrors the real WKWebView's
//!   `canGoBack()`/`canGoForward()` history availability fed through the
//!   real `CanGoBackChanged`/`CanGoForwardChanged` events (never a guess);
//! - the menu is navigable by pointer (hover/select + press/activate) AND
//!   keyboard (Up/Down skipping disabled rows, Enter activating, Escape
//!   dismissing) through the real GPUI input path;
//! - dismissing (Escape, the close seam, a disabled-row press) never
//!   creates, destroys, moves, or reselects a web session and restores the
//!   origin focus domain exactly;
//! - Copy Link writes the page URL to the system pasteboard (real `pbpaste`
//!   read-back) and Open in New Tab opens exactly one new tab/session;
//! - WebKit's own page menu stays native where specified: the real-app
//!   bridge is installed and observing right-clicks (typed diagnostics),
//!   and the page-vs-element classification is deterministic (adapter unit
//!   tests); the BBR open path never touches the ownership counters;
//! - resizing the parent while the menu is open re-pins the one portal
//!   host (same portal window, frame synced) and closing leaves no orphan
//!   (host back to its typed closed default).
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state or
//! semantic events under a deadline. The suite is split into focused
//! scenario modules (`navigation`, `dismissal`, `ownership`) so every file
//! stays under the ~500 line budget; `just e2e-shell-context-menu` gates
//! the one `e2e_shell_context_menu` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod dismissal;
mod navigation;
mod ownership;
mod support;
