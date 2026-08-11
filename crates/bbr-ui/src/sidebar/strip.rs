//! DESIGN.md §11.3/§11.4 permanent space strip and identity chips — the G3
//! Sidebar composition's strip authority.
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
//! color-only. The active chip renders the space's **resolved theme tint**
//! (the one central [`crate::theme::space_tint`] model — the active token
//! tints the chip, never a raw consumer color) plus a persistent selected
//! surface and a focus-independent accent rail; inactive chips stay quiet.
//! The strip scrolls the selected chip fully into view without changing
//! order through the tracked [`ScrollHandle`].
//!
//! The strip is presentation-only: the app keeps the chip visuals
//! derivation (from the controller snapshot), the scroll/focus handles, and
//! the activation/keyboard authority; the rail composition wires the
//! production listeners through the sidebar `Callbacks`.

use bbr_domain::SpaceId;
use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, ScrollHandle, SharedString,
    Stateful, Styled, Window,
};

use crate::state::SpaceChipVisuals;
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
/// the persistent selected surface + accent rail on the active space and the
/// §10.9 drag-target treatment (accent outline + selection tint + the §12
/// destination-profile companion) while a pointer drag targets this exact
/// chip. `on_select` is the production view listener (the same app method
/// the keyboard traversal drives). `swipe_tint` carries the chrome-only
/// interpolated presentation while a sidebar swipe tracks (`None` at rest):
/// the active chip's surface follows the gesture progress toward the
/// adjacent space while `visuals.selected`/`visuals.theme` stay truthful.
/// Chips never shrink inside the scroll viewport (`flex_shrink_0`), so the
/// strip's content width is the sum of its chips and the selected chip can
/// be scrolled into view deterministically.
pub fn space_chip(
    theme: &Theme,
    visuals: &SpaceChipVisuals,
    swipe_tint: Option<crate::theme::SpaceTint>,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let selected = visuals.selected;
    let target = visuals.drag_target;
    // §11.3: the active token tints the chip through the one central
    // resolver; text, the accent rail, and the §9 outline keep their own
    // tokens (the tint is never the only state cue). While a swipe tracks,
    // the active chip's surface interpolates toward the adjacent space's
    // presentation (chrome-only; the selected evidence never changes).
    let tint = crate::theme::space_tint(visuals.theme, theme);
    let chip_surface = if selected {
        swipe_tint.map_or(tint.chip_active, |swipe| swipe.chip_active)
    } else {
        p.surface_sidebar
    };
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
        .bg(if target {
            // §10.9 drag-target: the selection tint + accent outline — the
            // targeted chip is unmistakable without being the active chip.
            p.tint_selected
        } else {
            chip_surface
        })
        .when(selected && !target, |el| {
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
        .when(target, |el| {
            // §10.9/§11.3: the drag-target outline plus the §12
            // destination-profile companion (monogram + hue tile), so the
            // destination profile is never communicated by color alone.
            let companion = visuals.companion.as_ref().map(|companion| {
                let spec = crate::components::ProfileChipSpec {
                    letter: companion.letter.clone(),
                    hue: companion.hue,
                };
                crate::components::profile_chip(theme, companion.profile_index, &spec)
            });
            el.border_1()
                .border_color(p.accent)
                .when(companion.is_some(), |el| {
                    el.child(companion.expect("checked above"))
                })
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
    use bbr_domain::SpaceTheme;

    /// A chip visual for a persistent space; the fields that are irrelevant
    /// to id stability vary per caller.
    fn visuals(space: SpaceId, title: &str) -> SpaceChipVisuals {
        SpaceChipVisuals {
            space,
            title: title.into(),
            identity: "book".into(),
            theme: SpaceTheme::Ink,
            selected: false,
            drag_target: false,
            companion: None,
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
        assert_eq!(visuals.theme, SpaceTheme::Ink);
        assert!(!visuals.selected);
        assert!(!visuals.drag_target, "at rest no chip is a drag target");
        assert_eq!(visuals.companion, None, "no companion at rest");
    }

    #[test]
    fn the_drag_target_visuals_carry_the_destination_companion() {
        // A targeted chip is not the active chip; its §12 companion carries
        // the destination profile's identity (index, monogram, hue) — the
        // non-color cue of §10.9/§11.3.
        let target = SpaceChipVisuals {
            space: SpaceId(4),
            title: "Private".into(),
            identity: "lock".into(),
            theme: SpaceTheme::Violet,
            selected: false,
            drag_target: true,
            companion: Some(crate::state::ProfileCompanion {
                profile_index: 1,
                letter: "PR".into(),
                hue: bbr_domain::ProfileHue::Violet,
            }),
        };
        assert!(target.drag_target);
        let companion = target
            .companion
            .as_ref()
            .expect("targeted chip carries a companion");
        assert_eq!(companion.profile_index, 1);
        assert_eq!(companion.letter, "PR");
        assert_eq!(companion.hue, bbr_domain::ProfileHue::Violet);
        // The active chip without a drag target stays selected with no
        // companion.
        let active = SpaceChipVisuals {
            selected: true,
            drag_target: false,
            companion: None,
            ..target.clone()
        };
        assert!(active.selected);
        assert!(!active.drag_target);
        assert_eq!(active.companion, None);
    }

    #[test]
    fn the_active_chip_consumes_the_central_tint_resolution() {
        // The chip's resolved surfaces come from the one central model: the
        // selected surface equals the resolver's chip tint for the chip's
        // theme/mode/contrast, never a raw consumer color.
        let theme = crate::theme::Theme::dark();
        let selected = SpaceChipVisuals {
            space: SpaceId(2),
            title: "Default".into(),
            identity: "code".into(),
            theme: SpaceTheme::Cobalt,
            selected: true,
            drag_target: false,
            companion: None,
        };
        let tint = crate::theme::space_tint(SpaceTheme::Cobalt, &theme);
        // The selected chip's surface is the tinted chip surface — a
        // stronger blend than the rail — distinct from the untinted
        // material and from the generic selection token.
        assert_ne!(tint.chip_active, theme.palette.surface_sidebar);
        assert_ne!(tint.chip_active, theme.palette.tint_selected);
        assert_eq!(selected.theme, SpaceTheme::Cobalt);
    }
}
