//! The `BbrApp` root view: GPUI chrome (sidebar + command bar + status) with
//! the native web-view slot; also owns focus choreography and navigation
//! command routing.

use bbr_browser::{BrowserController, ControllerEvent};
use bbr_web::WebEvent;
use gpui::{App, Context, FocusHandle, Focusable, Window};
use std::cell::RefCell;
use std::rc::Rc;

use crate::command_bar::{CommandBarMode, CommandBarReturn};
use crate::engine::EngineRef;
#[cfg(feature = "testkit")]
use crate::performance::PerformanceProbe;
mod interaction;
mod render;
mod spaces;
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
    /// pointer-selected row keeps `chrome_focused` true.
    pub(super) fn chrome_focused(&self, window: &Window) -> bool {
        self.chrome_handle.is_focused(window)
            || self.space_list_handle.is_focused(window)
            || self.tab_list_handle.is_focused(window)
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
}

impl Focusable for BbrApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.chrome_handle.clone()
    }
}
