//! The gallery semantic snapshot (packet `component-gallery-mvp.md`): the
//! §15 mode fields + stable ids the E2E asserts. Kept in its own focused
//! module so `app.rs` stays under the ~500-line budget.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};

use crate::address_recorder::AddressRecorder;
use crate::group4_recorder::Group4Recorder;
use crate::library_recorder::LibraryRecorder;

/// The semantic snapshot the E2E asserts (§15 mode fields + stable ids).
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNodeSnapshot {
    pub id: u64,
    pub kind: &'static str,
    pub label: String,
    pub visible: bool,
    /// Group nodes: whether the group is expanded (drives the disclosure
    /// chevron and child visibility); always `false` for story nodes.
    pub expanded: bool,
    /// The story's stable `StoryId` for story nodes (`None` for groups) —
    /// the E2E asserts the declaration-order prefix on this, never on the
    /// tree node id (which also counts group nodes).
    pub story_id: Option<u64>,
}

/// The semantic snapshot the E2E asserts (§15 mode fields + stable ids).
#[derive(Debug, Clone, PartialEq)]
pub struct GallerySnapshot {
    pub tree_id: String,
    pub canvas_id: String,
    pub tree_nodes: Vec<TreeNodeSnapshot>,
    pub focus_node: u64,
    pub selected_story: u64,
    pub selected_story_label: String,
    pub canvas_story_id: String,
    pub theme: ThemeMode,
    pub contrast: ContrastMode,
    pub density: Density,
    pub reduced_motion: bool,
    pub story_state: String,
    pub story_state_index: usize,
    /// The honest scope copy (finding 3): "Foundations / Primitives /
    /// Browser Components / Compositions".
    pub scope: String,
    /// The honest continuation note pointing to G2/G3 (finding 3).
    pub continuation: String,
    /// The observed Core Controls state: what the real gpui-component
    /// controls reported through the production recorder path.
    pub button_clicks: u32,
    pub checkbox_checked: bool,
    pub switch_checked: bool,
    pub radio_index: usize,
    /// The real `InputState::value()` (typed text) when the Core Controls
    /// input exists.
    pub input_text: String,
    /// The real `SliderState::value().start()` when the Core Controls
    /// slider exists.
    pub slider_value: f32,
    /// The observed Tab Row state through the production recorder path.
    pub tab_selects: u32,
    pub tab_closes: u32,
    pub tab_pins: u32,
    pub tab_drag_begins: u32,
    /// The observed Saved Pages Library state: what real Library events
    /// (close, sort, select, rename, remove, collection, open) reported
    /// through the production recorder path.
    pub library: LibraryRecorder,
    /// The observed Address/Input state through the production recorder
    /// path: the hover states the production `on_hover` reported and the
    /// number of presses that fired the production `on_open`.
    pub address: AddressRecorder,
    /// The observed Group 4 sidebar-component state through the production
    /// recorder path: the tile indices the production favorites
    /// `on_activate` reported, the folder toggles/selects/closes, the
    /// footer downloads/plus presses, and the plus-menu item commits.
    pub group4: Group4Recorder,
}
