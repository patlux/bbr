//! Unit tests for the BBR page-background context-menu controller (task
//! packets `bbr-context-menus-over-web` and
//! `page-background-context-menu`). Pure rules: the row model and its
//! genuine enablement, the B2 bookmark gate (Bookmark Page / Remove
//! Bookmark / omitted), the deterministic disabled reasons, the omitted
//! unowned capabilities (Select All / Share / Save Page As / View Page
//! Source / Inspect never render), the disabled-skipping keyboard cursor,
//! the anchored placement clamp (height from the actual row model), and
//! the per-view action routing through the recorded engine seam.

use bbr_web::{WebAction, WebEngine, WebViewId};

use crate::portal::state::{DismissalPolicy, PortalRequest};

use super::*;

fn items(
    can_back: bool,
    can_forward: bool,
    url: Option<&str>,
    bookmark: BookmarkRow,
) -> Vec<ContextMenuItem> {
    ContextMenuController::build_items(can_back, can_forward, url, bookmark)
}

fn keys(items: &[ContextMenuItem]) -> Vec<&'static str> {
    items.iter().map(|item| item.key).collect()
}

#[test]
fn row_model_is_deterministic_and_enablement_is_genuine() {
    // Fresh history, navigable unsaved URL: Back/Forward disabled with
    // their deterministic reasons, Reload enabled, Bookmark Page enabled
    // (B2 has no record), Copy Page Link enabled.
    let fresh = items(false, false, Some("https://a.example"), BookmarkRow::Save);
    assert_eq!(
        keys(&fresh),
        vec![
            MENU_BACK,
            MENU_FORWARD,
            MENU_RELOAD,
            MENU_BOOKMARK,
            MENU_COPY_LINK
        ]
    );
    assert!(!fresh[0].enabled, "Back is disabled with no history");
    assert_eq!(fresh[0].disabled_reason, Some("No back history"));
    assert!(!fresh[1].enabled, "Forward is disabled with no history");
    assert_eq!(fresh[1].disabled_reason, Some("No forward history"));
    assert!(fresh[2].enabled);
    assert_eq!(fresh[2].disabled_reason, None);
    assert!(fresh[3].enabled, "Bookmark Page enabled for an unsaved URL");
    assert_eq!(fresh[3].title, "Bookmark Page");
    assert!(fresh[4].enabled);
    assert_eq!(fresh[4].disabled_reason, None);

    // Real history availability: Back enabled after one navigation, Forward
    // enabled only after a back navigation — and a saved page flips the
    // bookmark row to Remove Bookmark.
    let back = items(true, false, Some("https://a.example"), BookmarkRow::Remove);
    assert!(back[0].enabled);
    assert_eq!(back[0].disabled_reason, None);
    assert!(!back[1].enabled);
    let forward = items(true, true, Some("https://a.example"), BookmarkRow::Save);
    assert!(forward[0].enabled && forward[1].enabled);

    // No resolvable URL: the bookmark row is omitted (nothing to bookmark
    // truthfully) and Copy Page Link is genuinely disabled with its reason.
    let blank = items(false, false, None, BookmarkRow::Unavailable);
    assert_eq!(
        keys(&blank),
        vec![MENU_BACK, MENU_FORWARD, MENU_RELOAD, MENU_COPY_LINK]
    );
    assert!(!blank[3].enabled, "Copy Page Link disabled without a URL");
    assert_eq!(blank[3].disabled_reason, Some("No page URL to copy"));
}

#[test]
fn bookmark_row_is_capability_gated_by_b2_state() {
    // Save direction: the canonical URL has no active record.
    let save = items(false, false, Some("https://a.example"), BookmarkRow::Save);
    let bookmark = &save[3];
    assert_eq!(bookmark.key, MENU_BOOKMARK);
    assert_eq!(bookmark.id, "bbr-page-menu-bookmark");
    assert_eq!(bookmark.title, "Bookmark Page");
    assert!(bookmark.enabled);

    // Remove direction: an active record exists.
    let remove = items(false, false, Some("https://a.example"), BookmarkRow::Remove);
    let bookmark = &remove[3];
    assert_eq!(bookmark.title, "Remove Bookmark");
    assert!(bookmark.enabled);

    // Unavailable: the row is omitted entirely — never a disabled ghost,
    // never an invented behavior.
    let unavailable = items(
        false,
        false,
        Some("https://a.example"),
        BookmarkRow::Unavailable,
    );
    assert!(
        !keys(&unavailable).contains(&MENU_BOOKMARK),
        "the bookmark row is omitted when B2 cannot operate"
    );
}

