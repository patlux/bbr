//! The G3 Sidebar U2 regions (DESIGN.md §11.4/§11.10/§11.11/§11.12): the
//! favorites grid, the folders list with nested rows, the plus menu, the
//! permanent §11.3 space strip, and the footer row. The rail composition
//! (`super::render`) places them per §11.4; every listener is a callback
//! slot the caller filled. All ids come from the `components` contract
//! modules; the app keeps the behavior authority.

use gpui::{div, prelude::*, Div, MouseButton, Stateful, Styled};

use crate::components;
use crate::theme::{Radius, Spacing, Theme};

use super::callbacks;
use super::model::{Callbacks, FavoritesModel, FoldersModel, PlusMenuModel, SidebarModel};

/// The §11.10 favorites region: the zero-favorites drop band, or the
/// four-column tile grid with its trailing cell — the glyph-only rest slot,
/// or, while a tab drag is active, the drop cell (`bbr-favorites-drop-cell`)
/// that becomes the live §10.9 target when the pointer is over the region.
///
/// `band_anchor` is the app-owned invisible rect recorder that overlays
/// the region (absolute inset-0 within the `relative` wrapper): it
/// publishes the region's window-relative bounds every frame, so the app's
/// tab-drag resolver hits the real rendered band and the E2E snapshot
/// exposes its typed geometry — never a parallel layout model.
pub fn favorites_surface(
    theme: &Theme,
    model: &SidebarModel,
    favorites: &FavoritesModel,
    callbacks: &Callbacks,
    band_anchor: gpui::AnyElement,
) -> Stateful<Div> {
    let tile_width = components::favorites::favorite_tile_width(theme, model.sidebar_width);
    let children: Vec<gpui::AnyElement> = favorites
        .tiles
        .iter()
        .map(|tile| {
            components::favorite_tile(
                theme,
                tile,
                tile_width,
                favorites
                    .pop
                    .filter(|(tab, _)| *tab == tile.tab)
                    .map(|(_, key)| key),
                callbacks::row(&callbacks.on_activate_favorite, tile.index),
            )
            // §11.13: a secondary-click on the tile opens the Tab/Favorite
            // context menu bound to that tile's exact `TabId` (the app
            // resolves the favorite at open; opening never activates).
            .on_mouse_down(
                MouseButton::Right,
                callbacks::row(&callbacks.on_context_menu_favorite, tile.index),
            )
            .into_any()
        })
        .collect();
    // §11.10: zero favorites render the full-width drop band (region name
    // and gesture); otherwise the grid, plus the glyph-only slot in the next
    // free cell while the last row is not full.
    let body: gpui::AnyElement = if favorites.tiles.is_empty() {
        components::favorites::favorites_empty_band(theme, favorites.targeted).into_any()
    } else {
        let mut children = children;
        if favorites.offer {
            // §11.10: during a tab drag the grid offers one additional cell
            // at the end of the wrap order (it replaces the rest slot).
            children.push(
                components::favorites::favorites_drop_cell(theme, tile_width, favorites.targeted)
                    .into_any(),
            );
        } else if components::favorites::empty_slot_shown(favorites.tiles.len()) {
            children
                .push(components::favorites::favorites_empty_slot(theme, tile_width).into_any());
        }
        components::favorites_grid(theme, children).into_any()
    };
    div()
        .id(components::favorites::FAVORITES_SECTION_ID)
        .relative()
        .flex()
        .flex_col()
        .pt(Spacing::sp1())
        .pb(Spacing::sp2())
        // The reference presents Favorite tiles as an immediately
        // recognizable destination band without a redundant section label.
        // The stable section id remains on the wrapper for semantics/E2E.
        // §11.10: the region is never outlined as a whole — the drop cell
        // (or the zero-favorites band) names the exact slot, and an outline
        // would reflow the region under the pointer mid-drag.
        .child(body)
        .child(band_anchor)
}

