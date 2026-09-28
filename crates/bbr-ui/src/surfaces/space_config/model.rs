//! Space-configuration surface presentation model (DESIGN.md §11.3).
//!
//! The §11.3 configuration surface is presentation-only: `bbr-app` owns the
//! bound `SpaceId`, the rename/appearance authority, and every mutation;
//! this model is the plain data the surface renders and the gallery
//! fixtures. The identity and theme catalogs are closed by construction
//! ([`bbr_domain::SpaceTheme::ALL`], [`bbr_domain::ALLOWED_EMOJI`],
//! [`bbr_domain::BbrGlyph::ALL`]) — no free-form value can reach the
//! surface state.

use bbr_domain::{SpaceIdentity, SpacePaper, SpaceTheme};

/// The plain presentation model of the §11.3 configuration surface: the
/// header title, the name draft, and the selected curated theme/identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceConfigModel {
    /// The header title: the bound space's current title at open time.
    pub title: String,
    /// The rename draft (what the `bbr-space-config-name` field shows).
    pub name: String,
    /// The selected curated theme token.
    pub theme: SpaceTheme,
    /// The selected validated identity (exactly one emoji or glyph).
    pub identity: SpaceIdentity,
    /// v2 D6/D7: the selected intensity step and grain flag.
    pub paper: SpacePaper,
}

impl SpaceConfigModel {
    /// A fresh model for `title` with the canonical defaults (the rename
    /// draft equals the current title; theme/identity are the curated
    /// defaults — the same deterministic values a new Space receives).
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            name: title.clone(),
            title,
            theme: SpaceTheme::default(),
            identity: SpaceIdentity::default(),
            paper: SpacePaper::default(),
        }
    }

    /// Whether the identity picker's curated emoji option at `index` is the
    /// selected identity (deterministic catalog index).
    pub fn emoji_selected(&self, index: usize) -> bool {
        match &self.identity {
            SpaceIdentity::Emoji(emoji) => bbr_domain::ALLOWED_EMOJI
                .get(index)
                .is_some_and(|candidate| *candidate == emoji.as_str()),
            SpaceIdentity::Glyph(_) => false,
        }
    }

    /// Whether the identity picker's glyph option `glyph` is the selected
    /// identity.
    pub fn glyph_selected(&self, glyph: bbr_domain::BbrGlyph) -> bool {
        self.identity == SpaceIdentity::Glyph(glyph)
    }
}

/// The (fill, text) pair of the primary `bbr-space-config-done` action:
/// `on_accent` on the accent fill (§3.2). `accent_ink` is text on a neutral
/// surface and fails §3.1 on an accent fill.
pub(super) fn done_colors(p: &crate::palette::Palette) -> (gpui::Rgba, gpui::Rgba) {
    (p.accent, p.on_accent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{contrast, Palette};
    use bbr_domain::{BbrGlyph, SpaceEmoji};

    #[test]
    fn done_label_meets_the_text_contrast_bar_in_every_palette() {
        // Regression: Done painted accent.ink on accent (≈ 1.5 : 1).
        for palette in [
            Palette::light_standard(),
            Palette::light_increased(),
            Palette::dark_standard(),
            Palette::dark_increased(),
        ] {
            let (fill, text) = done_colors(&palette);
            let ratio = contrast(text, fill);
            assert!(ratio >= 4.5, "Done label is {ratio:.2}:1 on its fill");
        }
    }

    #[test]
    fn fresh_model_prefills_the_title_and_curated_defaults() {
        let model = SpaceConfigModel::new("Reading");
        assert_eq!(model.title, "Reading");
        assert_eq!(model.name, "Reading", "rename draft equals the title");
        assert_eq!(model.theme, SpaceTheme::default());
        assert_eq!(model.identity, SpaceIdentity::default());
        assert_eq!(model.paper, SpacePaper::default());
    }

    #[test]
    fn emoji_selection_tracks_the_catalog_index() {
        let emoji = SpaceEmoji::new("🌊").expect("curated emoji");
        let model = SpaceConfigModel {
            title: "S".into(),
            name: "S".into(),
            theme: SpaceTheme::Pine,
            identity: SpaceIdentity::Emoji(emoji),
            paper: SpacePaper::default(),
        };
        let index = bbr_domain::ALLOWED_EMOJI
            .iter()
            .position(|candidate| *candidate == "🌊")
            .expect("emoji in catalog");
        assert!(model.emoji_selected(index));
        assert!(!model.emoji_selected(index.wrapping_sub(1)));
        assert!(!model.glyph_selected(BbrGlyph::Code));
    }

    #[test]
    fn glyph_selection_tracks_the_glyph_identity() {
        let model = SpaceConfigModel {
            title: "S".into(),
            name: "S".into(),
            theme: SpaceTheme::Ink,
            identity: SpaceIdentity::Glyph(BbrGlyph::Terminal),
            paper: SpacePaper::default(),
        };
        assert!(model.glyph_selected(BbrGlyph::Terminal));
        assert!(!model.glyph_selected(BbrGlyph::Code));
        // An emoji identity never reads as a glyph selection.
        let emoji = SpaceConfigModel {
            identity: SpaceIdentity::Emoji(SpaceEmoji::new("🔥").expect("curated")),
            ..model.clone()
        };
        assert!(!emoji.glyph_selected(BbrGlyph::Terminal));
        // Exactly one identity kind is representable at a time.
        match &model.identity {
            SpaceIdentity::Glyph(_) => {}
            SpaceIdentity::Emoji(_) => panic!("glyph model holds an emoji"),
        }
    }
}
