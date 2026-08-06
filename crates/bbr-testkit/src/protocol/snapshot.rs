//! The typed `Snapshot` and its row/space/profile parts, plus the §15
//! visual-state vocabulary (DESIGN.md §15). All plain data with `serde`
//! derives so the wire format never leaks framework types.

use bbr_web::{ProfileId, WebViewId};
use serde::{Deserialize, Serialize};

use super::{CommandBarMode, ShellModes, TestProfileId, TestSpaceId, TestWebViewId};

/// Visual state of one tab row, mirroring the DESIGN.md §10 vocabulary that
/// the shell renders. Asserted by E2E via the snapshot's `tab_states`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabVisualState {
    Default,
    Selected,
    Loading,
    Error,
}

/// Visual state of one space row (DESIGN.md §11.3): the active space uses
/// the selected rail + tint exactly like a selected tab; every other space
/// rests. Asserted by E2E via the snapshot's `space_states`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceVisualState {
    Default,
    Selected,
}

/// One profile's identity on the wire (DESIGN.md §12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    /// Creation-order index — the stable `bbr-profile-chip-<n>` identity.
    pub index: usize,
    pub id: TestProfileId,
    pub title: String,
    /// One of the six DESIGN.md §12 hue names (cobalt/violet/fuchsia/
    /// amber/terracotta/slate).
    pub hue: String,
}

/// One space on the wire, including its own active tab and live session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceSnapshot {
    /// Creation-order index — the stable `bbr-space-row-<n>` identity.
    pub index: usize,
    pub space: TestSpaceId,
    pub profile: TestProfileId,
    pub title: String,
    /// The space's own selected tab (restored on activation), space-local.
    pub active_tab: Option<usize>,
    /// Live session of the space's active tab (`None` when empty/suspended).
    pub active_view: Option<TestWebViewId>,
    /// Whether this is the app's active space.
    pub active: bool,
    /// Tab count for the §11.3 count column.
    pub tab_count: usize,
}

/// One tab in the active space, including its disposable native session id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub index: usize,
    pub url: String,
    pub title: String,
    /// Leading tabs of the space are pinned (DESIGN.md §11.2).
    pub pinned: bool,
    pub loading: bool,
    /// The tab's web process terminated (DESIGN.md §10.7 error state).
    pub crashed: bool,
    pub active: bool,
    pub web_view: Option<WebViewId>,
}

/// A snapshot of the whole app, returned on demand by `GetSnapshot`.
///
/// Deliberately typed (no free-form JSON) so the harness can assert on
/// fields without stringly-typed marshalling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Active web view state (None until the first navigation event).
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// A live native web view backs the active tab.
    pub web_view_hosted: bool,
    pub active_profile: ProfileId,
    /// Display title of the active profile (DESIGN.md §11.8 pane naming).
    pub active_profile_title: Option<String>,
    /// The active profile's identity hue name (DESIGN.md §12).
    pub active_profile_hue: Option<String>,
    pub active_space: TestSpaceId,
    /// Active-space tabs and their current disposable session ids.
    pub tabs: Vec<TabSnapshot>,
    /// Complete profile collection in creation order (DESIGN.md §12).
    pub profiles: Vec<ProfileSnapshot>,
    /// Complete space collection in creation order, each with its own active
    /// tab and live session (the "each space remembers its active tab"
    /// contract).
    pub spaces: Vec<SpaceSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<WebViewId>,
    /// Number of native sessions currently owned by the engine.
    pub engine_sessions: usize,
    /// Engine implementation tag, e.g. "native-wkwebview" or "fake".
    pub engine_kind: String,
    /// GPUI chrome focus (command bar focused\) and the shell's resolved
    /// mode fields (theme/contrast/density) for §15 assertions.
    pub chrome_focused: bool,
    pub shell_modes: ShellModes,
    /// Per-row visual states (selected/loading/error/default) in tab order.
    pub tab_states: Vec<TabVisualState>,
    /// Per-row rendered titles (after §4 separator-aware truncation) in tab
    /// order, from the same `row_visuals` source the shell renders.
    pub row_titles: Vec<String>,
    /// §11.3 space states (selected/default) in render order, grouped by
    /// profile.
    pub space_states: Vec<SpaceVisualState>,
    /// §11.3 space row titles in render order, after §4 truncation.
    pub space_titles: Vec<String>,
    /// The stable `bbr-*` ids present in the current render (DESIGN.md §15).
    pub visible_ids: Vec<String>,
    pub command_bar_mode: CommandBarMode,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    /// The effective sidebar rail width (DESIGN §5 range, window-clamped;
    /// 0 when collapsed).
    pub sidebar_width: f32,
    /// The DESIGN §5 adjustable-range contract, exposed so E2E asserts the
    /// same constants the render enforces.
    pub sidebar_min: f32,
    pub sidebar_default: f32,
    pub sidebar_max: f32,
    /// The native-content floor (DESIGN §5/§13): the WKWebView slot is never
    /// laid out narrower, so the rail yields below the requested width in
    /// compact windows. Exposed so E2E and the benchmark drivers compute the
    /// exact clamped geometry the render enforces.
    pub content_min: f32,
    /// The resize divider drag state (idle vs dragging).
    pub divider_dragging: bool,
    /// The stable divider label (future Accessibility; §15 shared identity).
    pub divider_label: String,
    /// The last native slot rect handed to the engine (WKWebView frame).
    pub content_rect: [f32; 4],
    /// Window content size in points (diagnostics only).
    pub window_size: [f32; 2],
}
