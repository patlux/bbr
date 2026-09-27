//! The typed `Snapshot` and its row/space/profile parts, plus the §15
//! visual-state vocabulary (DESIGN.md §15). All plain data with `serde`
//! derives so the wire format never leaks framework types.

use bbr_web::{ProfileId, WebViewId};
use serde::{Deserialize, Serialize};

use super::{CommandBarMode, ShellModes, TestProfileId, TestSpaceId, TestWebViewId};

mod card;
mod u2;
pub use card::ContentCardSnapshot;
pub use u2::{
    ChipCompanionSnapshot, FavoriteTileSnapshot, FolderRowSnapshot, SpaceAppearanceSnapshot,
    SpaceChipSnapshot, SpaceConfigSnapshot, SpaceStripScrollSnapshot, SwipeSnapshot,
};

/// The L6 contract state of one download attempt (DESIGN.md §15): the panel
/// renders exactly these states and never invents a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadMachineState {
    /// The attempt was accepted; no bytes have flowed yet (destination
    /// unresolved or resolved).
    Waiting,
    /// Bytes are transferring to the resolved destination.
    Running,
    /// The transfer finished successfully (terminal).
    Succeeded,
    /// The transfer ended in a closed `DownloadFailure` (terminal; the
    /// retryable state).
    Failed,
    /// The attempt was cancelled on request (terminal; restartable per the
    /// settled §17.8 decision).
    Cancelled,
}

/// The panel group a row renders in (DESIGN.md §11.12): non-terminal
/// attempts above, terminal attempts below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadGroup {
    Active,
    History,
}

/// The exactly-one contextual primary action of a download row (DESIGN.md
/// §11.12): active → Cancel, failed → Retry, succeeded → Reveal-in-Finder,
/// cancelled → Restart (the settled §17.8 decision).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadRowAction {
    Cancel,
    Retry,
    /// The settled cancelled-row action (DESIGN.md §17.8): a fresh attempt
    /// linked via `retry_of` — the same machine path as a failed retry,
    /// with the panel label "Restart".
    Restart,
    /// Conventional macOS "Show in Finder"; enabled only when the terminal
    /// record's resolved path exists (§10.8 `disabled` with a reason
    /// otherwise).
    Reveal,
    /// No primary action.
    None,
}

/// One download row on the wire (DESIGN.md §15): the L6 machine state, the
/// panel group, the deterministic row index `<n>`, the byte progress
/// fraction, the §10 visual state, the contextual action with its enabled
/// flag, the secondary-line copy, and the retry linkage/attempt count. The
/// resolved path is exposed so E2E can assert the exact destination and the
/// disabled-reveal case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadRowSnapshot {
    /// The stable panel row index `<n>` (`bbr-download-row-<n>`): Active by
    /// ascending `DownloadId`, then History most-recent-terminal first.
    pub row: usize,
    /// The attempt id (fresh per retry; never reused).
    pub id: u64,
    /// The L6 contract state (DESIGN.md §15 vocabulary).
    pub machine_state: DownloadMachineState,
    /// The panel group the row renders in.
    pub group: DownloadGroup,
    /// The display filename (the panel's name line).
    pub filename: String,
    /// `received / total` when the total is known; `None` while the length
    /// is unknown (indeterminate bar). A zero-byte download reports `1.0`.
    pub progress_fraction: Option<f64>,
    /// The §10 visual state of the row.
    pub visual_state: TabVisualState,
    /// The exactly-one contextual primary action.
    pub action: DownloadRowAction,
    /// Whether the action accepts interaction (Reveal-in-Finder is enabled
    /// only for an existing resolved terminal path).
    pub action_enabled: bool,
    /// The disabled-action reason copy when `!action_enabled` (e.g. "file
    /// missing"; §10.8 tooltip deferred with §17.2).
    pub action_reason: Option<String>,
    /// The secondary-line copy (`type:meta`, `tnum`): active — transferred/
    /// total bytes; terminal — size + status copy naming the outcome.
    pub meta: String,
    /// The resolved destination path once the engine decided it.
    pub resolved_path: Option<String>,
    /// The failed attempt this attempt retries, when applicable (retry
    /// linkage evidence).
    pub retry_of: Option<u64>,
    /// 1-based attempt count (increments per retry).
    pub attempt: u32,
    /// The row offers the Delete control (DESIGN.md §17.8 settled: a
    /// cancelled row offers Restart **and** Delete — the one-row durable
    /// removal). False for every other state.
    pub delete_available: bool,
}

impl Default for DownloadRowSnapshot {
    fn default() -> Self {
        Self {
            row: 0,
            id: 0,
            machine_state: DownloadMachineState::Waiting,
            group: DownloadGroup::Active,
            filename: String::new(),
            progress_fraction: None,
            visual_state: TabVisualState::Default,
            action: DownloadRowAction::None,
            action_enabled: false,
            action_reason: None,
            meta: String::new(),
            resolved_path: None,
            retry_of: None,
            attempt: 1,
            delete_available: false,
        }
    }
}

/// The downloads panel render contract (DESIGN.md §11.12): the open flag,
/// the keyboard focus row, the deterministic scroll anchor, the bounded
/// rendered window (never the full history), the total row count, the
/// 6-row viewport, and the §11.12 `max_visible` bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadsPanelSnapshot {
    /// The anchored popover is open.
    pub open: bool,
    /// The keyboard-cursor row (`None` while closed; no persistent
    /// selection — focus is the cursor).
    pub focused_row: Option<usize>,
    /// First row at the scroll viewport top (deterministic scroll position).
    pub anchor: usize,
    /// How many rows the last render actually built (bounded by the window).
    pub rendered_rows: usize,
    /// The full deterministic row list length (active + history).
    pub total_rows: usize,
    /// The scroll viewport's row count (`min(6, total)`).
    pub viewport_rows: usize,
    /// The §11.12 bound: at most this many rows are visible at once.
    pub max_visible: usize,
    /// The §11.4 footer `[Downloads]` `tnum` badge count (DESIGN.md §17.11
    /// settled): **active (in-flight) attempts only** — terminal history
    /// never inflates the badge and the count returns to zero when nothing
    /// transfers.
    pub footer_count: usize,
}

