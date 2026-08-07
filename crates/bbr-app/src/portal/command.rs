//! U1 command surfaces rendered inside the one native portal window.
//!
//! The browser app remains the sole command-state authority. The portal root
//! reads one cloned [`CommandPortalModel`] per render and builds GPUI chrome;
//! AppKit continues to own only the native child-window boundary.

use gpui::{div, prelude::*, px, Div, SharedString, Stateful};

use super::geometry::Rect;
use crate::app::command_rank::{CommandResultRow, CommandResultState};
use crate::app::palette::{
    CommandSurface, OVERLAY_ID, PALETTE_CURSOR_ID, PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID,
    PALETTE_RESULT_PREFIX, PALETTE_TEXT_ID,
};
use crate::theme::{Density, Radius, Spacing, Theme};

/// Immutable command data consumed by one portal-window render.
#[derive(Clone, Debug)]
pub struct CommandPortalModel {
    pub surface: CommandSurface,
    pub theme: Theme,
    pub chrome_focused: bool,
    pub text: String,
    pub state: CommandResultState,
    pub results: Vec<CommandResultRow>,
    pub selected: usize,
}

/// Build the U1 card at the resolved portal rect. The caller owns the
/// full-window backdrop and pointer propagation policy.
pub fn command_card(model: &CommandPortalModel, rect: Rect) -> Stateful<Div> {
    match model.surface {
        CommandSurface::None => div().id("bbr-command-portal-closed"),
        CommandSurface::Palette => palette_card(model, rect),
        CommandSurface::Overlay => overlay_card(model, rect),
    }
}

fn palette_card(model: &CommandPortalModel, rect: Rect) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let radius = match theme.density {
        Density::ExtraLarge => theme.radius.xlarge(),
        _ => theme.radius.large(),
    };
    div()
        .id(PALETTE_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .max_h(px(rect.height))
        .overflow_hidden()
        .rounded(radius)
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2_shadows(theme))
        .flex()
        .flex_col()
        .child(palette_field(model))
        .children(result_rows(model, PALETTE_RESULT_PREFIX, "bbr-palette"))
}

fn palette_field(model: &CommandPortalModel) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let text = if model.text.is_empty() {
        SharedString::from("Type a URL…")
    } else {
        SharedString::from(model.text.clone())
    };
    div()
        .id(PALETTE_FIELD_ID)
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .gap(Spacing::sp2())
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child("+"),
        )
        .child(
            div()
                .id(PALETTE_TEXT_ID)
                .flex_1()
                .text_size(theme.type_.mono.size)
                .text_color(p.ink)
                .child(text),
        )
        .child(
            div()
                .id(PALETTE_MODE_ID)
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(p.accent_ink)
                .child("New Tab"),
        )
        .when(model.chrome_focused, |el| {
            el.child(div().id(PALETTE_CURSOR_ID).w(px(2.0)).h_full().bg(p.accent))
        })
}

fn overlay_card(model: &CommandPortalModel, rect: Rect) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    div()
        .id(OVERLAY_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .max_h(px(rect.height))
        .overflow_hidden()
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2_shadows(theme))
        .py(Spacing::sp1())
        .children(result_rows(model, "bbr-command-result", "bbr-command"))
}

fn result_rows(
    model: &CommandPortalModel,
    result_prefix: &str,
    state_prefix: &str,
) -> Vec<Stateful<Div>> {
    match model.state {
        CommandResultState::Results => model
            .results
            .iter()
            .enumerate()
            .map(|(index, row)| result_row(model, row, index == model.selected, result_prefix))
            .collect(),
        CommandResultState::Loading => vec![state_row(
            &model.theme,
            format!("{state_prefix}-loading"),
            "Ranking…",
        )],
        CommandResultState::Empty => vec![state_row(
            &model.theme,
            format!("{state_prefix}-empty"),
            "Type a destination",
        )],
        CommandResultState::ActivationError => vec![state_row(
            &model.theme,
            format!("{state_prefix}-error"),
            "Activation failed",
        )],
        CommandResultState::Inactive => Vec::new(),
    }
}

fn result_row(
    model: &CommandPortalModel,
    row: &CommandResultRow,
    selected: bool,
    id_prefix: &str,
) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let title_color = if row.destructive {
        p.status_error
    } else {
        p.ink
    };
    div()
        .id(SharedString::from(format!("{id_prefix}-{}", row.key)))
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .when(selected, |el| {
            el.bg(p.tint_selected).border_l_2().border_color(p.accent)
        })
        .child(
            div()
                .size(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(Radius.small())
                .bg(if row.destructive {
                    p.status_error_surface
                } else {
                    p.surface_field
                })
                .text_size(theme.type_.key.size)
                .font_weight(theme.type_.key.weight)
                .text_color(if row.destructive {
                    p.status_error
                } else {
                    p.ink_faint
                })
                .child(SharedString::from(group_glyph(row))),
        )
        .child(
            div()
                .max_w(px(140.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(title_color)
                .child(SharedString::from(row.title.clone())),
        )
        .when(row.subtitle.is_some(), |el| {
            el.child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(theme.type_.mono.size)
                    .text_color(p.ink_secondary)
                    .child(SharedString::from(
                        row.subtitle.clone().expect("checked above"),
                    )),
            )
        })
        .when(row.destructive || selected, |el| {
            el.child(
                div()
                    .text_size(theme.type_.key.size)
                    .text_color(if row.destructive {
                        p.status_error
                    } else {
                        p.ink_faint
                    })
                    .child(if row.destructive {
                        "destructive"
                    } else {
                        "↵"
                    }),
            )
        })
}

fn state_row(theme: &Theme, id: String, text: &str) -> Stateful<Div> {
    let error = id.ends_with("-error");
    div()
        .id(SharedString::from(id))
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.meta.size)
        .text_color(if error {
            theme.palette.status_error
        } else {
            theme.palette.ink_faint
        })
        .child(SharedString::from(text.to_string()))
}

fn elev2_shadows(theme: &Theme) -> Vec<gpui::BoxShadow> {
    use gpui::{hsla, point, BoxShadow};
    match theme.mode {
        crate::theme::ThemeMode::Dark => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 1.0, 0.05),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.45),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(32.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
        crate::theme::ThemeMode::Light => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.04),
                offset: point(px(0.0), px(1.0)),
                blur_radius: px(0.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.09, 0.10),
                offset: point(px(0.0), px(12.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
    }
}

fn group_glyph(row: &CommandResultRow) -> String {
    match row.group.as_str() {
        "tab" | "exact_tab" | "current_pinned" | "current_tab" | "other_space" => "T".into(),
        "space" => "S".into(),
        "action" => "A".into(),
        "history" => "H".into(),
        _ => "→".into(),
    }
}
