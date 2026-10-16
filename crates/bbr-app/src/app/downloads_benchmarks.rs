//! Testkit-only downloads-panel benchmark drivers (task packet
//! `downloads-sidebar-panel`, DESIGN.md §11.12/§15).
//!
//! `benchmark_downloads_panel_open` measures the production footer
//! `[Downloads]` open (`toggle_downloads` → `downloads_open`): the sample
//! commits only from a render of the anchored panel that drew it open with
//! the keyboard cursor over the unchanged active view (fast-but-wrong
//! guard — a render that omitted the panel or changed the view/session can
//! never commit). `benchmark_downloads_progress_commit` measures the
//! production L6 event path: a deterministic [`DownloadEvent::Progress`] is
//! applied through the same main-thread pump the native adapter's sink uses
//! (`on_download_event`), and the sample commits only when the panel render
//! drew the running row at the armed fraction.
//! `benchmark_downloads_row_retry` measures the production retry seam
//! (`controller.retry`): the fresh attempt must be drawn in the Active
//! group, linked via `retry_of`, with the Cancel action.
//!
//! Fixture/network time is excluded by construction: no network transfer is
//! involved in any measured path. The progress/retry attempts are seeded
//! through the portable controller with an empty web-view slot (the action
//! sink dispatch fails as a typed `NoWebView` and no `WKDownload` ever
//! starts), so the measured duration is the main-thread event application +
//! controller validation + panel re-render only. The real `WKDownload`
//! lifecycle is proven by the functional downloads E2E and the adapter
//! contract tests.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use bbr_web::downloads::{DestinationDecision, DownloadEvent, DownloadId, DownloadProgress};
use gpui::{Context, Window};

use super::downloads::row_visuals;
use super::downloads_behavior::{downloads_open, on_download_event};
use super::BbrApp;
use bbr_ui::surfaces::downloads::{DownloadActionKind, DownloadMachine};

