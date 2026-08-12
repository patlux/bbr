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
    /// A §11.3 sidebar swipe committed through the production gesture path
    /// (DESIGN.md §11.3 / task packet `space-footer-strip-appearance-ui`):
    /// the measured sample spans real `Started`/`Moved` wheel updates plus
    /// the zero-delta `TouchPhase::Ended` that resolves the tracking gesture
    /// into exactly one existing `activate_space` commit — never a direct
    /// activation and never a benchmark-only state machine. Same-profile
    /// warm switching (sessions untouched), so the sample shares the
    /// 50/100 ms warm class and commits only when the exact persistent
    /// destination space, its warm selected view, and the Idle machine
    /// phase are presented.
    SwipeCommit,
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
    /// A §11.14 titlebar navigation control (Back/Forward/Reload) committed
    /// through the exact production dispatch the click and the keyboard
    /// shortcut share (the same authorities as Cmd-[ / Cmd-] / Cmd-R). The
    /// sample commits only when the genuine availability precondition held
    /// and the production dispatch succeeded; the driver waits for the real
    /// WebKit history state afterwards, so a dispatch that never navigated
    /// is rejected. Warm-interaction class.
    NavigationControl,
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
    /// The same production command-text path as [`Self::CommandType`]
    /// measured against the history-rich corpus (Phase 7, command-history
    /// provider): at least 500 tabs plus a 10,000-entry in-memory history
    /// provider seed the ranking scale the portable command core proves
    /// (queue item 16, `command_bar_first_result`). The expected first
    /// result is a ranked history row (`history:<id>`), and the sample
    /// commits only when that exact first key is presented — the
    /// fast-but-wrong guard rejects a tab/fallback row or a stale
    /// generation. Same warm-interaction class as `command_type` (50/100).
    CommandBarFirstResult,
    /// The history-rich twin of [`Self::CommandBarFirstResult`] measuring
    /// the *complete ranked result-set presentation* (queue item 16,
    /// `command_bar_full_search`; performance contract "filtering"): the
    /// query matches far more rows than the limit-16 window, so the sample
    /// commits only from a render of the full window whose exact row-key
    /// set and truthful total match count match the armed expectation — a
    /// partial, reordered, truncated, or stale window, or a wrong total,
    /// rejects the measurement in the app. Same production command-text
    /// path, same 500-tab + 10k-history corpus, same warm-interaction
    /// class (50/100) as its first-result sibling.
    CommandBarFullSearch,
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
    /// Cmd-L / address-pill click opened the field-anchored overlay
    /// (DESIGN.md §11.4 `bbr-command-overlay`) over the address control:
    /// the sample commits only from a render that actually constructed the
    /// overlay with the Location mode, chrome focus, and the full URL
    /// preselected (one keystroke replaces it) — the fast-but-wrong guard
    /// rejects a missing or invented surface. Same warm-presentation class
    /// as `palette_open` — no tab or session is created before Enter.
    OverlayOpen,
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
    /// Restart on a cancelled row through the production `controller.retry`
    /// path (the settled DESIGN.md §17.8 decision): the fresh attempt
    /// (linked via `retry_of`) is presented in the Active group — the same
    /// warm-presentation class as the failed retry.
    DownloadsRowRestart,
    /// The §17.8 row Delete action on a cancelled row through the production
    /// `download_delete_row` path: exactly that one terminal record is
    /// removed and persisted durably; the sample commits when the panel
    /// render no longer draws it — warm-presentation class.
    DownloadsRowDelete,
    /// The §17.7 Clear panel action through the production `downloads_clear`
    /// path: the controller history is emptied and the empty slice persisted
    /// durably; the sample commits when the panel render draws the empty
    /// state — warm-presentation class.
    DownloadsClear,
    /// A classified web-error incident surfaced through the production
    /// `on_incident` path: the error surface rendered the contract-derived
    /// title/copy/action for the armed category over the unchanged active
    /// view — the warm-presentation class (no tab or session is created or
    /// touched). Fixture/network time is excluded by construction: the
    /// classified incident is applied in-process through the same seam the
    /// native delegate's incident sink drives.
    ErrorSurfacePresent,
    /// `Cmd-Plus` applied one contract zoom step on the active view's site
    /// through the production path (queue item 11, part A): the lifecycle
    /// change, the persistence write, and the native `pageZoom` application
    /// committed — the warm-interaction class (no tab or session is
    /// created or touched).
    ZoomStep,
    /// The production BBR context-menu open over the active web view (task
    /// packet `bbr-context-menus-over-web`): the sample starts at the exact
    /// event-driven open path the native bridge's `ContextMenuRequested`
    /// routes to and commits only from a render of the one `NativeAboveWeb`
    /// portal window that drew the menu over the same active view — the
    /// warm-presentation class, no tab or session touched.
    ContextMenuOpen,
    /// The credential sheet presented through the production path (queue
    /// item 10): the sheet renders in the one `NativeAboveWeb` portal window
    /// showing the awaiting challenge — the warm-presentation class on the
    /// existing command-surface budget (no tab or session is created). The
    /// awaiting challenge is the fixture, recorded before the sample.
    AuthSheetOpen,
    /// The production command-text path measured against the Saved Page
    /// provider (B2 packet `saved-pages-actions-command`,
    /// `saved_page_search_first_result`): Cmd-L, replace-selection typing,
    /// the real ranking debounce, and a render of the result surface
    /// committing the sample — the expected first result is a ranked Saved
    /// Page row (`saved_page:<id>`) over the seeded ≥1,000-record provider.
    /// The fast-but-wrong guard rejects a tab/fallback row or a stale
    /// generation. Same warm command-type class as `command_type` (50/100).
    SavedPageSearchFirstResult,
    /// The production open of a Saved Page result (B2 packet
    /// `saved-pages-actions-command`, `saved_page_open`): the record is
    /// revalidated in the in-memory provider and exactly one fresh regular
    /// tab/session opens in the currently active Space/Profile through the
    /// existing NewTab authority — the session-recreation class with its
    /// own budget (100/150), like `command_activate_new_tab` and
    /// `tab_reopen`. The sample commits only when the fresh tab/session is
    /// presented and focused with the surface closed.
    SavedPageOpen,
    /// A committed logical Back content gesture dispatched exactly one real
    /// history action against a deterministic already-loaded two-entry
    /// history (N1 packet `web-content-navigation-gestures`): the sample
    /// commits only when the correct URL/history state is presented after
    /// the dispatch — the warm interaction class with the unchanged warm
    /// ceiling (p95 ≤ 50 ms / max ≤ 100 ms), same as `active_tab_close`.
    /// At least 30 accepted optimized samples; external page/network
    /// latency stays outside BBR timings (the fixture is deterministic and
    /// already loaded before the sample).
    WebHistorySwipeBack,
    /// A committed logical Back content gesture at a settled regular
    /// child-tab root closed the child through the normal close path and
    /// presented the exact warm opener (N1 packet
    /// `web-content-navigation-gestures`): the sample commits only when
    /// the child session is gone and the exact warm opener is
    /// visible/focused — the active-tab-close warm ceiling (p95 ≤ 50 ms /
    /// max ≤ 100 ms), at least 30 accepted optimized samples.
    ChildTabSwipeReturn,
    /// An eligible user-initiated new-window request presented a transient
    /// Peek preview above its warm origin (Link Preview / Peek packet,
    /// `peek_request_to_present`): the sample starts at the production
    /// `BbrApp::on_new_window_request` seam and commits only from a render
    /// of the one `NativeAboveWeb` portal window that drew the Peek bar
    /// over the preview session with the covered origin still the active
    /// view and the preview's native subview above the origin's (truthful
    /// z-order, fast-but-wrong guard). The preview session is the fresh
    /// native-session class with its own budget (p95 ≤ 100 ms; max ≤
    /// 150 ms) — fixture/network page-load time is excluded by
    /// construction (the target fixture is deterministic and local, and
    /// the sample commits at presentation, not at page ready).
    PeekRequestToPresent,
    /// Dismissal of the transient Peek preview back to the exact warm
    /// origin (Link Preview / Peek packet, `peek_dismiss_to_origin`): the
    /// sample starts at the production `peek_close` path and commits only
    /// when the preview session is destroyed, the Peek bar is gone, and
    /// the warm origin is again the active, focused view — the warm
    /// interaction class (p95 ≤ 50 ms; max ≤ 100 ms), no session
    /// recreation.
    PeekDismissToOrigin,
    /// Promotion of the transient Peek preview to a regular tab without
    /// reload (Link Preview / Peek packet, `peek_promote_to_tab`): the
    /// sample starts at the production `peek_promote` path and commits
    /// only when the SAME preview `WebViewId` is a regular tab in the
    /// source space/profile, active and focused, with the preview state
    /// gone — the warm interaction class (p95 ≤ 50 ms; max ≤ 100 ms),
    /// the existing session adopted without recreation or page load.
    PeekPromoteToTab,
    /// A Command-primary / middle-click link opened one regular child tab
    /// in the background immediately after the exact opener (DD-LINK packet
    /// `background-link-tabs`, `background_link_open`): the sample starts at
    /// the production `BbrApp::on_new_window_request` seam and commits only
    /// from the authoritative browser state — the exact opener still the
    /// active, web-focused view and the new child a live session at the
    /// grouped insertion index directly after the opener, never
    /// foreground-activated (fast-but-wrong guard). The child session is
    /// the fresh native-session class with its own budget (p95 ≤ 100 ms;
    /// max ≤ 150 ms) — fixture/network page-load time is excluded by
    /// construction (the sample commits at insertion, not at page ready).
    BackgroundLinkOpen,
    /// The production Saved Pages Library search (B3 packet
    /// `saved-pages-library`, `saved_pages_library_search`): the typed
    /// search entry filters the loaded ≥1,000-record provider through the
    /// production `library_set_search` path and the bounded window
    /// re-renders; the sample commits only when the rendered model presents
    /// the exact armed query result count and window (fast-but-wrong
    /// guard: a missing/wrong filter, a stale window, or a wrong anchor
    /// rejects the measurement in the app). The warm command-text/next-frame
    /// presentation class (50/100) — the same budget as the other warm
    /// Library/interaction scenarios, no new class.
    SavedPagesLibrarySearch,
    /// The production Saved Pages Library scroll (B3 packet
    /// `saved-pages-library`, `saved_pages_library_scroll`): the tracked
    /// scroll offset windows the bounded render over the same loaded
    /// ≥1,000-record provider through the production `library_set_scroll`
    /// path; the sample commits only when the rendered model presents the
    /// exact armed window/anchor and preserves the exact selection set
    /// (fast-but-wrong guard: a no-op scroll, a wrong window/anchor, or a
    /// corrupted selection rejects the measurement in the app). Same warm
    /// next-frame presentation class (50/100).
    SavedPagesLibraryScroll,
    /// Under a reported hung active renderer, a normal Cmd-W close or warm
    /// neighbor selection reaches the exact next presentation. Samples use
    /// the unchanged warm interaction budget (p95 ≤ 50 ms / max ≤ 100 ms)
    /// and commit only when the armed unresponsive source, exact neighbor,
    /// session count, and focus domain are correct.
    UnresponsiveChrome,
    /// One production renderer-cadence tick while no probe is due. The sample
    /// proves the bounded idle path adds no visible frame work or state change;
    /// it shares the unchanged warm interaction budget and requires at least
    /// 30 optimized samples when performance execution is un-deferred.
    ProbeTick,
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
