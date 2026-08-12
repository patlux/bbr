//! BBR-owned Tab/Favorite context-menu presentation (task packet
//! `tab-favorite-context-menus`; DESIGN.md §11.13/§15) — the
//! presentation-only half of the production Tab/Favorite menu.
//!
//! The app remains the sole menu-state authority: it resolves the
//! `TabMenuSource` keyed by the exact persistent `TabId`, computes the
//! availability matrix (portable `bbr-browser::TabMenuController`), maps it
//! onto this plain [`TabMenuModel`] with pre-bound callbacks, computes the
//! clamped placement, and owns dismissal/focus choreography. This module
//! renders the card anatomy only — `material:overlay`, `radius:md`,
//! `elev:2`, `density:row`-tall entries, the §9 focus ring, the shared
//! keyboard/pointer cursor, the danger text/glyph companion, and one-level
//! submenu cards (`Move to Folder` / `Move to Space`) drawn to the right of
//! the parent row (edge-flipped by the caller's geometry).
//!
//! **No domain authority:** the model carries stable presentation ids,
//! labels, enablement, danger, and nested submenu rows — never a `TabId`,
//! space, profile, session, or controller identifier. The callbacks are
//! already bound by `bbr-app` to the real `TabId` that invoked the menu.
//!
//! Pointer paths: each **enabled** row's `on_hover` selects it (and, for a
//! submenu-bearing row, opens its submenu) and its mouse-down activates it;
//! disabled rows carry no activation listener and do not stop propagation,
//! so a press on one dismisses the menu exactly like a native menu's
//! disabled entry. The caller owns the outside-press dismissal.

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::icons::Icon;
use crate::theme::elevation::elev2;
use crate::theme::{Spacing, Theme};

/// Stable §15 id of the Tab/Favorite menu card (DESIGN.md §11.13/§15).
pub const TAB_MENU_ID: &str = "bbr-tab-menu";
/// Stable §15 id of the Favorite menu card (the same menu family; the app
/// registers the id it presents).
pub const FAVORITE_MENU_ID: &str = "bbr-favorite-menu";
/// The §15 id prefix of a top-level action row (`bbr-tab-menu-<action>`).
pub const TAB_MENU_ROW_PREFIX: &str = "bbr-tab-menu";
/// The §15 id prefix of a `Move to Folder` submenu row
/// (`bbr-tab-menu-folder-<n>`).
pub const TAB_MENU_FOLDER_PREFIX: &str = "bbr-tab-menu-folder";
/// The §15 id prefix of a `Move to Space` submenu row
/// (`bbr-tab-menu-space-<n>`).
pub const TAB_MENU_SPACE_PREFIX: &str = "bbr-tab-menu-space";
/// The §15 id of the B2 bookmark row (`Bookmark Page` / `Remove Bookmark`).
pub const TAB_MENU_BOOKMARK_ID: &str = "bbr-tab-menu-bookmark";
/// The §15 id prefix of the close rows (`bbr-tab-menu-close-<scope>`).
pub const TAB_MENU_CLOSE_PREFIX: &str = "bbr-tab-menu-close";

/// One deterministic menu row's presentation: the stable §15 id, the
/// display label, the genuine enablement computed by the app at open, the
/// danger flag (text/glyph companion required), and the optional one-level
/// submenu rows (folders / spaces / close scopes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabMenuRow {
    pub id: SharedString,
    pub label: SharedString,
    pub enabled: bool,
    pub danger: bool,
    pub submenu: Option<Vec<TabMenuRow>>,
}

impl TabMenuRow {
    /// A plain action row (no submenu).
    pub fn action(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        enabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled,
            danger: false,
            submenu: None,
        }
    }

    /// A danger action row (the text carries a glyph companion).
    pub fn danger(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        enabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled,
            danger: true,
            submenu: None,
        }
    }

    /// A submenu-bearing row (folders / spaces).
    pub fn with_submenu(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        enabled: bool,
        submenu: Vec<TabMenuRow>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled,
            danger: false,
            submenu: Some(submenu),
        }
    }
}

/// The menu card's presentation model. The theme is passed separately to
/// the render function (the anchor's model/callback contract), so the model
/// is plain presentation data: focus, the shared keyboard/pointer cursor,
/// the open submenu's parent row and its cursor, and the deterministic row
/// model in render order. `source_key` is the stable presentation source
/// key (the app registers `bbr-tab-menu` or `bbr-favorite-menu`); it
/// carries no domain identity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabMenuModel {
    /// Whether the menu presentation owns keyboard focus (the §9 accent
    /// ring follows it).
    pub focused: bool,
    /// The keyboard cursor / hovered top-level row.
    pub selected: Option<usize>,
    /// The top-level row whose submenu card is currently open.
    pub open_submenu: Option<usize>,
    /// The keyboard cursor / hovered row inside the open submenu.
    pub submenu_selected: Option<usize>,
    /// The deterministic top-level row model in render order.
    pub rows: Vec<TabMenuRow>,
    /// The stable presentation source key (`bbr-tab-menu` /
    /// `bbr-favorite-menu`).
    pub source_key: SharedString,
}

