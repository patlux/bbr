//! Space-configuration surface §15 ids (DESIGN.md §11.3/§15) — the stable
//! `bbr-*` id constants and the deterministic per-option id derivations.
//! `bbr-app`'s visible-ids/snapshot derivation imports the exact same ids,
//! so the shell and the surface can never drift.

/// §11.3/§15 the configuration surface container.
pub const SPACE_CONFIG_ID: &str = "bbr-space-config";
/// The surface header title (`Configure Space · <name>`).
pub const SPACE_CONFIG_TITLE_ID: &str = "bbr-space-config-title";
/// The rename field (the name draft).
pub const SPACE_CONFIG_NAME_ID: &str = "bbr-space-config-name";
/// The theme-picker group label.
pub const SPACE_CONFIG_THEME_LABEL_ID: &str = "bbr-space-config-theme-label";
/// The identity-picker group label.
pub const SPACE_CONFIG_IDENTITY_LABEL_ID: &str = "bbr-space-config-identity-label";
/// The emoji identity group label.
pub const SPACE_CONFIG_EMOJI_LABEL_ID: &str = "bbr-space-config-emoji-label";
/// The glyph identity group label.
pub const SPACE_CONFIG_GLYPH_LABEL_ID: &str = "bbr-space-config-glyph-label";
/// The Done control (commits the rename + appearance).
pub const SPACE_CONFIG_DONE_ID: &str = "bbr-space-config-done";
/// The Cancel control (keeps the created Space and its valid defaults).
pub const SPACE_CONFIG_CANCEL_ID: &str = "bbr-space-config-cancel";
/// The action row holding the Done/Cancel controls.
pub const SPACE_CONFIG_ACTIONS_ID: &str = "bbr-space-config-actions";

/// The `bbr-space-config-theme-<name>` id of one curated theme swatch
/// (keyed by the canonical token name).
pub fn space_config_theme_id(theme: bbr_domain::SpaceTheme) -> String {
    format!("bbr-space-config-theme-{}", theme.name())
}

/// The `bbr-space-config-emoji-<index>` id of one curated emoji option
/// (index into the canonical [`bbr_domain::ALLOWED_EMOJI`] catalog).
pub fn space_config_emoji_id(index: usize) -> String {
    format!("bbr-space-config-emoji-{index}")
}

/// The `bbr-space-config-glyph-<name>` id of one BBR-glyph identity option
/// (keyed by the canonical glyph name).
pub fn space_config_glyph_id(glyph: bbr_domain::BbrGlyph) -> String {
    format!("bbr-space-config-glyph-{}", glyph.name())
}

/// The complete id set the surface renders while open: the container, the
/// title, the name field, the group labels, every curated theme swatch,
/// every curated emoji and glyph option, and the Done/Cancel controls.
/// Exposed only while the surface is open (mirroring the render).
pub fn space_config_ids() -> Vec<String> {
    let mut ids = vec![
        SPACE_CONFIG_ID.to_string(),
        SPACE_CONFIG_TITLE_ID.to_string(),
        SPACE_CONFIG_NAME_ID.to_string(),
        SPACE_CONFIG_THEME_LABEL_ID.to_string(),
        SPACE_CONFIG_IDENTITY_LABEL_ID.to_string(),
        SPACE_CONFIG_EMOJI_LABEL_ID.to_string(),
        SPACE_CONFIG_GLYPH_LABEL_ID.to_string(),
        SPACE_CONFIG_ACTIONS_ID.to_string(),
        SPACE_CONFIG_DONE_ID.to_string(),
        SPACE_CONFIG_CANCEL_ID.to_string(),
    ];
    for theme in bbr_domain::SpaceTheme::ALL {
        ids.push(space_config_theme_id(theme));
    }
    for (index, _emoji) in bbr_domain::ALLOWED_EMOJI.iter().enumerate() {
        ids.push(space_config_emoji_id(index));
    }
    for glyph in bbr_domain::BbrGlyph::ALL {
        ids.push(space_config_glyph_id(glyph));
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_keyed_by_canonical_names() {
        assert_eq!(
            space_config_theme_id(bbr_domain::SpaceTheme::Cobalt),
            "bbr-space-config-theme-cobalt"
        );
        assert_eq!(
            space_config_theme_id(bbr_domain::SpaceTheme::Pine),
            "bbr-space-config-theme-pine"
        );
        assert_eq!(space_config_emoji_id(0), "bbr-space-config-emoji-0");
        assert_eq!(space_config_emoji_id(17), "bbr-space-config-emoji-17");
        assert_eq!(
            space_config_glyph_id(bbr_domain::BbrGlyph::Code),
            "bbr-space-config-glyph-code"
        );
        assert_eq!(
            space_config_glyph_id(bbr_domain::BbrGlyph::Globe),
            "bbr-space-config-glyph-globe"
        );
        assert_eq!(SPACE_CONFIG_ID, "bbr-space-config");
        assert_eq!(SPACE_CONFIG_ACTIONS_ID, "bbr-space-config-actions");
    }

    #[test]
    fn id_set_covers_every_curated_option_once() {
        let ids = space_config_ids();
        assert!(ids.contains(&SPACE_CONFIG_ACTIONS_ID.to_string()));
        assert!(ids.contains(&SPACE_CONFIG_DONE_ID.to_string()));
        assert!(ids.contains(&SPACE_CONFIG_CANCEL_ID.to_string()));
        for theme in bbr_domain::SpaceTheme::ALL {
            assert!(ids.contains(&space_config_theme_id(theme)));
        }
        for index in 0..bbr_domain::ALLOWED_EMOJI.len() {
            assert!(ids.contains(&space_config_emoji_id(index)));
        }
        for glyph in bbr_domain::BbrGlyph::ALL {
            assert!(ids.contains(&space_config_glyph_id(glyph)));
        }
        // Ids are unique (a `HashSet`-size check over the whole set).
        let unique: std::collections::HashSet<&str> = ids.iter().map(String::as_str).collect();
        assert_eq!(unique.len(), ids.len(), "no duplicate surface ids");
    }
}
