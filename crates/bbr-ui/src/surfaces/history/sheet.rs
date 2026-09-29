//! Blueprint D10 (board M-7): the Clear Browsing Data sheet on the
//! History page.

use gpui::{div, prelude::*, px, AnyElement, MouseButton, SharedString};

use super::{press, ClearDataModel, HistoryCallback, HistoryEvent, CLEAR_DATA_ID};
use crate::icons::Icon;
use crate::theme::Theme;

fn sheet_button(
    theme: &Theme,
    id: &'static str,
    label: &'static str,
    destructive: bool,
    enabled: bool,
    event: HistoryEvent,
    events: &HistoryCallback,
) -> AnyElement {
    let p = theme.palette;
    div()
        .id(id)
        .h(px(28.0))
        .px(px(14.0))
        .rounded(px(14.0))
        .border_1()
        .border_color(p.hairline)
        .bg(p.surface_field)
        .flex()
        .items_center()
        .text_size(theme.type_.label.size)
        .text_color(if !enabled {
            p.ink_faint
        } else if destructive {
            p.status_error
        } else {
            p.ink
        })
        .when(enabled, |el| {
            el.cursor_pointer()
                .on_mouse_down(MouseButton::Left, press(events, event))
        })
        .child(label)
        .into_any_element()
}

pub(super) fn clear_sheet(
    theme: &Theme,
    clear: &ClearDataModel,
    events: &HistoryCallback,
) -> AnyElement {
    let p = theme.palette;
    let range_label = clear.ranges.get(clear.range).cloned().unwrap_or_default();
    let menu = clear.range_open.then(|| {
        div()
            .id("bbr-clear-data-range-menu")
            .absolute()
            .right(px(0.0))
            .top(px(32.0))
            .w(px(170.0))
            .p(px(4.0))
            .rounded(px(8.0))
            .bg(p.surface_overlay)
            .border_1()
            .border_color(p.hairline)
            .shadow(crate::theme::elevation::elev2(theme))
            .children(clear.ranges.iter().enumerate().map(|(index, label)| {
                div()
                    .id(SharedString::from(format!("bbr-clear-data-range-{index}")))
                    .px(px(10.0))
                    .py(px(4.0))
                    .rounded(px(5.0))
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .when(index == clear.range, |el| el.bg(p.wash_active))
                    .hover(|el| el.bg(p.wash_hover))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        press(events, HistoryEvent::ClearRange(index)),
                    )
                    .child(label.clone())
            }))
    });
    let options = clear.options.iter().enumerate().map(|(index, option)| {
        let ink = if option.enabled { p.ink } else { p.ink_faint };
        div()
            .id(SharedString::from(format!("bbr-clear-data-option-{index}")))
            .flex()
            .items_center()
            .gap(px(8.0))
            .when(option.enabled, |el| {
                el.cursor_pointer().on_mouse_down(
                    MouseButton::Left,
                    press(events, HistoryEvent::ClearToggle(index)),
                )
            })
            .child(
                div()
                    .size(px(16.0))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(if option.checked { p.accent } else { p.hairline })
                    .bg(if option.checked {
                        p.accent
                    } else {
                        p.surface_field
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(option.checked, |el| {
                        el.child(Icon::Check.canvas_with(theme, px(11.0), p.on_accent))
                    }),
            )
            .child(
                div()
                    .text_size(theme.type_.label.size)
                    .text_color(ink)
                    .child(option.label.clone()),
            )
    });
    div()
        .absolute()
        .inset_0()
        .bg(p.wash_active)
        .flex()
        .justify_center()
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .id(CLEAR_DATA_ID)
                .mt(px(12.0))
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
                .gap(px(14.0))
                .child(
                    div()
                        .text_size(theme.type_.title.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink)
                        .child("Clear Browsing Data"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .size(px(22.0))
                                .rounded(px(6.0))
                                .bg(clear.profile_hue)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(theme.type_.badge.size)
                                .text_color(p.on_accent)
                                .child(clear.profile_letter.to_string()),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(p.ink_secondary)
                                .child(SharedString::from(format!("Profile · {}", clear.profile))),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(p.ink)
                                .child("Time range"),
                        )
                        .child(
                            div()
                                .relative()
                                .child(
                                    div()
                                        .id("bbr-clear-data-range")
                                        .w(px(170.0))
                                        .h(px(28.0))
                                        .px(px(10.0))
                                        .rounded(px(8.0))
                                        .border_1()
                                        .border_color(p.hairline)
                                        .bg(p.surface_field)
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_size(theme.type_.label.size)
                                        .text_color(p.ink)
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            press(events, HistoryEvent::ClearRangeMenu),
                                        )
                                        .child(range_label)
                                        .child(
                                            Icon::Disclosure { expanded: true }
                                                .canvas(theme, px(12.0)),
                                        ),
                                )
                                .children(menu.map(|menu| gpui::deferred(menu).with_priority(1))),
                        ),
                )
                .child(div().flex().flex_col().gap(px(10.0)).children(options))
                .child(
                    div()
                        .pt(px(4.0))
                        .flex()
                        .justify_end()
                        .gap(px(8.0))
                        .child(sheet_button(
                            theme,
                            "bbr-clear-data-cancel",
                            "Cancel",
                            false,
                            true,
                            HistoryEvent::ClearCancel,
                            events,
                        ))
                        .child(sheet_button(
                            theme,
                            "bbr-clear-data-confirm",
                            "Clear Data",
                            true,
                            clear.confirm_enabled,
                            HistoryEvent::ClearConfirm,
                            events,
                        )),
                ),
        )
        .into_any_element()
}
