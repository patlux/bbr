//! Blueprint D15 (boards L-1, L-2, PR-01, DESIGN §11.8): the first run.
//!
//! The first launch opens one empty space whose card shows a short
//! welcome: what BBR is, three things to know, Get Started and "Import from
//! Safari or Chrome…". The import link leads through three skippable steps
//! in the same place — bring things over, the default browser, name the
//! first space — each saying where the data lands.

use std::rc::Rc;

use gpui::{div, prelude::*, px, AnyElement, App, MouseButton, SharedString, Window};

use crate::icons::Icon;
use crate::surfaces::permissions::{checkbox, pill_button};
use crate::theme::Theme;

pub const WELCOME_ID: &str = "bbr-welcome";
pub const ONBOARDING_ID: &str = "bbr-onboarding";

/// One import source row.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportRow {
    pub label: SharedString,
    /// "1 204 bookmarks · 18 months of history" or why it cannot be read.
    pub detail: SharedString,
    pub checked: bool,
    pub enabled: bool,
}

/// Which page of the first run shows.
#[derive(Debug, Clone, PartialEq)]
pub enum WelcomeStep {
    Welcome,
    /// `None` while the sources are being read.
    Import(Option<Vec<ImportRow>>),
    DefaultBrowser {
        is_default: bool,
    },
    FirstSpace {
        name: SharedString,
        focused: bool,
    },
}

/// Every control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeEvent {
    GetStarted,
    StartImport,
    ToggleSource(usize),
    Skip,
    Import,
    MakeDefault,
    Done,
    FocusName,
}

pub type WelcomeCallback = Rc<dyn Fn(WelcomeEvent, &mut Window, &mut App)>;

fn send(events: &WelcomeCallback, event: WelcomeEvent) -> impl Fn(&mut Window, &mut App) + 'static {
    let events = events.clone();
    move |window, cx| events(event, window, cx)
}

fn heading(theme: &Theme, title: &str, body: &str) -> AnyElement {
    let p = theme.palette;
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_size(px(22.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(theme.type_.label.size)
                .text_color(p.ink_secondary)
                .child(body.to_string()),
        )
        .into_any_element()
}

fn buttons(
    theme: &Theme,
    events: &WelcomeCallback,
    primary: (&str, WelcomeEvent, bool),
) -> AnyElement {
    let (label, event, enabled) = primary;
    div()
        .pt(px(8.0))
        .flex()
        .justify_end()
        .gap(px(8.0))
        .child(pill_button(
            theme,
            "bbr-onboarding-skip".into(),
            "Skip",
            false,
            send(events, WelcomeEvent::Skip),
        ))
        .child(
            div()
                .opacity(if enabled { 1.0 } else { 0.45 })
                .child(pill_button(
                    theme,
                    "bbr-onboarding-next".into(),
                    label,
                    true,
                    send(events, event),
                )),
        )
        .into_any_element()
}

fn welcome(theme: &Theme, events: &WelcomeCallback) -> AnyElement {
    let p = theme.palette;
    let item = |icon: Icon, title: &str, body: &str| {
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
                    .child(icon.canvas_with(theme, px(16.0), p.accent)),
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
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .text_size(theme.type_.meta.size)
                            .text_color(p.ink_secondary)
                            .child(body.to_string()),
                    ),
            )
    };
    let import = send(events, WelcomeEvent::StartImport);
    div()
        .id(WELCOME_ID)
        .flex()
        .flex_col()
        .gap(px(18.0))
        .child(heading(
            theme,
            "Welcome to BBR",
            "A fast browser for many contexts at once.",
        ))
        .child(item(
            Icon::Search,
            "⌘T opens anything",
            "Tabs, history, saved pages, spaces and actions — one field.",
        ))
        .child(item(
            Icon::Bbr(bbr_domain::BbrGlyph::Globe),
            "Spaces keep contexts apart",
            "Work, research, home: each with its own tabs and colour.",
        ))
        .child(item(
            Icon::Person,
            "Profiles keep logins apart",
            "Separate cookies and site data when you need them.",
        ))
        .child(
            div()
                .pt(px(6.0))
                .flex()
                .items_center()
                .gap(px(14.0))
                .child(pill_button(
                    theme,
                    "bbr-welcome-start".into(),
                    "Get Started",
                    true,
                    send(events, WelcomeEvent::GetStarted),
                ))
                .child(
                    div()
                        .id("bbr-welcome-import")
                        .text_size(theme.type_.label.size)
                        .text_color(p.accent_ink)
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            cx.stop_propagation();
                            import(window, cx);
                        })
                        .child("Import from Safari or Chrome…"),
                ),
        )
        .into_any_element()
}

