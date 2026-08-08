//! §12 profile chip identity (and the canonical space-surface row type).
//!
//! The profile chip renders wherever a profile's §12 identity is needed —
//! the §11.3 command-field cross-context hint — keyed to the profile's
//! creation-order index, never the space index. The revealed `space_row` /
//! `space_new_row` list components were removed with the legacy §11.4
//! `[Spaces]` toggle: the permanent space strip renders identity chips
//! instead (see [`super::space_strip`]).

use gpui::{div, prelude::*, Div, SharedString, Stateful, Styled};

use crate::palette::ProfileHueRamp;
use crate::theme::{Radius, Theme};

use super::ProfileChipSpec;

/// The §15 semantic id of a profile chip, keyed to the *profile* index
/// (DESIGN.md §12) — never the space index. The §11.3 command-field
/// cross-context hint renders one chip for the destination profile; this
/// function is the single keying both the render and the snapshot share.
pub fn profile_chip_id(profile_index: usize) -> SharedString {
    SharedString::from(format!("bbr-profile-chip-{profile_index}"))
}

/// §12 profile chip: 24 pt square (20 at S, 28 at XL), `radius:sm`,
/// filled with the profile's hue tile and a 1–2 character monogram. The
/// letter derives from the profile title so identity is readable per row
/// without a header.
pub fn profile_chip(theme: &Theme, profile_index: usize, spec: &ProfileChipSpec) -> Stateful<Div> {
    let size = theme.density.profile_chip();
    let tile = ProfileHueRamp::tile(spec.hue, theme.mode);
    div()
        .id(profile_chip_id(profile_index))
        .size(size)
        .rounded(Radius.small())
        .bg(tile.tile)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(tile.letter)
        .child(spec.letter.clone())
}
