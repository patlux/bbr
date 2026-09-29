//! Blueprint D23: the collapsed split's "n of N" chip and its pane list in
//! the Portal's overlay layer (they sit over the page). Their laid-out
//! bounds become Portal hit regions (see `BbrApp::split_hit_rects`).

use std::rc::Rc;

use gpui::{canvas, div, prelude::*, px, AnyElement, Context};

use bbr_ui::surfaces::split_chip::{split_chip, split_list, SPLIT_CHIP_HEIGHT, SPLIT_CHIP_INSET};

use super::probe::PortalRoot;

/// Record `bounds` of the element it is laid over; a change re-syncs the
/// Portal regions once.
fn recorder(
    bounds: Rc<std::cell::Cell<Option<[f32; 4]>>>,
    app: gpui::Entity<crate::app::BbrApp>,
    browser_window: gpui::WindowHandle<gpui_component::Root>,
) -> impl IntoElement {
    canvas(
        move |item, _window, cx| {
            let rect = [
                f32::from(item.origin.x),
                f32::from(item.origin.y),
                f32::from(item.size.width),
                f32::from(item.size.height),
            ];
            if bounds.get() != Some(rect) {
                bounds.set(Some(rect));
                let app = app.clone();
                cx.defer(move |cx| {
                    let _ =
                        crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                            view.sync_portal_overlays(cx)
                        });
                });
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
}

/// The chip (and list) while the shown group rests collapsed.
pub(super) fn render(root: &PortalRoot, cx: &mut Context<PortalRoot>) -> Option<AnyElement> {
    let app = root.app.clone()?;
    let browser_window = root.browser_window;
    let (theme, card, model, chip_bounds, list_bounds) = {
        let app = app.read(cx);
        (
            app.theme(),
            app.transient.card?,
            app.split_chip_model()?,
            app.split_view.chip_bounds.clone(),
            app.split_view.list_bounds.clone(),
        )
    };
    let press_app = app.clone();
    let on_press: bbr_ui::surfaces::split_chip::SplitChipCallback =
        Rc::new(move |row, _window, cx| {
            let app = press_app.clone();
            cx.defer(move |cx| {
                let _ = crate::window_seam::with_app(&browser_window, &app, cx, |view, _w, cx| {
                    view.split_chip_press(row, cx)
                });
            });
        });
    let [x, y, width, _] = card;
    let chip = split_chip(&theme, &model, on_press.clone()).child(recorder(
        chip_bounds,
        app.clone(),
        browser_window,
    ));
    let list = split_list(&theme, &model, on_press)
        .map(|list| list.child(recorder(list_bounds, app.clone(), browser_window)));
    Some(
        div()
            .absolute()
            .top(px(y + SPLIT_CHIP_INSET))
            .left(px(x))
            .w(px(width - SPLIT_CHIP_INSET))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(6.0))
            .child(div().h(px(SPLIT_CHIP_HEIGHT)).child(chip))
            .children(list)
            .into_any_element(),
    )
}

/// The drop preview while a rail tab drag hovers a zone of the card: a 2 pt
/// `accent` frame with a 12 % `accent` fill and the split mark (board G-10).
/// It never takes the pointer (the drag belongs to the browser window).
pub(super) fn render_drop(root: &PortalRoot, cx: &mut Context<PortalRoot>) -> Option<AnyElement> {
    let app = root.app.clone()?;
    let app = app.read(cx);
    let drop = app.split_view.drop?;
    let theme = app.theme();
    let [x, y, w, h] = drop.rect;
    let mut fill = theme.palette.accent;
    fill.a = 0.12;
    Some(
        div()
            .id("bbr-split-zone")
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(w))
            .h(px(h))
            .rounded(px(12.0))
            .border_2()
            .border_color(theme.palette.accent)
            .bg(fill)
            .flex()
            .items_center()
            .justify_center()
            .child(bbr_ui::icons::Icon::Split.canvas_with(&theme, px(20.0), theme.palette.accent))
            .into_any_element(),
    )
}
