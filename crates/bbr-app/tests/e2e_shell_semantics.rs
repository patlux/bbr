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
        // §11.4/§11.14: the sidebar keeps its 7 regions (titlebar, brand,
        // command field, tabs, status, footer, divider) while the permanent
        // §11.3 space strip lives inside the footer (the flex region between
        // the fixed controls). The titlebar row is the rail's top strip.
        assert_eq!(
            sidebar.children.len(),
            7,
            "titlebar, brand, command field, tabs, status, footer, divider"
        );
        assert_eq!(
            sidebar.children.first().map(|n| n.id.as_str()),
            Some("bbr-titlebar"),
            "the titlebar row is the rail's top strip"
        );
        assert_ids(
            &initial,
            &[
                "bbr-root",
                "bbr-sidebar",
                "bbr-titlebar",
                "bbr-titlebar-sidebar-toggle",
                "bbr-brand",
                "bbr-address",
                "bbr-tab-list",
                "bbr-status",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
                "bbr-sidebar-divider",
                "bbr-native-web-slot-1",
            ],
        );
        assert!(
            initial.visible_ids.iter().any(|id| id == "bbr-space-strip"),
            "the permanent space strip is part of the footer"
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

        // Space strip: the §11.3 footer strip always renders one chip (and
        // its identity child) per space — selected exactly on the active
        // space — under the `bbr-space-strip` list node. The chip and
        // identity semantic ids derive from the real active persistent
        // SpaceId (the launch space is id 2: the default profile consumed
        // allocator id 1), never a hardcoded position literal.
        let present = strip_presence(&mut client);
        assert_node(
            &present.semantics,
            "bbr-space-strip",
            SemanticRole::List,
            Some("Spaces"),
            None,
            true,
            false,
            false,
        );
        let launch_space = initial.active_space;
        let chip = assert_node(
            &present.semantics,
            &format!("bbr-space-chip-{}", launch_space.0),
            SemanticRole::Row,
            Some("Default"),
            None,
            true,
            true,
            false,
        );
        assert!(chip.selected, "the active space's chip renders selected");
        assert_node(
            &present.semantics,
            &format!("bbr-space-identity-{}", launch_space.0),
            SemanticRole::Title,
            Some("code"),
            None,
            true,
            true,
            false,
        );

        // Command slot, status, divider: typed mode/value. The §11.5
        // address pill is a display-only Button carrying the derived domain
        // (scheme/path/query dropped); it is never a Field.
        assert_node(
            &initial.semantics,
            "bbr-address",
            SemanticRole::Button,
            Some("127.0.0.1"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &initial.semantics,
            "bbr-address-domain",
            SemanticRole::Title,
            Some("127.0.0.1"),
            None,
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
            Some("280 pt"),
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
