//! Space-configuration surface render (DESIGN.md §11.3): the BBR-owned
//! rename/identity surface presented after Space creation and from the
//! active Space's management affordance.
//!
//! The surface is ordinary GPUI sidebar chrome: an in-rail panel anchored
//! directly above the footer row (the same placement family as the §11.12
//! downloads panel), so it never overlaps the native web view and never
//! uses the NativeAboveWeb Portal host. Anatomy (top → bottom):
//!
//! 1. Header — `bbr-space-config-title` (`Configure Space · <name>`).
//! 2. Name row — the `bbr-space-config-name` rename field (the draft).
//! 3. Theme row — `bbr-space-config-theme-label` + one
//!    `bbr-space-config-theme-<name>` swatch per curated [`SpaceTheme`]
//!    token (the swatch fills with the one central
//!    [`crate::theme::space_tint`] identity hue — never a raw consumer
//!    color), the selected token carrying the §9 accent outline.
//! 4. Identity row — `bbr-space-config-identity-label` with the two
//!    curated families: `bbr-space-config-emoji-<index>` per [`ALLOWED_EMOJI`]
//!    scalar and `bbr-space-config-glyph-<name>` per [`BbrGlyph`] catalog
//!    entry. Emoji options render the scalar directly; glyph options render
//!    the drawn §7 path next to their label. The two-letter badge is the
//!    non-curated fallback and never appears in the picker — only curated
//!    values exist here. The selected identity carries the accent outline
//!    plus the `tint.accent`, so identity is never communicated by
//!    selection color alone (the non-selected options remain labeled).
//! 5. Actions — `bbr-space-config-cancel` (Cancel: keeps the created
//!    Space and its valid defaults) and `bbr-space-config-done` (Done:
//!    commit the rename + appearance for the exact bound SpaceId).
//!
//! The surface takes a plain presentation model ([`SpaceConfigModel`]) plus
//! the typed callback bundle ([`callbacks::SpaceConfigCallbacks`]). The
//! name field's text editing, Escape/Enter keyboard behavior, the outside-
//! press dismissal, and every mutation stay app-owned; the surface renders
//! only.

use bbr_domain::{BbrGlyph, SpaceIdentity, SpaceTheme};
use gpui::{
    div, prelude::*, px, AnyElement, App, Div, FocusHandle, MouseButton, MouseDownEvent,
    SharedString, Stateful, Styled, Window,
};

use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::callbacks::{
    SpaceConfigCallbacks, SpaceConfigInsidePress, SpaceConfigSelectIdentity, SpaceConfigSelectTheme,
};
use super::ids::{
    space_config_emoji_id, space_config_glyph_id, space_config_theme_id, SPACE_CONFIG_ACTIONS_ID,
    SPACE_CONFIG_CANCEL_ID, SPACE_CONFIG_DONE_ID, SPACE_CONFIG_EMOJI_LABEL_ID,
    SPACE_CONFIG_GLYPH_LABEL_ID, SPACE_CONFIG_ID, SPACE_CONFIG_IDENTITY_LABEL_ID,
    SPACE_CONFIG_NAME_ID, SPACE_CONFIG_THEME_LABEL_ID, SPACE_CONFIG_TITLE_ID,
};
use super::model::SpaceConfigModel;

/// The §11.3 configuration surface: the in-rail panel over the footer.
/// `focused` drives the §9 focus ring on the whole surface (the keyboard
/// domain of the rename field); `focus` is the surface's chrome focus
/// handle (the surface tracks focus on it, exactly like the §11.12
/// downloads panel); `on_key_down` is the app-layer keyboard seam
/// (printable chars edit the draft, Backspace deletes, Enter commits,
/// Escape cancels).
pub fn space_config_surface(
    theme: &Theme,
    model: &SpaceConfigModel,
    focused: bool,
    focus: &FocusHandle,
    callbacks: &SpaceConfigCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(SPACE_CONFIG_ID)
        .debug_selector(|| SPACE_CONFIG_ID.to_string())
        .track_focus(focus)
        .mx(Spacing::sp2())
        .mb(Spacing::sp1())
        .rounded(theme.radius.medium())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .p(Spacing::sp3())
        .gap(Spacing::sp2())
        .flex()
        .flex_col()
        .min_w_0()
        // §9: the focused surface carries the accent ring (the keyboard
        // cursor domain while the rename field is active).
        .when(focused, |el| el.border_2().border_color(p.accent))
        // §13 one pointer action: a press inside marks itself and stops
        // propagation so the root outside-press handler never dismisses it.
        .on_mouse_down(
            MouseButton::Left,
            callback_press(&callbacks.on_inside_press),
        )
        .child(header(theme, model))
        .child(name_row(theme, model))
        .child(theme_row(theme, model, callbacks))
        .child(identity_row(theme, model, callbacks))
        .child(actions(theme, callbacks))
}