impl Default for DownloadsPanelSnapshot {
    fn default() -> Self {
        Self {
            open: false,
            focused_row: None,
            anchor: 0,
            rendered_rows: 0,
            total_rows: 0,
            viewport_rows: 0,
            max_visible: 6,
            footer_count: 0,
        }
    }
}

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
    /// §10.11: the session's renderer stopped answering liveness probes
    /// (fail-closed; the session and tab stay alive under the §11.8a
    /// recovery surface).
    Unresponsive,
    Disabled,
    /// §10.9: the dragged row (92% opacity, elev:2 shadow, strong outline).
    Dragging,
    /// §10.9: the drop target (accent tick + target outline + tint; the
    /// space target replaces the tick with the destination profile chip).
    DragTarget,
    Focused,
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
    /// Creation-order index — the collection-order key (the strip chip ids
    /// derive from the persistent [`TestSpaceId`], never this index).
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
    /// The tab's renderer stopped answering liveness probes (DESIGN.md
    /// §10.11/§11.8a): a sibling of `crashed` — the session and tab stay
    /// alive under the bounded recovery surface.
    pub unresponsive: bool,
    pub active: bool,
    pub web_view: Option<WebViewId>,
    /// Resource policy state of the tab's disposable session (the dense-list
    /// lifecycle contract): "active" | "warm" | "suspended" | "discarded".
    pub lifecycle: String,
    /// Deterministic recency stamp (the session-budget policy's "recently
    /// used" order).
    pub last_used: u64,
    /// §7.2/§15 favicon presentation of the tab's tile (task packet
    /// `real-favicon-pipeline.md`): `"monogram"` until the owning profile's
    /// bounded host-keyed store holds resolved bytes for the tab's host,
    /// then `"image"`. The typed E2E evidence that a real favicon reached
    /// the rendered slot.
    pub favicon: String,
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
    /// A keyboard-focusable list (`bbr-space-strip` / `bbr-tab-list`).
    List,
    /// A selectable row (`bbr-tab-row-*` / `bbr-space-chip-*`).
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
    /// The §11.4 organized-regions hairline (`bbr-organized-hairline`):
    /// the 1 px structural rule between the organized Pinned/Folders area
    /// and the New Tab row.
    Separator,
    /// The glass content area: native slot, empty pane, or crash panel.
    Content,
    /// A row's favicon / monogram tile (`bbr-tab-favicon-*`).
    Favicon,
    /// A row's title text (`bbr-tab-title-*` / `bbr-space-identity-*`).
    Title,
    /// A row's count column (`bbr-folder-count-*`).
    Count,
    /// A row's trailing state slot (`bbr-tab-state-*`).
    State,
    /// The command-bar cursor (`bbr-command-cursor`).
    Cursor,
    /// A §11.5 command result row (`bbr-command-result-*`) inside the result
    /// list.
    Result,
    /// A §11.4 command-surface presentation — the centered Cmd-T palette
    /// (`bbr-palette`) or the field-anchored Cmd-L/click overlay
    /// (`bbr-command-overlay`) — both surfaces of the same §11.5 engine.
    Palette,
    Grid,
    Tile,
    Menu,
    MenuItem,
    Folder,
    Disclosure,
    Footer,
}

/// The open command-surface presentation (DESIGN.md §11.4): at most one
/// command surface is open at a time — the centered Cmd-T palette or the
/// field-anchored Cmd-L / field-click overlay. `None` when no command
/// surface is open. Opening either clears the §11.3 profile hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandSurface {
    /// No command surface is open.
    #[default]
    None,
    /// The centered Cmd-T palette (`bbr-palette`) in New Tab staging mode.
    Palette,
    /// The field-anchored Cmd-L / field-click overlay (`bbr-command-overlay`)
    /// in Location mode.
    Overlay,
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
    /// §10.9: whether the element is the current drag target (a space chip
    /// under the pointer renders the drag-target treatment plus its §12
    /// destination-profile companion). Derived from the same drag state the
    /// render and the snapshot consume.
    #[serde(default)]
    pub drag_target: bool,
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

/// §11.5 address rest fill — the field's rest surface is a quiet bordered
/// field one step off the rail: a white veil over the (tinted) rail plus a
/// hairline-strength border, per chrome polarity. The variant reports the
/// effective polarity of the §11.5 token set so E2E asserts the rest state
/// in both appearance modes semantically (no pixel reads).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressRestFill {
    /// Bordered field on dark chrome (`palette.address_fill` +
    /// `palette.address_border`).
    #[default]
    BorderedFieldOnDark,
    /// Bordered field on light chrome.
    BorderedFieldOnLight,
}

/// §11.14 Reload rest surface — the control rests with no surface
/// (transparent); it gains only the §10.2 hover wash and the §10.3 pressed
/// wash on interaction. Reported so E2E proves the transparent rest state
/// semantically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReloadRestSurface {
    #[default]
    Transparent,
}

/// §11.14 the native safe-titlebar layout guide as plain wire data (task
/// packet `titlebar-navigation-controls`): the titlebar strip height, the
/// traffic-light region (window coordinates, top-left origin), the sidebar
/// collapse state, the genuine navigation availability of the focused
/// pane, and the typed §11.5/§11.14 rest-state evidence (address
/// translucent borderless fill, Reload transparent rest, real
/// hovered/pressed presentation state). All values are what the row
/// actually renders (the app reads the live window; nothing is
/// hardcoded).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TitlebarSnapshot {
    /// The native titlebar strip height in points (0 before the window
    /// attaches / on non-macOS hosts).
    pub safe_area_height: f32,
    /// The traffic-light region `[left, top, width, height]` in window
    /// coordinates (top-left origin): the sidebar toggle starts after its
    /// right edge.
    pub traffic_lights: [f32; 4],
    /// The sidebar is collapsed: the toggle glyph and the semantic value
    /// reflect it, and the whole titlebar disappears with the rail.
    pub sidebar_collapsed: bool,
    /// The focused pane's real history availability (Back).
    pub can_go_back: bool,
    /// The focused pane's real history availability (Forward).
    pub can_go_forward: bool,
    /// Reload is enabled while the focused pane has a live session.
    pub can_reload: bool,
    /// §11.5 rest evidence: the address pill's translucent borderless fill
    /// token pair, reported by effective chrome polarity (follows the
    /// §15 theme mode; asserted in both appearance modes).
    pub address_rest_fill: AddressRestFill,
    /// §11.14 rest evidence: the Reload control's rest surface is
    /// transparent — no raised surface, §10.2/§10.3 washes only on
    /// interaction.
    pub reload_rest: ReloadRestSurface,
    /// §11.14 presentation state: the pointer hovers the Reload control
    /// (§10.2 wash active). Real titlebar-model state, not synthesized.
    pub reload_hovered: bool,
    /// §11.14 presentation state: the primary button is down on Reload
    /// (§10.3 pressed wash active). Real titlebar-model state.
    pub reload_pressed: bool,
}

impl Default for TitlebarSnapshot {
    fn default() -> Self {
        Self {
            safe_area_height: 0.0,
            traffic_lights: [0.0; 4],
            sidebar_collapsed: false,
            can_go_back: false,
            can_go_forward: false,
            can_reload: false,
            address_rest_fill: AddressRestFill::BorderedFieldOnDark,
            reload_rest: ReloadRestSurface::Transparent,
            reload_hovered: false,
            reload_pressed: false,
        }
    }
}

/// The typed state of the command result surface (DESIGN.md §11.5):
/// `Inactive` when no ranking surface is open, `Empty`/`Loading`/
/// `Results`/`ActivationError` while the location/new-tab command bar
/// is editing. There is no production no-match state: every non-empty
/// Location/NewTab query carries the navigate/search fallback row, so
/// the unreachable `NoMatch` variant was removed from the wire. Exposed
/// so E2E asserts the result state machine without stringly-typed
/// labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandResultState {
    /// The command bar is not ranking (no location/new-tab surface).
    #[default]
    Inactive,
    /// The ranking surface is open with no results (empty text).
    Empty,
    /// A debounce generation is pending; ranking has not published yet.
    Loading,
    /// The latest published generation produced ranked results.
    Results,
    /// The last activation was rejected (unavailable destination).
    ActivationError,
}

