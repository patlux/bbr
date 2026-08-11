//! BBR-owned context-menu presentation (task packet
//! `bbr-context-menus-over-web`; DESIGN.md §11.4/§11.13/§13/§15) — the
//! presentation half of the app's BBR page menu (Back, Forward, Reload,
//! Copy Link, Open in New Tab) rendered inside the one `NativeAboveWeb`
//! portal window.
//!
//! The app remains the sole menu-state authority: the controller owns the
//! deterministic row model (genuine Back/Forward enablement from the
//! session's real history availability), the dismissal/activation rules,
//! and the focus choreography; the portal root reads one live
//! [`ContextMenuModel`] per render and builds GPUI chrome. This module owns
//! the card anatomy only — `material:overlay`, `radius:md`, `elev:2`,
//! `density:row`-tall entries, the §9 focus ring on the focused card, and
//! the shared keyboard/pointer cursor (selected row) — exactly as the
//! pre-cutover portal render drew it, so §15 ids and the semantic tree are
//! byte-identical.
//!
//! Pointer paths: each **enabled** row's `on_hover` selects it (the shared
//! keyboard/pointer cursor) and its mouse-down activates it; disabled rows
//! carry no activation listener and do not stop propagation, so a press on
//! one dismisses the menu exactly like a native menu's disabled entry. The
//! caller owns the full-window backdrop and the §13 one-pointer-action
//! policy (the portal render stops propagation at the card).

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::theme::elevation::elev2;
use crate::theme::{Spacing, Theme};

/// Stable §15 id of the context-menu card (DESIGN.md §15; moved from the
/// app's portal render, exposed unchanged through the portal snapshot's
/// typed `ids` field).
pub const CONTEXT_MENU_ID: &str = "bbr-context-menu";
/// Stable §15 row-id prefix (`bbr-context-menu-row-<index>`; the row index
/// is the deterministic render position).
pub const CONTEXT_MENU_ROW_PREFIX: &str = "bbr-context-menu-row";

/// One deterministic menu row's presentation: the display title and the
/// genuine enablement computed by the app's controller at open (Back/
/// Forward from the session's real history availability; Copy Link / Open
/// in New Tab from a resolvable page URL). The app maps its controller row
/// (key/title/enabled) onto this DTO; the card renders title + enabled
/// only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMenuRow {
    pub title: SharedString,
    pub enabled: bool,
}

/// The context-menu card's presentation model. The theme is passed
/// separately to the render function (the anchor's model/callback
/// contract), so the model is plain presentation data: focus, the shared
/// keyboard/pointer cursor, and the deterministic row model in render
/// order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextMenuModel {
    /// Whether the menu presentation owns keyboard focus (the §9 accent
    /// ring follows it).
    pub focused: bool,
    /// The keyboard cursor / hovered row.
    pub selected: Option<usize>,
    /// The deterministic row model in render order.
    pub items: Vec<ContextMenuRow>,
}

/// The card's resolved placement rect in host layout coordinates
/// (top-left origin, points) — the anchored portal placement the app's
/// geometry resolves, passed in so the card draws at the resolved rect
/// without importing the portal geometry types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextMenuRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ContextMenuRect {
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

