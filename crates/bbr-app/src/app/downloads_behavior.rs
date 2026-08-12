//! Downloads panel behavior (DESIGN.md §11.12): the footer-control
//! activation, the anchored-popover open/close/dismissal, the contextual row
//! actions (Cancel / Retry / Restart / Reveal-in-Finder), and the L6 event
//! application. The presentation surface lives in
//! `bbr_ui::surfaces::downloads` (extracted in the G2 Group 5 cutover); the
//! pure state/row model in `app::downloads`; the durable history mutations
//! (Clear / row Delete / persist) in `app::downloads_history`.
//!
//! Machine fidelity: every mutation goes through the portable
//! [`DownloadController`]; the panel never invents a transition. Cancel is a
//! request that stays active until the engine's terminal event lands (the
//! adapter confirms with an L6 event); retry exists on `Failed` and restart
//! on `Cancelled` (the settled §17.8 decision) — both start a fresh attempt
//! linked via `retry_of`; Reveal-in-Finder runs only for an existing
//! resolved terminal path through the native adapter (a vanished file marks
//! the action §10.8 `disabled` with a reason). Every accepted terminal
//! transition persists the terminal history durably
//! (`app::downloads_history::persist_downloads`).

use bbr_browser::DownloadError;
#[cfg(feature = "testkit")]
use bbr_web::downloads::DestinationDecision;
use bbr_web::downloads::{DownloadEvent, DownloadId, DownloadState};
use gpui::{Context, KeyDownEvent, Window};

use super::downloads::{row_visuals, RevealEligibility};
use super::BbrApp;
use bbr_ui::surfaces::downloads::{DownloadActionKind, DOWNLOADS_MAX_VISIBLE, DOWNLOADS_PAGE_ROWS};

/// The §11.12 footer `[Downloads]` control: toggle the anchored popover
/// (pointer or keyboard activation; a second activation dismisses it).
pub fn toggle_downloads(app: &mut BbrApp, window: &mut Window, cx: &mut Context<BbrApp>) {
    if app.downloads.open {
        downloads_close(app, window, cx);
    } else {
        downloads_open(app, window, cx);
    }
}

/// Open the anchored popover: close any open plus menu (exactly one
/// popover surface per window), re-check Reveal-in-Finder eligibility for
/// every succeeded row (the file may have vanished since its event), reset
/// the keyboard cursor, capture the scroll anchor, arm the `dur:fine`
/// opacity fade-in, and focus the panel.
pub fn downloads_open(app: &mut BbrApp, window: &mut Window, cx: &mut Context<BbrApp>) {
    if app.downloads.open {
        return;
    }
    // At most one sidebar popover at a time: opening the downloads panel
    // dismisses the plus menu and the §11.3 configuration surface (the
    // config draft is not applied), and vice versa.
    if app.u2.plus_menu.open {
        app.u2.plus_menu.open = false;
    }
    if app.space_config.open {
        app.space_config.open = false;
        app.space_config.space = None;
        app.space_config.press_inside = false;
    }
    app.command_bar_mode = crate::command_bar::CommandBarMode::Inactive;
    app.command_bar_return = None;
    app.command_ranking.close();
    app.downloads.open = true;
    app.downloads.focus = 0;
    app.downloads.hovered_row = None;
    app.downloads.pressed_row = None;
    // §11.12 open/close: opacity fade `dur:fine` only. The transition is
    // created lazily by the render (it needs the window state); when it
    // already exists, retarget the goal to fully opaque.
    if let Some(fade) = &app.downloads.opacity {
        fade.update(cx, |value, cx| {
            *value = 1.0;
            cx.notify();
        });
    }
    app.refresh_all_reveal_eligibility();
    app.downloads
        .scroll
        .set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
    window.focus(&app.downloads_panel_handle, cx);
    app.engine.borrow_mut().focus_host_chrome();
    cx.notify();
    app.emit_focus(window);
}

/// Close the anchored popover and return focus to the footer `[Downloads]`
/// control (DESIGN.md §11.12 focus domain; §9 order: Downloads precedes the
/// space chips). The `dur:fine` opacity fade-out runs to zero; the render
/// keeps the panel mounted until the fade completes.
pub fn downloads_close(app: &mut BbrApp, window: &mut Window, cx: &mut Context<BbrApp>) {
    if !app.downloads.open {
        return;
    }
    app.downloads.open = false;
    app.downloads.hovered_row = None;
    app.downloads.pressed_row = None;
    if let Some(fade) = &app.downloads.opacity {
        fade.update(cx, |value, cx| {
            *value = 0.0;
            cx.notify();
        });
    }
    window.focus(&app.downloads_control_handle, cx);
    app.engine.borrow_mut().focus_host_chrome();
    cx.notify();
    app.emit_focus(window);
}

