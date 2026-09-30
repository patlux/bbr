//! App-side titlebar composition (task packet `titlebar-navigation-controls`):
//! the plain [`TitlebarModel`] builder, the presentation-state methods
//! (hover/pressed), and the native empty-titlebar drag entry.
//!
//! The titlebar row renders inside the expanded full-height Sidebar rail
//! (the `bbr-ui` composition owns the row); this module owns the app
//! authority: the safe-area geometry comes from the live native window
//! (`WebEngineWk::safe_titlebar_guide` — never hardcoded traffic-light
//! coordinates), the Back/Forward/Reload enablement comes from the
//! controller snapshot's real navigation state (the same authority the
//! web-content gesture packet and the context menus consume), and a press
//! on the empty titlebar space starts the native window drag through GPUI's
//! `start_window_move` (with the system double-click action on a double
//! click).

use bbr_ui::titlebar::{SafeArea, TitlebarControl, TitlebarModel};
use gpui::{Context, MouseDownEvent, Window};

use super::BbrApp;

impl BbrApp {
    /// The native safe-titlebar layout guide as plain presentation data:
    /// the strip height and the traffic-light region read from the live
    /// window (`None`/zero-height before the window attaches or on
    /// non-macOS hosts). The row renders only what this reports.
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))]
    pub fn safe_area(&self, window: &Window) -> SafeArea {
        let native = self.native_safe_area(window);
        // Blueprint D22: full screen hides the traffic lights; the row keeps
        // its controls without their clearance.
        if self.is_full_screen(window) {
            SafeArea::full_screen(native.height)
        } else {
            native
        }
    }

    fn native_safe_area(&self, _window: &Window) -> SafeArea {
        #[cfg(target_os = "macos")]
        {
            let guide = self
                .geometry_engine
                .as_ref()
                .and_then(|engine| engine.borrow().safe_titlebar_guide());
            guide.map_or_else(SafeArea::none, |guide| SafeArea {
                height: guide.height,
                traffic_lights: [
                    guide.traffic_lights.x,
                    guide.traffic_lights.y,
                    guide.traffic_lights.width,
                    guide.traffic_lights.height,
                ],
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
            SafeArea::none()
        }
    }

    /// The §11.14 titlebar model: the safe-area geometry (native), the
    /// sidebar collapse state, and the genuine navigation availability of
    /// the focused pane — all plain presentation data for the `bbr-ui`
    /// row. No browser/navigation authority lives here.
    pub fn titlebar_model(&self, window: &Window) -> TitlebarModel {
        let browser = self.controller.read().snapshot();
        TitlebarModel {
            safe_area: self.safe_area(window),
            sidebar_collapsed: !self.sidebar_visible,
            can_go_back: browser.nav.can_go_back,
            can_go_forward: browser.nav.can_go_forward,
            // Reload targets the focused command pane: enabled while it has
            // a live session.
            can_reload: browser.active_view.is_some(),
            hovered: self.titlebar_hovered,
            pressed: self.titlebar_pressed,
        }
    }

    /// DESIGN.md §10.2 production hover path for the titlebar controls:
    /// the pointer entered `control` (`hovered == true`) or left it.
    /// The exact app method the row's `on_hover` listener calls.
    pub fn set_titlebar_hover(
        &mut self,
        control: TitlebarControl,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        let changed = if hovered {
            self.titlebar_hovered != Some(control)
        } else {
            self.titlebar_hovered == Some(control)
        };
        self.titlebar_hovered = if hovered { Some(control) } else { None };
        if changed {
            cx.notify();
        }
    }

    /// Clear the titlebar hover state regardless of the control for the
    /// typed `UnhoverTitlebarControl` testkit seam. Production pointer leave
    /// is delivered through `set_titlebar_hover(control, false)`.
    #[cfg(feature = "testkit")]
    pub fn clear_titlebar_hover(&mut self, cx: &mut Context<Self>) {
        if self.titlebar_hovered.take().is_some() {
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production press path: the primary button went down
    /// on a titlebar control. The exact app method the row's mouse-down
    /// listener calls through the callbacks press slot.
    pub fn set_titlebar_pressed(&mut self, control: TitlebarControl, cx: &mut Context<Self>) {
        if self.titlebar_pressed != Some(control) {
            self.titlebar_pressed = Some(control);
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production release path: the primary button went up
    /// (or left the window). The exact app method the row's release slot
    /// and the root's mouse-up-out listener call.
    pub fn clear_titlebar_pressed(&mut self, cx: &mut Context<Self>) {
        if self.titlebar_pressed.take().is_some() {
            cx.notify();
        }
    }

    /// §11.14 empty-titlebar drag entry: a primary-button press on the
    /// empty titlebar space starts the native window drag (GPUI's
    /// `start_window_move` performs `performWindowDragWithEvent:`), and a
    /// double click runs the system titlebar double-click action (zoom /
    /// minimize per the user's "AppleActionOnDoubleClick" preference).
    /// Controls never reach this handler (their own listeners consume the
    /// press inside their hit targets).
    pub fn on_titlebar_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.click_count >= 2 {
            window.titlebar_double_click();
        } else {
            window.start_window_move();
        }
        cx.notify();
    }
}
