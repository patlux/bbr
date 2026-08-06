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
    Attach { since: Option<u64> },
    /// Navigate the active web view to a URL.
    Navigate { url: String },
    /// Create an isolated profile with one initial space.
    CreateProfile {
        profile_title: String,
        space_title: String,
    },
    /// Create another workspace in an existing profile.
    CreateSpace { profile: ProfileId, title: String },
    /// Activate a workspace by stable id.
    ActivateSpace { space: TestSpaceId },
    /// Open a new tab and make it active.
    OpenTab { url: String },
    /// Select an existing tab by active-space index (semantic setups).
    SelectTab { index: usize },
    /// Close a tab by active-space index.
    CloseTab { index: usize },
    /// Reopen the most recently closed tab (fresh session, preserved identity).
    ReopenTab,
    /// Pin the tab at the active-space index (moves into the pinned region).
    PinTab { index: usize },
    /// Unpin the tab at the active-space index (moves to the end of the
    /// regular region).
    UnpinTab { index: usize },
    /// Reorder the tab at `from` to `to` within the active space (same
    /// region only; the pinned-first invariant is domain-owned).
    ReorderTab { from: usize, to: usize },
    /// Activate a visible tab row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickTab { index: usize },
    /// Activate the visible tab row's close control through the production
    /// pointer path.
    ClickTabClose { index: usize },
    /// Toggle the visible tab row's pin control through the production
    /// pointer path (the pointer half of the pin/unpin command).
    ClickTabPin { index: usize },
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
    MoveTabDrag { dy: f32 },
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
    InjectWebEvent { event: bbr_web::WebEvent },
    /// Activate a visible space row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickSpace { space: TestSpaceId },
    /// Activate the New Space row through the production pointer path (stages
    /// the NewSpace command-bar mode).
    ClickSpaceNew,
    /// Move an active-space tab into another space.
    MoveTab { index: usize, to_space: TestSpaceId },
    /// Suspend a hidden tab and destroy its native session.
    SuspendTab { index: usize },
    /// Select a tab and measure until the correct native view is presented.
    BenchmarkSelectTab {
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
    BenchmarkReopenTab { operation_id: BenchmarkOperationId },
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
    BenchmarkToggleSidebar { operation_id: BenchmarkOperationId },
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
    /// Return keyboard focus to the web view — the E2E reader of Escape.
    FocusWeb,
    /// Set the command bar text directly (avoids per-keystroke round trips).
    SetCommandBarText { text: String },
    /// Dispatch a synthetic keystroke through GPUI's real input path
    /// (e.g. "cmd-l", "escape"). Zero permissions, zero OS event posting.
    DispatchKeystroke { keystroke: String },
    /// Resize the window content area (points). Used to observe layout
    /// re-flow via the snapshot; the native view is re-framed on the next
    /// frame.
    SetWindowSize { width: f32, height: f32 },
    /// Real pointer drag on the sidebar resize divider through GPUI's
    /// production mouse-event pipeline: mouse-down at the divider center,
    /// mouse-move by `dx` with the left button held, mouse-up. The same
    /// element listeners a physical drag drives.
    DragSidebarDivider { dx: f32 },
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
    BenchmarkRestoreSidebar { operation_id: BenchmarkOperationId },
    /// Measure the production pin toggle of the active tab until the new
    /// region order is presented (same native view, boundary moved).
    BenchmarkPinTab { operation_id: BenchmarkOperationId },
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
    /// Full app snapshot.
    GetSnapshot,
    /// Liveness probe.
    Ping,
    /// Graceful app shutdown (`quit()`, window closes, process exits clean).
    Quit,
}
