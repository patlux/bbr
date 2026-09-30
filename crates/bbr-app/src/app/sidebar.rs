//! Sidebar resize interaction (DESIGN.md §5/§11.4, §9).
//!
//! The divider surface lives in `bbr-ui::sidebar::geometry`; this module owns the
//! production input wiring: pointer drag through the GPUI listener path
//! (mouse-down on the divider starts the drag, mouse-move on the root
//! follows it even outside the 4 pt hairline, mouse-up commits), the
//! keyboard equivalents for the focused divider (`←`/`→` resize, `0`
//! reset), and the shared width mutation every path funnels through. The
//! chrome focus domain is preserved exactly like the tab/space pointer
//! paths, and the pointer + keyboard mutations land on the same
//! `SidebarGeometry` value, so parity holds by construction.

use gpui::{
    Context, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Window,
};

use crate::command_bar::CommandBarMode;
use bbr_ui::sidebar::geometry::{RESIZE_STEP, SIDEBAR_DEFAULT};

use super::BbrApp;

impl BbrApp {
    /// The production width mutation shared by every input path (pointer
    /// drag, keyboard steps, reset): set the geometry (which also remembers
    /// the last expanded width while visible), notify, and under testkit arm
    /// the pending presentation expectation so a fast-but-wrong sample cannot
    /// be accepted.
    #[cfg_attr(not(feature = "testkit"), allow(unused_variables))]
    pub fn resize_sidebar_to(&mut self, width: f32, window: &mut Window, cx: &mut Context<Self>) {
        let visible = self.sidebar_visible;
        self.sidebar.set_width(width, visible);
        self.persist_structure();
        cx.notify();
        #[cfg(feature = "testkit")]
        {
            self.performance.arm_browser_state_if_pending(
                self.controller.read().snapshot().active_view,
                visible,
            );
            if self.performance.presentation_pending()() {
                self.commit_benchmark_sync(window);
            }
        }
    }

    /// Focus the resize divider (chrome domain) — the pointer entry into the
    /// divider and the focus anchor for the keyboard equivalents.
    pub fn focus_sidebar_divider(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.focus(&self.divider_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Production pointer entry: mouse-down on the divider starts a drag
    /// anchored at the pointer position and keeps chrome focus on the
    /// divider handle (the sidebar/root focus handlers would otherwise
    /// overwrite it — the render stops propagation after this call).
    pub fn begin_sidebar_drag(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_sidebar_divider(window, cx);
        self.sidebar.begin_drag(
            f32::from(event.position.x),
            f32::from(window.bounds().size.width),
        );
        cx.notify();
    }

    /// Production drag-move: while a drag is active, the width follows the
    /// pointer. Runs from the root's mouse-move listener so the drag keeps
    /// working outside the 4 pt hairline; a move without the left button
    /// held (button released outside the window) also ends the drag.
    pub fn on_sidebar_drag_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.sidebar.is_dragging() {
            return;
        }
        if event.pressed_button != Some(MouseButton::Left) {
            self.sidebar.end_drag();
            cx.notify();
            return;
        }
        self.sidebar
            .drag_to(f32::from(event.position.x), self.sidebar_visible);
        cx.notify();
    }

    /// Production drag-end: mouse-up anywhere in the window (or, via the
    /// out-listener, released outside it) commits the drag. The width was
    /// already applied during the move; this only clears the drag state.
    pub fn on_sidebar_drag_end(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar.is_dragging() {
            self.sidebar.end_drag();
            cx.notify();
        }
    }

    /// Keyboard equivalents for the focused divider (DESIGN §9: every
    /// pointer action has a keyboard equivalent): `←`/`→` resize by the
    /// step, `0` resets to the §8 default. Mutates through the same
    /// [`Self::resize_sidebar_to`] the pointer path uses, so parity holds.
    pub(super) fn on_sidebar_divider_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        match event.keystroke.key.as_str() {
            "left" => {
                self.resize_sidebar_to(self.sidebar.requested() - RESIZE_STEP, window, cx);
                cx.stop_propagation();
            }
            "right" => {
                self.resize_sidebar_to(self.sidebar.requested() + RESIZE_STEP, window, cx);
                cx.stop_propagation();
            }
            "0" => {
                self.resize_sidebar_to(SIDEBAR_DEFAULT, window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// Testkit pointer-drag driver: walks the production pointer listener
    /// methods in exactly the sequence the GPUI element listeners drive them
    /// (divider mouse-down → root mouse-move → root mouse-up). GPUI 0.2.2
    /// exposes no public mouse-event dispatch (`Window::dispatch_event`
    /// returns a crate-private type), so — exactly like the established
    /// ClickTab/ClickSpace pointer seams — the E2E drives the app methods
    /// the listeners call, never a direct controller mutation.
    #[cfg(feature = "testkit")]
    pub fn drag_sidebar_divider(&mut self, dx: f32, window: &mut Window, cx: &mut Context<Self>) {
        use bbr_ui::sidebar::geometry::RESIZE_HANDLE_WIDTH;
        use gpui::{point, px, Modifiers, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
        let width = self
            .sidebar
            .effective_width(f32::from(window.bounds().size.width));
        let center = (width - RESIZE_HANDLE_WIDTH / 2.0).max(0.0);
        let y = 100.0;
        let start = point(px(center), px(y));
        let end = point(px((center + dx).max(0.0)), px(y));
        let modifiers = Modifiers::default();
        self.begin_sidebar_drag(
            &MouseDownEvent {
                button: MouseButton::Left,
                position: start,
                modifiers,
                click_count: 1,
                first_mouse: true,
            },
            window,
            cx,
        );
        self.on_sidebar_drag_move(
            &MouseMoveEvent {
                position: end,
                pressed_button: Some(MouseButton::Left),
                modifiers,
            },
            window,
            cx,
        );
        self.on_sidebar_drag_end(
            &MouseUpEvent {
                button: MouseButton::Left,
                position: end,
                modifiers,
                click_count: 1,
            },
            window,
            cx,
        );
    }
}
