//! The gateway request enum. One request per capability; pointer-path
//! requests route through the same app methods the element listeners call so
//! E2E exercises the production listener path.

use bbr_web::{ProfileId, WebAction, WebViewId};
use serde::{Deserialize, Serialize};

use super::{BenchmarkKind, BenchmarkOperationId, TestSpaceId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Handshake: authenticate and exchange capabilities.
    Hello {
        protocol_version: u32,
        token: String,
        capabilities: Vec<String>,
    },
    /// Subscribe to the event stream; `since` replays all events with
    /// seq > `since` from the server ring (live delivery continues after).
    Attach {
        since: Option<u64>,
    },
    /// Navigate the active web view to a URL.
    Navigate {
        url: String,
    },
    /// Create an isolated profile with one initial space.
    CreateProfile {
        profile_title: String,
        space_title: String,
    },
    /// Create another workspace in an existing profile.
    CreateSpace {
        profile: ProfileId,
        title: String,
    },
    /// Activate a workspace by stable id.
    ActivateSpace {
        space: TestSpaceId,
    },
    /// Open a new tab and make it active.
    OpenTab {
        url: String,
    },
    /// Select an existing tab by active-space index (semantic setups).
    SelectTab {
        index: usize,
    },
    /// Close a tab by active-space index.
    CloseTab {
        index: usize,
    },
    /// Reopen the most recently closed tab (fresh session, preserved identity).
    ReopenTab,
    /// Open the two-pane split over the two active-space tab indices: both
    /// panes become visible with live native sessions, the domain active
    /// tab stays the focused pane. Test-only seam into the production
    /// controller split path (the split-core lane owns the user-facing
    /// creation UI); proves keyed per-`WebViewId` native geometry.
    SetSplit {
        primary: usize,
        secondary: usize,
    },
    /// Close the active space's split layout: single-pane mode returns, both
    /// tabs stay open, the non-focused pane becomes a warm session. The
    /// focused pane (the domain active tab) keeps full-width geometry.
    CloseSplit,
    /// Pin the tab at the active-space index (moves into the pinned region).
    PinTab {
        index: usize,
    },
    /// Unpin the tab at the active-space index (moves to the end of the
    /// regular region).
    UnpinTab {
        index: usize,
    },
    /// Reorder the tab at `from` to `to` within the active space (same
    /// region only; the pinned-first invariant is domain-owned).
    ReorderTab {
        from: usize,
        to: usize,
    },
    /// Activate a visible tab row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickTab {
        index: usize,
    },
    /// Activate the visible tab row's close control through the production
    /// pointer path.
    ClickTabClose {
        index: usize,
    },
    /// Toggle the visible tab row's pin control through the production
    /// pointer path (the pointer half of the pin/unpin command).
    ClickTabPin {
        index: usize,
    },
    /// Pointer hover over a visible tab row through the production `on_hover`
    /// listener path (the exact app method the element handler calls): the
    /// row renders the DESIGN §10.2 hover wash and reveals its close control.
    HoverTabRow {
        index: usize,
    },
    /// Pointer leave through the same production hover listener path: clears
    /// the row's hover state.
    UnhoverTabRow,
    /// Pointer press on a visible tab row through the production mouse-down
    /// listener path: the row renders the DESIGN §10.3 active (pressed) wash
    /// while the button is down. Selection is not implied — the press
    /// listener is a separate production handler from the select listener.
    PressTabRow {
        index: usize,
    },
    /// Pointer release through the production mouse-up listener path: clears
    /// the pressed row state.
    ReleaseTabRow,
    /// Pointer drag entry: press on a visible tab row through the production
    /// mouse-down listener path (the same method the row's listener calls),
    /// anchored at `y` so the subsequent moves are relative to the press.
    /// Pointer press on a tab row, starting a drag at window y `y` with the
    /// press `press_in_row` within the source row (0 at the top edge, the
    /// row height at the bottom). The seam passes the row-center offset
    /// unless a scenario anchors an off-center press.
    BeginTabDrag {
        index: usize,
        y: f32,
        press_in_row: f32,
    },
    /// Pointer drag move: a `dy` displacement from the press position, sent
    /// through the production mouse-move listener path. The app resolves the
    /// typed target (tab slot or space row) and its legality.
    MoveTabDrag {
        dy: f32,
    },
    /// Pointer drag release through the production mouse-up listener path:
    /// commits a legal target (region-bounded reorder or move-between-spaces)
    /// and clears the drag.
    EndTabDrag,
    /// Cancel an in-flight drag (the production Escape path): nothing is
    /// committed, chrome focus stays in the tab list.
    CancelTabDrag,
    /// Deterministic engine-event injection (testkit-only): pushes a native
    /// `WebEvent` through the exact event transport the real engine uses, so
    /// E2E can exercise the production crash/error path without killing a
    /// real WKWebView process.
    InjectWebEvent {
        event: bbr_web::WebEvent,
    },
    /// Activate a visible space row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickSpace {
        space: TestSpaceId,
    },
    /// Activate the New Space row through the production pointer path (stages
    /// the NewSpace command-bar mode).
    ClickSpaceNew,
    /// Move an active-space tab into another space.
    MoveTab {
        index: usize,
        to_space: TestSpaceId,
    },
    /// Suspend a hidden tab and destroy its native session.
    SuspendTab {
        index: usize,
    },
    /// Deterministic dense-list seeding (testkit-only setup): open
    /// `tab_count` tabs through the production controller open path, pin the
    /// first `pinned_count` identities, immediately suspend every tab outside
    /// the keep set (first `pinned_count` + global index ≥ `recent_start`),
    /// then activate the pinned tab 0. Each transient session is destroyed
    /// before the next open, so the concurrent live-session population never
    /// exceeds `pinned_count + recent + 1` — never `tab_count` WKWebViews at
    /// once. Exercises the same domain/controller mutation and render paths
    /// the product uses; the state is in-memory (persisted-restore
    /// acceptance is deliberately open until the persistence repository).
    /// The E2E harness may call it in batches (each bounded by the gateway
    /// per-request deadline) and snapshot between batches to assert the
    /// live-session bound at the observable points.
    SeedDenseState {
        tab_count: usize,
        pinned_count: usize,
        recent_start: usize,
        url: String,
    },
    /// Deterministically converge the active space's live-session population
    /// to `cap` through the controller's session-budget policy (keeps
    /// visible, pinned, and most recently used identities live; suspends the
    /// least recently used warm tabs). Testkit-only setup; the budget policy
    /// itself is the production lifecycle path.
    EnforceSessionBudget {
        cap: usize,
    },
    /// A dense-list scroll through the production scroll state: the tracked
    /// `ScrollHandle` the native wheel listener, the keyboard traversal, and
    /// the render share is moved by `delta_px` in the wheel listener's units
    /// and sign (`delta_px < 0` scrolls down; the container offset goes
    /// negative). GPUI 0.2.2 exposes no public synthetic wheel dispatch
    /// (`Window::dispatch_event` returns a crate-private result type), so the
    /// seam mutates the exact offset a wheel event would drive; the render
    /// derives the bounded row window from it identically for a wheel, a
    /// `PgDn`, or this entry. Never a direct snapshot mutation.
    ScrollDenseTabs {
        delta_px: f32,
    },
    /// Select a tab and measure until the correct native view is presented.
    BenchmarkSelectTab {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// U2 warm presentation benchmarks through production sidebar paths.
    BenchmarkPlusMenu {
        operation_id: BenchmarkOperationId,
    },
    BenchmarkFavoriteToggle {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    BenchmarkFolderToggle {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// Activate a space and measure until its warm selected tab is presented.
    BenchmarkActivateSpace {
        operation_id: BenchmarkOperationId,
        space: TestSpaceId,
    },
    /// Activate a visible tab row through the pointer path and measure until
    /// the correct native view is presented with chrome focus preserved.
    BenchmarkClickTab {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// Reopen the most recently closed tab and measure until its fresh native
    /// session is presented.
    BenchmarkReopenTab {
        operation_id: BenchmarkOperationId,
    },
    /// Arm a deterministic-page-ready measurement for `expected_view`; it
    /// commits once the fixture navigation finishes with `expected_title`.
    /// The start reference is process spawn, so latency is comparable to
    /// [`BenchmarkKind::Startup`].
    BenchmarkPageReady {
        operation_id: BenchmarkOperationId,
        expected_view: WebViewId,
        expected_title: String,
    },
    /// Toggle the sidebar and measure until chrome plus native content geometry
    /// are presented in the resulting state.
    BenchmarkToggleSidebar {
        operation_id: BenchmarkOperationId,
    },
    /// Dispatch a real GPUI keystroke while measuring its presentation commit.
    BenchmarkDispatchKeystroke {
        operation_id: BenchmarkOperationId,
        kind: BenchmarkKind,
        keystroke: String,
    },
    /// Drive a [`WebAction`] on the active web view (Back/Forward/Reload/…).
    WebAction(WebAction),
    /// Focus the GPUI command bar (chrome) — the E2E reader of Cmd-L.
    ActivateCommandBar,
    /// Pointer press on the sidebar command field through the production
    /// pointer-entry path: the same app method the field's mouse-down
    /// listener calls (DESIGN.md §11.4 field-click entry into the
    /// field-anchored overlay, same engine as Cmd-L).
    ClickCommandField,
    /// Pointer press on the window content right of the rail while a §11.4
    /// palette/overlay presentation is open: the same app method the §13
    /// dismissal layer's mouse-down listener calls (one pointer action
    /// closes the surface and restores the origin focus domain).
    ClickContentOutside,
    /// Return keyboard focus to the web view — the E2E reader of Escape.
    FocusWeb,
    /// Set the command bar text directly (avoids per-keystroke round trips).
    SetCommandBarText {
        text: String,
    },
    /// Deterministic command-ranking flush (testkit-only): cancels the
    /// pending debounce timer and publishes the current generation's ranked
    /// results synchronously, so E2E asserts burst typing (only the final
    /// generation ever publishes) without waiting on real time.
    FlushCommandRanking,
    /// Issue a real WebKit content probe for `view` (testkit-only): reads
    /// the committed document in BBR's own client content world
    /// (`readyState`/`URL`/`title`/body) plus the native `WKWebView`
    /// `url`/`title`, and publishes the observation on the next snapshot's
    /// `web_content_probe`. Never mutates the page. Real content evidence
    /// for the schemeless-navigation regression: the visibility/geometry
    /// bookkeeping can be green while the user sees a blank page.
    ProbeWebContent {
        view: WebViewId,
    },
    /// Dispatch a synthetic keystroke through GPUI's real input path
    /// (e.g. "cmd-l", "escape"). Zero permissions, zero OS event posting.
    DispatchKeystroke {
        keystroke: String,
    },
    /// Open the native portal probe through the general Portal API (task
    /// packet `native-portal-host`): presents a centered `NativeAboveWeb`
    /// probe card with the stable `bbr-portal-probe`/`-card`/`-backdrop`
    /// ids through the same `portal_present` production seam a real chrome
    /// surface uses. The probe content is deliberately not U1 palette
    /// content; it exists only to prove the native host infrastructure.
    PortalOpenProbe,
    /// Close the portal probe through the production dismissal method
    /// (`BbrApp::portal_dismiss`): the active presentation is dismissed, the
    /// native portal window is ordered out, and the probe surface clears.
    PortalCloseProbe,
    /// Drive the exact production outside-dismiss method the portal
    /// backdrop's pointer-press listener calls (DESIGN.md §13 one-outside-
    /// pointer-action rule): the same `BbrApp::portal_outside_dismiss` path
    /// a real outside click on the backdrop takes. Only dismisses when the
    /// active presentation's `DismissalPolicy` allows outside-pointer
    /// dismissal.
    PortalOutsideDismiss,
    /// Deterministically tear down the one native portal host (task packet
    /// teardown evidence): `BbrApp::portal_teardown` removes the child
    /// relation, orders the portal window out, closes it, and drops the
    /// bridge — the app owns no portal afterwards. Terminal; the snapshot
    /// afterwards reports the empty/closed host.
    PortalTeardown,
    /// Open the find-in-page presentation over the active web view (task
    /// packet `find-in-page-portal-bar`; the production `Cmd-F` path). The
    /// bar renders in the one `NativeAboveWeb` portal window; opening never
    /// creates, moves, or reselects a web session.
    FindOpen,
    /// Close the find presentation through the production close path
    /// (Escape/close-button semantics): restores the origin focus domain and
    /// clears highlights on the bound view.
    FindClose,
    /// Set the find query and start a fresh search (the typing/backspace
    /// path; empty text clears highlights and the report).
    FindSetQuery {
        query: String,
    },
    /// Navigate to the next match (Enter / next-button path).
    FindNext,
    /// Navigate to the previous match (Shift-Enter / previous-button path).
    FindPrevious,
    /// Bring the app to the foreground (testkit-only): the shown-harness
    /// portal frontmost test needs the BBR window to be the active app's
    /// front window so the real `windowNumberAtPoint` evidence at the probe
    /// point resolves to the browser window (portal or parent), never a
    /// background app's window. Uses the same `App::activate(true)` the app
    /// calls at startup; it activates the browser window — never the
    /// non-activating portal.
    ActivateApp,
    /// Resize the window content area (points). Used to observe layout
    /// re-flow via the snapshot; the native view is re-framed on the next
    /// frame.
    SetWindowSize {
        width: f32,
        height: f32,
    },
    /// Real pointer drag on the sidebar resize divider through GPUI's
    /// production mouse-event pipeline: mouse-down at the divider center,
    /// mouse-move by `dx` with the left button held, mouse-up. The same
    /// element listeners a physical drag drives.
    DragSidebarDivider {
        dx: f32,
    },
    /// Focus the resize divider through the production focus path (the same
    /// entry a pointer click uses), the anchor for the keyboard resize keys.
    FocusSidebarDivider,
    /// Draw a real GPUI frame synchronously (testkit-only frame flush). The
    /// hidden E2E harness has no display link, so paint-driven native
    /// geometry callbacks would never fire after a mutation; this forces the
    /// exact production draw path (`Window::draw`) so the snapshot observes
    /// the WKWebView frame the next layout hands the slot.
    FlushFrame,
    /// Measure the production sidebar resize mutation until the resulting
    /// native geometry commits at the expected clamped width.
    BenchmarkResizeSidebar {
        operation_id: BenchmarkOperationId,
        width: f32,
    },
    /// Measure the production sidebar restoration: collapse (unmeasured
    /// setup), then expand and commit when the last expanded width is
    /// restored.
    BenchmarkRestoreSidebar {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production pin toggle of the active tab until the new
    /// region order is presented (same native view, boundary moved).
    BenchmarkPinTab {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production reorder of the active tab by `delta` rows
    /// within its region until the new row order is presented.
    BenchmarkReorderTab {
        operation_id: BenchmarkOperationId,
        delta: isize,
    },
    /// Measure the production move of the active tab into `to_space` until
    /// the source space's repaired selection is presented. Same-profile
    /// moves keep the live session; cross-profile moves are the
    /// session-recreation class (fresh view in the destination store).
    BenchmarkMoveTab {
        operation_id: BenchmarkOperationId,
        to_space: TestSpaceId,
    },
    /// Measure a dense-list scroll through the production scroll state: the
    /// tracked `ScrollHandle` the native wheel listener, keyboard traversal,
    /// and render share is moved by `delta_px` (no public synthetic wheel
    /// dispatch in GPUI 0.2.2), and the sample commits after the frame that
    /// renders the resulting bounded window.
    BenchmarkDenseScroll {
        operation_id: BenchmarkOperationId,
        delta_px: f32,
    },
    /// Measure the production selection of a dense tab (pointer or keyboard
    /// path): the sample commits after the frame that renders the selection
    /// with the active row inside the bounded window.
    BenchmarkDenseSelect {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// Measure the production pointer hover entry on a visible tab row
    /// (`set_row_hover`, the exact app method the row's `on_hover` element
    /// listener calls): the §10.2 hover wash + close reveal commit.
    BenchmarkHoverTab {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// Measure the production pointer press on a visible tab row
    /// (`set_row_pressed`, the exact app method the row's mouse-down
    /// listener calls): the §10.3 active wash commits.
    BenchmarkPressTab {
        operation_id: BenchmarkOperationId,
        index: usize,
    },
    /// Measure the production command-text path (Cmd-L selects the active
    /// URL, replace-selection typing enters `text`, and the real ranking
    /// debounce publishes the ranked results): the sample commits only from
    /// the published semantic presentation — the exact generation, first/
    /// selected result key, state, mode, and focus are enforced in the app
    /// (fast-but-wrong guard). The deterministic flush is never used inside
    /// this measured scenario.
    BenchmarkCommandType {
        operation_id: BenchmarkOperationId,
        text: String,
        expected_first_key: String,
    },
    /// Measure the production Enter activation of the command bar (the
    /// selected/typed destination routed through the app/controller seams):
    /// the sample commits only when the expected destination view is
    /// presented in web focus with the surface closed.
    BenchmarkCommandActivate {
        operation_id: BenchmarkOperationId,
        expected_view: Option<WebViewId>,
    },
    /// Measure the production Cmd-Enter activation (the destination opens in
    /// exactly one new tab/session and the fresh page is focused): the
    /// session-recreation class with its own budget. The sample commits only
    /// when a genuinely new tab/session is presented and focused.
    BenchmarkCommandActivateNewTab {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production Cmd-T open of the centered command palette
    /// (DESIGN.md §11.4 `bbr-palette`) in New Tab staging mode: the sample
    /// commits only from a render that actually constructed the palette with
    /// the staged mode and chrome focus (fast-but-wrong guard). No tab or
    /// session is created before Enter.
    BenchmarkPaletteOpen {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production `Cmd-F` find-bar open (the exact seam the
    /// `FindInPage` action routes to): the sample commits only from a render
    /// that presented the find bar with the field focused over the same
    /// active view — warm class, no tab/session touched.
    BenchmarkFindOpen {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production find-query path (`FindSetQuery`, the exact
    /// mutation typing/backspace route to): the sample commits only when
    /// WebKit reported the expected match count/ordinal through the portable
    /// contract and the bar re-rendered it (fast-but-wrong guard — a
    /// pending, stale, or mismatched report can never commit). Fixture and
    /// transport time are excluded by construction.
    BenchmarkFindQueryResult {
        operation_id: BenchmarkOperationId,
        query: String,
        expected_count: usize,
        expected_ordinal: Option<usize>,
    },
    /// Measure the production find step (`FindNext`/`FindPrevious`, the exact
    /// seams Enter/Shift-Enter and the bar buttons route to): the sample
    /// commits only when WebKit reported the expected ordinal and the bar
    /// re-rendered it — warm command-surface class.
    BenchmarkFindStep {
        operation_id: BenchmarkOperationId,
        next: bool,
        expected_count: usize,
        expected_ordinal: Option<usize>,
    },
    /// Measure the production footer `[Downloads]` open through
    /// [`Self::ClickFooterDownloads`]'s exact seam (`toggle_downloads` →
    /// `downloads_open`): the sample commits only from a render that drew
    /// the anchored panel open with the keyboard cursor (chrome focus) over
    /// the unchanged active view — warm-presentation class, no tab or
    /// session is created.
    BenchmarkDownloadsPanelOpen {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production L6 progress path: apply a deterministic
    /// [`DownloadEvent::Progress`] through the same main-thread pump the
    /// adapter's event sink uses, and commit only when the panel render
    /// drew the active row at the armed `received`/`total` fraction — warm
    /// command-surface class; fixture/network time is excluded (the event
    /// is applied in-process, never via a network transfer).
    BenchmarkDownloadsProgress {
        operation_id: BenchmarkOperationId,
        id: u64,
        received: u64,
        total: u64,
    },
    /// Measure the production retry row action (`DownloadRetry`'s exact
    /// seam `download_retry` → `controller.retry`): the fresh attempt
    /// (linked via `retry_of`) must be presented in the Active group — the
    /// user-visible row action, warm-presentation class.
    BenchmarkDownloadsRowRetry {
        operation_id: BenchmarkOperationId,
        row: usize,
    },
    /// Seed a deterministic non-transferring download attempt for the
    /// performance scenarios: mints the attempt through the portable
    /// controller with an empty web-view slot (no `WKDownload` starts) and
    /// applies `DestinationResolved` + an initial zero-byte `Progress`
    /// event through the production event path, leaving the attempt
    /// `Running` at the declared `total`. The driver reads the minted id
    /// from the snapshot. Testkit-only deterministic seeding (the same
    /// pattern as `SeedDenseState`); the real transfer lifecycle is proven
    /// by the functional E2E.
    SeedDownloadsAttempt {
        url: String,
        destination: String,
        total: u64,
    },
    /// U2 footer, favorites, and folders production pointer seams.
    ClickFooterDownloads,
    ClickFooterSpaces,
    ClickFooterPlus,
    ClickPlusMenuItem {
        index: usize,
    },
    ToggleFavoriteTab {
        index: usize,
    },
    ClickFavoriteTile {
        index: usize,
    },
    ClickFolderRow {
        index: usize,
    },
    ClickFolderToggle {
        index: usize,
    },
    ClickFolderMember {
        folder: usize,
        member: usize,
    },
    ClickFolderMemberClose {
        folder: usize,
        member: usize,
    },
    MoveTabIntoFolder {
        index: usize,
        folder: usize,
    },
    /// Start a download through the production controller + native adapter
    /// path: mints an attempt id, records the optimistic `Waiting` state,
    /// and starts the transfer on the active web view. `destination` is the
    /// suggested filename (L6 `Suggested`; the engine resolves the download
    /// folder and uniquifies) — the save-panel policy stays open (§17.9).
    DownloadStart {
        url: String,
        destination: String,
    },
    /// Request cancellation of the attempt (production `controller.cancel`
    /// path): the row stays active until the engine's terminal event lands.
    DownloadCancel {
        id: u64,
    },
    /// Retry the failed attempt (production `controller.retry` path): a
    /// fresh attempt id linked via `retry_of`.
    DownloadRetry {
        id: u64,
    },
    /// Activate a visible download row's contextual action through the
    /// production pointer path (the exact app method the row's action
    /// mouse-down listener calls): Cancel / Retry / Reveal-in-Finder.
    ClickDownloadRowAction {
        row: usize,
    },
    /// Drive the exact production outside-dismiss method the panel's
    /// outside-press handler calls (DESIGN.md §13 one pointer action): a
    /// pointer press outside the open panel dismisses it and returns focus
    /// to the footer control.
    ClickDownloadsOutside,
    /// Move the downloads panel's shared scroll state by `delta_px` in the
    /// native wheel listener's units and sign (DESIGN.md §11.12/§11.6
    /// bounded-window scroll: visible range + 8-row overscan, the full
    /// history never laid out).
    ScrollDownloads {
        delta_px: f32,
    },
    /// Full app snapshot.
    GetSnapshot,
    /// Liveness probe.
    Ping,
    /// Graceful app shutdown (`quit()`, window closes, process exits clean).
    Quit,
}
