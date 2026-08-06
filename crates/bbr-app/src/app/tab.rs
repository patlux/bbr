//! Tab-list pointer and keyboard interaction (DESIGN.md §11.1/§11.2).
//!
//! The visible tab surface lives in `app::sidebar_view`; this module owns
//! the production interaction paths: pointer selection/close through the row
//! listeners, keyboard traversal of the focused list (`↑`/`↓`, Enter,
//! backspace), reopen, and the crash-panel reload. Every mutation routes
//! through the controller and preserves the chrome focus domain, matching
//! the interaction contract.

use bbr_browser::ControllerEvent;
use gpui::{Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Pointer selection of a visible tab row: chrome focus domain preserved,
    /// selection routed through the same controller mutation the keyboard
    /// path uses (DESIGN.md §9 / interaction contract).
    pub fn select_row_via_pointer(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        if self.select_tab(index, cx).is_some() {
            cx.notify();
        }
    }

    /// Pointer close of a visible tab row: chrome focus domain preserved.
    /// Closing the final tab opens the empty New Tab command surface, the
    /// same outcome the Cmd-W path produces (interaction contract: empty
    /// space offers an obvious way to open a new tab).
    pub fn close_row_via_pointer(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        let closed = self.close_tab(index, cx).is_some();
        if closed && self.controller.snapshot().active_view.is_none() {
            self.enter_empty_new_tab(window, cx);
        }
    }

    /// Focus the tab list (chrome domain) without activating the command bar.
    pub(super) fn focus_tab_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = crate::command_bar::CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.focus(&self.tab_list_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Keyboard navigation of the visible tab list (DESIGN.md §11.1):
    /// `↑/↓` move selection, `backspace` closes the active tab, Enter
    /// activates. Runs only while the tab list owns chrome focus and no
    /// command mode is active.
    pub(super) fn on_tab_list_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let snapshot = self.controller.snapshot();
        let Some(active) = snapshot.active_index else {
            return;
        };
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                let len = snapshot.tabs.len();
                if len > 1 {
                    let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                    let next = (active as isize + delta).rem_euclid(len as isize) as usize;
                    if self.select_tab(next, cx).is_some() {
                        cx.notify();
                    }
                }
                cx.stop_propagation();
            }
            "enter" => {
                // DESIGN.md §11.1 / interaction contract: Enter activates the
                // selected tab — the chrome traversal commits and keyboard
                // focus moves into the page (the same outcome as a committed
                // New-Tab command). No-op when no live view is presented
                // (crashed tab or empty space); selection never changes here.
                if self.controller.snapshot().active_view.is_some() {
                    self.focus_web(window, cx);
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
                cx.stop_propagation();
            }
            "backspace" => {
                let _ = self.close_active_tab(window, cx);
                // Closing the final tab already opened the empty New Tab
                // command surface (command-bar focus, NewTab mode — the same
                // outcome as the pointer path). Only a close that leaves tabs
                // behind keeps the list focused so repeated keys keep working.
                if self.controller.snapshot().active_view.is_some() {
                    self.focus_tab_list(window, cx);
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// Reload the crashed active tab from the crash panel (fresh session).
    pub fn reload_active_from_panel(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.reload();
        cx.notify();
    }

    /// Reopen the most recently closed tab (`Cmd-Shift-T`). Contract: the
    /// tab keeps its identity/URL/title, receives a fresh native session, and
    /// the focus domain of the invoking command is preserved (chrome stays in
    /// the tab list, web stays in web content).
    pub fn reopen_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let chrome_focused = self.chrome_focused(window);
        let result = self
            .controller
            .reopen_tab()
            .map_err(|error| {
                log::error!("could not reopen tab: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            self.emit_browser(outcome);
            if chrome_focused {
                self.focus_tab_list(window, cx);
            } else {
                self.focus_web(window, cx);
            }
            cx.notify();
        }
        result
    }

    /// Pin the tab at `index` (moves it into the pinned region). Testkit-only
    /// until the pin keyboard command ships.
    #[cfg(feature = "testkit")]
    pub fn pin_tab(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .pin_tab(index)
            .map_err(|error| {
                log::error!("could not pin tab {index}: {error}");
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

    /// Toggle the pin state of the tab at `index` through the production
    /// controller path (pin → unpin → pin). The selected tab stays selected
    /// and the live session is preserved. `window` is threaded for symmetry
    /// with the other mutation paths; focus is left untouched so the invoking
    /// focus domain is preserved (interaction contract: tab mutations keep
    /// the domain that initiated them).
    pub fn toggle_pin(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let snapshot = self.controller.snapshot();
        let pinned = snapshot.tabs.get(index).is_some_and(|tab| tab.pinned);
        let result = if pinned {
            self.controller.unpin_tab(index)
        } else {
            self.controller.pin_tab(index)
        }
        .map_err(|error| {
            log::error!("could not toggle pin on tab {index}: {error}");
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

    /// `Cmd-Shift-P`: pin or unpin the active tab, preserving the invoking
    /// focus domain and the live session.
    pub fn pin_active_tab(&mut self, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let index = self.controller.snapshot().active_index?;
        self.toggle_pin(index, cx)
    }

    /// Pointer pin control of a visible tab row: chrome focus domain
    /// preserved (pointer interaction with sidebar controls gives focus to
    /// browser chrome), then toggled through the same controller mutation the
    /// keyboard command uses.
    pub fn toggle_row_pin(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_tab_list(window, cx);
        if self.toggle_pin(index, cx).is_some() {
            cx.notify();
        }
    }

    /// `Cmd-Shift-↑`/`Cmd-Shift-↓`: move the active tab one step within its
    /// current region. A step that would cross the pinned/regular boundary is
    /// rejected whole (no partial mutation; the order and sessions stay
    /// untouched), matching the interaction contract's reorder rule. Focus is
    /// left untouched so the invoking focus domain is preserved.
    pub fn reorder_active_tab(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let from = self.controller.snapshot().active_index?;
        let to = (from as isize + delta).max(0) as usize;
        self.reorder_tab_at(from, to, cx)
    }

    /// Reorder the tab at `from` to `to` through the production controller
    /// path. The step must stay inside the source region: the domain rejects
    /// a pinned/regular boundary crossing whole (no partial mutation), and
    /// the app additionally refuses to even attempt a step outside the
    /// region so the tick cannot suggest a boundary crossing the domain will
    /// reject. Sessions and selection are untouched.
    pub fn reorder_tab_at(
        &mut self,
        from: usize,
        to: usize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let snapshot = self.controller.snapshot();
        let len = snapshot.tabs.len();
        if len < 2 {
            return None;
        }
        let pinned = crate::shell::pinned_count(&snapshot.tabs);
        // Region-bound: the pinned region is [0, pinned), the regular region
        // [pinned, len). A step outside the source region is the boundary
        // crossing the domain rejects; never wrap around.
        let in_region = if from < pinned {
            (0..pinned).contains(&to)
        } else {
            (pinned..len).contains(&to)
        };
        if !in_region {
            return None;
        }
        let result = self
            .controller
            .reorder_tab(from, to)
            .map_err(|error| {
                log::error!("could not reorder tab {from} -> {to}: {error}");
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

    /// Move the active tab into `space` through the production controller
    /// path: same-profile moves keep the live session; cross-profile moves
    /// recreate it exactly once in the destination profile's store. The
    /// source space's selection is repaired by the domain; chrome focus stays
    /// in the tab list (both invocation paths — the staged move command and
    /// the drag — are chrome interactions).
    pub fn move_active_tab_to_space(
        &mut self,
        space: bbr_browser::SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let index = self.controller.snapshot().active_index?;
        self.move_tab_to_space_at(index, space, window, cx)
    }

    /// Move the tab at `index` into `space` through the production
    /// controller path (the shared mutation of the staged move command and
    /// the pointer drag). Same-profile moves keep the live session;
    /// cross-profile moves recreate it exactly once in the destination
    /// profile's store. The source space's selection is repaired by the
    /// domain; chrome focus stays in the tab list.
    pub fn move_tab_to_space_at(
        &mut self,
        index: usize,
        space: bbr_browser::SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self
            .controller
            .move_tab(index, space)
            .map_err(|error| {
                log::error!("could not move tab {index} to {space:?}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            self.emit_browser(outcome);
            self.focus_tab_list(window, cx);
            cx.notify();
        }
        result
    }

    /// Unpin the tab at `index` (moves it to the end of the regular region).
    /// Testkit-only until the unpin keyboard command ships alongside the pin
    /// toggle (`Cmd-Shift-P` toggles; this seam seeds the unpinned state in
    /// E2E).
    #[cfg(feature = "testkit")]
    pub fn unpin_tab(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .unpin_tab(index)
            .map_err(|error| {
                log::error!("could not unpin tab {index}: {error}");
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

    /// Reorder the tab at `from` to `to` (E2E semantic seam for setup; the
    /// user paths are the keyboard steps and the pointer drag, both of which
    /// enforce the region boundary before the controller rejects it).
    #[cfg(feature = "testkit")]
    pub fn reorder_tab_seam(
        &mut self,
        from: usize,
        to: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .reorder_tab(from, to)
            .map_err(|error| {
                log::error!("could not reorder tab {from} -> {to}: {error}");
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
}
