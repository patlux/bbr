//! Testkit-only surface of the root view: benchmark drivers, E2E mutation
//! seams, and the semantic snapshot.
//!
//! Only compiled under the `testkit` cargo feature. Production behavior
//! (open/select/close, focus, sidebar) stays in `app.rs`; this module holds
//! the typed benchmark drivers the performance harness drives and the
//! mutation seams the E2E gateway dispatches. The benchmark methods mirror
//! the production paths (same controller mutations and focus domains) and
//! commit synchronously from the authoritative post-mutation snapshot so a
//! fast-but-wrong sample cannot be accepted.

use bbr_browser::{ControllerError, ControllerEvent, SpaceId};
use bbr_web::{ProfileId, WebAction, WebViewId};
use gpui::{Context, Window};

use crate::shell::TestSnapshot;

use super::{BbrApp, BrowserSink, FocusSink};

impl BbrApp {
    /// Create a profile with its first space (E2E seam; production uses the
    /// startup profile).
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

    /// Create a space in an existing profile (E2E seam).
    pub fn create_space(
        &mut self,
        profile: ProfileId,
        title: &str,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.create_space_event(profile, title);
        self.apply_mutation(result, cx)
    }

    /// Benchmark the warm tab-selection path (production `select_tab`).
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

    /// Benchmark the production pointer-selection path (row mouse-down).
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

    /// Benchmark the reopen path (production `reopen_tab`).
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

    /// Move a tab to another space (E2E seam; follows the profile-boundary
    /// recreation policy inside the controller).
    pub fn move_tab(
        &mut self,
        index: usize,
        to_space: SpaceId,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self.controller.move_tab(index, to_space);
        self.apply_mutation(result, cx)
    }

    /// Suspend a tab's native session (E2E seam).
    pub fn suspend_tab(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let result = self.controller.suspend_tab(index);
        self.apply_mutation(result, cx)
    }

    /// Route a generic [`WebAction`] through the focused controller session.
    pub fn web_action(&mut self, action: WebAction) -> Option<ControllerEvent> {
        self.controller
            .navigate(action)
            .map_err(|error| {
                log::error!("could not route web action: {error}");
                error
            })
            .ok()
    }

    /// Install a focus-change sink (used by the E2E gateway).
    pub fn set_focus_sink(&mut self, sink: FocusSink) {
        *self.focus_sink.borrow_mut() = Some(sink);
    }

    /// Install a semantic browser-mutation sink (used by the E2E gateway).
    pub fn set_browser_sink(&mut self, sink: BrowserSink) {
        *self.browser_sink.borrow_mut() = Some(sink);
    }

    /// Arm the deterministic page-ready milestone for a view/title pair.
    pub fn benchmark_page_ready(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        expected_view: WebViewId,
        expected_title: impl Into<String>,
    ) -> bool {
        self.performance
            .arm_page_ready(operation_id, expected_view, expected_title);
        true
    }

    /// Begin a benchmark operation of the given kind.
    pub fn begin_benchmark(
        &self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        kind: bbr_testkit::protocol::BenchmarkKind,
    ) -> bool {
        self.performance.begin(operation_id, kind).is_ok()
    }

    /// Cancel the in-flight benchmark operation, if any.
    pub fn cancel_benchmark(&self) {
        self.performance.cancel();
    }

    /// Commit the pending benchmark synchronously from authoritative state.
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

    /// Apply a controller mutation result: log errors, broadcast the outcome,
    /// and notify the view. Returns the event when the mutation succeeded.
    fn apply_mutation(
        &mut self,
        result: Result<ControllerEvent, ControllerError>,
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

    /// Benchmark the sidebar toggle through the production path.
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
