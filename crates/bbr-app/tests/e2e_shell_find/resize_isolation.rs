//! Resize/tab-discard + page-world isolation scenarios (task packet
//! `find-in-page-portal-bar`).

use crate::e2e_common::*;
use crate::support::*;

/// Resizing the parent while the find bar is open re-pins the native portal
/// frame to the new content rect without recreating the portal window or
/// any web session (the find presentation stays open with its report);
/// switching tabs ends the one-per-window presentation, leaving both
/// sessions and the profile/space untouched.
#[test]
#[ignore]
fn find_resize_follow_and_tab_switch_discard() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, repeated, zero, _mutate, _isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness("find-resize", &repeated);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        let baseline_w = settled.window_size[0];
        let baseline_h = settled.window_size[1];
        let _ = wait_fixture_loaded(&mut client, first_view, FIND_REPEATED_TITLE, since);

        // Part A — resize while open: the portal frame follows, the portal
        // window and the session are never recreated, the bar keeps its
        // report (the card rect stays at its present-time resolution, the
        // same documented bounded behavior as the merged portal suite; the
        // native frame sync is the follow evidence).
        client.send_ok(&dsl::find_open()).expect("find open");
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        let open = wait_find_report(&mut client, 3, Some(1), deadline());
        let portal_window_number = open.portal.portal_window_number;
        assert!(portal_window_number > 0);

        let new_w = baseline_w - 220.0;
        let new_h = baseline_h - 140.0;
        assert!(
            new_w > FIND_BAR_WIDTH + FIND_GAP * 2.0,
            "resized window still fits the find bar"
        );
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: new_w,
                height: new_h,
            })
            .expect("resize parent");
        flush_frame(&mut client);
        let resized = loop {
            let snapshot = client.snapshot().expect("resize snapshot");
            if (snapshot.window_size[0] - new_w).abs() <= 0.5 && snapshot.portal.frame_synced {
                break snapshot;
            }
            assert!(
                Instant::now() < deadline(),
                "parent resize did not land; window {:?}, frame_synced {}",
                snapshot.window_size,
                snapshot.portal.frame_synced
            );
            std::thread::yield_now();
        };
        assert!(
            (resized.window_size[1] - baseline_h).abs() > 1.0,
            "the resize must change the window height"
        );
        assert_eq!(
            resized.portal.portal_window_number, portal_window_number,
            "resize must reuse the same portal window, never recreate it"
        );
        assert_eq!(
            resized.find.match_count,
            Some(3),
            "the find report survives the resize"
        );
        assert_eq!(
            resized.find.current_match,
            Some(1),
            "the ordinal survives the resize"
        );
        assert_eq!(
            resized.portal.rect, open.portal.rect,
            "the bar keeps its present-time rect (bounded behavior); the native frame followed"
        );
        assert_ownership_preserved(&resized, &settled);

        // Part B — a tab switch ends the one-per-window presentation without
        // touching either session or the profile/space.
        let second_view = open_tab(&mut client, &mut since, &zero, Some(FIND_ZERO_TITLE));
        assert_ne!(second_view, first_view);
        let switched = wait_for_find_closed(&mut client, deadline());
        assert_eq!(switched.active_web_view, Some(second_view));
        assert_eq!(switched.engine_sessions, 2, "the new tab's session exists");
        assert_eq!(switched.active_profile, settled.active_profile);
        assert_eq!(switched.active_space, settled.active_space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The strategy executes in `WKContentWorld::defaultClientWorld`: the
/// fixture page records every unexpected `__bbr*` global it can see (and
/// the `bbr-find` custom highlight) and reports a `LEAK:` title on any hit.
/// After a full find run (start, steps, close) the page world observes
/// nothing: the title stays the clean marker, proving the find machinery is
/// unobservable by and cannot collide with page scripts while the shared
/// DOM still carries the selection moves.
#[test]
#[ignore]
fn find_page_world_cannot_observe_find_machinery() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, _repeated, _zero, _mutate, isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness("find-isolation", &isolation);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, FIND_ISOLATION_TITLE, since);

        client.send_ok(&dsl::find_open()).expect("find open");
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        wait_find_report(&mut client, 3, Some(1), deadline());
        // More strategy runs: each moves the shared-DOM selection, which the
        // page-world probe observes via `selectionchange` and rescans.
        key(&mut client, "enter");
        wait_find_report(&mut client, 3, Some(2), deadline());
        key(&mut client, "shift-enter");
        wait_find_report(&mut client, 3, Some(1), deadline());

        // The probe rescans on a fast interval, so a bounded sequence of
        // deadline-polled snapshots must all keep the clean title; any leak
        // would have surfaced within one interval of a strategy run.
        let isol_deadline = deadline();
        for _ in 0..8 {
            let snapshot = client.snapshot().expect("isolation snapshot");
            assert_eq!(
                snapshot.title.as_deref(),
                Some(FIND_ISOLATION_TITLE),
                "the page world observed find machinery: {snapshot:?}"
            );
            assert!(
                Instant::now() < isol_deadline,
                "isolation title check timed out"
            );
            std::thread::yield_now();
        }

        // The clear op on close runs in the client world too; the page world
        // still observes nothing afterwards.
        client.send_ok(&dsl::find_close()).expect("find close");
        let closed = wait_for_find_closed(&mut client, deadline());
        assert_eq!(
            closed.title.as_deref(),
            Some(FIND_ISOLATION_TITLE),
            "the page world observed find machinery after close: {closed:?}"
        );
        assert_ownership_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
