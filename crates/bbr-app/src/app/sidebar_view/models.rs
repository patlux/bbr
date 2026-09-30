//! App-side G3 Sidebar model builders (DESIGN.md §11.1–§11.4): the
//! favorites/folders/tabs/plus-menu/footer/strip DTO construction from the
//! controller snapshot and the app presentation state. The `bbr-ui` rail
//! composition renders these plain models; the app keeps every authority
//! (dense window derivation, drag state, focus handles, downloads count).

use bbr_ui::components::{FavoriteTileSpec, FolderMemberVisual, FolderRowSpec};
use bbr_ui::sidebar::model::{
    DragProxyModel, FavoritesModel, FoldersModel, FooterModel, PlusMenuModel, StripModel, TabsModel,
};
use bbr_ui::theme::Theme;
use gpui::Window;

use super::BbrApp;

impl BbrApp {
    /// The §11.10 favorites region model: the deterministic tile grid plus
    /// the §10.9 drag-target state while a tab drag targets the grid, and
    /// the drop-cell offer while any tab drag is active. The region is
    /// always present: with zero favorites it renders the drop band that
    /// names the region and the gesture (the deliberate exception to the
    /// truthful-empty-region rule), so the first favorite can be created by
    /// dragging a concrete tab.
    pub(super) fn favorites_model(&self, tiles: &[FavoriteTileSpec]) -> Option<FavoritesModel> {
        // §11.10: the region is always present — the zero-favorites band
        // teaches the gesture at rest, so nothing reflows when a drag
        // starts.
        let targeted =
            self.tab_drag.target() == Some(crate::app::drag_target::DragTarget::Favorites);
        Some(FavoritesModel {
            tiles: tiles.to_vec(),
            targeted,
            offer: self.tab_drag.is_active(),
            pop: self.favorite_pop(),
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
        let focused = self.u2.folders_handle.is_focused(window);
        // §11.11 focus-reveal: while the folders region genuinely owns
        // chrome focus, the keyboard anchor row's `type:meta` count stays
        // revealed (the anchor is truthful — the exact row the region's
        // keyboard traversal keys on). When focus leaves the region, only
        // the pointer hover reveals. `u2_rail_data` computed the hover
        // half; this adds the focused half with the live window.
        let rows = if focused {
            let anchor = self.u2.folder_focus;
            rows.iter()
                .map(|row| {
                    let mut row = row.clone();
                    row.reveal_count = row.reveal_count || anchor == Some(row.id);
                    row
                })
                .collect()
        } else {
            rows.to_vec()
        };
        Some(FoldersModel {
            rows,
            members: members.to_vec(),
            focused,
        })
    }

    /// The §11.1/§11.2/§11.6 tab region model: the bounded dense window the
    /// app derived from the production scroll anchor, plus the drag tick and
    /// the list's keyboard-focus state. Also records what this render built
    /// so the testkit snapshot reports the bounded window and the
    /// deterministic scroll anchor (the snapshot is taken on demand; the
    /// recorded state mirrors the last frame).
    pub(super) fn tabs_model(
        &mut self,
        theme: &Theme,
        window: &Window,
        grouped: &[usize],
    ) -> TabsModel {
        let browser = self.controller.read().snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = &browser.tabs[pinned..];
        // §8 pitch: the regular region's rows render at surface + gap
        // (`density:pitch`), so the dense window, the scroll anchors and
        // the spacer extents all step in exact pitch units (§11.6).
        let pitch = theme.density.pitch();
        let offset_y = self.dense_scroll.offset().y;
        let anchor = crate::dense::anchor_from_offset(offset_y, pitch, regular.len());
        let row_window = crate::dense::window_for(anchor, regular.len());
        let drag_source = self.tab_drag.source();
        // §11.11: grouped pinned tabs render inside their folder only; the
        // loose block lists the rest (their tab indices stay the row ids).
        let loose: Vec<bbr_browser::TabSnapshot> =
            crate::shell::organized::loose_pinned(pinned, grouped)
                .into_iter()
                .map(|index| browser.tabs[index].clone())
                .collect();
        let pinned_rows = self.row_visuals(&loose, drag_source);
        let window_rows = self.row_visuals(&regular[row_window.clone()], drag_source);
        // Blueprint D19: a tab dragged in from another window shows its
        // landing slot here.
        let incoming = self.incoming_rail_slot();
        let tick = self.tab_drag.tab_slot().or(incoming);
        let tick_valid = self.tab_drag.valid() || incoming.is_some();
        // §11.2: the ghost row is offered to an active drag of an unpinned
        // tab; it is the live target while the pointer is over it.
        let pin_offer = self.tab_drag.is_active() && self.drag_source_unpinned();
        let pin_offer_targeted =
            self.tab_drag.target() == Some(crate::app::drag_target::DragTarget::PinOffer);
        let list_focused = self.tab_list_handle.is_focused(window);

        self.dense_window.set(crate::dense::DenseRenderState {
            first_rendered: pinned + row_window.start,
            rendered_rows: row_window.len(),
            item_count: regular.len(),
            scroll_anchor: anchor,
        });

        TabsModel {
            pinned: pinned_rows,
            pinned_end: pinned,
            window: window_rows,
            scroll_top: crate::dense::scroll_top_for_anchor(row_window.start, pitch),
            scroll_bottom: (regular.len().saturating_sub(row_window.end)) as f32 * f32::from(pitch),
            tick,
            tick_valid,
            list_focused,
            pin_offer,
            pin_offer_targeted,
        }
    }

    /// The §10.9 drag proxy: the dragged row's own visuals (not its
    /// `dragging` placeholder state) at the rect the shared `bbr-ui`
    /// geometry derives from the pointer. `None` unless a drag is active.
    pub(super) fn drag_proxy_model(
        &self,
        theme: &Theme,
        sidebar_width: f32,
    ) -> Option<DragProxyModel> {
        let (dx, pointer_y, press_in_row) = self.tab_drag.proxy_anchor()?;
        let source = self.tab_drag.source()?;
        let browser = self.controller.read().snapshot();
        let row = self
            .row_visuals(browser.tabs.get(source..=source)?, None)
            .into_iter()
            .next()?;
        let rect = bbr_ui::components::drag_offer::drag_proxy_rect(
            theme,
            sidebar_width,
            dx,
            pointer_y,
            press_in_row,
        );
        Some(DragProxyModel { row, rect })
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
