//! The shadow authorities (DESIGN.md §2, §18.2): `elev:2` for floating
//! surfaces and `elev:lift` for the one selected row / active space chip.
//!
//! Before the G1a cutover every floating chrome surface duplicated its own
//! `elev2_shadows`/`elev2_shadow` helper (`portal/command.rs`,
//! `portal/find.rs`, `portal/context_menu.rs`, `app/downloads_render.rs`);
//! this module is the single shadow authority and all of those call sites
//! consume [`elev2`] instead. Rows never shadow (list paint budget, §2)
//! with exactly one exception (D2): the static [`lift`] on the one selected
//! row and the active space chip. The dragged-row shadow (DESIGN.md §10.9)
//! stays a private detail of the `tab_row` component — it is a distinct
//! single shadow, not a duplicate of these.

use gpui::{point, px, BoxShadow, Hsla, Rgba};

use crate::theme::{Theme, ThemeMode};

/// `color:ink` (light) at `alpha` — the shadow colour of the light shell.
fn ink(alpha: f32) -> Hsla {
    Rgba {
        r: 0x11 as f32 / 255.0,
        g: 0x13 as f32 / 255.0,
        b: 0x15 as f32 / 255.0,
        a: alpha,
    }
    .into()
}

fn black(alpha: f32) -> Hsla {
    gpui::hsla(0.0, 0.0, 0.0, alpha)
}

fn white(alpha: f32) -> Hsla {
    gpui::hsla(0.0, 0.0, 1.0, alpha)
}

/// One outer shadow: `x y blur spread color`.
fn shadow(y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.0), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

/// The §18.2 `elev:2` v2 floating-surface shadows for the shell's active
/// mode: bounded GPUI shadows, no inset. The 1 px ring replaces the old
/// inset hairline edge of floating cards.
///
/// ```text
/// Light: 0 0 0 1px ink 7 %, 0 2 4 ink 5 %, 0 16 36 −6 ink 18 %
/// Dark:  0 0 0 1px white 8 %, 0 16 32 black 45 %
/// ```
pub fn elev2(theme: &Theme) -> Vec<BoxShadow> {
    match theme.mode {
        ThemeMode::Dark => vec![
            shadow(0.0, 0.0, 1.0, white(0.08)),
            shadow(16.0, 32.0, 0.0, black(0.45)),
        ],
        ThemeMode::Light => vec![
            shadow(0.0, 0.0, 1.0, ink(0.07)),
            shadow(2.0, 4.0, 0.0, ink(0.05)),
            shadow(16.0, 36.0, -6.0, ink(0.18)),
        ],
    }
}

/// The §18.2 `elev:lift` (D2): the one static shadow a row may carry —
/// painted with the selected row or active space chip surface, never
/// animated and never on hover. Increased contrast adds nothing beyond its
/// stronger surface values.
///
/// ```text
/// Light: 0 1 3 ink 10 %, 0 0 0 1px ink 4 %
/// Dark:  0 1 2 black 35 %, 0 0 0 1px white 4 %
/// ```
pub fn lift(theme: &Theme) -> Vec<BoxShadow> {
    match theme.mode {
        ThemeMode::Dark => vec![
            shadow(1.0, 2.0, 0.0, black(0.35)),
            shadow(0.0, 0.0, 1.0, white(0.04)),
        ],
        ThemeMode::Light => vec![
            shadow(1.0, 3.0, 0.0, ink(0.10)),
            shadow(0.0, 0.0, 1.0, ink(0.04)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, Theme, ThemeMode};

    fn theme(mode: ThemeMode, contrast: ContrastMode) -> Theme {
        Theme::new(mode, contrast, Density::Normal)
    }

    /// `(y, blur, spread, alpha)` of each shadow, for table comparisons.
    fn table(shadows: &[BoxShadow]) -> Vec<(f32, f32, f32, f32)> {
        shadows
            .iter()
            .map(|s| {
                assert!(!s.inset, "no inset shadows");
                assert_eq!(s.offset.x, px(0.0));
                (
                    f32::from(s.offset.y),
                    f32::from(s.blur_radius),
                    f32::from(s.spread_radius),
                    (s.color.a * 100.0).round() / 100.0,
                )
            })
            .collect()
    }

    #[test]
    fn elev2_matches_the_v2_token_list_per_mode() {
        for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
            assert_eq!(
                table(&elev2(&theme(ThemeMode::Dark, contrast))),
                vec![(0.0, 0.0, 1.0, 0.08), (16.0, 32.0, 0.0, 0.45)]
            );
            assert_eq!(
                table(&elev2(&theme(ThemeMode::Light, contrast))),
                vec![
                    (0.0, 0.0, 1.0, 0.07),
                    (2.0, 4.0, 0.0, 0.05),
                    (16.0, 36.0, -6.0, 0.18)
                ]
            );
        }
    }

    #[test]
    fn lift_is_a_short_static_shadow_plus_a_one_point_ring() {
        for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
            assert_eq!(
                table(&lift(&theme(ThemeMode::Light, contrast))),
                vec![(1.0, 3.0, 0.0, 0.10), (0.0, 0.0, 1.0, 0.04)]
            );
            assert_eq!(
                table(&lift(&theme(ThemeMode::Dark, contrast))),
                vec![(1.0, 2.0, 0.0, 0.35), (0.0, 0.0, 1.0, 0.04)]
            );
        }
        // The lift stays far quieter than a floating surface: its blur is
        // bounded to a few points (list paint budget, §2).
        let light = theme(ThemeMode::Light, ContrastMode::Standard);
        let max_blur = |shadows: Vec<BoxShadow>| {
            shadows
                .iter()
                .map(|s| f32::from(s.blur_radius))
                .fold(0.0, f32::max)
        };
        assert!(max_blur(lift(&light)) <= 3.0);
        assert!(max_blur(elev2(&light)) > max_blur(lift(&light)));
    }
}
