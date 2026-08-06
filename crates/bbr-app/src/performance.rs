//! Testkit-only presentation timing for real-app performance E2E.
//!
//! Operations start inside the app process immediately before the production
//! mutation. A sample completes at the end of the GPUI paint path after native
//! slot geometry is attached, immediately before GPUI presents the frame; socket
//! transport and harness scheduling are excluded.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{
    BenchmarkKind, BenchmarkOperationId, CommandBarMode, PresentationMeasurement, TestSpaceId,
};
use bbr_web::{WebEvent, WebRect, WebViewId};

pub type PresentationSink = Box<dyn FnMut(PresentationMeasurement)>;

#[derive(Clone)]
pub struct PerformanceProbe {
    inner: Rc<RefCell<State>>,
}

struct State {
    process_started: Instant,
    /// Testkit-only deterministic delay injected into the measured path. This
    /// must prove that budget gates catch real latency; it is armed only from
    /// a `testkit`-gated CLI flag and is impossible in release-without-testkit.
    inject_delay: Option<Duration>,
    latest_active_web_view: Option<WebViewId>,
    latest_active_space: TestSpaceId,
    latest_sidebar_visible: bool,
    latest_chrome_focused: bool,
    latest_command_bar_mode: CommandBarMode,
    content_rect: Option<WebRect>,
    pending: Option<Pending>,
    page_ready: Option<PageReady>,
    /// Last fixture title observed per view (drives deterministic page-ready).
    seen_titles: BTreeMap<WebViewId, String>,
    queued: VecDeque<PresentationMeasurement>,
    sink: Option<PresentationSink>,
}

/// A deterministic-page-ready measurement pending the expected fixture title.
struct PageReady {
    operation_id: BenchmarkOperationId,
    /// Start reference is process spawn so the latency is comparable to
    /// [`BenchmarkKind::Startup`] shell presentation.
    started_at: Instant,
    expected_view: WebViewId,
    expected_title: String,
}

struct Pending {
    operation_id: BenchmarkOperationId,
    kind: BenchmarkKind,
    started_at: Instant,
    armed: bool,
    expected_active_web_view: Option<Option<WebViewId>>,
    expected_active_space: Option<TestSpaceId>,
    expected_sidebar_visible: Option<bool>,
    /// The sidebar width (content-rect x) the presentation must show; the
    /// geometry check rejects fast-but-wrong resize/restore samples.
    expected_sidebar_width: Option<f32>,
    expected_chrome_focused: Option<bool>,
    expected_command_bar_mode: Option<CommandBarMode>,
}

