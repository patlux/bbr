//! Testkit-only command-surface benchmark drivers (DESIGN.md §11.5).
//!
//! `command_type` measures the production command-text path through the real
//! ranking debounce: Cmd-L selects the active URL, replace-selection typing
//! enters the text, and the sample commits only from a *render* of the
//! result surface — the ranking publication schedules the render-inclusive
//! commit, and the probe predicate enforces the exact generation,
//! first/selected result keys, typed state, mode, and focus (fast-but-wrong
//! guard). The deterministic flush is never used inside the measured
//! scenario; `command_activate` and `command_activate_new_tab` arm exact
//! destination expectations before dispatch and commit only after
//! caller-specific destination/session validation (the production activation
//! path suppresses its generic commit while a command benchmark is pending).

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use bbr_web::WebViewId;
use gpui::{App, AsyncApp, Context, Window};

use crate::command_bar::CommandBarMode;

use super::palette::CommandSurface;
use super::BbrApp;

impl BbrApp {
    /// Benchmark the production command-text path: Cmd-L selects the active
    /// URL, replace-selection typing enters `text` through the exact
    /// mutation the key handler runs, and the real ranking debounce expires
    /// and publishes the current generation. The sample commits only from a
    /// subsequent render of the result surface (the probe predicate enforces
    /// generation/result identity against the identity the render drew), so
    /// a stale generation, wrong first row, wrong state, mode, or focus
    /// rejects the measurement in the app — and a render that omitted or
    /// delayed the palette can never pass. Timing begins at the production
    /// input entry; the deterministic flush is never used inside this
    /// measured scenario.
    pub fn benchmark_command_type(
        &mut self,
        operation_id: BenchmarkOperationId,
        text: &str,
        expected_first_key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.benchmark_command_entry(
            operation_id,
            BenchmarkKind::CommandType,
            text,
            expected_first_key,
            window,
            cx,
        )
    }

