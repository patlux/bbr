//! Visible profile/space navigation (DESIGN.md §11.3, §12).
//!
//! The sidebar's space region renders every space grouped under its profile
//! with a per-row profile chip; this module owns the production activation
//! paths: pointer row selection, keyboard traversal (`↑`/`↓`/Enter on the
//! focused space list), and the `Cmd-Shift-N` / New Space row staging that
//! creates a space in the active profile through the command bar.
//!
//! Activation keeps the chrome focus domain and routes through the portable
//! controller, so switching spaces restores each space's own active tab while
//! warm sessions stay warm — including across profile boundaries (recreation
//! stays reserved for actual cross-profile tab moves).

use bbr_browser::{ControllerEvent, SpaceId};
use gpui::{Context, KeyDownEvent, Window};

use crate::command_bar::CommandBarMode;

use super::BbrApp;

impl BbrApp {
    /// The production space activation path: switch the controller's active
    /// space, keep chrome focus, and mirror the destination's tab into the
    /// command bar. Warm tabs and native sessions are preserved by the
    /// controller's show/hide policy.
    pub fn activate_space(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let result = self
            .controller
            .activate_space(space)
            .map_err(|error| {
                log::error!("could not activate space {space:?}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            self.emit_browser(outcome);
            self.focus_space_list(window, cx);
            self.ensure_active_visible();
            #[cfg(feature = "testkit")]
            self.commit_benchmark_sync(window);
            cx.notify();
        }
        result
    }

    /// Pointer activation of a visible space row: chrome focus domain
    /// preserved, activation routed through the same controller mutation the
    /// keyboard path uses (DESIGN.md §11.3 / interaction contract).
    pub fn select_space_via_pointer(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_space_list(window, cx);
        if self.activate_space(space, window, cx).is_some() {
            cx.notify();
        }
    }

    /// Focus the space list (chrome domain) without activating the command
    /// bar — the pointer path into the spaces region.
    pub(super) fn focus_space_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.focus(&self.space_list_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Keyboard navigation of the visible space list (DESIGN.md §9 order:
    /// command field → spaces → pinned → regular tabs): `↑/↓` activate the
    /// previous/next space in render order, Enter re-activates the focused
    /// space. Runs only while the space list owns chrome focus and no command
    /// mode is active. Activation keeps the chrome domain — the keyboard
    /// equivalent of clicking a space row.
    pub(super) fn on_space_list_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let snapshot = self.controller.snapshot();
        if snapshot.spaces.is_empty() {
            return;
        }
        // Render order is the grouped sidebar order (DESIGN.md §11.3).
        let order: Vec<SpaceId> = crate::shell::space_groups(&snapshot, usize::MAX)
            .into_iter()
            .flat_map(|group| group.rows.into_iter())
            .map(|row| {
                snapshot
                    .spaces
                    .iter()
                    .find(|s| s.index == row.index)
                    .expect("space row derives from the snapshot")
                    .id
            })
            .collect();
        let current = order
            .iter()
            .position(|id| *id == snapshot.active_space)
            .unwrap_or(0);
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                let len = order.len();
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let next = (current as isize + delta).rem_euclid(len as isize) as usize;
                if self.activate_space(order[next], window, cx).is_some() {
                    cx.notify();
                }
                cx.stop_propagation();
            }
            "enter" => {
                // Re-activate the focused space (no-op when already active).
                if self.activate_space(order[current], window, cx).is_some() {
                    cx.notify();
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// `Cmd-Shift-N` / New Space row: stage a space name without creating a
    /// profile or space until Enter commits it (the NewTab staging pattern).
    pub fn activate_new_space(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.command_bar_mode != CommandBarMode::NewSpace {
            self.command_bar_return = Some(crate::command_bar::CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_bar_text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::NewSpace;
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

    /// Commit a staged New Space name: create the space in the active
    /// profile and activate it. Chrome focus stays in the space list — a
    /// fresh space is empty, so there is no page to hand focus to.
    pub fn commit_new_space(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        let profile = self.controller.active_profile();
        let result = self
            .controller
            .create_space_event(profile, name)
            .map_err(|error| {
                log::error!("could not create space {name:?}: {error}");
                error
            })
            .ok();
        let outcome = result?;
        let space = match outcome {
            ControllerEvent::SpaceCreated { space, .. } => space,
            other => {
                log::error!("unexpected create-space outcome: {other:?}");
                return None;
            }
        };
        // The creation itself is a semantic browser mutation and must be
        // observable like every other mutation path (the gateway's CreateSpace
        // emits SpaceCreated; the keyboard path must emit the same event so
        // E2E and future consumers can distinguish creation from activation).
        self.emit_browser(&outcome);
        self.activate_space(space, window, cx);
        Some(outcome)
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_activate_space(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::WarmSpaceSwitch,
            )
            .is_err()
        {
            return false;
        }
        // The production activation path, exactly as the row mouse-down
        // listener drives it (chrome focus domain preserved). The benchmark
        // commits synchronously from the authoritative post-mutation snapshot
        // with the destination space as the semantic state check.
        let outcome = self.activate_space(space, window, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        let snapshot = self.controller.snapshot();
        let active_space = bbr_testkit::protocol::TestSpaceId(snapshot.active_space.0);
        self.performance.observe_active_space(active_space);
        self.performance.expect_space(active_space);
        self.performance.expect_state(
            snapshot.active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        self.performance.commit_current_presentation();
        true
    }
}
