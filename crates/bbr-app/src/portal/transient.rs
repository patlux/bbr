//! Blueprint D08: the Portal's overlay layer — the toast (bottom-centre of
//! the card) and the zoom readout (top-centre) above any active
//! presentation. The toast hugs its content, so its rendered bounds are
//! recorded here and handed back to the app, which converges the Portal's
//! hit regions (see `crate::app::transient`).

use gpui::{canvas, div, prelude::*, px, AnyElement, Context};

use bbr_ui::surfaces::toast::{toast, toast_rect, zoom_readout, zoom_readout_rect, TOAST_HEIGHT};

use super::probe::PortalRoot;

/// The overlay elements for the current app state (empty at rest).
pub(super) fn render_overlays(root: &PortalRoot, cx: &mut Context<PortalRoot>) -> Vec<AnyElement> {
    let Some(app) = root.app.clone() else {
        return Vec::new();
    };
    let (theme, card, toast_state, readout, bounds) = {
        let app = app.read(cx);
        (
            app.theme(),
            app.transient.card,
            app.transient.toast.clone(),
            app.transient.zoom_readout,
            app.transient.toast_bounds.clone(),
        )
    };
    let Some(card) = card else {
        return Vec::new();
    };
    let browser_window = root.browser_window;
    let mut overlays = Vec::new();
    if let Some(state) = toast_state {
        let [x, y, width, _] = toast_rect(card, card[2]);
        let action_app = app.clone();
        let element = toast(&theme, &state.model, move |_event, _window, cx| {
            let app = action_app.clone();
            cx.defer(move |cx| {
                let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                    view.toast_action(cx)
                });
            });
        });
        // Records the toast's laid-out bounds (window points) for its hit
        // region; a change re-syncs the Portal once.
        let record_app = app.clone();
        let recorder = canvas(
            move |item_bounds, _window, cx| {
                let rect = [
                    f32::from(item_bounds.origin.x),
                    f32::from(item_bounds.origin.y),
                    f32::from(item_bounds.size.width),
                    f32::from(item_bounds.size.height),
                ];
                if bounds.get() != Some(rect) {
                    bounds.set(Some(rect));
                    let app = record_app.clone();
                    cx.defer(move |cx| {
                        let _ = crate::window_seam::with_app(
                            &browser_window,
                            &app,
                            cx,
                            |view, _window, cx| view.sync_portal_overlays(cx),
                        );
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(width))
                .h(px(TOAST_HEIGHT))
                .flex()
                .justify_center()
                .child(div().relative().child(element).child(recorder))
                .into_any_element(),
        );
    }
    if let Some(readout) = readout {
        let [x, y, _, _] = zoom_readout_rect(card);
        let minus_app = app.clone();
        let plus_app = app.clone();
        overlays.push(
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .child(zoom_readout(
                    &theme,
                    readout.percent,
                    readout.generation,
                    move |_event, _window, cx| {
                        let app = minus_app.clone();
                        cx.defer(move |cx| {
                            let _ = crate::window_seam::with_app(
                                &browser_window,
                                &app,
                                cx,
                                |view, _w, cx| view.zoom_out(cx),
                            );
                        });
                    },
                    move |_event, _window, cx| {
                        let app = plus_app.clone();
                        cx.defer(move |cx| {
                            let _ = crate::window_seam::with_app(
                                &browser_window,
                                &app,
                                cx,
                                |view, _w, cx| view.zoom_in(cx),
                            );
                        });
                    },
                ))
                .into_any_element(),
        );
    }
    overlays
}
