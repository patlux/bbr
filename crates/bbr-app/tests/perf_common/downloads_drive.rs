//! Downloads-panel scenario driving (task packet `downloads-sidebar-panel`)
//! for the warm interaction matrix.
//!
//! Owns the downloads matrix block: the production footer `[Downloads]` open
//! (`downloads_panel_open`), the settled §17.7 panel Clear
//! (`downloads_clear`), the production L6 progress commit
//! (`download_progress_commit`), the retry row action (`download_row_retry`),
//! and the settled §17.8 cancelled-row actions (`download_row_restart`,
//! `download_row_delete`) — each with 30 accepted samples through the typed
//! benchmark requests, committing only from a render of the anchored panel
//! that carried the armed identity (fast-but-wrong guard). Fixture/network
//! time is excluded by construction: the panel history is seeded with real
//! deterministic fixture downloads before any sample (known.bin succeeds,
//! fail.bin fails), and the progress/retry/restart/delete/clear samples
//! apply deterministic L6 events in-process (never through a network
//! transfer). Clear runs first among the samples: production Clear keeps
//! live (non-terminal) attempts, so its canonical empty-presentation
//! commitment needs a panel that still holds only terminal rows. The block
//! restores the matrix state — panel closed, tabs/sessions untouched (the
//! panel never changes tab, session, WebView, profile, or space identity) —
//! before the dense seeding that follows.
//!
//! Split from `drive.rs` so the downloads additions live in a focused module
//! under the ~500-line budget (same pattern as `command_drive.rs` /
//! `find_drive.rs`).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{DownloadMachineState, Request};
use bbr_testkit::Client;
use bbr_web::WebViewId;

use super::downloads::{
    benchmark_downloads_clear, benchmark_downloads_panel_open, benchmark_downloads_progress,
    benchmark_downloads_row_delete, benchmark_downloads_row_restart, benchmark_downloads_row_retry,
};
use super::harness::AppHarness;
use super::scenarios::TrialConfig;

/// The deterministic known-total fixture used to seed the panel history and
/// the progress attempt (64 KiB, `Content-Length` present).
const KNOWN_PATH: &str = "/__fx/dl/known.bin";
/// The deterministic failure fixture (HTTP 500).
const FAIL_PATH: &str = "/__fx/dl/fail.bin";
/// The declared total of the progress attempt (the fixture's 64 KiB).
const BENCH_TOTAL: u64 = 65536;
/// The per-sample progress step (64 KiB / 30 samples ≈ 2 KiB, monotonic).
const PROGRESS_STEP: u64 = 2048;

