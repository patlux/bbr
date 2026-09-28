//! The exact floating-command render anatomy (DESIGN.md §11.4/§11.5/§13).
//!
//! The centered Cmd-T palette and the field-anchored Cmd-L/click overlay
//! are the same §11.5 engine and input component; this module draws both
//! cards from the one plain [`CommandModel`] at the resolved rect — the
//! exact render-only anatomy the app's portal root drew before the G3
//! Command cutover, so §15 ids and the semantic tree are byte-identical.
//! The caller owns the full-window backdrop and pointer propagation
//! policy; the card itself stops propagation so a press on it never
//! reaches the backdrop's one-pointer-action dismissal (DESIGN.md §13) —
//! the same two-listener split the portal root composed before.

use gpui::{
    div, prelude::*, px, Animation, AnimationExt, AnyElement, Div, MouseButton, MouseDownEvent,
    Pixels, SharedString, Stateful,
};

use crate::icons::Icon;
use crate::theme::elevation::elev2;
use crate::theme::{Motion, Radius, Spacing, Theme};

use super::geometry::CommandRect;
use super::ids::{
    OVERLAY_CURSOR_ID, OVERLAY_FADE_ELEMENT_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_STATE_EMPTY,
    OVERLAY_STATE_ERROR, OVERLAY_STATE_LOADING, OVERLAY_TEXT_ID, PALETTE_CURSOR_ID,
    PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID, PALETTE_RESULT_PREFIX, PALETTE_STATE_EMPTY,
    PALETTE_STATE_ERROR, PALETTE_STATE_LOADING, PALETTE_TEXT_ID,
};
use super::model::{CommandModel, CommandPresentation, CommandResultRow, CommandResultState};

/// Build the U1 card at the resolved portal rect. The caller owns the
/// full-window backdrop and pointer propagation policy; the card itself
/// stops pointer propagation so a press on it never reaches the backdrop's
/// one-pointer-action dismissal (DESIGN.md §13) — the same two-listener
/// split the portal root composed before.
pub fn command_card(model: &CommandModel, rect: CommandRect) -> AnyElement {
    match model.presentation {
        CommandPresentation::Closed => div()
            .id("bbr-command-portal-closed")
            .debug_selector(|| "bbr-command-portal-closed".to_string())
            .into_any(),
        CommandPresentation::CenteredPalette => palette_card(model, rect).into_any(),
        CommandPresentation::AnchoredOverlay => overlay_card(model, rect),
    }
}

/// §13 stop-propagation: a pointer press on the card never dismisses it
/// through the backdrop listener (the card is the surface, not an outside
/// press). Pure propagation control — no view state needed.
fn card_stop_propagation() -> impl Fn(&MouseDownEvent, &mut gpui::Window, &mut gpui::App) {
    |_e: &MouseDownEvent, _window: &mut gpui::Window, cx: &mut gpui::App| {
        cx.stop_propagation();
    }
}

/// §18.4 per-presentation anatomy: the ⌘T sheet and the ⌘L overlay share
/// the engine, the field component and the row content, and differ only in
/// these sizes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandAnatomy {
    /// The card corner radius.
    pub radius: Pixels,
    /// The field row height.
    pub field_height: Pixels,
    /// The query text size: `type:mono`'s size for the ⌘L URL, 17 pt for
    /// the ⌘T query (the shell renders both in the system UI face; the
    /// mono family is a DESIGN §4 role the text system does not load yet).
    pub query_size: Pixels,
    /// The padding around the result list.
    pub list_padding: Pixels,
    /// One result row's height.
    pub row_height: Pixels,
    /// One result row's (and its cursor's) corner radius.
    pub row_radius: Pixels,
}

/// §18.4: ⌘T — radius 16, a 56 pt field with the 17 pt UI-sans query, 40 pt
/// result rows at radius 10 inside an 8 pt padded list; ⌘L — radius 12, a
/// 40 pt field keeping `type:mono`, 36 pt rows at radius 10 in a 4 pt list.
pub fn anatomy(theme: &Theme, presentation: CommandPresentation) -> CommandAnatomy {
    match presentation {
        CommandPresentation::AnchoredOverlay => CommandAnatomy {
            radius: theme.radius.large(),
            field_height: px(40.0),
            query_size: theme.type_.mono.size,
            list_padding: Spacing::sp1(),
            row_height: px(36.0),
            row_radius: theme.radius.medium(),
        },
        _ => CommandAnatomy {
            radius: theme.radius.xlarge(),
            field_height: px(56.0),
            query_size: px(17.0),
            list_padding: Spacing::sp2(),
            row_height: px(40.0),
            row_radius: theme.radius.medium(),
        },
    }
}

fn palette_card(model: &CommandModel, rect: CommandRect) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let anatomy = anatomy(theme, CommandPresentation::CenteredPalette);
    div()
        .id(PALETTE_ID)
        // Test-support target: the production id as the gallery E2E lookup
        // key (no-op in release builds — the same pattern `auth`/
        // `tab_row` use so gallery scenarios can target the real surface).
        .debug_selector(|| PALETTE_ID.to_string())
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .max_h(px(rect.height))
        .overflow_hidden()
        .rounded(anatomy.radius)
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .flex()
        .flex_col()
        .child(field_row(
            model,
            anatomy,
            PALETTE_FIELD_ID,
            PALETTE_TEXT_ID,
            PALETTE_CURSOR_ID,
            "+",
            Some("New Tab"),
        ))
        .child(result_list(model, anatomy, PALETTE_RESULT_PREFIX))
        .on_mouse_down(MouseButton::Left, card_stop_propagation())
}

