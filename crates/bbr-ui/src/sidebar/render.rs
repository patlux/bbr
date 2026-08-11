//! The G3 Sidebar rail composition (DESIGN.md §11.1–§11.4) — the
//! `bbr-ui::sidebar` rendering authority.
//!
//! [`sidebar_surface`] renders the complete §11.4 rail from the plain
//! [`SidebarModel`] and the one [`Callbacks`] set: brand block, command
//! slot (address pill / staging surface), the in-rail command results, the
//! favorites grid, the pinned block, the folders list, the bounded regular
//! tab window, the status line, the collapse control, the plus menu, the
//! downloads panel, the footer row with the permanent space strip, and the
//! resize divider. Every visual comes from the production `bbr-ui`
//! components and tokens; every listener is a callback slot the caller
//! filled. The rail holds no browser/session/profile/space/ranking/
//! persistence authority — the app assembles the model and binds the
//! callbacks, and the gallery fills recorder callbacks.
//!
//! The app-owned child elements (the §11.4 command field anchor, the §11.5
//! in-rail command results, the §11.12 downloads panel) are passed
//! separately at this composition boundary so the [`SidebarModel`] DTO
//! stays plain `Clone` + `Debug` data. The dense regular region renders
//! only the bounded window the app derived from the production scroll
//! anchor ([`TabsModel`]) plus spacers that keep the container's content
//! height equal to the full collection — a 500-tab list never lays out 500
//! rows per frame (§11.6).

use gpui::{div, prelude::*, px, AnyElement, Div, MouseButton, Stateful, Styled};

use crate::components;
use crate::theme::{Spacing, Theme};

use super::callbacks;
use super::geometry::sidebar_divider;
use super::model::{Callbacks, CommandSlot, SidebarModel};
use super::regions::{favorites_surface, footer_surface, plus_menu_surface};
use super::tabs::tab_surface;

/// The §11.4 rail: the full sidebar column (brand, command slot, results,
/// favorites, tabs, status, toggle, plus menu, downloads panel, footer with
/// the permanent space strip) plus the §5 4 pt resize divider on the right
/// edge. The rail is a physical fixed-width surface (`flex_shrink_0`): the
/// content slot re-flows instantly on collapse and the native web view
/// always keeps strictly positive geometry (§5/§13).
///
/// `command_field_anchor` is the app-owned invisible §11.4 field anchor
/// overlaying the command slot (the rect recorder for the field-anchored
/// overlay); `command_results` is the app-owned §11.5 in-rail ranking
/// surface; `downloads_panel` is the app-owned §11.12 anchored downloads
/// surface. All three are app-owned elements passed at the composition
/// boundary — the model itself stays plain data.
pub fn sidebar_surface(
    theme: &Theme,
    model: &SidebarModel,
    callbacks: &Callbacks,
    command_field_anchor: AnyElement,
    command_results: Option<AnyElement>,
    downloads_panel: Option<AnyElement>,
) -> Stateful<Div> {
    let p = theme.palette;
    let command_bar = command_slot(theme, model, callbacks, command_field_anchor);
    let favorites = model
        .favorites
        .as_ref()
        .map(|favorites| favorites_surface(theme, model, favorites, callbacks));
    let plus_menu = model
        .plus_menu
        .as_ref()
        .map(|plus_menu| plus_menu_surface(theme, model, plus_menu, callbacks));
    let tabs = tab_surface(theme, model, callbacks);
    let footer = footer_surface(theme, model, callbacks);

    div()
        .id("bbr-sidebar")
        .w(px(model.sidebar_width))
        .h_full()
        // The rail is a physical fixed-width surface: GPUI's default
        // flex_shrink is 1, so without this the rail would absorb part
        // of the slot's relative(1.0) width and render narrower than its
        // effective geometry — breaking the §13 content boundary and the
        // §5 clamp (the content floor must stay exact).
        .flex_shrink_0()
        .flex()
        .flex_row()
        .bg(model.chrome_tint)
        .border_r_1()
        .border_color(p.hairline)
        .on_scroll_wheel(callbacks::wheel(&callbacks.on_rail_scroll_wheel))
        .on_mouse_down(
            MouseButton::Left,
            callbacks::mouse(&callbacks.on_rail_mouse_down),
        )
        .child(
            div()
                .id("bbr-sidebar-column")
                .flex_1()
                .flex()
                .flex_col()
                .min_w_0()
                // Strict height bound: the column must never grow to its
                // content, or the dense scroll container below would be
                // laid out at the full collection height (a 500-tab list
                // would hand the container ~16 000 pt) and the container's
                // prepaint would clamp the production scroll offset to 0.
                .min_h_0()
                .child(brand_block(theme, model))
                .child(command_bar)
                .when(command_results.is_some(), |column| {
                    column.child(command_results.expect("checked above"))
                })
                // §11.4: regions stack separated by `density:section-gap`
                // breathing (S `sp:3` / N `sp:4` / XL `sp:5`).
                .child(div().h(theme.density.section_gap()).flex_shrink_0())
                // §11.4 region order: favorites →
                // tabs → status → toggle → plus menu → footer (the
                // §11.3 space strip is the footer's flex region, always
                // part of the rail).
                .when(favorites.is_some(), |column| {
                    column.child(favorites.expect("checked above"))
                })
                .child(div().h(theme.density.section_gap()).flex_shrink_0())
                .child(tabs)
                .child(div().h(theme.density.section_gap()).flex_shrink_0())
                .child(components::status_line(theme, &model.status))
                .child(toggle_surface(theme, model, callbacks))
                // §11.12: the anchored downloads panel is the footer's
                // direct predecessor in the column (its bottom edge is
                // flush with the footer control's top edge); the plus menu
                // renders directly above the footer row. The two are
                // mutually exclusive (open closes the other), so they
                // never stack.
                .when(downloads_panel.is_some(), |column| {
                    column.child(downloads_panel.expect("checked above"))
                })
                .when(plus_menu.is_some(), |column| {
                    column.child(plus_menu.expect("checked above"))
                })
                .child(footer),
        )
        .child(sidebar_divider(
            theme,
            model.divider_handle.clone(),
            model.divider_focused,
            callbacks::mouse(&callbacks.on_divider_mouse_down),
            callbacks::key(&callbacks.on_divider_key_down),
        ))
}

