//! The G3 Sidebar rail composition (DESIGN.md §11.1–§11.4) — the
//! `bbr-ui::sidebar` rendering authority.
//!
//! [`sidebar_surface`] renders the complete §11.4 rail from the plain
//! [`SidebarModel`] and the one [`Callbacks`] set: unified titlebar,
//! command slot (address pill / staging surface), the in-rail command results,
//! the active Space heading, the
//! favorites grid, the pinned block, the folders list, the bounded regular
//! tab window, the status line, the collapse control, the plus menu, the
//! downloads panel, the footer row with the permanent space strip, and the
//! resize divider. Every visual comes from the production `bbr-ui`
//! components and tokens; every listener is a callback slot the caller
//! filled. The rail holds no browser/session/profile/space/ranking/
//! persistence authority — the app assembles the model and binds the
//! callbacks, and the gallery fills recorder callbacks.
//!
//! The app-owned child elements (field and Favorites-band anchors, command
//! results, downloads, and Space configuration) are grouped in
//! [`SidebarChildren`] at this composition boundary so the [`SidebarModel`]
//! DTO stays plain `Clone` + `Debug` data. The dense regular region renders
//! only the bounded window the app derived from the production scroll
//! anchor ([`TabsModel`]) plus spacers that keep the container's content
//! height equal to the full collection — a 500-tab list never lays out 500
//! rows per frame (§11.6).

use gpui::{div, prelude::*, px, AnyElement, Div, MouseButton, Stateful, Styled};

use crate::components;
use crate::theme::{Spacing, Theme};
use crate::titlebar::titlebar_surface;

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
/// overlay); `favorites_band_anchor` is the app-owned invisible §11.10
/// favorites-band anchor overlaying the favorites region (the rect
/// recorder the tab-drag resolver targets the real rendered drop band
/// with); `command_results` is the app-owned §11.5 in-rail ranking
/// surface; `space_config_panel` is the app-owned §11.3 rename/identity
/// configuration surface; `pin_offer_anchor` and `drag_proxy_anchor` are
/// the app-owned rect recorders inside the §11.2 ghost row and the §10.9
/// drag proxy (only used while those render); `folder_anchors` record the
/// folder rows' rects. All are app-owned elements
/// grouped at the composition boundary — the model itself stays plain data.
pub struct SidebarChildren {
    pub command_field_anchor: AnyElement,
    pub favorites_band_anchor: AnyElement,
    pub pin_offer_anchor: Option<AnyElement>,
    pub drag_proxy_anchor: Option<AnyElement>,
    /// Blueprint D02: one app-owned rect recorder per rendered folder row,
    /// keyed by the folder's domain id (the rows' drop rects).
    pub folder_anchors: Vec<(bbr_domain::FolderId, AnyElement)>,
    pub command_results: Option<AnyElement>,
    pub downloads_panel: Option<AnyElement>,
    pub space_config_panel: Option<AnyElement>,
}

