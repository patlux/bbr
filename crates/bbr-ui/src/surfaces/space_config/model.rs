//! Space-configuration surface presentation model (DESIGN.md §11.3).
//!
//! The §11.3 configuration surface is presentation-only: `bbr-app` owns the
//! bound `SpaceId`, the rename/appearance authority, and every mutation;
//! this model is the plain data the surface renders and the gallery
//! fixtures. The identity and theme catalogs are closed by construction
//! ([`bbr_domain::SpaceTheme::ALL`], [`bbr_domain::ALLOWED_EMOJI`],
//! [`bbr_domain::BbrGlyph::ALL`]) — no free-form value can reach the
//! surface state.

use bbr_domain::{SpaceIdentity, SpaceTheme};

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

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_domain::{BbrGlyph, SpaceEmoji};

    #[test]
    fn fresh_model_prefills_the_title_and_curated_defaults() {
        let model = SpaceConfigModel::new("Reading");
        assert_eq!(model.title, "Reading");
        assert_eq!(model.name, "Reading", "rename draft equals the title");
        assert_eq!(model.theme, SpaceTheme::default());
        assert_eq!(model.identity, SpaceIdentity::default());
    }

    #[test]
    fn emoji_selection_tracks_the_catalog_index() {
        let emoji = SpaceEmoji::new("🌊").expect("curated emoji");
        let model = SpaceConfigModel {
            title: "S".into(),
            name: "S".into(),
            theme: SpaceTheme::Pine,
            identity: SpaceIdentity::Emoji(emoji),
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