/// The header: `bbr-space-config-title` with the bound Space's name.
fn header(theme: &Theme, model: &SpaceConfigModel) -> Stateful<Div> {
    let title = format!("Configure Space · {}", model.title);
    div()
        .id(SPACE_CONFIG_TITLE_ID)
        .debug_selector(|| SPACE_CONFIG_TITLE_ID.to_string())
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .child(
            div()
                .size(theme.density.icon())
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::ChassisMark.canvas(theme, theme.density.icon())),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(theme.palette.ink)
                .child(title),
        )
}

/// The name row: the `bbr-space-config-name` rename field (the draft). The
/// field is a bordered input the app's keyboard seam edits; the row label
/// keeps the field's accessible identity even when the draft is empty.
fn name_row(theme: &Theme, model: &SpaceConfigModel) -> Div {
    let p = theme.palette;
    let text = if model.name.is_empty() {
        "Space name".into()
    } else {
        model.name.clone()
    };
    div()
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(p.ink_faint)
                .child("NAME"),
        )
        .child(
            div()
                .id(SPACE_CONFIG_NAME_ID)
                .debug_selector(|| SPACE_CONFIG_NAME_ID.to_string())
                .h(theme.density.row())
                .px(Spacing::sp2())
                .items_center()
                .flex()
                .flex_row()
                .rounded(Radius.small())
                .bg(p.surface_field)
                .border_1()
                .border_color(p.border_strong)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(theme.type_.title.size)
                        .font_weight(theme.type_.title.weight)
                        .text_color(if model.name.is_empty() {
                            p.ink_faint
                        } else {
                            p.ink
                        })
                        .child(text),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .child("Enter ↵"),
                ),
        )
}

/// The theme row: the `bbr-space-config-theme-label` group label plus one
/// `bbr-space-config-theme-<name>` swatch per curated token. The swatch
/// fills with the one central [`crate::theme::space_tint`] identity hue for
/// the shell's mode — the same resolver the footer chip consumes — so the
/// picker can never show a color the chip cannot render. The selected
/// swatch carries the §9 accent outline + `tint.accent`.
fn theme_row(theme: &Theme, model: &SpaceConfigModel, callbacks: &SpaceConfigCallbacks) -> Div {
    let swatches: Vec<AnyElement> = SpaceTheme::ALL
        .iter()
        .map(|token| {
            let selected = model.theme == *token;
            theme_swatch(theme, *token, selected, callbacks.on_select_theme.clone()).into_any()
        })
        .collect();
    div()
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .id(SPACE_CONFIG_THEME_LABEL_ID)
                .debug_selector(|| SPACE_CONFIG_THEME_LABEL_ID.to_string())
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_faint)
                .child("THEME"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(Spacing::sp1())
                .children(swatches),
        )
}

/// One curated theme swatch: a bounded rounded tile filled with the token's
/// identity hue plus the canonical token label underneath. The selected
/// tile carries the §9 accent outline; identity is never color-only (the
/// label is the non-color companion).
fn theme_swatch(
    theme: &Theme,
    token: SpaceTheme,
    selected: bool,
    on_select: SpaceConfigSelectTheme,
) -> Stateful<Div> {
    let tint = crate::theme::space_tint(token, theme);
    let p = theme.palette;
    let id = space_config_theme_id(token);
    let id_capture = id.clone();
    div()
        .id(id.clone())
        .debug_selector(move || id.clone())
        .w(px(48.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(Spacing::sp1())
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                let _ = &id_capture;
                on_select(token, window, cx)
            },
        )
        .child(
            div()
                .size(px(24.0))
                .rounded(Radius.small())
                .bg(tint.hue)
                .when(selected, |el| el.border_2().border_color(p.accent))
                .when(!selected, |el| el.border_1().border_color(p.hairline)),
        )
        .child(
            div()
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(if selected { p.ink } else { p.ink_secondary })
                .child(token.label()),
        )
}

/// The identity row: the `bbr-space-config-identity-label` group label plus
/// the two curated families — `bbr-space-config-emoji-<index>` per
/// [`bbr_domain::ALLOWED_EMOJI`] scalar and `bbr-space-config-glyph-<name>`
/// per [`BbrGlyph`] entry. Every option renders its canonical string form
/// (the same form the footer chip's `bbr-space-identity-<n>` renders) and
/// the selected option carries the accent outline + `tint.accent`.
fn identity_row(theme: &Theme, model: &SpaceConfigModel, callbacks: &SpaceConfigCallbacks) -> Div {
    let emoji_options: Vec<AnyElement> = bbr_domain::ALLOWED_EMOJI
        .iter()
        .enumerate()
        .map(|(index, scalar)| {
            let selected = model.emoji_selected(index);
            let identity = SpaceIdentity::Emoji(
                bbr_domain::SpaceEmoji::new(*scalar).expect("curated emoji by construction"),
            );
            identity_option(
                theme,
                space_config_emoji_id(index),
                scalar,
                selected,
                identity,
                callbacks.on_select_identity.clone(),
            )
            .into_any()
        })
        .collect();
    let glyph_options: Vec<AnyElement> = BbrGlyph::ALL
        .iter()
        .map(|glyph| {
            let selected = model.glyph_selected(*glyph);
            let identity = SpaceIdentity::Glyph(*glyph);
            identity_option(
                theme,
                space_config_glyph_id(*glyph),
                glyph.label(),
                selected,
                identity,
                callbacks.on_select_identity.clone(),
            )
            .into_any()
        })
        .collect();
    div()
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .id(SPACE_CONFIG_IDENTITY_LABEL_ID)
                .debug_selector(|| SPACE_CONFIG_IDENTITY_LABEL_ID.to_string())
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_faint)
                .child("IDENTITY"),
        )
        .child(
            div()
                .id(SPACE_CONFIG_EMOJI_LABEL_ID)
                .debug_selector(|| SPACE_CONFIG_EMOJI_LABEL_ID.to_string())
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_faint)
                .child("Emoji"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(Spacing::sp1())
                .children(emoji_options),
        )
        .child(
            div()
                .id(SPACE_CONFIG_GLYPH_LABEL_ID)
                .debug_selector(|| SPACE_CONFIG_GLYPH_LABEL_ID.to_string())
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_faint)
                .child("BBR glyphs"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(Spacing::sp1())
                .children(glyph_options),
        )
}

