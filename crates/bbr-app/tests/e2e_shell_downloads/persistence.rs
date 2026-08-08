//! Downloads history persistence across real process restarts (DESIGN.md
//! §17.7/§17.8/§17.10 settled): terminal history restores into the
//! controller before the first frame, Clear and a row Delete are durable
//! (a cleared or deleted row can never return from the store), and restored
//! terminal history never inflates the §17.11 active-only footer badge.
//!
//! Each scenario reuses the same explicit profile directory across process
//! launches (`--bbr-profile-dir`, the production store path): the first
//! launch performs the downloads and lets the synchronous terminal-event
//! persist path write the store, the process is terminated, and the second
//! launch on the same profile proves the restored rows. No sleeps — every
//! step synchronizes on typed snapshot state under a deadline.

use bbr_testkit::protocol::{
    DownloadGroup, DownloadMachineState, DownloadRowAction, Request, StoreLoadSnapshot,
};

use crate::e2e_common::*;
use crate::support::*;

/// Succeeded + cancelled terminal history restores across a real process
/// restart on the same profile directory: the second launch renders the
/// restored rows (state, action, linkage, resolved path) from the first
/// frame, the startup disposition is `Loaded` (nothing discarded), the
/// History ordering survives, and the restored terminal rows never inflate
/// the footer badge. Operating the restored panel never changes identity.
#[test]
#[ignore]
fn terminal_history_restores_across_process_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("downloads-persist-restore", pid);
    let profile = root.join("profile");
    let slow_url = fixtures.url("/__fx/dl/slow.bin");
    let known_url = fixtures.url("/__fx/dl/known.bin");
    let profile_before = std::cell::RefCell::new(None);

    let (mut harness, socket, token) =
        spawn_with_profile("downloads-persist-restore", &page_one, &profile);
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
        *profile_before.borrow_mut() = Some(initial.active_profile);
        let space = initial.active_space;
        assert_eq!(
            initial.store.store_load,
            StoreLoadSnapshot::Loaded,
            "the first launch creates and loads the v2 store"
        );

        // One succeeded + one cancelled terminal record; each accepted
        // terminal transition persisted the terminal history synchronously
        // (the repository whole-replace write outside render/ranking).
        let succeeded_id = start_download(&mut client, &known_url, "known.bin");
        let _ = wait_row_terminal(&mut client, succeeded_id, DownloadMachineState::Succeeded);
        let cancelled_id = start_download(&mut client, &slow_url, "slow.bin");
        let _ = wait_running_progress(&mut client, cancelled_id);
        client
            .send_ok(&Request::DownloadCancel { id: cancelled_id })
            .expect("cancel request");
        let _ = wait_row_terminal(&mut client, cancelled_id, DownloadMachineState::Cancelled);

        // The terminal rows are visible before the process dies, and the
        // badge already counts zero (both attempts are terminal).
        let before = client.snapshot().expect("before shutdown snapshot");
        assert_eq!(before.download_rows.len(), 2);
        assert_eq!(
            before.downloads.footer_count, 0,
            "terminal history never inflates the badge"
        );
        let opened = open_downloads(&mut client);
        assert_eq!(
            row_by_id(&opened, cancelled_id).action,
            DownloadRowAction::Restart
        );
        assert!(row_by_id(&opened, cancelled_id).delete_available);
        assert_eq!(
            row_by_id(&opened, succeeded_id).action,
            DownloadRowAction::Reveal
        );
        assert!(row_by_id(&opened, succeeded_id).resolved_path.is_some());
        assert_panel_identity(
            &opened,
            view,
            sessions,
            tabs,
            spaces,
            profile_before.borrow().unwrap(),
            space,
        );
    }));
    // Keep the profile directory for the restart: a successful finish would
    // let the harness Drop remove the root, destroying the store we must
    // reload. The failure path keeps the diagnostics as usual.
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the terminal history restores
    // into the controller before the first frame.
    let (mut harness2, socket2, token2) =
        spawn_with_profile("downloads-persist-restore", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // The store loaded truthfully — nothing was discarded on restart —
        // and the profile identity is the persisted one.
        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(
            store.store.store_load,
            StoreLoadSnapshot::Loaded,
            "the same v2 store loads on restart"
        );
        assert_eq!(
            store.active_profile,
            profile_before.borrow().unwrap(),
            "the profile identity survives the restart"
        );

        // The restored terminal history renders from the first observable
        // frame: History most-recent-terminal-first (the cancelled slow.bin
        // is newer than the succeeded known.bin), with the correct state,
        // action, linkage, and resolved path.
        let restored = client.snapshot().expect("restored rows snapshot");
        assert_eq!(restored.download_rows.len(), 2);
        assert_eq!(
            restored.download_rows[0].filename, "slow.bin",
            "most-recent-terminal first"
        );
        assert_eq!(
            restored.download_rows[0].machine_state,
            DownloadMachineState::Cancelled
        );
        assert_eq!(
            restored.download_rows[0].action,
            DownloadRowAction::Restart,
            "a restored cancelled row stays restartable"
        );
        assert!(restored.download_rows[0].action_enabled);
        assert!(
            restored.download_rows[0].delete_available,
            "a restored cancelled row keeps the Delete control"
        );
        assert_eq!(restored.download_rows[0].attempt, 1);
        assert_eq!(restored.download_rows[1].filename, "known.bin");
        assert_eq!(
            restored.download_rows[1].machine_state,
            DownloadMachineState::Succeeded
        );
        assert_eq!(
            restored.download_rows[1].action,
            DownloadRowAction::Reveal,
            "Succeeded stays Reveal-in-Finder only"
        );
        assert!(
            restored.download_rows[1].resolved_path.is_some(),
            "the resolved destination survived the restart"
        );
        assert!(
            restored
                .download_rows
                .iter()
                .all(|row| row.group == DownloadGroup::History),
            "restored rows are terminal History rows"
        );
        assert_eq!(
            restored.downloads.footer_count, 0,
            "restored terminal history never inflates the badge"
        );

        // The panel renders the restored rows with the §11.12 ids and keeps
        // the identity of the fresh process stable while operated.
        let opened = open_downloads(&mut client);
        assert!(opened.downloads.open);
        assert_ids(&opened, &["bbr-downloads-panel", "bbr-downloads-history"]);
        assert_panel_identity(
            &opened,
            opened.active_web_view.expect("restart view"),
            opened.engine_sessions,
            opened.tabs.len(),
            opened.spaces.len(),
            opened.active_profile,
            opened.active_space,
        );

        client.quit().expect("quit");
        let status = wait_for_exit(
            &mut harness2.child,
            Instant::now() + Duration::from_secs(10),
        );
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness2, &socket2, &token2);
}

