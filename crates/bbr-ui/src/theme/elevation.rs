//! The one `elev:2` floating-surface shadow helper (DESIGN.md §2).
//!
//! Before the G1a cutover every floating chrome surface duplicated its own
//! `elev2_shadows`/`elev2_shadow` helper (`portal/command.rs`,
//! `portal/find.rs`, `portal/context_menu.rs`, `app/downloads_render.rs`);
//! this module is the single shadow authority and all of those call sites
//! consume [`elev2`] instead. Rows never shadow (list paint budget, §2);
//! the dragged-row shadow (DESIGN.md §10.9) stays a private detail of the
//! `tab_row` component — it is a distinct single shadow, not a duplicate
//! of this floating surface.

use gpui::{hsla, point, px, BoxShadow};

use crate::theme::{Theme, ThemeMode};

/// The §2 `elev:2.shadow` token pair for the shell's active mode: bounded
/// GPUI shadows, no blur beyond the documented radii, no inset.
///
/// ```text
/// Light: 0 1 0 rgba(0,0,0,0.04), 0 12 24 rgba(23,26,29,0.10)
/// Dark:  0 0 0 1px rgba(255,255,255,0.05), 0 16 32 rgba(0,0,0,0.45)
/// ```
pub fn elev2(theme: &Theme) -> Vec<BoxShadow> {
    match theme.mode {
        ThemeMode::Dark => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 1.0, 0.05),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.45),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(32.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
        ThemeMode::Light => vec![
            BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.04),
                offset: point(px(0.0), px(1.0)),
                blur_radius: px(0.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(0.0, 0.0, 0.09, 0.10),
                offset: point(px(0.0), px(12.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, Theme, ThemeMode};

    #[test]
    fn elev2_matches_the_documented_token_pair_per_mode() {
        let dark = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
        let dark_shadows = elev2(&dark);
        assert_eq!(dark_shadows.len(), 2, "the §2 pair has exactly two shadows");
        assert!(!dark_shadows[0].inset);
        assert_eq!(dark_shadows[0].spread_radius, px(1.0));
        assert_eq!(dark_shadows[0].offset, point(px(0.0), px(0.0)));
        assert_eq!(dark_shadows[1].blur_radius, px(32.0));
        assert_eq!(dark_shadows[1].offset, point(px(0.0), px(16.0)));

        let light = Theme::new(ThemeMode::Light, ContrastMode::Standard, Density::Normal);
        let light_shadows = elev2(&light);
        assert_eq!(light_shadows.len(), 2);
        assert_eq!(light_shadows[0].offset, point(px(0.0), px(1.0)));
        assert_eq!(light_shadows[0].spread_radius, px(0.0));
        assert_eq!(light_shadows[1].blur_radius, px(24.0));
        assert_eq!(light_shadows[1].offset, point(px(0.0), px(12.0)));
    }
}
