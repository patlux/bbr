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

use crate::app::palette::CommandSurface;

pub mod command;
pub(crate) mod downloads;
mod error_surface;
mod find;
pub mod pane_geometry;
pub mod slot;
mod u2;
use command::{CommandExpectation, CommandObservation};
use error_surface::ErrorSurfaceObservation;
use pane_geometry::PaneGeometry;

pub type PresentationSink = Box<dyn FnMut(PresentationMeasurement)>;

#[derive(Clone)]
pub struct PerformanceProbe {
    inner: Rc<RefCell<State>>,
    /// Keyed per-pane native geometry (testkit-only split seam); see
    /// [`pane_geometry`].
    pane_geometry: PaneGeometry,
}

struct State {
    process_started: Instant,
    /// Testkit-only deterministic delay injected into the measured path. This
    /// must prove that budget gates catch real latency; it is armed only from
    /// a `testkit`-gated CLI flag and is impossible in release-without-testkit.
    inject_delay: Option<Duration>,
    /// The latest published command-result presentation (generation, first/
    /// selected result keys, typed state) observed at the ranking publish;
    /// the command benchmarks commit only against it.
    latest_command: Option<CommandObservation>,
    /// The find-bar presentation the portal render actually drew; the find
    /// benchmarks commit only against it (render-gated, see `find`).
    latest_find: Option<find::FindObservation>,
    /// The downloads-panel presentation the sidebar render actually drew;
    /// the downloads benchmarks commit only against it (render-gated, see
    /// `downloads`).
    latest_downloads: Option<downloads::DownloadsObservation>,
    /// The error-surface presentation the content-area render actually drew;
    /// the error-surface benchmark commits only against it (render-gated,
    /// see `error_surface`).
    latest_error_surface: ErrorSurfaceObservation,
    latest_active_web_view: Option<WebViewId>,
    latest_active_space: TestSpaceId,
    latest_sidebar_visible: bool,
    latest_chrome_focused: bool,
    latest_command_bar_mode: CommandBarMode,
    latest_surface: CommandSurface,
    /// The §11.3 cross-context command-field hint of the latest observed
    /// render (None when absent); the space-switch benchmark commits only
    /// when the profile-hint state matches the activation.
    latest_profile_hint: Option<String>,
    /// U2 rail state observed by the plus-menu/favorites/folder benchmarks.
    u2: u2::U2Observed,
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
    expected_surface: Option<CommandSurface>,
    /// The §11.3 profile-hint state the presentation must show (None = no
    /// hint; Some(title) = the hint with that copy). A cross-profile space
    /// activation must present the hint and a same-profile one must not, or
    /// the sample is rejected in the app.
    expected_profile_hint: Option<Option<String>>,
    /// U2 rail state the pending sample must present.
    u2: u2::U2Expected,
    /// The exact command-result presentation identity the pending sample must
    /// commit from: generation, first/selected result key, typed state. Only
    /// the publish that carries this identity can satisfy the commit.
    command: Option<CommandExpectation>,
    /// The exact find-bar presentation identity the pending sample must
    /// commit from (open/focused, or the WebKit-reported report).
    find: Option<find::FindExpectation>,
    /// The exact downloads-panel presentation identity the pending sample
    /// must commit from (open/focused, an armed progress fraction on the
    /// running row, or the fresh retry row in Active).
    downloads: Option<downloads::DownloadsExpectation>,
    /// The exact error-surface presentation identity the pending sample must
    /// commit from (the surface drawn for the armed closed category).
    error_surface: Option<error_surface::ErrorSurfaceExpectation>,
}