impl PerformanceProbe {
    pub fn new(process_started: Instant, inject_delay: Option<Duration>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(State {
                process_started,
                inject_delay,
                latest_active_web_view: None,
                latest_active_space: TestSpaceId(0),
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
                    expected_active_space: None,
                    expected_sidebar_visible: Some(true),
                    expected_sidebar_width: None,
                    expected_chrome_focused: None,
                    expected_command_bar_mode: None,
                }),
                page_ready: None,
                seen_titles: BTreeMap::new(),
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
            expected_active_space: None,
            expected_sidebar_visible: None,
            expected_sidebar_width: None,
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

    /// Record the active space observed at the mutation site; space-switch
    /// samples commit only when the destination space is actually presented.
    pub fn observe_active_space(&self, space: TestSpaceId) {
        self.inner.borrow_mut().latest_active_space = space;
    }

    /// Require the pending presentation to show `space` (semantic check for
    /// [`BenchmarkKind::WarmSpaceSwitch`]).
    pub fn expect_space(&self, space: TestSpaceId) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.expected_active_space = Some(space);
        }
    }

    /// Require the pending presentation to show `width` as the sidebar width
    /// (the content-rect x); used by resize/restore samples so a wrong
    /// geometry is rejected even when the visibility flag is right.
    pub fn expect_sidebar_width(&self, width: f32) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.expected_sidebar_width = Some(width);
        }
    }

    /// Observe the rect the native slot actually handed to the engine on the
    /// last paint (the WKWebView frame), for the semantic snapshot.
    pub fn observe_slot_rect(&self, rect: WebRect) {
        self.inner.borrow_mut().content_rect = Some(rect);
    }

    /// The last engine-received native content rect, as `[x, y, w, h]`.
    pub fn content_rect(&self) -> Option<[f32; 4]> {
        self.inner
            .borrow()
            .content_rect
            .map(|r| [r.x, r.y, r.width, r.height])
    }

    /// A callback that observes every slot rect (testkit-only geometry
    /// observation, independent of any pending measurement).
    pub fn slot_geometry_callback(&self) -> Rc<dyn Fn(WebRect)> {
        let probe = self.clone();
        Rc::new(move |rect| probe.observe_slot_rect(rect))
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

    /// Arm a deterministic-page-ready measurement. Commits immediately when
    /// the expected fixture title is already observed (the harness often arms
    /// after the deterministic page finished loading), otherwise it commits on
    /// the next matching title event. Start reference is process spawn so it is
    /// comparable to the startup milestone. Page readiness is a genuine content
    /// signal: the expected fixture title must actually have been observed, not
    /// merely that navigation reported loading-finished.
    pub fn arm_page_ready(
        &self,
        operation_id: BenchmarkOperationId,
        expected_view: WebViewId,
        expected_title: impl Into<String>,
    ) {
        let mut state = self.inner.borrow_mut();
        state.page_ready = Some(PageReady {
            operation_id,
            started_at: state.process_started,
            expected_view,
            expected_title: expected_title.into(),
        });
        Self::try_commit_page_ready(&mut state);
    }

    /// Observe a native engine event. Under testkit, records the deterministic
    /// fixture title and drives the page-ready milestone.
    pub fn note_web_event(&self, event: &WebEvent) {
        let mut state = self.inner.borrow_mut();
        if let WebEvent::TitleChanged(view, title) = event {
            state.seen_titles.insert(*view, title.clone());
        }
        Self::try_commit_page_ready(&mut state);
    }

    /// Commit a pending page-ready measurement once the expected deterministic
    /// fixture title has actually been observed on the expected view. Falling
    /// back on a loading-finished signal would record the expectation, not the
    /// observation, and turn the content guard into a shell-only check.
    fn try_commit_page_ready(state: &mut State) {
        let Some(ready) = state.page_ready.as_ref() else {
            return;
        };
        let title_ok = state.seen_titles.get(&ready.expected_view) == Some(&ready.expected_title);
        if !title_ok {
            return;
        }
        let ready = state.page_ready.take().expect("checked just above");
        let duration_ns = elapsed_ns(ready.started_at);
        let since_process_start_ns = elapsed_ns(state.process_started);
        let measurement = PresentationMeasurement {
            operation_id: ready.operation_id,
            kind: BenchmarkKind::PageReady,
            duration_ns,
            since_process_start_ns,
            monotonic_commit_ns: monotonic_now_ns(),
            active_web_view: state.latest_active_web_view,
            active_space: state.latest_active_space,
            sidebar_visible: state.latest_sidebar_visible,
            chrome_focused: state.latest_chrome_focused,
            command_bar_mode: state.latest_command_bar_mode,
            content_rect: state
                .content_rect
                .map_or([0.0; 4], |r| [r.x, r.y, r.width, r.height]),
            page_title: Some(ready.expected_title),
        };
        if let Some(sink) = state.sink.as_mut() {
            sink(measurement);
        } else {
            state.queued.push_back(measurement);
        }
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

    /// Commit the pending presentation from authoritative geometry instead of
    /// the last painted rect: the interaction benchmarks run synchronously
    /// after the mutation, before the next paint, so the stored rect would be
    /// the stale pre-mutation geometry. The computed rect (sidebar rail on
    /// the left, native content filling the remainder) is what the next
    /// layout hands the slot, and the semantic width check still rejects
    /// fast-but-wrong samples.
    pub fn commit_geometry(&self, window_width: f32, sidebar_width: f32, window_height: f32) {
        let rect = WebRect::new(
            sidebar_width,
            0.0,
            (window_width - sidebar_width).max(0.0),
            window_height,
        );
        self.commit_presentation(rect);
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
                .expected_active_space
                .is_some_and(|expected| expected != state.latest_active_space)
            || pending
                .expected_sidebar_visible
                .is_some_and(|expected| expected != state.latest_sidebar_visible)
            || pending
                .expected_sidebar_width
                .is_some_and(|expected| (expected - rect.x).abs() > 0.5)
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
        // Testkit-only injection: block the measured production path before
        // sampling so budget/self tests prove the gate detects real latency.
        // Applied only to interaction kinds (not Startup) so a delayed machine
        // does not fail the startup ceiling for the wrong reason; page-ready
        // already runs on a separate non-injected path, so this keeps startup
        // and page-ready measured with a consistent clock.
        if let Some(delay) = state.inject_delay {
            if pending.kind != BenchmarkKind::Startup {
                std::thread::sleep(delay);
            }
        }
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
            active_space: state.latest_active_space,
            sidebar_visible: state.latest_sidebar_visible,
            chrome_focused: state.latest_chrome_focused,
            command_bar_mode: state.latest_command_bar_mode,
            content_rect: [rect.x, rect.y, rect.width, rect.height],
            page_title: None,
        };
        if let Some(sink) = state.sink.as_mut() {
            sink(measurement);
        } else {
            state.queued.push_back(measurement);
        }
    }
}