/// One persistent tab identity across spaces (DESIGN.md §11.5 cross-space
/// command composition): the wire mirror of the controller's global tab
/// snapshot. E2E asserts exact tab/space/profile identity, warm-view reuse,
/// and profile-store isolation of other-space command results through it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalTabSnapshot {
    /// The persistent domain tab id (the `tab:<id>` command-result key).
    pub id: u64,
    /// The owning space id.
    pub space: TestSpaceId,
    /// The owning profile id (the profile-store isolation evidence).
    pub profile: u64,
    /// Position in the owning space's tab list.
    pub index: usize,
    pub url: String,
    pub title: String,
    pub pinned: bool,
    pub web_view: Option<WebViewId>,
    /// "active" | "warm" | "suspended" | "discarded".
    pub lifecycle: String,
    pub last_used: u64,
}

/// One ranked command-bar result on the wire (DESIGN.md §11.5): stable
/// key, priority group, display title/subtitle, selection state, and the
/// destructive flag that drives the error/destructive tint. `selected`
/// mirrors the palette's selection exactly (Up/Down move it without
/// leaving the field).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResultSnapshot {
    /// The stable result key (`tab:<n>` / `space:<n>` / `action:<n>` /
    /// `history:<n>` / `navigate:<url>` / `search:<terms>`).
    pub key: String,
    /// The Phase-5 priority group name (exact_tab / current_pinned /
    /// current_tab / other_space / space / history / action / fallback).
    pub group: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub selected: bool,
    /// True for destructive actions; the row renders the error/destructive
    /// visual language (DESIGN.md §11.5 settled value: tint only, no
    /// confirmation dialog).
    pub destructive: bool,
}

/// Typed diagnostics of the one native portal host (task packet
/// `native-portal-host`, DESIGN.md §13/§15). Exposes only the active
/// presentation: when no presentation is active every state field reports
/// the empty/closed host (logical visibility false, no rect, portal ordered
/// out). On non-macOS hosts the native fields stay in their default state;
/// the portable logical fields (presentation/host/visible/rect/ids) still
/// report the active request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PortalSnapshot {
    /// The active presentation's stable id (`None` when none is active).
    pub presentation: Option<u64>,
    /// The active presentation's host label (`gpui-deferred` /
    /// `native-above-web`), or `None`.
    pub host: Option<String>,
    /// Logical visibility commanded by the app (the probe surface state).
    pub visible: bool,
    /// The resolved portal rect in window coordinates `[x, y, w, h]`
    /// (points, top-left origin). `None` when no presentation is active.
    pub rect: Option<[f32; 4]>,
    /// The stable `bbr-portal-*` ids the portal surface renders while
    /// visible (DESIGN.md §15); empty while hidden.
    pub ids: Vec<String>,
    /// The native portal window is attached as a child of the browser
    /// window (macOS host evidence).
    pub child_attached: bool,
    /// The AppKit window's actual `isVisible` state.
    pub native_visible: bool,
    /// The portal frame is pinned to the parent content rect (parent-
    /// content/portal frame synchronization evidence).
    pub frame_synced: bool,
    /// The borderless non-activating-panel window policy is applied to the
    /// portal window (it can never become key/main merely by showing).
    pub nonactivating_panel: bool,
    /// The browser window's AppKit window number (0 when no host is
    /// attached).
    pub parent_window_number: i64,
    /// The portal window's AppKit window number (0 when no host is
    /// attached).
    pub portal_window_number: i64,
    /// The AppKit window number frontmost at the probe point (the centered
    /// probe card's point). Absolute cross-application query — on a locked
    /// desktop or with another app covering the point it reports that
    /// window. The deterministic z-order evidence is
    /// [`PortalSnapshot::below_portal_at_probe_point`].
    pub frontmost_at_probe_point: Option<i64>,
    /// Measured owner classification of
    /// [`PortalSnapshot::frontmost_at_probe_point`]: whether that window
    /// number belongs to this process (membership-tested against
    /// `NSApplication.windows` on the macOS host). `false` when no window is
    /// frontmost or when the frontmost window belongs to another
    /// application (a foreign overlay covering the probe point). Lets the
    /// harness distinguish a provably foreign overlay from a BBR-owned
    /// window without dropping the raw absolute query.
    #[serde(default)]
    pub frontmost_at_probe_point_is_this_app: bool,
    /// The AppKit window number frontmost at the probe point strictly BELOW
    /// the portal window. Raw diagnostic only (all-spaces/PopUp windows can
    /// answer with an unrelated overlay); the deterministic hit-test
    /// evidence is [`PortalSnapshot::portal_owns_probe_point`].
    pub below_portal_at_probe_point: Option<i64>,
    /// Deterministic AppKit hit-test evidence at the probe point: the portal
    /// is an attached child window ordered above its parent, visible, pinned
    /// to the parent content rect (covering the point), and never key/main.
    /// AppKit hit-tests child windows above their parent's views, so when
    /// true a click at the probe point is routed to the portal window —
    /// never to the WKWebView underneath. Environment-independent.
    pub portal_owns_probe_point: bool,
    /// The non-key policy holds: the portal window is never key merely by
    /// showing.
    pub portal_is_key: bool,
    /// The non-main policy holds: the portal window is never main merely by
    /// showing.
    pub portal_is_main: bool,
    /// The parent browser window is currently in native fullscreen
    /// (native-portal-hardening evidence; `NSWindowStyleMaskFullScreen`
    /// measured from the parent `NSWindow`). Fullscreen enter/leave must
    /// re-pin the portal frame and never recreate or orphan the host.
    #[serde(default)]
    pub parent_fullscreen: bool,
    /// The parent browser window is currently miniaturized
    /// (native-portal-hardening evidence; `isMiniaturized`). While
    /// minimized the portal must be ordered out — never occluding or
    /// intercepting — and re-shown on restore when a presentation is armed.
    #[serde(default)]
    pub parent_minimized: bool,
    /// The parent browser window is associated with the active space
    /// (real Window Server `isOnActiveSpace` query). Hardening invariant:
    /// the portal and its parent must always agree on active-space
    /// membership.
    #[serde(default)]
    pub parent_on_active_space: bool,
    /// The portal window is associated with the active space (same query).
    #[serde(default)]
    pub portal_on_active_space: bool,
    /// The portal window's backing scale factor (real display query).
    /// Frame sync is point-based, so a display with a different scale
    /// factor keeps the portal pinned in points (scale only changes the
    /// backing pixels).
    #[serde(default)]
    pub portal_scale_factor: f64,
}

