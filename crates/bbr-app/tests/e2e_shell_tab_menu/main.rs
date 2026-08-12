//! Real-app Tab/Favorite context-menu E2E (task packet
//! `tab-favorite-context-menus`; DESIGN.md §11.13/§15).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the menu
//! wiring end to end through the typed testkit protocol:
//!
//! - a secondary-click on a tab row or Favorite tile opens one production
//!   menu bound to the exact persistent `TabId` that invoked it (the
//!   Favorite is another presentation of the same concrete tab), anchored
//!   at the pointer and clamped to the window;
//! - opening, traversing, hovering, and dismissing never activate, select,
//!   move, reload, close, recreate, or change the profile/space/session of
//!   the source tab (snapshot byte-identity);
//! - keyboard semantics through the real GPUI input path: Up/Down skip
//!   disabled rows, Right/Left enter/leave submenus, Return commits, Escape
//!   dismisses; an outside pointer press dismisses with one action;
//! - every enabled action dispatches exactly one typed command against the
//!   bound `TabId`: pin/unpin, suspend, Favorite toggle, Copy Link (the
//!   real pasteboard carries the **source** URL), region ordering, close,
//!   reopen, `Move to Folder`, and same-/cross-profile `Move to Space`
//!   (same-profile keeps the live `WebViewId`; cross-profile recreates
//!   exactly once in the destination store);
//! - B2 bookmark parity: `Bookmark Page` / `Remove Bookmark` capture the
//!   bound source's current URL through the one app-owned Saved Page
//!   command without selecting it and without retaining the `TabId` in the
//!   record.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state or
//! semantic events under a deadline. The suite is split into focused
//! scenario modules (`open_dismiss`, `submenu`, `actions`, `bookmark`,
//! `favorite`) so every file stays under the ~500 line budget;
//! `just e2e-shell-tab-menu` gates the one `e2e_shell_tab_menu` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod actions;
mod bookmark;
mod favorite;
mod open_dismiss;
mod submenu;
mod support;
