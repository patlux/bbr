//! Blueprint D13 (board M-12, DESIGN §11.13): "Open in Zoom?" — the sheet
//! that asks before a page's link leaves the browser for another app.
//!
//! A 440 pt sheet at the card's top edge over the 9 % wash: the question,
//! which site wants to open which app, "Always allow zoom.us to open Zoom
//! links" (only when a site asked), then Cancel and the primary "Open
//! Zoom".

use std::rc::Rc;

use gpui::{div, prelude::*, px, App, MouseButton, SharedString, Window};

use crate::surfaces::permissions::{checkbox, pill_button};
use crate::theme::Theme;

pub const OPEN_APP_ID: &str = "bbr-external-open";
/// The sheet's `wash.active` veil.
pub const OPEN_APP_VEIL_ID: &str = "bbr-external-open-veil";

/// The sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenAppModel {
    /// "Open in Zoom?"
    pub title: SharedString,
    /// "zoom.us wants to open the Zoom app."
    pub message: SharedString,
    /// "Always allow zoom.us to open Zoom links" (`None` without a site).
    pub remember_label: Option<SharedString>,
    pub remember: bool,
    /// "Open Zoom".
    pub confirm: SharedString,
}

/// The sheet's controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAppEvent {
    Remember,
    Cancel,
    Open,
}

pub type OpenAppCallback = Rc<dyn Fn(OpenAppEvent, &mut Window, &mut App)>;

/// The veil and the sheet over the card (`card` in window points).
pub fn open_app_sheet(
    theme: &Theme,
    model: &OpenAppModel,
    card: [f32; 4],
    events: OpenAppCallback,
) -> impl IntoElement {
    let p = theme.palette;
    let [x, y, width, height] = card;
    let send = |event: OpenAppEvent| {
        let events = events.clone();
        move |window: &mut Window, cx: &mut App| events(event, window, cx)
    };
    let remember = model.remember_label.clone().map(|label| {
        let events = events.clone();
        div()
            .id("bbr-external-open-remember")
            .flex()
            .items_center()
            .gap(px(8.0))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                cx.stop_propagation();
                events(OpenAppEvent::Remember, window, cx);
            })
            .child(checkbox(theme, model.remember))
            .child(
                div()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .child(label),
            )
    });
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(width))
        .h(px(height))
        .bg(p.wash_active)
        .flex()
        // Patrick (2026-10-12): every blocking sheet sits centred on both
        // axes over its veil, at its content height (without the cross-axis
        // rule flex would stretch it to the whole veil).
        .justify_center()
        .items_center()
        // Testkit-only: the veil's drawn rect (`sheet_rects`).
        .children(crate::rect_probe::rect_probe(OPEN_APP_VEIL_ID))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .id(OPEN_APP_ID)
                // Testkit-only: the drawn sheet rect (`sheet_rects`).
                .children(crate::rect_probe::rect_probe(OPEN_APP_ID))
                .w(px(440.0))
                .h_auto()
                .p(px(20.0))
                .rounded(px(12.0))
                .bg(p.surface_overlay)
                .border_1()
                .border_color(p.hairline)
                .shadow(crate::theme::elevation::elev2(theme))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    div()
                        .text_size(theme.type_.title.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink)
                        .child(model.title.clone()),
                )
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(p.ink_secondary)
                        .child(model.message.clone()),
                )
                .children(remember)
                .child(
                    div()
                        .pt(px(4.0))
                        .flex()
                        .justify_end()
                        .gap(px(8.0))
                        .child(pill_button(
                            theme,
                            "bbr-external-open-cancel".into(),
                            "Cancel",
                            false,
                            send(OpenAppEvent::Cancel),
                        ))
                        .child(pill_button(
                            theme,
                            "bbr-external-open-confirm".into(),
                            &model.confirm,
                            true,
                            send(OpenAppEvent::Open),
                        )),
                ),
        )
}
