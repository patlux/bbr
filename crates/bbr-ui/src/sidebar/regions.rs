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
use super::model::{
    Callbacks, FavoritesModel, FoldersModel, PlusMenuModel, SidebarModel, StripModel,
};

/// The §11.10 favorites region: the `type:badge` "Favorites" label over
/// the deterministic tile grid. The label hides with the region (the caller
/// passes `None` while empty). While a tab drag targets the grid (a drop
/// toggles the dragged tab as a favorite), the region renders the §10.9
/// drop wash so the add/remove affordance is visible.
pub fn favorites_surface(
    theme: &Theme,
    _model: &SidebarModel,
    favorites: &FavoritesModel,
    callbacks: &Callbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let children: Vec<gpui::AnyElement> = favorites
        .tiles
        .iter()
        .map(|tile| {
            components::favorite_tile(
                theme,
                tile,
                callbacks::row(&callbacks.on_activate_favorite, tile.index),
            )
            .into_any()
        })
        .collect();
    let grid = components::favorites_grid(theme, children);
    div()
        .id(components::favorites::FAVORITES_SECTION_ID)
        .flex()
        .flex_col()
        .pb(Spacing::sp1())
        // The `type:badge` label text (the section id lives on the
        // wrapper, exactly like the §15 semantic section node).
        .child(
            div()
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(p.ink_faint)
                .child("FAVORITES"),
        )
        .child(grid)
        .when(favorites.targeted, |el| {
            el.rounded(Radius.medium())
                .border_2()
                .border_color(p.accent)
        })
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
) -> Stateful<Div> {
    let p = theme.palette;
    let focused = folders.focused;
    let mut children: Vec<gpui::AnyElement> =
        vec![components::section_label(theme, "Folders", focused).into_any()];
    for row in &folders.rows {
        let id = row.id;
        children.push(
            components::folder_row(
                theme,
                row,
                callbacks::folder(&callbacks.on_toggle_folder, id),
            )
            .on_mouse_down(
                MouseButton::Left,
                callbacks::folder(&callbacks.on_focus_folder, id),
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
                .when(is_selected, |el| {
                    el.bg(p.tint_selected).border_l_2().border_color(p.accent)
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
        .border_1()
        .border_color(p.hairline)
        .py(Spacing::sp1())
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
}

/// The §11.3 permanent space strip: one chip per space in canonical grouped
/// order inside a single-line horizontal scroll viewport. The rail builds
/// the chips from the [`StripModel`] visuals and wires the production
/// activation/keyboard callbacks; the strip consumes all footer width left
/// after `[Downloads]` and before `[+]` and never wraps or shrinks the
/// fixed controls.
fn space_strip_surface(theme: &Theme, strip: &StripModel, callbacks: &Callbacks) -> Stateful<Div> {
    let chips: Vec<Stateful<Div>> = strip
        .chips
        .iter()
        .map(|visual| {
            super::strip::space_chip(
                theme,
                visual,
                strip.swipe_tint,
                callbacks::space(&callbacks.on_select_space, visual.space),
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
    let strip = space_strip_surface(theme, &model.strip, callbacks);
    components::footer_surface(
        theme,
        footer.downloads_focus.clone(),
        footer.downloads_focused,
        footer.downloads_count,
        strip,
        callbacks::mouse(&callbacks.on_downloads),
        callbacks::mouse(&callbacks.on_plus),
    )
}
