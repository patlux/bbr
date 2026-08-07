//! Real-app command-bar composition E2E (Phase 5, command-composition slice).
//!
//! Covers DESIGN.md §11.5 over real WKWebViews: Cmd-L selects the active URL
//! and typing replaces the selection; the deterministic debounce flush
//! publishes only the final generation of ranked results; Up/Down moves the
//! selection without leaving the field; Enter activates the selected
//! tab/space/action through the app/controller seams; Cmd-Enter opens the
//! destination in exactly one new tab/session and focuses it; Escape
//! restores the exact origin text and focus domain (including after a
//! repeated Cmd-L); the typed empty/loading/results/activation-error states
//! and the destructive action rows expose stable §15 ids and typed
//! snapshots; and cross-space selection — of standalone spaces and of warm
//! tabs in other spaces/profiles — preserves persistent identity, warm
//! sessions, and profile-store isolation. The search fallback is asserted by
//! its encoded intent, never the live provider. No sleeps: every step
//! synchronizes on semantic events or typed snapshot state under a deadline.
//!
//! The suite is split into focused scenario modules (`text`, `activation`,
//! `states`, `cross_space`) so every file stays near the ~500-line budget;
//! `just e2e-shell-command` still gates the one `e2e_shell_command` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;
#[path = "../e2e_semantics_common/mod.rs"]
mod e2e_semantics_common;

mod actions;
mod activation;
mod cross_space;
mod moved;
mod states;
mod text;

pub use bbr_testkit::protocol::{CommandBarMode, CommandResultState, SemanticRole};
pub use e2e_common::*;
pub use e2e_semantics_common::*;

/// Dispatch one keystroke through the production GPUI input path.
pub fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Type one character through the production key-down handler (the `->`
/// syntax gives GPUI a key with a key-char, so the app's replace-selection
/// typing path runs).
pub fn type_char(client: &mut Client, ch: char) {
    key(client, &format!("{ch}->{ch}"));
}

/// Open the location bar through the production Cmd-L path and assert the
/// focused selection state.
pub fn cmd_l(client: &mut Client, since: &mut u64) -> bbr_testkit::protocol::Snapshot {
    key(client, "cmd-l");
    *since = wait_chrome_focus(client, true, *since);
    let snapshot = client.snapshot().expect("cmd-l snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Location);
    assert!(snapshot.chrome_focused);
    snapshot
}

/// The persistent `tab:<id>` result key of the tab hosting `url` (unique in
/// the corpus; E2E asserts identity through the persistent key, never a
/// position). `space` disambiguates when the URL appears in several spaces.
pub fn tab_key(
    snapshot: &bbr_testkit::protocol::Snapshot,
    url: &str,
    space: Option<bbr_testkit::protocol::TestSpaceId>,
) -> String {
    let tab = snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.url == url && space.is_none_or(|space| tab.space == space))
        .unwrap_or_else(|| panic!("no tab for {url}"));
    format!("tab:{}", tab.id)
}

/// The typed snapshot of the currently published command ranking, asserting
/// the state machine reports published results.
pub fn published_results(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    let snapshot = client.snapshot().expect("published results snapshot");
    assert_eq!(snapshot.command_result_state, CommandResultState::Results);
    snapshot
}
