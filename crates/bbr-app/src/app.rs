//! The `BbrApp` root view: GPUI chrome (sidebar + command bar + status) with
//! the native web-view slot; also owns focus choreography and navigation
//! command routing.

use bbr_browser::{BrowserController, ControllerEvent};
#[cfg(feature = "testkit")]
use bbr_web::WebAction;
use bbr_web::WebEvent;
use gpui::{App, Context, FocusHandle, Focusable, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::command_bar::{CommandBarMode, CommandBarReturn};
use crate::engine::EngineRef;
#[cfg(feature = "testkit")]
use crate::performance::PerformanceProbe;
#[cfg(feature = "testkit")]
use crate::shell::TestSnapshot;
mod interaction;
mod render;
mod spaces;

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
    /// Current command-bar intent. `NewTab` stages navigation without creating
    /// a domain tab or native session until Enter commits it.
    pub command_bar_mode: CommandBarMode,
    command_bar_return: Option<CommandBarReturn>,
    /// Current command-bar text (mirrors the live URL when not editing).
    pub command_bar_text: String,
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
            command_bar_mode: CommandBarMode::Inactive,
            command_bar_return: None,
            command_bar_text: String::new(),
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

    #[cfg(feature = "testkit")]
    pub fn create_profile_space(
        &mut self,
        profile_title: &str,
        space_title: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self
            .controller
            .create_profile_space(profile_title, space_title);
        self.apply_mutation(result, cx)
    }

    #[cfg(feature = "testkit")]
    pub fn create_space(
        &mut self,
        profile: bbr_web::ProfileId,
        title: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.create_space_event(profile, title);
        self.apply_mutation(result, cx)
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
            self.sync_command_bar();
            self.emit_browser(outcome);
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
            self.sync_command_bar();
            self.emit_browser(outcome);
            cx.notify();
        }
        result
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_select_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::WarmTabSwitch,
            )
            .is_err()
        {
            return None;
        }
        let outcome = self.select_tab(index, cx);
        if outcome.is_some() {
            // Mutations only schedule a render; the frame that runs
            // `observe_render_state` with the new active view has not happened
            // yet when this synchronous path commits. Refresh the probe with
            // the authoritative post-mutation controller snapshot so the
            // presentation commit sees the real state and cannot silently drop
            // a sample when the selection changes views.
            let active_view = self.controller.snapshot().active_view;
            let chrome_focused = self.chrome_focused(window);
            let mode = self.command_bar_mode.into();
            self.performance.observe_render_state(
                active_view,
                self.sidebar_visible,
                chrome_focused,
                mode,
            );
            self.performance
                .expect_browser_state(active_view, self.sidebar_visible);
            self.performance.commit_current_presentation();
        } else {
            self.performance.cancel();
        }
        outcome
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
            self.sync_command_bar();
            self.emit_browser(outcome);
            cx.notify();
        }
        result
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_click_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::TabSelectPointer,
            )
            .is_err()
        {
            return false;
        }
        // The production pointer path, exactly as the row mouse-down listener
        // drives it: chrome focus domain preserved while the controller
        // selects. The benchmark commits synchronously from the authoritative
        // post-mutation snapshot (same rationale as `benchmark_select_tab`).
        self.select_row_via_pointer(index, window, cx);
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        true
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_reopen_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::TabReopen,
            )
            .is_err()
        {
            return false;
        }
        // Production reopen path (contract: preserved identity, fresh session,
        // focus domain untouched). The benchmark arms and commits synchronously
        // from the authoritative post-mutation snapshot, like the other
        // interaction benchmarks.
        let outcome = self.reopen_tab(window, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        true
    }

    #[cfg(feature = "testkit")]
    pub fn move_tab(
        &mut self,
        index: usize,
        to_space: bbr_browser::SpaceId,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.move_tab(index, to_space);
        self.apply_mutation(result, cx)
    }

    #[cfg(feature = "testkit")]
    pub fn suspend_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self.controller.suspend_tab(index);
        self.apply_mutation(result, cx)
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

    /// Route a generic [`WebAction`] through the focused controller session.
    #[cfg(feature = "testkit")]
    pub fn web_action(&mut self, action: WebAction) -> Option<ControllerEvent> {
        self.controller
            .navigate(action)
            .map_err(|error| {
                log::error!("could not route web action: {error}");
                error
            })
            .ok()
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
    /// pointer-selected row keeps `chrome_focused` true.
    pub(super) fn chrome_focused(&self, window: &Window) -> bool {
        self.chrome_handle.is_focused(window)
            || self.space_list_handle.is_focused(window)
            || self.tab_list_handle.is_focused(window)
    }

    /// Install a focus-change sink (used by the E2E gateway).
    #[cfg(feature = "testkit")]
    pub fn set_focus_sink(&mut self, sink: FocusSink) {
        *self.focus_sink.borrow_mut() = Some(sink);
    }

    #[cfg(feature = "testkit")]
    pub fn set_browser_sink(&mut self, sink: BrowserSink) {
        *self.browser_sink.borrow_mut() = Some(sink);
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_page_ready(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        expected_view: bbr_web::WebViewId,
        expected_title: impl Into<String>,
    ) -> bool {
        self.performance
            .arm_page_ready(operation_id, expected_view, expected_title);
        true
    }

    #[cfg(feature = "testkit")]
    pub fn begin_benchmark(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        kind: bbr_testkit::protocol::BenchmarkKind,
    ) -> bool {
        self.performance.begin(operation_id, kind).is_ok()
    }

    #[cfg(feature = "testkit")]
    pub fn cancel_benchmark(&self) {
        self.performance.cancel();
    }

    #[cfg(feature = "testkit")]
    pub fn commit_benchmark_sync(&self, window: &Window) {
        // Refresh the probe with the authoritative post-mutation controller
        // state and commit synchronously. The `on_next_frame` path does not
        // reliably fire in the offscreen benchmark harness (startup/page-ready
        // only return through engine callbacks), so a frame-dependent commit
        // silently dropped interaction samples. The semantic state check inside
        // `commit_current_presentation` still rejects fast-but-wrong results.
        let active_view = self.controller.snapshot().active_view;
        let chrome_focused = self.chrome_focused(window);
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance.commit_current_presentation();
    }

    #[cfg(feature = "testkit")]
    fn apply_mutation(
        &mut self,
        result: Result<ControllerEvent, bbr_browser::ControllerError>,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let outcome = result
            .map_err(|error| {
                log::error!("browser mutation failed: {error}");
                error
            })
            .ok();
        if let Some(event) = &outcome {
            self.emit_browser(event);
            cx.notify();
        }
        outcome
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

    #[cfg(feature = "testkit")]
    pub fn benchmark_toggle_sidebar(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SidebarToggle,
            )
            .is_err()
        {
            return false;
        }
        // `toggle_sidebar` arms the pending and commits synchronously.
        self.toggle_sidebar(window, cx);
        true
    }

    /// A snapshot for the E2E protocol (also used by unit tests).
    #[cfg(feature = "testkit")]
    pub fn snapshot(&self, window: &Window) -> TestSnapshot {
        let browser = self.controller.snapshot();
        let theme = self.theme();
        let rows = self.row_visuals(&browser.tabs);
        let budget = crate::shell::space_title_budget_chars(&theme);
        TestSnapshot {
            url: browser.nav.url.clone(),
            title: browser.nav.title.clone(),
            loading: browser.nav.loading,
            can_go_back: browser.nav.can_go_back,
            can_go_forward: browser.nav.can_go_forward,
            web_view_hosted: browser.web_view_hosted,
            active_profile: browser.active_profile,
            active_profile_title: browser.active_profile_title.clone(),
            active_profile_hue: browser.active_profile_hue.map(|hue| hue.name().to_string()),
            active_space: bbr_testkit::protocol::TestSpaceId(browser.active_space.0),
            tabs: browser.tabs.clone(),
            profiles: browser.profiles.clone(),
            spaces: browser.spaces.clone(),
            active_tab_index: browser.active_index,
            active_web_view: browser.active_view,
            engine_sessions: browser.engine_sessions,
            engine_kind: self.engine_kind,
            chrome_focused: self.chrome_focused(window),
            shell_modes: bbr_testkit::protocol::ShellModes {
                theme: match theme.mode {
                    crate::theme::ThemeMode::Light => "light".into(),
                    crate::theme::ThemeMode::Dark => "dark".into(),
                },
                contrast: match theme.contrast {
                    crate::theme::ContrastMode::Standard => "standard".into(),
                    crate::theme::ContrastMode::Increased => "increased".into(),
                },
                density: match theme.density {
                    crate::theme::Density::Small => "small".into(),
                    crate::theme::Density::Normal => "normal".into(),
                    crate::theme::Density::ExtraLarge => "extralarge".into(),
                },
            },
            tab_states: rows
                .iter()
                .map(|row| match row.state {
                    crate::components::TabVisualState::Default => {
                        bbr_testkit::protocol::TabVisualState::Default
                    }
                    crate::components::TabVisualState::Selected => {
                        bbr_testkit::protocol::TabVisualState::Selected
                    }
                    crate::components::TabVisualState::Loading => {
                        bbr_testkit::protocol::TabVisualState::Loading
                    }
                    crate::components::TabVisualState::Error => {
                        bbr_testkit::protocol::TabVisualState::Error
                    }
                })
                .collect(),
            row_titles: rows.iter().map(|row| row.title.to_string()).collect(),
            space_states: crate::shell::space_states(&browser)
                .into_iter()
                .map(|state| match state {
                    crate::components::SpaceVisualState::Default => {
                        bbr_testkit::protocol::SpaceVisualState::Default
                    }
                    crate::components::SpaceVisualState::Selected => {
                        bbr_testkit::protocol::SpaceVisualState::Selected
                    }
                })
                .collect(),
            space_titles: crate::shell::space_titles(&browser, budget),
            visible_ids: crate::shell::visible_ids(
                &browser,
                self.chrome_focused(window),
                self.command_bar_mode,
                self.sidebar_visible,
            ),
            command_bar_mode: self.command_bar_mode.into(),
            command_bar_text: self.command_bar_text.clone(),
            sidebar_visible: self.sidebar_visible,
            window_size: [
                f32::from(window.bounds().size.width),
                f32::from(window.bounds().size.height),
            ],
        }
    }
}

impl Focusable for BbrApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.chrome_handle.clone()
    }
}
