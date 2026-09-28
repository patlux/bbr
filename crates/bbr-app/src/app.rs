//! The `BbrApp` root view: GPUI chrome (sidebar + command bar + status) with
//! the native web-view slot; also owns focus choreography and navigation
//! command routing.

use bbr_browser::{BrowserController, ControllerEvent};
use bbr_web::{WebEvent, WebRect};
use gpui::{App, Context, FocusHandle, Focusable, ScrollHandle, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::command_bar::{CommandBarMode, CommandBarReturn};
use crate::engine::EngineRef;
// The one production theme-selection authority (task packet
// `system-appearance-theme`): maps the GPUI window appearance to
// `ThemeMode` and resolves the effective mode (override-first).
mod appearance;
#[cfg(feature = "testkit")]
use crate::performance::PerformanceProbe;
use crate::portal::probe::PortalProbeState;
use crate::portal::state::PortalState;

use command_rank::CommandRanking;
use command_text::CommandText;
use palette::CommandSurface;
// The credential sheet's only production trigger — the WebKit
// authentication-challenge delegate — is later adapter work (contract packet
// non-goals: "WebKit delegates ... remain later adapter work"), so the
// challenge-arrival/record/present/snapshot seams are dead in a non-testkit
// build by construction; the testkit build drives the full production seam
// end to end (queue item 10 evidence).
#[cfg_attr(not(feature = "testkit"), allow(dead_code))]
pub(crate) mod auth;
#[cfg(feature = "testkit")]
mod auth_benchmarks;
#[cfg(feature = "testkit")]
mod background_link_benchmarks;
#[cfg(feature = "testkit")]
mod benchmarks;
#[cfg(feature = "testkit")]
mod benchmarks_dense;
#[cfg(feature = "testkit")]
mod benchmarks_pointer;
#[cfg(feature = "testkit")]
mod benchmarks_responsiveness;
#[cfg(feature = "testkit")]
mod benchmarks_u2;
mod command_activate;
#[cfg(feature = "testkit")]
mod command_benchmark_history;
#[cfg(feature = "testkit")]
mod command_benchmarks;
mod command_corpus;
pub(crate) mod command_rank;
mod command_result;
mod command_surface;
mod command_text;
mod content_card;
pub(crate) mod content_gesture;
pub(crate) mod context_menu;
#[cfg(feature = "testkit")]
mod context_menu_benchmarks;
pub(crate) mod downloads;
pub(crate) mod downloads_behavior;
#[cfg(feature = "testkit")]
mod downloads_benchmarks;
pub(crate) mod downloads_history;
mod drag;
#[cfg(feature = "testkit")]
mod drag_benchmarks;
mod drag_chip;
mod drag_state;
mod drag_target;
pub(crate) mod error_surface;
pub(crate) mod error_surface_behavior;
#[cfg(feature = "testkit")]
mod error_surface_benchmarks;
/// External-URL open composition (queue item 13): the app-lifetime
/// coordinator plus execution of the contract's routing outcome through the
/// existing controller. macOS-only body; the module is compiled everywhere
/// so the app composition stays portable.
pub(crate) mod external_urls;
/// App-owned favicon image mirror (task packet `real-favicon-pipeline.md`):
/// the presentation half of the acquisition join — decodes each acquired
/// record once, refreshes when the controller's store version changes, and
/// feeds every rendered favicon slot (tab rows, pinned rows, Favorites
/// tiles, folder items). Acquisition authority stays in the controller.
pub(crate) mod favicon_presentation;
mod favorites_band;
pub(crate) mod find;
#[cfg(feature = "testkit")]
mod find_benchmarks;
mod folder_rename;
#[cfg(feature = "testkit")]
mod gesture_benchmarks;
mod interaction;
pub(crate) mod library;
mod moments;
mod nav_history;
/// Blueprint D14 portal ids shared with the portal render.
pub(crate) mod nav_history_ids {
    pub const BACKDROP: &str = super::nav_history::NAV_HISTORY_BACKDROP_ID;
}
mod motion;
mod new_window;
pub(crate) mod palette;
pub(crate) mod peek;
#[cfg(feature = "testkit")]
mod peek_benchmarks;
mod portal_host;
mod render;
mod saved_pages;
#[cfg(feature = "testkit")]
mod saved_pages_benchmarks;
mod sidebar;
mod sidebar_view;
mod space_config;
mod space_delete;
mod spaces;
mod swipe;
#[cfg(feature = "testkit")]
mod swipe_benchmarks;
mod tab;
pub(crate) mod tab_menu;
#[cfg(feature = "testkit")]
mod tab_menu_benchmarks;
#[cfg(feature = "testkit")]
mod testkit;
mod titlebar;
mod u2;
mod u2_surface;
mod unresponsive;
// App-owned snapshot → bbr-ui DTO adapters (Favorite/Folder/browser
// snapshot conversions stay in bbr-app; the DTOs and rendering live in
// bbr-ui).
#[cfg(test)]
mod u2_tests;
pub(crate) mod ui_adapters;
pub(crate) mod zoom;
#[cfg(feature = "testkit")]
mod zoom_benchmarks;

/// Sidebar width follows the §8 normal-density token (300 pt); Phase-4
/// resize/drag will make it adjustable.
/// Focus-change sink: invoked on the main thread after any focus mutation.
pub type FocusSink = Box<dyn FnMut(bool)>;
/// Semantic browser-mutation sink shared by shortcuts and testkit commands.
pub type BrowserSink = Box<dyn FnMut(&ControllerEvent)>;

/// Default window size for the spike.
pub const DEFAULT_WINDOW_SIZE: (f32, f32) = (1080.0, 700.0);

/// The Phase-7 startup restore outcome, surfaced truthfully in the testkit
/// snapshot: whether a persisted browser model was applied chrome-first and
/// how many tab identities stayed session-less (deferred) after the restore.
/// A failed restore rolls back to a fresh default model and reports
/// [`RestoreEvidence::Failed`] — never a partial restored window presented
/// as valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreEvidence {
    /// No persisted browser model (fresh store) — the startup opened the
    /// initial tab as before.
    Fresh,
    /// A persisted browser model was applied; `deferred_tabs` identities
    /// stayed session-less until activation.
    Restored { deferred_tabs: usize },
    /// The persisted model was rejected (validation or mid-apply failure)
    /// and the controller rolled back to a fresh default model.
    Failed,
}