/// Typed credential-sheet presentation state (queue item 10, P8c
/// Auth-Adapter; DESIGN.md §15). Mirrors the app's redacted auth snapshot:
/// the presented challenge's identity and authority metadata, its contract
/// lifecycle state, the redacted decision representation (the contract's
/// redaction: `credential:#<n>`, `cancelled`, `superseded:#<n>`, …), the
/// focused field, and masked entered flags. By construction no field can
/// hold a credential: the typed values never leave the sheet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthSnapshot {
    /// A credential sheet is currently presented.
    pub open: bool,
    /// The presented challenge's stable id.
    pub challenge: Option<u64>,
    /// The owning web view of the presented challenge.
    pub web_view: Option<WebViewId>,
    /// The normalized host the challenge belongs to (authority metadata,
    /// never a credential).
    pub host: String,
    /// The realm label, when the challenge carried one.
    pub realm: Option<String>,
    /// The closed HTTP scheme name (`Basic`/`Digest`/`NTLM`/`Negotiate`).
    pub scheme: String,
    /// `origin` or `proxy`.
    pub authority: String,
    /// The presented challenge's contract lifecycle state label.
    pub state: String,
    /// The redacted decision representation (the contract's redaction) of
    /// the presented challenge's terminal state, or of the last decided
    /// challenge after the sheet closed (`credential:#<n>`, `cancelled`,
    /// `superseded:#<n>`, …).
    pub outcome: Option<String>,
    /// Which sheet field owns the keyboard cursor.
    pub focused_field: Option<String>,
    /// Masked: whether the username field holds any characters.
    pub username_entered: bool,
    /// Masked: whether the password field holds any characters.
    pub password_entered: bool,
    /// Challenge ids recorded but not yet presented (FIFO awaits).
    pub awaiting: Vec<u64>,
    /// The theme mode (`light`/`dark`) the portal render last drew the sheet
    /// in — render evidence that the sheet follows the shell appearance.
    /// `None` until the sheet has been presented.
    pub presented_theme: Option<String>,
}

/// Typed find-in-page presentation state (task packet
/// `find-in-page-portal-bar`; DESIGN.md §15). Mirrors the app's
/// `FindState` exactly: open presentation, field focus, the live query, the
/// WebKit-reported match count/ordinal, a pending request, and the stale
/// marker after a document change.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FindSnapshot {
    /// A find presentation is open for this window.
    pub open: bool,
    /// The find field owns keyboard focus (main-window GPUI focus; the
    /// portal window is never key by design).
    pub focused: bool,
    /// The live query text (what the field renders).
    pub query: String,
    /// Total matches reported by WebKit (`None` until a result lands).
    pub match_count: Option<usize>,
    /// Current 1-based ordinal (`None` for zero matches / no result).
    pub current_match: Option<usize>,
    /// A start/step request is awaiting a WebKit completion.
    pub pending: bool,
    /// A document change made the reported results stale.
    pub stale: bool,
}

/// Real WebKit content evidence (testkit-only gateway): the committed
/// document read in BBR's own client content world — `readyState`, `URL`,
/// `title`, body presence, computed root/body backgrounds — plus the native
/// `WKWebView` `url`/`title`
/// properties sampled at the same point in time. The shell's visibility
/// flags, geometry, and session bookkeeping can be perfectly green while
/// the user sees a blank content area (the schemeless Cmd-T destination
/// bug); this probe reports what WebKit actually committed, never our own
/// accounting. The native `url` reflects a provisional navigation
/// immediately, so "a navigation was accepted" is observable even before a
/// document commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebContentProbe {
    /// The view the probe was issued against (stale-guard: the snapshot
    /// holds the last completed probe, so E2E asserts the probed view).
    pub view: WebViewId,
    /// `document.readyState` ("loading" | "interactive" | "complete").
    pub ready_state: String,
    /// `document.URL` — the committed document's URL; "about:blank" for a
    /// fresh/blank view.
    pub dom_url: String,
    /// `document.title`.
    pub dom_title: String,
    /// A document body exists in the DOM.
    pub has_body: bool,
    /// Computed `background-color` of the document root.
    pub root_background_color: String,
    /// Computed `background-color` of the body, or empty when no body exists.
    pub body_background_color: String,
    /// The native `WKWebView.url` (provisional or committed; `None` when no
    /// navigation has been accepted at all).
    pub native_url: Option<String>,
    /// The native `WKWebView.title`.
    pub native_title: Option<String>,
}

/// The error-surface category on the wire (task packet
/// `error-surface-contract`, DESIGN.md §11.8): the closed contract category
/// the classified incident carries. The process-crash class is **not** a
/// wire variant here: a crashed tab is presented by the §11.8 crash panel
/// (the controller-driven `bbr-crash-*` surface), so the error surface
/// only ever presents TLS, navigation, and offline classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSurfaceCategory {
    Tls,
    Navigation,
    Offline,
}

/// The §11.8-style error-surface presentation (task packet
/// `error-surface-contract`): the classified incident the content area
/// renders — the contract category, the privacy-safe failing origin, the
/// calm title/copy the surface drew, the contract-legal primary action, and
/// the lifecycle state. The empty default means no error surface is shown.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ErrorSurfaceSnapshot {
    /// The surface is rendered in the content area.
    pub visible: bool,
    /// The web view the incident is scoped to (unchanged by showing /
    /// dismissing — identity is never touched).
    pub view: Option<WebViewId>,
    /// The closed contract category of the classified incident.
    pub category: Option<ErrorSurfaceCategory>,
    /// The contract subreason, e.g. "navigation: server not found".
    pub reason: Option<String>,
    /// The privacy-safe failing origin the surface presents readably
    /// (scheme://host[:port]); `None` when the origin is unknown.
    pub origin: Option<String>,
    /// The calm title the surface drew (never a raw platform code).
    pub title: Option<String>,
    /// The primary recovery action label the surface drew (contract-legal
    /// for the category).
    pub action: Option<String>,
    /// The contract lifecycle state (reported / recovery requested /
    /// recovery failed).
    pub lifecycle: Option<String>,
}

/// Per-site zoom state (queue item 11, part A; task packet
/// `site-settings-zoom-contract`). Mirrors the app's
/// `crate::app::zoom::ZoomSnapshotState`: the effective level of the active
/// view's site, the normalized origin, the view the level applies to
/// (identity-preservation evidence), the level actually applied to that
/// native view, and whether a persistence write is pending.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ZoomSnapshot {
    /// Effective zoom percent of the active view's site (the documented
    /// default 100 when no site is resolvable yet).
    pub level: u16,
    /// The normalized origin (`scheme://host:port`) the level belongs to,
    /// when resolvable.
    pub site: Option<String>,
    /// The view the level applies to (identity-preservation evidence).
    pub view: Option<WebViewId>,
    /// The level actually applied to that native view (`None` when the view
    /// has no live session or no site was resolved yet).
    pub applied: Option<u16>,
    /// A persistence write is scheduled but not yet acknowledged.
    pub pending: bool,
}

/// One BBR page-background menu entry on the wire (task packets
/// `bbr-context-menus-over-web` and `page-background-context-menu`): the
/// stable key, the stable per-action §15 id, the display title, the
/// genuine enablement (Back/Forward reflect the real session history
/// availability from `CanGoBackChanged`/`CanGoForwardChanged`; Copy Page
/// Link reflects a resolvable page URL; the bookmark row reflects B2's
/// saved state), and the deterministic disabled reason (DESIGN §11.13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextMenuItemSnapshot {
    /// Stable key: `back` | `forward` | `reload` | `bookmark` |
    /// `copy-link`.
    pub key: String,
    /// The stable per-action §15 id (`bbr-page-menu-back`, …).
    pub id: String,
    /// The display title (Back / Forward / Reload / Bookmark Page or
    /// Remove Bookmark / Copy Page Link).
    pub title: String,
    /// Whether the entry accepts interaction right now.
    pub enabled: bool,
    /// The deterministic reason a disabled row reports (`None` for enabled
    /// rows).
    pub disabled_reason: Option<String>,
}

