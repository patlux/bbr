//! App-side G3 Sidebar model builders (DESIGN.md §11.1–§11.4): the
//! favorites/folders/tabs/plus-menu/footer/strip DTO construction from the
//! controller snapshot and the app presentation state. The `bbr-ui` rail
//! composition renders these plain models; the app keeps every authority
//! (dense window derivation, drag state, focus handles, downloads count).

use bbr_ui::components::{FavoriteTileSpec, FolderMemberVisual, FolderRowSpec};
use bbr_ui::sidebar::model::{
    FavoritesModel, FoldersModel, FooterModel, PlusMenuModel, StripModel, TabsModel,
};
use bbr_ui::theme::Theme;
use gpui::Window;

use super::BbrApp;

impl BbrApp {
    /// The §11.10 favorites region model: the deterministic tile grid plus
    /// the §10.9 drag-target wash while a tab drag targets the grid. `None`
    /// while the region is empty at rest — the label hides with it (§11.4
    /// region rule) — except while a tab drag is in flight, when the region
    /// renders as the compact drop band (label + wash once targeted) so the
    /// first favorite can be created by dragging a concrete tab into the
    /// grid (DESIGN.md §11.10 data source / daily-driver Sidebar slice).
    pub(super) fn favorites_model(&self, tiles: &[FavoriteTileSpec]) -> Option<FavoritesModel> {
        let drag_active = self.tab_drag.is_active();
        let targeted =
            self.tab_drag.target() == Some(crate::app::drag_target::DragTarget::Favorites);
        if tiles.is_empty() && !drag_active {
            return None;
        }
        Some(FavoritesModel {
            tiles: tiles.to_vec(),
            targeted,
        })
    }

    /// The §11.11 folders region model: the folder rows plus the nested
    /// member rows of expanded folders, and the region's keyboard-focus
    /// state. `None` while empty.
    pub(super) fn folders_model(
        &self,
        rows: &[FolderRowSpec],
        members: &[FolderMemberVisual],
        window: &Window,
    ) -> Option<FoldersModel> {
        if rows.is_empty() {
            return None;
        }
        Some(FoldersModel {
            rows: rows.to_vec(),
            members: members.to_vec(),
            focused: self.u2.folders_handle.is_focused(window),
        })
    }

    /// The §11.1/§11.2/§11.6 tab region model: the bounded dense window the
    /// app derived from the production scroll anchor, plus the drag tick and
    /// the list's keyboard-focus state. Also records what this render built
    /// so the testkit snapshot reports the bounded window and the
    /// deterministic scroll anchor (the snapshot is taken on demand; the
    /// recorded state mirrors the last frame).
    pub(super) fn tabs_model(&mut self, theme: &Theme, window: &Window) -> TabsModel {
        let browser = self.controller.snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = &browser.tabs[pinned..];
        let row_h = theme.density.row();
        let offset_y = self.dense_scroll.offset().y;
        let anchor = crate::dense::anchor_from_offset(offset_y, row_h, regular.len());
        let row_window = crate::dense::window_for(anchor, regular.len());
        let drag_source = self.tab_drag.source();
        let pinned_rows = self.row_visuals(&browser.tabs[..pinned], drag_source);
        let window_rows = self.row_visuals(&regular[row_window.clone()], drag_source);
        let tick = self.tab_drag.tab_slot();
        let tick_valid = self.tab_drag.valid();
        let list_focused = self.tab_list_handle.is_focused(window);

        self.dense_window.set(crate::dense::DenseRenderState {
            first_rendered: pinned + row_window.start,
            rendered_rows: row_window.len(),
            item_count: regular.len(),
            scroll_anchor: anchor,
        });

        TabsModel {
            pinned: pinned_rows,
            window: window_rows,
            scroll_top: crate::dense::scroll_top_for_anchor(row_window.start, row_h),
            scroll_bottom: (regular.len().saturating_sub(row_window.end)) as f32 * f32::from(row_h),
            tick,
            tick_valid,
            list_focused,
        }
    }

    /// The §11.4 plus-menu model: the keyboard selection index into the
    /// fixed two-item v1 menu. `None` while closed.
    pub(super) fn plus_menu_model(&self) -> Option<PlusMenuModel> {
        if !self.u2.plus_menu.open {
            return None;
        }
        Some(PlusMenuModel {
            selected: self.u2.plus_menu.selection,
        })
    }

    /// The §11.4 footer model: the `[Downloads]` `tnum` count, the control's
    /// keyboard-focus state, and the focus handle (dismissal returns focus
    /// here).
    pub(super) fn footer_model(&self, window: &Window) -> FooterModel {
        FooterModel {
            downloads_count: self.downloads_controller_active_count(),
            downloads_focused: self.downloads_control_handle.is_focused(window),
            downloads_focus: self.downloads_control_handle.clone(),
        }
    }

    /// The §11.3 permanent space-strip model: the chip visuals in canonical
    /// order, the interpolated swipe tint, and the scroll/focus handles.
    pub(super) fn strip_model(&self) -> StripModel {
        StripModel {
            chips: self.space_chip_visuals(),
            swipe_tint: self.swipe_tint(),
            scroll: self.space_strip_scroll.clone(),
            focus: self.space_strip_handle.clone(),
        }
    }
}
