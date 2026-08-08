//! DESIGN.md §11.3/§11.4 permanent space strip and identity chips.
//!
//! The footer strip renders every space as one keyboard-focusable identity
//! chip in canonical grouped order (profile groups, creation order within
//! each profile) inside a single-line horizontal scroll viewport. The strip
//! consumes all footer width left after `[Downloads]` and before `[+]`; it
//! clips and scrolls, never wraps, and never shrinks the fixed controls.
//!
//! Each chip contains exactly one validated `SpaceIdentity` value from the
//! curated appearance contract (a selected emoji or a BBR icon glyph name —
//! the §7 glyph paths themselves are a later icon-system slice, so the
//! identity renders its canonical string form here). The chip's accessible
//! name carries the space and profile names, so identity is never
//! color-only. The active chip renders a persistent selected surface plus a
//! focus-independent accent rail; inactive chips stay quiet. The strip
//! scrolls the selected chip fully into view without changing order through
//! the tracked [`ScrollHandle`].

use bbr_browser::SpaceId;
use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, ScrollHandle, SharedString,
    Stateful, Styled, Window,
};

use crate::theme::{Radius, Spacing, Theme};

/// The §15 id of the permanent space strip.
pub const SPACE_STRIP_ID: &str = "bbr-space-strip";

/// The §15 id of one space chip, derived from the persistent [`SpaceId`]
/// (`bbr-space-chip-<persistent-space-id>`). The id never derives from the
/// render position, the space's creation-order index, or the profile index:
/// a space keeps the same chip id wherever it appears in the strip.
pub fn space_chip_id(space: SpaceId) -> SharedString {
    SharedString::from(format!("bbr-space-chip-{}", space.0))
}

/// The §15 id of one chip's identity element, derived from the same
/// persistent [`SpaceId`] (`bbr-space-identity-<persistent-space-id>`).
pub fn space_identity_id(space: SpaceId) -> SharedString {
    SharedString::from(format!("bbr-space-identity-{}", space.0))
}

/// One rendered space chip's identity, derived from the controller snapshot
/// plus the effective space appearance (the portable appearance contract's
/// validated identity). The render, the snapshot, and the semantic tree all
/// consume this same source so the chip ids can never drift.
///
/// Display ordering is the caller's vector order (canonical grouped order);
/// the persistent id is [`Self::space`] alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceChipVisuals {
    /// The persistent space identity — the only identity of the chip ids
    /// (`bbr-space-chip-<persistent-space-id>`). Never the render position,
    /// the creation-order index, or the profile index.
    pub space: SpaceId,
    /// The space title after §4 truncation.
    pub title: String,
    /// The validated curated identity in its canonical string form (the
    /// emoji scalar or the glyph name).
    pub identity: String,
    /// Whether this is the active space (the chip renders selected).
    pub selected: bool,
}

/// The §11.3 space strip: the single-line horizontal scroll viewport that
/// holds the identity chips. `chips` are the caller-built chip elements in
/// canonical order (the strip scrolls to the selected chip via the tracked
/// handle); `focus`/`on_key_down` are the app-layer keyboard seams.
pub fn space_strip(
    theme: &Theme,
    scroll: ScrollHandle,
    chips: Vec<Stateful<Div>>,
) -> Stateful<Div> {
    div()
        .id(SPACE_STRIP_ID)
        .flex_1()
        .min_w_0()
        .h(theme.density.row())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .overflow_x_scroll()
        .track_scroll(&scroll)
        .children(chips)
}

/// One §11.3 identity chip: `[identity] [title, trunc]`, `radius:sm`, with
/// the persistent selected surface + accent rail on the active space.
/// `on_select` is the production view listener (the same app method the
/// keyboard traversal drives). Chips never shrink inside the scroll
/// viewport (`flex_shrink_0`), so the strip's content width is the sum of
/// its chips and the selected chip can be scrolled into view determinis-
/// tically.
pub fn space_chip(
    theme: &Theme,
    visuals: &SpaceChipVisuals,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = visuals.selected;
    div()
        .id(space_chip_id(visuals.space))
        .flex_shrink_0()
        .h(theme.density.row() - px(4.0))
        .px(Spacing::sp2())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.small())
        .cursor_pointer()
        .bg(if selected {
            p.tint_selected
        } else {
            p.surface_sidebar
        })
        .when(selected, |el| {
            // The persistent selected surface plus a focus-independent
            // accent rail — the active chip is unmistakable without color
            // (the §9 outline rides the strip's keyboard focus).
            el.border_1().border_color(p.accent).child(
                div()
                    .w(px(2.0))
                    .h(theme.density.row() - px(10.0))
                    .rounded(Radius.small())
                    .bg(p.accent),
            )
        })
        .child(
            div()
                .id(space_identity_id(visuals.space))
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(p.ink)
                .child(visuals.identity.clone()),
        )
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .max_w(px(96.0))
                .text_size(theme.type_.meta.size)
                .font_weight(theme.type_.meta.weight)
                .text_color(if selected { p.ink } else { p.ink_secondary })
                .child(visuals.title.clone()),
        )
        .on_mouse_down(MouseButton::Left, on_select)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chip visual for a persistent space; the fields that are irrelevant
    /// to id stability vary per caller.
    fn visuals(space: SpaceId, title: &str) -> SpaceChipVisuals {
        SpaceChipVisuals {
            space,
            title: title.into(),
            identity: "book".into(),
            selected: false,
        }
    }

    #[test]
    fn chip_ids_derive_from_the_persistent_space_id() {
        assert_eq!(space_chip_id(SpaceId(7)), "bbr-space-chip-7");
        assert_eq!(space_chip_id(SpaceId(4242)), "bbr-space-chip-4242");
        assert_eq!(space_identity_id(SpaceId(7)), "bbr-space-identity-7");
        assert_eq!(SPACE_STRIP_ID, "bbr-space-strip");
    }

    #[test]
    fn chip_ids_are_stable_across_positions_and_order() {
        // The same persistent space keeps the same chip + identity ids no
        // matter where it appears in the strip: identity never derives from
        // the render position or the vector order.
        let first = visuals(SpaceId(3), "Reading");
        let last = visuals(SpaceId(3), "Private");
        assert_eq!(space_chip_id(first.space), space_chip_id(last.space));
        assert_eq!(
            space_identity_id(first.space),
            space_identity_id(last.space)
        );
        assert_eq!(space_chip_id(first.space), "bbr-space-chip-3");
        // Reordering the same spaces never changes their ids.
        let order_a = [first.space, SpaceId(9)];
        let order_b = [SpaceId(9), first.space];
        let ids_a: Vec<SharedString> = order_a.iter().map(|s| space_chip_id(*s)).collect();
        let ids_b: Vec<SharedString> = order_b.iter().map(|s| space_chip_id(*s)).collect();
        assert!(ids_a.contains(&space_chip_id(first.space)));
        assert!(ids_b.contains(&space_chip_id(first.space)));
        // Distinct spaces render distinct ids.
        assert_ne!(space_chip_id(SpaceId(3)), space_chip_id(SpaceId(9)));
    }

    #[test]
    fn chip_visuals_carry_persistent_identity_not_position() {
        let visuals = visuals(SpaceId(42), "Reading");
        assert_eq!(visuals.space, SpaceId(42));
        assert_eq!(visuals.identity, "book");
        assert!(!visuals.selected);
    }
}
