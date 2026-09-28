//! Real-app titlebar navigation E2E (task packet
//! `titlebar-navigation-controls.md`, DESIGN.md §11.14).
//!
//! Covers over real WKWebViews: the §11.14 titlebar row presenting the
//! native safe-area guide (strip height + traffic-light clearance, never
//! hardcoded coordinates) and the genuine navigation availability of the
//! focused pane; the sidebar toggle click sharing the exact Cmd-B
//! sidebar-width/collapse authority (collapse removes the **complete**
//! BBR-owned titlebar/navigation header — no content toolbar remains — and
//! expand restores the remembered width); Back/Forward/Reload clicks
//! reaching the identical state as the Cmd-[ / Cmd-] / Cmd-R shortcuts;
//! disabled controls staying visible, semantically explained, and no-ops;
//! and split-mode navigation targeting only the focused pane. No
//! sleeps/pixels: every step synchronizes on snapshot state or real WebKit
//! events under a deadline. The shared harness lives in `e2e_common`.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;
#[path = "../e2e_semantics_common/mod.rs"]
mod e2e_semantics_common;

use bbr_testkit::protocol::{dsl, SemanticRole};
use e2e_common::*;
use e2e_semantics_common::assert_node;
mod history;
mod support;
use support::*;

/// §11.14 titlebar presentation: the row renders inside the expanded rail
/// with the live safe-area guide (positive strip height, a non-empty
/// traffic-light clearance), the genuine initial availability (no history,
/// a live session → Reload enabled, Back/Forward disabled but present),
/// and the semantic tree explains every control.
#[test]
#[ignore]
fn titlebar_presents_safe_area_and_availability() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-presentation", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let _ = since;

        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(settled.sidebar_visible, "rail starts expanded");

        // The titlebar row and its four stable controls render inside the
        // rail (no status-row toggle remains).
        assert_ids(
            &settled,
            &[
                "bbr-titlebar",
                "bbr-titlebar-sidebar-toggle",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
            ],
        );
        assert!(
            !settled
                .visible_ids
                .iter()
                .any(|id| id == "bbr-sidebar-toggle"),
            "the pre-cutover status-row toggle is gone"
        );

        // Safe-area evidence: the strip is positive on a titled macOS
        // window and the traffic-light region is a non-degenerate rect the
        // toggle clears (the left edge starts at its right edge — never a
        // hardcoded offset).
        assert!(
            settled.titlebar.safe_area_height > 0.0,
            "native titlebar strip height is positive: {:?}",
            settled.titlebar
        );
        let [tl_x, tl_y, tl_w, tl_h] = settled.titlebar.traffic_lights;
        assert!(tl_w > 0.0 && tl_h > 0.0, "traffic-light region is real");
        assert!(
            tl_x >= 0.0 && tl_y >= 0.0,
            "traffic lights inside the window"
        );
        // Regression: the lights were measured in their superview's
        // coordinates and flipped against the whole window, which put them
        // near the window's bottom edge. In top-left window coordinates they
        // sit inside the titlebar strip.
        assert!(
            tl_y + tl_h <= settled.titlebar.safe_area_height,
            "traffic lights inside the titlebar strip: {:?} vs strip {}",
            settled.titlebar.traffic_lights,
            settled.titlebar.safe_area_height
        );
        assert!(!settled.titlebar.sidebar_collapsed);

        // Genuine availability: a fresh fixture page has no history (Back/
        // Forward disabled but present) and a live session (Reload on).
        assert!(!settled.titlebar.can_go_back);
        assert!(!settled.titlebar.can_go_forward);
        assert!(settled.titlebar.can_reload);
        assert!(!settled.can_go_back, "same authority as the nav snapshot");

        // The semantic tree explains every control with genuine enablement.
        let titlebar = assert_node(
            &settled.semantics,
            "bbr-titlebar",
            SemanticRole::Sidebar,
            Some("Titlebar"),
            None,
            true,
            false,
            false,
        );
        assert_eq!(
            titlebar.children.len(),
            4,
            "toggle + back + forward + reload"
        );
        assert_node(
            &titlebar.children,
            "bbr-titlebar-sidebar-toggle",
            SemanticRole::Button,
            Some("Sidebar (Cmd-B)"),
            Some("expanded"),
            true,
            false,
            false,
        );
        assert_node(
            &titlebar.children,
            "bbr-nav-back",
            SemanticRole::Button,
            Some("Back"),
            None,
            false,
            false,
            false,
        );
        assert_node(
            &titlebar.children,
            "bbr-nav-forward",
            SemanticRole::Button,
            Some("Forward"),
            None,
            false,
            false,
            false,
        );
        assert_node(
            &titlebar.children,
            "bbr-nav-reload",
            SemanticRole::Button,
            Some("Reload"),
            None,
            true,
            false,
            false,
        );
        // The row is the rail's top strip: the first sidebar child.
        let sidebar =
            e2e_semantics_common::find(&settled.semantics, "bbr-sidebar").expect("sidebar node");
        assert_eq!(
            sidebar.children.first().map(|n| n.id.as_str()),
            Some("bbr-titlebar"),
            "the titlebar row is the rail's top strip"
        );

        harness.success = true;
    }));
    if let Err(payload) = result {
        harness.diagnostics(&socket, &token);
        std::panic::resume_unwind(payload);
    }
    harness.terminate();
}