/// Drive the downloads matrix block and return the raw samples per
/// scenario. Runs after the find block and before the dense seeding; state
/// before it: the default space's two warm fixture tabs, no downloads
/// surface. The block seeds a small real history, measures the six warm
/// scenarios, then restores the matrix state (panel closed, the same tabs
/// and sessions).
#[allow(clippy::too_many_arguments)]
pub(super) fn drive_downloads_scenarios(
    client: &mut Client,
    harness: &mut AppHarness,
    since: &mut u64,
    config: &TrialConfig,
    fixtures_base: &str,
    operation: &mut u64,
) -> BTreeMap<String, Vec<u64>> {
    let start = client.snapshot().expect("downloads block start snapshot");
    let view = start.active_web_view.expect("active fixture view");
    let sessions_before = start.engine_sessions;
    let tabs_before = start.tabs.len();

    let known_url = format!("{fixtures_base}{KNOWN_PATH}");
    let fail_url = format!("{fixtures_base}{FAIL_PATH}");

    // Unmeasured setup: a real deterministic history — known.bin succeeds
    // (64 KiB written into the profile downloads dir), fail.bin fails (HTTP
    // 500). Both run through the real WKDownload lifecycle; the panel is
    // never open yet.
    client
        .send_ok(&Request::DownloadStart {
            url: known_url.clone(),
            destination: "known.bin".into(),
        })
        .expect("start known download");
    let _known_id = wait_download_row(client, DownloadMachineState::Succeeded, None);

    client
        .send_ok(&Request::DownloadStart {
            url: fail_url.clone(),
            destination: "fail.bin".into(),
        })
        .expect("start fail download");
    let _failed_id = wait_download_row(client, DownloadMachineState::Failed, None);
    // The succeeded row's resolved path exists on disk (testkit adapter
    // folder): the reveal eligibility re-check on every open is real.
    let _ = client.snapshot().expect("history snapshot");

    // downloads_panel_open × N: each sample is a fresh production open from
    // the closed panel; the sample commits when the main-window render drew
    // the panel open with the keyboard cursor. Escape (unmeasured) closes
    // the panel between samples so every sample measures the open
    // transition, never a re-focus.
    let mut open_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let measured = benchmark_downloads_panel_open(client, since, *operation, view);
        open_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("downloads_panel_open".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape after downloads open sample");
        wait_downloads_closed(client);
    }

    // downloads_clear × N: the settled §17.7 Clear panel action — the
    // controller history is emptied and the empty slice persisted durably.
    // Clear must run first in the downloads block, in its canonical state:
    // the panel currently holds only terminal rows (the two real seeded
    // records; later samples re-seed one deterministic terminal row
    // in-process, unmeasured). Production Clear keeps live (non-terminal)
    // attempts, so measuring it later — with the in-process progress/retry/
    // restart rows still alive — could never present the empty panel; the
    // canonical empty-presentation commitment stays here.
    client
        .send_ok(&Request::ClickFooterDownloads)
        .expect("open panel for clear benchmarks");
    wait_downloads_open(client, view);
    let mut clear_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        if sample > 0 {
            client
                .send_ok(&Request::SeedDownloadsCancelled {
                    url: known_url.clone(),
                    destination: "clear.bin".into(),
                    total: BENCH_TOTAL,
                })
                .expect("seed clear attempt");
        }
        let measured = benchmark_downloads_clear(client, since, *operation, view);
        clear_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("downloads_clear".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // download_progress_commit × N: the panel is already open (from the
    // clear block), so seed a deterministic non-transferring running
    // attempt at the known total directly, then apply 30 monotonic progress
    // events through the production event path. Each sample commits only
    // when the panel render drew the running row at the armed fraction; no
    // network transfer runs, so fixture time is excluded by construction.
    client
        .send_ok(&Request::SeedDownloadsAttempt {
            url: known_url.clone(),
            destination: "bench.bin".into(),
            total: BENCH_TOTAL,
        })
        .expect("seed running attempt");
    let seeded = client.snapshot().expect("seeded attempt snapshot");
    let running = seeded
        .download_rows
        .iter()
        .find(|row| {
            row.machine_state == DownloadMachineState::Running
                && row.group == bbr_testkit::protocol::DownloadGroup::Active
        })
        .unwrap_or_else(|| panic!("no running row after seed: {:#?}", seeded.download_rows));
    let running_id = running.id;

    let mut progress_samples = Vec::with_capacity(config.interaction_samples);
    for sample in 0..config.interaction_samples {
        let received = (sample + 1) as u64 * PROGRESS_STEP;
        let measured = benchmark_downloads_progress(
            client,
            since,
            *operation,
            running_id,
            received,
            BENCH_TOTAL,
            view,
        );
        progress_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("download_progress_commit".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // download_row_retry × N: retry a deterministic failed row through the
    // production retry seam; each sample commits only when the panel render
    // drew the fresh Waiting attempt in the Active group linked via
    // `retry_of`. The earlier Clear removed the original failed row, so one
    // deterministic failed row is re-seeded through the real fail.bin
    // download (unmeasured) before the samples; the failed row is
    // re-looked-up before every sample (its index grows by one per retry as
    // fresh attempts join the Active group), so the target is always the
    // canonical Failed row.
    client
        .send_ok(&Request::DownloadStart {
            url: fail_url.clone(),
            destination: "fail.bin".into(),
        })
        .expect("re-seed fail download");
    let _re_failed = wait_download_row(client, DownloadMachineState::Failed, None);
    let mut retry_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let before = client.snapshot().expect("pre retry lookup snapshot");
        let failed = before
            .download_rows
            .iter()
            .find(|row| row.machine_state == DownloadMachineState::Failed)
            .unwrap_or_else(|| panic!("no failed row: {:#?}", before.download_rows));
        let measured =
            benchmark_downloads_row_retry(client, since, *operation, failed.row, failed.id, view);
        retry_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("download_row_retry".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // download_row_restart × N: the settled §17.8 cancelled-row action —
    // the same machine path as a failed retry, labelled "Restart" by the
    // panel. A deterministic cancelled attempt is seeded in-process
    // (unmeasured; no network transfer runs) and each sample restarts it
    // through the production seam, committing only when the panel render
    // drew the fresh Waiting attempt linked via `retry_of`. The cancelled
    // source row is re-looked-up before every sample (its index grows as
    // fresh attempts join the Active group).
    let mut restart_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        let cancelled = client
            .snapshot()
            .expect("pre restart lookup snapshot")
            .download_rows
            .iter()
            .find(|row| row.machine_state == DownloadMachineState::Cancelled)
            .cloned()
            .unwrap_or_else(|| {
                client
                    .send_ok(&Request::SeedDownloadsCancelled {
                        url: known_url.clone(),
                        destination: "restart.bin".into(),
                        total: BENCH_TOTAL,
                    })
                    .expect("seed cancelled attempt");
                let snapshot = client.snapshot().expect("seeded cancelled snapshot");
                snapshot
                    .download_rows
                    .iter()
                    .find(|row| row.machine_state == DownloadMachineState::Cancelled)
                    .cloned()
                    .expect("cancelled row after seed")
            });
        let measured = benchmark_downloads_row_restart(
            client,
            since,
            *operation,
            cancelled.row,
            cancelled.id,
            view,
        );
        restart_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("download_row_restart".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // download_row_delete × N: the settled §17.8 cancelled-row Delete —
    // exactly that one terminal record is removed and persisted durably.
    // A deterministic cancelled attempt is seeded in-process before each
    // sample (unmeasured), and the sample commits only when the panel
    // render no longer draws the deleted row.
    let mut delete_samples = Vec::with_capacity(config.interaction_samples);
    for _ in 0..config.interaction_samples {
        client
            .send_ok(&Request::SeedDownloadsCancelled {
                url: known_url.clone(),
                destination: "delete.bin".into(),
                total: BENCH_TOTAL,
            })
            .expect("seed delete attempt");
        let snapshot = client.snapshot().expect("delete lookup snapshot");
        let cancelled = snapshot
            .download_rows
            .iter()
            .find(|row| row.machine_state == DownloadMachineState::Cancelled)
            .unwrap_or_else(|| panic!("no cancelled row: {:#?}", snapshot.download_rows));
        let measured = benchmark_downloads_row_delete(
            client,
            since,
            *operation,
            cancelled.row,
            cancelled.id,
            view,
        );
        delete_samples.push(measured.duration_ns);
        harness
            .raw_log
            .interactions
            .entry("download_row_delete".into())
            .or_default()
            .push(measured.duration_ns);
        *operation += 1;
    }

    // downloads_clear ran first above (its canonical empty presentation
    // needs a terminal-only panel); the later blocks keep the panel open
    // through progress/retry/restart/delete.

    // Restoration: Escape closes the panel (focus returns to the footer
    // control); the downloads surface never touched tabs, sessions, the
    // active view, or the space, so the matrix state is exactly what the
    // dense seeding expected before the block.
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: "escape".into(),
        })
        .expect("escape after downloads block");
    wait_downloads_closed(client);
    let restored = client
        .snapshot()
        .expect("downloads block restored snapshot");
    assert!(!restored.downloads.open, "panel closed");
    assert_eq!(
        restored.active_web_view,
        Some(view),
        "the downloads surface never changes the active view"
    );
    assert_eq!(restored.engine_sessions, sessions_before);
    assert_eq!(restored.tabs.len(), tabs_before);

    let mut raw = BTreeMap::new();
    raw.insert("downloads_panel_open".to_string(), open_samples);
    raw.insert("download_progress_commit".to_string(), progress_samples);
    raw.insert("download_row_retry".to_string(), retry_samples);
    raw.insert("download_row_restart".to_string(), restart_samples);
    raw.insert("download_row_delete".to_string(), delete_samples);
    raw.insert("downloads_clear".to_string(), clear_samples);
    raw
}

/// Poll the snapshot until a download row with `machine_state` exists (or,
/// when `exclude` is given, until one that is not that row exists — used for
/// the second terminal state after the first one is already present).
fn wait_download_row(
    client: &mut Client,
    machine_state: DownloadMachineState,
    _exclude: Option<u64>,
) -> u64 {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("download row snapshot");
        if let Some(row) = snapshot
            .download_rows
            .iter()
            .find(|row| row.machine_state == machine_state)
        {
            return row.id;
        }
        assert!(
            Instant::now() < until,
            "download row {machine_state:?} never appeared: {:#?}",
            snapshot.download_rows
        );
        std::thread::yield_now();
    }
}

/// Poll until the downloads panel is fully closed (bounded by the harness
/// deadline).
fn wait_downloads_closed(client: &mut Client) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("downloads closed snapshot");
        if !snapshot.downloads.open {
            return;
        }
        assert!(
            Instant::now() < until,
            "downloads panel did not close: {:?}",
            snapshot.downloads
        );
        std::thread::yield_now();
    }
}

/// Poll until the downloads panel is open (bounded by the harness
/// deadline).
fn wait_downloads_open(client: &mut Client, view: WebViewId) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("downloads open snapshot");
        if snapshot.downloads.open && snapshot.active_web_view == Some(view) {
            return;
        }
        assert!(
            Instant::now() < until,
            "downloads panel did not open: {:?}",
            snapshot.downloads
        );
        std::thread::yield_now();
    }
}
