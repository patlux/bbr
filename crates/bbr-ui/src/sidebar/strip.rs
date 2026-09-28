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
//! surface and stronger type weight; inactive chips stay quiet.
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

use crate::icons::{classify_identity, IdentityForm};
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
    let chips = centre_chips(chips);
    div()
        .id(SPACE_STRIP_ID)
        .debug_selector(|| SPACE_STRIP_ID.to_string())
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

/// §18.3: the chips centre in the strip viewport while they fit, and start
/// at the leading edge once they overflow. Auto margins on the first and
/// last chip give exactly that "safe centre": they absorb positive free
/// space equally and resolve to zero when the content overflows, so the
/// scroll origin never clips a chip (plain `justify-content: center` would
/// push overflow off the leading, unscrollable side). The scroll handle
/// still tracks the chips themselves as its direct children.
fn centre_chips(chips: Vec<Stateful<Div>>) -> Vec<Stateful<Div>> {
    let last = chips.len().saturating_sub(1);
    chips
        .into_iter()
        .enumerate()
        .map(|(index, chip)| {
            let chip = if index == 0 { chip.ml_auto() } else { chip };
            if index == last {
                chip.mr_auto()
            } else {
                chip
            }
        })
        .collect()
}

/// One §11.3 identity chip: `[identity]`, `radius:sm`, with
/// the persistent selected surface on the active space and the
/// §10.9 drag-target treatment (accent outline + `tint.accent` + the §12
/// destination-profile companion) while a pointer drag targets this exact
/// chip. DESIGN.md §11.3: the footer chip is **icon-only at rest** — the
/// validated identity renders, no Space title text appears in the footer
/// (the accessible/semantic name still carries the Space and Profile names,
/// so identity is never color-only). `on_select` is the production view
/// listener (the same app method the keyboard traversal drives).
/// `swipe_tint` carries the chrome-only interpolated presentation while a
/// sidebar swipe tracks (`None` at rest): the active chip's surface follows
/// the gesture progress toward the adjacent space while
/// `visuals.selected`/`visuals.theme` stay truthful. Chips never shrink
/// inside the scroll viewport (`flex_shrink_0`), so the strip's content
/// width is the sum of its chips and the selected chip can be scrolled into
/// view deterministically.
/// The §7 two-letter badge fallback for a non-curated identity value
/// (DESIGN.md §11.3: the badge is the missing-glyph fallback only — every
/// curated glyph renders its drawn path, every curated emoji renders
/// directly).
fn badge_fallback(identity: &str) -> SharedString {
    SharedString::from(
        identity
            .chars()
            .take(2)
            .flat_map(char::to_uppercase)
            .collect::<String>(),
    )
}

/// The §11.3 identity mark of one validated `SpaceIdentity` value: a
/// curated emoji directly, a curated catalog name as its drawn §7 path, a
/// non-curated value as the §7 two-letter badge. The one renderer the
/// footer chip and the active-space heading share, so a space never shows
/// two different marks.
pub fn identity_mark(
    theme: &Theme,
    identity: &str,
    text_weight: gpui::FontWeight,
) -> gpui::AnyElement {
    let p = theme.palette;
    match classify_identity(identity) {
        IdentityForm::Emoji => div()
            .text_size(theme.type_.badge.size)
            .font_weight(text_weight)
            .text_color(p.ink)
            .child(SharedString::from(identity.to_string()))
            .into_any(),
        IdentityForm::Glyph(glyph) => crate::icons::Icon::Bbr(glyph)
            .canvas(theme, theme.density.icon())
            .into_any(),
        IdentityForm::Badge => div()
            .text_size(theme.type_.badge.size)
            .font_weight(text_weight)
            .text_color(p.ink)
            .child(badge_fallback(identity))
            .into_any(),
    }
}

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
    let tint = crate::theme::space_tint(visuals.theme, visuals.intensity, theme);
    let chip_surface = if selected {
        swipe_tint.map_or(tint.chip_active, |swipe| swipe.chip_active)
    } else {
        // Inactive chips paint nothing: the footer veil and the rail tint
        // show through (§18.2).
        crate::components::tab_row::REST_SURFACE
    };
    // §11.3/§7: the identity renders its drawn §7 glyph path when the value
    // is a curated `BbrGlyph`, the emoji scalar directly when curated, and
    // the two-letter badge only for a non-curated value (the missing-glyph
    // fallback). The text weight steps up on the active chip; the glyph
    // canvas is the same ink at rest and selected (the §9 ring and the
    // surface carry selection).
    let text_weight = if selected {
        gpui::FontWeight(600.0)
    } else {
        theme.type_.badge.weight
    };
    let identity_slot = identity_mark(theme, &visuals.identity, text_weight);
    let space = visuals.space;
    div()
        .id(space_chip_id(space))
        // Test-support only (a release no-op): the chip as a geometry
        // target for the §18.3 centred-strip evidence.
        .debug_selector(move || space_chip_id(space).to_string())
        .flex_shrink_0()
        .size(theme.density.row() - px(4.0))
        .px(Spacing::sp1())
        .gap(Spacing::sp2())
        .items_center()
        .flex()
        .flex_row()
        .rounded(Radius.small())
        .cursor_pointer()
        .bg(if target {
            // §10.9/§18.2 drag-target: `tint.accent` + accent outline — the
            // targeted chip is unmistakable without being the active chip.
            p.tint_accent
        } else {
            chip_surface
        })
        .when(chip_lifts(selected, target), |el| {
            // §18.2/D2: the active chip is the lifted selection surface
            // carrying 8 % of the identity hue; the lift's own 1 pt ring
            // replaces the old hairline edge. Exact identity + stronger type
            // weight are the non-color cues; the strip's keyboard focus owns
            // the accent ring, so rest state carries no decorative rail.
            el.shadow(crate::theme::elevation::lift(theme))
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
                .flex()
                .items_center()
                .justify_center()
                .child(identity_slot),
        )
        .on_mouse_down(MouseButton::Left, on_select)
}