/// The card's resolved placement rect in host layout coordinates (top-left
/// origin, points) — the clamped placement the app's geometry resolves,
/// passed in so the card draws at the resolved rect without importing the
/// portal/geometry types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabMenuRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TabMenuRect {
    /// A fresh rect from its corner and size.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// The row hover callback type: the row index and the hover flag, already
/// bound by the app to the real `TabId` (enabled rows only).
pub type TabMenuHoverCallback = Rc<dyn Fn(usize, &bool, &mut Window, &mut App)>;

/// The row press callback type: the row index and the pointer event,
/// already bound by the app to the real `TabId` (enabled rows only).
pub type TabMenuPressCallback = Rc<dyn Fn(usize, &MouseDownEvent, &mut Window, &mut App)>;

/// One menu card's surface: the §11.4 command-surface family (`material:
/// overlay`, `radius:md`, `elev:2`) with `density:row`-tall rows at `sp:3`
/// horizontal / `sp:2` vertical padding inside `sp:1` card padding. The
/// caller owns the outside-press dismissal; the card does not stop
/// propagation.
///
/// `selected` is the shared keyboard/pointer cursor; a row with an open
/// submenu renders the `ChevronRight` glyph companion. Danger rows render
/// their text in `status_error` with the §11.13 glyph companion (`Close`
/// for Close Tab). The open submenu card renders at `submenu_rect` to the
/// right of the parent card (edge-flipped by the caller's geometry).
#[allow(clippy::too_many_arguments)]
pub fn tab_menu_card(
    theme: &Theme,
    model: &TabMenuModel,
    rect: TabMenuRect,
    submenu_rect: Option<TabMenuRect>,
    on_row_hover: impl Fn(usize, &bool, &mut Window, &mut App) + 'static,
    on_row_press: impl Fn(usize, &MouseDownEvent, &mut Window, &mut App) + 'static,
    on_submenu_hover: impl Fn(usize, &bool, &mut Window, &mut App) + 'static,
    on_submenu_press: impl Fn(usize, &MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let on_row_hover = Rc::new(on_row_hover);
    let on_row_press = Rc::new(on_row_press);
    let on_submenu_hover = Rc::new(on_submenu_hover);
    let on_submenu_press = Rc::new(on_submenu_press);
    let rows: Vec<Stateful<Div>> = model
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            render_row(
                theme,
                row,
                index,
                model.selected == Some(index),
                model.open_submenu == Some(index),
                on_row_hover.clone(),
                on_row_press.clone(),
            )
        })
        .collect();
    let mut card = div()
        .id(if model.source_key.is_empty() {
            SharedString::from(TAB_MENU_ID)
        } else {
            model.source_key.clone()
        })
        // The stable §15 id is also registered as a test-support
        // `debug_selector` target (the same registration the Tab Row /
        // Favicon components use, so the Gallery E2E can resolve the card
        // through GPUI's `debug_bounds`). No-op outside test builds.
        .debug_selector(move || {
            if model.source_key.is_empty() {
                TAB_MENU_ID.to_string()
            } else {
                model.source_key.to_string()
            }
        })
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
        .rounded(theme.radius.medium())
        .bg(p.surface_overlay)
        // The §9 focus ring: the menu card carries the accent border while
        // the menu owns keyboard focus (the main-window menu handle).
        .border_2()
        .border_color(if model.focused { p.accent } else { p.hairline })
        .shadow(elev2(theme))
        .py(Spacing::sp1())
        .flex()
        .flex_col()
        .children(rows);
    if let (Some(submenu_index), Some(submenu_rect)) = (model.open_submenu, submenu_rect) {
        if let Some(submenu) = model
            .rows
            .get(submenu_index)
            .and_then(|row| row.submenu.as_ref())
        {
            let sub_rows: Vec<Stateful<Div>> = submenu
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    render_row(
                        theme,
                        row,
                        index,
                        model.submenu_selected == Some(index),
                        false,
                        on_submenu_hover.clone(),
                        on_submenu_press.clone(),
                    )
                })
                .collect();
            card = card.child(
                div()
                    .id(SharedString::from(format!("{TAB_MENU_ID}-submenu")))
                    .debug_selector(|| format!("{TAB_MENU_ID}-submenu"))
                    .absolute()
                    .left(px(submenu_rect.x))
                    .top(px(submenu_rect.y))
                    .w(px(submenu_rect.width))
                    .h(px(submenu_rect.height))
                    .rounded(theme.radius.medium())
                    .bg(p.surface_overlay)
                    .border_1()
                    .border_color(p.hairline)
                    .shadow(elev2(theme))
                    .py(Spacing::sp1())
                    .flex()
                    .flex_col()
                    .children(sub_rows),
            );
        }
    }
    card
}