/// The §11.11 folders region: the `type:badge` "Folders" label over the
/// `bbr-folders` list — folder rows `[disclosure] [name, trunc] [count]`
/// plus the nested §11.1 rows of expanded folders. The region owns
/// keyboard focus: `↑`/`↓` move the focused row and Enter/Space toggle it
/// (§11.11). Placed by the rail inside the tab region between the Pinned
/// block and the Tabs section.
pub fn folders_surface(
    theme: &Theme,
    model: &SidebarModel,
    folders: &FoldersModel,
    callbacks: &Callbacks,
    mut anchors: Vec<(bbr_domain::FolderId, gpui::AnyElement)>,
) -> Stateful<Div> {
    let p = theme.palette;
    let focused = folders.focused;
    // The active Space heading already owns the list hierarchy. Keep the
    // stable folder wrappers/semantics, but do not repeat a generic visible
    // FOLDERS label above the user's real folder names.
    let mut children: Vec<gpui::AnyElement> = Vec::new();
    for row in &folders.rows {
        let id = row.id;
        // Blueprint D02: the app's rect recorder for this row's drop rect.
        let anchor = anchors
            .iter()
            .position(|(folder, _)| *folder == id)
            .map(|at| anchors.swap_remove(at).1);
        children.push(
            components::folder_row(
                theme,
                row,
                callbacks::folder_hover(&callbacks.on_hover_folder, id),
                callbacks::folder(&callbacks.on_toggle_folder, id),
            )
            .relative()
            .children(anchor)
            .on_mouse_down(
                MouseButton::Left,
                callbacks::folder(&callbacks.on_focus_folder, id),
            )
            // Blueprint D25: a secondary-click opens the folder menu.
            .on_mouse_down(
                MouseButton::Right,
                callbacks::folder(&callbacks.on_context_menu_folder, id),
            )
            .into_any(),
        );
        for member in folders
            .members
            .iter()
            .filter(|member| member.folder == row.id)
        {
            let folder = member.folder;
            let member_index = member.member;
            children.push(
                components::folder_item_row(
                    theme,
                    member,
                    callbacks::member(&callbacks.on_select_folder_member, folder, member_index),
                    callbacks::member(&callbacks.on_close_folder_member, folder, member_index),
                )
                .into_any(),
            );
        }
    }
    div()
        .id(components::folders::FOLDERS_ID)
        .track_focus(&model.folders_handle)
        .flex()
        .flex_col()
        .min_w_0()
        // §9 ring on the focused folders region (the same structural
        // cue the space/tab lists use).
        .when(focused, |el| {
            el.border_2()
                .border_color(p.accent)
                .rounded(Radius.medium())
        })
        .on_key_down(callbacks::key(&callbacks.on_folders_key_down))
        .children(children)
}

/// The §11.4 plus menu surface (`bbr-plus-menu`, `elev:2`,
/// `material:overlay`, `radius:md`), rendered directly above the footer
/// row while open. The menu owns keyboard focus (`↑`/`↓`/Enter/Escape)
/// and marks pointer presses inside so the root outside-press handler
/// never dismisses it (§13 one pointer action).
pub fn plus_menu_surface(
    theme: &Theme,
    model: &SidebarModel,
    plus_menu: &PlusMenuModel,
    callbacks: &Callbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = plus_menu.selected;
    let items: Vec<gpui::AnyElement> = components::menu::plus_menu_items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = index == selected;
            div()
                .id(item.id)
                .px(Spacing::sp3())
                .py(Spacing::sp2())
                .gap(Spacing::sp2())
                .items_center()
                .flex()
                .flex_row()
                .cursor_pointer()
                // §18.4: the shared cursor is the inset `tint.accent` surface.
                .when(is_selected, |el| {
                    crate::surfaces::floating::menu_cursor(el, theme)
                })
                .child(
                    div()
                        .size(theme.density.icon())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(item.glyph.canvas(theme, theme.density.icon())),
                )
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .font_weight(theme.type_.label.weight)
                        .text_color(if is_selected { p.ink } else { p.ink_secondary })
                        .child(item.label),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    callbacks::row(&callbacks.on_plus_menu_commit, index),
                )
                .into_any()
        })
        .collect();
    div()
        .id(components::menu::PLUS_MENU_ID)
        .track_focus(&model.plus_menu_handle)
        .mx(Spacing::sp2())
        .mb(Spacing::sp1())
        .rounded(theme.radius.medium())
        .bg(p.surface_overlay)
        // §18.4: 1 pt edge, 4 pt padding and the `elev:2` v2 shadow of the
        // menu family.
        .border_1()
        .border_color(crate::surfaces::floating::edge(theme, false))
        .shadow(crate::theme::elevation::elev2(theme))
        .p(crate::surfaces::floating::menu_padding())
        .on_key_down(callbacks::key(&callbacks.on_plus_menu_key_down))
        // The inside-press marker runs deepest in the bubble phase;
        // stopping propagation keeps the sidebar chrome-focus handler
        // and the root dismiss handler from fighting a press on the
        // open menu (focus stays on the menu surface, §13 one
        // pointer action — a press inside the menu never dismisses
        // it). Outside presses never reach this surface and close
        // the menu via the root handler.
        .on_mouse_down(
            MouseButton::Left,
            callbacks::mouse(&callbacks.on_plus_menu_inside_press),
        )
        .children(items)
        .children(crate::rect_probe::rect_probe(
            components::menu::PLUS_MENU_ID,
        ))
}

