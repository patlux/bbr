//! The typed `Snapshot` and its row/space/profile parts, plus the §15
//! visual-state vocabulary (DESIGN.md §15). All plain data with `serde`
//! derives so the wire format never leaks framework types.

use bbr_web::{ProfileId, WebViewId};
use serde::{Deserialize, Serialize};

use super::{CommandBarMode, ShellModes, TestProfileId, TestSpaceId, TestWebViewId};

mod u2;
pub use u2::{FavoriteTileSnapshot, FolderRowSnapshot, SpaceChipSnapshot};

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
    /// The glass content area: native slot, empty pane, or crash panel.
    Content,
    /// A row's favicon / monogram tile (`bbr-tab-favicon-*`).
    Favicon,
    /// A row's title text (`bbr-tab-title-*` / `bbr-space-identity-*`).
    Title,
    /// A row's count column (`bbr-space-count-*`).
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
/// `title`, body presence — plus the native `WKWebView` `url`/`title`
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

/// One BBR context-menu entry on the wire (task packet
/// `bbr-context-menus-over-web`): the stable key, the display title, and
/// the genuine enablement (Back/Forward reflect the real session history
/// availability from `CanGoBackChanged`/`CanGoForwardChanged`; Copy Link /
/// Open in New Tab reflect a resolvable page URL).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextMenuItemSnapshot {
    /// Stable key: `back` | `forward` | `reload` | `copy-link` |
    /// `open-new-tab`.
    pub key: String,
    /// The display title (Back / Forward / Reload / Copy Link /
    /// Open in New Tab).
    pub title: String,
    /// Whether the entry accepts interaction right now.
    pub enabled: bool,
}

/// Typed BBR context-menu presentation state (task packet
/// `bbr-context-menus-over-web`; DESIGN.md §13/§15). The menu renders
/// inside the one `NativeAboveWeb` portal window; this mirrors the app's
/// `ContextMenuState` exactly plus the native bridge's ownership
/// diagnostics.
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
    /// The page URL the menu acts on (Copy Link / Open in New Tab); `None`
    /// when no URL is resolvable (both entries then report disabled).
    pub url: Option<String>,
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
    /// region) or the §11.10 favorites grid.
    pub drag_slot: Option<usize>,
    /// Whether the current drag target is a legal drop (a boundary crossing
    /// or a non-droppable zone reports `false` and renders the muted tick).
    pub drag_valid: bool,
    /// §11.3 permanent space strip: one chip per space in canonical display
    /// order (the vector order is display order; the chip/identity ids
    /// derive from the persistent `space` id). The strip is always part of
    /// the rail.
    #[serde(default)]
    pub space_chips: Vec<SpaceChipSnapshot>,
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
}