pub fn sidebar_surface(
    theme: &Theme,
    model: &SidebarModel,
    callbacks: &Callbacks,
    children: SidebarChildren,
) -> Stateful<Div> {
    let SidebarChildren {
        command_field_anchor,
        favorites_band_anchor,
        pin_offer_anchor,
        drag_proxy_anchor,
        folder_anchors,
        command_results,
        downloads_panel,
        space_config_panel,
    } = children;
    let command_bar = command_slot(theme, model, callbacks, command_field_anchor);
    let favorites = model.favorites.as_ref().map(|favorites| {
        favorites_surface(theme, model, favorites, callbacks, favorites_band_anchor)
    });
    let plus_menu = model.plus_menu.as_ref().map(|plus_menu| {
        // V-6 (blueprint D08): the menu fades in over dur:fine.
        crate::surfaces::floating::appear(
            theme,
            "bbr-plus-menu-appear",
            plus_menu_surface(theme, model, plus_menu, callbacks),
        )
    });
    let tabs = tab_surface(theme, model, callbacks, pin_offer_anchor, folder_anchors);
    // §10.9: the lifted row follows the pointer above the whole rail (the
    // rail's origin is the window origin, so the proxy rect is in window
    // coordinates).
    let drag_proxy = model.drag_proxy.as_ref().map(|proxy| {
        let element = components::drag_offer::tab_drag_proxy(theme, &proxy.row, proxy.rect);
        match drag_proxy_anchor {
            Some(anchor) => element.child(anchor),
            None => element,
        }
    });
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
        .relative()
        .flex()
        .flex_row()
        // No right hairline (DESIGN §2 rule 4): the seated content card's
        // edge and shadow are the boundary; painting both is a double
        // border.
        // Blueprint D02: the floating panel paints the material itself (the
        // notch around the traffic lights stays clear).
        .when(!model.floating, |el| el.bg(model.chrome_tint))
        // D4: the source tint fades out above the committed paint and
        // beneath every child.
        .when_some(
            crate::components::tint_fade::tint_fade_layer(theme, "rail", model.tint_fade)
                .filter(|_| !model.floating),
            |el, layer| el.child(layer),
        )
        // D6: the static grain sits on the tint, beneath every child.
        .when(model.grain && !model.floating, |el| {
            el.child(crate::components::grain::grain_layer(theme))
        })
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
                // §11.14: the titlebar row is the rail's top strip — the
                // safe-area geometry, the sidebar toggle (the atomic
                // cutover from the status-row control), and the
                // Back/Forward/Reload cluster. It disappears with the rail
                // on collapse; no content toolbar replaces it.
                .child(titlebar_surface(
                    theme,
                    &model.titlebar,
                    &callbacks.titlebar,
                ))
                .child(command_bar)
                .when(command_results.is_some(), |column| {
                    column.child(command_results.expect("checked above"))
                })
                // Reference-authoritative hierarchy: Favorite destinations
                // sit directly below the address, then the active Space name
                // introduces the tab/folder list. There is no standalone app
                // brand block or technical FAVORITES/TABS chrome at rest.
                .when(favorites.is_some(), |column| {
                    column.child(favorites.expect("checked above"))
                })
                .child(active_space_heading(theme, model))
                .child(tabs)
                .when(model.status_visible, |column| {
                    column.child(components::status_line(theme, &model.status))
                })
                // §11.12: the anchored downloads panel is the footer's
                // direct predecessor in the column (its bottom edge is
                // flush with the footer control's top edge); the §11.3
                // space-configuration surface is the same in-rail panel
                // family directly above the footer row, and the plus menu
                // renders directly above the footer row. At most one of
                // the three is open at a time (the app closes the others
                // when one opens), so they never stack.
                .when(downloads_panel.is_some(), |column| {
                    column.child(downloads_panel.expect("checked above"))
                })
                .when(space_config_panel.is_some(), |column| {
                    column.child(space_config_panel.expect("checked above"))
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
        .when_some(drag_proxy, |rail, proxy| rail.child(proxy))
        // Blueprint D07: the destination-profile label above the footer.
        .when_some(model.drag_destination.as_ref(), |rail, destination| {
            rail.child(components::drag_offer::drag_destination_label(
                theme,
                destination,
            ))
        })
}

/// The active Space title — the primary list heading in the reference
/// composition. It comes from the controller snapshot and remains visible
/// even when the Space has no tabs, so the user always knows the current
/// browsing context without a separate BBR brand block.
fn active_space_heading(theme: &Theme, model: &SidebarModel) -> Stateful<Div> {
    // The identity is the active footer chip's own validated value, so the
    // heading and the chip can never show two different marks.
    let identity = model
        .strip
        .chips
        .iter()
        .find(|chip| chip.selected)
        .map(|chip| chip.identity.clone());
    div()
        .id("bbr-active-space-title")
        .flex()
        .flex_row()
        .items_center()
        // The mark sits on the tab rows' favicon column, so the title
        // starts on the same x as every tab title below it: the same
        // leading padding and gap as the §11.1 row content (the row's
        // `row_inset` belongs to its wash surface, not its content).
        .gap(Spacing::sp2())
        .px(theme.density.row_padding_x())
        .pt(Spacing::sp4())
        .pb(Spacing::sp2())
        // §18.3: the heading is `type:label` size (N 13) at 600 in
        // `ink.secondary` — quieter than the 14 pt rows it heads. (The
        // +0.01 em tracking is not expressible in GPUI text styles.)
        .text_size(heading_size(theme))
        .font_weight(HEADING_WEIGHT)
        .text_color(theme.palette.ink_secondary)
        .when_some(identity, |heading, identity| {
            heading.child(
                div()
                    .id(ACTIVE_SPACE_IDENTITY_ID)
                    .flex_shrink_0()
                    .w(theme.density.icon())
                    .flex()
                    .justify_center()
                    .child(super::strip::identity_mark(
                        theme,
                        &identity,
                        gpui::FontWeight(600.0),
                    )),
            )
        })
        .child(
            div()
                .debug_selector(|| ACTIVE_SPACE_NAME_SELECTOR.to_string())
                .child(model.active_space_title.clone()),
        )
}

/// §18.3 active-space heading size: the density's `type:label` step
/// (S 12 / N 13 / XL 14).
pub fn heading_size(theme: &Theme) -> gpui::Pixels {
    theme.type_.label.size
}

/// §18.3 active-space heading weight.
pub const HEADING_WEIGHT: gpui::FontWeight = gpui::FontWeight(600.0);

/// Test-support selector of the heading's title text (geometry evidence:
/// it starts on the tab titles' x).
pub const ACTIVE_SPACE_NAME_SELECTOR: &str = "bbr-active-space-name";

/// The stable id of the active-space heading's identity mark (§11.4).
pub const ACTIVE_SPACE_IDENTITY_ID: &str = "bbr-active-space-identity";

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
            callbacks::mouse(&callbacks.on_address_zoom_reset),
            callbacks::mouse(&callbacks.on_address_site),
        ),
        CommandSlot::Staging(staging) => components::staging_surface(
            theme,
            staging,
            command_field_anchor,
            callbacks::mouse(&callbacks.on_command_open),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, ThemeMode};

    #[test]
    fn the_active_space_heading_is_label_size_at_600() {
        // §18.3: 13 pt / 600 at normal density, one step under the 14 pt
        // row titles it heads; S/XL follow the `type:label` step.
        let at = |density| Theme::new(ThemeMode::Light, ContrastMode::Standard, density);
        assert_eq!(heading_size(&at(Density::Normal)), gpui::px(13.0));
        assert_eq!(heading_size(&at(Density::Small)), gpui::px(12.0));
        assert_eq!(heading_size(&at(Density::ExtraLarge)), gpui::px(14.0));
        assert!(heading_size(&at(Density::Normal)) < at(Density::Normal).type_.title.size);
        assert_eq!(HEADING_WEIGHT, gpui::FontWeight(600.0));
    }
}