fn import_step(
    theme: &Theme,
    rows: &Option<Vec<ImportRow>>,
    events: &WelcomeCallback,
) -> AnyElement {
    let p = theme.palette;
    let body: AnyElement = match rows {
        None => div()
            .text_size(theme.type_.label.size)
            .text_color(p.ink_faint)
            .child("Looking for Safari and Chrome…")
            .into_any_element(),
        Some(rows) if rows.is_empty() => div()
            .text_size(theme.type_.label.size)
            .text_color(p.ink_faint)
            .child("Neither Safari nor Chrome has anything to bring over.")
            .into_any_element(),
        Some(rows) => div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .children(rows.iter().enumerate().map(|(index, row)| {
                let toggle = send(events, WelcomeEvent::ToggleSource(index));
                let ink = if row.enabled { p.ink } else { p.ink_faint };
                div()
                    .id(SharedString::from(format!("bbr-onboarding-source-{index}")))
                    .p(px(10.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(p.hairline)
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .when(row.enabled, |el| {
                        el.cursor_pointer().on_mouse_down(
                            MouseButton::Left,
                            move |_e, window, cx| {
                                cx.stop_propagation();
                                toggle(window, cx);
                            },
                        )
                    })
                    .child(checkbox(theme, row.checked))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(theme.type_.label.size)
                                    .text_color(ink)
                                    .child(row.label.clone()),
                            )
                            .child(
                                div()
                                    .text_size(theme.type_.meta.size)
                                    .text_color(p.ink_secondary)
                                    .child(row.detail.clone()),
                            ),
                    )
            }))
            .into_any_element(),
    };
    let any = rows
        .as_ref()
        .is_some_and(|rows| rows.iter().any(|row| row.checked && row.enabled));
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(heading(
            theme,
            "Bring your things over",
            "Bookmarks become Saved Pages; history joins History. Nothing is imported until you choose Import.",
        ))
        .child(body)
        .child(buttons(theme, events, ("Import", WelcomeEvent::Import, any)))
        .into_any_element()
}

fn default_step(theme: &Theme, is_default: bool, events: &WelcomeCallback) -> AnyElement {
    let p = theme.palette;
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(heading(
            theme,
            "Open links in BBR?",
            "Links from Mail, Slack and other apps will open in the space you used last.",
        ))
        .child(
            div()
                .id("bbr-onboarding-default-status")
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(p.ink)
                        .child(if is_default {
                            "BBR is your default browser"
                        } else {
                            "BBR is not your default browser"
                        }),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .child("macOS asks to confirm."),
                ),
        )
        .child(buttons(
            theme,
            events,
            ("Make Default", WelcomeEvent::MakeDefault, !is_default),
        ))
        .into_any_element()
}

fn space_step(
    theme: &Theme,
    name: &SharedString,
    focused: bool,
    events: &WelcomeCallback,
) -> AnyElement {
    let p = theme.palette;
    let focus = send(events, WelcomeEvent::FocusName);
    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .child(heading(
            theme,
            "Name your first space",
            "Spaces keep contexts apart. Add more any time from the footer.",
        ))
        .child(
            div()
                .id("bbr-onboarding-space-name")
                .h(px(34.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(if focused { p.accent } else { p.hairline })
                .bg(p.surface_field)
                .flex()
                .items_center()
                .text_size(theme.type_.label.size)
                .text_color(if name.is_empty() { p.ink_faint } else { p.ink })
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    cx.stop_propagation();
                    focus(window, cx);
                })
                .child(if name.is_empty() {
                    SharedString::from("Personal")
                } else {
                    name.clone()
                }),
        )
        .child(buttons(theme, events, ("Done", WelcomeEvent::Done, true)))
        .into_any_element()
}

/// The first-run page for the content card.
pub fn welcome_surface(
    theme: &Theme,
    step: &WelcomeStep,
    events: WelcomeCallback,
) -> impl IntoElement {
    let p = theme.palette;
    let content = match step {
        WelcomeStep::Welcome => welcome(theme, &events),
        WelcomeStep::Import(rows) => import_step(theme, rows, &events),
        WelcomeStep::DefaultBrowser { is_default } => default_step(theme, *is_default, &events),
        WelcomeStep::FirstSpace { name, focused } => space_step(theme, name, *focused, &events),
    };
    div()
        .id(ONBOARDING_ID)
        .size_full()
        .bg(p.surface_canvas)
        .flex()
        .justify_center()
        .items_center()
        .child(div().w(px(440.0)).child(content))
}
