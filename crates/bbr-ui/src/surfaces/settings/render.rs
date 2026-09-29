//! The Settings window anatomy (boards L-3…L-9): the toolbar (the pane
//! title centered in the titlebar band, the seven pane buttons below it,
//! the selected one on a quiet pill) over the scrolling content — for each
//! section an uppercase label and one rounded card of hairline-separated
//! rows `[title / detail] … [control]`.

use gpui::{div, prelude::*, px, MouseButton, SharedString};

use super::controls::control;
use super::{row_id, tab_id, SettingsCallback, SettingsEvent, SettingsModel, SETTINGS_ID};

/// The toolbar height (title band plus pane buttons).
pub const TOOLBAR_HEIGHT: f32 = 96.0;

/// The whole Settings window content.
pub fn settings_surface(
    model: &SettingsModel,
    focus: &gpui::FocusHandle,
    events: SettingsCallback,
) -> impl IntoElement {
    let theme = &model.theme;
    let p = theme.palette;
    let dismiss = {
        let events = events.clone();
        move |_e: &gpui::MouseDownEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
            events(SettingsEvent::Dismiss, window, cx)
        }
    };
    let tabs = super::SettingsPane::ALL.into_iter().map(|pane| {
        let selected = pane == model.pane;
        let events = events.clone();
        div()
            .id(tab_id(pane))
            .w(px(78.0))
            .h(px(52.0))
            .rounded(px(8.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(3.0))
            .when(selected, |el| el.bg(p.wash_active))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                cx.stop_propagation();
                events(SettingsEvent::Pane(pane), window, cx)
            })
            .child(if selected {
                pane.icon().canvas_with(theme, px(18.0), p.accent)
            } else {
                pane.icon().canvas(theme, px(18.0))
            })
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(if selected { p.ink } else { p.ink_secondary })
                    .child(pane.title()),
            )
    });
    let toolbar = div()
        .id("bbr-settings-toolbar")
        .h(px(TOOLBAR_HEIGHT))
        .flex_shrink_0()
        .bg(p.surface_sidebar)
        .border_b_1()
        .border_color(p.hairline)
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .h(px(34.0))
                .flex()
                .items_center()
                .text_size(theme.type_.label.size)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink)
                .child(model.pane.title()),
        )
        .child(div().flex().gap(px(2.0)).children(tabs));
    let sections = model.sections.iter().map(|section| {
        let count = section.rows.len();
        let rows = section.rows.iter().enumerate().map(|(index, row)| {
            let id = row_id(&row.id);
            let open = model.open_select.as_ref() == Some(&row.id);
            let select = {
                let events = events.clone();
                let row = row.id.clone();
                move |_e: &gpui::MouseDownEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
                    events(SettingsEvent::Select(row.clone()), window, cx)
                }
            };
            div()
                .id(id)
                .on_mouse_down(MouseButton::Left, select)
                .min_h(px(44.0))
                .px(px(14.0))
                .py(px(8.0))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(12.0))
                .when(index + 1 < count, |el| {
                    el.border_b_1().border_color(p.hairline)
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .min_w_0()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(p.ink)
                                .child(row.title.clone()),
                        )
                        .when_some(row.detail.clone(), |el, detail| {
                            el.child(
                                div()
                                    .text_size(theme.type_.meta.size)
                                    .text_color(p.ink_faint)
                                    .child(detail),
                            )
                        }),
                )
                .child(control(theme, &row.id, &row.control, open, &events))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .when_some(section.title.clone(), |el, title| {
                el.child(
                    div()
                        .px(px(2.0))
                        .text_size(theme.type_.badge.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink_faint)
                        .child(SharedString::from(title.to_uppercase())),
                )
            })
            .child(
                div()
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(p.hairline)
                    .bg(p.surface_field)
                    .flex()
                    .flex_col()
                    .children(rows),
            )
    });
    let keys = {
        let events = events.clone();
        move |event: &gpui::KeyDownEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
            let keystroke = &event.keystroke;
            let text = keystroke
                .key_char
                .clone()
                .filter(|_| !keystroke.modifiers.platform && !keystroke.modifiers.control)
                .map(SharedString::from);
            events(
                SettingsEvent::Key {
                    key: SharedString::from(keystroke.key.clone()),
                    text,
                },
                window,
                cx,
            )
        }
    };
    let sheet = model
        .sheet
        .as_ref()
        .map(|sheet| super::sheet::sheet(theme, sheet, &events));
    div()
        .id(SETTINGS_ID)
        .relative()
        .size_full()
        .flex()
        .flex_col()
        .bg(p.surface_canvas)
        .track_focus(focus)
        .on_key_down(keys)
        .on_mouse_down(MouseButton::Left, dismiss)
        .child(toolbar)
        .child(
            div()
                .id("bbr-settings-content")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(40.0))
                .py(px(24.0))
                .flex()
                .flex_col()
                .gap(px(18.0))
                .children(sections),
        )
        .children(sheet)
}