/// Start a fresh download: mint the attempt through the controller (which
/// emits [`DownloadAction::Start`] into the installed adapter sink) and keep
/// the fresh active attempt visible by resetting the panel scroll to the
/// top. The destination policy is L6 [`DestinationDecision::Suggested`]
/// (the settled silent suggested-destination policy, DESIGN.md §17.9).
#[cfg(feature = "testkit")]
pub fn download_start(
    app: &mut BbrApp,
    url: &str,
    destination: DestinationDecision,
    cx: &mut Context<BbrApp>,
) -> Result<DownloadId, DownloadError> {
    #[cfg(target_os = "macos")]
    app.capture_download_web_view();
    let id = app.downloads.controller.start(url, destination)?;
    app.downloads
        .scroll
        .set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
    cx.notify();
    Ok(id)
}

/// Request cancellation of the attempt (L6 request/confirmation): the row
/// stays active until the engine's terminal event lands. No optimistic
/// state change.
pub fn download_cancel(
    app: &mut BbrApp,
    id: DownloadId,
    cx: &mut Context<BbrApp>,
) -> Result<(), DownloadError> {
    app.downloads.controller.cancel(id)?;
    cx.notify();
    Ok(())
}

/// Retry the failed attempt (or restart the cancelled attempt — the settled
/// §17.8 decision): a fresh attempt id linked via `retry_of` (L6
/// [`evaluate_retry`] accepts `Failed` and `Cancelled`; the panel labels the
/// same machine path differently by state).
pub fn download_retry(
    app: &mut BbrApp,
    id: DownloadId,
    cx: &mut Context<BbrApp>,
) -> Result<DownloadId, DownloadError> {
    #[cfg(target_os = "macos")]
    app.capture_download_web_view();
    let new_id = app.downloads.controller.retry(id)?;
    app.downloads
        .scroll
        .set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
    cx.notify();
    Ok(new_id)
}

/// Activate the visible row's contextual action through the production
/// pointer path (the exact app method the action's mouse-down listener
/// calls): Cancel / Retry / Restart / Reveal-in-Finder. A disabled action
/// (a vanished reveal target) is a no-op.
pub fn download_click_row_action(
    app: &mut BbrApp,
    row: usize,
    window: &mut Window,
    cx: &mut Context<BbrApp>,
) {
    let visuals = row_visuals(&app.downloads, Some(app.downloads.focus));
    let Some(visual) = visuals.get(row) else {
        return;
    };
    let id = DownloadId(visual.id);
    match visual.action {
        DownloadActionKind::Cancel => {
            let _ = download_cancel(app, id, cx);
        }
        DownloadActionKind::Retry => {
            let _ = download_retry(app, id, cx);
        }
        DownloadActionKind::Restart => {
            let _ = download_retry(app, id, cx);
        }
        DownloadActionKind::Reveal => {
            if !visual.action_enabled {
                return;
            }
            let Some(path) = visual.resolved_path.clone() else {
                return;
            };
            #[cfg(target_os = "macos")]
            if let Some(adapter) = app.downloads_adapter.as_ref() {
                if let Err(error) = adapter.reveal(&path) {
                    log::warn!("reveal in finder failed: {error:?}");
                    // A vanished file marks the action §10.8 disabled with a
                    // reason until the next successful reveal.
                    app.downloads.reveal.insert(
                        id,
                        RevealEligibility {
                            eligible: false,
                            reason: Some("file missing".into()),
                        },
                    );
                }
            }
            let _ = window;
        }
    }
    cx.notify();
}

/// Apply one L6 engine event through the controller (the adapter's event
/// sink routes into the main-thread pump). The first terminal event wins;
/// rejected events leave the record unchanged. An accepted **terminal**
/// transition persists the terminal history durably (the repository write
/// runs here on the main-thread pump — outside render and ranking; the
/// identical whole-replace write the Clear/Delete actions use). A row that
/// reached `Succeeded` re-checks its Reveal-in-Finder eligibility natively.
pub fn on_download_event(app: &mut BbrApp, event: DownloadEvent, cx: &mut Context<BbrApp>) {
    let id = event_id(&event);
    let outcome = app.downloads.controller.apply(event);
    if matches!(outcome, bbr_browser::ApplyOutcome::Applied) {
        let terminal = app
            .downloads
            .controller
            .record(id)
            .is_some_and(|record| record.state.is_terminal());
        app.refresh_reveal_eligibility(id);
        if terminal {
            app.persist_downloads();
        }
    }
    cx.notify();
}

