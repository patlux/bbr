//! The app-owned mapping between the persisted space appearance columns
//! (`spaces.tint`, `glyph_kind`/`glyph`, and the schema-v4 paper
//! `tint_intensity`/`grain`) and the domain `SpaceAppearance` record.
//!
//! Since schema v4 (v2 S10) the whole curated appearance persists: theme,
//! identity, intensity and static grain. Persisting only the new paper
//! while the theme reverted to its derived default would restore a mix the
//! user never chose. Restore keeps a record only where it differs from the
//! derived default (the domain rule "defaults never create records").
//!
//! The persistence crate mirrors the closed identity sets without depending
//! on the domain, so the mirrors are matched variant by variant here: a new
//! token on either side fails to compile until both agree.

use bbr_browser::{BrowserController, SpaceId};
use bbr_domain::{
    BbrGlyph, SpaceAppearance, SpaceEmoji, SpaceIdentity, SpacePaper, SpaceTheme, TintIntensity,
};
use bbr_persistence::{GlyphName, SpaceGlyph, SpaceTint};

/// Declares a pair of exhaustive variant-for-variant conversions.
macro_rules! mirror {
    ($to:ident, $from:ident, $a:ident, $b:ident, [$($variant:ident),+ $(,)?]) => {
        fn $to(value: $a) -> $b {
            match value {
                $($a::$variant => $b::$variant,)+
            }
        }
        fn $from(value: $b) -> $a {
            match value {
                $($b::$variant => $a::$variant,)+
            }
        }
    };
}

mirror!(
    tint_of,
    theme_of,
    SpaceTheme,
    SpaceTint,
    [Cobalt, Violet, Fuchsia, Amber, Terracotta, Slate, Pine, Plum, Ink]
);
mirror!(
    glyph_name_of,
    glyph_of,
    BbrGlyph,
    GlyphName,
    [
        Code, Terminal, Folder, Person, Briefcase, Heart, Star, Leaf, Compass, Moon, Book, Camera,
        Palette, Globe
    ]
);
mirror!(
    persisted_intensity,
    intensity_of,
    TintIntensity,
    PersistedIntensity,
    [Subtle, Standard, Rich]
);

use bbr_persistence::TintIntensity as PersistedIntensity;

/// The persisted columns of a domain appearance.
pub(crate) fn to_persisted(
    appearance: &SpaceAppearance,
) -> (SpaceTint, SpaceGlyph, bbr_persistence::SpacePaper) {
    let glyph = match &appearance.identity {
        SpaceIdentity::Emoji(emoji) => SpaceGlyph::Emoji(emoji.as_str().to_owned()),
        SpaceIdentity::Glyph(glyph) => SpaceGlyph::Glyph(glyph_name_of(*glyph)),
    };
    (
        tint_of(appearance.theme),
        glyph,
        bbr_persistence::SpacePaper {
            intensity: persisted_intensity(appearance.paper.intensity),
            grain: appearance.paper.grain,
        },
    )
}

/// The domain appearance of a persisted space row. The persisted emoji
/// allowlist mirrors the domain's; a value the domain refuses falls back to
/// the default identity (logged) rather than failing the restore.
pub(crate) fn from_persisted(space: &bbr_persistence::Space) -> SpaceAppearance {
    let identity = match &space.glyph {
        SpaceGlyph::Emoji(emoji) => match SpaceEmoji::new(emoji) {
            Ok(emoji) => SpaceIdentity::Emoji(emoji),
            Err(error) => {
                log::error!("persisted space {} identity refused: {error}", space.id);
                SpaceIdentity::default()
            }
        },
        SpaceGlyph::Glyph(name) => SpaceIdentity::Glyph(glyph_of(*name)),
    };
    SpaceAppearance {
        space: SpaceId(space.id),
        theme: theme_of(space.tint),
        identity,
        paper: SpacePaper {
            intensity: intensity_of(space.paper.intensity),
            grain: space.paper.grain,
        },
    }
}

/// Apply every persisted space appearance to a freshly restored controller
/// (the restore keeps the persisted `SpaceId`s).
pub(crate) fn restore_appearances(
    controller: &mut BrowserController,
    profiles: &[bbr_persistence::Profile],
) {
    for space in profiles.iter().flat_map(|profile| &profile.spaces) {
        if let Err(error) = controller.restore_space_appearance(from_persisted(space)) {
            log::error!("could not restore space {} appearance: {error}", space.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curated_value_round_trips_through_the_persisted_columns() {
        let emoji = SpaceIdentity::Emoji(SpaceEmoji::new("🚀").expect("curated"));
        let identities = BbrGlyph::ALL
            .into_iter()
            .map(SpaceIdentity::Glyph)
            .chain(std::iter::once(emoji));
        for (index, identity) in identities.enumerate() {
            let appearance = SpaceAppearance {
                space: SpaceId(10),
                theme: SpaceTheme::ALL[index % SpaceTheme::ALL.len()],
                identity,
                paper: SpacePaper {
                    intensity: TintIntensity::ALL[index % 3],
                    grain: index % 2 == 0,
                },
            };
            let (tint, glyph, paper) = to_persisted(&appearance);
            let space = bbr_persistence::Space {
                id: 10,
                title: "Main".into(),
                tint,
                glyph,
                paper,
                tabs: Vec::new(),
                active_tab: None,
                recently_closed: Vec::new(),
                split: None,
                folders: Vec::new(),
            };
            assert_eq!(from_persisted(&space), appearance);
        }
    }
}
