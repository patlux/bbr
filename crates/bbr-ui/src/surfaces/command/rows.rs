//! The floating command surfaces' result list (DESIGN.md §11.4/§11.5,
//! S12 Paper anatomy): the padded list, the state rows, and one result row
//! `[badge] [title · match] [url mono] … [kind hint]`. Split from
//! `render.rs` so both stay under the ~500-line module budget.

use gpui::{div, prelude::*, px, AnyElement, Div, SharedString, Stateful};

use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::ids::{
    OVERLAY_STATE_EMPTY, OVERLAY_STATE_ERROR, OVERLAY_STATE_LOADING, PALETTE_STATE_EMPTY,
    PALETTE_STATE_ERROR, PALETTE_STATE_LOADING,
};
use super::model::{CommandModel, CommandPresentation, CommandResultRow, CommandResultState};
use super::render::CommandAnatomy;

/// §18.4: the result rows inside their padded list (the padding keeps the
/// inset cursor off the card edge).
pub(super) fn result_list(
    model: &CommandModel,
    anatomy: CommandAnatomy,
    result_prefix: &str,
) -> Div {
    let rows = result_rows(model, anatomy, result_prefix);
    div()
        .relative()
        .flex()
        .flex_col()
        .min_h_0()
        .when(!rows.is_empty(), |el| el.p(anatomy.list_padding))
        // §18.5: the gliding cursor paints beneath the rows.
        .when_some(
            super::moments::glide_highlight(model, anatomy),
            |el, glide| el.child(glide),
        )
        .children(rows)
}

fn result_rows(
    model: &CommandModel,
    anatomy: CommandAnatomy,
    result_prefix: &str,
) -> Vec<AnyElement> {
    let cursor = super::moments::row_paints_cursor(model);
    match model.state {
        CommandResultState::Results => model
            .results
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let selected = cursor && index == model.selected;
                let row = result_row(model, anatomy, row, selected, result_prefix);
                super::moments::entering_row(model, index, row)
            })
            .collect(),
        CommandResultState::Loading => {
            vec![state_row(&model.theme, state_id(model, "loading"), "Ranking…").into_any_element()]
        }
        CommandResultState::Empty => {
            vec![
                state_row(&model.theme, state_id(model, "empty"), "Type a destination")
                    .into_any_element(),
            ]
        }
        CommandResultState::ActivationError => {
            vec![
                state_row(&model.theme, state_id(model, "error"), "Activation failed")
                    .into_any_element(),
            ]
        }
        CommandResultState::Inactive => Vec::new(),
    }
}

/// The typed state-row id of the open surface: the palette's own family
/// (`bbr-palette-loading` / `-empty` / `-error`) or the overlay's §11.5
/// state ids (`bbr-command-loading` / `-empty` / `-error`, the same
/// strings the in-rail surface uses).
pub(crate) fn state_id(model: &CommandModel, kind: &str) -> &'static str {
    match (model.presentation, kind) {
        (CommandPresentation::CenteredPalette, "loading") => PALETTE_STATE_LOADING,
        (CommandPresentation::CenteredPalette, "empty") => PALETTE_STATE_EMPTY,
        (CommandPresentation::CenteredPalette, "error") => PALETTE_STATE_ERROR,
        (CommandPresentation::AnchoredOverlay, "loading") => OVERLAY_STATE_LOADING,
        (CommandPresentation::AnchoredOverlay, "empty") => OVERLAY_STATE_EMPTY,
        (CommandPresentation::AnchoredOverlay, "error") => OVERLAY_STATE_ERROR,
        _ => unreachable!("state rows render only for an open presentation"),
    }
}

fn result_row(
    model: &CommandModel,
    anatomy: CommandAnatomy,
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
    let id = format!("{id_prefix}-{}", row.key);
    // S12 row anatomy: [badge] [title with the match in `accent.ink`]
    // [url mono, shortened] … [kind hint]. A subtitle (cross-space identity,
    // shortcut) takes the mono column when present — identity is never
    // hidden behind the URL.
    // Blueprint D04: an action's shortcut takes the kind column (so the
    // palette teaches the keyboard); it never repeats in the mono column.
    let detail = if row.group == "action" {
        None
    } else {
        row.subtitle
            .clone()
            .or_else(|| row.url.as_deref().map(crate::text::short_url))
    };
    // The `>` actions prefix is not part of the match highlight.
    let query = crate::surfaces::command::actions_query(&model.text);
    div()
        .id(SharedString::from(id.clone()))
        // Test-support target: the production result id as the gallery E2E
        // lookup key (no-op in release builds — the same pattern the
        // downloads/auth rows use).
        .debug_selector(move || id.clone())
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .gap(Spacing::sp2())
        .h(anatomy.row_height)
        .px(Spacing::sp3())
        .rounded(anatomy.row_radius)
        // §18.4: the cursor is the inset `tint.accent` row surface — the
        // accent rail is gone; the selected result is committed at once.
        .when(selected, |el| el.bg(p.tint_accent))
        .child(row_badge(theme, row))
        .child(title_line(theme, query, &row.title, title_color))
        .when_some(detail, |el, detail| {
            el.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_size(theme.type_.mono.size)
                    .text_color(p.ink_secondary)
                    .child(SharedString::from(detail)),
            )
        })
        .child(div().flex_1())
        .child(
            div()
                .flex_shrink_0()
                .text_size(theme.type_.key.size)
                .text_color(if row.destructive {
                    p.status_error
                } else {
                    p.ink_faint
                })
                .child(SharedString::from(kind_hint(row, selected))),
        )
}

