//! Real-app system-appearance dark shell evidence (task packet
//! `system-appearance-theme`): the permanent §11.3 space strip presents the
//! complete dark token set — the resolved chrome/chip tints are the dark
//! presentations of the active appearance — and a live flip to light
//! preserves the chip selection and the space/session identity while only
//! the token set swaps.

use super::*;

/// The resolved dark/standard chrome tint of a space token through the one
/// public presentation authority (the dark counterpart of the suite's
/// `light_standard_tints`; the E2E asserts the exact hex the dark render
/// paints — never copied palette literals).
fn dark_standard_tints(token: bbr_domain::SpaceTheme) -> (String, String) {
    let theme = bbr_ui::theme::Theme::new(
        bbr_ui::theme::ThemeMode::Dark,
        bbr_ui::theme::ContrastMode::Standard,
        bbr_ui::theme::Density::Normal,
    );
    let tint = bbr_ui::theme::space_tint(token, bbr_ui::theme::TintIntensity::Standard, &theme);
    (
        bbr_ui::theme::hex(tint.chrome),
        bbr_ui::theme::hex(tint.chip_active),
    )
}

/// System-appearance dark shell (task packet `system-appearance-theme`):
/// the permanent §11.3 space strip presents the complete dark token set —
/// the resolved chrome/chip tints are the dark presentations of the active
/// appearance — and a live flip to light preserves the chip selection and
/// the space/session identity while only the token set swaps.
#[test]
#[ignore]
fn dark_space_strip_presents_dark_tints_and_flip_preserves_chip_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-dark", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Force dark before the first asserted frame.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Dark,
            )))
            .expect("force dark");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush forced dark");
        let launch = client.snapshot().expect("dark launch snapshot");
        assert_eq!(launch.shell_modes.theme, "dark");
        let default_space = launch.active_space;
        let default_profile = launch.active_profile;

        // The strip and the resolved chrome/chip tints are the dark
        // presentations of the derived cobalt appearance.
        let (cobalt_dark_chrome, cobalt_dark_chip) =
            dark_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        let appearance = launch
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.space, default_space);
        assert_eq!(appearance.theme, "cobalt");
        assert_eq!(
            appearance.chrome_tint, cobalt_dark_chrome,
            "dark chrome wash"
        );
        assert_eq!(appearance.chip_tint, cobalt_dark_chip, "dark chip surface");
        assert_eq!(launch.space_chips.len(), 1);
        assert_eq!(launch.space_chips[0].theme, "cobalt");
        assert!(launch.space_chips[0].selected);
        assert_chip_evidence(&launch, default_space);

        // A second space with an explicit pine theme; activating it moves
        // the chip selection under the dark token set.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::set_space_theme(reading, "pine"))
            .expect("set pine theme");
        client
            .send_ok(&dsl::click_space(reading))
            .expect("activate reading");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush dark reading activation");
        let dark_reading = client.snapshot().expect("dark reading snapshot");
        let (pine_dark_chrome, pine_dark_chip) = dark_standard_tints(bbr_domain::SpaceTheme::Pine);
        let appearance = dark_reading
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.theme, "pine");
        assert_eq!(appearance.chrome_tint, pine_dark_chrome);
        assert_eq!(appearance.chip_tint, pine_dark_chip);
        assert_eq!(
            dark_reading
                .space_chips
                .iter()
                .find(|c| c.selected)
                .expect("selected chip")
                .space,
            reading,
            "the dark strip selects the active space"
        );
        assert_eq!(dark_reading.engine_sessions, 1, "no session churn");

        // Give Reading a real tab so the flip proves warm-session identity
        // preservation (its live WebView stays the active view).
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush reading tab");
        let reading_rect = client.snapshot().expect("reading tab rect");
        assert_eq!(reading_rect.active_web_view, Some(reading_view));
        assert_eq!(reading_rect.engine_sessions, 2);

        // Live flip to light: the chip selection and the space/session
        // identity survive; only the token set swaps.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("flip to light");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush light flip");
        let light_reading = client.snapshot().expect("light reading snapshot");
        assert_eq!(light_reading.shell_modes.theme, "light");
        let (pine_light_chrome, pine_light_chip) =
            light_standard_tints(bbr_domain::SpaceTheme::Pine);
        let appearance = light_reading
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.theme, "pine");
        assert_eq!(appearance.chrome_tint, pine_light_chrome);
        assert_eq!(appearance.chip_tint, pine_light_chip);
        assert_eq!(
            light_reading
                .space_chips
                .iter()
                .find(|c| c.selected)
                .expect("selected chip")
                .space,
            reading,
            "chip selection survives the flip"
        );
        assert_eq!(light_reading.active_web_view, Some(reading_view));
        assert_eq!(
            light_reading.engine_sessions, 2,
            "sessions survive the flip"
        );
        assert_eq!(light_reading.content_rect, reading_rect.content_rect);
        assert_eq!(light_reading.active_space, reading);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
