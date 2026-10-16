//! Blueprint D06 (board P-6): a sheet over the Settings content — New
//! Profile, Edit Profile, Delete Profile. The card sits near the top of
//! the window over a quiet wash: a title, the identity preview (the chip in
//! the chosen colour, the name, what a profile separates), labelled text
//! fields (the focused one with the accent edge and a caret), the colour
//! swatches, an optional message, then Cancel and the confirming button.

use gpui::{div, prelude::*, px, AnyElement, MouseButton, SharedString};

use crate::theme::Theme;

use super::{SettingsCallback, SettingsEvent, SettingsSheet};

/// The §15 id of the sheet card.
pub const SETTINGS_SHEET_ID: &str = "bbr-settings-sheet";
/// The sheet's `wash.active` veil.
pub const SETTINGS_SHEET_VEIL_ID: &str = "bbr-settings-sheet-veil";

fn press(
    events: &SettingsCallback,
    event: SettingsEvent,
) -> impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let events = events.clone();
    move |_e, window, cx| {
        cx.stop_propagation();
        events(event.clone(), window, cx);
    }
}

fn label(theme: &Theme, text: SharedString) -> AnyElement {
    div()
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_secondary)
        .child(text)
        .into_any_element()
}

/// The sheet overlay (backdrop plus card).
pub(super) fn sheet(theme: &Theme, sheet: &SettingsSheet, events: &SettingsCallback) -> AnyElement {
    let p = theme.palette;
    let mut body: Vec<AnyElement> = Vec::new();
    body.push(
        div()
            .text_size(theme.type_.title.size)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(p.ink)
            .child(sheet.title.clone())
            .into_any_element(),
    );
    if let Some(preview) = sheet.preview.as_ref() {
        body.push(
            div()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .size(px(44.0))
                        .rounded(px(8.0))
                        .bg(preview.hue)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(theme.type_.label.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.on_accent)
                        .child(preview.letter.to_string()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(p.ink)
                                .child(preview.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(p.ink_secondary)
                                .child(preview.note.clone()),
                        ),
                )
                .into_any_element(),
        );
    }
    for (index, field) in sheet.fields.iter().enumerate() {
        body.push(label(theme, field.label.clone()));
        body.push(
            div()
                .id(SharedString::from(format!(
                    "bbr-settings-sheet-field-{index}"
                )))
                .h(px(32.0))
                .px(px(10.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(if field.focused { p.accent } else { p.hairline })
                .bg(p.surface_field)
                .flex()
                .items_center()
                .text_size(theme.type_.label.size)
                .text_color(p.ink)
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    press(events, SettingsEvent::SheetField(index)),
                )
                .child(field.value.clone())
                .when(field.focused, |el| {
                    el.child(div().ml(px(1.0)).w(px(1.5)).h(px(16.0)).bg(p.accent))
                })
                .into_any_element(),
        );
        if index == 0 {
            if let Some((hues, chosen)) = sheet.swatches.as_ref() {
                body.push(label(theme, "Colour".into()));
                let letter = sheet.preview.as_ref().map_or('P', |preview| preview.letter);
                body.push(
                    div()
                        .flex()
                        .gap(px(10.0))
                        .children(hues.iter().enumerate().map(|(swatch, hue)| {
                            div()
                                .id(SharedString::from(format!(
                                    "bbr-settings-sheet-swatch-{swatch}"
                                )))
                                .size(px(28.0))
                                .rounded(px(7.0))
                                .border_2()
                                .border_color(if swatch == *chosen {
                                    p.accent
                                } else {
                                    gpui::transparent_black().into()
                                })
                                .p(px(2.0))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    press(events, SettingsEvent::SheetSwatch(swatch)),
                                )
                                .child(
                                    div()
                                        .size_full()
                                        .rounded(px(5.0))
                                        .bg(*hue)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(theme.type_.badge.size)
                                        .text_color(p.on_accent)
                                        .child(letter.to_string()),
                                )
                        }))
                        .into_any_element(),
                );
            }
        }
    }
    if let Some(message) = sheet.message.clone() {
        body.push(
            div()
                .text_size(theme.type_.label.size)
                .text_color(p.ink_secondary)
                .child(message)
                .into_any_element(),
        );
    }
    let confirm_color = if sheet.destructive {
        p.status_error
    } else {
        p.accent
    };
    body.push(
        div()
            .flex()
            .justify_end()
            .gap(px(8.0))
            .pt(px(6.0))
            .child(
                div()
                    .id("bbr-settings-sheet-cancel")
                    .h(px(28.0))
                    .px(px(14.0))
                    .rounded(px(14.0))
                    .border_1()
                    .border_color(p.hairline)
                    .bg(p.surface_field)
                    .flex()
                    .items_center()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, press(events, SettingsEvent::SheetCancel))
                    .child(sheet.cancel.clone()),
            )
            .child(
                div()
                    .id("bbr-settings-sheet-confirm")
                    .h(px(28.0))
                    .px(px(14.0))
                    .rounded(px(14.0))
                    .bg(confirm_color)
                    .opacity(if sheet.confirm_enabled { 1.0 } else { 0.45 })
                    .flex()
                    .items_center()
                    .text_size(theme.type_.label.size)
                    .text_color(p.on_accent)
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        press(events, SettingsEvent::SheetConfirm),
                    )
                    .child(sheet.confirm.clone()),
            )
            .into_any_element(),
    );
    div()
        .absolute()
        .inset_0()
        .bg(p.wash_active)
        .flex()
        // Patrick (2026-10-12): every blocking sheet sits centred on both
        // axes over its veil, at its content height (without the cross-axis
        // rule flex would stretch it to the whole veil).
        .justify_center()
        .items_center()
        // Testkit-only: the veil's drawn rect (`sheet_rects`).
        .children(crate::rect_probe::rect_probe(SETTINGS_SHEET_VEIL_ID))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .id(SETTINGS_SHEET_ID)
                // Testkit-only: the drawn sheet rect (`sheet_rects`).
                .children(crate::rect_probe::rect_probe(SETTINGS_SHEET_ID))
                .w(px(420.0))
                .h_auto()
                .p(px(20.0))
                .rounded(px(12.0))
                .bg(p.surface_overlay)
                .border_1()
                .border_color(p.hairline)
                .shadow(crate::theme::elevation::elev2(theme))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(body),
        )
        .into_any_element()
}
