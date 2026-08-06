//! Protocol types shared by the gateway server and the E2E client.
//!
//! All types are plain data with `serde` derives so they round-trip through
//! JSON without leaking any framework type onto the wire. `protocol_version`
//! is part of the handshake; bump it when a type here changes in a breaking
//! way (see `docs/e2e-protocol.md` in E2E_ARCHITECTURE.md).

use bbr_web::{ProfileId, WebAction, WebViewId};
use serde::{Deserialize, Serialize};

/// Current protocol version understood by this crate.
pub const PROTOCOL_VERSION: u32 = 6;

/// Observable command-bar intent, independent from native focus ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandBarMode {
    Inactive,
    Location,
    NewTab,
}

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

/// The shell's resolved §15 mode fields (theme/contrast/density).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShellModes {
    pub theme: String,
    pub contrast: String,
    pub density: String,
}

/// Harness-assigned identity for one measured production-path mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkOperationId(pub u64);

/// Initial real-app performance scenarios. Extend this enum with each visible
/// interaction rather than inventing string labels in individual harnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkKind {
    /// Process start → first usable shell presentation (sidebar + selected
    /// native web view).
    Startup,
    /// Deterministic local fixture content ready (navigation finished with
    /// the expected fixture title), measured separately from shell readiness.
    PageReady,
    WarmTabSwitch,
    /// Visual pointer selection of a visible tab row (production listener
    /// path, chrome focus preserved).
    TabSelectPointer,
    /// Enter on the focused tab list activates the selected tab (chrome
    /// traversal commits, keyboard focus moves into the page).
    TabActivateKeyboard,
    /// Reopen of the most recently closed tab (fresh session presented).
    TabReopen,
    SidebarToggle,
    NewTabCommandBar,
    NewTabOpened,
    ActiveTabClose,
}

/// One app-internal latency sample completed at the end of GPUI paint after
/// native content geometry is attached, immediately before frame presentation.
/// Durations deliberately exclude testkit transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresentationMeasurement {
    pub operation_id: BenchmarkOperationId,
    pub kind: BenchmarkKind,
    pub duration_ns: u64,
    pub since_process_start_ns: u64,
    /// macOS monotonic absolute clock converted to nanoseconds. The harness
    /// reads the same clock immediately before spawn to include process launch.
    pub monotonic_commit_ns: u64,
    pub active_web_view: Option<WebViewId>,
    pub sidebar_visible: bool,
    pub chrome_focused: bool,
    pub command_bar_mode: CommandBarMode,
    pub content_rect: [f32; 4],
    /// Deterministic fixture title observed for [`BenchmarkKind::PageReady`]
    /// samples; `None` for shell presentation samples.
    pub page_title: Option<String>,
}

/// Stable space identity on the portable test wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSpaceId(pub u64);

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
    pub active_space: TestSpaceId,
    /// Active-space tabs and their current disposable session ids.
    pub tabs: Vec<TabSnapshot>,
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
    /// The stable `bbr-*` ids present in the current render (DESIGN.md §15).
    pub visible_ids: Vec<String>,
    pub command_bar_mode: CommandBarMode,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    /// Window content size in points (diagnostics only).
    pub window_size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Fired once the window is open, the engine is attached and the
    /// gateway is accepting commands.
    AppReady { pid: u32, app_version: String },
    /// A navigation-state change reported by the native engine.
    Web(bbr_web::WebEvent),
    /// The GPUI chrome gained/lost keyboard focus.
    ChromeFocusChanged { chrome_focused: bool },
    /// The command bar's mode or text changed.
    CommandBarChanged { mode: CommandBarMode, text: String },
    /// One semantic browser-controller mutation completed.
    BrowserChanged { kind: BrowserChange },
    /// A startup or interaction mutation reached a semantically correct
    /// presentation frame inside the app process.
    PresentationCommitted(PresentationMeasurement),
    /// The app received a quit request (graceful shutdown started).
    Quitting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserChange {
    ProfileCreated {
        profile: ProfileId,
        space: TestSpaceId,
    },
    SpaceCreated {
        profile: ProfileId,
        space: TestSpaceId,
    },
    SpaceActivated {
        space: TestSpaceId,
    },
    TabOpened {
        index: usize,
    },
    TabSelected {
        index: usize,
    },
    TabClosed {
        index: usize,
    },
    TabPinned {
        index: usize,
    },
    TabReopened {
        index: usize,
    },
    TabMoved {
        to_space: TestSpaceId,
        recreated: bool,
    },
    TabSuspended {
        index: usize,
    },
}

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
    /// Activate a visible tab row through the production pointer path: the
    /// same app method the row's mouse-down listener calls, with chrome focus.
    ClickTab { index: usize },
    /// Activate the visible tab row's close control through the production
    /// pointer path.
    ClickTabClose { index: usize },
    /// Deterministic engine-event injection (testkit-only): pushes a native
    /// `WebEvent` through the exact event transport the real engine uses, so
    /// E2E can exercise the production crash/error path without killing a
    /// real WKWebView process.
    InjectWebEvent { event: bbr_web::WebEvent },
    /// Move an active-space tab into another space.
    MoveTab { index: usize, to_space: TestSpaceId },
    /// Suspend a hidden tab and destroy its native session.
    SuspendTab { index: usize },
    /// Select a tab and measure until the correct native view is presented.
    BenchmarkSelectTab {
        operation_id: BenchmarkOperationId,
        index: usize,
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
    /// Full app snapshot.
    GetSnapshot,
    /// Liveness probe.
    Ping,
    /// Graceful app shutdown (`quit()`, window closes, process exits clean).
    Quit,
}

