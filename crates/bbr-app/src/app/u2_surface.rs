//! U2 sidebar render surfaces (DESIGN.md §11.4/§11.10/§11.11): the plus
//! context menu, the favorites grid, and the folders list with nested rows.
//! The rail composition in `app::sidebar_view` places them per §11.4; the
//! behavior handlers live in `app::u2`. All ids come from the `components`
//! contract modules; listeners route through the production app methods.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window,
};

use crate::components;
use crate::theme::{Radius, Spacing, Theme};

use super::BbrApp;

impl BbrApp {
    /// The §11.4 plus menu surface (`bbr-plus-menu`, `elev:2`,
    /// `material:overlay`, `radius:md`), rendered directly above the footer
    /// row while open. The menu owns keyboard focus (`↑`/`↓`/Enter/Escape)
    /// and marks pointer presses inside so the root outside-press handler
    /// never dismisses it (§13 one pointer action). `None` while closed.
    pub fn plus_menu_surface(
        &self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        if !self.u2.plus_menu.open {
            return None;
        }
        let p = theme.palette;
        let selected = self.u2.plus_menu.selected_item();
        let items: Vec<gpui::AnyElement> = crate::components::plus_menu_items()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = item.id == selected.id;
                let commit = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.u2.plus_menu.selection = index;
                    cx.stop_propagation();
                    this.commit_plus_menu(window, cx);
                });
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
                            .size(px(14.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(item.glyph.canvas(theme, px(14.0))),
                    )
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .font_weight(theme.type_.label.weight)
                            .text_color(if is_selected { p.ink } else { p.ink_secondary })
                            .child(item.label),
                    )
                    .on_mouse_down(MouseButton::Left, commit)
                    .into_any()
            })
            .collect();
        Some(
            div()
                .id(components::menu::PLUS_MENU_ID)
                .track_focus(&self.u2.plus_menu_handle)
                .mx(Spacing::sp2())
                .mb(Spacing::sp1())
                .rounded(theme.radius.medium())
                .bg(p.surface_overlay)
                .border_1()
                .border_color(p.hairline)
                .py(Spacing::sp1())
                .on_key_down(cx.listener(Self::on_plus_menu_key_down))
                // The inside-press marker runs deepest in the bubble phase;
                // stopping propagation keeps the sidebar chrome-focus handler
                // and the root dismiss handler from fighting a press on the
                // open menu (focus stays on the menu surface, §13 one
                // pointer action — a press inside the menu never dismisses
                // it). Outside presses never reach this surface and close
                // the menu via the root handler.
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                        this.u2.plus_menu_press_inside = true;
                        cx.stop_propagation();
                    }),
                )
                .children(items),
        )
    }

    /// The §11.10 favorites grid surface: the `type:badge` "Favorites"
    /// label over the deterministic tile grid. `None` while the region is
    /// empty — the label hides with it (§11.4 region rule). The wrapper is
    /// the `bbr-section-favorites` section node; the grid inside carries the
    /// `bbr-favorites` id. While a tab drag targets the grid (a drop
    /// toggles the dragged tab as a favorite), the region renders the
    /// §10.9 drop wash so the add/remove affordance is visible.
    pub fn favorites_surface(
        &self,
        theme: &Theme,
        tiles: &[components::FavoriteTileSpec],
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        if tiles.is_empty() {
            return None;
        }
        let p = theme.palette;
        let targeted =
            self.tab_drag.target() == Some(crate::app::drag_target::DragTarget::Favorites);
        let children: Vec<gpui::AnyElement> = tiles
            .iter()
            .map(|tile| {
                let index = tile.index;
                let activate = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.activate_favorite(index, window, cx);
                });
                components::favorite_tile(theme, tile, activate).into_any()
            })
            .collect();
        let grid = components::favorites_grid(children);
        Some(
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
                .when(targeted, |el| {
                    el.rounded(Radius.medium())
                        .border_2()
                        .border_color(p.accent)
                }),
        )
    }

    /// The §11.11 folders surface: the `type:badge` "Folders" label over the
    /// `bbr-folders` list — folder rows `[disclosure] [name, trunc] [count]`
    /// plus the nested §11.1 rows of expanded folders. `None` while empty.
    /// The region owns keyboard focus: `↑`/`↓` move the focused row and
    /// Enter/Space toggle it (§11.11). Placed by the rail composition inside
    /// the tab region between the Pinned block and the Tabs section.
    pub fn folders_surface(
        &self,
        theme: &Theme,
        rows: &[components::FolderRowSpec],
        members: &[components::FolderMemberVisual],
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        if rows.is_empty() {
            return None;
        }
        let p = theme.palette;
        let focused = self.u2.folders_handle.is_focused(window);
        let mut children: Vec<gpui::AnyElement> =
            vec![components::section_label(theme, "Folders", focused).into_any()];
        for row in rows {
            let index = row.index;
            let focus = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                this.focus_folder_row(index, window, cx);
            });
            let toggle = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                this.focus_folder_row(index, window, cx);
                this.toggle_folder(index, cx);
            });
            children.push(
                components::folder_row(theme, row, toggle)
                    .on_mouse_down(MouseButton::Left, focus)
                    .into_any(),
            );
            for member in members.iter().filter(|member| member.folder == row.index) {
                let folder = member.folder;
                let member_index = member.member;
                let select = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.select_folder_member(folder, member_index, window, cx);
                });
                let close = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.close_folder_member(folder, member_index, window, cx);
                });
                children.push(components::folder_item_row(theme, member, select, close).into_any());
            }
        }
        Some(
            div()
                .id(components::folders::FOLDERS_ID)
                .track_focus(&self.u2.folders_handle)
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
                .on_key_down(cx.listener(Self::on_folders_key_down))
                .children(children),
        )
    }
}

