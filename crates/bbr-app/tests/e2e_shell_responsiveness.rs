//! Real-app renderer responsiveness E2E (renderer-isolation packet).
//!
//! Modular scenarios exercise the production WKWebView probe, §11.8a pane,
//! normal tab/chrome actions, the documented dialog/beforeunload boundary,
//! and the productive typed crash/recovery transport. No pixel assertions or
//! unbounded waits.

#![cfg(target_os = "macos")]

#[path = "e2e_shell_responsiveness/crash.rs"]
mod crash;
#[path = "e2e_shell_responsiveness/dialogs.rs"]
mod dialogs;
#[path = "e2e_common/mod.rs"]
mod e2e_common;
#[path = "e2e_shell_responsiveness/hang.rs"]
mod hang;
#[path = "e2e_shell_responsiveness/recovery.rs"]
mod recovery;
#[path = "e2e_shell_responsiveness/support.rs"]
mod support;

pub use e2e_common::*;
pub use support::*;
