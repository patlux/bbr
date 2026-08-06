//! Testkit-only presentation timing for real-app performance E2E.
//!
//! Operations start inside the app process immediately before the production
//! mutation. A sample completes at the end of the GPUI paint path after native
//! slot geometry is attached, immediately before GPUI presents the frame; socket
//! transport and harness scheduling are excluded.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement,
};
use bbr_web::{WebRect, WebViewId};

pub type PresentationSink = Box<dyn FnMut(PresentationMeasurement)>;

#[derive(Clone)]
pub struct PerformanceProbe {
    inner: Rc<RefCell<State>>,
}

struct State {
    process_started: Instant,
    latest_active_web_view: Option<WebViewId>,
    latest_sidebar_visible: bool,
    latest_chrome_focused: bool,
    latest_command_bar_mode: CommandBarMode,
    content_rect: Option<WebRect>,
    pending: Option<Pending>,
    queued: VecDeque<PresentationMeasurement>,
    sink: Option<PresentationSink>,
}

struct Pending {
    operation_id: BenchmarkOperationId,
    kind: BenchmarkKind,
    started_at: Instant,
    armed: bool,
    expected_active_web_view: Option<Option<WebViewId>>,
    expected_sidebar_visible: Option<bool>,
    expected_chrome_focused: Option<bool>,
    expected_command_bar_mode: Option<CommandBarMode>,
}

impl PerformanceProbe {
    pub fn new(process_started: Instant) -> Self {
        Self {
            inner: Rc::new(RefCell::new(State {
                process_started,
                latest_active_web_view: None,
                latest_sidebar_visible: true,
                latest_chrome_focused: false,
                latest_command_bar_mode: CommandBarMode::Inactive,
                content_rect: None,
                pending: Some(Pending {
                    operation_id: BenchmarkOperationId(0),
                    kind: BenchmarkKind::Startup,
                    started_at: process_started,
                    armed: true,
                    expected_active_web_view: None,
                    expected_sidebar_visible: Some(true),
                    expected_chrome_focused: None,
                    expected_command_bar_mode: None,
                }),
                queued: VecDeque::new(),
                sink: None,
            })),
        }
    }

    pub fn set_sink(&self, mut sink: PresentationSink) {
        let mut state = self.inner.borrow_mut();
        while let Some(measurement) = state.queued.pop_front() {
            sink(measurement);
        }
        state.sink = Some(sink);
    }

