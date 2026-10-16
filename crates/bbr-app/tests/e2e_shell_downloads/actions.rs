//! Row-action scenarios (task packet `downloads-sidebar-panel`, DESIGN.md
//! §11.12): Cancel is request/confirmation based (the row stays active until
//! the engine's terminal event lands — no optimistic state change), Retry
//! exists only on `Failed` and starts a fresh attempt linked via `retry_of`,
//! and Reveal-in-Finder runs only for an existing resolved terminal path.

use bbr_testkit::protocol::{DownloadGroup, DownloadMachineState, DownloadRowAction, Request};

use crate::e2e_common::*;
use crate::support::*;

/// Cancel a running `slow.bin` transfer through the production
/// request/confirmation path: the row stays active right after the request,
/// the engine's terminal event lands as `Cancelled` (the first terminal
/// event wins), and the cancelled row renders in History offering Restart
/// and Delete (the settled DESIGN.md §17.8 decision).
#[test]
#[ignore]
fn cancel_is_request_confirmation_and_the_terminal_event_wins() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-cancel", &page_one);
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

        let id = start_download(&mut client, &fixtures, &slow_url, "slow.bin");
        // Bytes flowed before the cancel request (deterministic slow window).
        let running = wait_running_progress(&mut client, id);
        assert_eq!(running.action, DownloadRowAction::Cancel);
        assert!(running.action_enabled);
        assert_eq!(running.group, DownloadGroup::Active);

        // Request cancellation through the production seam.
        client
            .send_ok(&Request::DownloadCancel { id })
            .expect("cancel request");

        // No optimistic state change: the request itself never mutates the
        // record — the row is either still active (the engine's terminal
        // event has not landed yet) or already terminal via the engine's
        // `Cancelled` event. Anything else is an invented transition.
        let after_request = client.snapshot().expect("after cancel request snapshot");
        let row = row_by_id(&after_request, id);
        match row.machine_state {
            DownloadMachineState::Running | DownloadMachineState::Waiting => {
                assert_eq!(row.group, DownloadGroup::Active);
                assert_eq!(
                    row.action,
                    DownloadRowAction::Cancel,
                    "the row stays cancellable while active"
                );
            }
            DownloadMachineState::Cancelled => {
                // The engine's terminal event landed before the snapshot:
                // the row is terminal with no action. Also valid — the first
                // terminal event won.
            }
            other => panic!(
                "cancel request produced an invented state {other:?} — the UI must never mutate the record"
            ),
        }

        // The engine's terminal event lands: `Cancelled`, terminal, History.
        let cancelled = wait_row_terminal(&mut client, id, DownloadMachineState::Cancelled);
        assert_eq!(cancelled.group, DownloadGroup::History);
        assert_eq!(
            cancelled.action,
            DownloadRowAction::Restart,
            "a cancelled row offers Restart (the settled §17.8 decision)"
        );
        assert!(cancelled.action_enabled);
        assert!(
            cancelled.delete_available,
            "a cancelled row also offers the §17.8 Delete control"
        );
        let after_terminal = client.snapshot().expect("after terminal snapshot");
        assert_eq!(
            row_by_id(&after_terminal, id).machine_state,
            DownloadMachineState::Cancelled
        );
        assert_panel_identity(
            &after_terminal,
            view,
            sessions,
            tabs,
            spaces,
            profile,
            space,
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Retry the failed attempt through the production seam: a fresh attempt id
/// in the Active group, `Waiting`, linked via `retry_of` with the attempt
/// count incremented and the Cancel action. The deterministic `fail.bin`
/// fixture fails again, so the fresh attempt runs its own lifecycle to a
/// second `Failed`.
#[test]
#[ignore]
fn retry_starts_a_fresh_linked_attempt_from_failed_only() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-retry", &page_one);
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

        let fail_id = start_download(&mut client, &fixtures, &fail_url, "fail.bin");
        let failed = wait_row_terminal(&mut client, fail_id, DownloadMachineState::Failed);
        assert_eq!(failed.action, DownloadRowAction::Retry);
        assert_eq!(failed.attempt, 1);

        // Retry through the production seam (`DownloadRetry` →
        // `download_retry` → `controller.retry`).
        client
            .send_ok(&Request::DownloadRetry { id: fail_id })
            .expect("retry request");
        let after_retry = client.snapshot().expect("after retry snapshot");
        let fresh = after_retry
            .download_rows
            .iter()
            .find(|row| row.retry_of == Some(fail_id))
            .unwrap_or_else(|| panic!("no fresh linked attempt: {:#?}", after_retry.download_rows));
        assert_eq!(fresh.machine_state, DownloadMachineState::Waiting);
        assert_eq!(fresh.group, DownloadGroup::Active);
        assert_eq!(fresh.action, DownloadRowAction::Cancel);
        assert!(fresh.action_enabled);
        assert_eq!(fresh.attempt, 2, "attempt count increments per retry");
        assert_eq!(fresh.filename, "fail.bin");
        assert_ne!(fresh.id, fail_id, "a fresh attempt id, never reused");

        // The fresh attempt runs its own lifecycle to a second terminal
        // `Failed` (the deterministic 500), linked to the source attempt.
        let fresh_failed = wait_row_terminal(&mut client, fresh.id, DownloadMachineState::Failed);
        assert_eq!(fresh_failed.retry_of, Some(fail_id));
        assert_eq!(fresh_failed.attempt, 2);
        assert_eq!(fresh_failed.group, DownloadGroup::History);

        // A cancelled row is restartable (the settled §17.8 decision) — the
        // same machine path as a failed retry: a fresh attempt linked via
        // `retry_of`, never a resumed id. The controller accepts `Cancelled`
        // in `evaluate_retry` (proven deterministically by the controller
        // unit tests; here the production Restart row action is proven by
        // the restart E2E scenario).
        let closed = client.snapshot().expect("retry identity snapshot");
        assert_panel_identity(&closed, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Restart a cancelled attempt through the production pointer path (the
/// settled §17.8 decision): the cancelled row's Restart action runs the
/// same machine path as a failed retry — a fresh attempt id in the Active
/// group, `Waiting`/`Running`, linked via `retry_of` with the attempt count
/// incremented and the Cancel action. The footer badge (§17.11 settled)
/// counts only in-flight attempts: it returns to zero while the fresh
/// attempt transfers and stays zero once it reaches its terminal state.
#[test]
#[ignore]
fn restart_offers_a_fresh_linked_attempt_from_cancelled() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-restart", &page_one);
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

        // Cancel a running transfer to a terminal Cancelled row (the §17.8
        // restartable state). The footer badge contract (§17.11) is
        // observed through the open panel: the in-flight transfer owns the
        // badge, terminal attempts never do.
        let cancelled_id = start_download(&mut client, &fixtures, &slow_url, "slow.bin");
        let _ = wait_running_progress(&mut client, cancelled_id);
        let running_open = open_downloads(&mut client);
        assert_eq!(
            running_open.downloads.footer_count, 1,
            "the in-flight transfer owns the badge"
        );
        // Paper S-7b: the count rides the `[Downloads]` control as the
        // badge, and the open panel keeps the control pressed.
        assert_eq!(running_open.footer.downloads_count, Some(1));
        assert!(running_open.footer.downloads_open);
        assert_ids(&running_open, &["bbr-downloads-count"]);
        client
            .send_ok(&Request::DownloadCancel { id: cancelled_id })
            .expect("cancel request");
        let cancelled =
            wait_row_terminal(&mut client, cancelled_id, DownloadMachineState::Cancelled);
        assert_eq!(
            cancelled.action,
            DownloadRowAction::Restart,
            "a cancelled row offers Restart (the settled §17.8 decision)"
        );
        assert!(cancelled.action_enabled);
        assert!(
            cancelled.delete_available,
            "a cancelled row also offers the §17.8 Delete control"
        );
        let terminal_badge = client.snapshot().expect("terminal badge snapshot");
        assert_eq!(
            terminal_badge.downloads.footer_count, 0,
            "terminal attempts never inflate the badge"
        );
        assert_eq!(terminal_badge.footer.downloads_count, None);
        assert!(
            !terminal_badge
                .visible_ids
                .iter()
                .any(|id| id == "bbr-downloads-count"),
            "no badge while nothing transfers"
        );

        // Restart through the production pointer path (`ClickDownloadRowAction`
        // on the cancelled row → the exact seam the row's action mouse-down
        // listener calls → `download_retry` → `controller.retry`). The
        // panel is already open; the fresh attempt joins the Active group.
        let opened = terminal_badge;
        let row = row_by_id(&opened, cancelled_id);
        client
            .send_ok(&Request::ClickDownloadRowAction { row: row.row })
            .expect("restart row action");
        let after = client.snapshot().expect("after restart snapshot");
        let fresh = after
            .download_rows
            .iter()
            .find(|candidate| candidate.retry_of == Some(cancelled_id))
            .unwrap_or_else(|| panic!("no fresh linked attempt: {:#?}", after.download_rows));
        assert!(
            matches!(
                fresh.machine_state,
                DownloadMachineState::Waiting | DownloadMachineState::Running
            ),
            "the fresh attempt is active (never resumed under the old id)"
        );
        assert_eq!(fresh.group, DownloadGroup::Active);
        assert_eq!(fresh.action, DownloadRowAction::Cancel);
        assert!(fresh.action_enabled);
        assert_eq!(fresh.attempt, 2, "the attempt count increments per restart");
        assert_eq!(fresh.filename, "slow.bin");
        assert_ne!(fresh.id, cancelled_id, "a fresh attempt id, never reused");
        assert_eq!(
            after.downloads.footer_count, 1,
            "the fresh in-flight attempt owns the badge"
        );

        // The fresh attempt runs its own lifecycle to a terminal Succeeded
        // (the deterministic slow fixture completes), linked to the source
        // attempt; the badge returns to zero.
        let fresh_done = wait_row_terminal(&mut client, fresh.id, DownloadMachineState::Succeeded);
        assert_eq!(fresh_done.retry_of, Some(cancelled_id));
        assert_eq!(fresh_done.attempt, 2);
        assert_eq!(fresh_done.group, DownloadGroup::History);
        let closed = client.snapshot().expect("restart identity snapshot");
        assert_eq!(
            closed.downloads.footer_count, 0,
            "the badge returns to zero when nothing transfers"
        );
        assert_eq!(
            closed.download_rows.len(),
            2,
            "the cancelled source row and its fresh attempt both remain"
        );
        assert_panel_identity(&closed, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Reveal-in-Finder request semantics: a succeeded row's resolved path
/// exists, so the contextual action is Reveal (enabled); activating it
/// through the production pointer path runs the native reveal request and
/// leaves the panel and identity intact.
#[test]
#[ignore]
fn reveal_is_enabled_for_the_existing_resolved_path_and_requests_semantics() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-reveal", &page_one);
    let known_url = fixtures.url("/__fx/dl/known.bin");

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

        let id = start_download(&mut client, &fixtures, &known_url, "known.bin");
        let succeeded = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
        assert_eq!(succeeded.action, DownloadRowAction::Reveal);
        assert!(
            succeeded.resolved_path.is_some(),
            "the succeeded record carries its resolved path"
        );

        // Open the panel: the §11.12 open-time eligibility re-check runs and
        // the existing file keeps the action enabled.
        let opened = open_downloads(&mut client);
        let row = row_by_id(&opened, id);
        assert_eq!(row.action, DownloadRowAction::Reveal);
        assert!(
            row.action_enabled,
            "reveal enabled for the existing resolved path ({:?})",
            row.action_reason
        );
        assert_eq!(row.group, DownloadGroup::History);
        assert!(opened.downloads.open);

        // Activate the row's action through the production pointer path: the
        // native reveal request runs (the app's NSWorkspace reveal seam),
        // the panel stays open, and the row keeps its Reveal action.
        client
            .send_ok(&Request::ClickDownloadRowAction { row: row.row })
            .expect("reveal row action");
        let after = client.snapshot().expect("after reveal snapshot");
        assert!(after.downloads.open, "the panel stays open");
        let row_after = row_by_id(&after, id);
        assert_eq!(row_after.action, DownloadRowAction::Reveal);
        assert_eq!(row_after.machine_state, DownloadMachineState::Succeeded);
        assert_panel_identity(&after, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
