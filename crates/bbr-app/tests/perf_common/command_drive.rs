//! Command-surface scenario driving (DESIGN.md §11.5) for the interaction
//! matrix.
//!
//! Owns the §11.5 matrix block: the production command-text path (Cmd-L
//! selects the active URL, typing replaces the selection, the real ranking
//! debounce publishes), the Enter activation of the selected exact-tab row,
//! and the Cmd-Enter new-tab activation (session-recreation class), plus the
//! unmeasured restoration back to the two original warm tabs the following
//! pointer-state block requires. The expected first-result keys derive from
//! the authoritative `all_tabs` collection (the reopen cycles reallocate
//! persistent ids). Split out of `drive.rs` so the command additions live in
//! a focused module under the ~500-line budget while the pre-existing
//! non-command matrix content stays where it was.

use std::collections::BTreeMap;

use bbr_testkit::protocol::{BrowserChange, CommandBarMode, Request};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::wait_browser_change;
use super::command::{
    benchmark_command_activate, benchmark_command_activate_new_tab, benchmark_command_type,
    benchmark_palette_open, command_setup,
};
use super::harness::AppHarness;
use super::scenarios::{TrialConfig, PAGE_ONE_TITLE};

/// The persistent `tab:<id>` command-result key of the tab hosting `url`,
/// read from the authoritative global tab collection (`all_tabs`). The
/// reopen cycles in the matrix reallocate persistent ids, so the expected
/// keys must be derived from the snapshot, never hard-coded positions.
pub(super) fn command_key(snapshot: &bbr_testkit::protocol::Snapshot, url: &str) -> String {
    let id = snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.url == url)
        .unwrap_or_else(|| panic!("no global tab for {url}"))
        .id;
    format!("tab:{id}")
}

/// Drive the §11.5 command-surface matrix block and return the raw samples
/// per command scenario. The block runs between the keyboard-activation and
/// the pointer-state blocks of the interaction matrix; state before it: two
/// warm tabs (page_one/page_two) with web focus and the session count at 2.
/// `tab0_view`/`tab1_view` are the live views after the reopen cycle; the
/// restoration at the end returns the matrix to those two original warm
/// tabs (2 sessions) with the surface closed in web focus.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_command_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    page_one: &str,
    page_two: &str,
    tab0_view: WebViewId,
    tab1_view: WebViewId,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    // Command typing never changes the active view: Cmd-L selects the
    // current URL and the debounce publishes the typed query's results
    // with the same tab presented, so every sample commits against the
    // view that was active before the loop (the keyboard-activation
    // block leaves tab 1 focused in web content). The expected first
    // result keys are derived from the authoritative global tab
    // collection (`all_tabs`): the reopen cycle reallocates persistent
    // ids, so a hard-coded `tab:0`/`tab:1` would reject every sample.
    let command_state = client.snapshot().expect("command type view");
    let command_type_view = command_state
        .active_web_view
        .expect("command type active view");
    let tab0_key = command_key(&command_state, page_one);
    let tab1_key = command_key(&command_state, page_two);
    let mut command_type_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (text, expected_key) = if sample % 2 == 0 {
            (&page_one, tab0_key.as_str())
        } else {
            (&page_two, tab1_key.as_str())
        };
        let measured = benchmark_command_type(
            client,
            since,
            *operation,
            text,
            expected_key,
            command_type_view,
        );
        command_type_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("command_type".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: Escape after every sample restores the web
        // origin (each sample then starts from web focus, and the next
        // Cmd-L captures that uniform origin).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after command type sample");
    }
    let escaped = client.snapshot().expect("escaped snapshot");
    assert_eq!(escaped.command_bar_mode, CommandBarMode::Inactive);
    assert!(!escaped.chrome_focused);

    // The §11.4 palette-open scenario: Cmd-T opens the centered command
    // palette in New Tab staging through the production seam; the sample
    // commits only from a render that actually constructed the palette
    // (the in-app fast-but-wrong guard rejects a missing or invented
    // surface). Staged semantics: no tab or session is created before
    // Enter, so the loop needs no tab restoration — only the unmeasured
    // Escape back to web focus between samples, so every sample starts
    // from the uniform web origin the command block requires.
    let palette_state = client.snapshot().expect("palette state");
    let palette_view = palette_state
        .active_web_view
        .expect("palette open active view");
    let palette_tabs = palette_state.tabs.len();
    let palette_sessions = palette_state.engine_sessions;
    let mut palette_open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured = benchmark_palette_open(
            client,
            since,
            *operation,
            palette_view,
            palette_tabs,
            palette_sessions,
        );
        palette_open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("palette_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        // Unmeasured setup: Escape after every sample restores the web
        // origin (each sample then starts from web focus, and the next
        // Cmd-T captures that uniform origin).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after palette open sample");
    }
    let palette_escaped = client.snapshot().expect("palette escaped snapshot");
    assert_eq!(
        palette_escaped.command_surface,
        bbr_testkit::protocol::CommandSurface::None
    );
    assert_eq!(palette_escaped.command_bar_mode, CommandBarMode::Inactive);
    assert!(!palette_escaped.chrome_focused);

    let mut command_activate_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let (text, expected_key, expected_view) = if sample % 2 == 0 {
            (&page_one, tab0_key.as_str(), tab0_view)
        } else {
            (&page_two, tab1_key.as_str(), tab1_view)
        };
        command_setup(client, text, expected_key);
        let measured = benchmark_command_activate(client, since, *operation, expected_view);
        command_activate_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("command_activate".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // Cmd-Enter: the selected exact-tab destination opens in exactly one
    // new tab/session and the fresh page is focused. The loop always
    // targets tab 0's URL so the destination (and its deterministic
    // fixture title) is stable across samples.
    let mut command_new_tab_samples = Vec::with_capacity(config.new_tab_samples);
    for _ in 0..config.new_tab_samples {
        command_setup(client, page_one, &tab0_key);
        let measured =
            benchmark_command_activate_new_tab(client, since, *operation, page_one, PAGE_ONE_TITLE);
        command_new_tab_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("command_activate_new_tab".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }
    // Unmeasured restoration: each Cmd-Enter appended a fresh tab at the
    // end and focused it, so closing the active tab through the
    // production Cmd-W path pops exactly the new tabs and returns the
    // matrix to the two original warm tabs (page_one + page_two, 2
    // sessions) the following pointer-state block requires.
    for _ in 0..config.new_tab_samples {
        let snapshot = client.snapshot().expect("restore snapshot");
        assert!(
            snapshot.tabs.len() > 2,
            "restore loop expects the appended new tabs"
        );
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-w".into(),
            })
            .expect("close appended new tab");
        *since = wait_browser_change(
            client,
            |kind| matches!(kind, BrowserChange::TabClosed { .. }),
            *since,
        )
        .0;
    }
    let restored = client.snapshot().expect("restored snapshot");
    assert_eq!(restored.tabs.len(), 2, "the two original tabs remain");
    assert_eq!(restored.engine_sessions, 2);
    assert!(!restored.chrome_focused);

    let mut raw = BTreeMap::new();
    raw.insert("command_type".to_string(), command_type_samples);
    raw.insert("command_activate".to_string(), command_activate_samples);
    raw.insert(
        "command_activate_new_tab".to_string(),
        command_new_tab_samples,
    );
    raw.insert("palette_open".to_string(), palette_open_samples);
    raw
}
