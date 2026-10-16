//! Blueprint D12 (board M-11, DESIGN §18.4): the client-certificate sheet.
//!
//! A 440 pt sheet at the card's top edge over the 9 % wash: "Choose a
//! certificate", which server asks and for which profile, the matching
//! Keychain identities (name; issuer and expiry — expired ones stay
//! visible, dimmed, and cannot be chosen), "Remember for this site", then
//! Don't Send, Cancel and Continue.

use std::rc::Rc;

use gpui::{div, prelude::*, px, AnyElement, App, MouseButton, SharedString, Window};

use crate::icons::Icon;
use crate::surfaces::permissions::{checkbox, pill_button};
use crate::theme::Theme;

pub const CLIENT_CERTIFICATE_ID: &str = "bbr-client-certificate";
/// The sheet's `wash.active` veil.
pub const CLIENT_CERTIFICATE_VEIL_ID: &str = "bbr-client-certificate-veil";

/// One identity row.
#[derive(Debug, Clone, PartialEq)]
pub struct CertificateRow {
    pub name: SharedString,
    /// "Corp Issuing CA 2 · expires 3 Mar 2027".
    pub detail: SharedString,
    pub expired: bool,
}

/// The sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientCertificateModel {
    /// "vpn.corp.example asks for a certificate to identify you. Profile ·
    /// Work."
    pub message: SharedString,
    pub rows: Vec<CertificateRow>,
    pub selected: Option<usize>,
    pub remember: bool,
}

/// The sheet's controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientCertificateEvent {
    Select(usize),
    Remember,
    DontSend,
    Cancel,
    Continue,
}

pub type ClientCertificateCallback = Rc<dyn Fn(ClientCertificateEvent, &mut Window, &mut App)>;

fn send(
    events: &ClientCertificateCallback,
    event: ClientCertificateEvent,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let events = events.clone();
    move |window, cx| events(event, window, cx)
}

fn row(
    theme: &Theme,
    index: usize,
    row: &CertificateRow,
    selected: bool,
    events: &ClientCertificateCallback,
) -> AnyElement {
    let p = theme.palette;
    let ink = if row.expired { p.ink_faint } else { p.ink };
    let events = events.clone();
    div()
        .id(SharedString::from(format!(
            "bbr-client-certificate-row-{index}"
        )))
        .px(px(10.0))
        .py(px(7.0))
        .rounded(px(8.0))
        .flex()
        .items_center()
        .gap(px(10.0))
        .border_1()
        .border_color(if selected { p.accent } else { p.hairline })
        .when(selected, |el| el.bg(p.tint_accent))
        .when(!row.expired, |el| {
            el.cursor_pointer()
                .hover(|el| el.bg(p.wash_hover))
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    cx.stop_propagation();
                    events(ClientCertificateEvent::Select(index), window, cx);
                })
        })
        .child(Icon::Person.canvas_with(theme, px(16.0), ink))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(ink)
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(row.name.clone()),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(if row.expired {
                            p.ink_faint
                        } else {
                            p.ink_secondary
                        })
                        .child(row.detail.clone()),
                ),
        )
        .into_any_element()
}

/// The veil and the sheet over the card (`card` = x, y, width, height in
/// the Portal's window points).
pub fn client_certificate_sheet(
    theme: &Theme,
    model: &ClientCertificateModel,
    card: [f32; 4],
    events: ClientCertificateCallback,
) -> impl IntoElement {
    let p = theme.palette;
    let [x, y, width, height] = card;
    let remember_events = events.clone();
    let rows = model
        .rows
        .iter()
        .enumerate()
        .map(|(index, r)| row(theme, index, r, model.selected == Some(index), &events));
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
        .children(crate::rect_probe::rect_probe(CLIENT_CERTIFICATE_VEIL_ID))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .id(CLIENT_CERTIFICATE_ID)
                // Testkit-only: the drawn sheet rect (`sheet_rects`).
                .children(crate::rect_probe::rect_probe(CLIENT_CERTIFICATE_ID))
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
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(theme.type_.title.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink)
                        .child("Choose a certificate"),
                )
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(p.ink_secondary)
                        .child(model.message.clone()),
                )
                .child(div().flex().flex_col().gap(px(6.0)).children(rows))
                .child(
                    div()
                        .id("bbr-client-certificate-remember")
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            cx.stop_propagation();
                            remember_events(ClientCertificateEvent::Remember, window, cx);
                        })
                        .child(checkbox(theme, model.remember))
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(p.ink)
                                .child("Remember for this site"),
                        ),
                )
                .child(
                    div()
                        .pt(px(4.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .id("bbr-client-certificate-none")
                                .px(px(8.0))
                                .py(px(3.0))
                                .rounded(px(6.0))
                                .text_size(theme.type_.label.size)
                                .text_color(p.accent)
                                .hover(|el| el.bg(p.wash_hover))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, {
                                    let dont = send(&events, ClientCertificateEvent::DontSend);
                                    move |_e, window, cx| {
                                        cx.stop_propagation();
                                        dont(window, cx);
                                    }
                                })
                                .child("Don’t Send"),
                        )
                        .child(div().flex_1())
                        .child(pill_button(
                            theme,
                            "bbr-client-certificate-cancel".into(),
                            "Cancel",
                            false,
                            send(&events, ClientCertificateEvent::Cancel),
                        ))
                        .child(
                            div()
                                .opacity(if model.selected.is_some() { 1.0 } else { 0.45 })
                                .child(pill_button(
                                    theme,
                                    "bbr-client-certificate-continue".into(),
                                    "Continue",
                                    true,
                                    send(&events, ClientCertificateEvent::Continue),
                                )),
                        ),
                ),
        )
}
