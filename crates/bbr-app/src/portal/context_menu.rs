//! The BBR context menu rendered inside the one native portal window (task
//! packet `bbr-context-menus-over-web`).
//!
//! The browser app remains the sole menu-state authority; the portal root
//! reads one cloned [`crate::app::context_menu::ContextMenuSurface`] per
//! render and builds GPUI chrome. AppKit continues to own only the native
//! child-window boundary. The card is the §11.4 command-surface family:
//! `material:overlay`, `radius:md`, `elev:2`, `density:row`-tall entries.
//!
//! Pointer paths: each row's `on_hover` selects it (the shared keyboard/
//! pointer cursor) and its mouse-down activates it through the production
//! app methods; the card stops propagation while the backdrop owns the §13
//! one-pointer-action dismissal. A disabled row carries no activation
//! listener and does not stop propagation, so a press on it dismisses the
//! menu exactly like a native menu's disabled entry.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful,
};

use super::geometry::Rect;
use super::probe::PortalRoot;
use crate::app::context_menu::{ContextMenuSurface, CONTEXT_MENU_ID, CONTEXT_MENU_ROW_PREFIX};
use crate::theme::Spacing;

/// Build the menu card at the resolved portal rect. The caller owns the
/// full-window backdrop and pointer propagation policy.
pub fn context_menu_card(
    model: &ContextMenuSurface,
    rect: Rect,
    cx: &mut Context<PortalRoot>,
) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
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
                .child(SharedString::from(item.title));
            if item.enabled {
                // The production selection/activation paths: the exact app
                // methods the hover and mouse-down listeners call.
                row = row
                    .cursor_pointer()
                    .on_hover(cx.listener(move |this, _hovered: &bool, _window, cx| {
                        this.context_menu_row_hovered(index, cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _e: &MouseDownEvent, _window, cx| {
                            this.context_menu_row_pressed(index, cx);
                        }),
                    );
            }
            row.when(is_selected, |el| {
                el.bg(p.tint_selected).border_l_2().border_color(p.accent)
            })
        })
        .collect();
    div()
        .id(CONTEXT_MENU_ID)
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
        .shadow(elev2_shadows(theme))
        .py(Spacing::sp1())
        .flex()
        .flex_col()
        .children(rows)
}

impl super::probe::PortalRoot {
    /// Row hover: the production `on_hover` listener path into the shared
    /// menu cursor (the exact app method a real pointer hover calls).
    pub(crate) fn context_menu_row_hovered(&self, index: usize, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                view.menu_select(index, cx)
            });
        });
    }

    /// Row press: the production mouse-down listener path (select +
    /// activate + dismiss; the exact app method a real pointer press calls).
    pub(crate) fn context_menu_row_pressed(&self, index: usize, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.menu_activate(index, window, cx)
            });
        });
    }
}

/// The §11.4 `elev:2` shadow (the same two-stop shadow the command surfaces
/// and the find bar use).
fn elev2_shadows(theme: &crate::theme::Theme) -> Vec<gpui::BoxShadow> {
    use gpui::{hsla, point, BoxShadow};
    match theme.mode {
        crate::theme::ThemeMode::Dark => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 1.0, 0.05),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.45),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(32.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
        crate::theme::ThemeMode::Light => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.04),
                offset: point(px(0.0), px(1.0)),
                blur_radius: px(0.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.09, 0.10),
                offset: point(px(0.0), px(12.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
    }
}
