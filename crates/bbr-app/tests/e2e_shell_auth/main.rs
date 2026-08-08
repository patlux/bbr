//! Real-app credential-sheet E2E (queue item 10).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the
//! conventional macOS credential sheet end to end through the typed testkit
//! protocol over the production seams (DESIGN.md §15, task-packet evidence
//! section):
//!
//! - a deterministic challenge arrival presents the sheet inside the one
//!   `NativeAboveWeb` portal window, centered, showing the realm and host
//!   of the protection space that asked, and wins real AppKit hit testing
//!   above the active WKWebView (owner-classified Window Server evidence,
//!   the merged portal suite's contract);
//! - typing edits the masked sheet state; Enter submits and Escape cancels
//!   through the merged portable contract — the sheet holds no authority;
//! - credentials never appear in any snapshot, log, event, or benchmark
//!   payload: the only representation that leaves the sheet is the
//!   contract's redaction (`credential:#<n>` / `cancelled`);
//! - a challenge belongs to its originating web view: a second tab's
//!   challenge can never be answered on behalf of its tab, and cancel fails
//!   the request rather than silently retrying;
//! - no tab/session/WebView identity, profile, or space changes.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state or
//! semantic events under a deadline. The suite is split into focused
//! modules (`flow`, `support`) so every file stays near the ~500 line
//! budget; `just e2e-auth` still gates the one `e2e_shell_auth` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;
#[path = "../e2e_semantics_common/mod.rs"]
mod e2e_semantics_common;

mod fallthrough;
mod flow;
mod support;
