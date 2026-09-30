//! U2 sidebar rail data (DESIGN.md §11.4/§11.10/§11.11): the L5
//! organization read-model inputs the G3 Sidebar assembler derives once per
//! frame — the favorites tiles, the folder rows/members, and the footer
//! `[Downloads]` `tnum` count. The rail composition itself lives in
//! `bbr-ui::sidebar`; the behavior handlers live in `app::u2`. All ids come
//! from the `bbr-ui` components contract modules; the app keeps the
//! authority.

use bbr_ui::components as ui;

use super::ui_adapters;
use super::BbrApp;

/// The §11.4/§11.10/§11.11 rail inputs derived once per frame from the L5
/// organization read model, shared by the tab surface, the favorites grid,
/// and the folders list (and by the snapshot's §15 derivation).
pub struct U2Rail {
    pub favorite_tiles: Vec<ui::FavoriteTileSpec>,
    pub folder_rows: Vec<ui::FolderRowSpec>,
    pub folder_members: Vec<ui::FolderMemberVisual>,
    /// §11.11: the active-space tab indices grouped into a folder (they
    /// render inside the folder only, never as loose pinned rows).
    pub grouped: Vec<usize>,
}

impl BbrApp {
    /// Derive the U2 rail inputs from the L5 organization read model
    /// (reconciled on read) plus the app-local presentation state, so the
    /// spaces/favorites/folders/plus-menu surfaces all consume the same
    /// source the snapshot reports.
    pub(crate) fn u2_rail_data(&mut self) -> U2Rail {
        // Refresh the decoded favicon mirror once per frame before any
        // presentation derives from it (the mirror is keyed by the same
        // `(profile, host)` records the controller's bounded store holds).
        self.refresh_favicon_mirror();
        let organization = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read());
        let favorite_tiles =
            ui_adapters::favorite_tiles(&organization.favorites, &self.favicon_images.read());
        // §11.11 hover-reveal: the pointer-hovered folder's count paints;
        // the focused half is applied by `folders_model` with the live
        // window (the region must genuinely own chrome focus for the
        // keyboard anchor to reveal). At rest every count stays quiet.
        let mut folder_rows = ui_adapters::folder_row_specs(
            &organization.folders,
            &self.u2.folder_presentation,
            self.hovered_folder,
            self.folder_rename_view(),
        );
        // §10.9/§11.11: the folder row under a dragged tab is the one live
        // drop target (blueprint D02).
        let drop_folder = self.tab_drag.folder_target();
        for row in &mut folder_rows {
            row.drop_target = drop_folder == Some(row.id);
        }
        let folder_members = ui_adapters::folder_member_visuals(
            &folder_rows,
            &organization.folders,
            &self.controller.read().snapshot(),
            &self.favicon_images.read(),
        );
        let grouped = crate::shell::organized::grouped_indices(
            &organization.folders,
            &self.controller.read().snapshot(),
        );
        U2Rail {
            favorite_tiles,
            folder_rows,
            folder_members,
            grouped,
        }
    }

    /// The §11.4 footer `[Downloads]` `tnum` count: present while any
    /// attempt exists. Per the settled §17.11 decision the count feeds
    /// **only in-flight (active) attempts** — it returns to zero when
    /// nothing is transferring, terminal history never inflates the badge.
    pub(crate) fn downloads_controller_active_count(&self) -> Option<usize> {
        let count = self.downloads.controller.read().active_ids().len();
        (count > 0).then_some(count)
    }
}
