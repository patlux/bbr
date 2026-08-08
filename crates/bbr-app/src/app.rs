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
mod benchmarks;
#[cfg(feature = "testkit")]
mod benchmarks_dense;
#[cfg(feature = "testkit")]
mod benchmarks_pointer;
#[cfg(feature = "testkit")]
mod benchmarks_u2;
mod command_activate;
#[cfg(feature = "testkit")]
mod command_benchmarks;
mod command_corpus;
pub(crate) mod command_rank;
mod command_surface;
mod command_text;
pub(crate) mod downloads;
pub(crate) mod downloads_behavior;
#[cfg(feature = "testkit")]
mod downloads_benchmarks;
pub(crate) mod downloads_render;
mod drag;
mod drag_target;
pub(crate) mod find;
#[cfg(feature = "testkit")]
mod find_benchmarks;
mod interaction;
pub(crate) mod palette;
mod portal_host;
mod render;
mod sidebar;
mod sidebar_view;
mod spaces;
mod tab;
#[cfg(feature = "testkit")]
mod testkit;
mod u2;
mod u2_surface;
#[cfg(test)]
mod u2_tests;

/// Sidebar width follows the §8 normal-density token (280 pt); Phase-4
/// resize/drag will make it adjustable.
/// Focus-change sink: invoked on the main thread after any focus mutation.
pub type FocusSink = Box<dyn FnMut(bool)>;
/// Semantic browser-mutation sink shared by shortcuts and testkit commands.
pub type BrowserSink = Box<dyn FnMut(&ControllerEvent)>;

/// Default window size for the spike.
pub const DEFAULT_WINDOW_SIZE: (f32, f32) = (1080.0, 700.0);

/// The root view of the browser window.
pub struct BbrApp {
    engine: EngineRef,
    controller: BrowserController,
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    engine_kind: &'static str,
    /// GPUI focus handle for the chrome (sidebar/command bar).
    chrome_handle: FocusHandle,
    /// GPUI focus handle for the visible space list (arrow/enter keys).
    space_list_handle: FocusHandle,
    /// GPUI focus handle for the visible tab list (arrow/backspace keys).
    tab_list_handle: FocusHandle,
    /// GPUI focus handle for the sidebar resize divider (←/→ resize, 0 reset).
    divider_handle: FocusHandle,
    /// Production scroll state of the regular tab region (DESIGN.md §11.6):
    /// the native wheel listener and the keyboard traversal mutate the same
    /// tracked offset, and the render derives its bounded row window from it.
    dense_scroll: ScrollHandle,
    /// The last render's dense window, recorded during `Render::render` so
    /// the testkit snapshot reports exactly what the shell built (bounded
    /// rendered rows, active-row visibility). `pub(crate)` because the
    /// gateway dispatch (`crate::testkit`) reads the anchor for the dense
    /// benchmark and the app testkit snapshot mirrors the full state.
    pub(crate) dense_window: std::cell::Cell<crate::dense::DenseRenderState>,
    /// Adjustable sidebar geometry (DESIGN.md §5/§11.4): min/default/max
    /// widths, narrow-window clamping, collapse memory, drag state.
    pub sidebar: crate::sidebar::SidebarGeometry,
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
    /// DESIGN.md §10.3 pressed state: the row whose primary button is
    /// currently down (driven by the production mouse-down/up listeners).
    pressed_row: Option<usize>,
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
    /// The last frame's command-field rect in window coordinates, recorded
    /// by the field anchor element (DESIGN.md §11.4): the exact anchor for
    /// the field-anchored overlay (top/leading edge flush, width from the
    /// field) without duplicating the chrome layout math. `None` until the
    /// field has rendered at least once.
    command_field_rect: std::rc::Rc<std::cell::RefCell<Option<WebRect>>>,
    /// The command-field text-edit model (DESIGN.md §11.5): UTF-8-safe
    /// cursor/selection, full selection on `Cmd-L`, replace-selection
    /// typing, backspace, and horizontal cursor movement. The text mirrors
    /// the live URL when not editing.
    pub command_text: CommandText,
    /// The cancellable, nonblocking ranking orchestration of the §11.5
    /// result surface (state machine + debounce handle). Inactive unless a
    /// location/new-tab mode is open.
    command_ranking: CommandRanking,
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
    /// Focus-change sink (main thread): called after any focus mutation with
    /// the actual chrome-focus state. Wired by the testkit glue to broadcast
    /// `ChromeFocusChanged`; unused in production builds.
    focus_sink: Rc<RefCell<Option<FocusSink>>>,
    browser_sink: Rc<RefCell<Option<BrowserSink>>>,
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

        BbrApp {
            engine: engine.clone(),
            controller,
            engine_kind,
            chrome_handle: cx.focus_handle(),
            space_list_handle: cx.focus_handle(),
            tab_list_handle: cx.focus_handle(),
            divider_handle: cx.focus_handle(),
            dense_scroll: ScrollHandle::new(),
            dense_window: std::cell::Cell::new(crate::dense::DenseRenderState::default()),
            sidebar: crate::sidebar::SidebarGeometry::new(),
            tab_drag: drag::TabDrag::default(),
            native_portal: None,
            portal: PortalState::new(crate::portal::probe::PORTAL_ID),
            portal_probe: Rc::new(RefCell::new(PortalProbeState::new())),
            hovered_row: None,
            pressed_row: None,
            command_bar_mode: CommandBarMode::Inactive,
            command_bar_return: None,
            command_surface: CommandSurface::None,
            find: crate::app::find::FindController::new(
                #[cfg(target_os = "macos")]
                find_tx,
            ),
            find_handle: cx.focus_handle(),
            auth: crate::app::auth::AuthCoordinator::new(),
            auth_handle: cx.focus_handle(),
            downloads: downloads::DownloadsState::new(),
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
            command_field_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            command_text: CommandText::new(),
            command_ranking: CommandRanking::default(),
            command_profile_hint: None,
            sidebar_visible: true,
            organization,
            u2: u2::U2State::new(cx),
            focus_sink: Rc::new(RefCell::new(None)),
            browser_sink: Rc::new(RefCell::new(None)),
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
            cx.notify();
        }
        result
    }

    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
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
            self.ensure_active_visible();
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
            || self.space_list_handle.is_focused(window)
            || self.tab_list_handle.is_focused(window)
            || self.divider_handle.is_focused(window)
            || self.u2.plus_menu_handle.is_focused(window)
            || self.u2.folders_handle.is_focused(window)
            || self.downloads_panel_handle.is_focused(window)
            || self.downloads_control_handle.is_focused(window)
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
