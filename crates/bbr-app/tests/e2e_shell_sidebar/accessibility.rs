//! Real-app system accessibility preferences E2E (task packet
//! `v2-system-prefs`): the production shell composes its §3.4 contrast
//! mode and its §6 motion mode from the macOS Increase Contrast and Reduce
//! Motion settings. The testkit accessibility override forces each option
//! regardless of the host, so the E2E proves:
//!
//! 1. Increase Contrast flips the §15 `contrast` field and the render paints
//!    the increased-contrast token set (the resolved chrome tint is the IC
//!    presentation of the active space).
//! 2. Reduce Motion flips the §15 `reduced_motion` field independently.
//! 3. Neither flip moves identity, session or geometry (commit-first).

use super::*;

/// The resolved light chrome tint of a space token for one contrast mode
/// through the one public presentation authority (never copied literals).
fn light_chrome_tint(
    token: bbr_domain::SpaceTheme,
    contrast: bbr_ui::theme::ContrastMode,
) -> String {
    let theme = bbr_ui::theme::Theme::new(
        bbr_ui::theme::ThemeMode::Light,
        contrast,
        bbr_ui::theme::Density::Normal,
    );
    bbr_ui::theme::hex(bbr_ui::theme::space_tint(token, &theme).chrome)
}

#[test]
#[ignore]
fn increase_contrast_and_reduce_motion_follow_the_system_options() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("a11y-display-options", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");

        // Deterministic start regardless of the host: light, both options
        // forced off.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light");
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("force both options off");
        flush_frame(&mut client);
        let base = client.snapshot().expect("baseline snapshot");
        assert_eq!(base.shell_modes.contrast, "standard");
        assert!(!base.shell_modes.reduced_motion);
        let standard_tint = light_chrome_tint(
            bbr_domain::SpaceTheme::Cobalt,
            bbr_ui::theme::ContrastMode::Standard,
        );
        assert_eq!(
            base.active_appearance
                .as_ref()
                .expect("appearance")
                .chrome_tint,
            standard_tint,
            "standard-contrast rail painted"
        );
        let view = base.active_web_view.expect("native view");
        let sessions = base.engine_sessions;
        let rect = base.content_rect;
        let width = base.sidebar_width;

        // Increase Contrast: the contrast field flips and the render paints
        // the increased-contrast token set; motion is untouched.
        client
            .send_ok(&dsl::set_accessibility_override(Some(true), Some(false)))
            .expect("increase contrast");
        flush_frame(&mut client);
        let contrast = client.snapshot().expect("increased-contrast snapshot");
        assert_eq!(contrast.shell_modes.contrast, "increased");
        assert!(!contrast.shell_modes.reduced_motion);
        let increased_tint = light_chrome_tint(
            bbr_domain::SpaceTheme::Cobalt,
            bbr_ui::theme::ContrastMode::Increased,
        );
        assert_ne!(increased_tint, standard_tint, "IC tint differs");
        assert_eq!(
            contrast
                .active_appearance
                .as_ref()
                .expect("appearance")
                .chrome_tint,
            increased_tint,
            "increased-contrast rail painted"
        );

        // Reduce Motion: the motion field flips independently.
        client
            .send_ok(&dsl::set_accessibility_override(Some(true), Some(true)))
            .expect("reduce motion");
        flush_frame(&mut client);
        let reduced = client.snapshot().expect("reduced-motion snapshot");
        assert!(
            reduced.shell_modes.reduced_motion,
            "reduced motion reported"
        );
        assert_eq!(reduced.shell_modes.contrast, "increased");

        // Contrast back off while motion stays reduced: each option is
        // independent.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("standard contrast, reduced motion");
        flush_frame(&mut client);
        let mixed = client.snapshot().expect("mixed snapshot");
        assert_eq!(mixed.shell_modes.contrast, "standard");
        assert!(mixed.shell_modes.reduced_motion);
        assert_eq!(
            mixed
                .active_appearance
                .as_ref()
                .expect("appearance")
                .chrome_tint,
            standard_tint,
            "standard rail painted again"
        );

        // No flip moved identity, sessions or geometry.
        for snap in [&contrast, &reduced, &mixed] {
            assert_eq!(snap.active_web_view, Some(view), "session identity");
            assert_eq!(snap.engine_sessions, sessions, "no session churn");
            assert_eq!(snap.content_rect, rect, "native slot unchanged");
            assert_eq!(snap.sidebar_width, width, "rail width unchanged");
            assert_eq!(snap.active_tab_index, Some(0), "selection preserved");
        }

        // Clearing both overrides returns to the observed system options.
        client
            .send_ok(&dsl::set_accessibility_override(None, None))
            .expect("clear overrides");
        flush_frame(&mut client);
        let system = client.snapshot().expect("system snapshot");
        assert!(
            system.shell_modes.contrast == "standard" || system.shell_modes.contrast == "increased"
        );
        assert_eq!(system.active_web_view, Some(view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