/// §11.14 toggle parity + complete-header disappearance: the titlebar
/// toggle click collapses the Sidebar through the exact Cmd-B authority —
/// the **complete** BBR-owned titlebar/navigation header disappears with
/// the rail (no content toolbar replaces it), the native web slot goes
/// full-bleed, and Cmd-B expands restoring the last expanded width and the
/// full header.
#[test]
#[ignore]
fn titlebar_toggle_parity_and_collapse_removes_the_header() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-toggle", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        let sessions = settled.engine_sessions;
        let window_w = settled.window_size[0];
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        let height = initial_rect.content_rect[3];

        // Remember a non-default width so the expand assertion is exact.
        client
            .send_ok(&dsl::drag_sidebar_divider(40.0))
            .expect("drag to 320");
        flush_frame(&mut client);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, window_w, height),
            deadline(),
        );

        // Pointer toggle collapse: the titlebar toggle click mutates the
        // same sidebar geometry authority as Cmd-B.
        client
            .send_ok(&dsl::click_titlebar_toggle())
            .expect("titlebar toggle click");
        flush_frame(&mut client);
        let collapsed = client.snapshot().expect("collapsed snapshot");
        assert!(!collapsed.sidebar_visible);
        assert_eq!(collapsed.sidebar_width, 0.0);
        assert!(collapsed.titlebar.sidebar_collapsed);
        // The complete BBR-owned header disappears with the rail: no
        // titlebar, no toggle, no navigation cluster, no content-toolbar
        // replacement over the page.
        for id in [
            "bbr-titlebar",
            "bbr-titlebar-sidebar-toggle",
            "bbr-nav-back",
            "bbr-nav-forward",
            "bbr-nav-reload",
            "bbr-sidebar",
        ] {
            assert!(
                !collapsed.visible_ids.iter().any(|present| present == id),
                "collapsed rail must not render {id}"
            );
        }
        assert_view_and_sessions_preserved(&collapsed, first_view, sessions);
        // The collapsed card keeps the native titlebar band as empty,
        // draggable chassis: the traffic lights never sit on web content.
        let collapsed_rect = wait_for_content_rect(
            &mut client,
            collapsed_rect_for(window_w, height, collapsed.titlebar.safe_area_height),
            deadline(),
        );
        assert_eq!(collapsed_rect.content_rect[0], CONTENT_INSET);
        assert_eq!(
            collapsed_rect.content_rect[2],
            window_w - 2.0 * CONTENT_INSET
        );
        // Checked against the native traffic-light geometry itself (not the
        // band height the app sized the card with, which would be circular).
        let [_, lights_top, _, lights_height] = settled.titlebar.traffic_lights;
        let lights_bottom = lights_top + lights_height;
        assert!(
            collapsed_rect.content_rect[1] >= lights_bottom,
            "no web content under the traffic lights: card top {} vs lights bottom {lights_bottom}",
            collapsed_rect.content_rect[1]
        );

        // Cmd-B expands: the last expanded width (320) and the complete
        // header (toggle + cluster) are restored.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let expanded = client.snapshot().expect("expanded snapshot");
        assert!(expanded.sidebar_visible);
        assert_eq!(
            expanded.sidebar_width,
            SIDEBAR_DEFAULT + 40.0,
            "expand restores the last expanded width"
        );
        assert!(!expanded.titlebar.sidebar_collapsed);
        assert_ids(
            &expanded,
            &[
                "bbr-titlebar",
                "bbr-titlebar-sidebar-toggle",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
            ],
        );
        assert_view_and_sessions_preserved(&expanded, first_view, sessions);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, window_w, height),
            deadline(),
        );

        harness.success = true;
    }));
    if let Err(payload) = result {
        harness.diagnostics(&socket, &token);
        std::panic::resume_unwind(payload);
    }
    harness.terminate();
}

