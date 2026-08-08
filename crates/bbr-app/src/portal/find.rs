//! The find-in-page bar rendered inside the one native portal window (task
//! packet `find-in-page-portal-bar`).
//!
//! The browser app remains the sole find-state authority; the portal root
//! reads one cloned [`crate::app::find::FindSurface`] per render and builds
//! GPUI chrome. AppKit continues to own only the native child-window
//! boundary. The bar is the §11.4 command-surface family: `material:overlay`,
//! `radius:lg`, `elev:2`, `density:row` tall, anchored top-right of the
//! content area.
//!
//! Controls: previous/next/close buttons route through the production app
//! methods (`find_previous`/`find_next`/`find_close`) via the portal root's
//! deferred app update — exactly like the command surfaces. The bar card
//! stops propagation; the backdrop owns the §13 one-pointer-action
//! dismissal.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful,
};

use super::geometry::Rect;
use super::probe::PortalRoot;
use crate::app::find::{
    FindSurface, FIND_BAR_ID, FIND_CLOSE_ID, FIND_COUNT_ID, FIND_FIELD_ID, FIND_NEXT_ID,
    FIND_PREVIOUS_ID, FIND_TEXT_ID,
};
use crate::components::Icon;
use crate::theme::Spacing;

/// Build the find bar at the resolved portal rect. The caller owns the
/// full-window backdrop and pointer propagation policy.
pub fn find_bar(model: &FindSurface, rect: Rect, cx: &mut Context<PortalRoot>) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    // §11.5 count copy: "n of m"; zero matches reads "0 of 0".
    let count = match (model.current_match, model.match_count) {
        (Some(ordinal), Some(total)) => format!("{ordinal} of {total}"),
        (None, Some(0)) | (None, Some(_)) => "0 of 0".to_string(),
        (_, None) => "".to_string(),
    };
    div()
        .id(FIND_BAR_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2_shadows(theme))
        .px(Spacing::sp2())
        .gap(Spacing::sp1())
        .flex()
        .flex_row()
        .items_center()
        .child(field(model))
        .child(
            div()
                .id(FIND_COUNT_ID)
                .min_w(px(64.0))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(count),
        )
        .child(
            control_button(FIND_PREVIOUS_ID, Icon::ChevronLeft, model).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.find_previous_clicked(cx);
                }),
            ),
        )
        .child(
            control_button(FIND_NEXT_ID, Icon::ChevronRight, model).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.find_next_clicked(cx);
                }),
            ),
        )
        .child(
            control_button(FIND_CLOSE_ID, Icon::Close, model).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.find_close_clicked(cx);
                }),
            ),
        )
}

/// The query field: the live text plus a placeholder when empty.
fn field(model: &FindSurface) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let text = if model.query.is_empty() {
        SharedString::from("Find in page")
    } else {
        SharedString::from(model.query.clone())
    };
    div()
        .id(FIND_FIELD_ID)
        .flex_1()
        .h(theme.density.row())
        .px(Spacing::sp2())
        .rounded(theme.radius.medium())
        // The find field owns focus while the presentation is open; the
        // §9 ring marks it without color-only feedback.
        .border_2()
        .border_color(if model.focused { p.accent } else { p.hairline })
        .bg(p.surface_field)
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .id(FIND_TEXT_ID)
                .flex_1()
                .text_size(theme.type_.mono.size)
                .text_color(if model.query.is_empty() {
                    p.ink_faint
                } else {
                    p.ink
                })
                .child(text),
        )
}

/// One bar control (previous / next / close), rendered with the §7 drawn
/// catalog glyph (`Icon::ChevronLeft` / `Icon::ChevronRight` / `Icon::Close`)
/// at the `icon:<density>` size. The close control uses the §8
/// `density:hit-min.close` hit square; the step controls use
/// `density:hit-min`. The owning div keeps the id, hit square, cursor, and
/// listeners — the glyph is pure presentation (§14: no Unicode substitutes
/// in chrome).
fn control_button(id: &'static str, icon: Icon, model: &FindSurface) -> Stateful<Div> {
    let theme = &model.theme;
    let is_close = id == FIND_CLOSE_ID;
    let hit = if is_close {
        theme.density.hit_min_close()
    } else {
        theme.density.hit_min()
    };
    div()
        .id(id)
        .w(hit)
        .h(hit)
        .rounded(theme.radius.medium())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .child(icon.canvas(theme, theme.density.icon()))
}

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