/// Clear is durable across a real process restart: the explicit §17.7 panel
/// action empties the controller history and the repository in one
/// transaction, and a second launch on the same profile renders the empty
/// panel — cleared history can never return from the store.
#[test]
#[ignore]
fn clear_is_durable_across_process_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("downloads-persist-clear", pid);
    let profile = root.join("profile");
    let known_url = fixtures.url("/__fx/dl/known.bin");

    let (mut harness, socket, token) =
        spawn_with_profile("downloads-persist-clear", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Two terminal history rows (each accepted terminal transition
        // persisted the terminal history synchronously).
        for index in 0..2 {
            let destination = format!("clear-{index}.bin");
            let id = start_download(&mut client, &known_url, &destination);
            let _ = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
        }

        // The explicit Clear control renders while any history exists
        // (§17.7) and clears the controller history durably.
        let opened = open_downloads(&mut client);
        assert_eq!(opened.downloads.total_rows, 2);
        assert_clear_control(&opened, true);
        client
            .send_ok(&Request::ClickDownloadsClear)
            .expect("click downloads clear");
        flush_frame(&mut client);
        let cleared = client.snapshot().expect("cleared snapshot");
        assert!(cleared.downloads.open, "the panel stays open after Clear");
        assert_eq!(cleared.downloads.total_rows, 0, "the panel is empty");
        assert!(cleared.download_rows.is_empty(), "no rows after Clear");
        assert_clear_control(&cleared, false);
        assert_ids(&cleared, &["bbr-downloads-empty"]);
        assert_eq!(cleared.downloads.footer_count, 0, "no attempts → no badge");
    }));
    // Keep the profile directory for the restart (same pattern as above).
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the cleared history is gone —
    // the empty panel renders and no row can return from the store.
    let (mut harness2, socket2, token2) =
        spawn_with_profile("downloads-persist-clear", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(
            store.store.store_load,
            StoreLoadSnapshot::Loaded,
            "the cleared store is a valid v2 store, never discarded"
        );
        let opened = open_downloads(&mut client);
        assert_eq!(
            opened.downloads.total_rows, 0,
            "cleared history stays cleared after the restart"
        );
        assert!(opened.download_rows.is_empty(), "no restored rows");
        assert_clear_control(&opened, false);
        assert_ids(&opened, &["bbr-downloads-panel", "bbr-downloads-empty"]);
        assert_panel_identity(
            &opened,
            opened.active_web_view.expect("restart view"),
            opened.engine_sessions,
            opened.tabs.len(),
            opened.spaces.len(),
            opened.active_profile,
            opened.active_space,
        );

        client.quit().expect("quit");
        let status = wait_for_exit(
            &mut harness2.child,
            Instant::now() + Duration::from_secs(10),
        );
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness2, &socket2, &token2);
}