/// The §11.4 field row shared by both command-surface presentations: the
/// prefix glyph, the `type:mono` text (`<text_id>`), an optional mode chip
/// (the centered palette only), and the block cursor (`<cursor_id>`) while
/// the surface owns chrome focus. The centered Cmd-T palette and the
/// field-anchored Cmd-L/click overlay are the same §11.5 engine and input
/// component; the overlay opens with the full current URL selected, so one
/// keystroke replaces it.
fn field_row(
    model: &CommandModel,
    anatomy: CommandAnatomy,
    field_id: &'static str,
    text_id: &'static str,
    cursor_id: &'static str,
    prefix: &'static str,
    mode_chip: Option<&'static str>,
) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let text = if model.text.is_empty() {
        SharedString::from("Type a URL…")
    } else {
        SharedString::from(model.text.clone())
    };
    div()
        .id(field_id)
        .debug_selector(|| field_id.to_string())
        .h(anatomy.field_height)
        .flex_shrink_0()
        .px(Spacing::sp3())
        .gap(Spacing::sp2())
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(prefix),
        )
        .child(
            div()
                .id(text_id)
                .flex_1()
                .text_size(anatomy.query_size)
                .text_color(p.ink)
                .child(text),
        )
        .when(mode_chip.is_some(), |el| {
            el.child(
                div()
                    .id(PALETTE_MODE_ID)
                    .text_size(theme.type_.badge.size)
                    .font_weight(theme.type_.badge.weight)
                    .text_color(p.accent_ink)
                    .child(mode_chip.expect("chip checked")),
            )
        })
        .when(model.chrome_focused, |el| {
            el.child(div().id(cursor_id).w(px(2.0)).h_full().bg(p.accent))
        })
}

fn overlay_card(model: &CommandModel, rect: CommandRect) -> AnyElement {
    let theme = &model.theme;
    let p = theme.palette;
    let anatomy = anatomy(theme, CommandPresentation::AnchoredOverlay);
    let card = div()
        .id(OVERLAY_ID)
        // Test-support target: the production id as the gallery E2E lookup
        // key (no-op in release builds).
        .debug_selector(|| OVERLAY_ID.to_string())
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .max_h(px(rect.height))
        .overflow_hidden()
        .rounded(anatomy.radius)
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .flex()
        .flex_col()
        // §11.4: the overlay's own field sits on top with the full URL
        // preselected (the shared §11.5 input component; no mode chip — the
        // overlay is Location mode by construction).
        .child(field_row(
            model,
            anatomy,
            OVERLAY_FIELD_ID,
            OVERLAY_TEXT_ID,
            OVERLAY_CURSOR_ID,
            "▶",
            None,
        ))
        .child(result_list(model, anatomy, "bbr-command-result"))
        .on_mouse_down(MouseButton::Left, card_stop_propagation());
    // §11.4: the overlay opens with an opacity fade `dur:fine` only — never
    // translate or scale (§6). The fade is per-open: the animation element
    // state is dropped when the overlay leaves the tree, so every re-present
    // (Cmd-L, pointer press, or a restored Location origin) fades in again.
    card.with_animation(
        OVERLAY_FADE_ELEMENT_ID,
        Animation::new(Motion::fine()),
        |el, delta| el.opacity(delta),
    )
    .into_any()
}

/// §18.4: the result rows inside their padded list (the padding keeps the
/// inset cursor off the card edge).
fn result_list(model: &CommandModel, anatomy: CommandAnatomy, result_prefix: &str) -> Div {
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
                .child(group_mark(theme, row)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, ThemeMode};

    #[test]
    fn the_palette_and_overlay_follow_the_v2_anatomy() {
        // §18.4: ⌘T radius 16 / 56 pt field / 17 pt query / 40 pt rows at
        // radius 10 in an 8 pt list; ⌘L radius 12 / 40 pt field / mono-size
        // query / 36 pt rows at radius 10 in a 4 pt list — at every density.
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = Theme::new(ThemeMode::Light, ContrastMode::Standard, density);
            let palette = anatomy(&theme, CommandPresentation::CenteredPalette);
            assert_eq!(palette.radius, px(16.0));
            assert_eq!(palette.field_height, px(56.0));
            assert_eq!(palette.query_size, px(17.0));
            assert_eq!(palette.list_padding, px(8.0));
            assert_eq!(palette.row_height, px(40.0));
            assert_eq!(palette.row_radius, px(10.0));
            let overlay = anatomy(&theme, CommandPresentation::AnchoredOverlay);
            assert_eq!(overlay.radius, px(12.0));
            assert_eq!(overlay.field_height, px(40.0));
            assert_eq!(overlay.query_size, theme.type_.mono.size);
            assert_eq!(overlay.list_padding, px(4.0));
            assert_eq!(overlay.row_height, px(36.0));
            assert_eq!(overlay.row_radius, px(10.0));
        }
    }
}
