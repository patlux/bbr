//! Blueprint D14: the portal render of the Back/Forward history menu — the
//! §11.13 menu card over web content with the page menu's one-pointer
//! backdrop dismissal — and its row paths into the production app methods.

use gpui::{
    div, prelude::*, transparent_black, AnyElement, App, Context, MouseButton, MouseDownEvent,
    Window,
};

use super::geometry::Rect;
use super::probe::PortalRoot;

impl PortalRoot {
    fn nav_history_call(
        &self,
        cx: &mut Context<Self>,
        call: impl FnOnce(&mut crate::app::BbrApp, &mut Window, &mut Context<crate::app::BbrApp>)
            + 'static,
    ) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, call);
        });
    }
}

pub(super) fn render(root: &PortalRoot, rect: Rect, cx: &mut Context<PortalRoot>) -> AnyElement {
    let Some(app) = root.app.as_ref() else {
        return div().size_full().bg(transparent_black()).into_any();
    };
    let theme = app.read(cx).theme();
    let model = app.read(cx).nav_history_model();
    let portal_root = cx.entity().downgrade();
    let on_row_hover = {
        let portal_root = portal_root.clone();
        move |index: usize, _hovered: &bool, _window: &mut Window, cx: &mut App| {
            if let Some(root) = portal_root.upgrade() {
                root.update(cx, |this, cx| {
                    this.nav_history_call(cx, move |view, _window, cx| {
                        view.nav_history_select(index, cx)
                    })
                });
            }
        }
    };
    let on_row_press =
        move |index: usize, _e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
            if let Some(root) = portal_root.upgrade() {
                root.update(cx, |this, cx| {
                    this.nav_history_call(cx, move |view, window, cx| {
                        view.nav_history_activate(index, window, cx)
                    })
                });
            }
        };
    let card = bbr_ui::surfaces::menu::menu_card(
        &theme,
        bbr_ui::surfaces::menu::NAV_HISTORY_MENU_ID,
        &model,
        bbr_ui::surfaces::menu::ContextMenuRect::new(rect.x, rect.y, rect.width, rect.height),
        on_row_hover,
        on_row_press,
    )
    .on_mouse_down(
        MouseButton::Left,
        cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
            cx.stop_propagation();
        }),
    );
    div()
        .id(crate::app::nav_history_ids::BACKDROP)
        .size_full()
        .bg(transparent_black())
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                this.nav_history_call(cx, |view, window, cx| view.nav_history_close(window, cx));
            }),
        )
        .child(card)
        .into_any()
}
