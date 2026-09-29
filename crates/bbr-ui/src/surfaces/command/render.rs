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

use crate::components::{edit_line, EditLineColors};
use crate::icons::Icon;
use crate::theme::elevation::elev2;
use crate::theme::{Motion, Radius, Spacing, Theme};

use super::geometry::CommandRect;
use super::ids::{
    OVERLAY_FADE_ELEMENT_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_TEXT_ID, PALETTE_FIELD_ID,
    PALETTE_ID, PALETTE_MODE_ID, PALETTE_RESULT_PREFIX, PALETTE_TEXT_ID,
};
use super::model::{CommandModel, CommandPresentation};

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
            // Blueprint D04: a leading `>` switches the palette to actions.
            Some(if has_actions_prefix(&model.text) {
                "Actions"
            } else {
                "New Tab"
            }),
        ))
        .child(super::rows::result_list(
            model,
            anatomy,
            PALETTE_RESULT_PREFIX,
        ))
        .on_mouse_down(MouseButton::Left, card_stop_propagation())
}

/// The §11.4 field row shared by both command-surface presentations: the
/// search glyph, the editable query line (`<text_id>`) and an optional mode
/// chip (the centered palette only). The line draws the selection wash, the
/// IME composition underline and — while the surface owns chrome focus and
/// nothing is selected — the caret (the logical `…-cursor` id) at the real
/// insertion point, scrolling horizontally to keep it visible. The centered
/// Cmd-T palette and the field-anchored Cmd-L/click overlay are the same
/// §11.5 engine and input component; the overlay opens with the full
/// current URL selected, so one keystroke replaces it.
fn field_row(
    model: &CommandModel,
    anatomy: CommandAnatomy,
    field_id: &'static str,
    text_id: &'static str,
    mode_chip: Option<&'static str>,
) -> Stateful<Div> {
    let theme = &model.theme;
    let p = theme.palette;
    let line = div()
        .id(text_id)
        .flex_1()
        .min_w_0()
        .h(anatomy.query_size * 1.5)
        .text_size(anatomy.query_size)
        .overflow_hidden()
        .child(edit_line(
            &model.text,
            model.edit,
            SharedString::from("Type a URL…"),
            EditLineColors {
                text: p.ink,
                placeholder: p.ink_faint,
                selection: p.tint_accent,
                caret: p.accent,
            },
            model.chrome_focused,
            model.line_sink.clone(),
        ));
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
        // S12: the field leads with the drawn search magnifier and closes
        // with a hairline above the results.
        .border_b_1()
        .border_color(p.hairline)
        .child(Icon::Search.canvas(theme, px(18.0)))
        .child(line)
        .when_some(mode_chip, |el, chip| {
            // The mode chip is an uppercase pill on `tint.accent`.
            el.child(
                div()
                    .id(PALETTE_MODE_ID)
                    .flex_shrink_0()
                    .px(Spacing::sp2())
                    .py(px(2.0))
                    .rounded(Radius.small())
                    .bg(p.tint_accent)
                    .text_size(theme.type_.badge.size)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(p.accent_ink)
                    .child(chip.to_uppercase()),
            )
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
            None,
        ))
        .child(super::rows::result_list(
            model,
            anatomy,
            "bbr-command-result",
        ))
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

/// Whether the field text asks for the actions mode (a leading `>`).
fn has_actions_prefix(text: &str) -> bool {
    text.trim_start().starts_with('>')
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
