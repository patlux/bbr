//! G3 Sidebar composition real-app evidence (packet
//! `component-gallery-compositions`): the full §11.4 rail renders through
//! the one `bbr_ui::sidebar` composition with stable §15 ids and the typed
//! semantic tree in §11.4 region order; the app-owned command and
//! downloads children compose without breaking focus/Escape/collapse or
//! mutual exclusion; and the production pointer seams (tab row, favorite
//! tile, folder row, space chip, plus menu) still route through the
//! composition. Every step reuses the existing typed requests, events,
//! snapshots, and deterministic fixtures — no sleeps, pixels, controller
//! mutation, fake engine, or new testkit seam. The split-collapse and
//! dense-window composition scenarios live in the sibling
//! `composition_geometry` module.

use super::*;
use crate::e2e_semantics_common::{assert_node, find};
use bbr_testkit::protocol::{dsl, SemanticRole};

/// The reference-authoritative settled rail order the semantic tree mirrors:
/// titlebar → command field → active Space title → tabs → footer → divider.
/// Favorites insert after the command field when populated; status, the plus
/// menu and downloads panel render only while their genuine state is active.
const RAIL_ORDER_AT_REST: [&str; 6] = [
    "bbr-titlebar",
    "bbr-address",
    "bbr-active-space-title",
    "bbr-tab-list",
    "bbr-footer",
    "bbr-sidebar-divider",
];

fn assert_rail_order(snapshot: &bbr_testkit::protocol::Snapshot, expected: &[&str]) {
    let sidebar = find(&snapshot.semantics, "bbr-sidebar")
        .unwrap_or_else(|| panic!("semantic tree missing bbr-sidebar"));
    let order: Vec<&str> = sidebar
        .children
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(order, expected, "§11.4 rail region order");
}

