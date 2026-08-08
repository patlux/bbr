//! Curated per-Space theme-token palette.

use crate::ProfileHue;

/// A curated per-Space theme token (product spec §1).
///
/// v1 deliberately offers **no free color picker**: a space's theme is
/// exactly one of these stable tokens. The first six tokens are the
/// DESIGN.md §12 profile identity hues, so the existing round-robin
/// profile identity "goes into" this mechanism through
/// [`SpaceTheme::from_profile_hue`]; the remaining tokens are curated
/// space-only themes without a profile-hue counterpart.
///
/// The domain carries only the stable curated token identity. Concrete
/// light/dark surface, tint, and text-contrast colors are a presentation
/// concern resolved by the shell's token layer from this name; the domain
/// never resolves RGB values (DESIGN.md §3/§15 keep color resolution in
/// the presentation layer, derived, not copied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpaceTheme {
    /// DESIGN.md §12 Cobalt (profile identity hue).
    Cobalt,
    /// DESIGN.md §12 Violet (profile identity hue).
    Violet,
    /// DESIGN.md §12 Fuchsia (profile identity hue).
    Fuchsia,
    /// DESIGN.md §12 Amber (profile identity hue).
    Amber,
    /// DESIGN.md §12 Terracotta (profile identity hue).
    Terracotta,
    /// DESIGN.md §12 Slate (profile identity hue).
    Slate,
    /// Curated space-only deep pine green.
    Pine,
    /// Curated space-only deep plum.
    Plum,
    /// Curated space-only neutral ink.
    Ink,
}

impl SpaceTheme {
    /// All curated tokens in stable palette order. Identity hues first,
    /// mirroring [`ProfileHue::ALL`], then the space-only tokens.
    pub const ALL: [SpaceTheme; 9] = [
        SpaceTheme::Cobalt,
        SpaceTheme::Violet,
        SpaceTheme::Fuchsia,
        SpaceTheme::Amber,
        SpaceTheme::Terracotta,
        SpaceTheme::Slate,
        SpaceTheme::Pine,
        SpaceTheme::Plum,
        SpaceTheme::Ink,
    ];

    /// The canonical machine-readable name used by snapshots and E2E
    /// assertions (the same convention as [`ProfileHue::name`]).
    pub fn name(self) -> &'static str {
        match self {
            SpaceTheme::Cobalt => "cobalt",
            SpaceTheme::Violet => "violet",
            SpaceTheme::Fuchsia => "fuchsia",
            SpaceTheme::Amber => "amber",
            SpaceTheme::Terracotta => "terracotta",
            SpaceTheme::Slate => "slate",
            SpaceTheme::Pine => "pine",
            SpaceTheme::Plum => "plum",
            SpaceTheme::Ink => "ink",
        }
    }

    /// A short human-readable display label for pickers and settings.
    pub fn label(self) -> &'static str {
        match self {
            SpaceTheme::Cobalt => "Cobalt",
            SpaceTheme::Violet => "Violet",
            SpaceTheme::Fuchsia => "Fuchsia",
            SpaceTheme::Amber => "Amber",
            SpaceTheme::Terracotta => "Terracotta",
            SpaceTheme::Slate => "Slate",
            SpaceTheme::Pine => "Pine",
            SpaceTheme::Plum => "Plum",
            SpaceTheme::Ink => "Ink",
        }
    }

    /// The theme corresponding to a DESIGN §12 profile identity hue.
    ///
    /// Bijective onto the six identity tokens: a profile hue always maps to
    /// a distinct theme whose name equals the hue name, so the existing
    /// identity hues carry into the space-theming mechanism unchanged.
    pub fn from_profile_hue(hue: ProfileHue) -> SpaceTheme {
        match hue {
            ProfileHue::Cobalt => SpaceTheme::Cobalt,
            ProfileHue::Violet => SpaceTheme::Violet,
            ProfileHue::Fuchsia => SpaceTheme::Fuchsia,
            ProfileHue::Amber => SpaceTheme::Amber,
            ProfileHue::Terracotta => SpaceTheme::Terracotta,
            ProfileHue::Slate => SpaceTheme::Slate,
        }
    }

    /// The profile identity hue this theme corresponds to, when it is one
    /// of the six identity tokens (`None` for the space-only tokens).
    pub fn profile_hue(self) -> Option<ProfileHue> {
        match self {
            SpaceTheme::Cobalt => Some(ProfileHue::Cobalt),
            SpaceTheme::Violet => Some(ProfileHue::Violet),
            SpaceTheme::Fuchsia => Some(ProfileHue::Fuchsia),
            SpaceTheme::Amber => Some(ProfileHue::Amber),
            SpaceTheme::Terracotta => Some(ProfileHue::Terracotta),
            SpaceTheme::Slate => Some(ProfileHue::Slate),
            SpaceTheme::Pine | SpaceTheme::Plum | SpaceTheme::Ink => None,
        }
    }
}

impl Default for SpaceTheme {
    /// Deterministic fallback: the first token in palette order.
    fn default() -> Self {
        SpaceTheme::Cobalt
    }
}
