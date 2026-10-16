//! U2 §15 visible-id derivation (DESIGN.md §11.3/§11.4/§11.10/§11.11): the
//! permanent §11.3 space strip (its chip/identity ids always render as part
//! of the footer) plus the footer row, favorites grid, folders list, and
//! plus menu ids the rail composition folds into [`super::visible_ids`].
//! The folder member ids derive from the same `folder_member_visuals`
//! resolution the render and the snapshot use, so a nested row id can never
//! drift from the render. Split from `super` so the snapshot module stays
//! under the ~500-line budget.

use bbr_browser::ControllerSnapshot;

/// The U2 §15 inputs `visible_ids` folds into its id list (bundled so the
/// legacy call sites stay short): the favorites/folders/plus-menu state the
/// U2 surfaces render. The §11.3 space strip is always part of the rail, so
/// its ids render unconditionally (mirroring the render — there is no
/// revealable spaces region or toggle state).
pub struct U2IdsInput<'a> {
    pub favorites: &'a [bbr_browser::FavoriteSnapshot],
    /// §10.9: a tab drag is active — a populated Favorites grid offers its
    /// drop cell and the drag proxy renders (the ids mirror the render).
    pub drag_active: bool,
    /// The active drag grabbed a Favorite tile: its own grid offers no drop
    /// cell (task packet `drag-back-out-of-pins`).
    pub favorite_drag: bool,
    /// §11.2: the organized area renders the "Pin tab here" ghost row.
    pub pin_offer: bool,
    pub folders: &'a [bbr_browser::FolderSnapshot],
    pub expanded: bbr_ui::components::folders::FolderPresentation,
    pub plus_menu: bbr_ui::components::menu::PlusMenuState,
    /// §11.12 downloads panel gating: open flag plus the bounded render
    /// window and group layout the render built (the visible ids mirror the
    /// panel's own gating — labels hidden with empty groups, rows only
    /// inside the rendered window, the empty hint when nothing exists).
    /// `downloads_delete_rows` are the window row indices that render the
    /// §17.8 Delete control (cancelled rows).
    pub downloads_open: bool,
    pub downloads_window: std::ops::Range<usize>,
    pub downloads_layout: bbr_ui::surfaces::downloads::PanelLayout,
    pub downloads_delete_rows: Vec<usize>,
    /// §11.8-style error-surface gating (task packet
    /// `error-surface-contract`): the classified incident surface is
    /// rendered in the content area, so it replaces the active view's
    /// native slot ids in the visible-id list (the surface render draws
    /// `bbr-error-*` ids and never the slot it replaced).
    pub error_surface_visible: bool,
    /// The surface's URL line is rendered (the contract knows the failing
    /// origin).
    pub error_surface_origin: bool,
    /// §11.16 Saved Pages Library gating (B3 packet
    /// `saved-pages-library`): while open the internal chrome surface
    /// replaces the active view's native slot in the content area (the
    /// slot ids are not advertised, mirroring the render — the Library
    /// wins over the error surface).
    pub library_open: bool,
    /// The rendered Library §15 ids, derived from the same presentation
    /// model the typed snapshot exposes (empty while closed — the surface
    /// is not drawn).
    pub library_ids: Vec<String>,
    /// Blueprint D10: the History page's ids while it is open (it replaces
    /// the native slot like the Library; empty while closed).
    pub history_ids: Vec<String>,
    /// §11.3 Space-configuration surface gating: while open the in-rail
    /// rename/identity panel renders its full §15 id set (container,
    /// title, name field, group labels, every curated option, and the
    /// Done/Cancel controls); closed, none of them render.
    pub space_config_open: bool,
    /// Blueprint D20: an inline folder rename is open (the
    /// `bbr-folder-rename` field replaces that row's name slot).
    pub folder_renaming: bool,
}

impl<'a> Default for U2IdsInput<'a> {
    fn default() -> Self {
        U2IdsInput {
            favorites: &[],
            drag_active: false,
            favorite_drag: false,
            pin_offer: false,
            folders: &[],
            expanded: bbr_ui::components::folders::FolderPresentation::default(),
            plus_menu: bbr_ui::components::menu::PlusMenuState::default(),
            downloads_open: false,
            downloads_window: 0..0,
            downloads_layout: bbr_ui::surfaces::downloads::PanelLayout {
                label_h: 0.0,
                active_row_h: 0.0,
                history_row_h: 0.0,
                active_count: 0,
                history_count: 0,
                hairline: 1.0,
            },
            downloads_delete_rows: Vec::new(),
            error_surface_visible: false,
            error_surface_origin: false,
            library_open: false,
            library_ids: Vec::new(),
            history_ids: Vec::new(),
            space_config_open: false,
            folder_renaming: false,
        }
    }
}

