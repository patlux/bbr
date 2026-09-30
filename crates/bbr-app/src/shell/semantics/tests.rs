use super::*;

fn browser() -> ControllerSnapshot {
    let tab = |index: usize, active: bool, url: &str, title: &str| bbr_browser::TabSnapshot {
        index,
        url: url.into(),
        title: title.into(),
        pinned: false,
        loading: false,
        crashed: false,
        unresponsive: false,
        active,
        web_view: Some(bbr_web::WebViewId(1)),
        lifecycle: bbr_browser::TabLifecycle::Warm,
        last_used: 0,
        favicon: bbr_browser::TabFavicon::Monogram,
    };
    ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: bbr_browser::SpaceId(1),
        tabs: vec![tab(0, true, "https://example.com", "Example")],
        all_tabs: vec![],
        profiles: vec![bbr_browser::ProfileSnapshot {
            index: 0,
            id: bbr_web::ProfileId(1),
            title: "Default".into(),
            hue: bbr_browser::ProfileHue::Cobalt,
        }],
        spaces: vec![bbr_browser::SpaceSnapshot {
            index: 0,
            id: bbr_browser::SpaceId(1),
            profile: bbr_web::ProfileId(1),
            title: "Default".into(),
            active_tab: Some(0),
            active_view: Some(bbr_web::WebViewId(1)),
            active: true,
            tab_count: 1,
        }],
        active_index: Some(0),
        nav: Default::default(),
        web_view_hosted: true,
        engine_sessions: 1,
        active_view: Some(bbr_web::WebViewId(1)),
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
        preview: None,
    }
}

fn row(index: usize, state: TabVisualState) -> RowVisuals {
    RowVisuals {
        index,
        title: "Example".into(),
        state,
        favicon: Some(bbr_ui::components::FaviconSpec {
            image: None,
            letter: 'E',
            hue: 0,
        }),
        pinned: false,
        show_close: true,
        reveal_close: true,
        split: None,
    }
}

fn space_chip() -> SpaceChipVisuals {
    SpaceChipVisuals {
        space: bbr_browser::SpaceId(1),
        title: "Default".into(),
        profile_title: "Default".into(),
        identity: "book".into(),
        theme: bbr_browser::SpaceTheme::Cobalt,
        intensity: bbr_domain::TintIntensity::Standard,
        selected: true,
        drag_target: false,
        companion: None,
    }
}

fn input<'a>(
    focus: SemanticFocus,
    browser: &'a ControllerSnapshot,
    rows: &'a [RowVisuals],
    space_chips: &'a [SpaceChipVisuals],
) -> SemanticInput<'a> {
    SemanticInput {
        browser,
        focus,
        command: SemanticCommand {
            mode: "inactive".into(),
            address: "example.com".into(),
            address_identity: "secure".into(),
            text: "".into(),
            hint: None,
            profile_hint: None,
            results: vec![],
        },
        sidebar_visible: true,
        sidebar_width: 300.0,
        divider_label: "bbr-sidebar-divider".into(),
        command_surface: CommandSurface::None,
        rows,
        space_chips,
        favorites: &[],
        favorites_band_targeted: false,
        folder_rows: &[],
        folder_members: &[],
        plus_menu: bbr_ui::components::menu::PlusMenuState::default(),
        content: SemanticContent::Native,
    }
}

fn find<'a>(nodes: &'a [SemanticNode], id: &str) -> Option<&'a SemanticNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = find(&node.children, id) {
            return Some(found);
        }
    }
    None
}

#[test]
fn tree_exposes_roles_names_values_and_hierarchy() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let tree = semantic_tree(&input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    ));
    let root = &tree[0];
    assert_eq!(root.role, SemanticRole::Root);
    let sidebar = find(&tree, "bbr-sidebar").expect("sidebar node");
    assert_eq!(sidebar.role, SemanticRole::Sidebar);
    // The hierarchy: sidebar → tab-list → tab-row → close button.
    let tab_list = find(&tree, "bbr-tab-list").expect("tab list");
    assert_eq!(tab_list.role, SemanticRole::List);
    let row = find(&tree, "bbr-tab-row-0").expect("tab row");
    assert_eq!(row.role, SemanticRole::Row);
    assert_eq!(row.name.as_deref(), Some("Example"));
    assert!(row.selected, "active tab renders selected");
    let close = find(&tree, "bbr-tab-close-0").expect("close control");
    assert_eq!(close.role, SemanticRole::Button);
    assert_eq!(close.name.as_deref(), Some("Close tab"));
    assert!(close.enabled);
    // The permanent space strip: one chip row per space with its identity
    // child; the active space's chip renders selected.
    let strip = find(&tree, "bbr-space-strip").expect("space strip");
    assert_eq!(strip.role, SemanticRole::List);
    assert_eq!(strip.name.as_deref(), Some("Spaces"));
    let chip = find(&tree, "bbr-space-chip-1").expect("space chip");
    assert_eq!(chip.role, SemanticRole::Row);
    assert_eq!(
        chip.name.as_deref(),
        Some("Default · in Profile · Default"),
        "the accessible name carries the Space and Profile names"
    );
    assert!(chip.selected, "the active space's chip renders selected");
    let identity = find(&tree, "bbr-space-identity-1").expect("identity child");
    assert_eq!(identity.role, SemanticRole::Title);
    assert_eq!(identity.name.as_deref(), Some("book"));
    assert!(
        identity.selected,
        "the identity mirrors the chip's selection"
    );
    // The divider exposes its width value and stable id.
    let divider = find(&tree, "bbr-sidebar-divider").expect("divider");
    assert_eq!(divider.role, SemanticRole::Divider);
    assert_eq!(divider.value.as_deref(), Some("300 pt"));
}