/// The root view of the browser window.
pub struct BbrApp {
    engine: EngineRef,
    controller: BrowserController,
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    engine_kind: &'static str,
    /// GPUI focus handle for the chrome (sidebar/command bar).
    chrome_handle: FocusHandle,
    /// GPUI focus handle for the permanent space strip (arrow/enter keys).
    space_strip_handle: FocusHandle,
    /// GPUI focus handle for the visible tab list (arrow/backspace keys).
    tab_list_handle: FocusHandle,
    /// GPUI focus handle for the sidebar resize divider (←/→ resize, 0 reset).
    divider_handle: FocusHandle,
    /// Production scroll state of the regular tab region (DESIGN.md §11.6):
    /// the native wheel listener and the keyboard traversal mutate the same
    /// tracked offset, and the render derives its bounded row window from it.
    dense_scroll: ScrollHandle,
    /// Production scroll state of the §11.3 space strip: chip activation
    /// scrolls the selected chip fully into view through this tracked
    /// handle (no reordering, no wrapping).
    space_strip_scroll: ScrollHandle,
    /// The last render's dense window, recorded during `Render::render` so
    /// the testkit snapshot reports exactly what the shell built (bounded
    /// rendered rows, active-row visibility). `pub(crate)` because the
    /// gateway dispatch (`crate::testkit`) reads the anchor for the dense
    /// benchmark and the app testkit snapshot mirrors the full state.
    pub(crate) dense_window: std::cell::Cell<crate::dense::DenseRenderState>,
    /// Adjustable sidebar geometry (DESIGN.md §5/§11.4): min/default/max
    /// widths, narrow-window clamping, collapse memory, drag state. The
    /// pure geometry model lives in `bbr-ui::sidebar::geometry` (the G3
    /// composition authority); the app keeps the input wiring.
    pub sidebar: bbr_ui::sidebar::SidebarGeometry,
    /// In-flight tab drag state (DESIGN.md §10.9/§11.2): the pointer drag
    /// for reorder and move-between-spaces. Idle when no drag is active.
    tab_drag: drag::TabDrag,
    /// The one native portal host for this browser window (task packet
    /// `native-portal-host`): created hidden and non-activating at window
    /// construction, retained here for app lifecycle. `None` until the
    /// composition root attaches it (macOS only; portable no-op elsewhere).
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    native_portal: Option<crate::portal::native::NativePortalBridge>,
    /// The one-active portal state for this browser window (task packet
    /// `native-portal-host`): exactly one presentation, deterministic
    /// dismissal, stable ids.
    portal: PortalState,
    /// The portal window's render state, shared with the portal window root
    /// (the probe surface): what the portal window renders is exactly what
    /// the app commanded.
    portal_probe: Rc<RefCell<PortalProbeState>>,
    /// DESIGN.md §10.2 hover state: the row the pointer currently hovers
    /// (driven by the production `on_hover` listeners). `None` at rest.
    hovered_row: Option<usize>,
    /// DESIGN.md §10.2/§11.11 hover state: the folder row the pointer
    /// currently hovers (driven by the production `on_hover_folder`
    /// listener). Hover/focus reveals the folder's `type:meta` count;
    /// `None` at rest.
    hovered_folder: Option<bbr_domain::FolderId>,
    /// DESIGN.md §10.3 pressed state: the row whose primary button is
    /// currently down (driven by the production mouse-down/up listeners).
    pressed_row: Option<usize>,
    /// DESIGN.md §11.5 address-control hover state: the pointer is over the
    /// domain pill (driven by the production `on_hover` listener). The
    /// hover moves the translucent fill one step brighter (dark chrome) /
    /// darker (light chrome) via the §11.5 token pair.
    address_hovered: bool,
    /// §6/§18.5 app-side motion state (D5 finish timer, idle evidence).
    motion: motion::MotionState,
    /// §18.5 after-commit moments (favorite pop, download check, ⌘T
    /// entrance/glide, D4 tint fade).
    moments: moments::Moments,
    /// §11.14 titlebar hover state: the control under the pointer (DESIGN
    /// §10.2 wash), driven by the production `on_hover` listener. `None` at
    /// rest.
    titlebar_hovered: Option<bbr_ui::titlebar::TitlebarControl>,
    /// §11.14 titlebar pressed state: the control whose primary button is
    /// down (DESIGN §10.3 wash), driven by the production press/release
    /// slots. `None` at rest.
    titlebar_pressed: Option<bbr_ui::titlebar::TitlebarControl>,
    /// Blueprint D14: the Back/Forward history menu, the in-flight
    /// Back/Forward press and its hold timer.
    nav_history: crate::app::nav_history::NavHistoryState,
    nav_press: Option<crate::app::nav_history::NavPress>,
    nav_press_generation: u64,
    nav_hold_timer: Option<gpui::Task<()>>,
    /// Current command-bar intent. `NewTab` stages navigation without creating
    /// a domain tab or native session until Enter commits it.
    pub command_bar_mode: CommandBarMode,
    /// DESIGN.md §11.3 cross-context transient: after a successful
    /// cross-profile space activation the destination profile appears at the
    /// top of the command field for one interaction as a §12 chip plus the
    /// "in Profile · <title>" hint, so a cross-profile destination is
    /// unmistakable without relying on color. `None` at rest. Same-profile
    /// switching never sets it (and clears any prior hint); the next chrome
    /// context mutation — another space activation, a tab mutation, or a
    /// command-bar mode entry — clears it (a deterministic user-interaction
    /// boundary, never a timer). Distinct from the staged-move
    /// `command_bar_hint`; the two never coexist.
    command_profile_hint: Option<String>,
    command_bar_return: Option<CommandBarReturn>,
    /// The open command-surface presentation (DESIGN.md §11.4): the centered
    /// Cmd-T palette, the field-anchored Cmd-L/click overlay, or none. At
    /// most one is open at a time (mutual exclusion by construction — a
    /// single enum field); opening either clears the §11.3
    /// `command_profile_hint`. The in-rail command bar is the baseline
    /// surface — this state only tracks the two overlay presentations of
    /// the same §11.5 engine.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    command_surface: CommandSurface,
    /// Find-in-page composition (task packet `find-in-page-portal-bar`):
    /// the one find presentation per window — portable session, native
    /// adapter bound to the active view, and snapshot/render state.
    find: crate::app::find::FindController,
    /// Web-error surface composition (task packet `error-surface-contract`):
    /// per-view incident lifecycles, the classified presentation the content
    /// area renders for TLS/navigation/offline failures, the contract-derived
    /// recovery routing, and the native-view hide/restore bookkeeping.
    pub(crate) error_surface: crate::app::error_surface::ErrorSurfaceState,
    /// Authentication challenge sheet composition (queue item 10): the
    /// coordinator (challenge records, contract lifecycle, redacted
    /// credential store) and the sheet's edit state. The sheet itself holds
    /// no authority over the challenge lifecycle; every decision routes
    /// through the merged portable contract.
    auth: crate::app::auth::AuthCoordinator,
    /// GPUI focus handle of the credential sheet (main-window focus; the
    /// portal window is never key, so keystrokes route through the root
    /// listener while the sheet is open).
    auth_handle: FocusHandle,
    /// Blueprint D05: the open Delete Space confirmation sheet and its
    /// focus handle (the sheet owns the keyboard while open).
    space_delete: Option<crate::app::space_delete::SpaceDeleteState>,
    space_delete_handle: FocusHandle,
    /// Downloads panel composition (task packet `downloads-sidebar-panel`):
    /// the controller-owned attempt/history store (L6), the anchored
    /// popover presentation state, and the deterministic row model.
    pub(crate) downloads: downloads::DownloadsState,
    /// GPUI focus handle of the open downloads panel (the keyboard cursor
    /// domain; DESIGN.md §11.12 chrome focus while open).
    downloads_panel_handle: FocusHandle,
    /// GPUI focus handle of the footer `[Downloads]` control: dismissal
    /// returns focus here (§11.12 focus domain; §9 order: Downloads
    /// precedes the space chips).
    downloads_control_handle: FocusHandle,
    /// §13 one-pointer-action marker: a pointer press inside the open panel
    /// (or on the footer control) sets it and stops propagation, so the
    /// root outside-press handler never dismisses a press inside the panel.
    downloads_press_inside: std::cell::Cell<bool>,
    /// The live `WKWebView` slot the download adapter dispatches `Start`
    /// requests on (the active session's view, refreshed before each
    /// start/retry). Opaque: only the macOS adapter interprets the payload.
    #[cfg(target_os = "macos")]
    download_web_view: bbr_web_macos::downloads_adapter::DownloadWebViewSlot,
    /// The concrete native download adapter (macOS only): routes portable
    /// actions to the real `WKDownload` lifecycle and emits L6 events. The
    /// action sink is wired at install time; the event sink routes through
    /// the main-thread pump channel.
    #[cfg(target_os = "macos")]
    downloads_adapter: Option<std::rc::Rc<bbr_web_macos::downloads_adapter::NativeDownloadAdapter>>,
    /// Concrete native engine handle (macOS only): `capture_download_web_view`
    /// resolves the active session's live `WKWebView` through it without
    /// creating or moving sessions.
    #[cfg(target_os = "macos")]
    download_engine: Option<crate::engine::NativeEngineRef>,
    /// GPUI focus handle of the find field. The find bar renders inside the
    /// non-activating portal window (never key by design), so keyboard input
    /// is handled by the main window: while a find presentation is open the
    /// root tracks this handle and routes key downs to find mode.
    find_handle: FocusHandle,
    /// Concrete native engine handle (macOS only): the find adapter binds
    /// the active `WKWebView` through it without creating/moving sessions.
    #[cfg(target_os = "macos")]
    find_engine: Option<crate::engine::NativeEngineRef>,
    /// The last completed real WebKit content probe (testkit-only; macOS):
    /// the completion block of an issued probe writes through this shared
    /// slot and the snapshot reads it, so the E2E gateway observes what
    /// WebKit actually committed — never our own visibility/geometry
    /// bookkeeping (which can be green while the user sees a blank page).
    #[cfg(all(target_os = "macos", feature = "testkit"))]
    web_content_probe: std::rc::Rc<
        std::cell::RefCell<
            Option<(
                bbr_web::WebViewId,
                bbr_web_macos::content_probe::ContentProbe,
            )>,
        >,
    >,
    /// External-URL open delivery store (macOS only, queue item 13): the
    /// app-lifetime coordinator plus request-id/tick watermarks. Fed one URL
    /// per OS delivery through the AppKit delegate path (or the testkit
    /// seam); the decision is always the merged portable contract's.
    #[cfg(target_os = "macos")]
    external_urls: bbr_web_macos::external_urls_adapter::ExternalUrlAdapterCore,
    /// Per-site zoom composition (queue item 11, part A): the app-side
    /// authority for the active view's zoom level — profile-scoped
    /// lifecycles over the merged `site_settings` contract, an in-memory
    /// store, and a native application target.
    zoom: crate::app::zoom::ZoomController,
    /// Concrete native engine handle (macOS only): the zoom target binds
    /// the active `WKWebView` through it (`WebKitZoomAdapter::apply`) without
    /// creating/moving/reselecting sessions.
    #[cfg(target_os = "macos")]
    zoom_engine: Option<crate::engine::NativeEngineRef>,
    /// BBR context menus over web content (queue item 12, part B; task
    /// packet `bbr-context-menus-over-web`): the app-side authority for the
    /// page menu (Back/Forward/Reload/Copy Link/Open in New Tab) presented
    /// through the one `NativeAboveWeb` portal host. Pure chrome state — no
    /// session or adapter — plus the deterministic dismissal/activation
    /// rules.
    context_menu: crate::app::context_menu::ContextMenuController,
    /// Production Tab/Favorite context menu (task packet
    /// `tab-favorite-context-menus`; DESIGN.md §11.13): the app-owned
    /// authority — the exact bound `TabId`, the deterministic row model
    /// with typed actions, the clamped placement, and the dismissal/focus
    /// rules. Ordinary GPUI chrome over the sidebar, never a portal
    /// surface; the presentation DTO lives in `bbr-ui::surfaces::tab_menu`.
    tab_menu: crate::app::tab_menu::TabMenuController,
    /// GPUI focus handle of the menu. The menu renders inside the
    /// non-activating portal window (never key by design), so keyboard
    /// input is handled by the main window: while a menu presentation is
    /// open the root tracks this handle and routes key downs to menu mode.
    menu_handle: FocusHandle,
    /// Concrete native engine handle (macOS only): the native context-menu
    /// bridge diagnostics (the right-click monitor and the page-vs-element
    /// ownership classification counters) are read through it.
    #[cfg(target_os = "macos")]
    menu_engine: Option<crate::engine::NativeEngineRef>,
    /// The app-owned persistence composition (Phase 7, command-history
    /// provider slice): the shared `bbr-app -> bbr-persistence` bootstrap
    /// that owns the truthful store disposition, the global search
    /// settings, and the in-memory history provider ranking consumes.
    store: crate::store::AppStore,
    /// The Phase-7 startup restore outcome (see [`RestoreEvidence`]).
    restore_evidence: RestoreEvidence,
    /// The last observed macOS system appearance mapped to a production
    /// [`bbr_ui::theme::ThemeMode`] (task packet `system-appearance-theme`):
    /// the one production theme source. Updated live by the GPUI
    /// window-appearance observer; the render and the snapshot resolve the
    /// effective mode from it (`ThemeMode::Light` until the composition
    /// root observes the window before the first presented frame).
    system_appearance: bbr_ui::theme::ThemeMode,
    /// Testkit-only deterministic appearance override (task packet
    /// `system-appearance-theme`): when set, the effective mode ignores the
    /// observed system appearance so real-app E2E can force light and dark
    /// regardless of the host. `None` follows the system.
    #[cfg(feature = "testkit")]
    appearance_override: Option<bbr_ui::theme::ThemeMode>,
    /// The observed macOS Increase Contrast / Reduce Motion options (task
    /// packet `v2-system-prefs`); corrected by `observe_system_accessibility`
    /// before the first presented frame and kept live by the native
    /// workspace notification.
    system_accessibility: appearance::AccessibilityPrefs,
    /// Testkit-only per-option accessibility override (`None` follows the
    /// system).
    #[cfg(feature = "testkit")]
    accessibility_override: appearance::AccessibilityOverride,
    /// Keeps the native accessibility-display registration alive; dropping
    /// it removes the observer.
    #[cfg(target_os = "macos")]
    accessibility_observer:
        Option<bbr_web_macos::accessibility_display::AccessibilityDisplayObserver>,
    /// The foreground task draining accessibility-change signals.
    accessibility_task: Option<gpui::Task<()>>,
    /// Testkit-only full-screen presentation override (DESIGN §2 full-bleed
    /// card): `None` follows the real window's full-screen state.
    #[cfg(feature = "testkit")]
    pub(crate) full_screen_override: Option<bool>,
    /// Testkit-only render evidence: the theme mode the portal render last
    /// drew the credential sheet / the Peek bar in (both follow the shell
    /// appearance; E2E proves it from what was drawn, not from the model).
    #[cfg(feature = "testkit")]
    pub(crate) auth_presented_theme: std::cell::Cell<Option<bbr_ui::theme::ThemeMode>>,
    #[cfg(feature = "testkit")]
    pub(crate) peek_presented_theme: std::cell::Cell<Option<bbr_ui::theme::ThemeMode>>,
    /// Keeps the GPUI window-appearance observer alive (task packet
    /// `system-appearance-theme`): a dropped subscription silently stops
    /// live flip observation.
    appearance_subscription: Option<gpui::Subscription>,
    /// The last rendered window content rect `[x, y, width, height]` in
    /// points (Phase 7, warm-restore slice): the persist-direction source
    /// for the optional window-geometry presentation row. The content rect
    /// (excluding the title bar) is measured natively where a live web
    /// session exists — GPUI's window options apply the persisted bounds as
    /// the content rect, so persisting the true content rect keeps the
    /// round trip exact (a frame-based value would grow by the title-bar
    /// height every launch) — with the frame bounds as the fallback before
    /// the first session exists. Updated in `render`; `None` until the
    /// first frame.
    window_geometry: std::cell::RefCell<Option<[f64; 4]>>,
    /// Concrete native engine handle (macOS only): the window-geometry
    /// persist source reads the live session's window content rect through
    /// it (`WebEngineWk::window_content_rect`) without creating/moving
    /// sessions.
    #[cfg(target_os = "macos")]
    geometry_engine: Option<crate::engine::NativeEngineRef>,
    /// The last frame's command-field rect in window coordinates, recorded
    /// by the field anchor element (DESIGN.md §11.4): the exact anchor for
    /// the field-anchored overlay (top/leading edge flush, width from the
    /// field) without duplicating the chrome layout math. `None` until the
    /// field has rendered at least once.
    command_field_rect: std::rc::Rc<std::cell::RefCell<Option<WebRect>>>,
    /// The last frame's favorites-region rect in window coordinates,
    /// recorded by the band anchor element (DESIGN.md §11.10): the exact
    /// drop-band span the tab-drag resolver and the E2E snapshot use, so
    /// the first favorite can be created by dragging a concrete tab into
    /// the rendered band. `None` while no band is rendered (no tiles and
    /// no in-flight drag).
    favorites_band_rect: std::rc::Rc<std::cell::RefCell<Option<WebRect>>>,
    /// The last frame's §11.2 ghost-row and §10.9 drag-proxy rects, recorded
    /// by their anchors while a tab drag renders them.
    drag_offer_rects: drag::DragOfferRects,
    /// The command-field text-edit model (DESIGN.md §11.5): UTF-8-safe
    /// cursor/selection, full selection on `Cmd-L`, replace-selection
    /// typing, backspace, and horizontal cursor movement. The text mirrors
    /// the live URL when not editing.
    pub command_text: CommandText,
    /// The cancellable, nonblocking ranking orchestration of the §11.5
    /// result surface (state machine + debounce handle). Inactive unless a
    /// location/new-tab mode is open.
    command_ranking: CommandRanking,
    /// The one app-owned Saved Page command authority (B2 packet
    /// `saved-pages-actions-command`): availability and dispatch for
    /// `Bookmark Page`, `Remove Bookmark`, and `Open Saved Page`, shared by
    /// Cmd-D, the Command Palette, and the capability-gated Z1/Z3 menu
    /// rows. Stateless: all state lives in the store provider and the
    /// controller.
    saved_pages: saved_pages::SavedPageCommand,
    /// App-owned Saved Pages Library composition (B3 packet
    /// `saved-pages-library`): the open/close entry, the local search/sort/
    /// selection/remove-confirm/scroll state, the B1/B2 binding for every
    /// Library operation, and the native-view hide/restore bookkeeping
    /// (the Library is the single visibility authority while open, exactly
    /// like the error surface).
    library: crate::app::library::LibraryState,
    /// GPUI focus handle of the Library surface (DESIGN.md §11.16 chrome
    /// focus domain while open).
    library_handle: FocusHandle,
    /// §11.8a unresponsive-pane composition (renderer-isolation lane): the
    /// native-view hide/restore bookkeeping for the pane over the exact
    /// active view (the session stays alive).
    unresponsive: unresponsive::UnresponsiveState,
    /// GPUI focus handles of the §11.8a recovery buttons in tab order
    /// Wait → Reload → Close tab (the sidebar follows in the global tab
    /// order).
    unresponsive_wait_handle: FocusHandle,
    unresponsive_reload_handle: FocusHandle,
    unresponsive_close_handle: FocusHandle,
    pub sidebar_visible: bool,
    /// The L5 session-local favorites/folders organization read model
    /// (DESIGN.md §11.10/§11.11): the source the U2 regions render and the
    /// snapshot report. Commands (NewFolder/NewSpace/FavoriteToggle/
    /// MoveIntoFolder) wire into the plus menu / command bar in
    /// `app::u2`; the read model is consumed as-is.
    organization: bbr_browser::OrganizationController,
    /// U2 sidebar presentation/focus state; organization authority stays in
    /// the controller above, while this group owns only local UI state.
    u2: u2::U2State,
    /// §11.3 Space-configuration surface state (open flag, the bound exact
    /// persistent SpaceId, the presentation draft, and the surface's chrome
    /// focus handle); every rename/appearance authority stays in the
    /// controller.
    space_config: space_config::SpaceConfigState,
    /// DESIGN.md §11.3 sidebar swipe composition (F2a swipe kernel): the one
    /// in-app gesture state. Idle unless a horizontal trackpad gesture is in
    /// progress; commits through the existing `activate_space` authority.
    swipe: swipe::SwipeState,
    /// Focus-change sink (main thread): called after any focus mutation with
    /// the actual chrome-focus state. Wired by the testkit glue to broadcast
    /// `ChromeFocusChanged`; unused in production builds.
    focus_sink: Rc<RefCell<Option<FocusSink>>>,
    browser_sink: Rc<RefCell<Option<BrowserSink>>>,
    /// Productive new-window seam diagnostics (requests, created regular tabs,
    /// reported blocks). Updated only by `on_new_window_request`.
    new_window: new_window::NewWindowState,
    /// Web-content gesture diagnostics (task packet
    /// `web-content-navigation-gestures`): how each committed logical
    /// gesture was classified (history dispatch / opener return / no-op).
    /// Updated only by `on_content_gesture_commit`.
    content_gesture: content_gesture::ContentGestureState,
    /// Transient Peek presentation state (Link Preview / Peek packet): the
    /// bar presentation and the captured warm-origin focus target. The
    /// preview *session* state lives in the controller snapshot.
    peek: peek::PeekState,
    /// The decoded favicon image mirror (task packet
    /// `real-favicon-pipeline.md`): `(profile, host) → decoded image`,
    /// rebuilt when the controller's bounded store version changes (bumped
    /// on every insert/eviction), so every presentation decodes each
    /// acquired record exactly once. Presentation-only; acquisition
    /// authority stays in the controller's bounded store.
    favicon_images: favicon_presentation::FaviconImageMirror,
    /// The controller store version [`Self::favicon_images`] was last
    /// rebuilt for.
    favicon_mirror_version: u64,
    #[cfg(feature = "testkit")]
    performance: PerformanceProbe,
}

