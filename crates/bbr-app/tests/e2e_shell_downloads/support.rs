//! Shared support for the downloads-panel real-app E2E (task packet
//! `downloads-sidebar-panel`, DESIGN.md §11.12/§15).
//!
//! The suite drives the real `bbr` app over the typed testkit protocol:
//! `DownloadStart` / `DownloadCancel` / `DownloadRetry` route through the
//! production controller + native `WKDownload` adapter path, the footer
//! activation and the row actions are the exact production seams, and every
//! synchronization step polls the typed snapshot under a deadline — no
//! sleeps, no pixel assertions. `wait_download_*` helpers poll until the
//! L6 machine state the scenario needs is observable (the deterministic
//! fixture server delivers known/unknown/slow/zero/fail bodies).

use std::time::{Duration, Instant};

use bbr_testkit::protocol::{DownloadMachineState, DownloadRowSnapshot, Request, Snapshot};
use bbr_testkit::Client;

/// One typed `DispatchKeystroke` through the real key handler.
pub fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

/// Draw one real GPUI frame so render-derived state (the panel's bounded
/// window, the scroll anchor, the visible ids) advances.
pub fn flush_frame(client: &mut Client) {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
}

/// Open the anchored popover through the production footer activation
/// (`ClickFooterDownloads` → `toggle_downloads` → `downloads_open`) and
/// return the frame that rendered it open.
pub fn open_downloads(client: &mut Client) -> Snapshot {
    client
        .send_ok(&Request::ClickFooterDownloads)
        .expect("click footer downloads");
    flush_frame(client);
    let snapshot = client.snapshot().expect("downloads open snapshot");
    assert!(snapshot.downloads.open, "panel opened");
    snapshot
}

/// Poll until the panel is fully closed (bounded by the suite deadline).
pub fn wait_downloads_closed(client: &mut Client) -> Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("downloads closed snapshot");
        if !snapshot.downloads.open {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "downloads panel did not close: {:?}",
            snapshot.downloads
        );
        std::thread::yield_now();
    }
}

/// Start a download through the production controller + native adapter path
/// and return the minted attempt id (observed as a `Waiting` row).
pub fn start_download(client: &mut Client, url: &str, destination: &str) -> u64 {
    client
        .send_ok(&Request::DownloadStart {
            url: url.into(),
            destination: destination.into(),
        })
        .expect("download start");
    // The controller mints the attempt synchronously; poll for the Waiting
    // row with the suggested filename.
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("waiting row snapshot");
        if let Some(row) = snapshot
            .download_rows
            .iter()
            .find(|row| row.filename == destination)
        {
            assert_eq!(row.machine_state, DownloadMachineState::Waiting);
            assert_eq!(row.action, bbr_testkit::protocol::DownloadRowAction::Cancel);
            return row.id;
        }
        assert!(
            Instant::now() < until,
            "waiting row {destination} never appeared: {:#?}",
            snapshot.download_rows
        );
        std::thread::yield_now();
    }
}

/// Poll until the attempt `id` reaches `machine_state` and return its row
/// snapshot (the terminal-state waiter; the first terminal event wins, so
/// polling for a terminal state is deterministic).
pub fn wait_row_terminal(
    client: &mut Client,
    id: u64,
    machine_state: DownloadMachineState,
) -> DownloadRowSnapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("row terminal snapshot");
        if let Some(row) = snapshot.download_rows.iter().find(|row| row.id == id) {
            if row.machine_state == machine_state {
                return row.clone();
            }
            // A different terminal state means the first terminal event won
            // with a different outcome — fail loudly instead of hanging.
            if matches!(
                machine_state,
                DownloadMachineState::Succeeded
                    | DownloadMachineState::Failed
                    | DownloadMachineState::Cancelled
            ) && matches!(
                row.machine_state,
                DownloadMachineState::Succeeded
                    | DownloadMachineState::Failed
                    | DownloadMachineState::Cancelled
            ) {
                panic!(
                    "attempt {id} reached {:?} but {machine_state:?} was expected",
                    row.machine_state
                );
            }
        }
        assert!(
            Instant::now() < until,
            "attempt {id} never reached {machine_state:?}: {:#?}",
            snapshot.download_rows
        );
        std::thread::yield_now();
    }
}

/// Poll until the attempt `id` is `Running` with an observable known-length
/// fraction strictly inside (0, 1) — deterministic slow-progress evidence.
pub fn wait_running_progress(client: &mut Client, id: u64) -> DownloadRowSnapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("running progress snapshot");
        if let Some(row) = snapshot.download_rows.iter().find(|row| row.id == id) {
            match row.progress_fraction {
                Some(fraction) if fraction > 0.0 && fraction < 1.0 => return row.clone(),
                Some(_) | None => {
                    // Still Waiting, or already terminal — keep polling.
                }
            }
        }
        assert!(
            Instant::now() < until,
            "attempt {id} never ran with a partial fraction: {:#?}",
            snapshot.download_rows
        );
        std::thread::yield_now();
    }
}

/// Poll until the attempt `id` is `Running` (any fraction, including the
/// unknown-length indeterminate case where `progress_fraction` is `None`).
pub fn wait_running(client: &mut Client, id: u64) -> DownloadRowSnapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("running snapshot");
        if let Some(row) = snapshot.download_rows.iter().find(|row| row.id == id) {
            if row.machine_state == DownloadMachineState::Running {
                return row.clone();
            }
        }
        assert!(
            Instant::now() < until,
            "attempt {id} never ran: {:#?}",
            snapshot.download_rows
        );
        std::thread::yield_now();
    }
}

/// The row snapshot for `id` in a snapshot (panics with context when gone).
pub fn row_by_id(snapshot: &Snapshot, id: u64) -> DownloadRowSnapshot {
    snapshot
        .download_rows
        .iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("no row for attempt {id}: {:#?}", snapshot.download_rows))
        .clone()
}

/// The §11.12 invariant that downloads chrome never changes tab, session,
/// WebView, profile, or space identity.
pub fn assert_panel_identity(
    snapshot: &Snapshot,
    view: bbr_web::WebViewId,
    sessions: usize,
    tabs: usize,
    spaces: usize,
    profile: bbr_web::ProfileId,
    space: bbr_testkit::protocol::TestSpaceId,
) {
    assert_eq!(
        snapshot.active_web_view,
        Some(view),
        "the downloads surface must never change the active WebViewId"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions,
        "the downloads surface must never recreate sessions"
    );
    assert_eq!(
        snapshot.tabs.len(),
        tabs,
        "the downloads surface must never change the tab list"
    );
    assert_eq!(
        snapshot.spaces.len(),
        spaces,
        "the downloads surface must never change the space list"
    );
    assert_eq!(
        snapshot.active_profile, profile,
        "the downloads surface must never change the profile"
    );
    assert_eq!(
        snapshot.active_space, space,
        "the downloads surface must never change the active space"
    );
}