/// The §11.4/§11.10/§11.11 rail inputs derived once per frame from the L5
/// organization read model, shared by the tab surface, the favorites grid,
/// and the folders list (and by the snapshot's §15 derivation).
pub struct U2Rail {
    pub favorite_tiles: Vec<crate::components::FavoriteTileSpec>,
    pub folder_rows: Vec<crate::components::FolderRowSpec>,
    pub folder_members: Vec<crate::components::FolderMemberVisual>,
}

impl BbrApp {
    /// Derive the U2 rail inputs from the L5 organization read model
    /// (reconciled on read) plus the app-local presentation state, so the
    /// spaces/favorites/folders/plus-menu surfaces all consume the same
    /// source the snapshot reports.
    pub(crate) fn u2_rail_data(&mut self) -> U2Rail {
        let organization = self.organization.organization_snapshot(&self.controller);
        let favorite_tiles = components::favorite_tiles(&organization.favorites);
        let folder_rows =
            components::folder_row_specs(&organization.folders, &self.u2.folder_presentation);
        let folder_members = components::folder_member_visuals(
            &folder_rows,
            &organization.folders,
            &self.controller.snapshot().all_tabs,
        );
        U2Rail {
            favorite_tiles,
            folder_rows,
            folder_members,
        }
    }

    /// §11.4 footer row with its production listeners wired: `[Downloads]`
    /// is the placeholder control (the surface is Phase 8), `[Spaces]`
    /// toggles the §11.3 space region, and `[+]` opens the plus menu. The
    /// plus press marks itself as an inside press and stops propagation so
    /// the sidebar's own chrome-focus handler and the root outside-press
    /// dismissal never fight the freshly opened menu (focus stays on the
    /// menu surface).
    pub(crate) fn u2_footer_surface(
        &mut self,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let on_downloads = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.activate_downloads(window, cx);
        });
        let on_spaces = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.toggle_spaces_region(window, cx);
        });
        let on_plus = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.u2.plus_menu_press_inside = true;
            cx.stop_propagation();
            this.toggle_plus_menu(window, cx);
        });
        components::footer_surface(theme, on_downloads, on_spaces, on_plus)
    }

    /// §11.4 pre-tab region: the §11.3 spaces region (only while the footer
    /// `[Spaces]` control reveals it) then the §11.10 favorites grid (hidden
    /// entirely while empty). Placed between the command results and the tab
    /// surface.
    pub(crate) fn u2_pre_tab_children(
        &self,
        column: Stateful<Div>,
        space_surface: Stateful<Div>,
        favorites: Option<Stateful<Div>>,
    ) -> Stateful<Div> {
        column
            .when(self.u2.spaces_visible, |column| column.child(space_surface))
            .when(favorites.is_some(), |column| {
                column.child(favorites.expect("checked above"))
            })
    }

    /// §11.4 post-tab region: the plus menu (only while open) then the
    /// footer row. Placed after the status row + collapse control.
    pub(crate) fn u2_post_tab_children(
        &mut self,
        column: Stateful<Div>,
        plus_menu: Option<Stateful<Div>>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        column
            .when(plus_menu.is_some(), |column| {
                column.child(plus_menu.expect("checked above"))
            })
            .child(self.u2_footer_surface(theme, cx))
    }
}