/// One row of a menu card: the stable id, the label, the shared cursor
/// (accent rail + wash), the submenu chevron, the danger text/glyph
/// companion, and the production hover/press listeners for enabled rows.
fn render_row(
    theme: &Theme,
    row: &TabMenuRow,
    index: usize,
    is_selected: bool,
    submenu_open: bool,
    on_row_hover: TabMenuHoverCallback,
    on_row_press: TabMenuPressCallback,
) -> Stateful<Div> {
    let p = theme.palette;
    let mut el = div()
        .id(row.id.clone())
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .text_size(theme.type_.label.size)
        .font_weight(theme.type_.label.weight)
        .text_color(if !row.enabled {
            p.ink_faint
        } else if row.danger {
            p.status_error
        } else if is_selected {
            p.ink
        } else {
            p.ink_secondary
        })
        .child(div().flex_1().min_w_0().child(row.label.clone()));
    if row.submenu.is_some() {
        el = el.child(
            div()
                .size(theme.density.icon())
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::ChevronRight.canvas(theme, theme.density.icon())),
        );
    }
    if row.danger {
        // DESIGN.md §11.13: the danger state has a text/glyph companion;
        // color never carries it alone.
        el = el.child(
            div()
                .size(theme.density.icon())
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::Close.canvas(theme, theme.density.icon())),
        );
    }
    if row.enabled {
        let on_row_hover = on_row_hover.clone();
        let on_row_press = on_row_press.clone();
        el = el
            .cursor_pointer()
            .on_hover(move |hovered: &bool, window: &mut Window, cx: &mut App| {
                on_row_hover(index, hovered, window, cx)
            })
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    on_row_press(index, event, window, cx)
                },
            );
    }
    if is_selected {
        el = el.bg(p.tint_selected).border_l_2().border_color(p.accent);
    } else if submenu_open {
        el = el.bg(p.wash_hover);
    }
    el
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_defaults_are_closed_and_empty() {
        let model = TabMenuModel::default();
        assert!(!model.focused);
        assert_eq!(model.selected, None);
        assert_eq!(model.open_submenu, None);
        assert_eq!(model.submenu_selected, None);
        assert!(model.rows.is_empty());
        assert!(model.source_key.is_empty());
    }

    #[test]
    fn row_constructors_carry_id_label_enablement_danger_and_submenu() {
        let plain = TabMenuRow::action("bbr-tab-menu-reload", "Reload Tab", true);
        assert_eq!(plain.id, "bbr-tab-menu-reload");
        assert_eq!(plain.label, "Reload Tab");
        assert!(plain.enabled);
        assert!(!plain.danger);
        assert!(plain.submenu.is_none());

        let danger = TabMenuRow::danger("bbr-tab-menu-close-tab", "Close Tab", true);
        assert!(danger.danger);

        let sub = TabMenuRow::with_submenu(
            "bbr-tab-menu-move-space",
            "Move to Space",
            true,
            vec![TabMenuRow::action("bbr-tab-menu-space-0", "Default", true)],
        );
        assert_eq!(sub.submenu.as_ref().expect("submenu").len(), 1);
        assert_eq!(
            sub.submenu.as_ref().expect("submenu")[0].id,
            "bbr-tab-menu-space-0"
        );
    }

    #[test]
    fn stable_ids_match_the_family_contract() {
        assert_eq!(TAB_MENU_ID, "bbr-tab-menu");
        assert_eq!(FAVORITE_MENU_ID, "bbr-favorite-menu");
        assert_eq!(TAB_MENU_ROW_PREFIX, "bbr-tab-menu");
        assert_eq!(TAB_MENU_FOLDER_PREFIX, "bbr-tab-menu-folder");
        assert_eq!(TAB_MENU_SPACE_PREFIX, "bbr-tab-menu-space");
        assert_eq!(TAB_MENU_BOOKMARK_ID, "bbr-tab-menu-bookmark");
        assert_eq!(TAB_MENU_CLOSE_PREFIX, "bbr-tab-menu-close");
        assert_eq!(
            format!("{TAB_MENU_FOLDER_PREFIX}-0"),
            "bbr-tab-menu-folder-0"
        );
        assert_eq!(format!("{TAB_MENU_SPACE_PREFIX}-3"), "bbr-tab-menu-space-3");
        assert_eq!(
            format!("{TAB_MENU_CLOSE_PREFIX}-tab"),
            "bbr-tab-menu-close-tab"
        );
    }

    #[test]
    fn rect_keeps_the_resolved_placement() {
        let rect = TabMenuRect::new(8.0, 8.0, 220.0, 148.0);
        assert_eq!(rect.x, 8.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 220.0);
        assert_eq!(rect.height, 148.0);
        assert_eq!(rect, TabMenuRect::new(8.0, 8.0, 220.0, 148.0));
    }
}