/// Typed BBR page-background menu presentation state (task packets
/// `bbr-context-menus-over-web` and `page-background-context-menu`;
/// DESIGN.md §13/§15). The menu renders inside the one `NativeAboveWeb`
/// portal window; this mirrors the app's `ContextMenuState` exactly plus
/// the native bridge's ownership diagnostics.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ContextMenuSnapshot {
    /// The menu presentation is open for this window.
    pub open: bool,
    /// The menu owns keyboard focus (main-window GPUI focus; the portal
    /// window is never key by design).
    pub focused: bool,
    /// The keyboard cursor / hovered row (None while closed).
    pub selected: Option<usize>,
    /// The deterministic row model in render order.
    pub items: Vec<ContextMenuItemSnapshot>,
    /// The web view the menu was requested on (the active view at open;
    /// identity-preservation evidence).
    pub bound_view: Option<WebViewId>,
    /// The pointer anchor in window content coordinates (top-left origin,
    /// points) the menu placement resolved from.
    pub point: [f32; 2],
    /// The page URL the menu acts on (Copy Page Link / the bookmark row);
    /// `None` when no URL is resolvable (Copy Page Link then reports
    /// disabled and the bookmark row is omitted).
    pub url: Option<String>,
    /// The captured page title at open (the bookmark row invokes B2's one
    /// app-owned command against the focused page's URL/title).
    pub title: Option<String>,
    /// The focus domain captured at open; dismissal restores it exactly
    /// (focus-return evidence).
    pub origin_chrome_focused: bool,
    /// The native bridge's ownership diagnostics: the right-click monitor
    /// is installed and classifies WebKit's page vs element menus
    /// (page-owned requests the BBR menu; element-owned stays native).
    #[serde(default)]
    pub native: ContextMenuNativeSnapshot,
}

/// The native context-menu bridge diagnostics (task packet
/// `bbr-context-menus-over-web`; the engine exposes them through
/// `WebEngineWk::context_menu_diagnostics`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ContextMenuNativeSnapshot {
    /// The bridge's local right-click monitor is installed.
    pub monitor_installed: bool,
    /// Right-clicks observed over a live WKWebView (all of them: every
    /// WebKit menu is classified, never suppressed blindly).
    pub right_clicks_over_web: u64,
    /// Classified as the generic page menu → the BBR menu was requested.
    pub page_owned: u64,
    /// Classified as element-owned → WebKit's native menu stayed untouched.
    pub element_owned: u64,
}

/// One typed Tab/Favorite context-menu row on the wire (task packet
/// `tab-favorite-context-menus`): the stable §15 id, the display label,
/// the genuine enablement computed at open, the danger flag, and the
/// optional one-level submenu rows (folders / spaces). The wire carries no
/// `TabId` — source identity lives only in the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabMenuRowSnapshot {
    /// The stable §15 id (`bbr-tab-menu-<action>`,
    /// `bbr-tab-menu-folder-<n>`, `bbr-tab-menu-space-<n>`,
    /// `bbr-tab-menu-bookmark`, `bbr-tab-menu-close-<scope>`).
    pub id: String,
    /// The display label (`Pin Tab`, `Move to Space`, …).
    pub label: String,
    /// Whether the entry accepts interaction right now.
    pub enabled: bool,
    /// The danger state (text/glyph companion required).
    pub danger: bool,
    /// The optional one-level submenu rows (folders / spaces).
    #[serde(default)]
    pub submenu: Vec<TabMenuRowSnapshot>,
}

/// Typed Tab/Favorite context-menu presentation state (task packet
/// `tab-favorite-context-menus`; DESIGN.md §11.13/§15). The menu is
/// ordinary GPUI chrome over the sidebar (never `NativeAboveWeb`); this
/// mirrors the app's `TabMenuState` exactly — source identity, the
/// keyboard/pointer cursor, the open submenu, the deterministic row model,
/// and the resolved clamped placement.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TabMenuSnapshot {
    /// The menu presentation is open for this window.
    pub open: bool,
    /// The menu owns keyboard focus (the main-window menu focus handle).
    pub focused: bool,
    /// The exact persistent `TabId` the menu is bound to (identity
    /// evidence: the source never changes on open/traverse/dismiss).
    pub bound_tab: Option<u64>,
    /// The stable presentation source key (`bbr-tab-menu` |
    /// `bbr-favorite-menu`).
    pub source_key: String,
    /// The keyboard cursor / hovered top-level row (None while closed).
    pub selected: Option<usize>,
    /// The top-level row whose submenu card is currently open.
    pub open_submenu: Option<usize>,
    /// The keyboard cursor / hovered row inside the open submenu.
    pub submenu_selected: Option<usize>,
    /// The deterministic top-level row model in render order.
    pub rows: Vec<TabMenuRowSnapshot>,
    /// The resolved card placement in window content coordinates
    /// `[x, y, width, height]` (points, top-left origin; clamped).
    pub anchor: [f32; 4],
    /// The resolved submenu card placement (same coordinate space), when a
    /// submenu is open.
    pub submenu_rect: Option<[f32; 4]>,
}

/// Productive new-window seam diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NewWindowSnapshot {
    pub requests: u64,
    pub created_tabs: u64,
    /// Regular children created by the explicit background disposition
    /// (Command-primary / middle click; background link tabs packet).
    #[serde(default)]
    pub created_background_tabs: u64,
    pub blocked_reported: u64,
}

/// Web-content navigation gesture diagnostics (N1): how the app classified
/// each committed logical gesture, plus the live child→opener provenance
/// pairs (`(child TabId, opener TabId)`, stable identities, never indices).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContentGestureSnapshot {
    /// Commits received from the native bridge or testkit injection.
    pub commits: u64,
    /// Commits that dispatched exactly one real history action.
    pub history_routed: u64,
    /// Commits that ran the atomic Back-at-root opener return.
    pub opener_returns: u64,
    /// Commits that were deterministic no-ops (disabled direction, stale
    /// view, missing/stale opener, provisional load, hidden target).
    pub no_ops: u64,
    /// Live session-local `(child, opener)` identity pairs.
    pub opener_relations: Vec<(u64, u64)>,
}

/// The transient Peek preview snapshot (Link Preview / Peek packet).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewSnapshot {
    /// The typed transient preview identity (distinct from `TabId`/`WebViewId`).
    pub id: u64,
    /// The disposable source-profile preview session.
    pub view: WebViewId,
    /// The covered warm origin session (same `WebViewId`, URL, and page
    /// state before/after the preview).
    pub source: WebViewId,
    /// The source profile whose website data store backs the preview.
    pub profile: ProfileId,
    pub url: String,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// The preview's web process terminated; only a live session can be
    /// promoted without recreation.
    pub crashed: bool,
}

