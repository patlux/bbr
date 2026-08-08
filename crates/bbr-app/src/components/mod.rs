//! Reusable typed GPUI Chassis components (DESIGN.md §11).
//!
//! Every shell surface composes these instead of inventing local layout:
//! tab rows (pinned/regular anatomy), favicon tiles (monogram fallback),
//! section labels, the close control, pin control, the drag insertion tick,
//! the empty-space pane, the crash panel, and — for the U2 sidebar — the
//! §11.10 favorites grid, the §11.11 folder rows, the §11.4 footer row, and
//! the plus-menu item contract. Visual state comes from the §10
//! vocabulary; listeners are passed in so any view can wire these without
//! reimplementing the anatomy.
//!
//! The components are split by concern so no module outgrows the ~500-line
//! budget: [`tab_row`] owns the row anatomy and its controls, [`space`] the
//! profile chips and space rows, [`pane`] the empty/crash panes, [`section`]
//! the section labels, [`icons`] the §7 drawn glyph catalog, [`favorites`]
//! the §11.10 grid, [`folders`] the §11.11 rows and presentation state,
//! [`footer`] the §11.4 footer row, and [`menu`] the plus-menu item
//! contract and state. Everything callers reference keeps the single
//! `crate::components::*` path through the re-exports below.

pub mod favorites;
pub mod folders;
pub mod footer;
pub mod icons;
pub mod menu;
pub mod pane;
pub mod section;
pub mod space;
pub mod space_strip;
pub mod tab_row;
mod u2_glyphs;

pub use favorites::{favorite_tile, favorite_tiles, favorites_grid, FavoriteTileSpec};
#[cfg(feature = "testkit")]
pub use favorites::{favorite_tile_icon_id, favorite_tile_id};
#[cfg(feature = "testkit")]
pub use folders::{
    folder_count_id, folder_item_close_id, folder_item_favicon_id, folder_item_id,
    folder_item_title_id, folder_name_id, folder_row_id, folder_toggle_id,
};
pub use folders::{
    folder_item_row, folder_member_visuals, folder_row, folder_row_specs, FolderMemberVisual,
    FolderRowSpec,
};
pub use footer::footer_surface;
pub use icons::Icon;
pub use menu::plus_menu_items;
pub use pane::{crash_panel, empty_space_pane};
pub use section::section_label;
pub use space::profile_chip;
pub use space_strip::{space_chip, space_strip, SpaceChipVisuals};
#[cfg(feature = "testkit")]
pub use space_strip::{space_chip_id, space_identity_id};
pub use tab_row::{drag_tick, tab_row};

/// §10 row states the shell renders and E2E asserts. The slice renders the
/// full §15 vocabulary; hover/active/focused/attention ride later polish
/// passes but the names are already contract (they are derived from
/// render state the polish passes introduce, so they legitimately stay
/// unconstructed until then).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TabVisualState {
    Default,
    Hover,
    Active,
    Selected,
    Loading,
    Attention,
    Error,
    Disabled,
    /// §10.9: the dragged row (92% opacity, elev:2 shadow, strong outline).
    Dragging,
    /// §10.9: the drop target (accent tick + target outline + tint; the
    /// space target replaces the tick with the destination profile chip).
    DragTarget,
    Focused,
}

/// The favicon tile: the deterministic monogram (§7.2) until a real favicon
/// exists; while loading the slot shows the accent spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaviconSpec {
    pub letter: char,
    pub hue: usize,
}

/// One row's rendered identity, derived from the controller snapshot.
#[derive(Debug, Clone)]
pub struct RowVisuals {
    pub index: usize,
    pub title: SharedString,
    pub state: TabVisualState,
    pub favicon: Option<FaviconSpec>,
    /// §11.2: whether the row is in the pinned region (drives the pin
    /// control's region identity).
    pub pinned: bool,
    pub show_close: bool,
    /// §10.2 hover-reveal: the close control is visible when the pointer
    /// hovers or presses the row, the row is selected (keyboard users), or
    /// the row is being dragged — never at rest, so the row stays quiet
    /// until interaction.
    pub reveal_close: bool,
}

/// §12 profile chip identity: the 1–2 character monogram plus the stable
/// round-robin hue the domain assigned to the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileChipSpec {
    pub letter: String,
    pub hue: bbr_browser::ProfileHue,
}

/// The §12 destination-profile companion of a chip during a drag (DESIGN.md
/// §10.9/§11.3): the destination space's profile identity — creation-order
/// index (the `bbr-profile-chip-<n>` id), monogram, and hue — rendered
/// inside the targeted chip so the cross-profile destination is unmistakable
/// without relying on color. Derived from the controller snapshot in the
/// same `space_chip_visuals` derivation the render, the snapshot, and the
/// semantic tree consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCompanion {
    /// The destination profile's creation-order index (the §12 chip id).
    pub profile_index: usize,
    /// The §12 monogram (the non-color identity).
    pub letter: String,
    /// The §12 hue (the tile's identity; never the only cue).
    pub hue: bbr_browser::ProfileHue,
}

/// One canonical space-surface row identity, derived from the controller
/// snapshot: the persistent-space lookup key and the §4-truncated title. The
/// `space_groups` derivation is the single canonical order the permanent
/// §11.3 strip (chips), the keyboard traversal, and the §15 id derivation
/// all consume; the chips themselves carry the appearance identity and the
/// selected state from the same controller snapshot.
#[derive(Debug, Clone)]
pub struct SpaceRowVisuals {
    /// The space's creation-order index (the persistent-space lookup key).
    pub index: usize,
    pub title: SharedString,
}

use gpui::SharedString;