/// The §11.4 brand block: the app mark + name at `sp:4` padding.
fn brand_block(theme: &Theme, model: &SidebarModel) -> Stateful<Div> {
    div()
        .id("bbr-brand")
        .px(Spacing::sp4())
        .py(Spacing::sp3())
        .text_size(theme.type_.title.size)
        .font_weight(theme.type_.title.weight)
        .text_color(theme.palette.ink)
        .child(model.brand.clone())
}

/// The §11.5 command slot: the address pill while no command surface is
/// open and no non-address staging mode is active; the `bbr-command-bar`
/// staging input during New Space / Move Tab / the empty New Tab surface.
/// The invisible §11.4 field anchor overlays the slot to record its
/// window-relative rect (the app-owned rect recorder).
fn command_slot(
    theme: &Theme,
    model: &SidebarModel,
    callbacks: &Callbacks,
    command_field_anchor: AnyElement,
) -> Stateful<Div> {
    match &model.command {
        CommandSlot::Address(address) => components::address_control(
            theme,
            address,
            command_field_anchor,
            callbacks::boolean(&callbacks.on_command_hover),
            callbacks::mouse(&callbacks.on_command_open),
        ),
        CommandSlot::Staging(staging) => components::staging_surface(
            theme,
            staging,
            command_field_anchor,
            callbacks::mouse(&callbacks.on_command_open),
        ),
    }
}

/// The §11.4 collapse control: the `Cmd-B` pointer entry. The label is the
/// model's toggle text; the id is the stable §15 `bbr-sidebar-toggle`.
fn toggle_surface(theme: &Theme, model: &SidebarModel, callbacks: &Callbacks) -> Stateful<Div> {
    div()
        .id("bbr-sidebar-toggle")
        .px(Spacing::sp4())
        .py(Spacing::sp2())
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            callbacks::mouse(&callbacks.on_toggle_sidebar),
        )
        .child(model.toggle_label.clone())
}
