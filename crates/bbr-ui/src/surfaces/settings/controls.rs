//! The Settings controls (boards L-3…L-9): switch, pop-up choice,
//! segmented choice, push button, text button, keycaps, status badge,
//! density cards and the profile chip. Each reports through the one
//! [`SettingsCallback`] with its row id.

use gpui::{
    anchored, deferred, div, prelude::*, px, Anchor, AnyElement, MouseButton, MouseDownEvent,
    SharedString,
};

use crate::icons::Icon;
use crate::theme::Theme;

use super::{SettingsCallback, SettingsControl, SettingsEvent};

fn on_press(
    events: &SettingsCallback,
    event: SettingsEvent,
) -> impl Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let events = events.clone();
    move |_e, window, cx| {
        cx.stop_propagation();
        events(event.clone(), window, cx);
    }
}

/// The control for `row` (`open`: its pop-up choice is open).
pub(super) fn control(
    theme: &Theme,
    row: &SharedString,
    control: &SettingsControl,
    open: bool,
    events: &SettingsCallback,
) -> AnyElement {
    let p = theme.palette;
    match control {
        SettingsControl::None => div().into_any_element(),
        SettingsControl::Badge(text) => div()
            .px(px(8.0))
            .py(px(2.0))
            .rounded(px(10.0))
            .bg(p.tint_accent)
            .text_size(theme.type_.badge.size)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(p.accent_ink)
            .child(text.to_uppercase())
            .into_any_element(),
        SettingsControl::Toggle(on) => {
            let on = *on;
            div()
                .id(SharedString::from(format!("{row}-toggle")))
                .w(px(30.0))
                .h(px(18.0))
                .rounded(px(9.0))
                .bg(if on { p.accent } else { p.hairline })
                .p(px(2.0))
                .flex()
                .when(on, |el| el.justify_end())
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    on_press(events, SettingsEvent::Toggle(row.clone())),
                )
                .child(div().size(px(14.0)).rounded(px(7.0)).bg(p.on_accent))
                .into_any_element()
        }
        SettingsControl::Select { value, options } => {
            let label = options.get(*value).cloned().unwrap_or_default();
            let menu = open.then(|| {
                let rows = options.iter().enumerate().map(|(index, option)| {
                    div()
                        .id(SharedString::from(format!("{row}-option-{index}")))
                        .px(px(10.0))
                        .py(px(4.0))
                        .rounded(px(5.0))
                        .flex()
                        .gap(px(6.0))
                        .items_center()
                        .text_size(theme.type_.label.size)
                        .text_color(p.ink)
                        .hover(|el| el.bg(p.tint_accent))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            on_press(events, SettingsEvent::Choose(row.clone(), index)),
                        )
                        .child(div().w(px(12.0)).when(index == *value, |el| {
                            el.child(Icon::Check.canvas(theme, px(12.0)))
                        }))
                        .child(option.clone())
                });
                deferred(
                    anchored().anchor(Anchor::TopRight).child(
                        div()
                            .id(SharedString::from(format!("{row}-menu")))
                            .mt(px(30.0))
                            .min_w(px(160.0))
                            .p(px(4.0))
                            .rounded(px(8.0))
                            .bg(p.surface_overlay)
                            .border_1()
                            .border_color(p.hairline)
                            .shadow(crate::theme::elevation::elev2(theme))
                            .flex()
                            .flex_col()
                            .children(rows),
                    ),
                )
                .with_priority(1)
            });
            div()
                .relative()
                .child(
                    div()
                        .id(SharedString::from(format!("{row}-select")))
                        .min_w(px(140.0))
                        .h(px(28.0))
                        .px(px(10.0))
                        .rounded(px(7.0))
                        .border_1()
                        .border_color(p.hairline)
                        .bg(p.surface_field)
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(8.0))
                        .text_size(theme.type_.label.size)
                        .text_color(p.ink)
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            on_press(events, SettingsEvent::OpenSelect(row.clone())),
                        )
                        .child(label)
                        .child(Icon::Disclosure { expanded: true }.canvas(theme, px(12.0))),
                )
                .children(menu)
                .into_any_element()
        }
        SettingsControl::Segmented { value, options } => div()
            .p(px(2.0))
            .rounded(px(8.0))
            .bg(p.wash_hover)
            .flex()
            .children(options.iter().enumerate().map(|(index, option)| {
                let selected = index == *value;
                div()
                    .id(SharedString::from(format!("{row}-segment-{index}")))
                    .min_w(px(72.0))
                    .px(px(10.0))
                    .py(px(3.0))
                    .rounded(px(6.0))
                    .flex()
                    .justify_center()
                    .text_size(theme.type_.label.size)
                    .text_color(if selected { p.ink } else { p.ink_secondary })
                    .when(selected, |el| {
                        el.bg(p.surface_field)
                            .shadow(crate::theme::elevation::lift(theme))
                    })
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        on_press(events, SettingsEvent::Choose(row.clone(), index)),
                    )
                    .child(option.clone())
            }))
            .into_any_element(),
        SettingsControl::Button { label, destructive } => {
            button(theme, row, label.clone(), *destructive, events)
        }
        SettingsControl::Link(label) => div()
            .id(SharedString::from(format!("{row}-link")))
            .px(px(8.0))
            .text_size(theme.type_.label.size)
            .text_color(p.ink)
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                on_press(events, SettingsEvent::Press(row.clone())),
            )
            .child(label.clone())
            .into_any_element(),
        SettingsControl::Keys(keys) => div()
            .flex()
            .gap(px(4.0))
            .children(keys.iter().map(|key| {
                div()
                    .min_w(px(22.0))
                    .h(px(20.0))
                    .px(px(5.0))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(p.hairline)
                    .bg(p.surface_canvas)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(theme.type_.key.size)
                    .text_color(p.ink_secondary)
                    .child(key.clone())
            }))
            .into_any_element(),
        SettingsControl::Density(value) => {
            div()
                .flex()
                .gap(px(8.0))
                .children(["Small", "Normal", "Large"].into_iter().enumerate().map(
                    |(index, label)| {
                        let selected = *value == Some(index);
                        let scale = [0.8_f32, 0.9, 1.0][index];
                        div()
                            .id(SharedString::from(format!("{row}-density-{index}")))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(4.0))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                on_press(events, SettingsEvent::Choose(row.clone(), index)),
                            )
                            .child(
                                div()
                                    .w(px(86.0))
                                    .h(px(28.0 + 6.0 * index as f32))
                                    .px(px(8.0))
                                    .rounded(px(8.0))
                                    .border_1()
                                    .border_color(if selected { p.accent } else { p.hairline })
                                    .bg(p.surface_field)
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .size(px(16.0 * scale))
                                            .rounded(px(4.0))
                                            .bg(p.ink_secondary),
                                    )
                                    .child(
                                        div().flex_1().h(px(4.0)).rounded(px(2.0)).bg(p.hairline),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(theme.type_.meta.size)
                                    .text_color(if selected { p.ink } else { p.ink_secondary })
                                    .child(label),
                            )
                    },
                ))
                .into_any_element()
        }
        SettingsControl::Chip { letter, hue } => chip(theme, *letter, *hue),
        SettingsControl::ChipButton { letter, hue, label } => div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(chip(theme, *letter, *hue))
            .child(button(theme, row, label.clone(), false, events))
            .into_any_element(),
    }
}

fn chip(theme: &Theme, letter: char, hue: gpui::Rgba) -> AnyElement {
    div()
        .size(px(22.0))
        .rounded(px(5.0))
        .bg(hue)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.palette.on_accent)
        .child(letter.to_string())
        .into_any_element()
}

fn button(
    theme: &Theme,
    row: &SharedString,
    label: SharedString,
    destructive: bool,
    events: &SettingsCallback,
) -> AnyElement {
    let p = theme.palette;
    div()
        .id(SharedString::from(format!("{row}-button")))
        .h(px(26.0))
        .px(px(12.0))
        .rounded(px(13.0))
        .border_1()
        .border_color(p.hairline)
        .bg(p.surface_field)
        .flex()
        .items_center()
        .text_size(theme.type_.label.size)
        .text_color(if destructive { p.status_error } else { p.ink })
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            on_press(events, SettingsEvent::Press(row.clone())),
        )
        .child(label)
        .into_any_element()
}
