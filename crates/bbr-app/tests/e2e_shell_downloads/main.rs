//! Real-app downloads sidebar panel E2E (task packet `downloads-sidebar-panel`).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the anchored
//! downloads panel end to end over the typed testkit protocol (DESIGN.md
//! §11.12/§15, packet evidence section):
//!
//! - the fixed footer `[Downloads]` control opens `bbr-downloads-panel`
//!   (empty hint + hidden group labels when empty; `bbr-download-*` ids),
//!   and Escape / outside press / re-activation dismiss it, returning focus
//!   to the footer control;
//! - deterministic fixture downloads (known / unknown length, slow progress,
//!   zero bytes, HTTP failure) run through the production controller +
//!   native adapter path: monotonic known-length progress, an indeterminate
//!   unknown-length bar, a zero-byte success with a full bar, failure with
//!   the Retry action, request/confirmation cancellation (the row stays
//!   active until the engine's terminal event lands), fresh retry attempts
//!   linked via `retry_of`, and Reveal-in-Finder enabled on the existing
//!   resolved path;
//! - bounded rendering: at most 6 visible rows with the §11.6 8-row overscan
//!   window (the full history is never laid out), deterministic ordering
//!   (Active ascending, History most-recent-terminal first), and wheel +
//!   keyboard traversal through the shared scroll state;
//! - opening and operating the panel never changes tab, session, WebView,
//!   profile, or space identity.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state under a
//! deadline. The shared harness lives in `e2e_common`; scenario modules stay
//! near the ~500 line budget; `just e2e-shell-downloads` gates the one
//! `e2e_shell_downloads` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod actions;
mod bounded;
mod floating;
mod lifecycle;
mod moments;
mod open_dismiss;
mod persistence;
mod support;
