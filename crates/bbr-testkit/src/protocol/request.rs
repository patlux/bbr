//! The gateway request enum. One request per capability; pointer-path
//! requests route through the same app methods the element listeners call so
//! E2E exercises the production listener path.

use bbr_web::auth_challenges::{AuthAuthority, AuthScheme, HttpScheme};
use bbr_web::{ProfileId, WebAction, WebViewId};
use serde::{Deserialize, Serialize};

use super::{BenchmarkKind, BenchmarkOperationId, SwipePhaseWire, TestSpaceId, ThemeModeOverride};

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
    /// Set the curated theme token of a space through the production
    /// controller appearance authority (`set_space_theme`; the explicit
    /// record becomes authoritative over the derived profile-hue default).
    /// `theme` is the canonical token name (`cobalt`…`ink`); unknown names
    /// reject whole. Appearance has no settings/picker UI and no
    /// persistence — this is the testkit seam into the already-supported
    /// controller mutation.
    SetSpaceTheme {
        space: TestSpaceId,
        theme: String,
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
    /// Pointer hover over a folder row through the production `on_hover`
    /// listener path (the exact app method the element handler calls): the
    /// §11.11 `type:meta` count reveals while the pointer is over the row
    /// (the hover half of the hover/focus-reveal contract; the keyboard
    /// half follows the folders region's genuine chrome focus).
    HoverFolderRow {
        folder_id: u64,
    },
    /// Pointer leave through the same production folder-hover path: the
    /// revealed count returns quiet.
    UnhoverFolderRow,
    /// §11.14 pointer-path activation of the titlebar sidebar toggle
    /// through the exact production app method the element listener calls
    /// (the same sidebar-width/collapse authority as Cmd-S).
    ClickTitlebarToggle,
    /// §11.14 pointer-path activation of the titlebar Back control through
    /// the same navigation authority as Cmd-[.
    ClickTitlebarBack,
    /// §11.14 pointer-path activation of the titlebar Forward control
    /// through the same navigation authority as Cmd-].
    ClickTitlebarForward,
    /// §11.14 pointer-path activation of the titlebar Reload control
    /// through the same navigation authority as Cmd-R.
    ClickTitlebarReload,
    /// §11.14 pointer hover over a titlebar control through the production
    /// `on_hover` listener path (`control` is the §15 id suffix: "toggle" /
    /// "back" / "forward" / "reload").
    HoverTitlebarControl {
        control: String,
    },
    /// §11.14 pointer leave from a titlebar control through the production
    /// `on_hover` listener path.
    UnhoverTitlebarControl,
    /// §11.14 primary-button down on a titlebar control through the
    /// production press slot (the §10.3 pressed wash).
    PressTitlebarControl {
        control: String,
    },
    /// §11.14 primary-button up through the production release path (the
    /// root's mouse-up: clears the pressed wash and — blueprint D14 —
    /// steps Back/Forward when a short press is released over its control).
    ReleaseTitlebarControl,
    /// Blueprint D14: the full production primary-button down on a
    /// titlebar control at the pointer `(x, y)` — the pressed wash plus the
    /// control's press slot (Back/Forward arm the history hold timer).
    PressTitlebarControlAt {
        control: String,
        x: f32,
        y: f32,
    },
    /// Blueprint D14: a secondary click on Back/Forward at `(x, y)` (the
    /// production listener opens the history menu).
    RightClickTitlebarControl {
        control: String,
        x: f32,
        y: f32,
    },
    /// Blueprint D14: the history menu's row hover / press and close paths.
    NavHistorySelect {
        index: usize,
    },
    NavHistoryActivate {
        index: usize,
    },
    NavHistoryClose,
    /// Pointer hover over the §11.5 address pill through the production
    /// `on_hover` listener path (the exact app method the pill's element
    /// handler calls): the pill renders the §10.2/§11.5 hover wash (one
    /// fill step brighter on dark chrome, one step darker on light chrome).
    HoverAddress,
    /// Pointer leave through the same production hover listener path: clears
    /// the pill's hover state.
    UnhoverAddress,
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
    /// anchored at window `x`/`y` so the subsequent moves are relative to
    /// the press. The press `press_in_row` is the offset within the source
    /// row (0 at the top edge, the row height at the bottom). The seam
    /// passes the row-center offset unless a scenario anchors an off-center
    /// press; the anchor x matters for the §11.3 drag-to-chip path (the
    /// pointer must enter a rendered chip's x/y span).
    BeginTabDrag {
        index: usize,
        x: f32,
        y: f32,
        press_in_row: f32,
    },
    /// Pointer drag move: `dx`/`dy` displacements from the press position,
    /// sent through the production mouse-move listener path. The app
    /// resolves the typed target (tab slot, favorites grid, or the §11.3
    /// space chip under the pointer) and its legality.
    MoveTabDrag {
        dx: f32,
        dy: f32,
    },
    /// Pointer drag release through the production mouse-up listener path:
    /// commits a legal target (region-bounded reorder or move-between-spaces)
    /// and clears the drag.
    EndTabDrag,
    /// Cancel an in-flight drag (the production Escape path): nothing is
    /// committed, chrome focus stays in the tab list.
    CancelTabDrag,
    /// Drive the exact productive app new-window execution method used by
    /// the native coordinator delivery pump. The hidden harness cannot
    /// synthesize a trustworthy LinkActivated click, so direct-activation
    /// E2E injects the already-extracted typed request here; it never calls
    /// the controller operation directly. `intent` selects the policy
    /// authority: [`bbr_web::NewWindowIntent::Default`] opens an eligible
    /// request as a transient Peek preview; the explicit `OpenTab` intent
    /// keeps the NW-SEAM regular-tab path executable for E2E (the preview
    /// never records regular-tab opener provenance). The `input` disposition
    /// mirrors the native modifier/button metadata: `Background` routes a
    /// direct activation to the grouped background-child path
    /// (background link tabs packet).
    NewWindow {
        source: WebViewId,
        source_profile: ProfileId,
        url: String,
        user_initiated: bool,
        #[serde(default)]
        intent: bbr_web::new_window::NewWindowIntent,
        #[serde(default)]
        input: bbr_web::new_window::NewWindowInput,
    },
    /// Dismiss the transient Peek preview through the production close path
    /// (the Escape key, the close control, and this seam all call the same
    /// `BbrApp::peek_close` method): the preview session is destroyed and
    /// focus returns to the exact warm origin. No-op when no preview is
    /// open.
    PeekClose,
    /// Promote the transient Peek preview through the production "Open as
    /// Tab" path (`BbrApp::peek_promote`): the SAME preview session is
    /// adopted as a regular tab in the source space/profile without reload
    /// or state loss. A crashed preview refuses promotion.
    PeekPromote,
    /// Route one browser navigation action to the open preview session
    /// (the production preview-command path; Back/Forward/Reload/Stop).
    PeekNavigate {
        action: WebAction,
    },
    /// Measure the production eligible new-window → Peek presentation path
    /// (`peek_request_to_present`): the sample starts inside the app when
    /// the request enters `BbrApp::on_new_window_request` and commits only
    /// from a portal render that drew the Peek bar over the preview session
    /// with the warm origin still active and the preview raised above it
    /// (the in-app probe owns the fast-but-wrong guard). `url` is the
    /// deterministic local preview target fixture; fixture/network time is
    /// excluded by construction.
    BenchmarkPeekRequestToPresent {
        operation_id: BenchmarkOperationId,
        url: String,
    },
    /// Measure the production background-link open (DD-LINK packet
    /// `background-link-tabs`, `background_link_open`): the sample starts
    /// when the request enters `on_new_window_request` and commits only
    /// when the exact opener is still the active, web-focused view and the
    /// new child is a live hidden session at the grouped insertion index
    /// directly after the opener (fast-but-wrong guard in the app).
    BenchmarkBackgroundLinkOpen {
        operation_id: BenchmarkOperationId,
        url: String,
    },
    /// Measure the production Peek dismissal path
    /// (`peek_dismiss_to_origin`): the sample starts when `peek_close`
    /// destroys the preview session and commits only when the exact warm
    /// origin is again the active, focused view with the Peek bar gone.
    BenchmarkPeekDismissToOrigin {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production Peek promotion path
    /// (`peek_promote_to_tab`): the sample starts when `peek_promote`
    /// adopts the preview session and commits only when the SAME preview
    /// `WebViewId` is a regular tab, active and focused, with no reload.
    BenchmarkPeekPromoteToTab {
        operation_id: BenchmarkOperationId,
    },
    /// Drive the exact productive app content-gesture execution method used
    /// by the native content-gesture bridge reporter pump (N1): pushes one
    /// typed logical `ContentGestureCommit` through the exact transport the
    /// native bridge uses, so E2E can drive real gesture phases and assert
    /// the bound pane, history availability, provenance, target identity,
    /// session lifecycle, and focus deterministically. It never calls the
    /// controller operation directly.
    ContentGestureCommit {
        view: WebViewId,
        direction: bbr_web::ContentGestureDirection,
    },
    /// Deterministic engine-event injection (testkit-only): pushes a native
    /// `WebEvent` through the exact event transport the real engine uses, so
    /// E2E can exercise the production crash/error path without killing a
    /// real WKWebView process.
    InjectWebEvent {
        event: bbr_web::WebEvent,
    },
    /// Deliver an external URL open through the exact production queue the
    /// AppKit delegate path (`application:openURLs:` via GPUI's
    /// `on_open_urls`) pushes into: the pump drains it into
    /// `BbrApp::on_external_url`, the app-lifetime coordinator decides, and
    /// the existing controller executes the outcome. Testkit-only seam into
    /// the production path, so E2E proves delivery, burst dedup, rejection,
    /// and identity preservation deterministically (queue item 13).
    DeliverExternalUrl {
        url: String,
    },
    /// Deterministic classified-incident injection (testkit-only): pushes a
    /// classified web-error incident through the exact seam the native
    /// delegate's incident sink drives (`BbrApp::on_incident`), so E2E can
    /// exercise the production error-surface presentation for a closed
    /// contract category without depending on host network state (a
    /// genuinely-offline host cannot be toggled deterministically). The
    /// native-code → taxonomy mapping itself stays unit-proven in
    /// `bbr-web-macos::error_mapping`; this injects the *classified* result
    /// (a contract subreason, which derives its category).
    InjectErrorIncident {
        profile: ProfileId,
        view: WebViewId,
        reason: bbr_web::web_errors::IncidentSubReason,
        url: Option<String>,
    },
    /// Press the error surface's primary action (the recovery button, task
    /// packet `error-surface-contract`): routes through the exact
    /// production mouse-down seam the `bbr-error-action` button's listener
    /// calls (`BbrApp::error_recover`), which requests the contract-legal
    /// action for the active view's incident and dispatches the engine
    /// action (Retry re-issues the failing URL).
    ClickErrorAction,
    /// Press the error surface's Dismiss button (task packet
    /// `error-surface-contract`): routes through the exact production
    /// mouse-down seam the `bbr-error-dismiss` button's listener calls
    /// (`BbrApp::error_dismiss`) — a lifecycle event, identity never
    /// touched.
    ClickErrorDismiss,
    /// Activate a visible space row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickSpace {
        space: TestSpaceId,
    },
    /// Activate the New Space row through the production pointer path (stages
    /// the NewSpace command-bar mode).
    ClickSpaceNew,
    /// Drive one synthetic GPUI scroll-wheel event through the production
    /// sidebar swipe handler (`BbrApp::on_sidebar_scroll_wheel` — the exact
    /// method the production `on_scroll_wheel` listener calls; DESIGN.md
    /// §11.3 swipe). `delta` is the raw wheel delta `[x, y]` in natural
    /// pixels and `phase` the touch phase; the handler applies the same
    /// horizontal-dominance filter, phase gating, commit threshold, and
    /// `activate_space` authority as a real trackpad gesture. Public
    /// synthetic GPUI event dispatch is unavailable, so the testkit drives
    /// the production handler directly — never a controller mutation.
    SwipeScroll {
        delta: [f32; 2],
        phase: SwipePhaseWire,
    },
    /// Explicitly cancel a tracking sidebar swipe (`BbrApp::cancel_swipe`):
    /// the machine returns Idle with no commit and the chrome tint returns
    /// to the exact source space presentation.
    SwipeCancel,
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
        id: u64,
    },
    /// Activate a space and measure until its warm selected tab is presented.
    BenchmarkActivateSpace {
        operation_id: BenchmarkOperationId,
        space: TestSpaceId,
    },
    /// Measure the §11.3 create→activate→configure lifecycle (packet
    /// `space-creation-appearance-ui.md`): create a space with `name` through
    /// the production `commit_new_space` authority, activate it immediately,
    /// and commit only when the configuration surface renders open bound to
    /// the exact new persistent `SpaceId` with the created chip selected.
    BenchmarkCreateSpace {
        operation_id: BenchmarkOperationId,
        name: String,
    },
    /// Measure the §11.3 identity/rename commit for the exact bound `space`
    /// (packet `space-creation-appearance-ui.md`): open the configuration
    /// surface through the production management-affordance entry, commit the
    /// curated `identity` through `space_config_done`, and commit only when
    /// the surface is drawn closed with the bound space still active and the
    /// same warm session. The committed chip identity/theme is re-verified
    /// from the typed snapshot by the driver.
    BenchmarkSpaceConfigIdentityUpdate {
        operation_id: BenchmarkOperationId,
        space: TestSpaceId,
        identity: String,
    },
    /// Measure a §11.3 sidebar swipe commit through the production gesture
    /// path: the app drives real `Started`/`Moved` wheel updates plus a
    /// zero-delta `TouchPhase::Ended` through the exact handler the sidebar
    /// `on_scroll_wheel` listener calls, and the ended release resolves into
    /// exactly one existing `activate_space` commit of the adjacent
    /// persistent `destination` (never a direct activation call, never a
    /// benchmark-only state machine). The sample commits only when the
    /// destination space, its warm selected view, the Idle machine phase,
    /// and the unchanged session count are presented — same warm class as
    /// [`Self::BenchmarkActivateSpace`], no session recreation.
    BenchmarkSwipeCommit {
        operation_id: BenchmarkOperationId,
        destination: TestSpaceId,
    },
    /// Measure one committed logical Back content gesture (N1 packet
    /// `web-content-navigation-gestures`): the app dispatches the commit
    /// through the exact production method the native gesture-bridge pump
    /// calls (`BbrApp::on_content_gesture_commit`), and the sample commits
    /// only when the correct URL/history state is presented on the bound
    /// `view` (a deterministic already-loaded two-entry history). The
    /// driver re-verifies the real WebKit state afterwards; external
    /// page/network latency stays outside BBR timings.
    BenchmarkWebHistorySwipeBack {
        operation_id: BenchmarkOperationId,
        view: WebViewId,
    },
    /// Measure one committed Back-at-settled-root opener return (N1 packet
    /// `web-content-navigation-gestures`): the app dispatches the commit
    /// through the exact production method the native gesture-bridge pump
    /// calls, and the sample commits only when the child session is gone
    /// and the exact warm opener is visible/focused. The driver re-verifies
    /// the typed snapshot (tabs/sessions/opener identity) afterwards.
    BenchmarkChildTabSwipeReturn {
        operation_id: BenchmarkOperationId,
        child_view: WebViewId,
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
    /// Measure renderer-hang chrome recovery through production actions.
    /// `close=true` dispatches the normal active-tab close and requires the
    /// exact warm neighbor to be presented with one fewer live session;
    /// `close=false` selects `neighbor_index` and requires the session count
    /// to remain unchanged. The source must already be reported
    /// unresponsive, otherwise the app rejects the sample fast-closed.
    BenchmarkUnresponsiveChrome {
        operation_id: BenchmarkOperationId,
        source: WebViewId,
        neighbor_index: usize,
        expected_neighbor: WebViewId,
        expected_sessions: usize,
        close: bool,
    },
    /// Measure one production responsiveness cadence tick in a proven idle
    /// window (no probe due, no state/session/focus change). Performance code
    /// only until the execution override is lifted.
    BenchmarkProbeTick {
        operation_id: BenchmarkOperationId,
        view: WebViewId,
        expected_sessions: usize,
    },
    /// Dispatch a real GPUI keystroke while measuring its presentation commit.
    BenchmarkDispatchKeystroke {
        operation_id: BenchmarkOperationId,
        kind: BenchmarkKind,
        keystroke: String,
    },
    /// Drive a [`WebAction`] on the active web view (Back/Forward/Reload/…).
    WebAction(WebAction),
    /// Zoom the active view's site one contract step up (queue item 11,
    /// part A; the exact seam the `Cmd-Plus` key binding routes to). Never
    /// creates, moves, or reselects a web session.
    ZoomIn,
    /// Zoom the active view's site one contract step down (the `Cmd-Minus`
    /// seam).
    ZoomOut,
    /// Reset the active view's site to the default zoom level (the `Cmd-0`
    /// seam); removes the override record.
    ZoomReset,
    /// Pointer-hover/select path into the open BBR context menu (task
    /// packet `bbr-context-menus-over-web`): the exact app method the row's
    /// `on_hover` element listener calls (the keyboard cursor and the
    /// pointer hover share one selection). Disabled entries are skipped by
    /// the keyboard traversal; this seam selects whatever row the pointer
    /// is over.
    ContextMenuSelect {
        index: usize,
    },
    /// Pointer-press path into the open BBR context menu: the exact app
    /// method the row's mouse-down listener calls — selects the row and
    /// activates it (Back/Forward/Reload/Copy Link/Open in New Tab), then
    /// dismisses the menu and restores the origin focus domain.
    ContextMenuActivate {
        index: usize,
    },
    /// Dismiss the open BBR context menu through the production close path
    /// (the portal backdrop's pointer press, the Escape key, and this seam
    /// all call the same app method): the menu closes and focus returns to
    /// the domain captured at open. No-op when no menu is open.
    ContextMenuClose,
    /// Secondary-click on a visible tab row through the production pointer
    /// path (task packet `tab-favorite-context-menus`): the exact app
    /// method the row's right-button mouse-down listener calls. Opens the
    /// Tab/Favorite menu bound to that exact row's `TabId`, anchored at
    /// `(x, y)` in window content coordinates; never selects or mutates the
    /// source. A press on another row while the menu is open re-binds it.
    RightClickRow {
        index: usize,
        x: f32,
        y: f32,
    },
    /// Secondary-click on a favorite tile through the production pointer
    /// path: opens the same menu bound to that tile's exact `TabId` (the
    /// Favorite is another presentation of the same concrete tab); never
    /// activates the tile. `(x, y)` in window content coordinates.
    RightClickFavorite {
        index: usize,
        x: f32,
        y: f32,
    },
    /// Pointer-hover/select path into the open Tab/Favorite menu: the exact
    /// app method the row's `on_hover` element listener calls (the keyboard
    /// cursor and the pointer hover share one selection). No-op when the
    /// menu is closed.
    TabMenuSelect {
        index: usize,
    },
    /// Pointer-hover/select path into the open Tab/Favorite submenu: the
    /// exact app method a submenu row's `on_hover` listener calls. No-op
    /// when no submenu is open.
    TabMenuSubmenuSelect {
        index: usize,
    },
    /// Pointer-press path into the open Tab/Favorite menu: the exact app
    /// method the row's mouse-down listener calls — selects the row and
    /// activates it (each enabled action dispatches exactly one typed
    /// command against the bound `TabId`), then dismisses the menu and
    /// restores the origin focus domain. A disabled row never activates;
    /// like a native menu it dismisses the menu.
    TabMenuActivate {
        index: usize,
    },
    /// Pointer-press path into the open Tab/Favorite submenu: commits the
    /// submenu row through the same typed-command authority, then dismisses
    /// the menu. No-op when no submenu is open.
    TabMenuSubmenuActivate {
        index: usize,
    },
    /// Dismiss the open Tab/Favorite menu through the production close path
    /// (the Escape key, an outside pointer press, and this seam all call the
    /// same app method): the menu closes and focus returns to the domain
    /// captured at open. No-op when no menu is open.
    TabMenuClose,
    /// A pointer press outside the open Tab/Favorite menu (the §13
    /// one-pointer-action dismissal): the exact app method the root
    /// mouse-down listener calls — one outside press dismisses the menu and
    /// restores the origin focus domain. No-op when no menu is open.
    TabMenuOutsideClick,
    /// Focus the GPUI command bar (chrome) — the E2E reader of Cmd-L.
    ActivateCommandBar,
    /// Pointer press on the §11.5 address control through the production
    /// pointer-entry path: the same app method the pill's mouse-down
    /// listener calls (DESIGN.md §11.4 pointer entry into the overlay that
    /// covers the control, same engine as Cmd-L). The address control is a
    /// display-only button — it never accepts keyboard text.
    ClickAddress,
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
    /// Deterministic persistence flush (testkit-only): synchronously writes
    /// the current in-memory command-history provider to the store in one
    /// transaction — the deterministic seam E2E uses to prove persistence
    /// across restarts (the harness terminates the app, so a clean quit
    /// flush cannot be observed; the production background writer uses the
    /// identical repository write).
    FlushPersistence,
    /// Deterministic browser-state flush (testkit-only, Phase 7 warm-restore
    /// slice): synchronously writes the current structural browser model
    /// (profiles/spaces/tabs/window+sidebar) to the store in one transaction
    /// — the deterministic seam the persistence/restart E2E uses to prove
    /// committed-state recovery across restarts (the harness terminates the
    /// app, so a clean quit flush cannot be observed; the production
    /// background writer uses the identical repository write).
    FlushBrowserState,
    /// Explicitly arm one real renderer-liveness probe for `view` through
    /// the same nonblocking controller/adapter path the production cadence
    /// uses. The result arrives asynchronously as
    /// `WebEvent::UnresponsiveChanged`.
    ProbeRendererResponsiveness {
        view: WebViewId,
    },
    /// Activate the §11.8a Wait/Reload/Close controls through the exact app
    /// methods their production pointer listeners call. These are product
    /// actions, not controller mutations, and exist so real-app E2E can drive
    /// recovery without depending on synthetic platform Tab traversal.
    ClickUnresponsiveWait,
    ClickUnresponsiveReload,
    ClickUnresponsiveClose,
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
    /// Blueprint D08: route a pointer at `(x, y)` (window points) through
    /// the Portal's hit-region decision — the exact path a real pointer
    /// move takes — without moving the user's cursor. The snapshot's
    /// `portal.ignores_mouse_events` reports the result.
    PortalRoutePointer {
        x: f32,
        y: f32,
    },
    /// Blueprint D02: drive the key window's `NSTextInputClient` exactly as
    /// an input method does (compose, commit, unmark) and read back the
    /// client's ranges and candidate-window rect into the snapshot's
    /// `command_input.native`. Runs outside any app borrow, on the pump.
    CommandTextInput {
        op: TextInputOp,
    },
    /// Blueprint D02: one pointer sample into the floating rail's reveal
    /// model — the exact method the native pointer observer feeds (window
    /// points; `time_ms` is the event time the velocity is measured on).
    RailPointer {
        x: f32,
        y: f32,
        time_ms: f64,
        pressed: bool,
    },
    /// Blueprint D16: open the Settings window through its production
    /// method (the ⌘, action and the BBR menu item call it too).
    OpenSettings,
    /// Blueprint D16: one Settings interaction through the method the
    /// window's controls call.
    SettingsInput {
        input: SettingsInput,
    },
    /// Blueprint D16: close the Settings window (its close button path).
    CloseSettings,
    /// Blueprint D10: open History through its production method (⌘Y,
    /// the History menu and the command bar call it too).
    OpenHistory,
    /// Blueprint D10: one History interaction through the method the
    /// page's controls call.
    HistoryInput {
        input: super::HistoryInput,
    },
    /// Blueprint D11: one permission-prompt or site-settings interaction
    /// through the method its control calls.
    PermissionInput {
        input: super::PermissionInput,
    },
    /// Blueprint D19: move browser window `window` by `(dx, dy)` screen
    /// points (the harness's windows are never shown, so they cascade on
    /// top of each other; a drag between them needs them apart).
    MoveWindowBy {
        window: u64,
        dx: f64,
        dy: f64,
    },
    /// Blueprint D19: make `window` the current window (as input in it
    /// does): later requests, engine events and timers act in it.
    SelectWindow {
        window: u64,
    },
    /// Blueprint D23: one split interaction.
    SplitInput {
        input: super::SplitInput,
    },
    /// Blueprint D24: one media-card press (or a raw command for an
    /// explicit session identity).
    MediaInput {
        input: super::MediaInput,
    },
    /// Blueprint D12: one client-certificate sheet interaction.
    ClientCertificateInput {
        input: super::ClientCertificateInput,
    },
    /// Blueprint D13: one open-in-app sheet interaction.
    OpenAppInput {
        input: super::OpenAppInput,
    },
    /// Blueprint D15: one first-run control.
    WelcomeInput {
        input: super::WelcomeInput,
    },
    /// Blueprint D19: invoke the menu item `menu › item` (`menu` = "Dock"
    /// for the Dock menu) through the same action dispatch an `NSMenuItem`
    /// click takes.
    InvokeMenuItem {
        menu: String,
        item: String,
    },
    /// Blueprint D17: press the recovery pane's action through its
    /// production method.
    RecoveryAction,
    /// Blueprint D17: the restore notice's Dismiss (its production method).
    DismissRestoreNotice,
    /// Blueprint D08: press the toast's action (e.g. Undo) through the
    /// production method its button calls.
    ToastAction,
    /// Deterministically tear down the one native portal host (task packet
    /// teardown evidence): `BbrApp::portal_teardown` removes the child
    /// relation, orders the portal window out, closes it, and drops the
    /// bridge — the app owns no portal afterwards. Terminal; the snapshot
    /// afterwards reports the empty/closed host.
    PortalTeardown,
    /// Drive the parent's real AppKit fullscreen transition
    /// (`toggleFullScreen:`) — native-portal-hardening driver. The E2E
    /// polls the typed `parent_fullscreen` fact plus frame sync until the
    /// Window Server settles the transition; the host must re-pin to the
    /// fullscreen content rect without recreating or orphaning the portal.
    PortalSetFullscreen {
        fullscreen: bool,
    },
    /// Drive the parent's real AppKit minimize/restore
    /// (`miniaturize:` / `deminiaturize:`) — native-portal-hardening
    /// driver. While minimized the portal must be ordered out (never
    /// occluding/intercepting) and re-shown on restore when a presentation
    /// is still armed.
    PortalSetMinimized {
        minimized: bool,
    },
    /// Drive a real Window Server space-assignment transition on the parent
    /// (set/clear the `CanJoinAllSpaces` collection behavior) —
    /// native-portal-hardening driver. The parent and its attached child
    /// must stay pinned and agree on active-space membership across the
    /// transition.
    PortalSetAllSpaces {
        all_spaces: bool,
    },
    /// Close the parent browser window through the real AppKit close path
    /// (`performClose:`) while a portal presentation is open —
    /// native-portal-hardening driver. The host's will-close observer must
    /// tear the portal down deterministically: no BBR window may remain on
    /// screen afterwards (never orphaned, never occluding, never
    /// intercepting input).
    PortalCloseWindow,
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
    /// §11.14 titlebar navigation-control commit (packet
    /// `titlebar-navigation-controls`): measure one Back/Forward/Reload
    /// dispatch through the exact production authority the titlebar click
    /// and the Cmd-[ / Cmd-] / Cmd-R shortcuts share. `control` is the §15
    /// id suffix (`back` / `forward` / `reload`); the sample commits only
    /// when the genuine availability precondition held and the production
    /// dispatch succeeded (the driver waits for the real WebKit history
    /// state afterwards).
    BenchmarkNavigationControlCommit {
        operation_id: BenchmarkOperationId,
        control: String,
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
    /// Measure the same tab move through the production drag-to-chip path:
    /// the concrete row `from_index` is pressed, the pointer is moved onto
    /// the destination space's rendered chip (the exact production chip-hit
    /// geometry), and the release commits through the drag drop → the
    /// existing move authority. Same typed kind/report key/budget class as
    /// [`Request::BenchmarkMoveTab`]; only the measured production entry is
    /// the drag-to-chip path rather than the direct move command.
    BenchmarkDragMoveTab {
        operation_id: BenchmarkOperationId,
        from_index: usize,
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
    /// Measure the production command-text path against the history-rich
    /// corpus (Phase 7, command-history provider; queue item 16,
    /// `command_bar_first_result`): identical to `BenchmarkCommandType`
    /// except the expected first result is a ranked history row. The sample
    /// commits only from a render of the result surface presenting the
    /// exact first/selected key, generation, state, mode, and focus.
    BenchmarkCommandBarFirstResult {
        operation_id: BenchmarkOperationId,
        text: String,
        expected_first_key: String,
    },
    /// Measure the production command-text path against the same history-rich
    /// corpus for the *complete ranked result-set presentation* (queue item
    /// 16, `command_bar_full_search`): the query matches more rows than the
    /// limit-16 window, so the expected identity is the exact full row-key
    /// window plus the truthful total match count. The sample commits only
    /// from a render of the result surface presenting exactly that key set
    /// and total (fast-but-wrong guard: a partial, reordered, truncated, or
    /// stale window or a wrong total rejects the measurement).
    BenchmarkCommandBarFullSearch {
        operation_id: BenchmarkOperationId,
        text: String,
        expected_keys: Vec<String>,
        expected_total: usize,
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
    /// Measure the production Cmd-L / address-pill click open of the
    /// field-anchored overlay (DESIGN.md §11.4 `bbr-command-overlay`) in
    /// Location mode: the sample commits only from a render that actually
    /// constructed the overlay with the full URL preselected and chrome
    /// focus (fast-but-wrong guard). No tab or session is created before
    /// Enter.
    BenchmarkOverlayOpen {
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
    /// Measure the production `Cmd-Plus` zoom step (queue item 11, part A):
    /// the exact seam the key binding routes to — the lifecycle change, the
    /// persistence write, and the native `pageZoom` application on the
    /// active view. The sample commits only from the app's observed zoom
    /// state at the armed level — the warm-interaction class, no tab or
    /// session is touched.
    BenchmarkZoomStep {
        operation_id: BenchmarkOperationId,
        level: u16,
    },
    /// Measure the production BBR context-menu open over the active web
    /// view (task packet `bbr-context-menus-over-web`): the exact
    /// event-driven open path the native bridge's `ContextMenuRequested`
    /// routes to (`menu_open_from_event`, anchored at `point` in window
    /// content coordinates). The sample commits only from a render of the
    /// one `NativeAboveWeb` portal window that drew the menu over the same
    /// active view with the armed pointer anchor — the warm-presentation
    /// class, no tab or session is touched.
    BenchmarkContextMenuOpen {
        operation_id: BenchmarkOperationId,
        view: WebViewId,
        point: [f32; 2],
    },
    /// Measure the production Tab/Favorite context-menu open over the
    /// inactive tab row at `row` (task packet
    /// `tab-favorite-context-menus`): the exact app method the row's
    /// right-button mouse-down listener routes to
    /// (`open_tab_menu_from_row`, anchored at `point` in window content
    /// coordinates). The sample commits only from a main-window render that
    /// drew the menu open, focused, and bound to the exact source `TabId`
    /// at the armed anchor — the warm-presentation class; the source
    /// tab/session/selection is never touched.
    BenchmarkTabMenuOpen {
        operation_id: BenchmarkOperationId,
        row: usize,
        point: [f32; 2],
    },
    /// Measure the production Tab/Favorite menu submenu commit (task packet
    /// `tab-favorite-context-menus`): the menu must already be open bound
    /// to the source `TabId` (the driver opens it unmeasured between
    /// samples); the sample starts at the exact hover seam that opens the
    /// one-level submenu card for `parent_row` (`tab_menu_select`) and
    /// commits only from a main-window render that drew the open submenu
    /// card bound to the same source — the warm-presentation class, source
    /// identity preserved.
    BenchmarkTabMenuSubmenuCommit {
        operation_id: BenchmarkOperationId,
        parent_row: usize,
    },
    /// Measure the production bookmark-row activation of the enriched page
    /// menu (task packet `page-background-context-menu.md`): the sample
    /// starts at the exact `menu_activate` seam a pointer press on the
    /// `Bookmark Page` / `Remove Bookmark` row dispatches — B2's one
    /// app-owned `SavedPageCommand::toggle` against the captured
    /// focused-page URL/title plus the menu dismissal. The menu must be
    /// open over `view` at `point`; the sample commits only from a render
    /// that drew the menu closed over the same active view (fast-but-wrong
    /// guard) with the store truth verified at the mutation site. The
    /// warm-interaction class, no tab or session is created or touched.
    BenchmarkContextMenuBookmark {
        operation_id: BenchmarkOperationId,
        view: WebViewId,
        point: [f32; 2],
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
    /// Measure the production **restart** row action on a cancelled row
    /// (the settled §17.8 decision): `controller.retry` on a `Cancelled`
    /// attempt mints a fresh attempt linked via `retry_of`, presented in
    /// the Active group — the same warm-presentation class as the failed
    /// retry.
    BenchmarkDownloadsRowRestart {
        operation_id: BenchmarkOperationId,
        row: usize,
    },
    /// Measure the production row Delete action (the settled §17.8
    /// decision): the cancelled row's `download_delete_row` seam removes
    /// exactly that one terminal record and persists the remaining history
    /// durably; the sample commits when the panel render no longer draws
    /// the deleted row — warm-presentation class.
    BenchmarkDownloadsRowDelete {
        operation_id: BenchmarkOperationId,
        row: usize,
    },
    /// Measure the production Clear panel action (the settled §17.7
    /// decision): `downloads_clear` empties the controller history and
    /// persists the empty slice durably; the sample commits when the panel
    /// render draws the empty state (no rows) — warm-presentation class.
    BenchmarkDownloadsClear {
        operation_id: BenchmarkOperationId,
    },
    /// Measure the production error-surface presentation (task packet
    /// `error-surface-contract`): a classified incident enters through the
    /// exact seam the native incident sink drives (`on_incident` → the
    /// contract `present` path) and the sample commits only from a render
    /// that drew the error surface for the armed category over the unchanged
    /// active view — warm-presentation class, no tab or session is created
    /// or touched. Fixture/network time is excluded by construction (the
    /// incident is applied in-process, never through a network transfer).
    BenchmarkErrorSurfacePresent {
        operation_id: BenchmarkOperationId,
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
    /// Seed a deterministic non-transferring **cancelled** terminal attempt
    /// for the restart/delete performance scenarios: the same in-process
    /// seeding as [`Self::SeedDownloadsAttempt`], then the engine's terminal
    /// `Cancelled` event through the production event path (no network
    /// transfer runs). The driver reads the minted id from the snapshot.
    SeedDownloadsCancelled {
        url: String,
        destination: String,
        total: u64,
    },
    /// U2 footer, favorites, and folders production pointer seams. There
    /// is no `[Spaces]` toggle request: the §11.3 space strip is always
    /// part of the rail and space activation routes through `ClickSpace`.
    ClickFooterDownloads,
    ClickFooterPlus,
    ClickPlusMenuItem {
        index: usize,
    },
    /// §11.3 Space-configuration surface seams (packet
    /// `space-creation-appearance-ui.md`): the management-affordance entry
    /// opens the surface for the exact persistent space (the production
    /// create paths also open it after creation); the draft seams select
    /// the curated theme/identity in the open draft; Done commits the
    /// rename + appearance for the bound SpaceId through the production
    /// authorities; Cancel closes without applying (the created Space and
    /// its valid defaults stay); `SpaceConfigPressOutside` drives the exact
    /// production outside-press dismissal.
    OpenSpaceConfig {
        space: TestSpaceId,
    },
    SpaceConfigSelectTheme {
        theme: String,
    },
    SpaceConfigSelectIdentity {
        identity: String,
    },
    /// v2 D7/D6 paper draft seams: select a curated intensity step by its
    /// canonical name; toggle the static grain.
    SpaceConfigSelectIntensity {
        intensity: String,
    },
    SpaceConfigToggleGrain,
    SpaceConfigDone,
    SpaceConfigCancel,
    SpaceConfigPressOutside,
    /// A primary press on bare chrome (no control under the pointer): runs
    /// the window root's production left-press handler, which dismisses
    /// every open outside-press surface — and commits an open inline
    /// folder rename (blueprint D20, DESIGN §11.11).
    ChromeBackgroundPress,
    /// Blueprint D05: a secondary press on the space chip of `space` at the
    /// pointer `(x, y)` opens the space menu (the exact production listener
    /// method). Its rows then take the `TabMenu*` seams.
    RightClickSpaceChip {
        space: TestSpaceId,
        x: f32,
        y: f32,
    },
    /// Blueprint D25: a secondary press on the folder row of `folder` at
    /// `(x, y)` opens the folder menu (the exact production listener
    /// method). Its rows then take the `TabMenu*` seams.
    RightClickFolderRow {
        folder: u64,
        x: f32,
        y: f32,
    },
    /// Blueprint D05: the Delete Space sheet's Cancel / Delete Space button
    /// paths (the production app methods the portal buttons call).
    SpaceDeleteCancel,
    SpaceDeleteConfirm,
    ToggleFavoriteTab {
        index: usize,
    },
    ClickFavoriteTile {
        index: usize,
    },
    ClickFolderRow {
        id: u64,
    },
    ClickFolderToggle {
        id: u64,
    },
    ClickFolderMember {
        folder: u64,
        member: usize,
    },
    ClickFolderMemberClose {
        folder: u64,
        member: usize,
    },
    MoveTabIntoFolder {
        index: usize,
        folder: u64,
    },
    /// Ungroup the active space's tab at `index` from `folder` through the
    /// organization authority: the member stays a loose pinned tab.
    RemoveTabFromFolder {
        index: usize,
        folder: u64,
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
    /// fresh attempt id linked via `retry_of`. The same seam restarts a
    /// cancelled attempt (the settled §17.8 decision); the panel labels the
    /// machine path differently by state.
    DownloadRetry {
        id: u64,
    },
    /// Activate a visible download row's contextual action through the
    /// production pointer path (the exact app method the row's action
    /// mouse-down listener calls): Cancel / Retry / Restart /
    /// Reveal-in-Finder.
    ClickDownloadRowAction {
        row: usize,
    },
    /// The settled row Delete action (DESIGN.md §17.8): remove exactly one
    /// terminal record through the production `download_delete_row` seam
    /// (the exact app method the row's Delete control mouse-down listener
    /// calls) — durable, so the deleted row can never return from the
    /// store.
    ClickDownloadRowDelete {
        row: usize,
    },
    /// The settled Clear panel action (DESIGN.md §17.7): clear the
    /// controller history and the repository durably through the production
    /// `downloads_clear` seam (the exact app method the panel's Clear
    /// control mouse-down listener calls).
    ClickDownloadsClear,
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
    /// Seed a deterministic authentication challenge through the production
    /// challenge-arrival seam (`BbrApp::auth_on_challenge`): records the
    /// challenge bound to `web_view` and presents the credential sheet over
    /// the active window (queue item 10). Carries only authority metadata
    /// (host/port/realm/scheme/authority) — never credential text;
    /// credentials are typed through the production key path, so the only
    /// representation that ever leaves the sheet is the contract's redacted
    /// handle.
    AuthPresentChallenge {
        profile: ProfileId,
        web_view: WebViewId,
        http_scheme: HttpScheme,
        scheme: AuthScheme,
        host: String,
        port: u16,
        realm: Option<String>,
        authority: AuthAuthority,
        user_initiated: bool,
    },
    /// Testkit-only deterministic seeding for the auth performance scenario:
    /// record a challenge WITHOUT presenting the sheet (the awaiting
    /// challenge the measured open presents). Carries only authority
    /// metadata, never credentials.
    AuthRecordChallenge {
        profile: ProfileId,
        web_view: WebViewId,
        http_scheme: HttpScheme,
        scheme: AuthScheme,
        host: String,
        port: u16,
        realm: Option<String>,
        authority: AuthAuthority,
        user_initiated: bool,
    },
    /// Measure the production credential-sheet open (queue item 10): the
    /// sample commits only from a render of the portal window that drew the
    /// sheet presenting the awaiting challenge (fast-but-wrong guard). The
    /// awaiting challenge is the fixture, recorded before the sample;
    /// fixture/network time is excluded by construction.
    BenchmarkAuthSheetOpen {
        operation_id: BenchmarkOperationId,
    },
    /// Save the canonical URL as a Saved Page through the one app-owned
    /// command authority (B2 packet `saved-pages-actions-command`): the
    /// same `SavedPageCommand::save` seam Cmd-D and the menu rows share.
    /// Canonical-URL-idempotent: a duplicate returns the existing
    /// `BookmarkId` and never creates a second record. `title` is the
    /// captured title (the display ladder falls back to the URL when
    /// empty).
    SaveSavedPage {
        url: String,
        title: String,
    },
    /// Probe the availability of `Bookmark Page` and `Remove Bookmark` for
    /// `url` through the one app-owned command authority (B2 packet
    /// `saved-pages-actions-command`): the availability matrix (saved/
    /// unsaved, navigable/invalid/missing URL) without mutating source
    /// state. Replies with [`Response::SavedPageAvailability`].
    SavedPageAvailability {
        url: String,
    },
    /// Remove the Saved Page record by stable `BookmarkId` through the one
    /// app-owned command authority (idempotent: an unknown or already-
    /// removed identity is a no-op success). Never mutates a tab.
    RemoveSavedPage {
        id: u64,
    },
    /// Open the Saved Page record by stable `BookmarkId` through the one
    /// app-owned command authority: the record is revalidated in the
    /// in-memory provider, then exactly one fresh regular tab opens in the
    /// currently active Space/Profile through the existing NewTab
    /// authority. Never silently selects an existing matching tab.
    OpenSavedPage {
        id: u64,
    },
    /// Deterministic Saved Pages flush (testkit-only): synchronously
    /// persists the current in-memory Saved Page provider to the store in
    /// one transaction — the deterministic seam the restart E2E uses to
    /// prove committed Saved Pages across restarts (the harness terminates
    /// the app, so a clean quit flush cannot be observed; the production
    /// background writer uses the identical repository write).
    FlushSavedPages,
    /// Measure the production command-text path against the Saved Page
    /// provider (B2 packet `saved-pages-actions-command`,
    /// `saved_page_search_first_result`): identical to
    /// `BenchmarkCommandBarFirstResult` except the expected first result is
    /// a ranked Saved Page row (`saved_page:<id>`) over the seeded
    /// ≥1,000-record provider. The sample commits only from a render of the
    /// result surface presenting the exact first/selected key, generation,
    /// state, mode, and focus — the warm command-type class (50/100).
    BenchmarkSavedPageSearchFirstResult {
        operation_id: BenchmarkOperationId,
        text: String,
        expected_first_key: String,
    },
    /// Measure the production open of a Saved Page result (B2 packet
    /// `saved-pages-actions-command`, `saved_page_open`): the record is
    /// revalidated in the in-memory provider and exactly one fresh regular
    /// tab/session opens in the currently active Space/Profile through the
    /// existing NewTab authority — the session-recreation class (100/150).
    /// The sample commits only when the fresh tab/session is presented and
    /// focused with the surface closed.
    BenchmarkSavedPageOpen {
        operation_id: BenchmarkOperationId,
        bookmark_id: u64,
    },
    /// Measure the production Saved Pages Library search (B3 packet
    /// `saved-pages-library`, `saved_pages_library_search`): the typed
    /// search entry (`library_set_search`, shared with the keyboard path)
    /// filters the loaded ≥1,000-record provider and the bounded window
    /// re-renders. The sample commits only when the rendered model
    /// presents the exact armed query result count and window — a
    /// missing/wrong filter or a stale window rejects the measurement
    /// (fast-but-wrong guard). Warm next-frame presentation class (50/100).
    BenchmarkSavedPagesLibrarySearch {
        operation_id: BenchmarkOperationId,
        text: String,
        expected_total: usize,
        expected_first: usize,
        expected_last: usize,
    },
    /// Measure the production Saved Pages Library scroll (B3 packet
    /// `saved-pages-library`, `saved_pages_library_scroll`): the tracked
    /// scroll offset (`library_set_scroll`, the seam the wheel listener
    /// drives) re-anchors the bounded window over the loaded ≥1,000-record
    /// provider. The sample commits only when the rendered model presents
    /// the exact armed window/anchor and preserves the exact selection set
    /// — a no-op scroll, a wrong window/anchor, or a corrupted selection
    /// rejects the measurement (fast-but-wrong guard). Same warm
    /// next-frame presentation class (50/100).
    BenchmarkSavedPagesLibraryScroll {
        operation_id: BenchmarkOperationId,
        offset: usize,
        expected_first: usize,
        expected_last: usize,
        expected_anchor: u64,
    },
    /// Open the Saved Pages Library through its integrated production entry
    /// (B3 packet `saved-pages-library`): the internal chrome surface
    /// covers the content area, the active view's native slot is hidden
    /// (never destroyed), and the Library owns keyboard focus.
    OpenSavedPagesLibrary,
    /// Close the Saved Pages Library (Escape / the close control / this
    /// request): the exact native view the Library hid is restored and
    /// focus returns to the page.
    CloseSavedPagesLibrary,
    /// Set the Library search query through the production search entry
    /// (resets the scroll offset; the deterministic in-memory filter).
    SetSavedPagesLibrarySearch {
        text: String,
    },
    /// Cycle the deterministic Library sort order.
    CycleSavedPagesLibrarySort,
    /// Toggle one row's multi-select membership.
    ToggleSavedPagesLibrarySelect {
        id: u64,
    },
    /// Clear the Library multi-select set.
    ClearSavedPagesLibrarySelection,
    /// Set the tracked production scroll offset (rows scrolled past); the
    /// bounded render window derives from it.
    SetSavedPagesLibraryScroll {
        offset: usize,
    },
    /// Stage the destructive remove confirmation for the target ids (the
    /// multi-select removal contract: a clear confirmation before any
    /// record is removed; never touches an open tab).
    BeginSavedPagesLibraryRemove {
        ids: Vec<u64>,
    },
    /// Commit the staged destructive remove through the one AppStore
    /// authority.
    ConfirmSavedPagesLibraryRemove,
    /// Cancel the staged destructive remove.
    CancelSavedPagesLibraryRemove,
    /// Rename one record's user label through the one AppStore authority
    /// (the B3 Library rename; invalid labels are a no-op at the store).
    RenameSavedPagesLibraryRow {
        id: u64,
        label: String,
    },
    /// Enter the pointer rename entry for one row: stage the inline rename
    /// input seeded with the record's current user label (the B3 pointer
    /// rename flow; the production row rename control routes here).
    BeginSavedPagesLibraryRowRename {
        id: u64,
    },
    /// Type into the staged rename draft (the inline input's text model;
    /// the production key handler routes printable input here).
    SetSavedPagesLibraryRenameDraft {
        text: String,
    },
    /// Confirm the staged rename: commit the non-empty draft through the
    /// one existing `RenameSavedPagesLibraryRow` authority and leave the
    /// inline input.
    ConfirmSavedPagesLibraryRename,
    /// Cancel the staged rename: leave the inline input without committing
    /// (the record's label is untouched).
    CancelSavedPagesLibraryRename,
    /// One flat-collection mutation through the one AppStore authority
    /// (create / rename / remove; removal moves members out first).
    SavedPagesLibraryCollection {
        op: SavedPagesLibraryCollectionOp,
    },
    /// Move the current multi-select into (or out of) one flat collection
    /// through the one AppStore authority (`None` clears the membership).
    MoveSavedPagesLibrarySelection {
        collection: Option<u64>,
    },
    /// The selection toolbar Remove control (the pointer path the
    /// `bbr-saved-pages-selection-remove` control calls): stage the
    /// destructive remove confirmation for exactly the current selection.
    ClickSavedPagesLibrarySelectionRemove,
    /// The selection toolbar Move control (the pointer path the
    /// `bbr-saved-pages-selection-move` control calls): toggle the move-to
    /// sheet.
    ClickSavedPagesLibrarySelectionMove,
    /// The selection toolbar Clear control (the pointer path the
    /// `bbr-saved-pages-selection-clear` control calls): clear the whole
    /// multi-select set.
    ClickSavedPagesLibrarySelectionClear,
    /// A move-to sheet option (the pointer path the per-collection and
    /// "No collection" sheet rows call): move the current multi-select
    /// into (or out of) one flat collection (`None` clears the
    /// membership) through the one AppStore authority.
    ClickSavedPagesLibraryMoveOption {
        collection: Option<u64>,
    },
    /// A flat collection row click (the pointer path the
    /// `bbr-saved-pages-collection-<id>` row calls): stage the
    /// deterministic rename → remove → none cycle for the collection.
    ClickSavedPagesLibraryCollectionRow {
        id: u64,
    },
    /// The collection create entry (the pointer path the
    /// `bbr-saved-pages-collection-create` control calls): stage the
    /// inline create input.
    ClickSavedPagesLibraryCollectionCreate,
    /// Commit whatever collection action is staged (create / rename /
    /// remove) through the one AppStore authority (the staged Confirm
    /// controls' path).
    ConfirmSavedPagesLibraryCollection,
    /// Leave the staged collection action without committing (the staged
    /// Cancel controls' path).
    CancelSavedPagesLibraryCollection,
    /// Set the staged collection rename/create draft directly (the typed
    /// seam of the staged inline input; the production keyboard routes
    /// printable input into the draft through the same app state).
    SetSavedPagesLibraryCollectionDraft {
        text: String,
    },
    /// Scroll the bounded Library list by `delta_rows` (row units) through
    /// the exact wheel clamp math the production list wheel listener runs
    /// and the one production scroll entry (`library_set_scroll`).
    ScrollSavedPagesLibraryWheel {
        delta_rows: i64,
    },
    /// Open one Library row through the B2 Saved Page command authority:
    /// exactly one fresh regular tab in the currently active Space/Profile
    /// — never a silent selection of an existing matching tab.
    ActivateSavedPagesLibraryRow {
        id: u64,
    },
    /// Capture the current production GPUI frame directly from its renderer
    /// into a PNG under `/tmp`. Testkit-only visual evidence: the authenticated
    /// local gateway exposes no arbitrary path or native-web screenshot API.
    CaptureChrome {
        path: String,
    },
    /// Full app snapshot.
    GetSnapshot,
    /// Force the effective production `ThemeMode` regardless of the host
    /// appearance (task packet `system-appearance-theme`): `None` clears
    /// the override and returns to the observed macOS system appearance.
    /// The §15 snapshot `theme` field reports the effective mode, so E2E
    /// forces light and dark deterministically and asserts the flip
    /// re-presented the complete token set.
    SetAppearanceOverride {
        mode: Option<ThemeModeOverride>,
    },
    /// Force macOS Increase Contrast and/or Reduce Motion regardless of the
    /// host (task packet `v2-system-prefs`): each `None` returns that option
    /// to the observed system value. The §15 snapshot `contrast` and
    /// `reduced_motion` fields report the effective values.
    SetAccessibilityOverride {
        increase_contrast: Option<bool>,
        reduce_motion: Option<bool>,
    },
    /// Present the shell as if the window were in native full screen (or
    /// not), regardless of the AppKit state (DESIGN §2 full-bleed card):
    /// programmatic `toggleFullScreen:` never completes on this host's
    /// Window Server, so E2E proves the full-screen presentation and native
    /// clip through this seam; `None` follows the real window.
    SetFullScreenOverride {
        full_screen: Option<bool>,
    },
    /// Press the §17.14 address zoom chip through the exact app method its
    /// pointer listener calls (resets the active site to 100 %).
    ClickAddressZoom,
    /// Liveness probe.
    Ping,
    /// Close the browser window through GPUI's production window-removal
    /// path. BBR must terminate when its browser window closes, including a
    /// native macOS traffic-light close.
    CloseWindow,
    /// Graceful app shutdown (`quit()`, window closes, process exits clean).
    Quit,
}

/// One flat-collection mutation the Library composes through the one
/// AppStore authority (B3 packet `saved-pages-library`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedPagesLibraryCollectionOp {
    Create { title: String },
    Rename { id: u64, title: String },
    Remove { id: u64 },
}

/// Blueprint D02: one input-method message to the command field
/// (UTF-16 ranges as `[start, end]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextInputOp {
    /// `setMarkedText:selectedRange:replacementRange:` with `selected`
    /// relative to `text`.
    SetMarked { text: String, selected: [usize; 2] },
    /// `insertText:replacementRange:`.
    Insert { text: String },
    /// `unmarkText`.
    Unmark,
    /// Only read the client back.
    Probe,
}

/// Blueprint D16: one Settings interaction (row ids without the
/// `bbr-settings-row-` prefix; panes by key).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsInput {
    Pane {
        pane: String,
    },
    Toggle {
        row: String,
    },
    Choose {
        row: String,
        index: usize,
    },
    Press {
        row: String,
    },
    OpenSelect {
        row: String,
    },
    Dismiss,
    /// Blueprint D06: a press on a row, and the profile sheet.
    Select {
        row: String,
    },
    SheetField {
        index: usize,
    },
    SheetSwatch {
        index: usize,
    },
    SheetCancel,
    SheetConfirm,
    /// A key in the Settings window (`text` for printable input).
    Key {
        key: String,
        text: Option<String>,
    },
}
