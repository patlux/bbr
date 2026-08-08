//! U2 §15 visible-id derivation (DESIGN.md §11.4/§11.10/§11.11): the §11.4
//! spaces-region gating (rows/chips render only while the footer `[Spaces]`
//! control reveals them) plus the footer row, favorites grid, folders list,
//! and plus menu ids the rail composition folds into [`super::visible_ids`].
//! The folder member ids derive from the same `folder_member_visuals`
//! resolution the render and the snapshot use, so a nested row id can never
//! drift from the render. Split from `super` so the snapshot module stays
//! under the ~500-line budget.

use bbr_browser::ControllerSnapshot;

/// The U2 §15 inputs `visible_ids` folds into its id list (bundled so the
/// legacy call sites stay short): the §11.4 spaces-region toggle plus the
/// favorites/folders/plus-menu state the U2 surfaces render. The default
/// mirrors the pre-compact-rail rail (region revealed, no U2 content) —
/// same convention as the wire `Snapshot::spaces_visible`.
pub struct U2IdsInput<'a> {
    pub spaces_visible: bool,
    pub favorites: &'a [bbr_browser::FavoriteSnapshot],
    pub folders: &'a [bbr_browser::FolderSnapshot],
    pub expanded: crate::components::folders::FolderPresentation,
    pub plus_menu: crate::components::menu::PlusMenuState,
    /// §11.12 downloads panel gating: open flag plus the bounded render
    /// window and group layout the render built (the visible ids mirror the
    /// panel's own gating — labels hidden with empty groups, rows only
    /// inside the rendered window, the empty hint when nothing exists).
    pub downloads_open: bool,
    pub downloads_window: std::ops::Range<usize>,
    pub downloads_layout: crate::app::downloads::PanelLayout,
}

impl<'a> Default for U2IdsInput<'a> {
    fn default() -> Self {
        U2IdsInput {
            spaces_visible: true,
            favorites: &[],
            folders: &[],
            expanded: crate::components::folders::FolderPresentation::default(),
            plus_menu: crate::components::menu::PlusMenuState::default(),
            downloads_open: false,
            downloads_window: 0..0,
            downloads_layout: crate::app::downloads::PanelLayout {
                label_h: 0.0,
                row_h: 0.0,
                active_count: 0,
                history_count: 0,
                hairline: 1.0,
            },
        }
    }
}

/// The §11.4 spaces-region ids plus the U2 surface ids, folded into the
/// caller's §15 list in render order (space rows/chips only while the
/// region is revealed, the U2 surfaces with the same gating the render
/// uses).
pub(crate) fn extend_visible_ids(
    ids: &mut Vec<String>,
    browser: &ControllerSnapshot,
    u2: &U2IdsInput<'_>,
) {
    // §11.4 spaces region: the space rows (and their §12 chips) render
    // only while the footer `[Spaces]` control reveals the region.
    if u2.spaces_visible {
        ids.push("bbr-space-list".into());
        ids.push("bbr-space-new".into());
        for space in &browser.spaces {
            ids.push(format!("bbr-space-row-{}", space.index));
            ids.push(format!("bbr-space-title-{}", space.index));
            ids.push(format!("bbr-space-count-{}", space.index));
        }
        for profile in &browser.profiles {
            ids.push(crate::components::profile_chip_id(profile.index).to_string());
        }
    }
    // The §11.4 U2 surfaces: the footer row (always part of the rail), the
    // favorites grid / folders list (each hidden entirely while empty), the
    // plus menu (only while open), and the §11.12 downloads panel (only
    // while open) — the same gating the render uses.
    ids.extend(u2_visible_ids(
        browser,
        u2.favorites,
        u2.folders,
        &u2.expanded,
        &u2.plus_menu,
    ));
    if u2.downloads_open {
        ids.extend(crate::app::downloads_render::downloads_visible_ids(
            u2.downloads_layout.active_count + u2.downloads_layout.history_count,
            &u2.downloads_window,
            &u2.downloads_layout,
        ));
    }
}

/// The U2 §15 id additions once the U2 surfaces render: the §11.4 footer row
/// (always part of the rail), the §11.10 favorites grid (section label +
/// tiles, hidden entirely while empty), the §11.11 folders list (rows,
/// disclosure/name/count ids, and — for expanded folders — the nested
/// `bbr-folder-item-<n>-<m>` ids plus their title/favicon/close slots), and
/// the §11.4 plus menu (surface + two items, only while open).
///
/// The folder member ids derive from the same `folder_member_visuals`
/// resolution the render and the snapshot use (the controller's global tab
/// model resolves the members of expanded folders), so a nested row id can
/// never drift from the render.
pub(crate) fn u2_visible_ids(
    browser: &ControllerSnapshot,
    favorites: &[bbr_browser::FavoriteSnapshot],
    folders: &[bbr_browser::FolderSnapshot],
    expanded: &crate::components::folders::FolderPresentation,
    plus_menu: &crate::components::menu::PlusMenuState,
) -> Vec<String> {
    let mut ids = crate::components::footer::footer_ids();
    ids.extend(crate::components::favorites::favorites_ids(
        &crate::components::favorites::favorite_tiles(favorites),
    ));
    let rows = crate::components::folders::folder_row_specs(folders, expanded);
    let members =
        crate::components::folders::folder_member_visuals(&rows, folders, &browser.all_tabs);
    ids.extend(crate::components::folders::folders_ids(&rows, &members));
    ids.extend(crate::components::menu::plus_menu_ids(plus_menu));
    ids
}