/// One identity option: a bounded rounded tile carrying the canonical
/// identity rendering — the curated emoji scalar, or the curated glyph's
/// drawn §7 path next to its label (the badge is never used in the picker:
/// only curated values exist here). The selected option carries the §9
/// accent outline + `tint.accent`; the label is the non-color companion,
/// so selection is never color-only.
fn identity_option(
    theme: &Theme,
    id: String,
    label: &str,
    selected: bool,
    identity: SpaceIdentity,
    on_select: SpaceConfigSelectIdentity,
) -> Stateful<Div> {
    let p = theme.palette;
    let label = label.to_string();
    let id_capture = id.clone();
    let content: AnyElement = match identity.clone() {
        SpaceIdentity::Emoji(emoji) => div()
            .text_size(theme.type_.label.size)
            .font_weight(theme.type_.label.weight)
            .text_color(if selected { p.ink } else { p.ink_secondary })
            .child(SharedString::from(emoji.as_str().to_owned()))
            .into_any(),
        SpaceIdentity::Glyph(glyph) => div()
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::sp1())
            .child(crate::icons::Icon::Bbr(glyph).canvas(theme, theme.density.icon()))
            .child(
                div()
                    .text_size(theme.type_.label.size)
                    .font_weight(theme.type_.label.weight)
                    .text_color(if selected { p.ink } else { p.ink_secondary })
                    .child(label.clone()),
            )
            .into_any(),
    };
    div()
        .id(id.clone())
        .debug_selector(move || id.clone())
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.small())
        .cursor_pointer()
        .bg(if selected {
            p.tint_accent
        } else {
            p.surface_field
        })
        .when(selected, |el| el.border_1().border_color(p.accent))
        .when(!selected, |el| el.border_1().border_color(p.hairline))
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                let _ = &id_capture;
                on_select(identity.clone(), window, cx)
            },
        )
        .child(content)
}

/// The action row: `bbr-space-config-cancel` (keeps the created Space and
/// its valid defaults) and `bbr-space-config-done` (commit the rename +
/// appearance for the exact bound SpaceId).
fn actions(theme: &Theme, callbacks: &SpaceConfigCallbacks) -> Stateful<Div> {
    let p = theme.palette;
    let (done_fill, done_text) = super::model::done_colors(&p);
    let on_cancel = callbacks.on_cancel.clone();
    let on_done = callbacks.on_done.clone();
    div()
        .id(SPACE_CONFIG_ACTIONS_ID)
        .debug_selector(|| SPACE_CONFIG_ACTIONS_ID.to_string())
        .flex()
        .flex_row()
        .justify_end()
        .gap(Spacing::sp2())
        .child(
            div()
                .id(SPACE_CONFIG_CANCEL_ID)
                .debug_selector(|| SPACE_CONFIG_CANCEL_ID.to_string())
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.small())
                .cursor_pointer()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(p.ink_secondary)
                .child("Cancel")
                .on_mouse_down(
                    MouseButton::Left,
                    move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on_cancel(window, cx)
                    },
                ),
        )
        .child(
            div()
                .id(SPACE_CONFIG_DONE_ID)
                .debug_selector(|| SPACE_CONFIG_DONE_ID.to_string())
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.small())
                .bg(done_fill)
                .cursor_pointer()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .text_color(done_text)
                .child("Done")
                .on_mouse_down(
                    MouseButton::Left,
                    move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on_done(window, cx)
                    },
                ),
        )
}

/// Adapter: the §13 inside-press marker callback as a concrete GPUI
/// mouse-down listener. GPUI element listeners require concrete `Fn`
/// closures (they are never `Rc<dyn Fn>`-callable), so the slot is cloned
/// into a small forwarding closure at the composition boundary — the same
/// adapter shape the sidebar rail uses for its `Rc<dyn Fn>` slots. The
/// product authority stays in the caller (`bbr-app` fills the slot with the
/// production inside-press marker; the gallery fills recorder callbacks).
fn callback_press(
    callback: &SpaceConfigInsidePress,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
    let callback = callback.clone();
    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| callback(event, window, cx)
}
