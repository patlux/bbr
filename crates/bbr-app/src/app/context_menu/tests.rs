//! Unit tests for the BBR context-menu controller (task packet
//! `bbr-context-menus-over-web`). Pure rules: the row model and its genuine
//! enablement, the disabled-skipping keyboard cursor, the anchored placement
//! clamp (the menu always stays fully on screen), and the per-view action
//! routing through the recorded engine seam.

use bbr_web::{WebAction, WebEngine, WebViewId};

use crate::portal::state::{DismissalPolicy, PortalRequest};

use super::*;

fn items(can_back: bool, can_forward: bool, url: Option<&str>) -> Vec<ContextMenuItem> {
    ContextMenuController::build_items(can_back, can_forward, url)
}

fn keys(items: &[ContextMenuItem]) -> Vec<&'static str> {
    items.iter().map(|item| item.key).collect()
}

#[test]
fn row_model_is_deterministic_and_enablement_is_genuine() {
    // Fresh history: Back/Forward disabled, Reload enabled, link entries
    // enabled for a real URL.
    let fresh = items(false, false, Some("https://a.example"));
    assert_eq!(
        keys(&fresh),
        vec![
            MENU_BACK,
            MENU_FORWARD,
            MENU_RELOAD,
            MENU_COPY_LINK,
            MENU_OPEN_NEW_TAB
        ]
    );
    assert!(!fresh[0].enabled, "Back is disabled with no history");
    assert!(!fresh[1].enabled, "Forward is disabled with no history");
    assert!(fresh[2].enabled);
    assert!(fresh[3].enabled);
    assert!(fresh[4].enabled);

    // Real history availability: Back enabled after one navigation, Forward
    // enabled only after a back navigation.
    let back = items(true, false, Some("https://a.example"));
    assert!(back[0].enabled);
    assert!(!back[1].enabled);
    let forward = items(true, true, Some("https://a.example"));
    assert!(forward[0].enabled && forward[1].enabled);

    // No resolvable URL: the link entries are genuinely disabled.
    let blank = items(false, false, None);
    assert!(!blank[3].enabled, "Copy Link disabled without a URL");
    assert!(!blank[4].enabled, "Open in New Tab disabled without a URL");
    assert_eq!(
        keys(&blank),
        vec![
            MENU_BACK,
            MENU_FORWARD,
            MENU_RELOAD,
            MENU_COPY_LINK,
            MENU_OPEN_NEW_TAB
        ]
    );
}

#[test]
fn first_enabled_lands_on_back_or_reload() {
    let back_items = items(true, false, Some("https://a.example"));
    assert_eq!(
        ContextMenuController::first_enabled(&back_items),
        Some(0),
        "Back first when enabled"
    );
    let blank_items = items(false, false, Some("https://a.example"));
    assert_eq!(
        ContextMenuController::first_enabled(&blank_items),
        Some(2),
        "Reload when Back/Forward disabled"
    );
}

#[test]
fn keyboard_cursor_skips_disabled_rows_and_wraps() {
    // Back/Forward disabled → the enabled cursor moves only among
    // {Reload, Copy Link, Open in New Tab} with wrap-around.
    let mut controller = ContextMenuController {
        state: ContextMenuState {
            items: items(false, false, Some("https://a.example")),
            selected: Some(2),
            ..ContextMenuState::default()
        },
    };
    controller.menu_move_selection(1);
    assert_eq!(controller.state.selected, Some(3), "Reload → Copy Link");
    controller.menu_move_selection(1);
    assert_eq!(
        controller.state.selected,
        Some(4),
        "Copy Link → Open in New Tab"
    );
    controller.menu_move_selection(1);
    assert_eq!(
        controller.state.selected,
        Some(2),
        "wrap to the first enabled"
    );
    controller.menu_move_selection(-1);
    assert_eq!(controller.state.selected, Some(4), "up wraps backwards");
    // Down from Back (enabled) lands on Forward (disabled) → skips to Reload.
    let mut controller = ContextMenuController {
        state: ContextMenuState {
            items: items(true, false, Some("https://a.example")),
            selected: Some(0),
            ..ContextMenuState::default()
        },
    };
    controller.menu_move_selection(1);
    assert_eq!(
        controller.state.selected,
        Some(2),
        "Back → Reload, skipping Forward"
    );
}

#[test]
fn placement_anchors_at_the_pointer_and_clamps_fully_on_screen() {
    // A right-click near the bottom-right corner: the menu must stay fully
    // inside the window (margin 8) while its top-left still tracks the
    // pointer when there is room.
    let viewport = Sz::new(1080.0, 700.0);
    let theme = Theme::for_collection(
        bbr_ui::theme::ThemeMode::Dark,
        bbr_ui::theme::ContrastMode::Standard,
        0,
    );
    let placement = context_menu_placement(WebRect::new(1060.0, 680.0, 0.0, 0.0), &theme);
    let rect = crate::portal::state::PortalPresentation::from(PortalRequest::native(
        CONTEXT_MENU_PRESENTATION_ID,
        placement,
        CONTEXT_MENU_PRIORITY,
        DismissalPolicy::any(),
    ))
    .resolve(viewport, Edges::none());
    assert!(
        rect.right() <= viewport.width - 8.0 + 0.5,
        "right edge snapped inside the window: {rect:?}"
    );
    assert!(
        rect.bottom() <= viewport.height - 8.0 + 0.5,
        "bottom edge snapped inside the window: {rect:?}"
    );

    // Top-left placement with room: the menu starts exactly at the pointer.
    let placement = context_menu_placement(WebRect::new(40.0, 60.0, 0.0, 0.0), &theme);
    let rect = crate::portal::state::PortalPresentation::from(PortalRequest::native(
        CONTEXT_MENU_PRESENTATION_ID,
        placement,
        CONTEXT_MENU_PRIORITY,
        DismissalPolicy::any(),
    ))
    .resolve(viewport, Edges::none());
    assert_eq!(rect.x, 40.0, "top-left tracks the pointer");
    assert_eq!(rect.y, 60.0, "top-left tracks the pointer");
    assert_eq!(rect.width, CONTEXT_MENU_WIDTH);
}

#[test]
fn copy_link_routes_to_the_bound_view_engine_seam() {
    // The action routing is per-view through the recorded engine seam:
    // Copy Link targets the bound view's current URL, never a caller string.
    let mut app = crate::engine::NullEngine::default();
    app.create_web_view(WebViewId(7), bbr_web::ProfileId(1), "https://a.example");
    app.action(WebViewId(7), WebAction::CopyLink);
    assert!(
        app.actions.iter().any(|a| a == "copy-link"),
        "the Copy Link action reached the engine: {:?}",
        app.actions
    );
}
