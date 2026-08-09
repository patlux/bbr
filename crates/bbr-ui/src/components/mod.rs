//! Reusable typed GPUI Chassis components (DESIGN.md §11) — the honest
//! repeated primitives of the BBR chrome.
//!
//! Every shell surface composes these instead of inventing local layout:
//! section labels, favicon tiles (monogram fallback), tab rows (pinned/
//! regular anatomy), the close/pin controls, the drag insertion tick,
//! profile chips, the empty-space pane, the crash panel, the §11.10
//! favorites grid, the §11.11 folder rows, the §11.4 footer row, the
//! plus-menu item contract, and the §11.5 address pill + staging surface.
//! Visual state comes from the §10 vocabulary ([`crate::state`]);
//! listeners are passed in so any view can wire these without
//! reimplementing the anatomy.
//!
//! The components are split by concern so no module outgrows the ~500-line
//! budget. Everything callers reference keeps the single
//! `bbr_ui::components::*` path through the re-exports below (the same
//! path shape the pre-cutover `crate::components` had, so the app
//! migration is a pure import-path change).

pub mod address;
pub mod favicon;
pub mod favorites;
pub mod folders;
pub mod footer;
pub mod menu;
pub mod pane;
pub mod section;
pub mod space;
pub mod tab_row;

pub use address::{
    address_control, staging_surface, AddressModel, AddressProfileHint, StagingModel,
};
pub use favicon::{favicon_tile, FaviconSpec};
pub use favorites::{
    favorite_tile, favorite_tile_icon_id, favorite_tile_id, favorites_grid, FavoriteTileSpec,
};
pub use folders::{
    folder_count_id, folder_item_close_id, folder_item_favicon_id, folder_item_id, folder_item_row,
    folder_item_title_id, folder_name_id, folder_row, folder_row_id, folder_toggle_id,
    FolderMemberVisual, FolderPresentation, FolderRowSpec,
};
pub use footer::{footer_control, footer_surface, FooterControl};
pub use menu::{plus_menu_items, PlusMenuItem, PlusMenuState};
pub use pane::{crash_panel, empty_space_pane};
pub use section::section_label;
pub use space::{profile_chip, profile_chip_id, ProfileChipSpec, SpaceRowVisuals};
pub use tab_row::{drag_tick, tab_row, RowVisuals};