/// §11.14 click/keyboard parity: Back/Forward/Reload clicks reach the
/// identical state as the Cmd-[ / Cmd-] / Cmd-R shortcuts over real
/// WKWebView history — same URL, same can_go_back/can_go_forward, same
/// titlebar availability, same session.
#[test]
#[ignore]
fn nav_clicks_match_keyboard_shortcuts() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-nav-parity", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        // The initial fixture must commit before the first real
        // navigation: WebKit cancels a still-provisional load (error
        // -999), which would leave a single-entry history and no back
        // availability (the same two-entry precondition the
        // web-content-gesture suite documents).
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        assert_ids(
            &settled,
            &[
                "bbr-titlebar",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
            ],
        );

        // Two navigations give both history directions over the real
        // WKWebView session. History availability is a session fact the
        // engine reports as `CanGoBackChanged` (the same authority the
        // web-content gesture packet consumes); wait for the event before
        // asserting the enablement.
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate two");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == view),
            since,
        );
        flush_frame(&mut client);
        let at_two = client.snapshot().expect("page two snapshot");
        assert_eq!(at_two.title.as_deref(), Some(PAGE_TWO_TITLE));
        assert!(at_two.titlebar.can_go_back, "back available after one step");
        assert!(!at_two.titlebar.can_go_forward);

        // Cmd-[ (keyboard Back) lands on page one with back now off and
        // forward on. The real session reports both history flips as
        // engine events; wait for them so the snapshot assertion cannot
        // race the commit.
        keystroke(&mut client, "cmd-[");
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, false) if *id == view),
            since,
        );
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, true) if *id == view),
            since,
        );
        flush_frame(&mut client);
        let key_back = client.snapshot().expect("keyboard back snapshot");
        assert_eq!(key_back.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(!key_back.can_go_back);
        assert!(key_back.can_go_forward);
        assert_view_and_sessions_preserved(&key_back, view, settled.engine_sessions);

        // Forward again (so the click path starts from the same state as
        // the keyboard path did).
        keystroke(&mut client, "cmd-]");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == view),
            since,
        );
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, false) if *id == view),
            since,
        );
        flush_frame(&mut client);
        let at_two_again = client.snapshot().expect("forward snapshot");
        assert_eq!(at_two_again.title.as_deref(), Some(PAGE_TWO_TITLE));

        // Titlebar Back click: identical state as the Cmd-[ above.
        client
            .send_ok(&dsl::click_titlebar_back())
            .expect("titlebar back click");
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, false) if *id == view),
            since,
        );
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, true) if *id == view),
            since,
        );
        flush_frame(&mut client);
        let click_back = client.snapshot().expect("click back snapshot");
        assert_eq!(click_back.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(!click_back.can_go_back);
        assert!(click_back.can_go_forward);
        assert_eq!(
            click_back.titlebar, key_back.titlebar,
            "the click and the shortcut reach the identical titlebar availability"
        );
        assert_view_and_sessions_preserved(&click_back, view, settled.engine_sessions);

        // Titlebar Forward click: identical state as the Cmd-] above.
        client
            .send_ok(&dsl::click_titlebar_forward())
            .expect("titlebar forward click");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == view),
            since,
        );
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, false) if *id == view),
            since,
        );
        flush_frame(&mut client);
        let click_forward = client.snapshot().expect("click forward snapshot");
        assert_eq!(click_forward.title.as_deref(), Some(PAGE_TWO_TITLE));
        assert!(click_forward.can_go_back);
        assert!(!click_forward.can_go_forward);
        assert_view_and_sessions_preserved(&click_forward, view, settled.engine_sessions);

        // Reload parity: the Cmd-R shortcut and the titlebar Reload click
        // both reload the focused page (same session, page stays). The
        // reload re-commits the document, so the title event re-fires.
        keystroke(&mut client, "cmd-r");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        flush_frame(&mut client);
        let key_reload = client.snapshot().expect("keyboard reload snapshot");
        assert_eq!(key_reload.title.as_deref(), Some(PAGE_TWO_TITLE));
        assert_view_and_sessions_preserved(&key_reload, view, settled.engine_sessions);

        client
            .send_ok(&dsl::click_titlebar_reload())
            .expect("titlebar reload click");
        let _ = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        flush_frame(&mut client);
        let click_reload = client.snapshot().expect("click reload snapshot");
        assert_eq!(click_reload.title.as_deref(), Some(PAGE_TWO_TITLE));
        assert_eq!(
            click_reload.titlebar.can_reload, key_reload.titlebar.can_reload,
            "reload availability is identical"
        );
        assert_view_and_sessions_preserved(&click_reload, view, settled.engine_sessions);

        harness.success = true;
    }));
    if let Err(payload) = result {
        harness.diagnostics(&socket, &token);
        std::panic::resume_unwind(payload);
    }
    harness.terminate();
}