// The `Snapshot` payload grew with the §15 mode fields and per-row states;
// boxing it would churn the wire format for a lint that only measures enum
// size on the harness side.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Reply to [`Request::Hello`].
    Welcome {
        protocol_version: u32,
        app_version: String,
        pid: u32,
        capabilities: Vec<String>,
        next_seq: u64,
    },
    /// Plain success.
    Ok,
    /// Reply to [`Request::Attach`].
    Replay {
        events: Vec<(u64, Event)>,
        next_seq: u64,
    },
    /// Reply to [`Request::GetSnapshot`].
    Snapshot(Snapshot),
    /// Failure carrying a human-readable reason.
    Error(Error),
}

/// Structured error payload. Kept as data (not `anyhow`) so the client can
/// match on it without importing the app's error types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
}

impl Error {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

// ---------------------------------------------------------------------------
// Indirection layer: bbr-web already carries WebViewId/ProfileId/WebAction
// with serde support; re-export the ids used by harnesses so they don't
// need to reach into bbr-web themselves for constructing requests.
// ---------------------------------------------------------------------------

pub use bbr_web::{ProfileId as TestProfileId, WebViewId as TestWebViewId};

/// Context-free request helpers, so harness code reads as a script.
pub mod dsl {
    use super::*;

    pub fn hello(token: &str, capabilities: &[&str]) -> Request {
        Request::Hello {
            protocol_version: PROTOCOL_VERSION,
            token: token.to_string(),
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn navigate(url: impl Into<String>) -> Request {
        Request::Navigate { url: url.into() }
    }

    pub fn create_profile(
        profile_title: impl Into<String>,
        space_title: impl Into<String>,
    ) -> Request {
        Request::CreateProfile {
            profile_title: profile_title.into(),
            space_title: space_title.into(),
        }
    }

    pub fn create_space(profile: ProfileId, title: impl Into<String>) -> Request {
        Request::CreateSpace {
            profile,
            title: title.into(),
        }
    }

    pub fn activate_space(space: TestSpaceId) -> Request {
        Request::ActivateSpace { space }
    }

    pub fn open_tab(url: impl Into<String>) -> Request {
        Request::OpenTab { url: url.into() }
    }

    pub fn select_tab(index: usize) -> Request {
        Request::SelectTab { index }
    }

    pub fn close_tab(index: usize) -> Request {
        Request::CloseTab { index }
    }

    pub fn reopen_tab() -> Request {
        Request::ReopenTab
    }

    pub fn pin_tab(index: usize) -> Request {
        Request::PinTab { index }
    }

    pub fn click_tab(index: usize) -> Request {
        Request::ClickTab { index }
    }

    pub fn click_tab_close(index: usize) -> Request {
        Request::ClickTabClose { index }
    }

    pub fn inject_web_event(event: bbr_web::WebEvent) -> Request {
        Request::InjectWebEvent { event }
    }

    pub fn move_tab(index: usize, to_space: TestSpaceId) -> Request {
        Request::MoveTab { index, to_space }
    }

    pub fn suspend_tab(index: usize) -> Request {
        Request::SuspendTab { index }
    }

    pub fn back() -> Request {
        Request::WebAction(WebAction::Back)
    }

    pub fn forward() -> Request {
        Request::WebAction(WebAction::Forward)
    }

    pub fn reload() -> Request {
        Request::WebAction(WebAction::Reload)
    }

    pub fn stop() -> Request {
        Request::WebAction(WebAction::Stop)
    }

    /// The canonical ids used by the Phase-1 spike (one profile, one view).
    pub fn test_ids() -> (ProfileId, WebViewId) {
        (ProfileId(1), WebViewId(1))
    }
}
