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
    /// The active tab's pin state toggled through the production path (the
    /// pinned region boundary moved, the session untouched).
    TabPin,
    /// The active tab moved within its region through the production path
    /// (row order changed, the session untouched).
    TabReorder,
    /// The active tab moved into another space of the same profile: the live
    /// session follows the tab (same WebViewId, no recreation).
    TabMoveSameProfile,
    /// The active tab moved across a profile boundary: the old session is
    /// destroyed and recreated exactly once in the destination profile's
    /// store — the session-recreation class with its own budget.
    TabMoveCrossProfile,
    /// A dense-list scroll through the production scroll state
    /// (`ScrollDenseTabs`/`BenchmarkDenseScroll`): the tracked `ScrollHandle`
    /// the native wheel listener, the keyboard traversal, and the render
    /// share is moved by `delta_px`, and the sample commits after the frame
    /// that renders the resulting bounded window (the strongest honest
    /// semantic presentation metric GPUI 0.2.2 exposes — no public scanout
    /// callback, no public synthetic wheel dispatch).
    DenseScroll,
    /// Selection of a tab inside the dense collection through the production
    /// selection path (pointer or keyboard): the sample commits after the
    /// frame that renders the selection with the active row inside the
    /// bounded window (instant scroll-into-view, no animation).
    DenseSelect,
    /// The pointer entered a visible tab row through the production `on_hover`
    /// listener path (`set_row_hover`): the §10.2 hover wash + close reveal
    /// commit on the next frame. Transient indication, warm-interaction
    /// class — a dense-list hover regression trips this gate.
    TabHover,
    /// The pointer pressed a visible tab row through the production
    /// mouse-down listener path (`set_row_pressed`): the §10.3 active wash
    /// commits on the next frame, warm-interaction class.
    TabPress,
    /// A keystroke through the production command-text path (Cmd-L selects
    /// the URL, typing replaces the selection): the debounce published the
    /// ranked results for the typed text — the correct first result is
    /// presented. The sample commits only when the first result key matches
    /// the expectation (fast-but-wrong guard).
    CommandType,
    /// Enter activated the selected/typed command-bar destination through
    /// the production activation path: the correct destination, focus
    /// domain, and mode are presented (warm class — no session
    /// recreation).
    CommandActivate,
    /// Cmd-Enter opened the destination in a new tab: a fresh native
    /// session is presented and focused — the session-recreation class
    /// with its own budget.
    CommandActivateNewTab,
    /// Cmd-T opened the centered command palette (DESIGN.md §11.4
    /// `bbr-palette`) in New Tab staging mode: the sample commits only
    /// from a render that actually constructed the palette with the staged
    /// mode and chrome focus (the fast-but-wrong guard rejects a missing
    /// or invented surface). Warm-interaction class — no tab or session is
    /// created before Enter.
    PaletteOpen,
    /// The §11.4 footer plus menu presented through the production path.
    SidebarPlusMenu,
    /// The §11.10 favorites grid committed the expected tile count.
    SidebarFavorites,
    /// The §11.11 folder disclosure committed the expected state.
    SidebarFolderToggle,
    /// `Cmd-F` opened the find bar through the production path: the §11.4
    /// command-surface-family presentation rendered in the one
    /// `NativeAboveWeb` portal window with the find field focused — the
    /// warm-presentation class (no tab or session is created before Enter).
    FindOpen,
    /// A find query entered through the production path: WebKit's content
    /// process computed the match count/ordinal, the completion rode the
    /// portable contract, and the bar re-rendered the report — the warm
    /// command-surface class.
    FindQueryResult,
    /// Enter/Shift-Enter stepped the find session through the production
    /// path: WebKit reported the new ordinal and the bar re-rendered it —
    /// the warm command-surface class.
    FindStep,
    /// The footer `[Downloads]` control opened the anchored downloads panel
    /// through the production path: the sample commits only from a render
    /// that drew the panel open with the keyboard cursor (DESIGN.md §11.12
    /// chrome focus) over the unchanged active view — the warm-presentation
    /// class (no tab or session is created).
    DownloadsPanelOpen,
    /// A progress update landed on a running attempt through the production
    /// L6 event path and the panel re-rendered the row at the armed
    /// fraction: the sample commits only from a render that drew the
    /// exact received/total progress on the active row — the warm
    /// command-surface class. Fixture/network time is excluded by
    /// construction (the event is applied in-process through the same
    /// main-thread pump the adapter's sink uses).
    DownloadsProgressCommit,
    /// Retry on a failed row through the production `controller.retry` path:
    /// the fresh attempt (linked via `retry_of`) is presented in the Active
    /// group — the user-visible row action, warm-presentation class.
    DownloadsRowRetry,
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
    /// U2 presentation guards; ignored by unrelated benchmark kinds.
    #[serde(default)]
    pub plus_menu_open: bool,
    #[serde(default)]
    pub favorites_len: usize,
    #[serde(default)]
    pub folder_expanded: bool,
    /// Deterministic fixture title observed for [`BenchmarkKind::PageReady`]
    /// samples; `None` for shell presentation samples.
    pub page_title: Option<String>,
}