impl BbrApp {
    /// The wiring constructor: engine, controller, organization, event
    /// channels, native find channel (macOS), and the testkit performance
    /// probe are all injected together.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: EngineRef,
        controller: BrowserController,
        organization: bbr_browser::OrganizationController,
        engine_kind: &'static str,
        evt_tx: smol::channel::Sender<WebEvent>,
        #[cfg(target_os = "macos")] find_tx: smol::channel::Sender<
            bbr_web_macos::find_mapping::FindCompletion,
        >,
        #[cfg(feature = "testkit")] performance: PerformanceProbe,
        store: crate::store::AppStore,
        cx: &mut Context<Self>,
    ) -> Self {
        // Engine → app events: the sink runs on the main thread (delegate
        // callbacks); the channel is drained by the pump task that mirrors
        // into `shell` and (under testkit) broadcasts to E2E clients.
        let sink_tx = evt_tx.clone();
        engine
            .borrow_mut()
            .set_events(Box::new(move |ev: WebEvent| {
                let _ = sink_tx.try_send(ev);
            }));

        // Startup restore of the terminal download history (DESIGN.md §17.10
        // settled: terminal history persists through restarts): the persisted
        // terminal records are mapped onto the controller **before the first
        // frame**, so the panel and the footer badge render the restored rows
        // from the first user-observable paint. Live attempts are never
        // persisted and never resurrected; the repository validates outcomes
        // on load, so a non-terminal record cannot reach the restore seam.
        let mut downloads_state = downloads::DownloadsState::new();
        match crate::store::downloads::from_persisted(store.restored_downloads())
            .into_iter()
            .map(bbr_browser::DownloadRecord::restored_terminal)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(records) => {
                if let Err(error) = downloads_state.controller.restore_terminal_history(records) {
                    log::warn!("downloads history restore failed: {error:?}");
                }
            }
            Err(error) => log::warn!("downloads history restore failed: {error:?}"),
        }

        BbrApp {
            engine: engine.clone(),
            controller,
            engine_kind,
            chrome_handle: cx.focus_handle(),
            space_strip_handle: cx.focus_handle(),
            tab_list_handle: cx.focus_handle(),
            divider_handle: cx.focus_handle(),
            dense_scroll: ScrollHandle::new(),
            space_strip_scroll: ScrollHandle::new(),
            dense_window: std::cell::Cell::new(crate::dense::DenseRenderState::default()),
            sidebar: bbr_ui::sidebar::SidebarGeometry::new(),
            tab_drag: drag::TabDrag::default(),
            native_portal: None,
            portal: PortalState::new(crate::portal::probe::PORTAL_ID),
            portal_probe: Rc::new(RefCell::new(PortalProbeState::new())),
            hovered_row: None,
            hovered_folder: None,
            pressed_row: None,
            address_hovered: false,
            motion: motion::MotionState::default(),
            moments: moments::Moments::default(),
            titlebar_hovered: None,
            titlebar_pressed: None,
            nav_history: Default::default(),
            nav_press: None,
            nav_press_generation: 0,
            nav_hold_timer: None,
            command_bar_mode: CommandBarMode::Inactive,
            command_bar_return: None,
            command_surface: CommandSurface::None,
            find: crate::app::find::FindController::new(
                #[cfg(target_os = "macos")]
                find_tx,
            ),
            error_surface: crate::app::error_surface::ErrorSurfaceState::default(),
            find_handle: cx.focus_handle(),
            auth: crate::app::auth::AuthCoordinator::new(),
            auth_handle: cx.focus_handle(),
            space_delete: None,
            space_delete_handle: cx.focus_handle(),
            downloads: downloads_state,
            downloads_panel_handle: cx.focus_handle(),
            downloads_control_handle: cx.focus_handle(),
            downloads_press_inside: std::cell::Cell::new(false),
            #[cfg(target_os = "macos")]
            download_web_view: bbr_web_macos::downloads_adapter::DownloadWebViewSlot::new(),
            #[cfg(target_os = "macos")]
            downloads_adapter: None,
            #[cfg(target_os = "macos")]
            download_engine: None,
            #[cfg(target_os = "macos")]
            find_engine: None,
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: std::rc::Rc::new(std::cell::RefCell::new(None)),
            #[cfg(target_os = "macos")]
            external_urls: bbr_web_macos::external_urls_adapter::ExternalUrlAdapterCore::new(),
            zoom: crate::app::zoom::ZoomController::new(crate::app::zoom::noop_target()),
            #[cfg(target_os = "macos")]
            zoom_engine: None,
            context_menu: crate::app::context_menu::ContextMenuController::new(),
            tab_menu: crate::app::tab_menu::TabMenuController::new(),
            menu_handle: cx.focus_handle(),
            #[cfg(target_os = "macos")]
            menu_engine: None,
            #[cfg(target_os = "macos")]
            geometry_engine: None,
            command_field_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            favorites_band_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            drag_offer_rects: drag::DragOfferRects::default(),
            command_text: CommandText::new(),
            command_ranking: CommandRanking::default(),
            saved_pages: saved_pages::SavedPageCommand,
            library: crate::app::library::LibraryState::default(),
            library_handle: cx.focus_handle(),
            unresponsive: unresponsive::UnresponsiveState::new(),
            unresponsive_wait_handle: cx.focus_handle(),
            unresponsive_reload_handle: cx.focus_handle(),
            unresponsive_close_handle: cx.focus_handle(),
            command_profile_hint: None,
            store,
            restore_evidence: RestoreEvidence::Fresh,
            // The observed system appearance is corrected by
            // `observe_system_appearance` at composition (before the first
            // presented frame); Light is the deterministic pre-observation
            // default.
            system_appearance: bbr_ui::theme::ThemeMode::Light,
            #[cfg(feature = "testkit")]
            appearance_override: None,
            system_accessibility: appearance::AccessibilityPrefs::default(),
            #[cfg(feature = "testkit")]
            accessibility_override: appearance::AccessibilityOverride::default(),
            #[cfg(target_os = "macos")]
            accessibility_observer: None,
            accessibility_task: None,
            #[cfg(feature = "testkit")]
            full_screen_override: None,
            #[cfg(feature = "testkit")]
            auth_presented_theme: std::cell::Cell::new(None),
            #[cfg(feature = "testkit")]
            peek_presented_theme: std::cell::Cell::new(None),
            appearance_subscription: None,
            window_geometry: std::cell::RefCell::new(None),
            sidebar_visible: true,
            organization,
            u2: u2::U2State::new(cx),
            space_config: space_config::SpaceConfigState::new(cx),
            swipe: swipe::SwipeState::new(),
            focus_sink: Rc::new(RefCell::new(None)),
            browser_sink: Rc::new(RefCell::new(None)),
            new_window: new_window::NewWindowState::default(),
            content_gesture: content_gesture::ContentGestureState::default(),
            peek: peek::PeekState::default(),
            favicon_images: favicon_presentation::FaviconImageMirror::default(),
            favicon_mirror_version: 0,
            #[cfg(feature = "testkit")]
            performance,
        }
    }

    /// Attach the concrete native engine handle (macOS only; the composition
    /// root passes it after construction, like the portal host). The find
    /// adapter binds a live `WKWebView` through it.
    #[cfg(target_os = "macos")]
    pub fn set_find_engine(&mut self, engine: crate::engine::NativeEngineRef) {
        self.find_engine = Some(engine);
    }

    /// Attach the concrete native engine handle (macOS only; the composition
    /// root passes it after construction, like the portal host). The
    /// window-geometry persist source reads the live session's window
    /// content rect through it.
    #[cfg(target_os = "macos")]
    pub fn set_geometry_engine(&mut self, engine: crate::engine::NativeEngineRef) {
        self.geometry_engine = Some(engine);
    }

    /// Deterministic persistence flush (testkit gateway seam): synchronously
    /// writes the in-memory command-history provider to the store in one
    /// transaction — the deterministic seam E2E uses to prove persistence
    /// across restarts; the production background writer uses the identical
    /// repository write.
    #[cfg(feature = "testkit")]
    pub(crate) fn flush_persistence(&self) {
        self.store.flush_persistence();
    }

    /// The production Saved Page quit flush: synchronously persists the
    /// current in-memory Saved Page provider to the store in one transaction
    /// (B2 packet `saved-pages-actions-command`). The owning window-close
    /// seam runs it right before `cx.quit()` so a normal quit never loses a
    /// committed save; the deterministic testkit seam and the background
    /// writer use the identical repository write.
    pub(crate) fn flush_saved_pages(&self) {
        self.store.flush_saved_pages();
    }

    /// The one app-owned Saved Page command authority (B2 packet
    /// `saved-pages-actions-command`): Cmd-D, the testkit dispatch, and the
    /// B3 Library route save/remove/open through the same seam the menu
    /// rows share.
    pub(crate) fn saved_pages_command(&self) -> saved_pages::SavedPageCommand {
        self.saved_pages
    }

    // ---- Phase-7 warm-restore composition ----------------------------------

    /// The startup restore evidence (see [`RestoreEvidence`]); set by the
    /// composition root after the controller was built. Testkit-only: the
    /// testkit snapshot surfaces it; production composition consumes the
    /// returned evidence directly at build time.
    #[cfg(feature = "testkit")]
    pub fn restore_evidence(&self) -> RestoreEvidence {
        self.restore_evidence
    }

    /// Record the startup restore evidence (the composition root passes the
    /// chrome-first plan's deferred count, or nothing on the fresh path).
    pub fn set_restore_evidence(&mut self, evidence: RestoreEvidence) {
        self.restore_evidence = evidence;
    }

    /// The current structural browser model as the persistence DTO: the
    /// controller's live model export mapped through the app-owned mapping,
    /// plus the single window's presentation rows (geometry + sidebar). The
    /// window identity is derived above every domain id (global uniqueness
    /// across kinds), so a fresh creation can never collide with the window
    /// row. Appearance values are the controller's effective per-space
    /// appearance (schema v4 persists theme, identity and paper).
    fn current_browser_state(&self) -> bbr_persistence::BrowserState {
        use bbr_persistence::{SidebarState, WindowGeometry, WindowState};
        let intent = self.controller.intent();
        let profiles = crate::store::browser::intent_to_persisted(&intent, |space| {
            self.controller.space_appearance(space)
        });
        let window_id = crate::store::browser::derived_window_id(&profiles);
        let geometry = self
            .window_geometry
            .borrow()
            .map(|[x, y, width, height]| WindowGeometry {
                x,
                y,
                width,
                height,
            });
        bbr_persistence::BrowserState {
            profiles,
            windows: vec![WindowState {
                id: window_id,
                geometry,
                sidebar: SidebarState {
                    width: f64::from(self.sidebar.requested()),
                    collapsed: !self.sidebar_visible,
                    last_expanded_width: f64::from(self.sidebar.last_expanded()),
                },
            }],
        }
    }

    /// Poke the bounded browser-state writer after a structural mutation
    /// (open/close/pin/reorder/move/split/space/profile/sidebar): the latest
    /// model replaces the shared slot and the background writer coalesces
    /// the write, so a force quit recovers the last structurally persisted
    /// model. Off the render path; never blocks.
    pub fn persist_structure(&self) {
        self.store.poke_browser_state(self.current_browser_state());
    }

    /// The production quit flush: refresh the slot from the current model
    /// and write the browser tables synchronously in one transaction (the
    /// same repository write the background writer uses).
    pub fn flush_browser_state(&self) {
        self.persist_structure();
        self.store.flush_browser_state();
    }

    /// Restore the persisted window presentation state (sidebar geometry +
    /// collapsed flag) after the window opened. The window bounds were
    /// already applied by the composition root through the window options;
    /// this composes the remaining presentation row truthfully (invalid
    /// optional values were already recovered by the repository).
    pub fn restore_window_state(&mut self, state: &bbr_persistence::SidebarState) {
        self.sidebar
            .restore(state.width as f32, state.last_expanded_width as f32);
        self.sidebar_visible = !state.collapsed;
    }

    /// Issue a real WebKit content probe for a live view (testkit-only;
    /// macOS): reads the committed document in BBR's own client content
    /// world plus the native `WKWebView` url/title through the concrete
    /// engine handle (never creating or moving sessions) and publishes the
    /// observation on the next snapshot's `web_content_probe`. Returns
    /// `false` when the native engine is absent or the view is unknown.
    #[cfg(all(target_os = "macos", feature = "testkit"))]
    pub fn probe_web_content(&mut self, view: bbr_web::WebViewId) -> bool {
        let Some(native) = self.find_engine.as_ref() else {
            return false;
        };
        let Some(web_view) = native.borrow().web_view(view) else {
            return false;
        };
        let slot = self.web_content_probe.clone();
        bbr_web_macos::content_probe::probe_web_view(&web_view, move |probe| {
            *slot.borrow_mut() = Some((view, probe));
        });
        true
    }

    /// Attach the concrete native engine handle (macOS only; the composition
    /// root passes it after construction, like the find handle). The zoom
    /// target binds the active `WKWebView` through
    /// [`WebEngineWk::web_view`](bbr_web_macos::WebEngineWk::web_view) and
    /// applies the level with `WebKitZoomAdapter` — a retained clone, never
    /// a session mutation.
    #[cfg(target_os = "macos")]
    pub fn set_zoom_engine(&mut self, engine: crate::engine::NativeEngineRef) {
        let target_engine = engine.clone();
        self.zoom.set_target(std::rc::Rc::new(move |view, zoom| {
            let Some(web_view) = target_engine.borrow().web_view(view) else {
                return false;
            };
            let mtm = objc2::MainThreadMarker::new().expect("zoom target on the main thread");
            let adapter = bbr_web_macos::WebKitZoomAdapter::new(mtm, web_view);
            adapter.apply(zoom);
            true
        }));
        self.zoom_engine = Some(engine);
    }

    /// Attach the concrete native engine handle (macOS only; the composition
    /// root passes it after construction, like the find/zoom handles). The
    /// context-menu bridge diagnostics (right-click monitor + ownership
    /// classification counters) are read through it; the menu itself routes
    /// actions through the portable engine seam.
    #[cfg(target_os = "macos")]
    pub fn set_menu_engine(&mut self, engine: crate::engine::NativeEngineRef) {
        self.menu_engine = Some(engine);
    }

    /// Install the macOS download composition (macOS only): the concrete
    /// adapter, the engine handle (for the live-web-view capture), and the
    /// L6 event receiver the pump drains into [`Self::on_download_event`].
    /// The controller's action sink dispatches through the adapter with the
    /// shared web-view slot; the adapter's event sink forwards every L6
    /// event into the channel the pump delivers on the main thread.
    #[cfg(target_os = "macos")]
    pub fn install_downloads(
        &mut self,
        adapter: std::rc::Rc<bbr_web_macos::downloads_adapter::NativeDownloadAdapter>,
        engine: crate::engine::NativeEngineRef,
        event_tx: smol::channel::Sender<bbr_web::downloads::DownloadEvent>,
    ) {
        // Adapter → pump: every native terminal/progress signal becomes an
        // L6 event on the main thread (the adapter itself already filtered
        // and mirrored the machine, so the pump applies only accepted
        // events).
        adapter.set_event_sink(Box::new(move |event| {
            let _ = event_tx.try_send(event);
        }));
        let action_adapter = adapter.clone();
        let action_slot = self.download_web_view.clone();
        // Controller → adapter: `DownloadAction`s emitted synchronously by
        // the portable controller (start/retry/cancel) dispatch onto the
        // real WKDownload lifecycle with the active session's view.
        self.downloads
            .controller
            .set_action_sink(Box::new(move |action| {
                if let Err(error) = bbr_web_macos::downloads_adapter::dispatch_action(
                    &action_adapter,
                    action,
                    &action_slot,
                ) {
                    log::warn!("download action dispatch failed: {error:?}");
                }
            }));
        self.downloads_adapter = Some(adapter);
        self.download_engine = Some(engine);
    }

    /// Open the initial controller-backed tab after the first window frame.
    pub fn open_initial_tab(&mut self, initial_url: &str) {
        if let Err(error) = self.controller.open_tab(initial_url) {
            log::error!("could not open initial tab: {error}");
        }
    }

    /// Clear the §11.3 cross-context command-field hint: the deterministic
    /// lifetime boundary is the next chrome context mutation — a space
    /// activation re-evaluates it, a tab mutation or command-bar entry ends
    /// the one-interaction display.
    pub(super) fn clear_command_profile_hint(&mut self) {
        self.command_profile_hint = None;
    }

    pub fn open_tab(&mut self, url: &str, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self
            .controller
            .open_tab(url)
            .map_err(|error| {
                log::error!("could not open tab: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            // A new active tab/session ends the one-per-window find
            // presentation (canonical interaction contract).
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            self.ensure_active_visible();
            // The new active view may carry a tracked incident (a tab
            // switched away from while its surface was shown): re-assert
            // native visibility against the surface.
            self.sync_error_surface_visibility();
            // The new active view may be reported unresponsive (a hung tab
            // the user switched to): re-assert the §11.8a pane visibility.
            self.sync_unresponsive_visibility();
            // While the Library is open it is the single visibility
            // authority: the fresh active view is hidden (never destroyed).
            self.sync_library_visibility();
            self.zoom_after_activation();
            // Blueprint D25: a tab committed from New Tab in Folder joins
            // its folder.
            self.join_pending_new_tab_folder(cx);
            self.persist_structure();
            self.sync_peek_surface(cx);
            cx.notify();
        }
        result
    }

    pub fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self
            .controller
            .select_tab(index)
            .map_err(|error| {
                log::error!("could not select tab {index}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            self.ensure_active_visible();
            // The newly selected view may carry a tracked incident (the
            // surface shows for the active view only): re-assert native
            // visibility against the surface.
            self.sync_error_surface_visibility();
            // The newly selected view may be reported unresponsive (the
            // §11.8a pane shows for the exact active view only): re-assert
            // its visibility.
            self.sync_unresponsive_visibility();
            // While the Library is open it is the single visibility
            // authority: the newly selected view is hidden (never
            // destroyed).
            self.sync_library_visibility();
            self.zoom_after_activation();
            self.persist_structure();
            self.sync_peek_surface(cx);
            cx.notify();
        }
        result
    }

    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        // The closed tab's disposable session id, captured before the
        // controller destroys it (the `TabClosed` outcome only carries the
        // index): its error-surface presentation must be dropped with it.
        let closed_view = self
            .controller
            .snapshot()
            .tabs
            .get(index)
            .and_then(|tab| tab.web_view);
        let result = self
            .controller
            .close_tab(index)
            .map_err(|error| {
                log::error!("could not close tab {index}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            // A closed tab's session is gone: drop its error-surface
            // presentation entirely (no native restore is needed — the
            // view id is released with the tab).
            if let Some(view) = closed_view {
                self.error_surface.forget_view(view);
                // The closed view's session is gone: drop the Library's
                // hide bookkeeping too (no native restore is needed).
                self.library.forget_view(view);
                // And the §11.8a pane bookkeeping for the closed session.
                self.unresponsive_forget_view(view);
            }
            self.ensure_active_visible();
            // The close may have surfaced a different active view (or the
            // empty pane): re-assert the §11.8a pane visibility.
            self.sync_unresponsive_visibility();
            self.persist_structure();
            self.sync_peek_surface(cx);
            cx.notify();
        }
        result
    }

    pub fn close_active_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let chrome_focused = self.chrome_focused(window);
        let prior_mode = self.command_bar_mode;
        let outcome = self
            .controller
            .snapshot()
            .active_index
            .and_then(|index| self.close_tab(index, cx));
        outcome.as_ref()?;

        if self.controller.snapshot().active_view.is_none() {
            self.enter_empty_new_tab(window, cx);
        } else if chrome_focused {
            self.command_bar_mode = if prior_mode.is_active() {
                CommandBarMode::Location
            } else {
                CommandBarMode::Inactive
            };
            self.command_bar_return = None;
            self.sync_command_bar();
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
        #[cfg(feature = "testkit")]
        {
            self.performance.expect_state(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
                self.chrome_focused(window),
                self.command_bar_mode.into(),
            );
            self.commit_benchmark_sync(window);
        }
        outcome
    }

    /// Report the actual chrome-focus state through the focus sink (main
    /// thread). No-op when no sink is installed.
    fn emit_focus(&mut self, window: &Window) {
        self.commit_folder_rename_on_focus_change(window);
        if let Some(sink) = self.focus_sink.borrow_mut().as_mut() {
            let focused = self.chrome_focused(window);
            sink(focused);
        }
    }

    /// True when any chrome control owns GPUI focus (command bar, sidebar, or
    /// the visible tab list). The interaction contract's "browser chrome"
    /// domain is the union: the tab list handle is a chrome control, so a
    /// pointer-selected row keeps `chrome_focused` true — and the resize
    /// divider, the plus menu, and the folders region are chrome too, so
    /// their focused states report the same domain.
    pub(super) fn chrome_focused(&self, window: &Window) -> bool {
        self.chrome_handle.is_focused(window)
            || self.space_strip_handle.is_focused(window)
            || self.tab_list_handle.is_focused(window)
            || self.divider_handle.is_focused(window)
            || self.u2.plus_menu_handle.is_focused(window)
            || self.u2.folders_handle.is_focused(window)
            || self.downloads_panel_handle.is_focused(window)
            || self.downloads_control_handle.is_focused(window)
            || self.library_handle.is_focused(window)
            || self.space_config.handle.is_focused(window)
    }

    fn emit_browser(&mut self, event: &ControllerEvent) {
        if let Some(sink) = self.browser_sink.borrow_mut().as_mut() {
            sink(event);
        }
    }

    /// `window` is used only by the testkit feature (synchronous benchmark
    /// commit); it is otherwise just threaded through for caller symmetry.
    #[cfg_attr(not(feature = "testkit"), allow(unused_variables))]
    pub fn toggle_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sidebar_visible {
            self.sidebar.collapse();
        } else {
            self.sidebar.expand();
        }
        self.sidebar_visible = !self.sidebar_visible;
        self.persist_structure();
        #[cfg(feature = "testkit")]
        {
            self.performance.arm_browser_state_if_pending(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
            );
            // A benchmark keystroke path arms a pending here and relies on a
            // later commit; commit synchronously from authoritative state so
            // the sample is not dropped when no engine-slot callback fires.
            if self.performance.presentation_pending()() {
                self.commit_benchmark_sync(window);
            }
        }
        cx.notify();
    }
}

impl Focusable for BbrApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.chrome_handle.clone()
    }
}