#[test]
fn cross_context_profile_hint_exposes_chip_and_label_under_the_field() {
    // §11.3: the command field renders the destination profile chip + the
    // "in Profile · <title>" label as a typed Chip node (with the §12 chip
    // monogram child) for one interaction after a cross-profile switch.
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    input.command.profile_hint = Some(SemanticProfileHint {
        title: "Private".into(),
        monogram: "PR".into(),
        profile_index: 1,
    });
    let tree = semantic_tree(&input);
    // At rest the §11.5 address pill (display-only Button) is the command
    // slot; the cross-context hint renders under it.
    let address = find(&tree, "bbr-address").expect("address pill");
    let hint = address
        .children
        .iter()
        .find(|node| node.id == "bbr-command-profile-hint")
        .expect("profile hint node under the address pill");
    assert_eq!(hint.role, SemanticRole::Chip);
    assert_eq!(hint.name.as_deref(), Some("in Profile · Private"));
    let chip = hint
        .children
        .iter()
        .find(|node| node.id == "bbr-profile-chip-1")
        .expect("hint chip child");
    assert_eq!(chip.role, SemanticRole::Chip);
    assert_eq!(chip.name.as_deref(), Some("Profile · PR"));
    // At rest (no hint) the node is absent.
    input.command.profile_hint = None;
    let tree = semantic_tree(&input);
    assert!(find(&tree, "bbr-command-profile-hint").is_none());
}

#[test]
fn focus_lands_on_the_owning_list_and_selected_row() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let tree = semantic_tree(&input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: true,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    ));
    let tab_list = find(&tree, "bbr-tab-list").expect("tab list");
    assert!(tab_list.focused, "the focused list is exposed typed");
    let row = find(&tree, "bbr-tab-row-0").expect("tab row");
    assert!(
        row.focused,
        "the selected row of the focused list is focused"
    );
    let strip = find(&tree, "bbr-space-strip").expect("space strip");
    assert!(!strip.focused, "only the owning list is focused");
}

#[test]
fn dragged_row_disables_its_controls() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Dragging)];
    let space_chips = [space_chip()];
    let input = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    let tree = semantic_tree(&input);
    let close = find(&tree, "bbr-tab-close-0").expect("close control");
    assert!(!close.enabled, "a dragged row's controls are disabled");
    assert!(
        find(&tree, "bbr-tab-pin-0").is_none(),
        "rows carry no pin control (DESIGN §11.1)"
    );
}

#[test]
fn content_branch_mirrors_the_render() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    input.content = SemanticContent::Empty {
        profile_line: "in Profile · Default".into(),
    };
    let tree = semantic_tree(&input);
    let pane = find(&tree, "bbr-empty-new-tab").expect("empty pane");
    assert_eq!(pane.role, SemanticRole::Content);
    assert_eq!(pane.value.as_deref(), Some("in Profile · Default"));
    assert!(find(&tree, "bbr-empty-new-tab-button").is_some());

    input.content = SemanticContent::Crash;
    let tree = semantic_tree(&input);
    assert!(find(&tree, "bbr-crash-reload").is_some());
    assert!(find(&tree, "bbr-crash-close").is_some());

    // §11.8a unresponsive pane: the panel node plus the closed recovery set
    // Wait → Reload → Close tab in tree (focus) order, with the exact §15
    // names.
    input.content = SemanticContent::Unresponsive;
    let tree = semantic_tree(&input);
    let panel = find(&tree, "bbr-unresponsive-panel").expect("unresponsive pane");
    assert_eq!(panel.role, SemanticRole::Content);
    assert_eq!(panel.name.as_deref(), Some("This page is not responding"));
    let wait = find(&tree, "bbr-unresponsive-wait").expect("wait button");
    assert_eq!(wait.role, SemanticRole::Button);
    assert_eq!(wait.name.as_deref(), Some("Wait"));
    let reload = find(&tree, "bbr-unresponsive-reload").expect("reload button");
    assert_eq!(reload.role, SemanticRole::Button);
    assert_eq!(reload.name.as_deref(), Some("Reload"));
    let close = find(&tree, "bbr-unresponsive-close").expect("close button");
    assert_eq!(close.role, SemanticRole::Button);
    assert_eq!(close.name.as_deref(), Some("Close tab"));
    let buttons: Vec<&str> = panel
        .children
        .iter()
        .filter(|node| node.role == SemanticRole::Button)
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(
        buttons,
        [
            "bbr-unresponsive-wait",
            "bbr-unresponsive-reload",
            "bbr-unresponsive-close"
        ],
        "recovery buttons in focus order"
    );
}