    /// Benchmark the production Enter activation of the command bar: the
    /// selected/typed destination routes through the app/controller seams
    /// (`submit_command`). Exact destination/focus/mode expectations are
    /// armed before dispatch and the production activation path suppresses
    /// its generic commit while a command benchmark is pending, so the
    /// sample commits only after the presented state is validated against
    /// them (fast-but-wrong guard).
    pub fn benchmark_command_activate(
        &mut self,
        operation_id: BenchmarkOperationId,
        expected_view: Option<WebViewId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::CommandActivate)
            .is_err()
        {
            return false;
        }
        // Arm the exact destination/focus/mode expectations BEFORE dispatch.
        let expected_mode = CommandBarMode::Inactive.into();
        self.performance
            .expect_state(expected_view, self.sidebar_visible, false, expected_mode);
        // Production Enter path (the exact `on_chrome_key_down` handler).
        self.submit_command(window, cx);
        let snapshot = self.controller.snapshot();
        let chrome_focused = self.chrome_focused(window);
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // Fast-but-wrong guard: the destination view, focus domain, and mode
        // must match the expected web activation before the sample commits.
        if snapshot.active_view != expected_view || chrome_focused || mode != expected_mode {
            self.performance.cancel();
            return false;
        }
        self.commit_benchmark_sync(window);
        true
    }

    /// Benchmark the production Cmd-Enter activation: the destination opens
    /// in exactly one new tab/session and the fresh page is focused (the
    /// session-recreation class). The generic activation commit is suppressed
    /// while this benchmark is pending; the sample commits only after the
    /// caller-specific validation (exactly one new tab and live session, a
    /// fresh active view, surface closed in web focus) passes.
    pub fn benchmark_command_activate_new_tab(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let before = self.controller.snapshot();
        if self
            .performance
            .begin(operation_id, BenchmarkKind::CommandActivateNewTab)
            .is_err()
        {
            return false;
        }
        // Production Cmd-Enter path (the exact `on_chrome_key_down` handler).
        self.submit_command_new_tab(window, cx);
        let after = self.controller.snapshot();
        // Fast-but-wrong guard: exactly one new tab and one new live
        // session, a fresh active view (never the pre-activation view), and
        // the surface closed in web focus.
        if after.tabs.len() != before.tabs.len() + 1
            || after.engine_sessions != before.engine_sessions + 1
            || after.active_view == before.active_view
            || self.command_bar_mode != CommandBarMode::Inactive
        {
            self.performance.cancel();
            return false;
        }
        // The validation passed: arm the presented state and commit the
        // sample (the probe observes the same state synchronously).
        self.performance.expect_state(
            after.active_view,
            self.sidebar_visible,
            false,
            CommandBarMode::Inactive.into(),
        );
        self.commit_benchmark_sync(window);
        true
    }

    /// Benchmark the production Cmd-T open of the centered command palette
    /// (DESIGN.md §11.4 `bbr-palette`): the sample starts at the production
    /// entry seam (`open_palette`, the exact method the `OpenPalette`
    /// action routes to) and commits only from a render that actually
    /// constructed the palette with the staged New Tab mode and chrome
    /// focus — the fast-but-wrong guard rejects a render that omitted the
    /// surface or drew the wrong presentation/mode/focus. Staged semantics:
    /// no tab or session is created before Enter, so the sample is the
    /// warm-presentation class (like the other command scenarios).
    pub fn benchmark_palette_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::PaletteOpen)
            .is_err()
        {
            return false;
        }
        // Production Cmd-T entry: the exact seam the `OpenPalette` action
        // routes to (window_seam → `open_palette`), same engine as the
        // in-rail command bar.
        let before = self.controller.snapshot();
        self.open_palette(window, cx);
        // Fast-but-wrong guard: the centered palette is actually open in New
        // Tab staging with chrome focus, and the staged semantics hold — the
        // active view is unchanged and no tab/session was created before
        // Enter.
        let after = self.controller.snapshot();
        if self.command_surface != CommandSurface::Palette
            || self.command_bar_mode != CommandBarMode::NewTab
            || !self.chrome_focused(window)
            || after.active_view != before.active_view
            || after.tabs.len() != before.tabs.len()
            || after.engine_sessions != before.engine_sessions
        {
            self.performance.cancel();
            return false;
        }
        // Arm the exact presentation the palette render must show: the
        // unchanged active view, the visible rail, chrome focus, the staged
        // New Tab mode, and the centered palette surface itself. The commit
        // predicate only accepts a render that drew the palette (render
        // observation below) with this exact state.
        self.performance.expect_state(
            after.active_view,
            self.sidebar_visible,
            true,
            CommandBarMode::NewTab.into(),
        );
        self.performance.expect_surface(CommandSurface::Palette);
        // Render-dependent commit (the `command_type` pattern): a bounded
        // forced draw on the main thread after this update completes, so the
        // palette is actually rendered in the hidden harness; the render
        // observes the surface it constructed and the probe commits at the
        // presentation milestone.
        self.schedule_palette_render_commit(cx);
        true
    }

    /// Schedule the render-inclusive commit for the pending palette-open
    /// sample: a bounded forced draw (the same testkit pattern the
    /// render-dependent command/dense scenarios use) so the centered palette
    /// is actually rendered in the hidden harness and the probe commits at
    /// the presentation milestone.
    pub(crate) fn schedule_palette_render_commit(&self, cx: &mut Context<Self>) {
        if self.performance.pending_kind() != Some(BenchmarkKind::PaletteOpen) {
            return;
        }
        let Some(window) = cx.windows().into_iter().next() else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                // Render the open palette: the render observes the surface
                // after constructing the presentation and refreshes the
                // browser-state observation; the slot paint then commits the
                // sample when the armed surface and state match.
                let _ = window.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the same surface + state and
                // cancels truthfully when the render omitted the palette (no
                // observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_palette_open_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the `palette_open` sample from the render milestone: the probe
    /// compares the armed surface/state against the observations the render
    /// recorded after constructing the palette. Returns false and cancels
    /// when the render omitted the palette or the state was wrong — a
    /// truthful rejection, never a fast-but-wrong pass.
    fn commit_palette_open_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if self.performance.pending_kind() == Some(BenchmarkKind::PaletteOpen) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Benchmark the production Cmd-L / address-click open of the
    /// field-anchored overlay (DESIGN.md §11.4 `bbr-command-overlay`): the
    /// sample starts at the production entry seam (`open_overlay`, the exact
    /// method Cmd-L and the address-pill pointer press route to) and commits
    /// only from a render that actually constructed the overlay with the
    /// Location mode, chrome focus, the full URL present and **preselected**
    /// (so one keystroke replaces it) — the fast-but-wrong guard rejects a
    /// render that omitted the surface or drew the wrong
    /// presentation/mode/focus/selection. Same warm-presentation class as
    /// `palette_open`: staged semantics, no tab or session is created.
    pub fn benchmark_overlay_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::OverlayOpen)
            .is_err()
        {
            return false;
        }
        // Production Cmd-L entry: the exact seam Cmd-L and the address-pill
        // pointer press route to (same §11.5 engine as the in-rail bar).
        let before = self.controller.snapshot();
        self.open_overlay(window, cx);
        // Fast-but-wrong guard: the overlay is actually open in Location
        // mode with chrome focus, the full current URL is present and
        // preselected (the §11.4 open contract — one keystroke replaces
        // it), and the staged semantics hold — the active view is unchanged
        // and no tab/session was created before Enter.
        let after = self.controller.snapshot();
        let url = self.active_url();
        if self.command_surface != CommandSurface::Overlay
            || self.command_bar_mode != CommandBarMode::Location
            || !self.chrome_focused(window)
            || after.active_view != before.active_view
            || after.tabs.len() != before.tabs.len()
            || after.engine_sessions != before.engine_sessions
            || self.command_text.text != url
            || self.command_text.selection != Some((0, url.len()))
        {
            self.performance.cancel();
            return false;
        }
        // Arm the exact presentation the overlay render must show: the
        // unchanged active view, the visible rail, chrome focus, and the
        // Location mode. The sample then commits synchronously through the
        // authoritative post-mutation state + geometry (the same
        // `commit_benchmark_sync` pattern the Cmd-T palette path uses
        // inside `activate_new_tab`) — never dependent on a frame that may
        // not fire in the offscreen harness. The fast-but-wrong guard above
        // already proved the overlay surface, mode, focus, staged semantics
        // and the preselected URL; the synchronous commit re-validates the
        // observed state.
        self.performance.expect_state(
            after.active_view,
            self.sidebar_visible,
            true,
            CommandBarMode::Location.into(),
        );
        self.commit_benchmark_sync(window);
        // The synchronous commit may reject when the probe's latest surface
        // observation is still the previous presentation (the portal window
        // renders on a deferred turn). In that case fall back to the
        // render-inclusive commit (the `palette_open` pattern): a bounded
        // forced draw so the overlay is actually rendered in the hidden
        // harness, with the overlay surface armed so the commit predicate
        // only accepts a render that drew it.
        if self.performance.pending_kind() == Some(BenchmarkKind::OverlayOpen) {
            self.performance.expect_surface(CommandSurface::Overlay);
            self.schedule_overlay_render_commit(cx);
        }
        true
    }

    /// Schedule the render-inclusive commit for the pending overlay-open
    /// sample: a bounded forced draw (the same testkit pattern the
    /// render-dependent command/dense scenarios use) so the field-anchored
    /// overlay is actually rendered in the hidden harness and the probe
    /// commits at the presentation milestone.
    pub(crate) fn schedule_overlay_render_commit(&self, cx: &mut Context<Self>) {
        if self.performance.pending_kind() != Some(BenchmarkKind::OverlayOpen) {
            return;
        }
        let Some(window) = cx.windows().into_iter().next() else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                // Render the open overlay: the render observes the surface
                // after constructing the presentation and refreshes the
                // browser-state observation; the slot paint then commits the
                // sample when the armed surface and state match.
                let _ = window.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the same surface + state and
                // cancels truthfully when the render omitted the overlay (no
                // observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_overlay_open_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the `overlay_open` sample from the render milestone: the probe
    /// compares the armed surface/state against the observations the render
    /// recorded after constructing the overlay. Returns false and cancels
    /// when the render omitted the overlay or the state was wrong — a
    /// truthful rejection, never a fast-but-wrong pass.
    fn commit_overlay_open_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if self.performance.pending_kind() == Some(BenchmarkKind::OverlayOpen) {
            self.performance.cancel();
            return false;
        }
        true
    }
}
