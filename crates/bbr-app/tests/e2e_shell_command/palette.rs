//! Palette / overlay real-app E2E (DESIGN.md §11.4/§13/§15, packet U1).
//!
//! Over real WKWebViews: Cmd-T opens the centered `bbr-palette` in New Tab
//! staging mode (same §11.5 engine as the in-rail bar); typing filters and
//! exposes `bbr-palette-result-<k>` ids; Enter commits exactly one new
//! tab/session and moves focus into the page; Escape cancels and restores
//! the origin field text, mode, and chrome/web focus domain. Cmd-L and a
//! pointer click on the §11.5 address control open the anchored
//! `bbr-command-overlay` in Location mode with the full URL selected; the
//! overlay **covers** the control (top/leading edges flush) rather than
//! dropping below it; one pointer press outside (the §13 dismissal layer)
//! closes it and restores the origin domain. Assertions are typed-snapshot
//! and §15 semantic — no sleeps, no pixel comparisons. The mutual-exclusion
//! and profile-hint evidence lives in the focused `palette_surfaces` module.

use super::command_portal;
use super::command_portal::{assert_command_portal_closed, assert_command_portal_open};
use super::*;
use bbr_testkit::protocol::CommandSurface;

#[test]
#[ignore]
fn cmd_t_opens_centered_palette_in_new_tab_staging_and_escape_restores() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-palette", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let before_tabs = first.tabs.len();
        let before_sessions = first.engine_sessions;

        // Cmd-T opens the centered palette in New Tab staging: the §11.4
        // surface state, the staged mode, chrome focus, and the stable
        // `bbr-palette*` ids + §15 semantic mirror. Staged semantics: no
        // tab or session before Enter.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let palette = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Palette,
            deadline(),
        );
        assert_command_portal_open(&palette, CommandSurface::Palette);
        // §18.1 D9: the centered palette is `win:palette` wide for the
        // shell's density (normal: 600 pt) whenever the window can host it
        // with the 8 pt Portal margins.
        assert_eq!(palette.shell_modes.density, "normal");
        let palette_width = f32::from(bbr_ui::theme::Density::Normal.palette_width());
        let palette_rect = palette.portal.rect.expect("resolved palette rect");
        if palette.window_size[0] >= palette_width + 16.0 {
            assert!(
                (palette_rect[2] - palette_width).abs() <= 0.5,
                "palette width {} vs win:palette {palette_width}",
                palette_rect[2]
            );
        }
        let portal_window = palette.portal.portal_window_number;
        assert_eq!(palette.command_bar_mode, CommandBarMode::NewTab);
        assert!(palette.chrome_focused, "Cmd-T focuses the palette field");
        assert_eq!(palette.command_bar_text, "", "staged mode starts empty");
        assert_ids(
            &palette,
            &[
                "bbr-palette",
                "bbr-palette-field",
                "bbr-palette-text",
                "bbr-palette-mode",
                "bbr-palette-cursor",
            ],
        );
        assert!(
            !palette
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-overlay"),
            "the field-anchored overlay never coexists with the palette"
        );
        assert_eq!(palette.tabs.len(), before_tabs, "no tab before Enter");
        assert_eq!(
            palette.engine_sessions, before_sessions,
            "no session before Enter"
        );
        let node = find(&palette.semantics, "bbr-palette").expect("palette semantic node");
        assert_eq!(node.role, SemanticRole::Palette);
        assert_eq!(node.name.as_deref(), Some("Command palette"));
        assert!(node.focused, "the palette field owns chrome focus");
        let mode = find(&palette.semantics, "bbr-palette-mode").expect("mode chip node");
        assert_eq!(mode.role, SemanticRole::Chip);
        assert_eq!(mode.name.as_deref(), Some("newtab"), "staged New Tab chip");
        let field = find(&palette.semantics, "bbr-palette-field").expect("field node");
        assert_eq!(field.role, SemanticRole::Field);
        assert_eq!(field.name.as_deref(), Some("Type a URL…"));
        assert!(
            find(&palette.semantics, "bbr-palette-cursor").is_some(),
            "the block cursor mirrors while the field is focused"
        );

        // Typing filters through the shared engine: the published results
        // expose the palette's own `bbr-palette-result-<k>` ids and the
        // in-rail list ids are hidden (the surface renders the engine's
        // results once).
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "pag".into() })
            .expect("type in palette");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush palette ranking");
        let filtered = client.snapshot().expect("filtered palette snapshot");
        assert_eq!(filtered.command_result_state, CommandResultState::Results);
        assert!(
            filtered
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette-result-")),
            "palette result rows render: {:?}",
            filtered.visible_ids
        );
        assert!(
            !filtered
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-results"),
            "the in-rail result list is hidden while a surface is open"
        );
        assert_eq!(
            filtered.tabs.len(),
            before_tabs,
            "typing never stages a tab"
        );

        // Escape cancels: surface closed, staged mode restored (Inactive),
        // the origin field text (the seeded active URL) restored, focus back
        // to web, nothing created.
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let restored =
            command_portal::wait_command_portal_closed(&mut client, portal_window, deadline());
        assert_command_portal_closed(&restored, portal_window);
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(
            restored.command_bar_text, page_one,
            "the origin field text is restored exactly"
        );
        assert!(!restored.chrome_focused, "web origin restored");
        assert_eq!(restored.tabs.len(), before_tabs);
        assert_eq!(restored.engine_sessions, before_sessions);
        assert!(
            !restored
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette")),
            "no palette ids once closed"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn enter_in_palette_commits_exactly_one_new_tab_and_focuses_the_page() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, _page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-palette-enter", &page_two);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_TWO_TITLE, since);
        }
        let before_tabs = first.tabs.len();
        let before_sessions = first.engine_sessions;
        let before_view = first.active_web_view.expect("source view");

        // Stage a destination in the centered palette: the typed URL ranks
        // the exact-tab row first (persistent identity from the authoritative
        // global collection).
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("stage destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush staging");
        let staged = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Palette,
            deadline(),
        );
        assert_command_portal_open(&staged, CommandSurface::Palette);
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        let expected_key = tab_key(&staged, &page_two, None);
        assert_eq!(
            staged.command_results.first().map(|row| row.key.as_str()),
            Some(expected_key.as_str()),
            "the exact-tab destination ranks first in the palette"
        );
        assert_eq!(
            staged.tabs.len(),
            before_tabs,
            "staged: no tab before Enter"
        );
        assert_eq!(staged.engine_sessions, before_sessions);

        // Enter commits the staged destination: exactly one new tab and one
        // new live session, the fresh page is focused, the surface closes.
        key(&mut client, "enter");
        let after = client.snapshot().expect("post-enter snapshot");
        assert_eq!(after.tabs.len(), before_tabs + 1, "exactly one new tab");
        assert_eq!(
            after.engine_sessions,
            before_sessions + 1,
            "exactly one new live session"
        );
        let new_view = after.active_web_view.expect("fresh view");
        assert_ne!(new_view, before_view, "the fresh view is not the source");
        assert_eq!(after.command_surface, CommandSurface::None);
        assert_eq!(after.command_bar_mode, CommandBarMode::Inactive);
        assert!(!after.chrome_focused, "the fresh page owns focus");
        if after.title.as_deref() != Some(PAGE_TWO_TITLE) {
            wait_title(&mut client, new_view, PAGE_TWO_TITLE, since);
        }

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn cmd_l_opens_field_anchored_overlay_with_full_url_selected_and_escape_restores() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-overlay", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let before_tabs = first.tabs.len();

        // Cmd-L opens the field-anchored overlay in Location mode with the
        // full URL selected; the palette ids never appear; the §15 overlay
        // node carries the field text as its value.
        cmd_l(&mut client, &mut since);
        let overlay = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Overlay,
            deadline(),
        );
        assert_command_portal_open(&overlay, CommandSurface::Overlay);
        let portal_window = overlay.portal.portal_window_number;
        assert_eq!(overlay.command_bar_mode, CommandBarMode::Location);
        assert!(overlay.chrome_focused);
        assert_eq!(overlay.command_bar_text, page_one);
        assert_eq!(
            overlay.command_selection,
            Some([0, page_one.len()]),
            "Cmd-L selects the full URL"
        );
        assert_ids(&overlay, &["bbr-command-overlay"]);
        assert!(
            !overlay
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette")),
            "the centered palette never coexists with the overlay"
        );
        let node = find(&overlay.semantics, "bbr-command-overlay").expect("overlay semantic node");
        assert_eq!(node.role, SemanticRole::Palette);
        assert_eq!(node.name.as_deref(), Some("Command overlay"));
        assert_eq!(node.value.as_deref(), Some(page_one.as_str()));

        // Escape cancels: the overlay closes and the origin web domain and
        // text are restored.
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let restored =
            command_portal::wait_command_portal_closed(&mut client, portal_window, deadline());
        assert_command_portal_closed(&restored, portal_window);
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(restored.command_bar_text, page_one, "origin text restored");
        assert!(!restored.chrome_focused, "web origin restored");
        assert_eq!(restored.tabs.len(), before_tabs, "no tab was created");
        assert!(
            !restored
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-overlay"),
            "no overlay ids once closed"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn address_click_opens_overlay_and_outside_press_closes_web_and_chrome_origins() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-overlay-click", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Web origin: a pointer press on the §11.5 address control opens
        // the overlay that **covers** it (top and leading edges flush with
        // the pill, DESIGN.md §11.4) in Location mode with the full URL
        // selected (the same engine and presentation as Cmd-L).
        client
            .send_ok(&bbr_testkit::Request::ClickAddress)
            .expect("address click");
        since = wait_chrome_focus(&mut client, true, since);
        let overlay = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Overlay,
            deadline(),
        );
        assert_command_portal_open(&overlay, CommandSurface::Overlay);
        let portal_window = overlay.portal.portal_window_number;
        assert_eq!(overlay.command_bar_mode, CommandBarMode::Location);
        assert!(overlay.chrome_focused);
        assert_eq!(overlay.command_bar_text, page_one);
        assert_eq!(
            overlay.command_selection,
            Some([0, page_one.len()]),
            "address click selects the full URL"
        );
        // §11.4: the overlay covers the measured address pill — top and
        // leading edges flush (tolerance 0.5 pt, the suite's geometry bar),
        // the overlay strictly at least as wide/tall — so the address never
        // appears twice and the eye stays where the pointer clicked.
        let address_rect = overlay.address_rect.expect("measured address pill rect");
        let portal_rect = overlay.portal.rect.expect("resolved overlay rect");
        let flush = |a: f32, b: f32| (a - b).abs() <= 0.5;
        assert!(
            flush(portal_rect[0], address_rect[0]) && flush(portal_rect[1], address_rect[1]),
            "top/leading edges flush with the control: pill {address_rect:?}, overlay {portal_rect:?}"
        );
        // §18.1 D9: the overlay is never narrower than 440 pt (and never
        // wider than `win:palette`), however narrow the address field is.
        let min_width = f32::from(bbr_ui::theme::Density::Normal.overlay_min_width());
        assert!(
            portal_rect[2] >= min_width - 0.5
                && portal_rect[2]
                    <= f32::from(bbr_ui::theme::Density::Normal.palette_width()) + 0.5,
            "overlay width {} within 440 pt–win:palette",
            portal_rect[2]
        );
        assert!(
            portal_rect[2] >= address_rect[2] - 0.5 && portal_rect[3] >= address_rect[3] - 0.5,
            "overlay must cover the pill: pill {address_rect:?}, overlay {portal_rect:?}"
        );

        // One pointer press outside the surface (the §13 dismissal layer):
        // closes it and restores the origin (web) focus domain and text.
        client
            .send_ok(&bbr_testkit::Request::ClickContentOutside)
            .expect("outside press");
        let _ = wait_chrome_focus(&mut client, false, since);
        let dismissed =
            command_portal::wait_command_portal_closed(&mut client, portal_window, deadline());
        assert_eq!(dismissed.command_surface, CommandSurface::None);
        assert_command_portal_closed(&dismissed, portal_window);
        assert_eq!(dismissed.command_bar_mode, CommandBarMode::Inactive);
        assert!(!dismissed.chrome_focused, "web origin restored");
        assert_eq!(dismissed.command_bar_text, page_one, "origin text restored");

        // Chrome origin: a pointer-selected tab row owns chrome focus; the
        // address click still opens the overlay and Escape restores the
        // chrome focus domain (never jumps to the page).
        client
            .send_ok(&bbr_testkit::Request::ClickTab { index: 0 })
            .expect("chrome focus");
        assert!(client.snapshot().expect("chrome snapshot").chrome_focused);
        client
            .send_ok(&bbr_testkit::Request::ClickAddress)
            .expect("address click from chrome origin");
        since = wait_chrome_focus(&mut client, true, since);
        let chrome_overlay = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Overlay,
            deadline(),
        );
        assert_command_portal_open(&chrome_overlay, CommandSurface::Overlay);
        assert!(chrome_overlay.chrome_focused);
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, true, since);
        let chrome_restored =
            command_portal::wait_command_portal_closed(&mut client, portal_window, deadline());
        assert_command_portal_closed(&chrome_restored, portal_window);
        assert_eq!(chrome_restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            chrome_restored.chrome_focused,
            "the chrome origin is preserved across the overlay"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