#[test]
fn collapsed_sidebar_leaves_only_root_and_content() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    input.sidebar_visible = false;
    let tree = semantic_tree(&input);
    let sidebar = find(&tree, "bbr-sidebar");
    assert!(sidebar.is_none(), "a collapsed sidebar renders no nodes");
    assert!(find(&tree, "bbr-native-web-slot").is_some());
}

#[test]
fn command_result_surface_exposes_typed_result_rows_under_the_staging_field() {
    // §11.5/§15: while the staging surface (empty New Tab / New Space /
    // Move Tab — a non-address command mode without a floating
    // presentation) shows ranked results, the staging input exposes a typed
    // result list with one Result node per row carrying the selection
    // state; with no results the list is absent. The address pill at rest
    // never carries a result list (the pill is display-only).
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: true,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    input.command.mode = "newtab".into();
    input.command.results = vec![
        SemanticResultRow {
            key: "tab:0".into(),
            title: "Example".into(),
            subtitle: Some("https://example.com".into()),
            selected: true,
            destructive: false,
        },
        SemanticResultRow {
            key: "action:2".into(),
            title: "Close Tab".into(),
            subtitle: None,
            selected: false,
            destructive: true,
        },
    ];
    let tree = semantic_tree(&input);
    let field = find(&tree, "bbr-command-bar").expect("command field");
    let list = field
        .children
        .iter()
        .find(|node| node.id == "bbr-command-results")
        .expect("result list under the command field");
    assert_eq!(list.role, SemanticRole::List);
    assert_eq!(list.children.len(), 2);
    let first = list
        .children
        .iter()
        .find(|node| node.id == "bbr-command-result-tab:0")
        .expect("first result row");
    assert_eq!(first.role, SemanticRole::Result);
    assert_eq!(first.name.as_deref(), Some("Example"));
    assert!(first.selected, "the selected row is marked selected");
    let close = list
        .children
        .iter()
        .find(|node| node.id == "bbr-command-result-action:2")
        .expect("destructive result row");
    assert!(close.value.is_none() || close.name.is_some());
    assert!(!close.selected);
    // No results: the list node is absent.
    input.command.results.clear();
    let tree = semantic_tree(&input);
    let field = find(&tree, "bbr-command-bar").expect("command field");
    assert!(
        field
            .children
            .iter()
            .all(|node| node.id != "bbr-command-results"),
        "no result list without results"
    );
}