/// D2: the active chip lifts (painted with its surface, never animated);
/// a drag-targeted chip shows the §10.9 treatment instead.
pub fn chip_lifts(selected: bool, drag_target: bool) -> bool {
    selected && !drag_target
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_domain::SpaceTheme;

    #[test]
    fn only_the_active_undragged_chip_lifts() {
        assert!(chip_lifts(true, false));
        assert!(!chip_lifts(true, true));
        assert!(!chip_lifts(false, false));
        assert!(!chip_lifts(false, true));
    }

    /// A chip visual for a persistent space; the fields that are irrelevant
    /// to id stability vary per caller.
    fn visuals(space: SpaceId, title: &str) -> SpaceChipVisuals {
        SpaceChipVisuals {
            space,
            title: title.into(),
            profile_title: "Default".into(),
            identity: "book".into(),
            theme: SpaceTheme::Ink,
            intensity: bbr_domain::TintIntensity::Standard,
            selected: false,
            drag_target: false,
            companion: None,
        }
    }

    #[test]
    fn identity_forms_classify_curated_values_and_keep_the_badge_fallback() {
        // DESIGN.md §11.3/§7: a curated emoji scalar renders directly, a
        // curated glyph name renders its drawn path, and only a non-curated
        // value falls back to the two-letter badge.
        assert_eq!(classify_identity("🔥"), IdentityForm::Emoji);
        assert_eq!(
            classify_identity("code"),
            IdentityForm::Glyph(bbr_domain::BbrGlyph::Code)
        );
        assert_eq!(
            classify_identity("briefcase"),
            IdentityForm::Glyph(bbr_domain::BbrGlyph::Briefcase)
        );
        assert_eq!(classify_identity("CO"), IdentityForm::Badge);
        assert_eq!(classify_identity("not-curated"), IdentityForm::Badge);
        assert_eq!(badge_fallback("not-curated"), "NO");
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
            profile_title: "Private".into(),
            identity: "lock".into(),
            theme: SpaceTheme::Violet,
            intensity: bbr_domain::TintIntensity::Standard,
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
            profile_title: "Default".into(),
            identity: "code".into(),
            theme: SpaceTheme::Cobalt,
            intensity: bbr_domain::TintIntensity::Standard,
            selected: true,
            drag_target: false,
            companion: None,
        };
        let tint = crate::theme::space_tint(
            SpaceTheme::Cobalt,
            crate::theme::TintIntensity::Standard,
            &theme,
        );
        // The selected chip's surface is the tinted chip surface — a
        // stronger blend than the rail — distinct from the untinted
        // material and from the generic selection token.
        assert_ne!(tint.chip_active, theme.palette.surface_sidebar);
        assert_ne!(tint.chip_active, theme.palette.tint_selected);
        assert_eq!(selected.theme, SpaceTheme::Cobalt);
    }
}
