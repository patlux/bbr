//! Sidebar address control (DESIGN.md §11.5) and the non-address command
//! staging surface — the plain presentation halves of the app's sidebar
//! command slot.
//!
//! The sidebar address is a **display control, not a text field**: at rest
//! (and behind an open palette/overlay presentation) the slot renders the
//! domain pill `bbr-address` — borderless, translucent (token pair driven by
//! theme and space tint), hover wash, button semantics, a reserved trailing
//! accessory slot, and no border or focus ring in any state. All address
//! text entry lives in the §11.4 palette presentations; the pill holds no
//! editing state, cursor or selection and never accepts keyboard text.
//!
//! The same slot renders a separate **command staging surface**
//! (`bbr-command-bar`) only for the non-address staged modes — New Space,
//! Move Tab, and the §11.8 empty New Tab surface — which collect a space
//! name (never an address) and keep their existing field styling, cursor,
//! and hints. The address control itself is never that surface.
//!
//! Both components are pure presentation: the caller (the app's
//! `sidebar_view` command slot) owns the mode authority, the domain
//! derivation ([`crate::text`]), the §11.4 field anchor, the controller
//! snapshot, and the `cx.listener` closures; it builds the DTOs below and
//! wires the callbacks. The ids and the element tree are byte-identical to
//! the pre-cutover app render, so §15 semantics never drift.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use super::space::{profile_chip, ProfileChipSpec};
use crate::text::accessory_slot_width;
use crate::theme::{Spacing, Theme};

/// The §11.3 cross-context profile hint rendered at the pill's leading edge
/// for its one-interaction lifetime: the §12 chip (creation-order index,
/// monogram, hue) plus the hint label. Derived by the app from the active
/// profile; never the only identity cue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressProfileHint {
    /// The profile's creation-order index (the `bbr-profile-chip-<n>` id).
    pub profile_index: usize,
    /// The §12 chip identity (monogram + hue).
    pub spec: ProfileChipSpec,
    /// The hint text rendered next to the chip.
    pub label: SharedString,
}

/// The §11.5 address pill's presentation model. The app derives the domain
/// label from the live URL via [`crate::text::address_domain`] +
/// [`crate::text::address_domain_budget_chars`] + [`crate::text::mid_ellipsis`];
/// the pill itself holds no editing state.
#[derive(Debug, Clone)]
pub struct AddressModel {
    /// The mid-ellipsis'd domain label.
    pub domain: SharedString,
    /// §10.2 hover wash: the pointer hovers the pill.
    pub hovered: bool,
    /// The §11.3 cross-context profile hint; `None` at rest.
    pub profile_hint: Option<AddressProfileHint>,
}

/// The §11.5 command **staging** surface's presentation model (`bbr-command-bar`):
/// the input shown only while a non-address staged mode is active without a
/// floating presentation — New Space, Move Tab, and the §11.8 empty New Tab
/// surface. The caller resolves the `CommandBarMode` authority (which stays
/// app-owned with its testkit protocol mapping) into the presentation fields
/// below; the surface itself only renders.
#[derive(Debug, Clone)]
pub struct StagingModel {
    /// §9: whether the browser chrome is focused (the field background and
    /// the accent border follow it).
    pub focused: bool,
    /// The staged command text. Empty renders the "Type a URL…" placeholder
    /// (the legacy field's hint), exactly like the pre-cutover surface.
    pub text: String,
    /// The §11.3 hint suffix, if any.
    pub hint: Option<SharedString>,
    /// Whether the block cursor renders — `chrome_focused && mode.is_active()`
    /// resolved by the caller.
    pub show_cursor: bool,
    /// The §11.3 cross-context profile hint; `None` at rest.
    pub profile_hint: Option<AddressProfileHint>,
}