/// The context-menu card at the resolved rect. The card is the §11.4
/// command-surface family: `material:overlay`, `radius:md`, `elev:2`,
/// `density:row`-tall entries with `sp:2` row padding inside `sp:1` card
/// padding. The caller owns the full-window backdrop and pointer
/// propagation policy (the card itself does not stop propagation; the
/// portal render attaches the §13 stop at the card).
///
/// `on_row_hover`/`on_row_press` are the production listener closures the
/// enabled rows route through — the exact app methods a real pointer
/// hover/press calls (selection for hover, select + activate + dismiss for
/// press). Disabled rows render without cursor or listeners.
pub fn context_menu_card(
    theme: &Theme,
    model: &ContextMenuModel,
    rect: ContextMenuRect,
    on_row_hover: impl Fn(usize, &bool, &mut Window, &mut App) + 'static,
    on_row_press: impl Fn(usize, &MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    // Each enabled row routes through the same shared listener closures
    // (Rc-shared so every row owns a `'static` handle to the production
    // callbacks) and pins its own index.
    let on_row_hover = Rc::new(on_row_hover);
    let on_row_press = Rc::new(on_row_press);
    let rows: Vec<Stateful<Div>> = model
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = model.selected == Some(index);
            let mut row = div()
                .id(SharedString::from(format!(
                    "{CONTEXT_MENU_ROW_PREFIX}-{index}"
                )))
                .px(Spacing::sp3())
                .py(Spacing::sp2())
                .flex()
                .flex_row()
                .items_center()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(if item.enabled {
                    if is_selected {
                        p.ink
                    } else {
                        p.ink_secondary
                    }
                } else {
                    p.ink_faint
                })
                .child(item.title.clone());
            if item.enabled {
                // The production selection/activation paths: the exact app
                // methods the hover and mouse-down listeners call.
                let on_row_hover = on_row_hover.clone();
                let on_row_press = on_row_press.clone();
                row = row
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
            row.when(is_selected, |el| {
                el.bg(p.tint_selected).border_l_2().border_color(p.accent)
            })
        })
        .collect();
    div()
        .id(CONTEXT_MENU_ID)
        // The stable §15 id is also registered as a test-support
        // `debug_selector` target (the same registration the Tab Row /
        // Favicon components use, so the Gallery E2E can resolve the card
        // through GPUI's `debug_bounds`). No-op outside test builds.
        .debug_selector(|| CONTEXT_MENU_ID.to_string())
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
        .children(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_defaults_are_closed_and_empty() {
        let model = ContextMenuModel::default();
        assert!(!model.focused);
        assert_eq!(model.selected, None);
        assert!(model.items.is_empty());
    }

    #[test]
    fn row_dto_round_trips_title_and_enablement() {
        let row = ContextMenuRow {
            title: "Reload".into(),
            enabled: true,
        };
        assert_eq!(row.title, "Reload");
        assert!(row.enabled);
        let carried = row.clone();
        assert_eq!(carried, row, "Clone keeps the DTO fields");
    }

    #[test]
    fn model_round_trips_focus_selection_and_rows() {
        let model = ContextMenuModel {
            focused: true,
            selected: Some(2),
            items: vec![
                ContextMenuRow {
                    title: "Back".into(),
                    enabled: false,
                },
                ContextMenuRow {
                    title: "Reload".into(),
                    enabled: true,
                },
            ],
        };
        assert!(model.focused);
        assert_eq!(model.selected, Some(2));
        assert_eq!(model.items.len(), 2);
        assert!(!model.items[0].enabled);
        assert_eq!(model.items[1].title, "Reload");
        assert_eq!(model.clone(), model, "Clone keeps the whole model");
    }

    #[test]
    fn rect_keeps_the_resolved_placement() {
        let rect = ContextMenuRect::new(8.0, 8.0, 220.0, 148.0);
        assert_eq!(rect.x, 8.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 220.0);
        assert_eq!(rect.height, 148.0);
        assert_eq!(rect, ContextMenuRect::new(8.0, 8.0, 220.0, 148.0));
    }

    #[test]
    fn stable_ids_match_the_portal_snapshot_contract() {
        assert_eq!(CONTEXT_MENU_ID, "bbr-context-menu");
        assert_eq!(CONTEXT_MENU_ROW_PREFIX, "bbr-context-menu-row");
        // The row id shape the portal snapshot registers (0..5 fixed rows).
        assert_eq!(
            format!("{CONTEXT_MENU_ROW_PREFIX}-0"),
            "bbr-context-menu-row-0"
        );
        assert_eq!(
            format!("{CONTEXT_MENU_ROW_PREFIX}-4"),
            "bbr-context-menu-row-4"
        );
    }
}
