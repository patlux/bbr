//! The typed `Snapshot` and its row/space/profile parts, plus the §15
//! visual-state vocabulary (DESIGN.md §15). All plain data with `serde`
//! derives so the wire format never leaks framework types.

use bbr_web::{ProfileId, WebViewId};
use serde::{Deserialize, Serialize};

use super::{CommandBarMode, ShellModes, TestProfileId, TestSpaceId, TestWebViewId};

/// Visual state of one tab row, mirroring the DESIGN.md §10 vocabulary that
/// the shell renders. Asserted by E2E via the snapshot's `tab_states`.
/// The slice renders the full §15 vocabulary; hover/active/focused ride the
/// later polish passes but the names are already contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// Visual state of one space row (DESIGN.md §11.3): the active space uses
/// the selected rail + tint exactly like a selected tab; every other space
/// rests. Asserted by E2E via the snapshot's `space_states`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceVisualState {
    Default,
    Selected,
    /// §10.9: a tab drag currently targets this space row (the destination
    /// profile chip replaces the insertion tick per §12.3).
    DragTarget,
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
    /// Resource policy state of the tab's disposable session (the dense-list
    /// lifecycle contract): "active" | "warm" | "suspended" | "discarded".
    pub lifecycle: String,
    /// Deterministic recency stamp (the session-budget policy's "recently
    /// used" order).
    pub last_used: u64,
}

/// Lifecycle population counts of the active space (the dense-list
/// session-budget evidence): `active + warm` is the live-session population,
/// `suspended + discarded` own no session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleCounts {
    pub active: usize,
    pub warm: usize,
    pub suspended: usize,
    pub discarded: usize,
}

/// The dense-list render contract (DESIGN.md §11.6): total row counts, the
/// bounded rendered window, the deterministic scroll anchor, and active-row
/// visibility. `item_count`/`pinned_rows` are the full collection; the shell
/// renders only `[first_rendered, first_rendered + rendered_rows)` of the
/// regular region (bounded by `window_rows`), so a 500-tab list never lays
/// out 500 rows per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenseListSnapshot {
    /// Regular-region tab count (the scrollable collection).
    pub item_count: usize,
    /// Pinned-region row count (fixed block above the scroll view).
    pub pinned_rows: usize,
    /// First regular row index of the rendered window.
    pub first_rendered: usize,
    /// How many regular rows the last render actually built.
    pub rendered_rows: usize,
    /// The bounded render window (visible range plus overscan).
    pub window_rows: usize,
    /// First regular row at the viewport top, derived from the production
    /// scroll offset (deterministic scroll position).
    pub scroll_anchor: usize,
    /// Whether the active row lies inside the rendered window.
    pub active_visible: bool,
    /// The active tab's regular-region index, if any.
    pub active_row: Option<usize>,
}

/// DESIGN.md §15 semantic role of one chrome element in the internal
/// semantic model. This is the **internal** contract shared by the testkit
/// and the future platform Accessibility bridge — GPUI 0.2.2 implements no
/// platform accessibility layer (verified in its source: no NSAccessibility,
/// no accesskit), so these nodes are not exposed to macOS Accessibility
/// today. The platform bridge + packaged-app VoiceOver/Appium acceptance
/// are Phase 9; the model itself ships with the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRole {
    /// The window root (`bbr-root`).
    Root,
    /// The sidebar rail (`bbr-sidebar`).
    Sidebar,
    /// The brand block (`bbr-brand`).
    Brand,
    /// A text field / command surface (`bbr-command-bar`).
    Field,
    /// A keyboard-focusable list (`bbr-space-list` / `bbr-tab-list`).
    List,
    /// A selectable row (`bbr-tab-row-*` / `bbr-space-row-*`).
    Row,
    /// A profile identity chip (`bbr-profile-chip-*`).
    Chip,
    /// An interactive control (close, pin, toggle, New Space, pane buttons).
    Button,
    /// A section label ("Pinned" / "Tabs").
    Section,
    /// The status line (`bbr-status`).
    Status,
    /// The sidebar resize divider (`bbr-sidebar-divider`).
    Divider,
    /// The glass content area: native slot, empty pane, or crash panel.
    Content,
    /// A row's favicon / monogram tile (`bbr-tab-favicon-*`).
    Favicon,
    /// A row's title text (`bbr-tab-title-*` / `bbr-space-title-*`).
    Title,
    /// A row's count column (`bbr-space-count-*`).
    Count,
    /// A row's trailing state slot (`bbr-tab-state-*`).
    State,
    /// The command-bar cursor (`bbr-command-cursor`).
    Cursor,
}

