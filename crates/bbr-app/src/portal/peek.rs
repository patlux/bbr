//! The transient Peek preview bar rendered inside the one native portal
//! window (Link Preview / Peek packet).
//!
//! The browser app/controller remains the sole preview authority; the portal
//! root reads one cloned [`crate::app::peek::PeekSurface`] per render and
//! builds GPUI chrome. AppKit continues to own only the native child-window
//! boundary and the preview WKWebView z-order (the preview view is raised
//! above its warm origin by the controller's `raise_web_view` operation).
//!
//! The bar is the §11.4 command-surface family: `material:overlay`,
//! `radius:lg`, `elev:2`, density-row tall, anchored to the top of the web
//! content area. Controls (back/forward/reload/Open as Tab/close) route
//! through the production app methods via the portal root's deferred app
//! update — exactly like the find bar. Every control is keyboard-operable
//! through the app's shortcut surface (Back/Forward/Reload route to the
//! preview while it owns focus; Escape and the close control dismiss; the
//! Open as Tab action has a keyboard path through the app's action set).
//!
//! Loading, failure, and crash states are surfaced honestly in the bar: a
//! crashed preview cannot promote (the controller refuses a crashed session
//! adoption), and the failure state reports the navigation failure without
//! texture-composing page content into GPUI.

use gpui::{div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, Stateful};

use super::geometry::Rect;
use super::probe::PortalRoot;
use crate::app::peek::{
    PeekSurface, PEEK_BACK_ID, PEEK_BAR_ID, PEEK_CLOSE_ID, PEEK_FORWARD_ID, PEEK_OPEN_AS_TAB_ID,
    PEEK_RELOAD_ID, PEEK_STATE_ID, PEEK_URL_ID,
};
use bbr_ui::icons::Icon;
use bbr_ui::theme::elevation::elev2;
use bbr_ui::theme::{Spacing, Theme};

/// Build the Peek bar at the resolved portal rect. The caller owns the
/// full-window backdrop and pointer propagation policy.
pub fn peek_bar(
    surface: &PeekSurface,
    theme: &Theme,
    rect: Rect,
    cx: &mut Context<PortalRoot>,
) -> Stateful<Div> {
    let theme = *theme;
    let p = theme.palette;
    // The bar must stay readable in compact windows: title/url truncate and
    // the controls keep their minimum sizes.
    let label = surface
        .title
        .clone()
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| surface.url.clone());
    let (state_text, state_icon) = if surface.crashed {
        ("Crashed", Some(Icon::Lifering))
    } else if surface.loading {
        ("Loading", Some(Icon::LoadingArc))
    } else {
        ("Peek", None)
    };
    div()
        .id(PEEK_BAR_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(&theme))
        .px(Spacing::sp2())
        .gap(Spacing::sp1())
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .id(PEEK_STATE_ID)
                .flex()
                .flex_row()
                .items_center()
                .gap(Spacing::sp1())
                .child(match state_icon {
                    Some(icon) => div().child(icon.canvas(&theme, px(12.0))),
                    None => div().h(px(12.0)),
                })
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(if surface.crashed {
                            p.status_error
                        } else {
                            p.ink_secondary
                        })
                        .child(state_text),
                ),
        )
        .child(
            div()
                .id(PEEK_URL_ID)
                .flex_1()
                .min_w(px(80.0))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink)
                .child(label),
        )
        .child(
            peek_control(&theme, PEEK_BACK_ID, "Back", surface.can_go_back).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.peek_back_clicked(cx);
                }),
            ),
        )
        .child(
            peek_control(&theme, PEEK_FORWARD_ID, "Forward", surface.can_go_forward).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.peek_forward_clicked(cx);
                }),
            ),
        )
        .child(
            peek_control(&theme, PEEK_RELOAD_ID, "Reload", true).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.peek_reload_clicked(cx);
                }),
            ),
        )
        .child(
            peek_control(&theme, PEEK_OPEN_AS_TAB_ID, "Open as Tab", !surface.crashed)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                        this.peek_promote_clicked(cx);
                    }),
                ),
        )
        .child(
            peek_control(&theme, PEEK_CLOSE_ID, "Close", true).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.peek_close_clicked(cx);
                }),
            ),
        )
}

/// One Peek bar control: a density-row chip with the §9 ring on hover and
/// the muted/disabled variant when unavailable.
fn peek_control(
    theme: &Theme,
    id: &'static str,
    label: &'static str,
    enabled: bool,
) -> Stateful<Div> {
    let p = theme.palette;
    let text_color = if enabled {
        p.ink_secondary
    } else {
        p.ink_faint
    };
    div()
        .id(id)
        .h(theme.density.row())
        .px(Spacing::sp1())
        .rounded(theme.radius.medium())
        .border_2()
        .border_color(if enabled {
            p.hairline
        } else {
            p.surface_overlay
        })
        .text_size(theme.type_.meta.size)
        .text_color(text_color)
        .flex()
        .flex_row()
        .items_center()
        .child(label)
}