impl PerformanceProbe {
    pub fn new(process_started: Instant, inject_delay: Option<Duration>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(State {
                process_started,
                inject_delay,
                latest_command: None,
                latest_find: None,
                latest_downloads: None,
                latest_error_surface: error_surface::ErrorSurfaceObservation::default(),
                latest_active_web_view: None,
                latest_active_space: TestSpaceId(0),
                latest_sidebar_visible: true,
                latest_chrome_focused: false,
                latest_command_bar_mode: CommandBarMode::Inactive,
                latest_surface: CommandSurface::None,
                latest_profile_hint: None,
                u2: u2::U2Observed::default(),
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
                    expected_surface: None,
                    expected_profile_hint: None,
                    u2: u2::U2Expected::default(),
                    command: None,
                    find: None,
                    downloads: None,
                    error_surface: None,
                }),
                page_ready: None,
                seen_titles: BTreeMap::new(),
                queued: VecDeque::new(),
                sink: None,
            })),
            pane_geometry: PaneGeometry::default(),
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
            expected_surface: None,
            expected_profile_hint: None,
            u2: u2::U2Expected::default(),
            command: None,
            find: None,
            downloads: None,
            error_surface: None,
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

    /// Record the §11.3 profile-hint state observed at the mutation site, so
    /// the space-switch benchmark can commit only when the presented
    /// command-field hint matches the activation (fast-but-wrong guard).
    pub fn observe_profile_hint(&self, hint: Option<String>) {
        self.inner.borrow_mut().latest_profile_hint = hint;
    }

    /// Require the pending presentation to show `hint` (or no hint when
    /// `None`) as the §11.3 cross-context command-field transient.
    pub fn expect_profile_hint(&self, hint: Option<String>) {
        if let Some(pending) = self.inner.borrow_mut().pending.as_mut() {
            pending.expected_profile_hint = Some(hint);
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

    /// The keyed per-pane geometry observed since the last prune.
    pub fn pane_rects(&self) -> BTreeMap<WebViewId, WebRect> {
        self.pane_geometry.snapshot()
    }

    /// Drop keyed geometry for every view outside the current pane layout,
    /// so destroyed/hidden panes leave no orphan geometry evidence.
    pub fn prune_pane_rects(&self, keep: &[WebViewId]) {
        self.pane_geometry.prune(keep);
    }

    /// A callback that observes every pane slot rect keyed by view id
    /// (testkit-only keyed geometry observation for the split seam).
    pub fn keyed_geometry_callback(&self) -> Rc<dyn Fn(WebViewId, WebRect)> {
        self.pane_geometry.callback()
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
            plus_menu_open: state.u2.plus_menu_open,
            favorites_len: state.u2.favorites_len,
            folder_expanded: state.u2.folder_expanded,
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
        #[cfg(feature = "testkit")]
        if !pending.armed {
            log::warn!(
                "probe reject op{:?} {:?}: not armed",
                pending.operation_id,
                pending.kind
            );
        }
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
            || pending
                .expected_surface
                .is_some_and(|expected| expected != state.latest_surface)
            || pending
                .expected_profile_hint
                .as_ref()
                .is_some_and(|expected| expected != &state.latest_profile_hint)
            || pending.u2.mismatches(&state.u2)
            || !command::command_matches(pending, &state)
            || !find::find_matches(pending, &state)
            || !downloads::downloads_matches(pending, &state)
            || !error_surface::error_surface_matches(pending, &state)
        {
            #[cfg(feature = "testkit")]
            if let Some(_d) = pending.downloads.as_ref() {
                log::warn!(
                    "probe reject op{:?} {:?}: view={:?}/{:?} space={:?}/{:?} sidebar={:?}/{:?} chrome={:?}/{:?} mode={:?}/{:?} downloads_obs={:?}",
                    pending.operation_id,
                    pending.kind,
                    pending.expected_active_web_view,
                    state.latest_active_web_view,
                    pending.expected_active_space,
                    state.latest_active_space,
                    pending.expected_sidebar_visible,
                    state.latest_sidebar_visible,
                    pending.expected_chrome_focused,
                    state.latest_chrome_focused,
                    pending.expected_command_bar_mode,
                    state.latest_command_bar_mode,
                    state.latest_downloads,
                );
            }
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
            plus_menu_open: state.u2.plus_menu_open,
            favorites_len: state.u2.favorites_len,
            folder_expanded: state.u2.folder_expanded,
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
mod downloads_tests;
#[cfg(test)]
mod find_tests;
#[cfg(test)]
mod tests;
