//! Blueprint D11 (board M-8): the site-settings popover.

use std::rc::Rc;

use gpui::{div, prelude::*, px, AnyElement, App, MouseButton, SharedString, Window};

use super::pill_button;
use crate::icons::Icon;
use crate::theme::Theme;

pub const SITE_SETTINGS_ID: &str = "bbr-site-settings";
pub const SITE_SETTINGS_WIDTH: f32 = 320.0;

/// One permission row.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteSettingsRow {
    /// `camera`, `microphone`.
    pub key: SharedString,
    pub label: SharedString,
    pub icon: Icon,
    /// 0 Allow, 1 Ask, 2 Block.
    pub value: usize,
}

/// The popover.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteSettingsModel {
    pub host: SharedString,
    /// "Connection is secure · Profile Work".
    pub connection: SharedString,
    pub rows: Vec<SiteSettingsRow>,
    /// The site's zoom while it is not 100 %.
    pub zoom: Option<u16>,
    /// Clear Site Data… asked for its confirmation.
    pub clear_confirm: bool,
}

/// The popover's controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteSettingsEvent {
    /// Row index, value (0 Allow, 1 Ask, 2 Block).
    Set(usize, usize),
    ResetZoom,
    Clear,
    ClearCancel,
    ClearConfirm,
    AllSettings,
}

pub type SiteSettingsCallback = Rc<dyn Fn(SiteSettingsEvent, &mut Window, &mut App)>;

const VALUES: [&str; 3] = ["Allow", "Ask", "Block"];

fn send(
    events: &SiteSettingsCallback,
    event: SiteSettingsEvent,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let events = events.clone();
    move |window, cx| events(event.clone(), window, cx)
}

fn segmented(
    theme: &Theme,
    index: usize,
    row: &SiteSettingsRow,
    events: &SiteSettingsCallback,
) -> AnyElement {
    let p = theme.palette;
    div()
        .p(px(2.0))
        .rounded(px(7.0))
        .bg(p.wash_hover)
        .flex()
        .children(VALUES.iter().enumerate().map(|(value, label)| {
            let selected = value == row.value;
            let events = events.clone();
            div()
                .id(SharedString::from(format!(
                    "bbr-site-settings-{}-{value}",
                    row.key
                )))
                .px(px(9.0))
                .py(px(2.0))
                .rounded(px(5.0))
                .text_size(theme.type_.meta.size)
                .text_color(if selected { p.ink } else { p.ink_secondary })
                .when(selected, |el| {
                    el.bg(p.surface_field)
                        .shadow(crate::theme::elevation::lift(theme))
                })
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    cx.stop_propagation();
                    events(SiteSettingsEvent::Set(index, value), window, cx);
                })
                .child(label.to_string())
        }))
        .into_any_element()
}

fn quiet_button(
    theme: &Theme,
    id: &'static str,
    label: SharedString,
    destructive: bool,
    on_press: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let p = theme.palette;
    div()
        .id(id)
        .px(px(8.0))
        .py(px(3.0))
        .rounded(px(6.0))
        .text_size(theme.type_.label.size)
        .text_color(if destructive {
            p.status_error
        } else {
            p.accent
        })
        .hover(|el| el.bg(p.wash_hover))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
            cx.stop_propagation();
            on_press(window, cx);
        })
        .child(label)
        .into_any_element()
}

/// The popover card.
pub fn site_settings_popover(
    theme: &Theme,
    model: &SiteSettingsModel,
    events: SiteSettingsCallback,
) -> gpui::Stateful<gpui::Div> {
    let p = theme.palette;
    let rows = model.rows.iter().enumerate().map(|(index, row)| {
        div()
            .id(SharedString::from(format!("bbr-site-settings-{}", row.key)))
            .h(px(34.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(row.icon.canvas(theme, px(16.0)))
            .child(
                div()
                    .flex_1()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .child(row.label.clone()),
            )
            .child(segmented(theme, index, row, &events))
    });
    let zoom = model.zoom.map(|level| {
        div()
            .id("bbr-site-settings-zoom")
            .h(px(34.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(Icon::Search.canvas(theme, px(16.0)))
            .child(
                div()
                    .flex_1()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .child("Zoom"),
            )
            .child(
                div()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink_secondary)
                    .child(SharedString::from(format!("{level} %"))),
            )
            .child(quiet_button(
                theme,
                "bbr-site-settings-zoom-reset",
                "Reset".into(),
                false,
                send(&events, SiteSettingsEvent::ResetZoom),
            ))
    });
    let footer = if model.clear_confirm {
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .flex_1()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_secondary)
                    .child(SharedString::from(format!(
                        "Sign out of {} and remove its data?",
                        model.host
                    ))),
            )
            .child(pill_button(
                theme,
                "bbr-site-settings-clear-cancel".into(),
                "Cancel",
                false,
                send(&events, SiteSettingsEvent::ClearCancel),
            ))
            .child(
                div()
                    .id("bbr-site-settings-clear-confirm")
                    .h(px(28.0))
                    .px(px(14.0))
                    .rounded(px(14.0))
                    .bg(p.status_error)
                    .text_color(p.on_accent)
                    .flex()
                    .items_center()
                    .text_size(theme.type_.label.size)
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, {
                        let confirm = send(&events, SiteSettingsEvent::ClearConfirm);
                        move |_e, window, cx| {
                            cx.stop_propagation();
                            confirm(window, cx);
                        }
                    })
                    .child("Clear"),
            )
            .into_any_element()
    } else {
        div()
            .flex()
            .justify_between()
            .child(quiet_button(
                theme,
                "bbr-site-settings-clear",
                "Clear Site Data…".into(),
                true,
                send(&events, SiteSettingsEvent::Clear),
            ))
            .child(quiet_button(
                theme,
                "bbr-site-settings-all",
                "All Site Settings".into(),
                false,
                send(&events, SiteSettingsEvent::AllSettings),
            ))
            .into_any_element()
    };
    div()
        .id(SITE_SETTINGS_ID)
        .w(px(SITE_SETTINGS_WIDTH))
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(crate::theme::elevation::elev2(theme))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .pb(px(8.0))
                .border_b_1()
                .border_color(p.hairline)
                .child(
                    div()
                        .text_size(theme.type_.title.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink)
                        .child(model.host.clone()),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_secondary)
                        .child(model.connection.clone()),
                ),
        )
        .children(rows)
        .children(zoom)
        .child(
            div()
                .mt(px(4.0))
                .pt(px(8.0))
                .border_t_1()
                .border_color(p.hairline)
                .child(footer),
        )
}
