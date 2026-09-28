//! Real-app system-appearance theme E2E (task packet
//! `system-appearance-theme`): the production shell derives its `ThemeMode`
//! from the macOS system appearance through the testkit appearance
//! override, so E2E can force light and dark regardless of the host.
//!
//! Scenarios:
//!
//! 1. **Forced-dark launch:** the §15 snapshot `theme` field reports
//!    `dark`; the sidebar, selected row, hover wash, footer and address
//!    surfaces keep their exact §10/§11 states (only the token set swaps),
//!    and the resolved chrome tint is the dark presentation of the active
//!    space's appearance.
//! 2. **Live flip mid-session:** an appearance override change re-presents
//!    the complete other token set on the next frame (`theme` flips) while
//!    the selected tab, its WebViewId/session, chrome focus, sidebar
//!    geometry and the native content rect stay untouched — commit-first,
//!    no layout movement with the theme.

use super::*;

/// The resolved dark/standard chrome tint of a space token through the one
/// public presentation authority (mirrors `light_standard_tints` in the
/// spaces suite; the E2E asserts the exact hex the dark render paints —
/// never copied palette literals).
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

/// The resolved light/standard chrome tint of a space token (the
/// VS-REFERENCE light evidence stays byte-identical).
fn light_standard_tints(token: bbr_domain::SpaceTheme) -> (String, String) {
    let theme = bbr_ui::theme::Theme::new(
        bbr_ui::theme::ThemeMode::Light,
        bbr_ui::theme::ContrastMode::Standard,
        bbr_ui::theme::Density::Normal,
    );
    let tint = bbr_ui::theme::space_tint(token, bbr_ui::theme::TintIntensity::Standard, &theme);
    (
        bbr_ui::theme::hex(tint.chrome),
        bbr_ui::theme::hex(tint.chip_active),
    )
}