/// The permanent §11.3 space-strip ids plus the U2 surface ids, folded into
/// the caller's §15 list in render order (the strip's chip/identity ids
/// derive from each space's persistent [`SpaceId`] — never the render
/// position or creation-order index — and always render, mirroring the
/// always-present footer strip).
pub(crate) fn extend_visible_ids(
    ids: &mut Vec<String>,
    browser: &ControllerSnapshot,
    u2: &U2IdsInput<'_>,
) {
    // §11.3 permanent strip: the footer's flex region always renders one
    // chip (and its identity child) per space in canonical display order;
    // the chip ids derive from the persistent space id alone.
    ids.push(bbr_ui::components::footer::FOOTER_STRIP_ID.into());
    for space in &browser.spaces {
        ids.push(bbr_ui::sidebar::space_chip_id(space.id).to_string());
        ids.push(bbr_ui::sidebar::space_identity_id(space.id).to_string());
    }
    // The §11.4 U2 surfaces: the footer row (always part of the rail), the
    // favorites grid / folders list (each hidden entirely while empty), the
    // plus menu (only while open), and the §11.12 downloads panel (only
    // while open) — the same gating the render uses.
    ids.extend(u2_visible_ids(
        browser,
        u2.favorites,
        u2.drag_active && !u2.favorite_drag,
        u2.pin_offer,
        u2.folders,
        &u2.expanded,
        &u2.plus_menu,
    ));
    if u2.drag_active && u2.favorite_drag {
        // A tile's drag shows the proxy but no drop cell.
        ids.push(bbr_ui::components::drag_offer::TAB_DRAG_PROXY_ID.into());
    }
    if u2.downloads_open {
        ids.extend(downloads_visible_ids(
            u2.downloads_layout.active_count + u2.downloads_layout.history_count,
            &u2.downloads_window,
            &u2.downloads_layout,
            &u2.downloads_delete_rows,
        ));
    }
    // §11.3: the in-rail rename/identity configuration surface (only while
    // open, mirroring the render gating).
    if u2.space_config_open {
        ids.extend(bbr_ui::surfaces::space_config::space_config_ids());
    }
    if u2.folder_renaming {
        ids.push(bbr_ui::components::folders::FOLDER_RENAME_ID.into());
    }
}

/// The §11.12/§15 download id set for the visible-ids derivation: the panel
/// surface plus the group labels, the bounded rendered rows (never the full
/// history), the empty hint, and — while any history exists — the Clear
/// control. Mirrors the render's gating exactly.
#[cfg(feature = "testkit")]
fn downloads_visible_ids(
    total: usize,
    window: &std::ops::Range<usize>,
    layout: &bbr_ui::surfaces::downloads::PanelLayout,
    delete_rows: &[usize],
) -> Vec<String> {
    use bbr_ui::surfaces::downloads::{
        download_row_id, download_slot_id, DOWNLOADS_ACTIVE_ID, DOWNLOADS_CLEAR_ID,
        DOWNLOADS_EMPTY_ID, DOWNLOADS_HISTORY_ID, DOWNLOADS_PANEL_ID,
    };
    let mut ids = vec![DOWNLOADS_PANEL_ID.to_string()];
    if layout.has_active() {
        ids.push(DOWNLOADS_ACTIVE_ID.to_string());
    }
    if layout.has_history() {
        ids.push(DOWNLOADS_HISTORY_ID.to_string());
    }
    if total == 0 {
        ids.push(DOWNLOADS_EMPTY_ID.to_string());
    } else {
        for index in window.clone() {
            if index >= total {
                break;
            }
            ids.push(download_row_id(index));
            ids.push(download_slot_id("status", index));
            ids.push(download_slot_id("name", index));
            ids.push(download_slot_id("meta", index));
            // §11.12 / Paper M-4: only Active rows carry the progress bar.
            if index < layout.active_count {
                ids.push(download_slot_id("progress", index));
            }
            ids.push(download_slot_id("action", index));
            if delete_rows.contains(&index) {
                ids.push(download_slot_id("delete", index));
            }
        }
    }
    if layout.has_history() {
        ids.push(DOWNLOADS_CLEAR_ID.to_string());
    }
    ids
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
    drag_active: bool,
    pin_offer: bool,
    folders: &[bbr_browser::FolderSnapshot],
    expanded: &bbr_ui::components::folders::FolderPresentation,
    plus_menu: &bbr_ui::components::menu::PlusMenuState,
) -> Vec<String> {
    let mut ids = bbr_ui::components::footer::footer_ids();
    // The tile/member id derivations read only ids, never the favicon
    // image, so the empty mirror is the correct deterministic input here.
    let tile_specs = crate::app::ui_adapters::favorite_tiles(
        favorites,
        &crate::app::favicon_presentation::FaviconImageMirror::default(),
    );
    // §11.10: the region always renders — the zero-favorites band or the
    // grid with its trailing cell (the rest slot, or the drop cell while a
    // tab drag is active).
    ids.extend(bbr_ui::components::favorites::favorites_ids(
        &tile_specs,
        drag_active,
    ));
    // §11.2/§10.9: the ghost row and the drag proxy render only while a tab
    // drag offers them.
    if pin_offer {
        ids.push(bbr_ui::components::drag_offer::PINNED_DROP_ROW_ID.into());
    }
    if drag_active {
        ids.push(bbr_ui::components::drag_offer::TAB_DRAG_PROXY_ID.into());
    }
    // The ids derivation is hover-independent: the `bbr-folder-count-<n>`
    // id stays in the tree at rest (zero-opacity count), so no reveal state
    // is needed here.
    let rows = crate::app::ui_adapters::folder_row_specs(folders, expanded, None, None);
    let members = crate::app::ui_adapters::folder_member_visuals(
        &rows,
        folders,
        browser,
        &crate::app::favicon_presentation::FaviconImageMirror::default(),
    );
    ids.extend(bbr_ui::components::folders::folders_ids(&rows, &members));
    ids.extend(bbr_ui::components::menu::plus_menu_ids(plus_menu));
    ids
}
