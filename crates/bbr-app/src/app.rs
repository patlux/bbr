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
use crate::portal::geometry::Sz;
use crate::portal::probe::PortalProbeState;
use crate::portal::state::{PortalHost, PortalRequest, PortalState, PortalTransition};

use command_rank::CommandRanking;
use command_text::CommandText;
use palette::CommandSurface;
#[cfg(feature = "testkit")]
mod benchmarks;
#[cfg(feature = "testkit")]
mod benchmarks_dense;
#[cfg(feature = "testkit")]
mod benchmarks_pointer;
mod command_activate;
#[cfg(feature = "testkit")]
mod command_benchmarks;
mod command_corpus;
pub(crate) mod command_rank;
mod command_surface;
mod command_text;
mod drag;
mod drag_target;
mod interaction;
pub(crate) mod palette;
mod render;
mod sidebar;
mod sidebar_view;
mod spaces;
mod tab;
#[cfg(feature = "testkit")]
mod testkit;

/// Sidebar width follows the §8 normal-density token (260 pt); Phase-4
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
    /// Focus-change sink (main thread): called after any focus mutation with
    /// the actual chrome-focus state. Wired by the testkit glue to broadcast
    /// `ChromeFocusChanged`; unused in production builds.
    focus_sink: Rc<RefCell<Option<FocusSink>>>,
    browser_sink: Rc<RefCell<Option<BrowserSink>>>,
    #[cfg(feature = "testkit")]
    performance: PerformanceProbe,
}

impl BbrApp {
    pub fn new(
        engine: EngineRef,
        controller: BrowserController,
        engine_kind: &'static str,
        evt_tx: smol::channel::Sender<WebEvent>,
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
            command_field_rect: std::rc::Rc::new(std::cell::RefCell::new(None)),
            command_text: CommandText::new(),
            command_ranking: CommandRanking::default(),
            command_profile_hint: None,
            sidebar_visible: true,
            focus_sink: Rc::new(RefCell::new(None)),
            browser_sink: Rc::new(RefCell::new(None)),
            #[cfg(feature = "testkit")]
            performance,
        }
    }

    /// Open the initial controller-backed tab after the first window frame.
    pub fn open_initial_tab(&mut self, initial_url: &str) {
        if let Err(error) = self.controller.open_tab(initial_url) {
            log::error!("could not open initial tab: {error}");
        }
    }

    /// Attach the one native portal window (macOS construction seam).
    /// Exactly one per browser window; hidden and non-activating until a
    /// native presentation drives it (task packet `native-portal-host`).
    #[cfg(target_os = "macos")]
    pub fn set_native_portal(&mut self, bridge: crate::portal::native::NativePortalBridge) {
        self.native_portal = Some(bridge);
    }

    /// Narrow main-thread diagnostics of the native portal host, when
    /// attached (DESIGN §15 snapshot semantics; the testkit snapshot
    /// mirrors these fields). `None` when no portal is attached.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit snapshot only; production consumers arrive with U1
    pub fn native_portal_diagnostics(
        &self,
    ) -> Option<bbr_web_macos::portal::PortalNativeDiagnostics> {
        self.native_portal
            .as_ref()
            .and_then(crate::portal::native::NativePortalBridge::diagnostics)
    }

    /// The shared portal render state (the composition root passes a clone
    /// into the native bridge so the portal window renders exactly the
    /// active probe or product presentation commanded by the app).
    pub fn portal_probe_handle(&self) -> Rc<RefCell<PortalProbeState>> {
        self.portal_probe.clone()
    }

    /// Present (or refresh) a portal request through the general Portal API
    /// (task packet `native-portal-host`): updates the one-active portal
    /// state and, for a [`PortalHost::NativeAboveWeb`] presentation,
    /// resolves the placement against the window, records the active
    /// presentation, and orders the native portal window front (a non-activating
    /// panel never becomes key or main). Exactly one presentation stays
    /// active; re-presenting the same id refreshes it in place. A
    /// [`PortalHost::GpuiDeferred`] request is state-only until a production
    /// deferred caller exists.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // probe + U1 production presentation seam
    pub fn portal_present(
        &mut self,
        request: PortalRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> PortalTransition {
        if request.host == PortalHost::NativeAboveWeb {
            let viewport = Sz::new(
                f32::from(window.bounds().size.width),
                f32::from(window.bounds().size.height),
            );
            self.portal_probe.borrow_mut().present(request, viewport);
        }
        self.activate_portal_request(request, cx)
    }

    /// Activate an already-prepared presentation; render state is populated
    /// first so ordering the child in cannot expose stale content.
    pub(crate) fn activate_portal_request(
        &mut self,
        request: PortalRequest,
        cx: &mut Context<Self>,
    ) -> PortalTransition {
        let transition = self.portal.present(request);
        #[cfg(target_os = "macos")]
        if request.host == PortalHost::NativeAboveWeb {
            self.drive_native_present(cx);
        }
        transition
    }

    /// Deterministically dismiss the active presentation (the production
    /// dismissal method): the active portal content clears and the native portal
    /// window orders out. Returns whether a presentation was actually
    /// active.
    pub fn portal_dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.portal.dismiss() {
            return false;
        }
        self.portal_probe.borrow_mut().clear();
        #[cfg(target_os = "macos")]
        {
            if let Some(bridge) = self.native_portal.as_mut() {
                bridge.set_visible(false);
            }
            // Re-render the portal root so the dismissed surface is cleared.
            // Deferred: dismissal can run inside the portal window's own
            // event dispatch (real backdrop press), where updating it again
            // would double-lease the root entity.
            if let Some(handle) = self
                .native_portal
                .as_ref()
                .map(|bridge| bridge.portal_handle())
            {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_root, _window, cx| cx.notify());
                });
            }
        }
        true
    }

    /// The exact production outside-dismiss method (DESIGN §13 one-outside-
    /// pointer-action): the portal backdrop's pointer-press listener and the
    /// testkit `PortalOutsideDismiss` request both call this. Only dismisses
    /// when the active presentation's [`DismissalPolicy`] allows
    /// outside-pointer dismissal.
    pub fn portal_outside_dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.portal.dismisses_on_outside_press() {
            return false;
        }
        self.portal_dismiss(cx)
    }

    /// Order the native portal window front and re-render its active content.
    /// Deferred to avoid re-entering another window during an app update.
    #[cfg(target_os = "macos")]
    fn drive_native_present(&mut self, cx: &mut Context<Self>) {
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.set_visible(true);
        }
        if let Some(handle) = self
            .native_portal
            .as_ref()
            .map(|bridge| bridge.portal_handle())
        {
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_root, _window, cx| cx.notify());
            });
        }
    }

    /// Deterministically tear down the one native portal host (task packet
    /// teardown evidence): removes the child relation, orders the portal
    /// window out, closes it, and drops the bridge — the app owns no portal
    /// afterwards. Idempotent and safe when no host was ever attached; the
    /// probe surface and the one-active presentation state clear with it.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit teardown seam; production callers arrive with U1
    pub fn portal_teardown(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(mut bridge) = self.native_portal.take() {
            bridge.teardown();
        }
        self.portal_probe.borrow_mut().clear();
        self.portal.dismiss();
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
    /// divider is chrome too, so its focused state reports the same domain.
    pub(super) fn chrome_focused(&self, window: &Window) -> bool {
        self.chrome_handle.is_focused(window)
            || self.space_list_handle.is_focused(window)
            || self.tab_list_handle.is_focused(window)
            || self.divider_handle.is_focused(window)
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
