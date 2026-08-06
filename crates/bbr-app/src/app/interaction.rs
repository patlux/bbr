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
                        self.commit_benchmark_after_frame(window, cx);
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
        let _ = self.controller.apply_event(event);
        if !self.command_bar_mode.is_active() {
            self.sync_command_bar();
        }
        cx.notify();
    }

    /// `Cmd-L`: edit the active tab's location.
    pub fn activate_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Location;
        self.command_bar_return = None;
        self.sync_command_bar();
        self.focus_command_bar(window, cx);
    }

    /// `Cmd-T`: stage a destination without creating a tab or WKWebView.
    pub fn activate_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_bar_mode != CommandBarMode::NewTab {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_bar_text.clone(),
                chrome_focused: self.chrome_handle.is_focused(window),
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
            self.commit_benchmark_after_frame(window, cx);
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

    /// Escape cancels a staged tab; location mode returns to web content.
    pub fn cancel_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_bar_mode != CommandBarMode::NewTab {
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