/// One typed semantic node of the chrome (DESIGN.md §15 / accessibility
/// track): the stable `bbr-*` id, role, name, value, enabled/selected/
/// focused state, and children. The tree is derived from the same structures
/// the render consumes, so the E2E semantic assertions cannot drift from the
/// shell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticNode {
    /// The stable `bbr-*` element id (DESIGN.md §15).
    pub id: String,
    pub role: SemanticRole,
    /// Human-readable name (e.g. the truncated row title).
    pub name: Option<String>,
    /// A value (e.g. the row URL, the space tab count, the divider width).
    pub value: Option<String>,
    /// Whether the control accepts interaction (a dragged row disables its
    /// own controls).
    pub enabled: bool,
    /// Whether the element is selected (active tab / active space).
    pub selected: bool,
    /// Whether the element owns keyboard focus (GPUI focus tree; §9 ring).
    pub focused: bool,
    pub children: Vec<SemanticNode>,
}

/// The authoritative pane layout on the wire (the split shared contract):
/// primary/secondary/focused `WebViewId`s. `secondary` is `None` in
/// single-pane mode; the focused pane is the domain active tab's view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneLayoutSnapshot {
    pub primary: WebViewId,
    pub secondary: Option<WebViewId>,
    pub focused: WebViewId,
}

/// Keyed per-pane native geometry: the last engine-received WKWebView frame
/// of one pane, ordered by pane layout (primary, then secondary) in the
/// snapshot's `pane_rects` — never map iteration or creation order.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaneRectSnapshot {
    pub view: WebViewId,
    pub rect: [f32; 4],
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
    /// The engine's live-session count (native destruction evidence).
    pub engine_sessions: usize,
    /// Lifecycle population counts of the active space (dense-list evidence).
    pub lifecycle_counts: LifecycleCounts,
    /// The dense-list render contract (§11.6): bounded rendered window,
    /// deterministic scroll anchor, active-row visibility.
    pub dense: DenseListSnapshot,
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
    /// The pinned/regular boundary (DESIGN.md §11.2): leading `pinned_count`
    /// tabs of the active space are pinned. Typed explicitly so E2E asserts
    /// the boundary itself, never by re-deriving it from tab flags.
    pub pinned_count: usize,
    /// A command-bar destination hint (DESIGN.md §11.3): e.g. the destination
    /// profile when a staged move matches a space. `None` when no hint is
    /// shown.
    pub command_bar_hint: Option<String>,
    /// The §11.3 cross-context transient: after a cross-profile space
    /// activation the destination profile appears at the top of the command
    /// field for one interaction ("in Profile · <title>"). `None` at rest;
    /// same-profile switching never sets it; the next chrome context
    /// mutation clears it. Distinct from `command_bar_hint` (the staged-move
    /// hint); the two never coexist.
    pub command_profile_hint: Option<String>,
    /// The dragged tab row's active-space index (§10.9). `None` when no drag
    /// is in flight; the dragged row renders the `dragging` state.
    pub drag_source: Option<usize>,
    /// The current drag target: the tab insertion slot (within the source
    /// region) or the targeted space's stable id.
    pub drag_slot: Option<usize>,
    pub drag_space: Option<TestSpaceId>,
    /// Whether the current drag target is a legal drop (a boundary crossing
    /// or a non-droppable zone reports `false` and renders the muted tick).
    pub drag_valid: bool,
    /// §11.3 space states (selected/default) in render order, grouped by
    /// profile.
    pub space_states: Vec<SpaceVisualState>,
    /// §11.3 space row titles in render order, after §4 truncation.
    pub space_titles: Vec<String>,
    /// The stable `bbr-*` ids present in the current render (DESIGN.md §15).
    pub visible_ids: Vec<String>,
    /// The typed semantic tree of the current render (DESIGN.md §15 /
    /// accessibility track): roles, names, values, enabled/selected/focused
    /// state, and hierarchy. Internal model — GPUI 0.2.2 has no platform
    /// accessibility layer, so these are not macOS Accessibility nodes; the
    /// platform bridge is Phase 9.
    pub semantics: Vec<SemanticNode>,
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
    /// The authoritative pane layout (primary/secondary/focused ids), or
    /// `None` when no live pane is rendered (empty/crashed state).
    pub pane_layout: Option<PaneLayoutSnapshot>,
    /// Keyed per-pane native geometry in pane-layout order (primary, then
    /// secondary). Empty in single-pane mode only when the pane has not yet
    /// published a frame; the focused pane's entry equals `content_rect`.
    pub pane_rects: Vec<PaneRectSnapshot>,
    /// Window content size in points (diagnostics only).
    pub window_size: [f32; 2],
}