/// A row Delete is durable across a real process restart: the §17.8
/// cancelled-row Delete removes exactly that one terminal record, and a
/// second launch on the same profile renders the remaining history without
/// the deleted row.
#[test]
#[ignore]
fn delete_is_durable_across_process_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("downloads-persist-delete", pid);
    let profile = root.join("profile");
    let slow_url = fixtures.url("/__fx/dl/slow.bin");
    let known_url = fixtures.url("/__fx/dl/known.bin");
    let deleted_id = std::cell::RefCell::new(0_u64);

    let (mut harness, socket, token) =
        spawn_with_profile("downloads-persist-delete", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Three terminal rows: cancelled (1), succeeded (2), cancelled (3).
        let first_cancelled = cancel_slow(&mut client, &slow_url);
        let succeeded_id = start_download(&mut client, &known_url, "known.bin");
        let _ = wait_row_terminal(&mut client, succeeded_id, DownloadMachineState::Succeeded);
        let second_cancelled = cancel_slow(&mut client, &slow_url);

        // The most-recent cancelled row offers the §17.8 Delete control;
        // Delete removes exactly that one row.
        let opened = open_downloads(&mut client);
        assert_eq!(opened.downloads.total_rows, 3);
        let target = opened
            .download_rows
            .iter()
            .find(|row| row.id == second_cancelled)
            .unwrap_or_else(|| {
                panic!("no row for {second_cancelled}: {:#?}", opened.download_rows)
            });
        assert!(
            target.delete_available,
            "the cancelled row offers the Delete control"
        );
        assert_eq!(target.action, DownloadRowAction::Restart);
        client
            .send_ok(&Request::ClickDownloadRowDelete { row: target.row })
            .expect("click row delete");
        flush_frame(&mut client);
        let after_delete = client.snapshot().expect("after delete snapshot");
        assert_eq!(
            after_delete.downloads.total_rows, 2,
            "exactly one row removed"
        );
        assert!(
            after_delete
                .download_rows
                .iter()
                .all(|row| row.id != second_cancelled),
            "the deleted row is gone"
        );
        assert!(
            after_delete
                .download_rows
                .iter()
                .any(|row| row.id == first_cancelled)
                && after_delete
                    .download_rows
                    .iter()
                    .any(|row| row.id == succeeded_id),
            "the other rows remain"
        );
        assert_clear_control(&after_delete, true);
        *deleted_id.borrow_mut() = second_cancelled;
    }));
    // Keep the profile directory for the restart (same pattern as above).
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the deleted row stays gone and
    // the remaining history restores.
    let (mut harness2, socket2, token2) =
        spawn_with_profile("downloads-persist-delete", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(store.store.store_load, StoreLoadSnapshot::Loaded);
        let opened = open_downloads(&mut client);
        let ids: Vec<u64> = opened.download_rows.iter().map(|row| row.id).collect();
        assert_eq!(opened.downloads.total_rows, 2, "two rows restore");
        assert!(
            !ids.contains(&deleted_id.borrow().clone()),
            "the deleted row never returns from the store"
        );
        // The remaining rows are the older cancelled slow.bin and the
        // succeeded known.bin, most-recent-terminal first.
        assert_eq!(opened.download_rows.len(), 2);
        assert_eq!(opened.download_rows[0].filename, "known.bin");
        assert_eq!(
            opened.download_rows[0].machine_state,
            DownloadMachineState::Succeeded
        );
        assert_eq!(opened.download_rows[1].filename, "slow.bin");
        assert_eq!(
            opened.download_rows[1].machine_state,
            DownloadMachineState::Cancelled
        );
        assert!(opened.download_rows[1].delete_available);
        assert_clear_control(&opened, true);
        assert_eq!(opened.downloads.footer_count, 0);
        assert_panel_identity(
            &opened,
            opened.active_web_view.expect("restart view"),
            opened.engine_sessions,
            opened.tabs.len(),
            opened.spaces.len(),
            opened.active_profile,
            opened.active_space,
        );

        client.quit().expect("quit");
        let status = wait_for_exit(
            &mut harness2.child,
            Instant::now() + Duration::from_secs(10),
        );
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness2, &socket2, &token2);
}

/// Cancel a running `slow.bin` transfer through the production
/// request/confirmation path and return its terminal cancelled id (the
/// deterministic fixture trickles, so the cancel lands mid-transfer).
fn cancel_slow(client: &mut Client, slow_url: &str) -> u64 {
    let id = start_download(client, slow_url, "slow.bin");
    let _ = wait_running_progress(client, id);
    client
        .send_ok(&Request::DownloadCancel { id })
        .expect("cancel request");
    let _ = wait_row_terminal(client, id, DownloadMachineState::Cancelled);
    id
}