/// Forced-dark launch: the snapshot reports the effective dark mode, the
/// shell surfaces keep their exact §10/§11 states (selected row, hover
/// wash, footer, address pill, divider) and the resolved chrome tint is
/// the dark presentation of the active space's appearance.
#[test]
#[ignore]
fn forced_dark_launch_presents_the_dark_shell_with_unchanged_states() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("theme-dark-launch", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Force dark before the first asserted frame: the effective mode
        // must flip regardless of the host appearance.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Dark,
            )))
            .expect("force dark");
        flush_frame(&mut client);
        let initial = client.snapshot().expect("dark launch snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        // The fixture title settles the first tab out of `loading` (the
        // composed state keeps `loading` until the real title lands), so
        // the §10 states below are asserted on a settled row.
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // §15 mode field: the snapshot reports the effective (dark) mode.
        assert_eq!(
            initial.shell_modes.theme, "dark",
            "the effective mode is dark under the override"
        );
        // The resolved chrome tint is the dark presentation of the active
        // space's appearance (cobalt, the derived default): the render
        // actually painted the dark token set — no pixel reads.
        let appearance = initial
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        let (cobalt_dark_chrome, cobalt_dark_chip) =
            dark_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        assert_eq!(
            appearance.chrome_tint, cobalt_dark_chrome,
            "dark chrome wash"
        );
        assert_eq!(appearance.chip_tint, cobalt_dark_chip, "dark chip surface");

        // Sidebar surfaces and their §10/§11 states are unchanged by the
        // mode: the same ids, the same selected row, the same footer and
        // address surfaces.
        assert!(initial.sidebar_visible);
        assert_eq!(initial.sidebar_width, SIDEBAR_DEFAULT);
        assert_ids(
            &initial,
            &[
                "bbr-sidebar",
                "bbr-titlebar",
                "bbr-address",
                "bbr-address-domain",
                "bbr-active-space-title",
                "bbr-new-tab",
                "bbr-tab-list",
                "bbr-tab-row-0",
                "bbr-downloads",
                "bbr-space-strip",
                "bbr-plus",
                "bbr-sidebar-divider",
                "bbr-native-web-slot-1",
            ],
        );
        assert_eq!(initial.active_tab_index, Some(0), "launch selects tab 0");
        assert_eq!(initial.tab_states[0], TabVisualState::Selected);
        assert_eq!(initial.space_chips.len(), 1);
        assert!(initial.space_chips[0].selected, "active chip selected");

        // Hover wash: hovering the (unselected) first row after opening a
        // second tab drives the §10.2 state in dark — the state vocabulary
        // is mode-independent.
        let _blank = open_tab(&mut client, &mut since, "about:blank", None);
        client.send_ok(&dsl::hover_tab_row(0)).expect("hover row 0");
        flush_frame(&mut client);
        let hovered = client.snapshot().expect("dark hover snapshot");
        assert_eq!(
            hovered.tab_states[0],
            TabVisualState::Hover,
            "§10.2 hover wash state in dark"
        );
        assert_eq!(hovered.active_tab_index, Some(1), "hover never selects");
        let hovered_view = hovered.active_web_view;
        let hovered_sessions = hovered.engine_sessions;

        // Layout never moves with the theme: the native content rect is
        // identical to the light rail's geometry (rail 300, content fill).
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light");
        flush_frame(&mut client);
        let light = client.snapshot().expect("light snapshot");
        assert_eq!(light.shell_modes.theme, "light");
        assert_eq!(light.sidebar_width, SIDEBAR_DEFAULT);
        assert_eq!(
            light.active_web_view, hovered_view,
            "the flip never moves the active view"
        );
        assert_eq!(
            light.engine_sessions, hovered_sessions,
            "no session churn with the theme"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Live flip mid-session: the appearance override re-presents the complete
/// other token set (`theme` flips, the resolved chrome tint swaps) while
/// the selected tab, its WebViewId/session, chrome focus, sidebar geometry
/// and the native content rect stay untouched — commit-first, no
/// intermediate layout.
#[test]
#[ignore]
fn live_flip_mid_session_presents_the_other_token_set_and_preserves_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("theme-flip", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");

        // Deterministic start: force light, then capture the identity +
        // geometry baseline.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light start");
        flush_frame(&mut client);
        let light = client.snapshot().expect("light baseline");
        let first_view = light.active_web_view.expect("first native view");
        let sessions = light.engine_sessions;
        assert_eq!(light.shell_modes.theme, "light");
        let (cobalt_light_chrome, _) = light_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        assert_eq!(
            light.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_light_chrome,
            "light chrome wash baseline"
        );
        let light_rect = light.content_rect;
        let light_width = light.sidebar_width;
        let active_space = light.active_space;
        let active_chip_theme = light.space_chips[0].theme.clone();
        let launch_chrome_focus = light.chrome_focused;

        // Flip to dark: the whole chrome re-presents with the other token
        // set on the next frame; identity and geometry never move.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Dark,
            )))
            .expect("flip to dark");
        flush_frame(&mut client);
        let dark = client.snapshot().expect("dark flip snapshot");
        assert_eq!(dark.shell_modes.theme, "dark", "theme field flips");
        let (cobalt_dark_chrome, cobalt_dark_chip) =
            dark_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        assert_eq!(
            dark.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_dark_chrome,
            "dark chrome wash presented"
        );
        assert_eq!(
            dark.active_appearance.as_ref().unwrap().chip_tint,
            cobalt_dark_chip,
            "dark chip surface presented"
        );
        assert_eq!(dark.active_web_view, Some(first_view), "session identity");
        assert_eq!(dark.engine_sessions, sessions, "no session recreated");
        assert_eq!(dark.active_tab_index, Some(0), "selection preserved");
        assert_eq!(dark.active_space, active_space, "space identity preserved");
        assert_eq!(
            dark.space_chips[0].theme, active_chip_theme,
            "chip appearance preserved"
        );
        assert!(dark.space_chips[0].selected, "chip selection preserved");
        assert_eq!(
            dark.chrome_focused, launch_chrome_focus,
            "chrome focus preserved"
        );
        assert_eq!(dark.sidebar_width, light_width, "rail width unchanged");
        assert_eq!(dark.content_rect, light_rect, "native slot unchanged");

        // Flip back to light: the light rail returns byte-identically.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("flip back to light");
        flush_frame(&mut client);
        let light_again = client.snapshot().expect("light return snapshot");
        assert_eq!(light_again.shell_modes.theme, "light");
        assert_eq!(
            light_again.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_light_chrome,
            "light chrome wash restored"
        );
        assert_eq!(light_again.active_web_view, Some(first_view));
        assert_eq!(light_again.engine_sessions, sessions);
        assert_eq!(light_again.sidebar_width, light_width);
        assert_eq!(light_again.content_rect, light_rect);
        assert_eq!(light_again.active_tab_index, Some(0));

        // Clearing the override returns to the observed system appearance
        // without churn (the snapshot always reports the effective mode).
        client
            .send_ok(&dsl::set_appearance_override(None))
            .expect("clear override");
        flush_frame(&mut client);
        let system = client.snapshot().expect("system appearance snapshot");
        assert!(
            system.shell_modes.theme == "light" || system.shell_modes.theme == "dark",
            "the effective mode follows the system appearance"
        );
        assert_eq!(system.active_web_view, Some(first_view));
        assert_eq!(system.engine_sessions, sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
