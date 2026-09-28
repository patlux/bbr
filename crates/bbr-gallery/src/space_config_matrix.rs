//! The Space-configuration matrix authority (packet
//! `space-creation-appearance-ui.md`, DESIGN.md §11.3): **one declaration
//! source** for every valid presentation family the production
//! `bbr_ui::surfaces::space_config` surface renders. The gallery story
//! renders exactly these families through the production surface with
//! fixture models and recorder callbacks — never a gallery-local copy.
//!
//! Matrix rows are closed by construction: the header title and the rename
//! draft are plain strings, but the theme and identity come only from the
//! curated catalogs ([`SpaceTheme::ALL`], [`ALLOWED_EMOJI`],
//! [`BbrGlyph::ALL`]) — the same closed sets the domain enforces, so the
//! story can never present a value the production surface cannot render.

use bbr_domain::{BbrGlyph, SpaceEmoji, SpaceIdentity, SpacePaper, SpaceTheme, TintIntensity};
use bbr_ui::surfaces::space_config::SpaceConfigModel;

/// One matrix case: a family label plus the plain production
/// [`SpaceConfigModel`] the surface renders. Every theme/identity value is
/// a curated catalog member by construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceConfigCase {
    pub label: &'static str,
    /// Whether the surface owns chrome focus (the §9 ring).
    pub focused: bool,
    pub model: SpaceConfigModel,
}

/// The deterministic matrix: the fresh created-space defaults, a curated
/// theme + emoji identity, a curated theme + BBR-glyph identity, and a
/// renamed draft with the default identity — every family the production
/// surface can render.
pub fn space_config_cases() -> Vec<SpaceConfigCase> {
    let glyph_code = SpaceIdentity::Glyph(BbrGlyph::Code);
    let emoji_wave = SpaceIdentity::Emoji(SpaceEmoji::new("🌊").expect("curated emoji"));
    vec![
        SpaceConfigCase {
            label: "Fresh defaults",
            focused: true,
            model: SpaceConfigModel::new("New Space"),
        },
        SpaceConfigCase {
            label: "Emoji identity",
            focused: false,
            model: SpaceConfigModel {
                title: "Reading".into(),
                name: "Reading".into(),
                theme: SpaceTheme::Cobalt,
                identity: emoji_wave,
                paper: SpacePaper::default(),
            },
        },
        SpaceConfigCase {
            label: "BBR glyph identity",
            focused: true,
            model: SpaceConfigModel {
                title: "Work".into(),
                name: "Work notes".into(),
                theme: SpaceTheme::Pine,
                identity: glyph_code,
                paper: SpacePaper {
                    intensity: TintIntensity::Rich,
                    grain: true,
                },
            },
        },
        SpaceConfigCase {
            label: "Renamed draft",
            focused: false,
            model: SpaceConfigModel {
                title: "Private".into(),
                name: "Personal vault".into(),
                theme: SpaceTheme::Violet,
                identity: SpaceIdentity::default(),
                paper: SpacePaper {
                    intensity: TintIntensity::Subtle,
                    grain: false,
                },
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_case_theme_and_identity_is_curated() {
        for case in space_config_cases() {
            assert!(
                SpaceTheme::ALL.contains(&case.model.theme),
                "case {} theme is a curated token",
                case.label
            );
            match &case.model.identity {
                SpaceIdentity::Emoji(emoji) => assert!(
                    bbr_domain::ALLOWED_EMOJI.contains(&emoji.as_str()),
                    "case {} emoji is curated",
                    case.label
                ),
                SpaceIdentity::Glyph(glyph) => assert!(
                    BbrGlyph::ALL.contains(glyph),
                    "case {} glyph is curated",
                    case.label
                ),
            }
        }
    }

    #[test]
    fn the_matrix_has_unique_deterministic_labels() {
        let cases = space_config_cases();
        assert!(!cases.is_empty());
        let labels: HashSet<&str> = cases.iter().map(|c| c.label).collect();
        assert_eq!(labels.len(), cases.len(), "labels are unique");
    }
}
