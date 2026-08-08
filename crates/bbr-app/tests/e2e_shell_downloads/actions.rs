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
/// event wins), and the cancelled row renders quiet in History — no primary
/// action in v1 (DESIGN.md §17.8 open).
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

        let id = start_download(&mut client, &slow_url, "slow.bin");
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
            DownloadRowAction::None,
            "cancelled rows render no primary action in v1 (§17.8)"
        );
        assert!(!cancelled.action_enabled);
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

        let fail_id = start_download(&mut client, &fail_url, "fail.bin");
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

        // Cancelled rows are never retried by the UI: the controller rejects
        // a retry of a `Cancelled` attempt (proven by the controller unit
        // tests deterministically; here the seam rejects non-Failed targets).
        let closed = client.snapshot().expect("retry identity snapshot");
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

        let id = start_download(&mut client, &known_url, "known.bin");
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