/// §11.14 disabled controls: visible, semantically explained (the semantic
/// tree reports `enabled = false`), and no-ops — a disabled Back/Forward
/// click changes nothing on the live session.
#[test]
#[ignore]
fn disabled_nav_controls_are_visible_and_noop() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-disabled", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        // The fixture page must be loaded before the no-op assertions: the
        // page, the URL, and the session stay exactly as they were.
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        // Fresh page: Back and Forward disabled, Reload enabled.
        assert!(!settled.titlebar.can_go_back);
        assert!(!settled.titlebar.can_go_forward);
        assert!(settled.titlebar.can_reload);
        // The ids are present (visible) even when disabled.
        assert_ids(&settled, &["bbr-nav-back", "bbr-nav-forward"]);
        // The semantic tree explains the disabled state.
        assert_node(
            &settled.semantics,
            "bbr-nav-back",
            SemanticRole::Button,
            Some("Back"),
            None,
            false,
            false,
            false,
        );

        // Disabled clicks are no-ops: the page, the URL, and the session
        // stay exactly as they were.
        client
            .send_ok(&dsl::click_titlebar_back())
            .expect("disabled back click");
        client
            .send_ok(&dsl::click_titlebar_forward())
            .expect("disabled forward click");
        flush_frame(&mut client);
        let after = client.snapshot().expect("after disabled clicks snapshot");
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(!after.can_go_back);
        assert!(!after.can_go_forward);
        assert_eq!(after.engine_sessions, settled.engine_sessions);
        assert_eq!(after.active_web_view, Some(view));
        assert!(after.titlebar.can_reload, "reload stays enabled");

        harness.success = true;
    }));
    if let Err(payload) = result {
        harness.diagnostics(&socket, &token);
        std::panic::resume_unwind(payload);
    }
    harness.terminate();
}

/// §11.14 split routing: Back/Forward/Reload target the focused pane only —
/// the focused pane's history flips and the sibling pane's state stays
/// untouched.
#[test]
#[ignore]
fn split_navigation_targets_only_the_focused_pane() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-split", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);

        // Two tabs, then the two-pane split. Opening a tab activates it
        // (the same documented behavior the split-geometry suite asserts),
        // so the focused pane is the newly opened tab's view.
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open tab two");
        let opened = client.snapshot().expect("opened snapshot");
        let second = opened.active_web_view.expect("second view");
        if opened.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, second, PAGE_TWO_TITLE, since);
        }
        client.send_ok(&dsl::set_split(0, 1)).expect("set split");
        flush_frame(&mut client);
        let split = client.snapshot().expect("split snapshot");
        let layout = split.pane_layout.expect("pane layout");
        assert_eq!(
            layout.secondary,
            Some(second),
            "secondary is the second view"
        );
        // The focused pane is the domain active tab (the opened tab's
        // view); the primary member is the unfocused sibling.
        assert_eq!(
            layout.focused, second,
            "the opened tab is the active/focused pane"
        );
        assert_ne!(
            layout.focused, layout.primary,
            "the primary member is the unfocused sibling"
        );

        // Navigate the focused pane one step so it has back history of its
        // own (the titlebar navigation routes to the focused pane only).
        client
            .send_ok(&dsl::navigate(page_one.clone()))
            .expect("navigate focused pane");
        since = wait_title(&mut client, second, PAGE_ONE_TITLE, since);
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == second),
            since,
        );
        flush_frame(&mut client);
        let after_nav = client.snapshot().expect("after focused nav snapshot");
        assert!(
            after_nav.titlebar.can_go_back,
            "focused pane gained back history"
        );
        let sibling_before = after_nav
            .pane_rects
            .iter()
            .find(|pane| pane.view == layout.primary)
            .expect("sibling pane rect")
            .rect;

        // Titlebar Back click: the focused pane steps back; the unfocused
        // sibling's native geometry and session are untouched.
        client
            .send_ok(&dsl::click_titlebar_back())
            .expect("titlebar back in split");
        since = wait_title(&mut client, second, PAGE_TWO_TITLE, since);
        let _ = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, false) if *id == second),
            since,
        );
        flush_frame(&mut client);
        let after_back = client.snapshot().expect("after split back snapshot");
        assert!(
            !after_back.titlebar.can_go_back,
            "focused pane back consumed"
        );
        assert!(after_back.titlebar.can_go_forward);
        let sibling_after = after_back
            .pane_rects
            .iter()
            .find(|pane| pane.view == layout.primary)
            .expect("sibling pane rect")
            .rect;
        assert_eq!(
            sibling_after, sibling_before,
            "the unfocused split sibling is never reframed"
        );
        assert!(
            after_back
                .pane_rects
                .iter()
                .any(|pane| pane.view == layout.primary),
            "the sibling session survives"
        );
        assert_eq!(after_back.engine_sessions, after_nav.engine_sessions);

        harness.success = true;
    }));
    if let Err(payload) = result {
        harness.diagnostics(&socket, &token);
        std::panic::resume_unwind(payload);
    }
    harness.terminate();
}