/// Truthful native z-order evidence (Link Preview / Peek packet): the
/// AppKit subview indices of the preview and its covered origin (a higher
/// index draws above a lower one).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewZOrder {
    pub preview_index: usize,
    pub source_index: usize,
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
    /// The native dark cover is masking WebKit's provisional-navigation plane.
    #[serde(default)]
    pub navigation_cover_visible: bool,
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
    /// DESIGN §2/§18.1 D2: the §15 ids of every rail surface that renders
    /// the static `elev:lift` — the selected tab row (`bbr-tab-row-<n>`), a
    /// selected folder member (`bbr-folder-item-<n>-<m>`) and the active
    /// space chip (`bbr-space-chip-<space>`) — resolved by the same
    /// `row_lift`/member selection/`chip_lifts` the render paints with.
    #[serde(default)]
    pub lifted_ids: Vec<String>,
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
    /// region). `None` when the drag targets the favorites grid, a chip, or
    /// nothing (an invalid no-target zone).
    pub drag_slot: Option<usize>,
    /// §10.9/§11.10: whether the in-flight drag currently targets the
    /// favorites grid (a drop toggles the dragged tab in/out of the
    /// window-global favorites). Typed alongside `drag_slot`/`drag_chip`
    /// so a favorites-targeted drag is asserted by state, not by geometry
    /// guessing.
    #[serde(default)]
    pub drag_favorites: bool,
    /// The §11.10 favorites region's rendered window-relative rect
    /// `[x, y, width, height]`, recorded by the production band anchor
    /// (the exact drop-band span the resolver hits). `None` while no band
    /// renders (no tiles and no in-flight drag).
    #[serde(default)]
    pub favorites_rect: Option<[f32; 4]>,
    /// The current drag target's persistent space chip (DESIGN.md §11.3):
    /// the exact `bbr-space-chip-<persistent-space-id>` the pointer hovers
    /// during a drag-to-chip. `None` when the drag targets a tab slot, the
    /// favorites grid, or nothing (an invalid zone outside chips/list/
    /// favorites). The identity is the persistent `SpaceId`, never a render
    /// position.
    #[serde(default)]
    pub drag_chip: Option<TestSpaceId>,
    /// Whether the current drag target is a legal drop (a boundary crossing
    /// or a non-droppable zone reports `false` and renders the muted tick).
    pub drag_valid: bool,
    /// §11.3 permanent space strip: one chip per space in canonical display
    /// order (the vector order is display order; the chip/identity ids
    /// derive from the persistent `space` id). The strip is always part of
    /// the rail.
    #[serde(default)]
    pub space_chips: Vec<SpaceChipSnapshot>,
    /// §11.3 resolved appearance evidence of the active space: effective
    /// theme/identity names plus the resolved chrome/chip tint hex the
    /// render paints. `None` only in the invariant-forbidden empty browser;
    /// every real active space resolves an appearance.
    #[serde(default)]
    pub active_appearance: Option<SpaceAppearanceSnapshot>,
    /// §11.3/§11.4 strip scroll evidence: the strip viewport, the signed
    /// tracked-scroll offset, the max offset, and each chip's raw layout
    /// bounds in canonical display order — read-only from the production
    /// `space_strip_scroll` handle so E2E can prove overflow and that
    /// selecting an offscreen chip scrolls it fully into view.
    #[serde(default)]
    pub space_strip_scroll: SpaceStripScrollSnapshot,
    /// §11.3 Space-configuration surface state while open: the bound exact
    /// persistent SpaceId, the header title, the rename draft, the selected
    /// curated theme/identity names, and whether the surface owns chrome
    /// focus. `None` while closed (the surface is not rendered).
    #[serde(default)]
    pub space_config: Option<SpaceConfigSnapshot>,
    /// DESIGN.md §11.3 swipe gesture evidence: the active space (truthful
    /// at every phase — a tracking gesture never changes the selected
    /// space), the source/destination persistent space ids and display-order
    /// direction resolved at begin, the signed progress clamped to [-1, 1],
    /// and the resolved chrome/chip tints the render interpolates while
    /// tracking (from the one central `space_tint` resolver; `None` while
    /// Idle). The selected chip changes only when a committed swipe reaches
    /// the destination through `activate_space`.
    #[serde(default)]
    pub swipe: SwipeSnapshot,
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
    /// The §11.5 address pill's derived domain (display-only — the pill
    /// never holds the editable text, a cursor or a selection).
    #[serde(default)]
    pub address_domain: String,
    /// The §11.5 address-pill hover state (the pointer is over the pill;
    /// the translucent fill steps one brighter/darker via the token pair).
    #[serde(default)]
    pub address_hovered: bool,
    /// The §11.5 address pill's measured window-relative rect (the §11.4
    /// overlay anchor): the overlay covers this rect, so E2E asserts the
    /// overlay's top/leading edges are flush with the pill's.
    #[serde(default)]
    pub address_rect: Option<[f32; 4]>,
    /// The open command-surface presentation (DESIGN.md §11.4): the centered
    /// Cmd-T palette, the field-anchored Cmd-L/click overlay, or none. At
    /// most one is open at a time (mutual exclusion by construction).
    #[serde(default)]
    pub command_surface: CommandSurface,
    /// The typed state of the command result surface (DESIGN.md §11.5).
    #[serde(default)]
    pub command_result_state: CommandResultState,
    /// The ranked results of the latest published generation (limit 16).
    #[serde(default)]
    pub command_results: Vec<CommandResultSnapshot>,
    /// The truthful total match count of the latest published generation
    /// before the limit-16 window truncation (the full-search benchmark
    /// presentation evidence: the window is full and the total exceeds it).
    #[serde(default)]
    pub command_result_total: usize,
    /// Byte offset of the command-text cursor (UTF-8 safe; the text model
    /// never splits a character).
    #[serde(default)]
    pub command_cursor: usize,
    /// Selected byte range `[start, end)` of the command text when
    /// non-empty (`None` = no selection; Cmd-L selects the whole URL).
    #[serde(default)]
    pub command_selection: Option<[usize; 2]>,
    /// Complete persistent tab identity across spaces (the command corpus
    /// source): space order, then tab order.
    #[serde(default)]
    pub all_tabs: Vec<GlobalTabSnapshot>,
    pub sidebar_visible: bool,
    /// §11.14 titlebar presentation evidence (task packet
    /// `titlebar-navigation-controls`): the native safe-area guide the row
    /// renders from (strip height + the traffic-light region — never
    /// hardcoded coordinates), the sidebar collapse state, and the genuine
    /// Back/Forward/Reload availability of the focused pane. E2E asserts
    /// the row disappears with the collapsed rail via
    /// [`Self::sidebar_visible`].
    #[serde(default)]
    pub titlebar: TitlebarSnapshot,
    /// DESIGN §2 seated content card frame and native clip.
    #[serde(default)]
    pub content_card: ContentCardSnapshot,
    /// §17.14 address zoom chip: the zoom percent it shows (`None` while
    /// the page zoom is 100 % — the accessory slot stays empty).
    #[serde(default)]
    pub address_zoom: Option<u16>,
    /// §11.4 the identity value the active-space heading renders
    /// (`bbr-active-space-identity`) — the active footer chip's own value.
    #[serde(default)]
    pub active_space_identity: Option<String>,
    /// §11.10 grid geometry the render uses: the constant column count and
    /// the derived `[tile_width, tile_height]` for the current rail width.
    #[serde(default)]
    pub favorites_columns: usize,
    #[serde(default)]
    pub favorites_tile_size: [f32; 2],
    /// §11.10 `favorites_region_height` for the current count and drag
    /// offer (the E2E compares it with the measured `favorites_rect`).
    #[serde(default)]
    pub favorites_region_height: f32,
    /// §10.9/§11.2/§11.10 drag offers of the in-flight tab drag (all empty
    /// at rest).
    #[serde(default)]
    pub drag_offers: DragOffersSnapshot,
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
    /// Window frame size in points (`window.bounds()`, which includes the
    /// macOS title bar). Diagnostics only; the truthful restored content
    /// area is [`Snapshot::window_content_size`].
    pub window_size: [f32; 2],
    /// The native window content-area size `[width, height]` in points
    /// (excluding the title bar): the truthful restored geometry — GPUI's
    /// window options apply the persisted bounds as the content rect, so
    /// this is the size a persisted window row reproduces. Measured via
    /// the live session's window `contentLayoutRect`; `[0.0, 0.0]` before
    /// the first native session exists.
    #[serde(default)]
    pub window_content_size: [f32; 2],
    /// Typed diagnostics of the one native portal host (task packet
    /// `native-portal-host`; DESIGN.md §15). Exposes only the active
    /// presentation and its native attachment/frame/visibility evidence.
    #[serde(default)]
    pub portal: PortalSnapshot,
    /// Typed find-in-page presentation state (task packet
    /// `find-in-page-portal-bar`): open presentation, field focus, query,
    /// WebKit-reported count/ordinal, pending request, and staleness.
    #[serde(default)]
    pub find: FindSnapshot,
    /// Typed per-site zoom state (queue item 11, part A; task packet
    /// `site-settings-zoom-contract`): effective level of the active view's
    /// site, the normalized origin, the applied level, and pending write.
    #[serde(default)]
    pub zoom: ZoomSnapshot,
    /// Typed BBR context-menu presentation state (task packet
    /// `bbr-context-menus-over-web`): open presentation, focus, selection,
    /// the deterministic row model with genuine enablement, the pointer
    /// anchor, and the native bridge's ownership diagnostics.
    #[serde(default)]
    pub context_menu: ContextMenuSnapshot,
    /// Typed Tab/Favorite context-menu presentation state (task packet
    /// `tab-favorite-context-menus`; DESIGN.md §11.13): the exact bound
    /// `TabId`, the keyboard/pointer cursor, the open submenu, the
    /// deterministic row model with genuine enablement, and the resolved
    /// clamped placement. Ordinary GPUI chrome — never a portal surface.
    #[serde(default)]
    pub tab_menu: TabMenuSnapshot,
    /// Typed credential-sheet presentation state (queue item 10): the
    /// presented challenge's identity/metadata, lifecycle state, redacted
    /// outcome, and masked edit flags. By construction no credential can
    /// appear in any wire shape.
    #[serde(default)]
    pub auth: AuthSnapshot,
    /// U2 compact-sidebar presentation state.
    #[serde(default)]
    pub plus_menu_open: bool,
    #[serde(default)]
    pub plus_menu_selection: usize,
    #[serde(default)]
    pub favorites: Vec<FavoriteTileSnapshot>,
    #[serde(default)]
    pub folders: Vec<FolderRowSnapshot>,
    /// §11.12 download rows in deterministic panel order (Active ascending
    /// `DownloadId`, then History most-recent-terminal first); row index
    /// `<n>` is stable for E2E.
    #[serde(default)]
    pub download_rows: Vec<DownloadRowSnapshot>,
    /// §11.12 downloads panel render contract (open, focus, bounded window).
    #[serde(default)]
    pub downloads: DownloadsPanelSnapshot,
    /// The last completed real WebKit content probe (testkit-only; `None`
    /// until a [`crate::protocol::Request::ProbeWebContent`] completion
    /// landed). Real committed-document evidence, never our own bookkeeping.
    #[serde(default)]
    pub web_content_probe: Option<WebContentProbe>,
    /// §11.8 error-surface presentation (task packet
    /// `error-surface-contract`): the classified incident the content area
    /// renders, or the empty default when no error surface is shown.
    #[serde(default)]
    pub error_surface: ErrorSurfaceSnapshot,
    /// Phase-7 app-owned persistence composition (task packet
    /// `persistence-core` + the command-history provider slice): the truthful
    /// startup store disposition, the effective global search engine, and the
    /// in-memory history provider size. `Unavailable` only for the ephemeral
    /// fallback after a fatal open failure.
    #[serde(default)]
    pub store: StoreSnapshot,
    /// Phase-7 startup restore evidence (warm-restore slice): whether a
    /// persisted browser model was applied chrome-first and how many tab
    /// identities stayed session-less (deferred) after the restore. A failed
    /// restore rolls back to a fresh default model and reports `Failed` —
    /// never a partial restored window presented as valid.
    #[serde(default)]
    pub restore: RestoreDisposition,
    /// Productive new-window request/open/block counters.
    #[serde(default)]
    pub new_window: NewWindowSnapshot,
    /// Web-content navigation gesture diagnostics (N1): commit
    /// classification counters and live opener-provenance pairs.
    #[serde(default)]
    pub content_gesture: ContentGestureSnapshot,
    /// The transient Peek preview (Link Preview / Peek packet): `None` at
    /// rest. The preview is transient controller state — never a persisted
    /// `Tab`, recently-closed row, sidebar row, or history item. The
    /// snapshot exposes the disposable preview session id, its warm origin,
    /// the source profile, and the live navigation state so E2E can prove
    /// warmth (same origin `WebViewId`), dismissal (state returns to
    /// `None`), promotion (the same `WebViewId` becomes a regular tab), and
    /// replacement without texture composition.
    #[serde(default)]
    pub preview: Option<PreviewSnapshot>,
    /// Whether the Peek bar is currently presented in the one native portal
    /// window (the transient preview surface above the web content area).
    #[serde(default)]
    pub peek_presented: bool,
    /// The theme mode (`light`/`dark`) the portal render last drew the Peek
    /// bar in — render evidence that the bar follows the shell appearance.
    /// `None` until the bar has been presented.
    #[serde(default)]
    pub peek_theme: Option<String>,
    /// Truthful native z-order evidence (Link Preview / Peek packet): while
    /// a preview is open the preview WKWebView's subview index must exceed
    /// its covered origin's. `None` at rest.
    #[serde(default)]
    pub preview_z_order: Option<PreviewZOrder>,
    /// Saved Pages Library presentation state (B3 packet
    /// `saved-pages-library`): open flag, search/sort, the bounded row
    /// window, the flat collections, the multi-select set, the destructive
    /// remove confirmation, and the native-view hide evidence. The empty
    /// default means the Library is closed.
    #[serde(default)]
    pub library: SavedPagesLibrarySnapshot,
}

