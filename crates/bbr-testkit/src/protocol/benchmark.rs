//! Benchmark kinds and in-process presentation measurements for the real-app
//! performance gates. Operation IDs and kinds are typed so harnesses never
//! invent string labels.

use bbr_web::WebViewId;
use serde::{Deserialize, Serialize};

use super::{BenchmarkOperationId, CommandBarMode, TestSpaceId};

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
    /// Activation of a space with an already-warm selected tab (production
    /// activation path, warm sessions preserved).
    WarmSpaceSwitch,
    /// The sidebar visibility changed through the production Cmd-B path
    /// (collapse and open/toggle share this kind).
    SidebarToggle,
    /// The sidebar width changed through the production resize path (pointer
    /// drag or keyboard step); the sample commits only at the expected
    /// clamped geometry.
    SidebarResize,
    /// Expand restored the last expanded sidebar width through the production
    /// Cmd-B path.
    SidebarRestore,
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
    /// The space presented after the mutation (semantic state check for
    /// space-switch samples).
    pub active_space: TestSpaceId,
    pub sidebar_visible: bool,
    pub chrome_focused: bool,
    pub command_bar_mode: CommandBarMode,
    pub content_rect: [f32; 4],
    /// Deterministic fixture title observed for [`BenchmarkKind::PageReady`]
    /// samples; `None` for shell presentation samples.
    pub page_title: Option<String>,
}
