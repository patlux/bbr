//! Command-surface benchmark drivers (DESIGN.md §11.5) for the real-app
//! performance suites.
//!
//! `benchmark_command_type` measures the production command-text path:
//! Cmd-L selects the active URL, typing replaces the selection, and the real
//! ranking debounce publishes the current generation; the sample commits in
//! the app only when a render of the result surface presents the exact
//! first/selected key, generation, state, mode, and focus (the in-app probe
//! owns the render-inclusive guard; the driver asserts the measured state
//! and the typed snapshot). `benchmark_command_activate` /
//! `benchmark_command_activate_new_tab` measure the Enter / Cmd-Enter
//! activations through the same app/controller seams, with the destination
//! asserted from the accepted sample and snapshot. `command_setup` is the
//! unmeasured pre-activation setup (production Cmd-L + typing + real
//! debounce publication; the deterministic flush stays reserved for
//! functional E2E).
//!
//! This focused module owns the command additions so `benchmarks.rs` stays
//! at its pre-command size (its remaining scenarios are pre-existing
//! non-command content).

use std::time::{Duration, Instant};

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, CommandSurface, PresentationMeasurement,
    Request,
};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::benchmarks::{wait_measurement, wait_title};

/// Deterministic command-surface setup (unmeasured): open the location bar
/// through the production Cmd-L path and type `text` through the testkit
/// text seam (which restarts the ranking debounce exactly like a keystroke).
/// The real production debounce then publishes the current generation; this
/// helper waits for that publication (bounded deadline, no deterministic
/// flush — the flush stays reserved for functional E2E) and asserts the
/// published first result key plus the typed surface state. Used before
/// every measured command activation so the benchmark measures only the
/// activation itself, never the setup.
pub(crate) fn command_setup(client: &mut Client, text: &str, expected_first_key: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "cmd-l".into(),
        })
        .expect("cmd-l setup");
    let focused = client.snapshot().expect("location setup snapshot");
    assert_eq!(focused.command_bar_mode, CommandBarMode::Location);
    assert!(focused.chrome_focused, "Cmd-L focuses the field");
    assert!(
        focused.command_selection.is_some(),
        "Cmd-L selects the active URL"
    );
    client
        .send_ok(&Request::SetCommandBarText {
            text: text.to_string(),
        })
        .expect("command text setup");
    let published = wait_command_results_published(client, expected_first_key);
    assert_eq!(
        published.command_result_state,
        bbr_testkit::protocol::CommandResultState::Results,
        "the real debounce published ranked results"
    );
    let first = published
        .command_results
        .first()
        .expect("published first result");
    assert_eq!(
        first.key, expected_first_key,
        "the exact-tab destination ranks first"
    );
    assert!(first.selected, "the first result is the selected row");
}

/// Wait for the real ranking debounce to publish the current generation and
/// return the snapshot carrying the published results. Bounded by the
/// harness deadline; polls the typed snapshot (the publication is
/// deterministic, only its arrival time varies).
fn wait_command_results_published(
    client: &mut Client,
    expected_first_key: &str,
) -> bbr_testkit::protocol::Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("command results snapshot");
        if snapshot.command_result_state == bbr_testkit::protocol::CommandResultState::Results
            && snapshot
                .command_results
                .first()
                .is_some_and(|row| row.key == expected_first_key)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "command results did not publish within the deadline"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The production command-text path (Cmd-L selects the URL, typing replaces
/// the selection, the real ranking debounce publishes, and a render of the
/// result surface commits the sample in the app): the sample arrives only
/// when the correct first result key is presented, so a fast-but-wrong
/// rank/mode/focus or an omitted palette is rejected in the app before the
/// sample leaves.
pub(crate) fn benchmark_command_type(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    text: &str,
    expected_first_key: &str,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkCommandType {
            operation_id,
            text: text.to_string(),
            expected_first_key: expected_first_key.to_string(),
        })
        .expect("benchmark command type");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::CommandType, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "the typed surface keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Location);
    let snapshot = client.snapshot().expect("command type snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Location);
    assert!(snapshot.chrome_focused);
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.command_results.first().map(|row| row.key.as_str()),
        Some(expected_first_key),
        "the published first result is the expected exact-tab row"
    );
    measured
}

/// The production Enter activation of the command bar: the selected exact-tab
/// row activates through the app/controller seams and the expected
/// destination view is presented in web focus with the surface closed.
pub(crate) fn benchmark_command_activate(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkCommandActivate {
            operation_id,
            expected_view: Some(expected_view),
        })
        .expect("benchmark command activate");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::CommandActivate, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        !measured.chrome_focused,
        "Enter activates into the page (web focus)"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    let snapshot = client.snapshot().expect("command activate snapshot");
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Inactive);
    assert!(!snapshot.chrome_focused);
    measured
}