/// The Phase-7 startup restore outcome, surfaced truthfully in the testkit
/// snapshot: whether a persisted browser model was applied chrome-first and
/// how many tab identities stayed session-less (deferred) after the restore.
/// The fast-but-wrong guard pairs this disposition with the snapshot's
/// `engine_sessions` count: a restore that eagerly instantiated every
/// persisted session reports the same deferred count but a session count of
/// hundreds, so the E2E rejects it. A failed restore rolls back to a fresh
/// default model and reports [`RestoreDisposition::Failed`] — never a
/// partial restored window presented as valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RestoreDisposition {
    /// No persisted browser model (fresh store) — the startup opened the
    /// initial tab as before.
    #[default]
    Fresh,
    /// A persisted browser model was applied; `deferred_tabs` identities
    /// stayed session-less until activation.
    Restored { deferred_tabs: usize },
    /// The persisted model was rejected (validation or mid-apply failure)
    /// and the controller rolled back to a fresh default model.
    Failed,
}

/// The truthful startup disposition of the app-owned persistence store
/// (schema v2: no migrations, no backward compatibility — a mismatched or
/// corrupt store is discarded and recreated empty, never loaded partially).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StoreLoadSnapshot {
    /// The store's stamp matched (or the store was a fresh empty database)
    /// and it was loaded.
    Loaded,
    /// The store was discarded and recreated empty, with the typed reason
    /// surfaced verbatim.
    Discarded { reason: StoreDiscardReasonSnapshot },
    /// No store exists (the ephemeral fallback after a fatal open failure).
    #[default]
    Unavailable,
}