#[test]
fn palette_and_overlay_surfaces_mirror_on_the_tree_only_while_open() {
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];

    // Closed: neither surface node exists anywhere on the tree.
    let tree = semantic_tree(&input(
        SemanticFocus {
            command: false,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    ));
    assert!(find(&tree, "bbr-palette").is_none());
    assert!(find(&tree, "bbr-command-overlay").is_none());

    // The centered palette mirrors with its field row, mode chip, and the
    // structural cursor while chrome focus is in the palette field.
    let mut palette_input = input(
        SemanticFocus {
            command: true,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    palette_input.command.mode = "newtab".into();
    palette_input.command.text = "https://example.com".into();
    palette_input.command_surface = CommandSurface::Palette;
    palette_input.command.results = vec![SemanticResultRow {
        key: "tab:0".into(),
        title: "Example".into(),
        subtitle: Some("https://example.com".into()),
        selected: true,
        destructive: false,
    }];
    let tree = semantic_tree(&palette_input);
    let palette = find(&tree, "bbr-palette").expect("palette node while open");
    assert_eq!(palette.role, SemanticRole::Palette);
    assert!(palette.focused);
    let field = find(&tree, "bbr-palette-field").expect("palette field row");
    assert_eq!(field.role, SemanticRole::Field);
    assert_eq!(field.name.as_deref(), Some("https://example.com"));
    assert!(field.focused);
    let mode = find(&tree, "bbr-palette-mode").expect("palette mode chip");
    assert_eq!(mode.role, SemanticRole::Chip);
    assert_eq!(mode.name.as_deref(), Some("newtab"));
    assert!(find(&tree, "bbr-palette-cursor").is_some());
    assert!(find(&tree, "bbr-command-overlay").is_none());
    // The published result rows mirror under the palette field with the
    // surface's own id family; the in-rail list is hidden while a surface
    // presentation is open (the same engine renders its results once).
    let result = find(&tree, "bbr-palette-result-tab:0").expect("palette result row");
    assert_eq!(result.role, SemanticRole::Result);
    assert_eq!(result.name.as_deref(), Some("Example"));
    assert!(result.selected);
    assert!(find(&tree, "bbr-command-results").is_none());

    // The field-anchored overlay mirrors with the field text as its value.
    let mut overlay_input = input(
        SemanticFocus {
            command: true,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    overlay_input.command.mode = "location".into();
    overlay_input.command.text = "https://example.com".into();
    overlay_input.command_surface = CommandSurface::Overlay;
    overlay_input.command.results = vec![SemanticResultRow {
        key: "tab:0".into(),
        title: "Example".into(),
        subtitle: Some("https://example.com".into()),
        selected: true,
        destructive: false,
    }];
    let tree = semantic_tree(&overlay_input);
    let overlay = find(&tree, "bbr-command-overlay").expect("overlay node while open");
    assert_eq!(overlay.role, SemanticRole::Palette);
    assert_eq!(overlay.value.as_deref(), Some("https://example.com"));
    // §11.4: the overlay's own field row carries the full URL and the
    // structural cursor while the overlay owns chrome focus.
    let field = find(&tree, "bbr-command-overlay-field").expect("overlay field row");
    assert_eq!(field.role, SemanticRole::Field);
    assert_eq!(field.name.as_deref(), Some("https://example.com"));
    assert!(field.focused);
    let field_text = find(&tree, "bbr-command-overlay-text").expect("overlay field text");
    assert_eq!(field_text.role, SemanticRole::Title);
    assert_eq!(field_text.name.as_deref(), Some("https://example.com"));
    assert!(find(&tree, "bbr-command-overlay-cursor").is_some());
    // The overlay keeps the §11.5 result ids of the in-rail surface.
    let result = find(&tree, "bbr-command-result-tab:0").expect("overlay result row");
    assert_eq!(result.role, SemanticRole::Result);
    assert!(result.selected);
    assert!(find(&tree, "bbr-palette").is_none());
}

#[test]
fn address_pill_is_a_display_only_button_never_a_field() {
    // §11.5: the sidebar address is a display control — the semantic node is
    // a Button carrying the derived domain, with the domain title and the
    // reserved accessory slot as children; it is never a Field and never
    // carries a cursor or selection, in any state.
    let browser = browser();
    let rows = [row(0, TabVisualState::Selected)];
    let space_chips = [space_chip()];
    let mut input = input(
        SemanticFocus {
            command: true,
            spaces: false,
            tabs: false,
            divider: false,
            folders: false,
            plus_menu: false,
        },
        &browser,
        &rows,
        &space_chips,
    );
    input.command.address = "example.com".into();
    let tree = semantic_tree(&input);
    let pill = find(&tree, "bbr-address").expect("address pill");
    assert_eq!(pill.role, SemanticRole::Button);
    assert_eq!(pill.name.as_deref(), Some("example.com"));
    assert!(pill.enabled);
    assert!(!pill.focused, "the pill is never a focus ring target");
    assert!(find(&tree, "bbr-command-bar").is_none(), "no field at rest");
    assert!(
        find(&tree, "bbr-command-cursor").is_none(),
        "no cursor at rest"
    );
    assert!(
        find(&tree, "bbr-command-text").is_none(),
        "no editable text at rest"
    );
    let domain = find(&tree, "bbr-address-domain").expect("domain title");
    assert_eq!(domain.role, SemanticRole::Title);
    assert_eq!(domain.name.as_deref(), Some("example.com"));
    let accessory = find(&tree, "bbr-address-accessory").expect("accessory slot");
    assert_eq!(accessory.role, SemanticRole::Button);
    assert!(
        !accessory.enabled,
        "the v1 accessory slot is empty (DESIGN.md §17.14 open)"
    );

    // The pill stays display-only behind an open overlay presentation.
    input.command.mode = "location".into();
    input.command.text = "https://example.com/a".into();
    input.command_surface = CommandSurface::Overlay;
    let tree = semantic_tree(&input);
    let behind = find(&tree, "bbr-address").expect("pill behind the overlay");
    assert_eq!(behind.role, SemanticRole::Button);
    assert_eq!(behind.name.as_deref(), Some("example.com"));
    assert!(find(&tree, "bbr-command-bar").is_none());
}

mod u2;
