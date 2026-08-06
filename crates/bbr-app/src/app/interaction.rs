//! Command-bar, navigation, and native-focus choreography.

use bbr_web::{WebAction, WebEvent};
use gpui::{Context, KeyDownEvent, Window};

use crate::command_bar::{CommandBarMode, CommandBarReturn};

use super::BbrApp;

impl BbrApp {
    /// Navigate the active view (used by location-mode Enter and the E2E gateway).
    pub fn navigate(&mut self, input: &str, window: &mut Window, cx: &mut Context<Self>) {
        let url = bbr_web::normalize_url(input);
        if url.is_empty() {
            return;
        }
        if self
            .controller
            .navigate(WebAction::Load { url: url.clone() })
            .is_err()
        {
            return;
        }
        self.command_bar_text = url;
        self.focus_web(window, cx);
    }

    fn submit_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.command_bar_text.clone();
        match self.command_bar_mode {
            CommandBarMode::Inactive => {}
            CommandBarMode::Location => self.navigate(&input, window, cx),
            CommandBarMode::NewTab => {
                let url = bbr_web::normalize_url(&input);
                if url.is_empty() {
                    return;
                }
                if self.open_tab(&url, cx).is_some() {
                    self.command_bar_text = url;
                    self.command_bar_return = None;
                    self.focus_web(window, cx);
                    #[cfg(feature = "testkit")]
                    {
                        self.performance.expect_state(
                            self.controller.snapshot().active_view,
                            self.sidebar_visible,
                            false,
                            self.command_bar_mode.into(),
                        );
                        self.commit_benchmark_sync(window);
                    }
                }
            }
            CommandBarMode::NewSpace => {
                // Create the space in the active profile and activate it;
                // chrome focus stays in the space list (a fresh space is
                // empty, so there is no page to hand focus to).
                let name = input.trim().to_string();
                if name.is_empty() {
                    return;
                }
                self.command_bar_text.clear();
                self.command_bar_return = None;
                if self.commit_new_space(&name, window, cx).is_some() {
                    #[cfg(feature = "testkit")]
                    {
                        self.performance.expect_state(
                            self.controller.snapshot().active_view,
                            self.sidebar_visible,
                            true,
                            self.command_bar_mode.into(),
                        );
                        self.commit_benchmark_sync(window);
                    }
                }
            }
            CommandBarMode::MoveTab => {
                // Move the active tab into the destination space named by the
                // staged text (exact title match, else the first space whose
                // title starts with it). Same-profile moves keep the session;
                // cross-profile moves recreate it once in the destination
                // store. Chrome focus stays in the tab list.
                let Some((space, _, _)) = self.resolve_move_space(&input) else {
                    return;
                };
                self.command_bar_text.clear();
                self.command_bar_return = None;
                if self.move_active_tab_to_space(space, window, cx).is_some() {
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
                }
            }
        }
    }

    pub fn go_back(&mut self) {
        let _ = self.controller.navigate(WebAction::Back);
    }

    pub fn go_forward(&mut self) {
        let _ = self.controller.navigate(WebAction::Forward);
    }

    pub fn reload(&mut self) {
        let _ = self.controller.navigate(WebAction::Reload);
    }

    /// Apply an engine event to the owning controller tab and refresh chrome.
    pub fn on_web_event(&mut self, event: WebEvent, cx: &mut Context<Self>) {
        let _ = self.controller.apply_event(event.clone());
        if !self.command_bar_mode.is_active() {
            self.sync_command_bar();
        }
        #[cfg(feature = "testkit")]
        self.performance.note_web_event(&event);
        cx.notify();
    }

    /// `Cmd-L`: edit the active tab's location.
    pub fn activate_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_command_profile_hint();
        self.command_bar_mode = CommandBarMode::Location;
        self.command_bar_return = None;
        self.sync_command_bar();
        self.focus_command_bar(window, cx);
    }

    /// `Cmd-T`: stage a destination without creating a tab or WKWebView.
    pub fn activate_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_command_profile_hint();
        if self.command_bar_mode != CommandBarMode::NewTab {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_bar_text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::NewTab;
        self.command_bar_text.clear();
        self.focus_command_bar(window, cx);
        #[cfg(feature = "testkit")]
        {
            self.performance.expect_state(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
                true,
                self.command_bar_mode.into(),
            );
            self.commit_benchmark_sync(window);
        }
    }

    /// `Cmd-Shift-M`: stage a destination space for the active tab without
    /// moving anything until Enter commits it. The staged text is matched
    /// against space titles; Escape restores the previous command-bar state
    /// and focus domain exactly like the other staged modes.
    pub fn activate_move_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.controller.snapshot().active_index.is_none() {
            return;
        }
        // The staged move has its own hint (command_bar_hint); the §11.3
        // space-switch transient must not coexist with it.
        self.clear_command_profile_hint();
        if self.command_bar_mode != CommandBarMode::MoveTab {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_bar_text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::MoveTab;
        self.command_bar_text.clear();
        self.focus_command_bar(window, cx);
        #[cfg(feature = "testkit")]
        {
            self.performance.expect_state(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
                true,
                self.command_bar_mode.into(),
            );
            self.commit_benchmark_sync(window);
        }
    }

    /// Resolve a staged move destination: the space whose title matches the
    /// text exactly (trimmed), else the first space whose title starts with
    /// it. Returns the space id, its title, and its profile id.
    pub(super) fn resolve_move_space(
        &self,
        text: &str,
    ) -> Option<(bbr_browser::SpaceId, String, bbr_web::ProfileId)> {
        let needle = text.trim();
        if needle.is_empty() {
            return None;
        }
        let snapshot = self.controller.snapshot();
        let exact = snapshot
            .spaces
            .iter()
            .find(|space| space.title.trim() == needle);
        let matched = exact.or_else(|| {
            snapshot
                .spaces
                .iter()
                .find(|space| space.title.trim().starts_with(needle))
        })?;
        let profile = snapshot.profiles.iter().find(|p| p.id == matched.profile)?;
        Some((
            bbr_browser::SpaceId(matched.id.0),
            matched.title.clone(),
            profile.id,
        ))
    }

    /// The §11.3 command-field destination hint for a staged move: "→ Space"
    /// within the active profile, "in Profile · <profile> → Space" when the
    /// destination crosses a profile boundary (the contract's cross-context
    /// communication). `None` when no move is staged or nothing matches.
    /// Rendered in the command field so the user sees the destination before
    /// committing.
    pub(super) fn command_bar_hint(&self) -> Option<String> {
        if self.command_bar_mode != CommandBarMode::MoveTab {
            return None;
        }
        let (_, title, profile) = self.resolve_move_space(&self.command_bar_text)?;
        if profile == self.controller.active_profile() {
            Some(format!("→ {title}"))
        } else {
            let snapshot = self.controller.snapshot();
            let profile_title = snapshot
                .profiles
                .iter()
                .find(|p| p.id == profile)
                .map(|p| p.title.clone())
                .unwrap_or_else(|| "other".to_string());
            Some(format!("in Profile · {profile_title} → {title}"))
        }
    }

    pub(super) fn enter_empty_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_return = None;
        self.command_bar_mode = CommandBarMode::NewTab;
        self.command_bar_text.clear();
        self.focus_command_bar(window, cx);
    }

    pub(super) fn focus_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.chrome_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// DESIGN.md §10.2 production hover path: the pointer entered row
    /// `index` (`hovered == true`) or left it (`false`). This is the exact
    /// app method the row's `on_hover` element listener calls, so the
    /// testkit seam drives the same path a real pointer move runs.
    pub fn set_row_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        let changed = if hovered {
            self.hovered_row != Some(index)
        } else {
            self.hovered_row == Some(index)
        };
        self.hovered_row = if hovered { Some(index) } else { None };
        if changed {
            cx.notify();
        }
    }

    /// Clear the hover state regardless of the row (the `UnhoverTabRow` seam
    /// and a pointer leaving the window).
    pub fn clear_row_hover(&mut self, cx: &mut Context<Self>) {
        if self.hovered_row.take().is_some() {
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production press path: the row's primary button went
    /// down. The exact app method the row's mouse-down listener calls.
    pub fn set_row_pressed(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.pressed_row != Some(index) {
            self.pressed_row = Some(index);
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production release path: the primary button went up
    /// (or left the window). The exact app method the row's mouse-up / the
    /// root's mouse-up-out listeners call.
    pub fn clear_row_pressed(&mut self, cx: &mut Context<Self>) {
        if self.pressed_row.take().is_some() {
            cx.notify();
        }
    }

    /// Escape cancels a staged tab, space, or move; location mode returns to
    /// web content. An active tab drag is cancelled first (chrome focus
    /// stays in the tab list).
    pub fn cancel_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tab_drag.is_active() {
            self.cancel_tab_drag(window, cx);
            return;
        }
        if !matches!(
            self.command_bar_mode,
            CommandBarMode::NewTab | CommandBarMode::NewSpace | CommandBarMode::MoveTab
        ) {
            self.focus_web(window, cx);
            return;
        }
        if let Some(restore) = self.command_bar_return.take() {
            self.command_bar_mode = restore.mode;
            self.command_bar_text = restore.text;
            if restore.chrome_focused {
                self.focus_command_bar(window, cx);
            } else {
                self.focus_web(window, cx);
            }
        } else if self.controller.snapshot().active_view.is_some() {
            self.sync_command_bar();
            self.focus_web(window, cx);
        }
    }

    /// Move GPUI focus + AppKit first responder back to the web view.
    pub fn focus_web(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.blur();
        if let Some(view) = self.controller.snapshot().active_view {
            self.engine.borrow_mut().focus_native_web(view);
        }
        cx.notify();
        self.emit_focus(window);
    }

    /// GPUI chrome gained focus without activating the command bar (click).
    pub(super) fn focus_chrome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.focus(&self.chrome_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    pub(super) fn sync_command_bar(&mut self) {
        let snapshot = self.controller.snapshot();
        self.command_bar_text = snapshot
            .nav
            .url
            .or_else(|| {
                snapshot
                    .active_index
                    .and_then(|index| snapshot.tabs.get(index).map(|tab| tab.url.clone()))
            })
            .unwrap_or_default();
    }

    /// Handle printable typing + backspace/enter while the command bar is active.
    pub(super) fn on_chrome_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.command_bar_mode.is_active() {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "backspace" => {
                self.command_bar_text.pop();
                cx.stop_propagation();
            }
            "enter" => {
                cx.stop_propagation();
                self.submit_command_bar(window, cx);
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        self.command_bar_text.push_str(ch);
                        cx.stop_propagation();
                    }
                }
            }
        }
        cx.notify();
    }
}