/// The §11.3 permanent space strip: one chip per space in canonical grouped
/// order inside a single-line horizontal scroll viewport. The rail builds
/// the chips from the [`StripModel`] visuals and wires the production
/// activation/keyboard callbacks; the strip consumes all footer width left
/// after `[Downloads]` and before `[+]` and never wraps or shrinks the
/// fixed controls.
fn space_strip_surface(
    theme: &Theme,
    model: &SidebarModel,
    callbacks: &Callbacks,
) -> Stateful<Div> {
    let strip = &model.strip;
    let chips: Vec<Stateful<Div>> = strip
        .chips
        .iter()
        .map(|visual| {
            let target = components::footer::FooterTarget::Space(visual.space);
            let hover = callbacks.on_footer_hover.clone();
            super::strip::space_chip(
                theme,
                visual,
                strip.swipe_tint,
                super::strip::ChipState {
                    hovered: model.footer.hovered == Some(target),
                    focused: strip.focused && visual.selected,
                    rail: model.chrome_tint,
                },
                callbacks::space(&callbacks.on_select_space, visual.space),
            )
            .on_hover(move |hovered: &bool, window, cx| hover(target, *hovered, window, cx))
            // Blueprint D05 (§11.13): a secondary-click opens the space menu
            // bound to this exact space; opening never activates it.
            .on_mouse_down(
                MouseButton::Right,
                callbacks::space(&callbacks.on_context_menu_space, visual.space),
            )
        })
        .collect();
    super::strip::space_strip(theme, strip.scroll.clone(), chips)
        .track_focus(&strip.focus)
        .on_key_down(callbacks::key(&callbacks.on_space_strip_key_down))
}

/// The §11.4 footer row: exactly `[Downloads] [permanent space strip] [+]`
/// through the production `bbr-ui` `footer_surface`, separated from the
/// list above by a hairline. The `[Downloads]` control carries the §11.4
/// `type:meta` `tnum` count while any active attempt exists; the strip is
/// the rail-built permanent §11.3 surface; `[+]` opens the plus menu. There
/// is no `[Spaces]` toggle and no revealable space-row region.
pub fn footer_surface(theme: &Theme, model: &SidebarModel, callbacks: &Callbacks) -> Stateful<Div> {
    let footer = &model.footer;
    let strip = space_strip_surface(theme, model, callbacks);
    components::footer_surface(
        theme,
        &components::footer::FooterControls {
            downloads_focus: footer.downloads_focus.clone(),
            downloads_focused: footer.downloads_focused,
            downloads_open: footer.downloads_open,
            downloads_count: footer.downloads_count,
            plus_open: model.plus_menu.is_some(),
            hovered: footer.hovered,
            rail: model.chrome_tint,
        },
        strip,
        callbacks::mouse(&callbacks.on_downloads),
        callbacks::mouse(&callbacks.on_plus),
        callbacks.on_footer_hover.clone(),
    )
}
