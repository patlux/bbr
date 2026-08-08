//! §11.3 swipe **commit** evidence: above-threshold release commits exactly
//! one switch in both display directions with warm WebView/session identity,
//! unchanged content/pane geometry, and chrome focus preserved. The release
//! phase drives the production handler with the zero delta a real macOS
//! `Ended` carries.

use super::*;

/// Above-threshold release commits exactly one switch in both display
/// directions with warm WebView/session identity, unchanged content/pane
/// geometry, and chrome focus preserved.
#[test]
#[ignore]
fn above_threshold_swipe_commits_exactly_once_in_both_directions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-swipe-commit", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // Three spaces in the default profile: canonical order [2, 4, 7].
        // Each destination owns a real warm WebView.
        let reading = create_space_id(&mut client, &mut since, default_profile, "Reading");
        activate_space_id(&mut client, &mut since, reading);
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let reading_geometry = client.snapshot().expect("reading geometry");
        let reading_content_rect = reading_geometry.content_rect;
        let reading_pane = reading_geometry.pane_layout.expect("single pane");
        let reading_pane_rects = reading_geometry.pane_rects.clone();
        activate_space_id(&mut client, &mut since, default_space);

        let work = create_space_id(&mut client, &mut since, default_profile, "Work");
        activate_space_id(&mut client, &mut since, work);
        let _work_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let work_content_rect = client.snapshot().expect("work geometry").content_rect;
        activate_space_id(&mut client, &mut since, default_space);

        let baseline = client.snapshot().expect("baseline");
        assert_eq!(baseline.engine_sessions, 3, "three warm sessions");
        assert_eq!(baseline.active_web_view, Some(first_view));

        // Swipe next (positive delta) → exactly Reading, never Work.
        client
            .send_ok(&dsl::swipe_scroll(
                [240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe started");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("swipe ended");
        let seq = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        since = seq;
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after swipe commit");
        let committed = client.snapshot().expect("committed snapshot");
        assert_eq!(
            committed.active_space, reading,
            "exactly the adjacent destination"
        );
        assert_eq!(
            committed.active_web_view,
            Some(reading_view),
            "warm view returns"
        );
        assert_eq!(committed.engine_sessions, 3, "no session recreated");
        assert_eq!(
            selected_space(&committed),
            reading,
            "selected chip flips on commit"
        );
        assert_eq!(committed.swipe.phase, "idle");
        assert_eq!(
            committed.content_rect, reading_content_rect,
            "slot geometry unchanged"
        );
        assert_eq!(committed.pane_layout, Some(reading_pane));
        assert_eq!(committed.pane_rects, reading_pane_rects);
        assert!(committed.chrome_focused, "chrome focus stays in the strip");

        // Swipe prev (negative delta) → exactly back to Default.
        client
            .send_ok(&dsl::swipe_scroll(
                [-240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe prev started");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("swipe prev ended");
        let seq = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let _seq = seq;
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after swipe back");
        let back = client.snapshot().expect("swiped back snapshot");
        assert_eq!(back.active_space, default_space);
        assert_eq!(
            back.active_web_view,
            Some(first_view),
            "warm first view restores"
        );
        assert_eq!(back.engine_sessions, 3);
        assert_eq!(selected_space(&back), default_space);
        assert_eq!(
            back.content_rect, work_content_rect,
            "geometry stable across the round trip"
        );
        assert!(back.chrome_focused);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
