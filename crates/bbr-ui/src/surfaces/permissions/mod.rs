//! Blueprint D11 (boards M-8, M-9, PR-14, DESIGN §11.5): the permission
//! prompt and the site-settings popover. Both hang from the address control
//! and float over the card in the native Portal's overlay layer.
//!
//! The prompt names the site, what it wants and the profile, offers
//! "Remember for this site", then Don't Allow and Allow; nothing is granted
//! by default. The popover lists the site's settings for the profile —
//! Allow / Ask / Block per permission, the zoom with Reset, Clear Site
//! Data… (confirmed inline) and All Site Settings.

use std::rc::Rc;

use gpui::{div, prelude::*, px, AnyElement, App, MouseButton, SharedString, Window};

use crate::icons::Icon;
use crate::theme::Theme;

mod site_settings;
pub use site_settings::{
    site_settings_popover, SiteSettingsEvent, SiteSettingsModel, SiteSettingsRow, SITE_SETTINGS_ID,
    SITE_SETTINGS_WIDTH,
};

/// The §15 ids.
pub const PERMISSION_PROMPT_ID: &str = "bbr-permission-prompt";
/// The prompt's width.
pub const PERMISSION_PROMPT_WIDTH: f32 = 360.0;

/// What the prompt asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionPromptModel {
    /// "meet.example.com wants to use your camera and microphone".
    pub message: SharedString,
    /// "Profile · Work".
    pub profile: SharedString,
    pub camera: bool,
    pub microphone: bool,
    pub remember: bool,
    /// Requests waiting behind this one.
    pub queued: usize,
}

/// The prompt's controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionPromptEvent {
    Remember,
    Deny,
    Allow,
}

pub type PermissionPromptCallback = Rc<dyn Fn(PermissionPromptEvent, &mut Window, &mut App)>;

pub(crate) fn pill_button(
    theme: &Theme,
    id: SharedString,
    label: &str,
    primary: bool,
    on_press: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let p = theme.palette;
    div()
        .id(id)
        .h(px(28.0))
        .px(px(14.0))
        .rounded(px(14.0))
        .when(primary, |el| el.bg(p.accent).text_color(p.on_accent))
        .when(!primary, |el| {
            el.border_1()
                .border_color(p.hairline)
                .bg(p.surface_field)
                .text_color(p.ink)
        })
        .flex()
        .items_center()
        .text_size(theme.type_.label.size)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
            cx.stop_propagation();
            on_press(window, cx);
        })
        .child(label.to_string())
        .into_any_element()
}

pub(crate) fn checkbox(theme: &Theme, checked: bool) -> impl IntoElement {
    let p = theme.palette;
    div()
        .size(px(16.0))
        .flex_shrink_0()
        .rounded(px(4.0))
        .border_1()
        .border_color(if checked { p.accent } else { p.hairline })
        .bg(if checked { p.accent } else { p.surface_field })
        .flex()
        .items_center()
        .justify_center()
        .when(checked, |el| {
            el.child(Icon::Check.canvas_with(theme, px(11.0), p.on_accent))
        })
}

/// The prompt card.
pub fn permission_prompt(
    theme: &Theme,
    model: &PermissionPromptModel,
    events: PermissionPromptCallback,
) -> gpui::Stateful<gpui::Div> {
    let p = theme.palette;
    let icon = if model.camera {
        Icon::Camera { blocked: false }
    } else {
        Icon::Microphone { blocked: false }
    };
    let send = |event: PermissionPromptEvent| {
        let events = events.clone();
        move |window: &mut Window, cx: &mut App| events(event, window, cx)
    };
    let remember_events = events.clone();
    div()
        .id(PERMISSION_PROMPT_ID)
        .w(px(PERMISSION_PROMPT_WIDTH))
        .p(px(16.0))
        .rounded(px(12.0))
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(crate::theme::elevation::elev2(theme))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .flex()
                .gap(px(12.0))
                .child(
                    div()
                        .size(px(32.0))
                        .flex_shrink_0()
                        .rounded(px(8.0))
                        .bg(p.tint_accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon.canvas_with(theme, px(18.0), p.accent)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(p.ink)
                                .child(model.message.clone()),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(p.ink_secondary)
                                .child(model.profile.clone()),
                        ),
                ),
        )
        .child(
            div()
                .id("bbr-permission-remember")
                .flex()
                .items_center()
                .gap(px(8.0))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    cx.stop_propagation();
                    remember_events(PermissionPromptEvent::Remember, window, cx);
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
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .flex_1()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .when(model.queued > 0, |el| {
                            el.child(SharedString::from(match model.queued {
                                1 => "1 more request waiting".to_string(),
                                n => format!("{n} more requests waiting"),
                            }))
                        }),
                )
                .child(pill_button(
                    theme,
                    "bbr-permission-deny".into(),
                    "Don’t Allow",
                    false,
                    send(PermissionPromptEvent::Deny),
                ))
                .child(pill_button(
                    theme,
                    "bbr-permission-allow".into(),
                    "Allow",
                    true,
                    send(PermissionPromptEvent::Allow),
                )),
        )
}
