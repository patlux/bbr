//! Reusable typed GPUI Chassis components (DESIGN.md §11).
//!
//! Every shell surface composes these instead of inventing local layout:
//! tab rows (pinned/regular anatomy), favicon tiles (monogram fallback),
//! section labels, the close control, pin control, the drag insertion tick,
//! the empty-space pane, and the crash panel. Visual state comes from the
//! §10 vocabulary; listeners are passed in so any view can wire these
//! without reimplementing the anatomy.
//!
//! The components are split by concern so no module outgrows the ~500-line
//! budget: [`tab_row`] owns the row anatomy and its controls, [`space`] the
//! profile chips and space rows, [`pane`] the empty/crash panes, and
//! [`section`] the section labels. Everything callers reference keeps the
//! single `crate::components::*` path through the re-exports below.

pub mod pane;
pub mod section;
pub mod space;
pub mod tab_row;

pub use pane::{crash_panel, empty_space_pane};
pub use section::section_label;
#[cfg(feature = "testkit")]
pub use space::profile_chip_id;
pub use space::{profile_chip, space_new_row, space_row};
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

/// The one space-row visual state the shell renders (DESIGN.md §10/§11.3):
/// the active space is `selected` (accent rail + tint), every other space is
/// `default`. The states vocabulary matches the tab rows so the rail is the
/// primary selection cue in both surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceVisualState {
    Default,
    Selected,
    /// §10.9: a tab drag currently targets this space row (the destination
    /// profile chip replaces the insertion tick per §12.3).
    DragTarget,
}

/// §12 profile chip identity: the 1–2 character monogram plus the stable
/// round-robin hue the domain assigned to the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileChipSpec {
    pub letter: String,
    pub hue: bbr_browser::ProfileHue,
}

/// One rendered space row's identity, derived from the controller snapshot.
#[derive(Debug, Clone)]
pub struct SpaceRowVisuals {
    /// The space's stable creation-order index (`bbr-space-row-<n>`).
    pub index: usize,
    /// The owning profile's creation-order index — the stable
    /// `bbr-profile-chip-<n>` identity (DESIGN.md §12), never the space
    /// index, so the rendered chip matches the E2E contract even when space
    /// and profile indices diverge.
    pub profile_index: usize,
    pub title: SharedString,
    /// The §11.3 count column (`type:meta` tabular figures).
    pub count: usize,
    pub state: SpaceVisualState,
    pub chip: ProfileChipSpec,
}

use gpui::SharedString;
