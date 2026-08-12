//! Testkit-only Tab/Favorite context-menu benchmark drivers (task packet
//! `tab-favorite-context-menus`).
//!
//! `begin_tab_menu_open_benchmark` starts a sample at the exact production
//! open seam the sidebar tab row's right-button mouse-down listener routes
//! to (`open_tab_menu_from_row` at the invoking pointer); the dispatch
//! layer draws one real frame (the main-window render records the drawn
//! menu state into the probe), then `commit_tab_menu_open_benchmark`
//! commits only when that render drew the menu open, focused, bound to the
//! exact source `TabId`, under the armed source key, at the armed anchor —
//! the fast-but-wrong guard. `begin_tab_menu_submenu_commit_benchmark`
//! starts a sample at the production hover seam that opens the one-level
//! submenu card (`tab_menu_select`); the commit requires a render that drew
//! the open submenu card bound to the same source `TabId` at the armed
//! parent row. Neither path ever selects, moves, reloads, closes,
//! recreates, or changes the profile/space/session of the source tab.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{point, px, Context, Modifiers, MouseButton, MouseDownEvent, Window};

use super::BbrApp;

impl BbrApp {
    /// Begin a `tab_context_menu_open` sample: begin the probe, dispatch
    /// the production right-button open at the exact invoking pointer, arm
    /// the browser-state expectations (the same active view, sidebar,
    /// non-chrome focus domain, and command mode — the menu never changes
    /// them), and arm the exact menu presentation identity (bound source
    /// `TabId`, source key, anchor). The dispatch layer draws the frame
    /// that records the drawn menu state, then commits.
    pub fn begin_tab_menu_open_benchmark(
        &mut self,
        operation_id: BenchmarkOperationId,
        row: usize,
        at: [f32; 2],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::TabMenuOpen)
            .is_err()
        {
            return false;
        }
        if self.tab_menu.state.open {
            self.performance.cancel();
            return false;
        }
        // Production open: the exact seam the row's right-button mouse-down
        // listener routes to, with the synthetic event anchored at the
        // invoking pointer (the same construction the testkit right-click
        // seam uses).
        self.open_tab_menu_from_row(
            row,
            &MouseDownEvent {
                button: MouseButton::Right,
                position: point(px(at[0]), px(at[1])),
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: true,
            },
            window,
            cx,
        );
        let Some(tab) = self.tab_menu.state.bound_tab else {
            self.performance.cancel();
            return false;
        };
        let source_key = self.tab_menu.state.source_key.clone();
        let active_view = self.controller.snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        let chrome_focused = self.chrome_focused(window);
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, the menu focus is a non-chrome domain (so
        // the chrome-focus state is unchanged), and no command surface is
        // active.
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        self.performance.expect_tab_menu_open(tab.0, at, source_key);
        true
    }

    /// Commit the pending `tab_context_menu_open` sample from the drawn
    /// render milestone: the probe compares the armed open identity
    /// (bound tab, source key, anchor) against the observation the
    /// main-window render recorded, plus the armed browser state. Returns
    /// false and cancels when the render omitted the menu or the identity
    /// was wrong — a truthful rejection, never a fast-but-wrong pass.
    pub fn commit_tab_menu_open_benchmark(&self, window: &Window) -> bool {
        let active_view = self.controller.snapshot().active_view;
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::TabMenuOpen)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Begin a `tab_context_menu_submenu_commit` sample: the menu must
    /// already be open bound to the source `TabId` (the driver opens it
    /// unmeasured between samples); the sample starts at the production
    /// hover seam that opens the one-level submenu card for `parent_row`
    /// (`tab_menu_select`), then arms the browser state and the exact
    /// open-submenu identity. The dispatch layer draws the frame, then
    /// commits.
    pub fn begin_tab_menu_submenu_commit_benchmark(
        &mut self,
        operation_id: BenchmarkOperationId,
        parent_row: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::TabMenuSubmenuCommit)
            .is_err()
        {
            return false;
        }
        if !self.tab_menu.state.open {
            self.performance.cancel();
            return false;
        }
        let Some(tab) = self.tab_menu.state.bound_tab else {
            self.performance.cancel();
            return false;
        };
        // Production hover path into the open menu (the exact method the
        // submenu-bearing row's `on_hover` listener routes to): entering
        // the row opens its one-level submenu card.
        self.tab_menu_select(parent_row, true, window, cx);
        let active_view = self.controller.snapshot().active_view;
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
            .expect_tab_menu_submenu_commit(tab.0, parent_row);
        true
    }

    /// Commit the pending submenu-commit sample from the drawn render
    /// milestone: the probe compares the armed open-submenu identity
    /// against the observation the render recorded. Returns false and
    /// cancels when the submenu card was not drawn.
    pub fn commit_tab_menu_submenu_commit_benchmark(&self, window: &Window) -> bool {
        let active_view = self.controller.snapshot().active_view;
        let mode = self.command_bar_mode.into();
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            mode,
        );
        self.performance
            .expect_browser_state(active_view, self.sidebar_visible);
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::TabMenuSubmenuCommit)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }
}