    pub fn begin(
        &self,
        operation_id: BenchmarkOperationId,
        kind: BenchmarkKind,
    ) -> Result<(), &'static str> {
        let mut state = self.inner.borrow_mut();
        if state.pending.is_some() {
            return Err("another presentation measurement is pending");
        }
        state.pending = Some(Pending {
            operation_id,
            kind,
            started_at: Instant::now(),
            armed: false,
            expected_active_web_view: None,
            expected_sidebar_visible: None,
            expected_chrome_focused: None,
            expected_command_bar_mode: None,
        });
        Ok(())
    }

    pub fn expect_browser_state(&self, active_web_view: Option<WebViewId>, sidebar_visible: bool) {
        let state = self.inner.borrow();
        let chrome_focused = state.latest_chrome_focused;
        let command_bar_mode = state.latest_command_bar_mode;
        drop(state);
        self.expect_state(
            active_web_view,
            sidebar_visible,
            chrome_focused,
            command_bar_mode,
        );
    }

    pub fn expect_state(
        &self,
        active_web_view: Option<WebViewId>,
        sidebar_visible: bool,
        chrome_focused: bool,
        command_bar_mode: CommandBarMode,
    ) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.armed = true;
            pending.expected_active_web_view = Some(active_web_view);
            pending.expected_sidebar_visible = Some(sidebar_visible);
            pending.expected_chrome_focused = Some(chrome_focused);
            pending.expected_command_bar_mode = Some(command_bar_mode);
        }
    }

    pub fn arm_browser_state_if_pending(
        &self,
        active_web_view: Option<WebViewId>,
        sidebar_visible: bool,
    ) {
        if self.inner.borrow().pending.is_some() {
            self.expect_browser_state(active_web_view, sidebar_visible);
        }
    }

    pub fn cancel(&self) {
        self.inner.borrow_mut().pending = None;
    }

    pub fn observe_render_state(
        &self,
        active_web_view: Option<WebViewId>,
        sidebar_visible: bool,
        chrome_focused: bool,
        command_bar_mode: CommandBarMode,
    ) {
        let mut state = self.inner.borrow_mut();
        state.latest_active_web_view = active_web_view;
        state.latest_sidebar_visible = sidebar_visible;
        state.latest_chrome_focused = chrome_focused;
        state.latest_command_bar_mode = command_bar_mode;
    }

    pub fn presentation_pending(&self) -> Rc<dyn Fn() -> bool> {
        let probe = self.clone();
        Rc::new(move || probe.inner.borrow().pending.is_some())
    }

    pub fn presentation_callback(&self) -> Rc<dyn Fn(WebRect)> {
        let probe = self.clone();
        Rc::new(move |rect| probe.commit_presentation(rect))
    }

    pub fn commit_current_presentation(&self) {
        let rect = { self.inner.borrow().content_rect };
        if let Some(rect) = rect {
            self.commit_presentation(rect);
        }
    }

    fn commit_presentation(&self, rect: WebRect) {
        let mut state = self.inner.borrow_mut();
        state.content_rect = Some(rect);
        if rect.width <= 0.0 || rect.height <= 0.0 || state.latest_active_web_view.is_none() {
            return;
        }

        let Some(pending) = state.pending.as_ref() else {
            return;
        };
        if !pending.armed
            || pending
                .expected_active_web_view
                .is_some_and(|expected| expected != state.latest_active_web_view)
            || pending
                .expected_sidebar_visible
                .is_some_and(|expected| expected != state.latest_sidebar_visible)
            || pending
                .expected_chrome_focused
                .is_some_and(|expected| expected != state.latest_chrome_focused)
            || pending
                .expected_command_bar_mode
                .is_some_and(|expected| expected != state.latest_command_bar_mode)
        {
            return;
        }

        let pending = state.pending.take().expect("pending measurement exists");
        let duration_ns = pending
            .started_at
            .elapsed()
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        let since_process_start_ns = state
            .process_started
            .elapsed()
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        let measurement = PresentationMeasurement {
            operation_id: pending.operation_id,
            kind: pending.kind,
            duration_ns,
            since_process_start_ns,
            monotonic_commit_ns: monotonic_now_ns(),
            active_web_view: state.latest_active_web_view,
            sidebar_visible: state.latest_sidebar_visible,
            chrome_focused: state.latest_chrome_focused,
            command_bar_mode: state.latest_command_bar_mode,
            content_rect: [rect.x, rect.y, rect.width, rect.height],
        };
        if let Some(sink) = state.sink.as_mut() {
            sink(measurement);
        } else {
            state.queued.push_back(measurement);
        }
    }
}

#[cfg(target_os = "macos")]
fn monotonic_now_ns() -> u64 {
    let mut info = mach2::mach_time::mach_timebase_info_data_t { numer: 0, denom: 0 };
    // SAFETY: mach_timebase_info initializes the provided plain data struct;
    // mach_absolute_time has no preconditions and returns the host monotonic clock.
    unsafe {
        mach2::mach_time::mach_timebase_info(&mut info);
        let ticks = mach2::mach_time::mach_absolute_time() as u128;
        let nanos = ticks * u128::from(info.numer) / u128::from(info.denom.max(1));
        nanos.min(u64::MAX as u128) as u64
    }
}

#[cfg(not(target_os = "macos"))]
fn monotonic_now_ns() -> u64 {
    0
}
