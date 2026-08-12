//! §11.3 swipe **cancel** evidence: below-threshold release and explicit
//! cancel emit no switch, change no controller state, and return the chrome
//! to the exact source presentation. The release phase drives the
//! production handler with the zero delta a real macOS `Ended` carries —
//! the handler must resolve it independent of horizontal dominance.

use super::*;

/// Below-threshold release and explicit cancel: no switch, no controller
/// state change, and the chrome returns to the exact source presentation.
#[test]
#[ignore]
fn below_threshold_and_explicit_cancel_emit_no_switch_and_restore_source() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-swipe-cancel", &page_one);

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

        // A warm second space: Reading owns its own live WebView.
        let reading = create_space_id(&mut client, &mut since, default_profile, "Reading");
        activate_space_id(&mut client, &mut since, reading);
        let _reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        activate_space_id(&mut client, &mut since, default_space);

        let baseline = client.snapshot().expect("baseline snapshot");
        let (cobalt_chrome, _) = light_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        assert_eq!(baseline.active_space, default_space);
        assert_eq!(baseline.active_web_view, Some(first_view));
        assert_eq!(baseline.engine_sessions, 2);
        assert_eq!(selected_space(&baseline), default_space);
        assert_eq!(baseline.swipe.phase, "idle");

        // Below-threshold gesture: 40 px ≈ 0.17 progress — release must
        // cancel with no switch and no state change.
        client
            .send_ok(&dsl::swipe_scroll(
                [40.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe started");
        let tracking = client.snapshot().expect("tracking snapshot");
        assert_eq!(tracking.swipe.phase, "tracking");
        assert_eq!(
            tracking.swipe.active, default_space,
            "active stays truthful"
        );
        assert_eq!(tracking.swipe.source, Some(default_space));
        assert_eq!(tracking.swipe.destination, Some(reading));
        assert_eq!(tracking.swipe.direction.as_deref(), Some("next"));
        assert!((tracking.swipe.progress - 40.0 / 240.0).abs() < 1e-9);
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Moved,
            ))
            .expect("swipe moved");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("swipe ended");
        let below = client.snapshot().expect("below-threshold snapshot");
        assert_eq!(
            below.swipe.phase, "idle",
            "below-threshold release returns Idle"
        );
        assert_eq!(below.swipe.progress, 0.0);
        assert_eq!(
            below.active_space, default_space,
            "no switch below threshold"
        );
        assert_eq!(below.active_web_view, Some(first_view), "no session change");
        assert_eq!(below.engine_sessions, 2);
        assert_eq!(selected_space(&below), default_space);
        assert_eq!(
            below.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_chrome,
            "chrome stays the exact source presentation"
        );

        // Explicit cancel after a full swipe: progress clamps to 1.0, the
        // interpolated chrome equals the destination wash, then SwipeCancel
        // restores the exact source with no switch.
        client
            .send_ok(&dsl::swipe_scroll(
                [400.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("full swipe started");
        let full = client.snapshot().expect("full tracking snapshot");
        assert_eq!(full.swipe.phase, "tracking");
        assert_eq!(full.swipe.progress, 1.0, "progress clamps at 1.0");
        assert!(full.swipe.interpolated_chrome_tint.is_some());
        client
            .send_ok(&dsl::swipe_cancel())
            .expect("explicit cancel");
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_eq!(cancelled.swipe.phase, "idle");
        assert_eq!(cancelled.swipe.progress, 0.0);
        assert_eq!(cancelled.swipe.interpolated_chrome_tint, None);
        assert_eq!(
            cancelled.active_space, default_space,
            "cancel commits nothing"
        );
        assert_eq!(cancelled.active_web_view, Some(first_view));
        assert_eq!(cancelled.engine_sessions, 2);
        assert_eq!(selected_space(&cancelled), default_space);
        assert_eq!(
            cancelled.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_chrome,
            "cancel returns the exact source chrome"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