fn elapsed_ns(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u64::MAX as u128) as u64
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

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_testkit::protocol::BenchmarkOperationId;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    fn probe_with_sink() -> (PerformanceProbe, Rc<RefCell<Vec<PresentationMeasurement>>>) {
        let probe = PerformanceProbe::new(Instant::now(), None);
        let collected = Rc::new(RefCell::new(Vec::new()));
        let sink_collected = collected.clone();
        probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
        (probe, collected)
    }

    #[test]
    fn page_ready_emits_only_on_expected_view_and_title() {
        let (probe, collected) = probe_with_sink();
        probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
        probe.arm_page_ready(BenchmarkOperationId(0), WebViewId(1), "Fixture");
        // Wrong title must not commit.
        probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Other".into()));
        assert_eq!(collected.borrow().len(), 0);
        // Wrong view must not commit.
        probe.note_web_event(&WebEvent::TitleChanged(WebViewId(2), "Fixture".into()));
        assert_eq!(collected.borrow().len(), 0);
        // Correct title + view commits a single PageReady sample.
        probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Fixture".into()));
        let samples = collected.borrow();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].kind, BenchmarkKind::PageReady);
        assert_eq!(samples[0].page_title.as_deref(), Some("Fixture"));
        // A later unrelated event must not emit a duplicate.
        probe.note_web_event(&WebEvent::LoadingChanged(WebViewId(1), false));
        assert_eq!(samples.len(), 1);
        drop(samples);
    }

    #[test]
    fn page_ready_requires_observed_title_not_idle_signal() {
        // L4 regression: a loading-finished (idle) signal with no matching
        // title must NOT commit page-ready, otherwise the milestone records the
        // expectation instead of genuine fixture-content evidence.
        let (probe, collected) = probe_with_sink();
        probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
        probe.arm_page_ready(BenchmarkOperationId(0), WebViewId(1), "Fixture");
        probe.note_web_event(&WebEvent::LoadingChanged(WebViewId(1), false));
        assert_eq!(
            collected.borrow().len(),
            0,
            "idle-only must not satisfy the page-ready content guard"
        );
        // The observed title is what commits it.
        probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Fixture".into()));
        assert_eq!(collected.borrow().len(), 1);
    }

    #[test]
    fn startup_is_not_injected() {
        // L5: injection targets interaction latency, not the cold-startup
        // ceiling, so a delayed self-test run still yields an honest startup
        // sample that shares the page-ready (uninjected) clock.
        let probe = PerformanceProbe::new(Instant::now(), Some(Duration::from_millis(40)));
        let collected = Rc::new(RefCell::new(Vec::new()));
        let sink_collected = collected.clone();
        probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
        probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
        probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
        assert_eq!(
            collected.borrow().len(),
            1,
            "initial startup presentation must commit"
        );
        let sample = collected.borrow()[0].clone();
        assert_eq!(sample.kind, BenchmarkKind::Startup);
        assert!(
            sample.duration_ns < 40_000_000,
            "startup must not absorb the injected delay"
        );
    }

    #[test]
    fn injected_delay_inflates_measured_duration() {
        let probe = PerformanceProbe::new(Instant::now(), Some(Duration::from_millis(40)));
        let collected = Rc::new(RefCell::new(Vec::new()));
        let sink_collected = collected.clone();
        probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
        probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
        // A fresh probe starts with an armed startup pending; clear it so the
        // interaction measurement below is the only one collected.
        probe.cancel();
        probe
            .begin(BenchmarkOperationId(1), BenchmarkKind::WarmTabSwitch)
            .unwrap();
        probe.expect_browser_state(Some(WebViewId(1)), true);
        probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
        let samples = collected.borrow();
        let sample = samples.first().expect("one sample");
        assert_eq!(sample.kind, BenchmarkKind::WarmTabSwitch);
        assert!(
            sample.duration_ns >= 40_000_000,
            "injected delay must be reflected in measured work"
        );
    }
}
