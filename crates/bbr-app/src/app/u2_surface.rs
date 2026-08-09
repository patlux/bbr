//! U2 sidebar render surfaces (DESIGN.md §11.4/§11.10/§11.11): the plus
//! context menu, the favorites grid, and the folders list with nested rows.
//! The rail composition in `app::sidebar_view` places them per §11.4; the
//! behavior handlers live in `app::u2`. All ids come from the `components`
//! contract modules; listeners route through the production app methods.

use gpui::{div, prelude::*, Context, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use bbr_ui::components as ui;
use bbr_ui::theme::{Radius, Spacing, Theme};

use super::ui_adapters;
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
        let items: Vec<gpui::AnyElement> = ui::plus_menu_items()
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
                    .on_mouse_down(MouseButton::Left, commit)
                    .into_any()
            })
            .collect();
        Some(
            div()
                .id(ui::menu::PLUS_MENU_ID)
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
        tiles: &[ui::FavoriteTileSpec],
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
                ui::favorite_tile(theme, tile, activate).into_any()
            })
            .collect();
        let grid = ui::favorites_grid(theme, children);
        Some(
            div()
                .id(ui::favorites::FAVORITES_SECTION_ID)
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
        rows: &[ui::FolderRowSpec],
        members: &[ui::FolderMemberVisual],
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        if rows.is_empty() {
            return None;
        }
        let p = theme.palette;
        let focused = self.u2.folders_handle.is_focused(window);
        let mut children: Vec<gpui::AnyElement> =
            vec![ui::section_label(theme, "Folders", focused).into_any()];
        for row in rows {
            let id = row.id;
            let focus = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                this.focus_folder_row(id, window, cx);
            });
            let toggle = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                this.focus_folder_row(id, window, cx);
                this.toggle_folder(id, cx);
            });
            children.push(
                ui::folder_row(theme, row, toggle)
                    .on_mouse_down(MouseButton::Left, focus)
                    .into_any(),
            );
            for member in members.iter().filter(|member| member.folder == row.id) {
                let folder = member.folder;
                let member_index = member.member;
                let select = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.select_folder_member(folder, member_index, window, cx);
                });
                let close = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.close_folder_member(folder, member_index, window, cx);
                });
                children.push(ui::folder_item_row(theme, member, select, close).into_any());
            }
        }
        Some(
            div()
                .id(ui::folders::FOLDERS_ID)
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
    pub favorite_tiles: Vec<ui::FavoriteTileSpec>,
    pub folder_rows: Vec<ui::FolderRowSpec>,
    pub folder_members: Vec<ui::FolderMemberVisual>,
}

impl BbrApp {
    /// Derive the U2 rail inputs from the L5 organization read model
    /// (reconciled on read) plus the app-local presentation state, so the
    /// spaces/favorites/folders/plus-menu surfaces all consume the same
    /// source the snapshot reports.
    pub(crate) fn u2_rail_data(&mut self) -> U2Rail {
        let organization = self.organization.organization_snapshot(&self.controller);
        let favorite_tiles = ui_adapters::favorite_tiles(&organization.favorites);
        let folder_rows =
            ui_adapters::folder_row_specs(&organization.folders, &self.u2.folder_presentation);
        let folder_members = ui_adapters::folder_member_visuals(
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
    /// toggles the §11.12 downloads panel, the permanent §11.3 space strip
    /// sits between the fixed controls (one chip per space in canonical
    /// grouped order, pointer activation routed through the production
    /// [`Self::select_space_via_pointer`] path), and `[+]` opens the plus
    /// menu. There is no `[Spaces]` toggle control in the rendered
    /// composition — the strip is always part of the rail. The downloads
    /// and plus presses mark themselves as inside presses and stop
    /// propagation so the sidebar's own chrome-focus handler and the root
    /// outside-press dismissal never fight the freshly opened surface
    /// (focus stays on the panel/menu surface). The downloads control
    /// carries the §11.4 `type:meta` `tnum` count while any active attempt
    /// exists.
    pub(crate) fn u2_footer_surface(
        &mut self,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let on_downloads = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.mark_downloads_press_inside();
            cx.stop_propagation();
            crate::app::downloads_behavior::toggle_downloads(this, window, cx);
        });
        let on_plus = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.u2.plus_menu_press_inside = true;
            cx.stop_propagation();
            this.toggle_plus_menu(window, cx);
        });
        // §11.3/§11.4 permanent strip: one chip per space from the same
        // `space_chip_visuals` vector the render, the snapshot, and the
        // scroll seam consume. Each chip's pointer listener routes through
        // `select_space_via_pointer` → `activate_space` (chrome focus stays
        // in the strip); the strip carries the §9 keyboard traversal and the
        // tracked scroll handle that brings the selected chip into view.
        // While a sidebar swipe tracks, the active chip's surface
        // interpolates toward the adjacent space's presentation (the same
        // chrome-only interpolation the rail consumes; selection evidence
        // stays truthful).
        let swipe_tint = self.swipe_tint();
        let chips = self
            .space_chip_visuals()
            .iter()
            .map(|visual| {
                let space = visual.space;
                let select = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.select_space_via_pointer(space, window, cx)
                });
                crate::components::space_chip(theme, visual, swipe_tint, select)
            })
            .collect::<Vec<Stateful<Div>>>();
        let strip = crate::components::space_strip(theme, self.space_strip_scroll.clone(), chips)
            .track_focus(&self.space_strip_handle)
            .on_key_down(cx.listener(Self::on_space_strip_key_down));
        ui::footer_surface(
            theme,
            self.downloads_control_handle.clone(),
            self.downloads_control_handle.is_focused(window),
            self.downloads_controller_active_count(),
            strip,
            on_downloads,
            on_plus,
        )
    }

    /// The §11.4 footer `[Downloads]` `tnum` count: present while any
    /// attempt exists. Per the settled §17.11 decision the count feeds
    /// **only in-flight (active) attempts** — it returns to zero when
    /// nothing is transferring, terminal history never inflates the badge.
    pub(crate) fn downloads_controller_active_count(&self) -> Option<usize> {
        let count = self.downloads.controller.active_ids().len();
        (count > 0).then_some(count)
    }

    /// §11.4 pre-tab region: the §11.10 favorites grid (hidden entirely
    /// while empty). Placed between the command results and the tab
    /// surface. The §11.3 space strip is part of the footer (always
    /// rendered), not a revealable region above the tab list.
    pub(crate) fn u2_pre_tab_children(
        &self,
        column: Stateful<Div>,
        favorites: Option<Stateful<Div>>,
    ) -> Stateful<Div> {
        column.when(favorites.is_some(), |column| {
            column.child(favorites.expect("checked above"))
        })
    }

    /// §11.4 post-tab region: the downloads panel (only while open or
    /// fading out — §11.12 anchors its bottom edge flush with the footer
    /// control's top edge by being the footer's direct predecessor in the
    /// column), the plus menu (only while open), then the footer row.
    /// The downloads panel and the plus menu are mutually exclusive (open
    /// closes the other), so the two never stack.
    pub(crate) fn u2_post_tab_children(
        &mut self,
        column: Stateful<Div>,
        plus_menu: Option<Stateful<Div>>,
        theme: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let downloads_panel = self.downloads_panel_surface(theme, window, cx);
        column
            .when(downloads_panel.is_some(), |column| {
                column.child(downloads_panel.expect("checked above"))
            })
            .when(plus_menu.is_some(), |column| {
                column.child(plus_menu.expect("checked above"))
            })
            .child(self.u2_footer_surface(theme, window, cx))
    }
}
