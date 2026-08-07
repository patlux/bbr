//! Real-app pointer state and typed semantic-tree E2E (Phase-4 closeout).

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::SemanticRole;
use e2e_common::*;
use e2e_semantics_common::*;

#[test]
#[ignore]
fn shell_semantic_tree_and_fallbacks() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let long_title = fixtures.url("/long-title.html");
    let (mut harness, socket, token) = spawn_harness("sem-base", &page_one);

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
        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(
            settled.window_size[0] >= SIDEBAR_DEFAULT + 64.0,
            "settled window must fit the default sidebar",
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush first frame");
        let initial = client.snapshot().expect("initial snapshot");

        // 1. The typed tree mirrors the render: root → sidebar → surfaces.
        assert_eq!(initial.semantics.len(), 1, "one root node");
        let _root = assert_node(
            &initial.semantics,
            "bbr-root",
            SemanticRole::Root,
            Some("BBR window"),
            None,
            true,
            false,
            false,
        );
        let sidebar = assert_node(
            &initial.semantics,
            "bbr-sidebar",
            SemanticRole::Sidebar,
            Some("Sidebar"),
            None,
            true,
            false,
            false,
        );
        // §11.4 compact rail: the sidebar keeps its 7 regions (brand,
        // command field, tabs, status, toggle, footer, divider) while the
        // spaces region stays hidden — its rows are not part of the §9
        // traversal order until the footer `[Spaces]` control reveals them.
        assert_eq!(
            sidebar.children.len(),
            7,
            "brand, command field, tabs, status, toggle, footer, divider"
        );
        assert_ids(
            &initial,
            &[
                "bbr-root",
                "bbr-sidebar",
                "bbr-brand",
                "bbr-command-bar",
                "bbr-tab-list",
                "bbr-status",
                "bbr-sidebar-toggle",
                "bbr-sidebar-divider",
                "bbr-native-web-slot-1",
            ],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id == "bbr-space-list" || id.starts_with("bbr-space-row")),
            "hidden region renders no space rows before the reveal"
        );

        // Hierarchy and typed values: tab list → row → controls.
        let tab_list = assert_node(
            &initial.semantics,
            "bbr-tab-list",
            SemanticRole::List,
            Some("Tabs"),
            None,
            true,
            false,
            false,
        );
        let row = assert_node(
            &initial.semantics,
            "bbr-tab-row-0",
            SemanticRole::Row,
            Some(PAGE_ONE_TITLE),
            None,
            true,
            true,
            false,
        );
        assert!(row.children.iter().any(|c| c.role == SemanticRole::Favicon));
        assert_node(
            &initial.semantics,
            "bbr-tab-title-0",
            SemanticRole::Title,
            Some(PAGE_ONE_TITLE),
            None,
            true,
            true,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-tab-pin-0",
            SemanticRole::Button,
            Some("Pin tab"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-tab-state-0",
            SemanticRole::State,
            Some("selected"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-tab-close-0",
            SemanticRole::Button,
            Some("Close tab"),
            None,
            true,
            false,
            false,
        );
        // The single regular tab renders the §11.2 "Tabs" section label and
        // no pinned section.
        assert_node(
            &initial.semantics,
            "bbr-section-tabs",
            SemanticRole::Section,
            Some("TABS"),
            None,
            true,
            false,
            false,
        );
        assert!(
            find(&initial.semantics, "bbr-section-pinned").is_none(),
            "no pinned region renders no Pinned section"
        );
        assert!(
            tab_list.children.iter().any(|c| c.id == "bbr-section-tabs"),
            "the section label is a child of the tab list"
        );
        assert!(
            tab_list.children.iter().any(|c| c.id == "bbr-tab-row-0"),
            "the row is a child of the tab list"
        );

        // Space surface: reveal the §11.3 region through the footer
        // `[Spaces]` production pointer path, then assert the selected space
        // row, profile chip, count, and the rows live under the space list.
        let revealed = reveal_spaces_region(&mut client);
        assert_node(
            &revealed.semantics,
            "bbr-space-list",
            SemanticRole::List,
            Some("Spaces"),
            None,
            true,
            false,
            false,
        );
        let space = assert_node(
            &revealed.semantics,
            "bbr-space-row-0",
            SemanticRole::Row,
            Some("Default"),
            Some("1"),
            true,
            true,
            false,
        );
        assert_node(
            &revealed.semantics,
            "bbr-profile-chip-0",
            SemanticRole::Chip,
            Some("Profile · DE"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &revealed.semantics,
            "bbr-space-count-0",
            SemanticRole::Count,
            None,
            Some("1"),
            true,
            false,
            false,
        );
        assert!(space.selected, "the active space renders selected");
        assert_node(
            &revealed.semantics,
            "bbr-space-new",
            SemanticRole::Button,
            Some("New Space"),
            None,
            true,
            false,
            false,
        );

        // Command field, status, divider: typed mode/value. The field
        // mirrors the live URL once the title landed (§11.5).
        assert_node(
            &initial.semantics,
            "bbr-command-bar",
            SemanticRole::Field,
            Some(page_one.as_str()),
            Some("inactive"),
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-status",
            SemanticRole::Status,
            Some(PAGE_ONE_TITLE),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-sidebar-divider",
            SemanticRole::Divider,
            Some("Sidebar divider"),
            Some("260 pt"),
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-native-web-slot",
            SemanticRole::Content,
            Some("Web content"),
            None,
            true,
            false,
            false,
        );

        // 2. Long titles and missing favicons flow into the typed tree from
        // the same row_visuals the render uses (§4 truncation + §7.2
        // monogram fallback + §11.9 title fallback).
        let blank = open_tab(&mut client, &mut since, "about:blank", None);
        assert_eq!(blank, WebViewId(2), "sequential ids");
        let _long = open_tab(&mut client, &mut since, &long_title, Some(LONG_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush fallback frame");
        let fallback = client.snapshot().expect("fallback snapshot");
        assert_eq!(fallback.row_titles[1], "about", "§11.9 scheme fallback");
        assert_node(
            &fallback.semantics,
            "bbr-tab-title-1",
            SemanticRole::Title,
            Some("about"),
            None,
            true,
            false,
            false,
        );
        // The favicon node's name is the §7.2 monogram letter (scheme fallback
        // for the hostless URL): the tree and the render share the source.
        assert_node(
            &fallback.semantics,
            "bbr-tab-favicon-1",
            SemanticRole::Favicon,
            Some("A"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &fallback.semantics,
            "bbr-tab-title-2",
            SemanticRole::Title,
            Some(LONG_TITLE),
            None,
            true,
            true,
            false,
        );
        assert_eq!(fallback.active_tab_index, Some(2));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