/// The leading 18 pt badge: the host monogram on its slate (the §7.2 tab
/// tile) for rows with a URL, the drawn bookmark for a Saved Page, the
/// group letter otherwise; destructive rows keep the error surface.
fn row_badge(theme: &Theme, row: &CommandResultRow) -> Div {
    let p = theme.palette;
    let size = px(18.0);
    let base = div()
        .size(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(Radius.small())
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight);
    if row.destructive {
        return base
            .bg(p.status_error_surface)
            .text_color(p.status_error)
            .child(group_mark(theme, row));
    }
    match (group_glyph(row), row.url.as_deref()) {
        (None, _) => base.bg(p.surface_field).child(group_mark(theme, row)),
        (Some(_), Some(url)) => {
            let hue = crate::components::favicon::monogram_hue(url);
            base.bg(match theme.mode {
                crate::theme::ThemeMode::Dark => crate::palette::FaviconSlate::dark(hue),
                crate::theme::ThemeMode::Light => crate::palette::FaviconSlate::light(hue),
            })
            .text_color(crate::palette::FaviconSlate::letter(theme.mode))
            .child(SharedString::from(
                crate::components::favicon::monogram_letter(url).to_string(),
            ))
        }
        (Some(_), None) => base
            .bg(p.surface_field)
            .text_color(p.ink_secondary)
            .child(group_mark(theme, row)),
    }
}

/// The title with the query's first (case-insensitive) occurrence in
/// `accent.ink` — the §11.4 match highlighting.
fn title_line(theme: &Theme, query: &str, title: &str, color: gpui::Rgba) -> Div {
    let p = theme.palette;
    let highlights: Vec<(std::ops::Range<usize>, gpui::HighlightStyle)> = match_range(title, query)
        .map(|(start, end)| {
            (
                start..end,
                gpui::HighlightStyle {
                    color: Some(p.accent_ink.into()),
                    ..Default::default()
                },
            )
        })
        .into_iter()
        .collect();
    div()
        .flex_shrink_0()
        .max_w(px(260.0))
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .text_size(theme.type_.title.size)
        .font_weight(theme.type_.title.weight)
        .text_color(color)
        .child(gpui::StyledText::new(title.to_string()).with_highlights(highlights))
}

/// The byte range of the first ASCII-case-insensitive occurrence of the
/// trimmed query in the title (on char boundaries), if any.
pub(super) fn match_range(title: &str, query: &str) -> Option<(usize, usize)> {
    let needle = query.trim();
    if needle.is_empty() || needle.len() > title.len() {
        return None;
    }
    let lower_title = title.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let start = lower_title.find(&lower_needle)?;
    let end = start + needle.len();
    (title.is_char_boundary(start) && title.is_char_boundary(end)).then_some((start, end))
}

/// The trailing hint: `↵` on the selected row, the result kind otherwise
/// (destructive rows say so).
pub(super) fn kind_hint(row: &CommandResultRow, selected: bool) -> String {
    if row.destructive {
        return "destructive".into();
    }
    if selected {
        return "↵".into();
    }
    // Blueprint D04: an action shows its shortcut when it has one.
    if row.group == "action" {
        if let Some(shortcut) = &row.subtitle {
            return shortcut.clone();
        }
    }
    let kind = match row.group.as_str() {
        "tab" | "exact_tab" | "current_pinned" | "current_tab" | "other_space" => "Tab",
        "space" => "Space",
        "saved_page" => "Saved",
        "history" => "History",
        "action" => "Action",
        _ if row.url.is_some() => "URL",
        _ => "Search",
    };
    kind.into()
}

fn state_row(theme: &Theme, id: &str, text: &str) -> Stateful<Div> {
    let error = id.ends_with("-error");
    div()
        .id(SharedString::from(id))
        // Test-support target: the production state-row id as the gallery
        // E2E lookup key (no-op in release builds).
        .debug_selector(move || id.to_string())
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

/// The result row's leading mark: the drawn §11.16 bookmark glyph for a
/// Saved Page (never a Unicode `★`, never the `→` fallback), otherwise
/// the group letter.
fn group_mark(theme: &Theme, row: &CommandResultRow) -> AnyElement {
    match group_glyph(row) {
        Some(letter) => SharedString::from(letter).into_any_element(),
        None => Icon::Bookmark.canvas(theme, px(12.0)).into_any_element(),
    }
}

/// The group letter, or `None` for a Saved Page (drawn glyph instead).
pub(super) fn group_glyph(row: &CommandResultRow) -> Option<&'static str> {
    match row.group.as_str() {
        "tab" | "exact_tab" | "current_pinned" | "current_tab" | "other_space" => Some("T"),
        "space" => Some("S"),
        "saved_page" => None,
        "action" => Some("A"),
        "history" => Some("H"),
        _ => Some("→"),
    }
}