/// The §11.5 address control: a borderless translucent pill carrying the
/// domain only, with a reserved trailing accessory slot. `type:title`
/// domain text (`bbr-address-domain`, mid-ellipsis), the slot
/// (`bbr-address-accessory`) reserving one §14 glyph's `density:hit-min`
/// width so a later icon changes no layout, hover wash on both chrome
/// polarities, button semantics (pointer press opens the §11.4 overlay
/// over the control), and — per §11.5 — **no border and no focus ring
/// at any time**. The §11.3 cross-context profile hint renders at the
/// leading edge for its one-interaction lifetime. `field_anchor` is the
/// caller-built invisible §11.4 field anchor overlaying the pill to record
/// its window-relative rect; `on_hover`/`on_open` are the production
/// `cx.listener` closures.
pub fn address_control(
    theme: &Theme,
    model: &AddressModel,
    field_anchor: gpui::AnyElement,
    on_hover: impl Fn(&bool, &mut Window, &mut App) + 'static,
    on_open: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id("bbr-address")
        // The relative box is the containing block for the absolute
        // field anchor (§11.4 overlay anchor).
        .relative()
        .mx(Spacing::sp1())
        .my(Spacing::sp2())
        .px(Spacing::sp3())
        // §11.5: pill height = `density:row` (S 32 / N 36 / XL 44);
        // content centers inside the fixed-height pill.
        .h(theme.density.row())
        .rounded(theme.radius.large())
        // §11.5 fill: translucent token pair (white on dark chrome,
        // black on light chrome) following theme and space tint through
        // the material beneath; the hover wash steps one fill brighter
        // (dark) / darker (light) at `dur:fine`. No border, no ring.
        .bg(if model.hovered {
            p.address_fill_hover
        } else {
            p.address_fill
        })
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .cursor_pointer()
        // §10.2 production hover path: the wash follows the pointer.
        .on_hover(on_hover)
        // §11.5 activation: pointer press opens the §11.4 overlay with
        // the full URL selected (the production pointer entry, same
        // engine as Cmd-L). The caller's listener stops propagation so the
        // sidebar's click handler (which cancels the command surface)
        // never runs after it.
        .on_mouse_down(MouseButton::Left, on_open)
        .when(model.profile_hint.is_some(), |el| {
            let hint = model.profile_hint.as_ref().expect("checked above");
            el.child(
                div()
                    .id("bbr-command-profile-hint")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(Spacing::sp2())
                    .child(profile_chip(theme, hint.profile_index, &hint.spec))
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .font_weight(theme.type_.label.weight)
                            .text_color(p.ink_secondary)
                            .child(hint.label.clone()),
                    ),
            )
        })
        .child(
            div()
                .id("bbr-address-domain")
                // §11.5: the domain text yields width by ellipsis
                // (mid-ellipsis at the char budget, GPUI `text_ellipsis`
                // as the pixel-exact backstop); it never pushes the
                // reserved accessory out of the pill.
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(model.domain.clone()),
        )
        // §11.5 trailing accessory slot: reserved geometry (one §14
        // glyph at `density:hit-min`), `flex-shrink-0`, empty in v1
        // (the §17.14 decision stays open). The slot's width never
        // changes, so later icons cause no layout change.
        .child(
            div()
                .id("bbr-address-accessory")
                .w(accessory_slot_width(theme.density))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center(),
        )
        // The §11.4 field anchor overlays the pill (absolute, inset-0):
        // invisible, takes no layout space, records the pill rect.
        .child(field_anchor)
}

/// The §11.5 command **staging** surface (`bbr-command-bar`): the input
/// shown only while a non-address staged mode is active without a
/// floating presentation — New Space, Move Tab, and the §11.8 empty New
/// Tab surface. It collects a space name (never an address), keeps its
/// existing field styling, block cursor, and the §11.3 hints, and opens
/// the §11.4 overlay on pointer press like the legacy field. The address
/// control above is never this surface; the two are mutually exclusive by
/// the caller's `command_bar_mode` decision.
pub fn staging_surface(
    theme: &Theme,
    model: &StagingModel,
    field_anchor: gpui::AnyElement,
    on_open: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id("bbr-command-bar")
        .relative()
        .mx(Spacing::sp1())
        .my(Spacing::sp2())
        .px(Spacing::sp3())
        .h(theme.density.row())
        .rounded(theme.radius.large())
        .bg(if model.focused {
            p.surface_field
        } else {
            p.surface_sidebar
        })
        .on_mouse_down(MouseButton::Left, on_open)
        .border_2()
        .border_color(if model.focused { p.accent } else { p.hairline })
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .when(model.profile_hint.is_some(), |el| {
            let hint = model.profile_hint.as_ref().expect("checked above");
            el.child(
                div()
                    .id("bbr-command-profile-hint")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(Spacing::sp2())
                    .child(profile_chip(theme, hint.profile_index, &hint.spec))
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .font_weight(theme.type_.label.weight)
                            .text_color(p.ink_secondary)
                            .child(hint.label.clone()),
                    ),
            )
        })
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child("▶"),
        )
        .child(
            div()
                .id("bbr-command-text")
                .flex_1()
                .text_size(theme.type_.mono.size)
                .text_color(p.ink)
                .child(if model.text.is_empty() {
                    SharedString::from("Type a URL…")
                } else {
                    SharedString::from(model.text.clone())
                }),
        )
        .when(model.show_cursor, |el| {
            el.child(
                div()
                    .id("bbr-command-cursor")
                    .w(px(2.0))
                    .h_full()
                    .bg(p.accent),
            )
        })
        .when(model.hint.is_some(), |el| {
            el.child(
                div()
                    .id("bbr-command-hint")
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_secondary)
                    .child(model.hint.clone().expect("hint is some")),
            )
        })
        .child(field_anchor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_models_round_trip_their_fields() {
        let hint = AddressProfileHint {
            profile_index: 1,
            spec: ProfileChipSpec {
                letter: "PR".into(),
                hue: bbr_domain::ProfileHue::Violet,
            },
            label: "Switch to Private".into(),
        };
        let model = AddressModel {
            domain: "example.com".into(),
            hovered: false,
            profile_hint: Some(hint.clone()),
        };
        assert_eq!(model.domain, "example.com");
        assert!(!model.hovered);
        let carried = model.profile_hint.clone().expect("hint set");
        assert_eq!(carried.profile_index, 1);
        assert_eq!(carried.spec.letter, "PR");
        assert_eq!(carried.label, "Switch to Private");
    }

    #[test]
    fn staging_models_carry_the_presentation_fields() {
        let model = StagingModel {
            focused: true,
            text: "Reading".into(),
            hint: Some("New Space".into()),
            show_cursor: true,
            profile_hint: None,
        };
        assert!(model.focused);
        assert_eq!(model.text, "Reading");
        assert!(model.show_cursor);
        assert_eq!(model.hint.as_deref(), Some("New Space"));
        assert!(model.profile_hint.is_none());
    }
}
