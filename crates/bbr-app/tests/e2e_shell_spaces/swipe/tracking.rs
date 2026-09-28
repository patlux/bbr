//! §11.3 swipe **tracking truthfulness** evidence: the snapshot carries
//! active/source/destination/direction/progress plus the interpolated
//! chrome/chip tints, the semantic selected chip (recursive lookup under
//! `bbr-root`) stays on the source until commit, and commit lands on the
//! exact destination presentation.

use super::*;

/// Tracking is typed and truthful: the snapshot carries
/// active/source/destination/direction/progress plus the interpolated
/// chrome/chip tints, the semantic selected chip stays on the source until
/// commit, and commit lands on the exact destination presentation.
#[test]
#[ignore]
fn tracking_snapshot_and_semantics_stay_truthful_until_commit() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-swipe-track", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Light-calibrated swipe-tint evidence (task packet
        // `system-appearance-theme`): force the light rail regardless of
        // the host appearance so the interpolated chrome/chip tint
        // assertions stay byte-identical to the integrated reference
        // evidence.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush forced light");

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // Reading gets an explicit pine theme so the source (cobalt) and
        // destination (pine) chrome washes differ measurably, plus a warm
        // tab so the commit can be proven session-stable.
        let reading = create_space_id(&mut client, &mut since, default_profile, "Reading");
        client
            .send_ok(&dsl::set_space_theme(reading, "pine"))
            .expect("set pine theme");
        activate_space_id(&mut client, &mut since, reading);
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        activate_space_id(&mut client, &mut since, default_space);

        let baseline = client.snapshot().expect("baseline");
        let (cobalt_chrome, cobalt_chip) = light_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        let (pine_chrome, pine_chip) = light_standard_tints(bbr_domain::SpaceTheme::Pine);
        assert_eq!(
            baseline.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_chrome,
            "cobalt source chrome"
        );
        assert_ne!(
            cobalt_chip, pine_chip,
            "the stronger chip tint distinguishes the two curated tokens"
        );

        // Half-progress tracking: typed snapshot evidence, truthful selected
        // semantics, and a strictly-between interpolated chrome wash.
        client
            .send_ok(&dsl::swipe_scroll(
                [120.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe started");
        let half = client.snapshot().expect("half tracking snapshot");
        assert_eq!(half.swipe.phase, "tracking");
        assert_eq!(half.swipe.active, default_space, "active stays truthful");
        assert_eq!(half.swipe.source, Some(default_space));
        assert_eq!(half.swipe.destination, Some(reading));
        assert_eq!(half.swipe.direction.as_deref(), Some("next"));
        assert!((half.swipe.progress - 0.5).abs() < 1e-9);
        assert_eq!(
            half.swipe.interpolated_chrome_tint,
            Some(half_way_light_chrome(
                bbr_domain::SpaceTheme::Cobalt,
                bbr_domain::SpaceTheme::Pine
            )),
            "half progress paints the exact half-way chrome wash"
        );
        let interpolated_chip = half
            .swipe
            .interpolated_chip_tint
            .as_deref()
            .expect("interpolated chip while tracking");
        assert_ne!(
            interpolated_chip, cobalt_chip,
            "chip tint is strictly away from source"
        );
        assert_ne!(
            interpolated_chip, pine_chip,
            "chip tint is strictly before destination"
        );
        // Truthful appearance: the active appearance never lies during the
        // gesture (no selected-space change before commit).
        assert_eq!(
            half.active_appearance.as_ref().unwrap().theme,
            "cobalt",
            "active appearance stays source"
        );
        assert_eq!(
            half.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_chrome,
            "active chrome evidence stays source"
        );
        assert!(
            chip_selected(&half, default_space),
            "source chip stays selected"
        );
        assert!(
            !chip_selected(&half, reading),
            "destination chip is not selected yet"
        );
        assert!(half.visible_ids.iter().any(|id| id == "bbr-space-strip"));
        assert_eq!(
            half.active_web_view,
            Some(first_view),
            "no view change while tracking"
        );
        assert_eq!(half.engine_sessions, 2);

        // Full progress: the interpolated chrome reaches the exact
        // destination wash.
        client
            .send_ok(&dsl::swipe_scroll(
                [240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Moved,
            ))
            .expect("swipe moved");
        let full = client.snapshot().expect("full tracking snapshot");
        assert_eq!(full.swipe.progress, 1.0);
        assert_eq!(
            full.swipe.interpolated_chrome_tint.as_deref(),
            Some(pine_chrome.as_str()),
            "full progress lands on the exact destination chrome"
        );
        assert_eq!(
            full.swipe.interpolated_chip_tint.as_deref(),
            Some(pine_chip.as_str()),
            "full progress lands on the exact destination chip tint"
        );
        assert!(
            chip_selected(&full, default_space),
            "still truthful before commit"
        );

        // Commit: exactly one switch, destination chip selected, appearance
        // flips to the exact destination, tint evidence clears.
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
        let _seq = seq;
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after commit");
        let committed = client.snapshot().expect("committed snapshot");
        assert_eq!(committed.swipe.phase, "idle");
        assert_eq!(committed.swipe.progress, 0.0);
        assert_eq!(committed.swipe.interpolated_chrome_tint, None);
        assert_eq!(committed.active_space, reading);
        assert_eq!(
            selected_space(&committed),
            reading,
            "selected chip flips only on commit"
        );
        assert!(chip_selected(&committed, reading));
        assert!(!chip_selected(&committed, default_space));
        assert_eq!(
            committed.active_appearance.as_ref().unwrap().theme,
            "pine",
            "exact destination appearance"
        );
        assert_eq!(
            committed.active_appearance.as_ref().unwrap().chrome_tint,
            pine_chrome,
            "exact destination chrome"
        );
        assert_eq!(
            committed.active_web_view,
            Some(reading_view),
            "warm destination view"
        );
        assert_eq!(committed.engine_sessions, 2);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The light-standard chrome wash half-way between two curated tokens,
/// computed from the same resolver the app paints with (per-channel linear
/// interpolation of the unrounded washes, then the snapshot's hex form).
fn half_way_light_chrome(
    source: bbr_domain::SpaceTheme,
    destination: bbr_domain::SpaceTheme,
) -> String {
    let theme = bbr_ui::theme::Theme::new(
        bbr_ui::theme::ThemeMode::Light,
        bbr_ui::theme::ContrastMode::Standard,
        bbr_ui::theme::Density::Normal,
    );
    let (a, b) = (
        bbr_ui::theme::space_tint(source, bbr_ui::theme::TintIntensity::Standard, &theme).chrome,
        bbr_ui::theme::space_tint(destination, bbr_ui::theme::TintIntensity::Standard, &theme)
            .chrome,
    );
    let lerp = |x: f32, y: f32| x + (y - x) * 0.5;
    bbr_ui::theme::hex(gpui::Rgba {
        r: lerp(a.r, b.r),
        g: lerp(a.g, b.g),
        b: lerp(a.b, b.b),
        a: 1.0,
    })
}
