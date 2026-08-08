//! §11.3 swipe **no-op** evidence: boundary gestures (no adjacent space in
//! either display direction) and vertical-dominant deltas are no-ops — the
//! machine never begins and the active space never changes.

use super::*;

/// Boundary gestures (no adjacent space) and vertical-dominant deltas are
/// no-ops: the machine never begins and the active space never changes.
#[test]
#[ignore]
fn boundary_and_vertical_deltas_are_noops() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-swipe-boundary", &page_one);

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

        // Single-space collection: no adjacent space in either direction.
        client
            .send_ok(&dsl::swipe_scroll(
                [-240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("boundary prev");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("boundary prev end");
        client
            .send_ok(&dsl::swipe_scroll(
                [240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("boundary next");
        let boundary = client.snapshot().expect("boundary snapshot");
        assert_eq!(
            boundary.swipe.phase, "idle",
            "boundary gesture never begins"
        );
        assert_eq!(boundary.active_space, default_space);
        assert_eq!(boundary.active_web_view, Some(first_view));

        // Vertical-dominant delta is not a gesture: it never begins and is
        // never stopped (ordinary scrolling stays untouched).
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 240.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("vertical started");
        let vertical = client.snapshot().expect("vertical snapshot");
        assert_eq!(vertical.swipe.phase, "idle", "vertical delta never tracks");
        assert_eq!(vertical.active_space, default_space);
        assert_eq!(vertical.active_web_view, Some(first_view));

        // Last-space boundary: from the last space in display order, `next`
        // has no adjacent destination.
        let reading = create_space_id(&mut client, &mut since, default_profile, "Reading");
        activate_space_id(&mut client, &mut since, reading);
        client
            .send_ok(&dsl::swipe_scroll(
                [240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("last-space next");
        let last = client.snapshot().expect("last-space boundary snapshot");
        assert_eq!(last.swipe.phase, "idle", "last-space next never begins");
        assert_eq!(last.active_space, reading);
        assert_eq!(selected_space(&last), reading);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