fn event_id(event: &DownloadEvent) -> DownloadId {
    match event {
        DownloadEvent::DestinationResolved { id, .. }
        | DownloadEvent::Progress { id, .. }
        | DownloadEvent::Succeeded { id }
        | DownloadEvent::Failed { id, .. }
        | DownloadEvent::Cancelled { id } => *id,
    }
}

impl BbrApp {
    /// Recompute the Reveal-in-Finder eligibility of every succeeded record
    /// (run on panel open: the file may have vanished since its terminal
    /// event; DESIGN.md §11.12 enables the action only for an existing
    /// resolved path).
    pub(crate) fn refresh_all_reveal_eligibility(&mut self) {
        for id in self.downloads.row_ids() {
            self.refresh_reveal_eligibility(id);
        }
    }

    /// Recompute one record's Reveal-in-Finder eligibility through the
    /// native adapter (a pure existence query — never opens Finder). Only
    /// `Succeeded` records with a resolved path are eligible.
    pub(crate) fn refresh_reveal_eligibility(&mut self, id: DownloadId) {
        let Some(record) = self.downloads.controller.record(id) else {
            return;
        };
        if !matches!(record.state, DownloadState::Succeeded) {
            return;
        }
        let Some(path) = record.resolved_path.clone() else {
            self.downloads.reveal.insert(
                id,
                RevealEligibility {
                    eligible: false,
                    reason: Some("no resolved path".into()),
                },
            );
            return;
        };
        #[cfg(target_os = "macos")]
        let eligible = self
            .downloads_adapter
            .as_ref()
            .map(|adapter| adapter.reveal_eligible(&path))
            .unwrap_or(true);
        #[cfg(not(target_os = "macos"))]
        let eligible = true;
        self.downloads.reveal.insert(
            id,
            RevealEligibility {
                eligible,
                reason: if eligible {
                    None
                } else {
                    Some("file missing".into())
                },
            },
        );
    }