/// Why a store was discarded and recreated empty (mirrors the persistence
/// contract's typed reasons — never silently converted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreDiscardReasonSnapshot {
    /// The store carried no version stamp but contained data (legacy).
    Unstamped,
    /// The store carried a different schema version stamp.
    VersionMismatch { found: u32 },
    /// The stamp matched but the structure failed validation.
    Corrupt { detail: String },
}

/// Phase-7 store composition snapshot: disposition, effective settings, and
/// the in-memory history provider population.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StoreSnapshot {
    pub store_load: StoreLoadSnapshot,
    /// The effective global search-engine id (`duckduckgo` | `google` |
    /// `bing` | `ecosia` | `kagi` | `custom`), feeding the command-bar
    /// search fallback URL.
    pub search_engine: String,
    /// The number of provider history entries ranking consumes. No
    /// retention cap in this slice (retention deferred in Phase 7); the
    /// 10,000 scale is only the test/performance corpus size.
    pub history_count: usize,
    /// The number of in-memory Saved Page provider records ranking consumes
    /// (B2 packet `saved-pages-actions-command`): the single runtime
    /// authority, loaded once at startup and updated only after accepted
    /// save/remove mutations — never queried from SQLite during ranking or
    /// render.
    pub saved_pages_count: usize,
}

/// Typed Saved Pages Library presentation state (B3 packet
/// `saved-pages-library`; DESIGN.md §11.16/§15). The empty default means
/// the Library is closed.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SavedPagesLibrarySnapshot {
    /// The Library surface is open (the internal chrome surface covers the
    /// content area and the active view's native slot is hidden).
    pub open: bool,
    /// The presentation stage (`loading` while a snapshot is attached;
    /// production opens `ready`).
    pub stage: String,
    /// The active search query (empty = unfiltered).
    pub search: String,
    /// The deterministic sort machine name (`updated_newest` |
    /// `updated_oldest` | `title_ascending` | `title_descending`).
    pub sort: String,
    /// The total number of matching records behind the window.
    pub total: usize,
    /// The bounded row window the surface renders — never the full
    /// collection (the 1k bounded-render evidence).
    pub rows: Vec<SavedPagesLibraryRowSnapshot>,
    /// The windowed render bounds `[first, last)` plus the stable scroll
    /// anchor (the `BookmarkId` of the first visible row).
    pub window_first: usize,
    pub window_last: usize,
    pub window_anchor: Option<u64>,
    /// The flat collections, one level only.
    pub collections: Vec<SavedPagesCollectionSnapshot>,
    /// The multi-select set (stable `BookmarkId`s).
    pub selected: Vec<u64>,
    /// True while the destructive remove confirmation is presented.
    pub remove_confirm: bool,
    /// The ids targeted by the pending destructive remove.
    pub remove_targets: Vec<u64>,
    /// The collection row whose pointer action is staged
    /// (`rename` | `remove`), when any.
    pub collection_action: Option<(u64, String)>,
    /// The staged collection rename/create draft label (empty at rest).
    pub collection_draft: String,
    /// True while the collection create input is staged (the action
    /// surface's create operation).
    pub collection_create_staged: bool,
    /// True while the selection move-to sheet is open.
    pub move_sheet_open: bool,
    /// The tracked production scroll offset (rows scrolled past) the wheel
    /// listener updates/clamps and the bounded window derives from.
    pub scroll_offset: usize,
    /// The row whose inline/staged rename input is presented (the pointer
    /// rename entry, B3); `None` at rest.
    pub editing_row: Option<u64>,
    /// The staged rename draft label (empty at rest).
    pub rename_draft: String,
    /// The active native view is hidden by the open Library (never
    /// destroyed; restored exactly on close).
    pub native_view_hidden: bool,
    /// The in-memory provider record count (the row evidence behind the
    /// window; the store snapshot's `saved_pages_count` mirror).
    pub saved_pages_count: usize,
}

/// One bounded Library row (§11.16 row contract, typed for E2E).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SavedPagesLibraryRowSnapshot {
    /// The stable `BookmarkId` (the §15 id suffix).
    pub bookmark: u64,
    /// Display title (user label → captured title → hostname/URL fallback).
    pub title: String,
    /// Mid-ellipsized canonical URL.
    pub url: String,
    /// Optional one-level collection label.
    pub collection: Option<String>,
    /// The §10 `selected` state.
    pub selected: bool,
    /// The §7.2 monogram fallback letter.
    pub favicon_letter: String,
}

/// One flat collection row (§11.16 flat collections, one level only).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SavedPagesCollectionSnapshot {
    pub id: u64,
    pub title: String,
    pub member_count: usize,
}

/// The drag offers of an in-flight tab drag (DESIGN.md §10.9, §11.2,
/// §11.10), recorded from the render: which offer is the live target, the
/// rendered ghost-row and drag-proxy rects, and the rail growth the offers
/// committed at activation. All `None`/`false` at rest.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DragOffersSnapshot {
    /// The pointer is over the §11.2 "Pin tab here" ghost row (a drop pins
    /// the dragged tab).
    #[serde(default)]
    pub pin_offer_targeted: bool,
    /// The ghost row's rendered window rect `[x, y, width, height]`.
    #[serde(default)]
    pub pin_offer_rect: Option<[f32; 4]>,
    /// The §10.9 drag proxy's rendered window rect.
    #[serde(default)]
    pub proxy_rect: Option<[f32; 4]>,
    /// The rail growth (points) the offers committed above the dragged row
    /// once, at activation.
    #[serde(default)]
    pub growth: Option<f32>,
}