/// The full §11.4 rail composition over the real app: titlebar controls,
/// command slot (address pill), active Space heading, favorites grid,
/// pinned/regular tabs, folders, visible New Tab action, conditional status,
/// footer with the permanent space strip and plus control, and the resize
/// divider all render through the one `bbr_ui::sidebar` composition with
/// stable §15 ids; the typed semantic tree mirrors the production region
/// order; no legacy `[Spaces]` reveal surface remains (one rail authority);
/// and the production pointer seams (tab row, favorite tile, folder row,
/// space chip, plus menu) still route through the composition.
#[test]
#[ignore]
fn full_rail_composition_region_and_semantic_ordering() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-composition", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_profile = initial.active_profile;

        // Full rail at rest: every settled reference-authoritative region
        // id is present — titlebar controls, address pill, active Space
        // heading, visible New Tab action, footer and divider. The obsolete
        // standalone brand and settled status regions stay absent, and empty
        // favorites/folders regions hide their labels.
        assert_ids(
            &initial,
            &[
                "bbr-sidebar",
                "bbr-titlebar",
                "bbr-titlebar-sidebar-toggle",
                "bbr-address",
                "bbr-address-domain",
                "bbr-active-space-title",
                "bbr-new-tab",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
                "bbr-footer",
                "bbr-downloads",
                "bbr-space-strip",
                "bbr-plus",
                "bbr-sidebar-divider",
            ],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id == "bbr-brand" || id == "bbr-status"),
            "settled rail omits standalone brand and redundant status"
        );
        // §11.10: the Favorites region is always present — at zero
        // favorites its drop band names the region and the gesture (never a
        // grid); the empty folders region still hides with its label.
        assert_ids(
            &initial,
            &[
                "bbr-section-favorites",
                "bbr-favorites-empty",
                "bbr-favorites-empty-label",
                "bbr-favorites-empty-hint",
            ],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id == "bbr-favorites" || id == "bbr-section-folders"),
            "no grid without favorites; the empty folders region hides"
        );
        // One rail authority: no legacy [Spaces] toggle or revealed-list
        // ids render anywhere in the rail.
        assert!(
            !initial.visible_ids.iter().any(|id| id == "bbr-spaces"
                || id.starts_with("bbr-space-row")
                || id.starts_with("bbr-space-list")),
            "no legacy [Spaces] surface remains"
        );
        // The semantic tree mirrors the §11.4 region order at rest.
        assert_rail_order(&initial, &RAIL_ORDER_AT_REST);
        let footer = find(&initial.semantics, "bbr-footer").expect("footer node");
        let footer_order: Vec<&str> = footer
            .children
            .iter()
            .map(|node| node.id.as_str())
            .collect();
        assert_eq!(
            footer_order,
            vec!["bbr-downloads", "bbr-space-strip", "bbr-plus"],
            "footer is exactly [Downloads] [strip] [+]"
        );
        assert_node(
            &initial.semantics,
            "bbr-active-space-title",
            SemanticRole::Title,
            Some("Default"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-new-tab",
            SemanticRole::Button,
            Some("New Tab"),
            Some("Cmd-T"),
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-titlebar-sidebar-toggle",
            SemanticRole::Button,
            Some("Sidebar (Cmd-B)"),
            Some("expanded"),
            true,
            false,
            false,
        );

        // Open a second tab, favorite the first, create a folder: the
        // regions appear in the rail and the semantic tree folds them into
        // the reference-authoritative order (favorites between the command
        // field and active Space heading; folders inside the tab list).
        let _second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite the first tab");
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after regions");
        let composed = client.snapshot().expect("composed snapshot");
        assert_ids(
            &composed,
            &[
                "bbr-section-favorites",
                "bbr-favorites",
                "bbr-fav-0",
                "bbr-section-folders",
                "bbr-folders",
                "bbr-folder-row-0",
            ],
        );
        assert_rail_order(
            &composed,
            &[
                "bbr-titlebar",
                "bbr-address",
                "bbr-section-favorites",
                "bbr-active-space-title",
                "bbr-tab-list",
                "bbr-footer",
                "bbr-sidebar-divider",
            ],
        );
        let favorites = find(&composed.semantics, "bbr-section-favorites").expect("favorites");
        assert_eq!(favorites.role, SemanticRole::Section);
        let grid = find(&composed.semantics, "bbr-favorites").expect("favorites grid");
        assert_eq!(grid.role, SemanticRole::Grid);
        let tile = find(&composed.semantics, "bbr-fav-0").expect("favorite tile");
        assert_eq!(tile.role, SemanticRole::Tile);
        let folder_row = find(&composed.semantics, "bbr-folder-row-0").expect("folder row");
        assert_eq!(folder_row.role, SemanticRole::Folder);
        assert_eq!(folder_row.value.as_deref(), Some("0"), "empty folder count");

        // Callbacks through the composition: the favorite tile activates
        // the exact favorited tab/session; the folder row entry focuses the
        // folders region; the plus menu commits a real space; the new
        // space chip activates it; the tab row selects. All through the
        // existing production seams.
        let sessions_before = composed.engine_sessions;
        client
            .send_ok(&dsl::click_favorite_tile(0))
            .expect("activate favorite tile");
        let activated = client.snapshot().expect("favorite activated snapshot");
        assert_eq!(activated.active_tab_index, Some(0), "exact favorited tab");
        assert_eq!(
            activated.active_web_view,
            Some(first_view),
            "the exact existing session is presented"
        );
        assert_eq!(activated.engine_sessions, sessions_before);
        assert!(activated.chrome_focused);

        let folder_id = composed.folders[0].id;
        client
            .send_ok(&dsl::click_folder_row(folder_id))
            .expect("focus folder row");
        let focused = client.snapshot().expect("folder focused snapshot");
        assert!(focused.chrome_focused, "folder entry keeps chrome focus");

        // The tab row pointer path selects the second tab in the default
        // space.
        client.send_ok(&dsl::click_tab(1)).expect("click tab row");
        let tab_selected = client.snapshot().expect("tab selected snapshot");
        assert_eq!(tab_selected.active_tab_index, Some(1));
        assert_eq!(tab_selected.engine_sessions, sessions_before);

        // The plus menu commits a real space; the new space chip renders in
        // the permanent strip.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(0))
            .expect("commit New Space");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let created_space = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        let with_space = client.snapshot().expect("new space snapshot");
        assert_eq!(with_space.spaces.len(), 2, "plus menu created a real space");
        assert!(
            with_space
                .visible_ids
                .iter()
                .any(|id| id == &format!("bbr-space-chip-{}", created_space.0)),
            "the new space chip renders in the permanent strip"
        );
        assert_eq!(with_space.engine_sessions, sessions_before);

        // The space chip pointer path activates the new space in the same
        // window without touching sessions.
        client
            .send_ok(&dsl::click_space(created_space))
            .expect("activate new space chip");
        let space_activated = client.snapshot().expect("space activated snapshot");
        assert_eq!(space_activated.active_space, created_space);
        assert_eq!(space_activated.active_profile, default_profile);
        assert_eq!(space_activated.engine_sessions, sessions_before);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The app-owned command and downloads children compose with the rail:
/// Cmd-L opens the field-anchored overlay from the sidebar address pill,
/// ranked results render in-rail, Escape closes and restores the origin
/// focus; the footer `[Downloads]` opens the anchored panel as a sidebar
/// child; the command surface and the downloads panel are mutually
/// exclusive; Cmd-B collapse/expand keeps the command surface and the
/// native geometry exact; and no session/WebView identity churns.
#[test]
#[ignore]
fn command_and_downloads_compose_with_sidebar_focus_and_collapse() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-command-downloads", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let sessions = initial.engine_sessions;
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        let height = initial_rect.content_rect[3];

        // Cmd-L opens the field-anchored overlay from the sidebar address
        // pill; the rail keeps its ids and the native geometry is
        // untouched while the command surface is open.
        keystroke(&mut client, "cmd-l");
        since = wait_chrome_focus(&mut client, true, since);
        let overlay = client.snapshot().expect("overlay snapshot");
        assert_eq!(
            overlay.command_surface,
            bbr_testkit::protocol::CommandSurface::Overlay
        );
        assert!(overlay.chrome_focused);
        assert_ids(&overlay, &["bbr-address", "bbr-command-overlay"]);
        assert_eq!(overlay.active_web_view, Some(first_view));
        assert_eq!(overlay.engine_sessions, sessions);

        // Ranked results publish in-rail through the deterministic flush.
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush ranking");
        let results = client.snapshot().expect("results snapshot");
        assert_eq!(
            results.command_result_state,
            bbr_testkit::protocol::CommandResultState::Results
        );
        assert!(
            !results.command_results.is_empty(),
            "the location corpus publishes results"
        );
        assert!(
            results
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-command-result")),
            "result rows render in the rail"
        );

        // Escape closes the overlay, restores the origin focus domain, and
        // leaves the rail and native geometry exact.
        keystroke(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(
            closed.command_surface,
            bbr_testkit::protocol::CommandSurface::None
        );
        assert!(!closed.chrome_focused);
        assert_eq!(closed.active_web_view, Some(first_view));
        assert_eq!(closed.engine_sessions, sessions);
        assert_eq!(closed.sidebar_width, SIDEBAR_DEFAULT);
        let rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        assert_eq!(rect.content_rect[3], height, "no vertical drift");

        // The footer `[Downloads]` opens the anchored panel as a sidebar
        // child; the command surface stays closed (the panel is a sidebar
        // surface, never a command surface).
        client
            .send_ok(&dsl::click_footer_downloads())
            .expect("open downloads");
        let downloads = client.snapshot().expect("downloads snapshot");
        assert!(downloads.downloads.open);
        assert_ids(&downloads, &["bbr-downloads-panel", "bbr-downloads-empty"]);
        assert_eq!(
            downloads.command_surface,
            bbr_testkit::protocol::CommandSurface::None,
            "the downloads panel never opens a command surface"
        );
        assert_eq!(downloads.active_web_view, Some(first_view));
        assert_eq!(downloads.engine_sessions, sessions);

        // The command surface and the downloads panel coexist: the overlay
        // is the one native-above-web portal, the downloads panel stays a
        // sidebar child — the composition renders both without breaking
        // either (child visibility/exclusion).
        keystroke(&mut client, "cmd-l");
        let _ = wait_chrome_focus(&mut client, true, since);
        let coexisting = client.snapshot().expect("coexisting snapshot");
        assert_eq!(
            coexisting.command_surface,
            bbr_testkit::protocol::CommandSurface::Overlay
        );
        assert!(
            coexisting.downloads.open,
            "the downloads panel stays a visible sidebar child"
        );
        assert_ids(&coexisting, &["bbr-command-overlay", "bbr-downloads-panel"]);
        assert_eq!(coexisting.active_web_view, Some(first_view));
        assert_eq!(coexisting.engine_sessions, sessions);

        // Escape closes the downloads panel first (the §11.12 dismissal
        // priority), then the command surface; the rail and native
        // geometry stay exact throughout. The second Escape restores the
        // origin captured at this Cmd-L — chrome, because the downloads
        // panel was open — so chrome focus is retained.
        keystroke(&mut client, "escape");
        let downloads_closed = client.snapshot().expect("downloads closed snapshot");
        assert!(
            !downloads_closed.downloads.open,
            "Escape dismisses downloads"
        );
        assert_eq!(
            downloads_closed.command_surface,
            bbr_testkit::protocol::CommandSurface::Overlay,
            "the command surface survives the downloads dismissal"
        );
        keystroke(&mut client, "escape");
        let closed = client.snapshot().expect("closed snapshot");
        assert_eq!(
            closed.command_surface,
            bbr_testkit::protocol::CommandSurface::None
        );
        assert!(closed.chrome_focused, "origin was chrome (downloads open)");
        assert_eq!(closed.active_web_view, Some(first_view));
        assert_eq!(closed.engine_sessions, sessions);
        assert_eq!(closed.sidebar_width, SIDEBAR_DEFAULT);
        let rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        assert_eq!(rect.content_rect[3], height, "no vertical drift");

        // Cmd-B collapse/expand with the command surface closed: the rail
        // yields to width 0, the native slot re-flows to the full window,
        // expand restores the last expanded width, and the selected
        // session never changes.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let collapsed = client.snapshot().expect("collapsed snapshot");
        assert!(!collapsed.sidebar_visible);
        assert_eq!(collapsed.sidebar_width, 0.0);
        assert!(
            !collapsed
                .visible_ids
                .iter()
                .any(|id| id == "bbr-sidebar" || id == "bbr-sidebar-divider"),
            "collapsed rail renders no sidebar ids"
        );
        assert_eq!(collapsed.active_web_view, Some(first_view));
        assert_eq!(collapsed.engine_sessions, sessions);
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

        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let expanded = client.snapshot().expect("expanded snapshot");
        assert!(expanded.sidebar_visible);
        assert_eq!(expanded.sidebar_width, SIDEBAR_DEFAULT);
        assert_eq!(expanded.active_web_view, Some(first_view));
        assert_eq!(expanded.engine_sessions, sessions);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT, window_w, height),
            deadline(),
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