    /// §11.12 keyboard operation: `↑`/`↓`/`PgUp`/`PgDn` move the keyboard
    /// cursor and scroll, Enter activates the focused row's action,
    /// Tab/Shift-Tab move the cursor between rows, Escape dismisses and
    /// returns focus to the footer control. The panel stops propagation so
    /// the app-wide Escape binding never fires while it is open.
    pub(super) fn on_downloads_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.downloads.open {
            return;
        }
        // Escape dismisses unconditionally (DESIGN.md §11.12 Dismiss — it is
        // independent of content). In practice GPUI dispatches the app-wide
        // `escape → FocusWeb` binding first and actions stop propagation, so
        // Escape reaches the panel only if the binding is ever skipped; the
        // binding's own route (`cancel_command_bar`) is the primary
        // dismissal. Either way the outcome is identical: close and return
        // focus to the footer control. The empty panel is covered too — only
        // the row-navigation keys (↑/↓/PgUp/PgDn/Tab/Enter) need rows.
        if event.keystroke.key.as_str() == "escape" {
            cx.stop_propagation();
            downloads_close(self, window, cx);
            cx.notify();
            return;
        }
        let total = self.downloads.controller.len();
        if total == 0 {
            return;
        }
        match event.keystroke.key.as_str() {
            "up" => {
                self.downloads_focus_move(-1, total);
                cx.stop_propagation();
            }
            "down" => {
                self.downloads_focus_move(1, total);
                cx.stop_propagation();
            }
            "pageup" => {
                self.downloads_focus_page(-1, total);
                cx.stop_propagation();
            }
            "pagedown" => {
                self.downloads_focus_page(1, total);
                cx.stop_propagation();
            }
            "tab" => {
                let delta = if event.keystroke.modifiers.shift {
                    -1
                } else {
                    1
                };
                self.downloads_focus_move(delta, total);
                cx.stop_propagation();
            }
            "enter" => {
                cx.stop_propagation();
                download_click_row_action(self, self.downloads.focus.min(total - 1), window, cx);
            }
            _ => {}
        }
        cx.notify();
    }

    /// Move the keyboard cursor by `delta` rows (clamped to the deterministic
    /// row list) and scroll the focused row into view instantly.
    fn downloads_focus_move(&mut self, delta: isize, total: usize) {
        let current = self.downloads.focus.min(total.saturating_sub(1));
        let next = (current as isize + delta).clamp(0, total as isize - 1) as usize;
        self.downloads.focus = next;
        self.ensure_download_focus_visible();
    }

    /// Page the keyboard cursor by one viewport of rows and scroll a page.
    fn downloads_focus_page(&mut self, direction: isize, total: usize) {
        let current = self.downloads.focus.min(total.saturating_sub(1));
        let step = DOWNLOADS_PAGE_ROWS as isize * direction;
        let next = (current as isize + step).clamp(0, total as isize - 1) as usize;
        self.downloads.focus = next;
        // A page move re-anchors the viewport so the focused row lands at
        // the viewport's leading edge (page up) or trailing edge (page down).
        let row_h = f32::from(self.theme().density.row());
        let anchor = if direction < 0 {
            next
        } else {
            next.saturating_add(1).saturating_sub(DOWNLOADS_MAX_VISIBLE)
        };
        self.downloads.scroll.set_offset(gpui::point(
            gpui::px(0.0),
            gpui::px(-(anchor as f32 * row_h)),
        ));
    }

    /// DESIGN.md §11.12 keyboard scroll: the focused row is scrolled into
    /// view instantly when the keyboard cursor moves outside the visible
    /// viewport (the same instant scroll contract as §11.6).
    fn ensure_download_focus_visible(&mut self) {
        let total = self.downloads.controller.len();
        if total == 0 {
            return;
        }
        let row_h = f32::from(self.theme().density.row());
        let anchor = self.downloads.anchor_from_offset(row_h);
        let focus = self.downloads.focus.min(total - 1);
        if focus < anchor {
            self.downloads.scroll.set_offset(gpui::point(
                gpui::px(0.0),
                gpui::px(-(focus as f32 * row_h)),
            ));
        } else if focus >= anchor + DOWNLOADS_MAX_VISIBLE {
            let anchor = focus
                .saturating_add(1)
                .saturating_sub(DOWNLOADS_MAX_VISIBLE);
            self.downloads.scroll.set_offset(gpui::point(
                gpui::px(0.0),
                gpui::px(-(anchor as f32 * row_h)),
            ));
        }
    }

    /// §11.12 pointer hover/press: the row's hover wash (§10.2) and pressed
    /// wash (§10.3) are app presentation state the render derives.
    pub fn set_downloads_hover(&mut self, row: usize, hovered: bool, cx: &mut Context<Self>) {
        if hovered {
            self.downloads.hovered_row = Some(row);
        } else if self.downloads.hovered_row == Some(row) {
            self.downloads.hovered_row = None;
        }
        cx.notify();
    }

    pub fn set_downloads_pressed(&mut self, row: Option<usize>, cx: &mut Context<Self>) {
        self.downloads.pressed_row = row;
        cx.notify();
    }

    /// §13 one-pointer-action dismissal: a pointer press outside the open
    /// panel dismisses it and returns focus to the footer control. The panel
    /// surface marks presses inside via `downloads_press_inside` (its
    /// handler runs first in the bubble phase); any other press closes it.
    pub fn dismiss_downloads_on_outside_press(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inside = self.downloads_press_inside.get();
        self.downloads_press_inside.set(false);
        if self.downloads.open && !inside {
            downloads_close(self, window, cx);
        }
    }

    /// Dense-list scroll entry for the panel (testkit seam; DESIGN.md
    /// §11.12/§11.6): move the tracked scroll state by `delta_px` in the
    /// wheel listener's units and sign, exactly like a native wheel event.
    #[cfg(feature = "testkit")]
    pub fn scroll_downloads(&self, delta_px: f32) {
        let current = self.downloads.scroll.offset().y;
        self.downloads
            .scroll
            .set_offset(gpui::point(gpui::px(0.0), current + gpui::px(delta_px)));
    }

    /// Capture the active view's live `WKWebView` for the adapter's Start
    /// requests (macOS). The action sink reads the shared slot when the
    /// controller emits [`DownloadAction::Start`]; this refreshes it right
    /// before a start/retry so the sink always dispatches on the current
    /// active session.
    #[cfg(target_os = "macos")]
    pub(crate) fn capture_download_web_view(&mut self) {
        let active = self.controller.snapshot().active_view;
        let web_view = self
            .download_engine
            .as_ref()
            .and_then(|engine| active.and_then(|view| engine.borrow().web_view(view)));
        self.download_web_view.set(web_view);
    }

    /// The panel-surface inside-press marker (shared with the render): the
    /// panel's mouse-down handler sets it and stops propagation, so the root
    /// outside-press handler never dismisses a press inside the panel.
    pub(crate) fn mark_downloads_press_inside(&mut self) {
        self.downloads_press_inside.set(true);
    }
}
