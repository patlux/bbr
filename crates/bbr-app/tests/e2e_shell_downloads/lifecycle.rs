//! Transfer lifecycle scenarios over real WKWebView downloads (task packet
//! `downloads-sidebar-panel`, DESIGN.md §11.12 machine fidelity).
//!
//! The deterministic fixture server delivers the exact content shapes the
//! L6 contract requires: `slow.bin` (known length, trickled 8 KiB chunks —
//! monotonic progress on a stable window), `unknown.bin` (no
//! `Content-Length` — indeterminate bar, known total may become known),
//! `zero.bin` (zero bytes — succeeds with a full bar), and `fail.bin`
//! (HTTP 500 — `Failed`, the only retryable state). Every download runs
//! through the production controller + native adapter path in the real app;
//! progress is monotonic and a known total never reverts to unknown.

use bbr_testkit::protocol::{DownloadGroup, DownloadMachineState, DownloadRowAction};

use crate::e2e_common::*;
use crate::support::*;

/// `slow.bin` (known length): progress commits monotonic partial fractions,
/// then the attempt succeeds with a full bar, the resolved destination path,
/// and the Reveal-in-Finder action enabled on the existing file.
#[test]
#[ignore]
fn known_length_progress_is_monotonic_and_succeeds_with_path_and_reveal() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-lifecycle-known", &page_one);
    let slow_url = fixtures.url("/__fx/dl/slow.bin");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("active view");
        let sessions = initial.engine_sessions;
        let tabs = initial.tabs.len();
        let spaces = initial.spaces.len();
        let profile = initial.active_profile;
        let space = initial.active_space;

        // The panel may stay closed — the controller records states
        // regardless; the snapshot exposes the model rows either way. Open
        // it to observe the panel rendering too.
        let id = start_download(&mut client, &fixtures, &slow_url, "slow.bin");

        // Monotonic known-length progress: two partial observations, the
        // later one never behind the earlier one, and the total stays known.
        let first = wait_running_progress(&mut client, id);
        let first_fraction = first.progress_fraction.expect("known total");
        let second = wait_running_progress(&mut client, id);
        let second_fraction = second.progress_fraction.expect("known total stays known");
        assert!(
            second_fraction >= first_fraction,
            "progress is monotonic: {first_fraction} then {second_fraction}"
        );
        assert!(
            first_fraction > 0.0 && first_fraction < 1.0,
            "the slow fixture runs an observable partial fraction"
        );
        assert_eq!(first.group, DownloadGroup::Active);
        assert_eq!(first.action, DownloadRowAction::Cancel);

        // The terminal event wins once: Succeeded with a full bar.
        let succeeded = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
        assert_eq!(
            succeeded.progress_fraction,
            Some(1.0),
            "full bar at success"
        );
        assert_eq!(succeeded.group, DownloadGroup::History);
        assert_eq!(succeeded.machine_state, DownloadMachineState::Succeeded);
        assert!(
            succeeded.resolved_path.is_some(),
            "the resolved destination path is recorded"
        );
        assert!(
            succeeded.meta.contains("Completed"),
            "status copy: {}",
            succeeded.meta
        );
        assert_eq!(succeeded.action, DownloadRowAction::Reveal);
        assert!(
            succeeded.action_enabled,
            "the written file exists → Reveal-in-Finder enabled ({:?})",
            succeeded.action_reason
        );

        // The open panel renders the history row with the §11.12 ids and the
        // machine/group/action snapshot fields.
        let opened = open_downloads(&mut client);
        let row = row_by_id(&opened, id);
        assert_eq!(row.machine_state, DownloadMachineState::Succeeded);
        assert_eq!(row.group, DownloadGroup::History);
        assert_eq!(row.filename, "slow.bin");
        assert_ids(
            &opened,
            &[
                "bbr-downloads-panel",
                "bbr-downloads-history",
                "bbr-download-row-0",
                "bbr-download-status-0",
                "bbr-download-name-0",
                "bbr-download-meta-0",
                "bbr-download-action-0",
            ],
        );
        assert!(
            !opened
                .visible_ids
                .iter()
                .any(|id| id == "bbr-downloads-active"),
            "the empty Active group hides its label"
        );
        // §11.12 / Paper M-4: a terminal row names size and outcome only —
        // no progress bar.
        assert!(
            !opened
                .visible_ids
                .iter()
                .any(|id| id == "bbr-download-progress-0"),
            "terminal rows carry no progress bar"
        );
        // The painted panel holds its whole content: the History label, one
        // 48 pt History row and the Clear control inside the 6 pt padding
        // (Paper M-4), never rows squeezed into a shorter panel. The probe
        // reports the padding box: 6 pt each side at rest, 5 pt while the
        // 2 pt focus ring takes one point from the padding.
        let painted = wait_downloads_painted(&mut client);
        let layout = bbr_ui::surfaces::downloads::PanelLayout::for_theme(
            &bbr_ui::theme::Theme::new(
                bbr_ui::theme::ThemeMode::Light,
                bbr_ui::theme::ContrastMode::Standard,
                bbr_ui::theme::Density::Normal,
            ),
            0,
            1,
        );
        let content = layout.content_height() + 32.0;
        let padding = painted.rect[3] - content;
        assert!(
            (padding - 12.0).abs() < 0.5 || (padding - 10.0).abs() < 0.5,
            "panel interior {:?} holds label + row + Clear ({content}) plus its padding",
            painted.rect
        );
        assert_panel_identity(&opened, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// `unknown.bin` runs indeterminate (unknown length → `progress_fraction`
/// `None`) and still succeeds; `zero.bin` succeeds with a full bar; `fail.bin`
/// fails with the Retry action — the only retryable state.
#[test]
#[ignore]
fn unknown_length_zero_bytes_and_failure_reach_their_terminal_states() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-lifecycle-terminal", &page_one);
    let unknown_url = fixtures.url("/__fx/dl/unknown.bin");
    let zero_url = fixtures.url("/__fx/dl/zero.bin");
    let fail_url = fixtures.url("/__fx/dl/fail.bin");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("active view");
        let sessions = initial.engine_sessions;
        let tabs = initial.tabs.len();
        let spaces = initial.spaces.len();
        let profile = initial.active_profile;
        let space = initial.active_space;

        // Unknown length: Running with an indeterminate bar (fraction None),
        // then the known total may become known at success (full bar).
        let unknown_id = start_download(&mut client, &fixtures, &unknown_url, "unknown.bin");
        let running = wait_running(&mut client, unknown_id);
        assert_eq!(
            running.progress_fraction, None,
            "unknown length runs the indeterminate bar (transferred only)"
        );
        assert_eq!(running.group, DownloadGroup::Active);
        let unknown_done =
            wait_row_terminal(&mut client, unknown_id, DownloadMachineState::Succeeded);
        assert_eq!(unknown_done.filename, "unknown.bin");
        assert_eq!(unknown_done.progress_fraction, Some(1.0));
        assert!(unknown_done.resolved_path.is_some());

        // Zero bytes: succeeds with a full bar (`fraction()` is never NaN).
        let zero_id = start_download(&mut client, &fixtures, &zero_url, "zero.bin");
        let zero_done = wait_row_terminal(&mut client, zero_id, DownloadMachineState::Succeeded);
        assert_eq!(zero_done.filename, "zero.bin");
        assert_eq!(
            zero_done.progress_fraction,
            Some(1.0),
            "a zero-byte download succeeds with a full bar"
        );
        assert_eq!(zero_done.group, DownloadGroup::History);

        // Failure: HTTP 500 → Failed with the Retry action enabled.
        let fail_id = start_download(&mut client, &fixtures, &fail_url, "fail.bin");
        let failed = wait_row_terminal(&mut client, fail_id, DownloadMachineState::Failed);
        assert_eq!(failed.filename, "fail.bin");
        assert_eq!(failed.group, DownloadGroup::History);
        assert_eq!(failed.action, DownloadRowAction::Retry);
        assert!(failed.action_enabled, "failed rows are retryable");

        // Deterministic ordering: History is most-recent-terminal first, so
        // the newest terminal row is row index 0 and the oldest is last.
        let ordered = client.snapshot().expect("ordering snapshot");
        assert_eq!(ordered.download_rows.len(), 3);
        assert_eq!(
            ordered.download_rows[0].id, fail_id,
            "most-recent-terminal first"
        );
        assert_eq!(
            ordered.download_rows[2].id, unknown_id,
            "oldest terminal last"
        );
        assert!(
            ordered
                .download_rows
                .iter()
                .all(|row| row.group == DownloadGroup::History),
            "all three attempts are terminal"
        );
        assert_panel_identity(&ordered, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
