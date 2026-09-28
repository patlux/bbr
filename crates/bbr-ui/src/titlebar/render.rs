//! The §11.14 titlebar row composition (DESIGN.md §11.14, packet
//! `titlebar-navigation-controls.md`) — the `bbr-ui` rendering authority
//! for the expanded full-height Sidebar's top strip.
//!
//! The row renders
//! `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]` inside
//! the actual safe-titlebar layout guide: the strip height and the
//! traffic-light clearance come from the model's [`SafeArea`] (the macOS
//! adapter produces it; nothing is hardcoded here), the sidebar toggle
//! sits immediately after the traffic lights with the identical
//! sidebar-width/collapse authority as Cmd-B, and Back/Forward/Reload form
//! the compact trailing cluster. The empty region between the toggle and
//! the cluster is the draggable titlebar space; controls never intercept
//! a drag outside their hit targets. Glyphs are the BBR geometric 18 pt
//! family with ≥32 pt hit areas, transparent rest state, and the §10.2
//! hover / §10.3 pressed / §10.8 disabled tokens — no bordered toolbar
//! capsule, no duplicate controls.
//!
//! The row holds no browser/session/navigation authority: the model
//! carries the genuine enablement and the callbacks route through the
//! existing app authorities. The rail (`bbr-ui::sidebar`) renders this
//! row as its first child, so collapsing the Sidebar removes the complete
//! BBR-owned header and navigation cluster with it.

use gpui::{div, prelude::*, px, Div, MouseButton, Stateful, Styled};

use crate::icons::Icon;
use crate::theme::{Spacing, Theme};

use super::callbacks;
use super::model::{TitlebarCallbacks, TitlebarControl, TitlebarModel, TITLEBAR_ID};

/// The §11.14 normal-density hit target: at least 32 pt per control.
pub const TITLEBAR_HIT: f32 = 32.0;
/// The §11.14 18 pt glyph size (the BBR geometric instrument family).
pub const TITLEBAR_GLYPH: f32 = 18.0;
/// The gap between the traffic-light region and the sidebar toggle.
pub const TRAFFIC_GAP: f32 = 6.0;
/// The §11.14 gap between the trailing navigation controls.
pub const NAV_GAP: f32 = 4.0;

/// One §11.14 control button: the transparent-rest hit square with the §7
/// glyph, the §10.2 hover wash, the §10.3 pressed wash, the §10.8 disabled
/// treatment, and the stable §15 id. A disabled control renders visibly
/// (the `ink.faint` glyph), registers no listener, and can never dispatch
/// by construction.
#[allow(clippy::too_many_arguments)]
fn control_button(
    theme: &Theme,
    control: TitlebarControl,
    enabled: bool,
    hovered: bool,
    pressed: bool,
    icon: Icon,
    callbacks: &TitlebarCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let glyph_color = if enabled {
        p.ink_secondary
    } else {
        p.ink_faint
    };
    let wash = if pressed {
        p.wash_active
    } else if hovered {
        p.wash_hover
    } else {
        gpui::rgba(0x00000000)
    };
    let mut button = div()
        .id(control.id())
        .size(px(TITLEBAR_HIT))
        .flex()
        .items_center()
        .justify_center()
        .rounded(theme.radius.small())
        .bg(wash)
        .child(icon.canvas_with(theme, px(TITLEBAR_GLYPH), glyph_color));
    if enabled {
        let activation = match control {
            TitlebarControl::Toggle => &callbacks.on_toggle_sidebar,
            TitlebarControl::Back => &callbacks.on_back,
            TitlebarControl::Forward => &callbacks.on_forward,
            TitlebarControl::Reload => &callbacks.on_reload,
        };
        button = button
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                callbacks::control_mouse_down(activation, control, callbacks),
            )
            .on_hover(callbacks::control_hover(callbacks, control));
        if matches!(control, TitlebarControl::Back | TitlebarControl::Forward) {
            // Blueprint D14: a secondary click lists the session history.
            let secondary = callbacks.on_control_secondary.clone();
            button = button.on_mouse_down(
                MouseButton::Right,
                move |event: &gpui::MouseDownEvent,
                      window: &mut gpui::Window,
                      cx: &mut gpui::App| { secondary(control, event, window, cx) },
            );
        }
    }
    button
}

/// The §11.14 titlebar row: the strip at the top of the expanded rail.
/// `[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]` — the
/// toggle starts at the traffic-light region's right edge (never a
/// hardcoded x offset), the spacer between the toggle and the trailing
/// cluster is the draggable empty titlebar space, and the row height is
/// the native titlebar strip height.
pub fn titlebar_surface(
    theme: &Theme,
    model: &TitlebarModel,
    callbacks: &TitlebarCallbacks,
) -> Stateful<Div> {
    let safe = &model.safe_area;
    let left = safe.control_left() + TRAFFIC_GAP;
    div()
        .id(TITLEBAR_ID)
        .h(px(safe.height.max(0.0)))
        .flex()
        .flex_row()
        .items_center()
        .pl(px(left))
        .pr(Spacing::sp2())
        .child(control_button(
            theme,
            TitlebarControl::Toggle,
            true,
            model.hovered == Some(TitlebarControl::Toggle),
            model.pressed == Some(TitlebarControl::Toggle),
            Icon::Sidebar {
                expanded: !model.sidebar_collapsed,
            },
            callbacks,
        ))
        // The draggable empty titlebar space: any press here starts the
        // native window drag (and the double-click action on a double
        // click). Controls sit inside their own hit targets and never
        // intercept the drag.
        .child(
            div()
                .id("bbr-titlebar-drag")
                .flex_1()
                .h_full()
                .on_mouse_down(
                    MouseButton::Left,
                    callbacks::mouse(&callbacks.on_titlebar_mouse_down),
                ),
        )
        // The compact trailing cluster: Back / Forward / Reload at the
        // rail's trailing edge.
        .child(control_button(
            theme,
            TitlebarControl::Back,
            model.can_go_back,
            model.hovered == Some(TitlebarControl::Back),
            model.pressed == Some(TitlebarControl::Back),
            Icon::NavBack,
            callbacks,
        ))
        .child(control_button(
            theme,
            TitlebarControl::Forward,
            model.can_go_forward,
            model.hovered == Some(TitlebarControl::Forward),
            model.pressed == Some(TitlebarControl::Forward),
            Icon::NavForward,
            callbacks,
        ))
        .child(control_button(
            theme,
            TitlebarControl::Reload,
            model.can_reload,
            model.hovered == Some(TitlebarControl::Reload),
            model.pressed == Some(TitlebarControl::Reload),
            Icon::Reload,
            callbacks,
        ))
}

/// The empty draggable titlebar spacer element (used by tests and the
/// matrix unit tests to assert the drag region id).
pub fn titlebar_drag_spacer() -> Stateful<Div> {
    div().id("bbr-titlebar-drag").flex_1().h_full()
}