/// The production Cmd-Enter activation: the selected exact-tab destination
/// opens in exactly one new tab/session and the fresh page is focused — the
/// session-recreation class. Asserts the new tab/session counts and the
/// freshly presented view, then waits for the deterministic fixture title of
/// the opened destination.
pub(crate) fn benchmark_command_activate_new_tab(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    destination_url: &str,
    destination_title: &str,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    let before = client.snapshot().expect("before command new-tab snapshot");
    let before_tabs = before.tabs.len();
    let before_sessions = before.engine_sessions;
    let previous_view = before
        .active_web_view
        .expect("active view before Cmd-Enter");
    client
        .send_ok(&Request::BenchmarkCommandActivateNewTab { operation_id })
        .expect("benchmark command activate new tab");
    let (seq, measured) = wait_measurement(
        client,
        BenchmarkKind::CommandActivateNewTab,
        operation_id,
        *since,
    );
    *since = seq;
    let after = client.snapshot().expect("command new-tab snapshot");
    assert_eq!(after.tabs.len(), before_tabs + 1, "exactly one new tab");
    assert_eq!(
        after.engine_sessions,
        before_sessions + 1,
        "exactly one new live session"
    );
    assert_eq!(after.command_bar_mode, CommandBarMode::Inactive);
    assert!(!after.chrome_focused, "the fresh page owns focus");
    let new_view = after.active_web_view.expect("fresh view");
    assert_ne!(new_view, previous_view, "the fresh view is not the source");
    assert_eq!(measured.active_web_view, Some(new_view));
    assert!(!measured.chrome_focused);
    assert_eq!(measured.command_bar_mode, CommandBarMode::Inactive);
    assert_eq!(
        after.command_bar_text, destination_url,
        "the opened destination is the selected tab's URL"
    );
    // Deterministic content: the opened destination's fixture title presents.
    if after.title.as_deref() != Some(destination_title) {
        *since = wait_title(client, new_view, destination_title, *since);
    }
    measured
}

/// The production Cmd-T open of the centered command palette (DESIGN.md
/// §11.4 `bbr-palette`): the request routes through the production
/// `open_palette` seam in the app, and the sample commits only from a render
/// that actually constructed the palette with the staged New Tab mode and
/// chrome focus (the in-app fast-but-wrong guard rejects a missing or
/// invented surface). The driver asserts the measured surface state and the
/// typed snapshot: the palette surface open, the staged mode, chrome focus,
/// the stable `bbr-palette*` ids exposed, and the staged semantics (no tab
/// or session before Enter).
#[allow(clippy::too_many_arguments)]
pub(crate) fn benchmark_palette_open(
    client: &mut Client,
    since: &mut u64,
    operation: u64,
    expected_view: WebViewId,
    expected_tabs: usize,
    expected_sessions: usize,
) -> PresentationMeasurement {
    let operation_id = BenchmarkOperationId(operation);
    client
        .send_ok(&Request::BenchmarkPaletteOpen { operation_id })
        .expect("benchmark palette open");
    let (seq, measured) =
        wait_measurement(client, BenchmarkKind::PaletteOpen, operation_id, *since);
    *since = seq;
    assert_eq!(measured.active_web_view, Some(expected_view));
    assert!(
        measured.chrome_focused,
        "the palette keeps the chrome focus domain"
    );
    assert_eq!(measured.command_bar_mode, CommandBarMode::NewTab);
    let snapshot = client.snapshot().expect("palette open snapshot");
    assert_eq!(snapshot.command_surface, CommandSurface::Palette);
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::NewTab);
    assert!(snapshot.chrome_focused, "Cmd-T focuses the palette field");
    assert_eq!(snapshot.active_web_view, Some(expected_view));
    assert_eq!(
        snapshot.tabs.len(),
        expected_tabs,
        "staged semantics: no tab before Enter"
    );
    assert_eq!(
        snapshot.engine_sessions, expected_sessions,
        "staged semantics: no session before Enter"
    );
    assert!(
        snapshot.visible_ids.iter().any(|id| id == "bbr-palette"),
        "the centered palette ids are exposed: {:?}",
        snapshot.visible_ids
    );
    assert!(
        !snapshot
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-overlay"),
        "the field-anchored overlay never coexists with the palette"
    );
    measured
}
