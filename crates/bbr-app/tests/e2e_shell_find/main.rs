//! Real-app find-in-page E2E (task packet `find-in-page-portal-bar`).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the find
//! bar end to end through the typed testkit protocol over the production
//! seams (DESIGN.md §13/§15, task packet evidence section):
//!
//! - `Cmd-F` (the real GPUI key binding) opens one `NativeAboveWeb` find
//!   presentation anchored to the content area's top-right edge; the bar
//!   wins real AppKit hit testing above the active WKWebView (owner-
//!   classified Window Server evidence, the merged portal suite's
//!   contract); one query reports the exact WebKit-computed count and
//!   ordinal;
//! - Enter/Shift-Enter navigation wraps at the boundaries; a replacement
//!   query restarts deterministically; zero matches report count 0;
//! - Escape (and the close path) restores the captured origin focus domain
//!   (web or chrome) and clears the presentation, leaving tab/session/
//!   WebView identity and profile/space untouched;
//! - a navigation of the bound view invalidates the report in place (stale)
//!   without closing the bar; a page content mutation is observed by the
//!   strategy and reported stale;
//! - resizing the parent re-pins the native portal frame without recreating
//!   the portal window or any web session; switching tabs ends the
//!   one-per-window presentation;
//! - the strategy runs in `WKContentWorld::defaultClientWorld`: a fixture
//!   page that records every unexpected global it can see reports none
//!   after a find run (page-world isolation).
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state or
//! semantic events under a deadline. The shared harness lives in
//! `e2e_common`; the native-evidence contract is the merged portal suite's.
//! The suite is split into focused scenario modules (`open_navigation`,
//! `focus_stale`, `resize_isolation`) so every file stays near the ~500
//! line budget; `just e2e-shell-find` still gates the one
//! `e2e_shell_find` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod focus_stale;
mod open_navigation;
mod resize_isolation;
mod support;