#[test]
fn unowned_capabilities_never_render_rows() {
    // Select All (no portable WebAction), Share (no native adapter), Save
    // Page As / View Page Source / Inspect (no integrated capability), and
    // the retired Open in New Tab row must never appear in the row model —
    // omitting beats inventing.
    let all = items(true, true, Some("https://a.example"), BookmarkRow::Save);
    let keys = keys(&all);
    for forbidden in [
        "select-all",
        "share",
        "save",
        "view-source",
        "inspect",
        "open-new-tab",
    ] {
        assert!(
            !keys.contains(&forbidden),
            "the unowned {forbidden:?} row never renders"
        );
    }
    // The stable per-action ids follow the packet's family contract.
    for item in &all {
        assert!(
            item.id.starts_with("bbr-page-menu-"),
            "row id {item:?} belongs to the bbr-page-menu family"
        );
    }
    assert_eq!(all[0].id, "bbr-page-menu-back");
    assert_eq!(all[1].id, "bbr-page-menu-forward");
    assert_eq!(all[2].id, "bbr-page-menu-reload");
    assert_eq!(all[3].id, "bbr-page-menu-bookmark");
    assert_eq!(all[4].id, "bbr-page-menu-copy-link");
}

#[test]
fn first_enabled_lands_on_back_or_reload() {
    let back_items = items(true, false, Some("https://a.example"), BookmarkRow::Save);
    assert_eq!(
        ContextMenuController::first_enabled(&back_items),
        Some(0),
        "Back first when enabled"
    );
    let blank_items = items(false, false, Some("https://a.example"), BookmarkRow::Save);
    assert_eq!(
        ContextMenuController::first_enabled(&blank_items),
        Some(2),
        "Reload when Back/Forward disabled"
    );
    // A non-navigable URL omits the bookmark row; the first enabled row is
    // still Reload.
    let omitted = items(false, false, None, BookmarkRow::Unavailable);
    assert_eq!(
        ContextMenuController::first_enabled(&omitted),
        Some(2),
        "Reload remains the first enabled row with the bookmark row omitted"
    );
}

#[test]
fn keyboard_cursor_skips_disabled_rows_and_wraps() {
    // Back/Forward disabled → the enabled cursor moves only among
    // {Reload, Bookmark Page, Copy Page Link} with wrap-around.
    let mut controller = ContextMenuController {
        state: ContextMenuState {
            items: items(false, false, Some("https://a.example"), BookmarkRow::Save),
            selected: Some(2),
            ..ContextMenuState::default()
        },
    };
    controller.menu_move_selection(1);
    assert_eq!(controller.state.selected, Some(3), "Reload → Bookmark Page");
    controller.menu_move_selection(1);
    assert_eq!(
        controller.state.selected,
        Some(4),
        "Bookmark Page → Copy Page Link"
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
            items: items(true, false, Some("https://a.example"), BookmarkRow::Save),
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
    let placement = context_menu_placement(WebRect::new(1060.0, 680.0, 0.0, 0.0), 5, &theme);
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
    let placement = context_menu_placement(WebRect::new(40.0, 60.0, 0.0, 0.0), 5, &theme);
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

    // The card height derives from the ACTUAL row model: a four-row menu
    // (bookmark row omitted for a non-navigable URL) is one row shorter
    // than the full five-row menu — the placement never leaves a ghost row.
    let five = crate::portal::state::PortalPresentation::from(PortalRequest::native(
        CONTEXT_MENU_PRESENTATION_ID,
        context_menu_placement(WebRect::new(40.0, 60.0, 0.0, 0.0), 5, &theme),
        CONTEXT_MENU_PRIORITY,
        DismissalPolicy::any(),
    ))
    .resolve(viewport, Edges::none());
    let four = crate::portal::state::PortalPresentation::from(PortalRequest::native(
        CONTEXT_MENU_PRESENTATION_ID,
        context_menu_placement(WebRect::new(40.0, 60.0, 0.0, 0.0), 4, &theme),
        CONTEXT_MENU_PRIORITY,
        DismissalPolicy::any(),
    ))
    .resolve(viewport, Edges::none());
    let row_height =
        f32::from(theme.type_.label.size) + 2.0 * f32::from(bbr_ui::theme::Spacing::sp2());
    assert!(
        (five.height - four.height - row_height).abs() < 0.5,
        "the card height tracks the row count: five={} four={} row={row_height}",
        five.height,
        four.height
    );
}

#[test]
fn copy_link_routes_to_the_bound_view_engine_seam() {
    // The action routing is per-view through the recorded engine seam:
    // Copy Page Link targets the bound view's current URL, never a caller
    // string.
    let mut app = crate::engine::NullEngine::default();
    app.create_web_view(WebViewId(7), bbr_web::ProfileId(1), "https://a.example");
    app.action(WebViewId(7), WebAction::CopyLink);
    assert!(
        app.actions.iter().any(|a| a == "copy-link"),
        "the Copy Page Link action reached the engine: {:?}",
        app.actions
    );
}
