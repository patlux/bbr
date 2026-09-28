//! Blueprint D05: the portal render of the Delete Space confirmation
//! sheet — the veil over the content card and the sheet attached to the
//! card's top edge — plus its button paths into the production app
//! methods.

use gpui::{
    div, prelude::*, px, transparent_black, AnyElement, Context, MouseButton, MouseDownEvent,
};

use super::geometry::Rect;
use super::probe::PortalRoot;

impl PortalRoot {
    /// Blueprint D05: the Delete Space sheet's Cancel button path.
    fn space_delete_cancel_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.space_delete_cancel(window, cx)
            });
        });
    }

    /// Blueprint D05: the Delete Space sheet's danger-button path.
    fn space_delete_confirm_clicked(&self, cx: &mut Context<Self>) {
        let Some(app) = self.app.clone() else {
            return;
        };
        let browser_window = self.browser_window;
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, window, cx| {
                view.space_delete_confirm(window, cx)
            });
        });
    }
}

/// The sheet surface for the resolved sheet `rect`. A blocking sheet: the
/// backdrop stops every press (no outside dismissal).
pub(super) fn render(root: &PortalRoot, rect: Rect, cx: &mut Context<PortalRoot>) -> AnyElement {
    let Some(app) = root.app.as_ref() else {
        return div().size_full().bg(transparent_black()).into_any();
    };
    let Some((model, card)) = app.read(cx).space_delete_surface() else {
        return div().size_full().bg(transparent_black()).into_any();
    };
    let theme = app.read(cx).theme();
    // Board P-4: the veil dims the content card only (never the rail) with
    // the `wash.active` wash; the sheet attaches to the card's top edge.
    let veil = div()
        .id(bbr_ui::surfaces::space_delete::SPACE_DELETE_VEIL_ID)
        .absolute()
        .left(px(card.x))
        .top(px(card.y))
        .w(px(card.width))
        .h(px(card.height))
        .rounded(theme.radius.large())
        .bg(theme.palette.wash_active);
    let sheet = div()
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .child(bbr_ui::surfaces::space_delete::space_delete_sheet(
            &theme,
            &model,
            cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                this.space_delete_cancel_clicked(cx);
            }),
            cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                this.space_delete_confirm_clicked(cx);
            }),
        ));
    div()
        .id("bbr-space-delete-portal-surface")
        .size_full()
        .bg(transparent_black())
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|_this, _e: &MouseDownEvent, _window, cx| {
                cx.stop_propagation();
            }),
        )
        .child(veil)
        .child(sheet)
        .into_any()
}
