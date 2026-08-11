//! §12 profile chip identity (and the canonical space-surface row type).
//!
//! The profile chip renders wherever a profile's §12 identity is needed —
//! the §11.3 command-field cross-context hint — keyed to the profile's
//! creation-order index, never the space index. The revealed `space_row` /
//! `space_new_row` list components were removed with the legacy §11.4
//! `[Spaces]` toggle: the permanent space strip renders identity chips
//! instead (see `bbr_ui::sidebar::strip`).

use bbr_domain::ProfileHue;
use gpui::{div, prelude::*, Div, SharedString, Stateful, Styled};

use crate::palette::ProfileHueRamp;
use crate::theme::{Radius, Theme};

/// §12 profile chip identity: the 1–2 character monogram plus the stable
/// round-robin hue the domain assigned to the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileChipSpec {
    pub letter: String,
    pub hue: ProfileHue,
}

/// One canonical space-surface row identity, derived from the controller
/// snapshot: the persistent-space lookup key and the §4-truncated title. The
/// `space_groups` derivation is the single canonical order the permanent
/// §11.3 strip (chips), the keyboard traversal, and the §15 id derivation
/// all consume.
#[derive(Debug, Clone)]
pub struct SpaceRowVisuals {
    /// The space's creation-order index (the persistent-space lookup key).
    pub index: usize,
    pub title: SharedString,
}

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
        .debug_selector(move || profile_chip_id(profile_index).to_string())
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