impl BbrApp {
    /// Benchmark the production footer `[Downloads]` open: the sample starts
    /// at the exact seam the footer control's activation routes to
    /// (`toggle_downloads` → `downloads_open`) and commits only from a
    /// render that drew the anchored panel open with the keyboard cursor.
    /// The active view/sidebar stay unchanged and the panel owns chrome
    /// focus; the armed expectations reject any render that omitted the
    /// panel or touched the session. The panel must start closed (the
    /// driver closes it between samples with an unmeasured Escape).
    pub fn benchmark_downloads_panel_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsPanelOpen)
            .is_err()
        {
            return false;
        }
        if self.downloads.open {
            self.performance.cancel();
            return false;
        }
        // Production footer activation (the exact seam `ClickFooterDownloads`
        // and the pointer/keyboard listeners route to).
        downloads_open(self, window, cx);
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, and the panel owns chrome focus with no
        // command surface active. Refresh the probe from the authoritative
        // post-mutation state so the presentation commit sees the real
        // state (the open render has not painted yet).
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.expect_downloads_open();
        true
    }

    /// Benchmark the production L6 progress path: a deterministic
    /// [`DownloadEvent::Progress`] is applied through the same main-thread
    /// pump the native adapter's sink uses, and the sample commits only
    /// when the panel render drew the running row at the armed
    /// `received`/`total` fraction. The attempt must already exist in
    /// `Running` at `total` (seeded by [`Self::seed_downloads_attempt`]
    /// through the portable controller with an empty web-view slot — no
    /// network transfer, so fixture/network time is excluded by
    /// construction). A rejected event (monotonicity, stable total, or a
    /// terminal record) cancels the sample truthfully instead of hanging.
    pub fn benchmark_downloads_progress_commit(
        &mut self,
        operation_id: BenchmarkOperationId,
        id: u64,
        received: u64,
        total: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsProgressCommit)
            .is_err()
        {
            return false;
        }
        let attempt = DownloadId(id);
        let Some(progress) = DownloadProgress::new(received, Some(total)).ok() else {
            self.performance.cancel();
            return false;
        };
        // Production event application: the exact seam the main-thread pump
        // drains adapter events through (`on_download_event`).
        on_download_event(
            self,
            DownloadEvent::Progress {
                id: attempt,
                progress,
            },
            cx,
        );
        // Truthful rejection: the event was rejected (regressed progress,
        // total change, or terminal record) when the record is not Running
        // at the armed received/total.
        let applied = matches!(
            self.downloads.controller.read().record(attempt).map(|r| &r.state),
            Some(bbr_web::downloads::DownloadState::Running { progress, .. })
                if progress.received() == received && progress.total() == Some(total)
        );
        if !applied {
            self.performance.cancel();
            return false;
        }
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance
            .expect_downloads_progress(id, received, total);
        true
    }

    /// Benchmark the production retry row action (`controller.retry`, the
    /// exact seam `DownloadRetry` and the row action listener route to):
    /// the sample commits only when the panel render drew the fresh attempt
    /// in the Active group — `Waiting`, linked via `retry_of` to the failed
    /// source row, with the Cancel action. The web-view slot is cleared so
    /// the action sink's `Start` dispatch fails as a typed `NoWebView` and
    /// no real `WKDownload` starts (the measured path is the portable retry
    /// seam + panel presentation; the real retry lifecycle is proven by the
    /// functional E2E). `Cancelled` rows are never retried; the handler
    /// rejects any non-`Failed` target.
    pub fn benchmark_downloads_row_retry(
        &mut self,
        operation_id: BenchmarkOperationId,
        row: usize,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> bool {
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));
        let Some(target) = visuals.get(row) else {
            return false;
        };
        if target.action != DownloadActionKind::Retry
            || !matches!(target.machine, DownloadMachine::Failed)
        {
            return false;
        }
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsRowRetry)
            .is_err()
        {
            return false;
        }
        let old_id = DownloadId(target.id);
        // No real transfer: clear the web-view slot so the portable
        // `DownloadAction::Start` dispatch fails as a typed `NoWebView`
        // (logged) and the fresh attempt stays `Waiting` — the deterministic
        // state the render commits from.
        #[cfg(target_os = "macos")]
        self.download_web_view.set(None);
        let Ok(fresh) = self.downloads.controller.get().retry(old_id) else {
            self.performance.cancel();
            return false;
        };
        let fresh_id = fresh.0;
        // The fresh attempt joins the Active group at its end (ascending
        // `DownloadId`), so once the seeded rows exceed the bounded render
        // window the commit render cannot draw it without scrolling. Slide
        // the panel anchor so the fresh row stays inside
        // `DOWNLOADS_WINDOW_ROWS` — presentation housekeeping before the
        // probe arms (the measured seam is the portable retry + the render
        // that presents the fresh row; the scroll state itself is
        // unmeasured).
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));
        if let Some(fresh_index) = visuals.iter().position(|row| row.id == fresh_id) {
            let total = visuals.len();
            let anchor = total
                .saturating_sub(bbr_ui::surfaces::downloads::DOWNLOADS_WINDOW_ROWS)
                .min(fresh_index);
            let layout = self.downloads_layout();
            self.downloads.scroll.set_offset(gpui::point(
                gpui::px(0.0),
                gpui::px(-layout.row_offset(anchor)),
            ));
        }
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.expect_downloads_retry(target.id, fresh_id);
        true
    }

    /// Benchmark the production restart row action on a cancelled row (the
    /// settled §17.8 decision): the same machine path as the failed retry
    /// (`controller.retry` accepts `Cancelled`; the panel labels it
    /// "Restart"). The sample commits only when the panel render drew the
    /// fresh attempt in the Active group — `Waiting`, linked via `retry_of`
    /// to the cancelled source row, with the Cancel action. The web-view
    /// slot is cleared so the action sink's `Start` dispatch fails as a
    /// typed `NoWebView` and no real `WKDownload` starts.
    pub fn benchmark_downloads_row_restart(
        &mut self,
        operation_id: BenchmarkOperationId,
        row: usize,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> bool {
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));
        let Some(target) = visuals.get(row) else {
            return false;
        };
        if target.action != DownloadActionKind::Restart
            || !matches!(target.machine, DownloadMachine::Cancelled)
        {
            return false;
        }
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsRowRestart)
            .is_err()
        {
            return false;
        }
        let old_id = DownloadId(target.id);
        #[cfg(target_os = "macos")]
        self.download_web_view.set(None);
        let Ok(fresh) = self.downloads.controller.get().retry(old_id) else {
            self.performance.cancel();
            return false;
        };
        let fresh_id = fresh.0;
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));
        if let Some(fresh_index) = visuals.iter().position(|row| row.id == fresh_id) {
            let total = visuals.len();
            let anchor = total
                .saturating_sub(bbr_ui::surfaces::downloads::DOWNLOADS_WINDOW_ROWS)
                .min(fresh_index);
            let layout = self.downloads_layout();
            self.downloads.scroll.set_offset(gpui::point(
                gpui::px(0.0),
                gpui::px(-layout.row_offset(anchor)),
            ));
        }
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance
            .expect_downloads_restart(target.id, fresh_id);
        true
    }

    /// Benchmark the production row Delete action on a cancelled row (the
    /// settled §17.8 decision): `download_delete_row` removes exactly that
    /// one terminal record and persists the remaining history durably. The
    /// sample commits only when the panel render no longer draws the
    /// deleted row (open, focused).
    pub fn benchmark_downloads_row_delete(
        &mut self,
        operation_id: BenchmarkOperationId,
        row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));
        let Some(target) = visuals.get(row) else {
            return false;
        };
        if !target.delete_available {
            return false;
        }
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsRowDelete)
            .is_err()
        {
            return false;
        }
        let id = DownloadId(target.id);
        if super::downloads_history::download_delete_row(self, id, cx).is_err() {
            self.performance.cancel();
            return false;
        }
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.expect_downloads_deleted(target.id);
        true
    }

    /// Benchmark the production Clear panel action (the settled §17.7
    /// decision): `downloads_clear` empties the controller history and
    /// persists the empty slice durably. The sample commits only when the
    /// panel render draws the empty state (open, focused, zero rows).
    pub fn benchmark_downloads_clear(
        &mut self,
        operation_id: BenchmarkOperationId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::DownloadsClear)
            .is_err()
        {
            return false;
        }
        super::downloads_history::downloads_clear(self, cx);
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.expect_downloads_clear();
        true
    }

    /// Seed a deterministic non-transferring download attempt for the
    /// performance scenarios: mint the attempt through the portable
    /// controller (the action sink's `Start` dispatch fails as a typed
    /// `NoWebView` because the web-view slot is cleared — no `WKDownload`
    /// starts) and apply `DestinationResolved` + an initial zero-byte
    /// `Progress` event through the production event path, leaving the
    /// attempt `Running` at the declared `total`. Returns the minted id.
    pub fn seed_downloads_attempt(
        &mut self,
        url: &str,
        destination: &str,
        total: u64,
        cx: &mut Context<Self>,
    ) -> Option<DownloadId> {
        #[cfg(target_os = "macos")]
        self.download_web_view.set(None);
        let id = self
            .downloads
            .controller
            .get()
            .start(url, DestinationDecision::Suggested(destination.to_owned()))
            .ok()?;
        on_download_event(
            self,
            DownloadEvent::DestinationResolved {
                id,
                destination: format!("downloads/{destination}"),
            },
            cx,
        );
        let progress = DownloadProgress::new(0, Some(total)).expect("valid zero-byte progress");
        on_download_event(self, DownloadEvent::Progress { id, progress }, cx);
        cx.notify();
        Some(id)
    }

    /// Seed a deterministic non-transferring **cancelled** terminal attempt
    /// for the restart/delete performance scenarios: [`Self::seed_downloads_attempt`]
    /// leaves the attempt `Running`, then the engine's terminal `Cancelled`
    /// event lands through the production event path (the first terminal
    /// event wins; no network transfer runs). Returns the cancelled id.
    pub fn seed_downloads_cancelled(
        &mut self,
        url: &str,
        destination: &str,
        total: u64,
        cx: &mut Context<Self>,
    ) -> Option<DownloadId> {
        let id = self.seed_downloads_attempt(url, destination, total, cx)?;
        on_download_event(self, DownloadEvent::Cancelled { id }, cx);
        cx.notify();
        Some(id)
    }
}
