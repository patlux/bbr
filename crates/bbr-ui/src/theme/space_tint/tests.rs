//! Contrast gates and resolver contracts of the per-space chrome tint.

use super::*;
use crate::theme::Density;

mod intensity;

/// WCAG 2.x contrast of two rendered colors (computed from the actual
/// blended `Rgba` values, never copied ratios).
fn ratio(a: &Rgba, b: &Rgba) -> f64 {
    let lum = |c: f64| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let l =
        |r: &Rgba| 0.2126 * lum(r.r as f64) + 0.7152 * lum(r.g as f64) + 0.0722 * lum(r.b as f64);
    let (hi, lo) = if l(a) >= l(b) {
        (l(a), l(b))
    } else {
        (l(b), l(a))
    };
    (hi + 0.05) / (lo + 0.05)
}

/// §11.3/§3.1 contrast gate on the *tinted* chrome: for every curated
/// token, mode, and contrast level, the rail material with the active
/// space's tint still satisfies the text/hairline/accent contracts
/// (D3: ink ≥ 12 standard / ≥ 14 IC on tinted chrome, secondary ≥ 5.9
/// standard / ≥ 9.0 IC, accent ≥ 3.0, accent.ink ≥ 5.9 light / 9.0
/// dark), and the active chip surface keeps the identity glyph and the
/// rail legible.
#[test]
fn every_curated_tint_keeps_the_contrast_contract() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
            let theme = Theme::new(mode, contrast, Density::Normal);
            let p = theme.palette;
            let base = p.surface_sidebar;
            // v2 D7: every curated intensity step holds every bar.
            for (token, intensity) in SpaceTheme::ALL
                .into_iter()
                .flat_map(|token| TintIntensity::ALL.map(|step| (token, step)))
            {
                let tint = space_tint(token, intensity, &theme);
                let token = format!("{token:?}/{intensity:?}");
                // The tint is strictly a desaturated variant: it must
                // stay visibly close to the base material (the blend is
                // small) and never flip to the hue itself.
                let tint_vs_base = ratio(&tint.chrome, &base);
                assert!(
                    (1.0..1.3).contains(&tint_vs_base),
                    "{token:?} {mode:?} {contrast:?} chrome stays a quiet variant (ratio {tint_vs_base:.2})"
                );
                // Text contracts on the tinted rail (DESIGN §18.1 D3).
                let ink_min = match contrast {
                    ContrastMode::Standard => 12.0,
                    ContrastMode::Increased => 14.0,
                };
                assert!(
                    ratio(&p.ink, &tint.chrome) >= ink_min,
                    "{token:?} {mode:?} {contrast:?} ink ≥ {ink_min}:1 on tinted chrome"
                );
                let secondary_min = match contrast {
                    // DESIGN §3.1: IC hardens ink.secondary to ≥ 9:1.
                    ContrastMode::Standard => 5.9,
                    ContrastMode::Increased => 9.0,
                };
                assert!(
                    ratio(&p.ink_secondary, &tint.chrome) >= secondary_min,
                    "{token:?} {mode:?} {contrast:?} secondary ≥ {secondary_min}:1"
                );
                assert!(
                    ratio(&p.accent, &tint.chrome) >= 3.0,
                    "{token:?} {mode:?} {contrast:?} accent rail ≥ 3:1"
                );
                // DESIGN §3.1: accent.ink ≥ 5.9 light / ≥ 9.0 dark.
                let accent_ink_min = match mode {
                    ThemeMode::Light => 5.9,
                    ThemeMode::Dark => 9.0,
                };
                assert!(
                    ratio(&p.accent_ink, &tint.chrome) >= accent_ink_min,
                    "{token:?} {mode:?} {contrast:?} accent.ink ≥ {accent_ink_min}:1"
                );
                // The active chip surface (selection + 8 % hue) still keeps
                // the identity legible: the identity glyph renders at
                // ink.secondary on it, and the selected rail accent reads.
                assert!(
                    ratio(&p.ink_secondary, &tint.chip_active) >= 4.5,
                    "{token:?} {mode:?} {contrast:?} chip identity ≥ 4.5:1"
                );
                assert!(
                    ratio(&p.accent, &tint.chip_active) >= 3.0,
                    "{token:?} {mode:?} {contrast:?} chip rail ≥ 3:1"
                );
            }
        }
    }
}

/// §18.2: the active chip is the selection surface carrying 8 % of the
/// identity hue — lighter than the rail (it lifts), distinct per token,
/// and never the saturated hue itself.
#[test]
fn the_active_chip_is_the_selection_carrying_the_identity_hue() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
            let theme = Theme::new(mode, contrast, Density::Normal);
            let mut chips: Vec<Rgba> = Vec::new();
            for token in SpaceTheme::ALL {
                let tint = space_tint(token, TintIntensity::Standard, &theme);
                let selection = crate::palette::over(theme.palette.tint_selected, tint.chrome);
                assert_eq!(tint.chip_active, blend(selection, tint.hue, 0.08));
                let luminance = |c: &Rgba| ratio(c, &rgb(0x000000));
                assert!(
                    luminance(&tint.chip_active) > luminance(&tint.chrome),
                    "{token:?} {mode:?} {contrast:?} chip lifts above the rail"
                );
                assert!(
                    !chips.contains(&tint.chip_active),
                    "{token:?} chip is distinct"
                );
                chips.push(tint.chip_active);
            }
        }
    }
}

/// D3: the tint is visible — every hued token moves the rail at least
/// 2 CIE ΔE76 away from the untinted material in both modes (the neutral
/// Ink and the blue-grey Slate are the quiet end of the family).
#[test]
fn hued_tokens_visibly_tint_the_rail() {
    fn lab(c: &Rgba) -> [f64; 3] {
        let lin = |v: f32| {
            let v = v as f64;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
        let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
        let f = |t: f64| {
            if t > 0.008856 {
                t.cbrt()
            } else {
                7.787 * t + 16.0 / 116.0
            }
        };
        [
            116.0 * f(y) - 16.0,
            500.0 * (f(x) - f(y)),
            200.0 * (f(y) - f(z)),
        ]
    }
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
        let base = lab(&theme.palette.surface_sidebar);
        for token in SpaceTheme::ALL {
            if matches!(token, SpaceTheme::Ink | SpaceTheme::Slate) {
                continue;
            }
            let tinted = lab(&space_tint(token, TintIntensity::Standard, &theme).chrome);
            let delta = base
                .iter()
                .zip(tinted)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(delta >= 2.0, "{token:?} {mode:?} tint ΔE {delta:.2} ≥ 2");
        }
    }
}

/// IC deepens the tint (the §3.4 selected-tint deepening spirit) without
/// breaking the gate above. Since S12 every step keeps the rail's
/// luminance, so the deepening is chroma: the IC chrome moves strictly
/// further from the base material in CIE ΔE76 for every hued token (the
/// neutral Ink has no hue to deepen and stays at the rail), and never
/// changes the luminance the bars are proven on.
#[test]
fn increased_contrast_deepens_the_tint() {
    fn delta(a: &Rgba, b: &Rgba) -> f64 {
        let lab = |c: &Rgba| {
            let lin = |v: f32| {
                let v = f64::from(v);
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
            let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
            let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
            let f = |t: f64| {
                if t > 0.008856 {
                    t.cbrt()
                } else {
                    7.787 * t + 16.0 / 116.0
                }
            };
            [
                116.0 * f(y) - 16.0,
                500.0 * (f(x) - f(y)),
                200.0 * (f(y) - f(z)),
            ]
        };
        let (a, b) = (lab(a), lab(b));
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let standard = Theme::new(mode, ContrastMode::Standard, Density::Normal);
        let increased_theme = Theme::new(mode, ContrastMode::Increased, Density::Normal);
        for token in SpaceTheme::ALL {
            if token == SpaceTheme::Ink {
                continue;
            }
            let standard_tint = space_tint(token, TintIntensity::Standard, &standard);
            let increased = space_tint(token, TintIntensity::Standard, &increased_theme);
            let d_std = delta(&standard.palette.surface_sidebar, &standard_tint.chrome);
            let d_ic = delta(&increased_theme.palette.surface_sidebar, &increased.chrome);
            assert!(
                d_ic > d_std,
                "{token:?} {mode:?} IC chrome moves strictly further from the base material (ΔE std {d_std:.3}, IC {d_ic:.3})"
            );
        }
    }
}

/// The six identity tokens consume the §12 profile tiles verbatim
/// (identity hues carry into the theming mechanism unchanged): the
/// resolved hue equals the existing `ProfileHueRamp` tile for the same
/// hue/mode, so there is exactly one authored identity hue per token.
#[test]
fn identity_tokens_reuse_the_profile_hue_tiles() {
    use bbr_domain::ProfileHue;
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
        for hue in [
            ProfileHue::Cobalt,
            ProfileHue::Violet,
            ProfileHue::Fuchsia,
            ProfileHue::Amber,
            ProfileHue::Terracotta,
            ProfileHue::Slate,
        ] {
            let token = SpaceTheme::from_profile_hue(hue);
            let tint = space_tint(token, TintIntensity::Standard, &theme);
            assert_eq!(
                tint.hue,
                ProfileHueRamp::tile(hue, mode).tile,
                "{token:?} {mode:?} hue equals the §12 tile"
            );
        }
    }
}

/// The three space-only tokens are distinct from the identity ramp and
/// from each other in both modes, so a space-only theme can never be
/// confused with a profile-hue theme by its resolved color.
#[test]
fn space_only_tokens_stay_distinct_from_the_identity_ramp() {
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let identity_hues: Vec<Rgba> = SpaceTheme::ALL
            .iter()
            .filter_map(|token| token.profile_hue())
            .map(|hue| ProfileHueRamp::tile(hue, mode).tile)
            .collect();
        for token in [SpaceTheme::Pine, SpaceTheme::Plum, SpaceTheme::Ink] {
            let hue = identity_hue(token, mode);
            assert!(
                !identity_hues.contains(&hue),
                "{token:?} {mode:?} hue stays outside the identity ramp"
            );
        }
        assert_ne!(
            identity_hue(SpaceTheme::Pine, mode),
            identity_hue(SpaceTheme::Plum, mode)
        );
        assert_ne!(
            identity_hue(SpaceTheme::Plum, mode),
            identity_hue(SpaceTheme::Ink, mode)
        );
    }
}

#[test]
fn ink_token_is_the_only_neutral_theme() {
    // The Ink token is the neutral family: its identity hue is
    // achromatic in both modes, so its chrome variant is a pure
    // luminance step.
    for mode in [ThemeMode::Dark, ThemeMode::Light] {
        let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
        let tint = space_tint(SpaceTheme::Ink, TintIntensity::Standard, &theme);
        assert!((tint.hue.r - tint.hue.g).abs() < 0.02, "ink hue is neutral");
        assert!((tint.hue.g - tint.hue.b).abs() < 0.02, "ink hue is neutral");
    }
    // The other eight tokens keep a hue: no token besides Ink is
    // achromatic, so identity is never confused with the neutral family.
    for token in SpaceTheme::ALL {
        if token == SpaceTheme::Ink {
            continue;
        }
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let theme = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            let tint = space_tint(token, TintIntensity::Standard, &theme);
            assert!(
                (tint.hue.r - tint.hue.g).abs() >= 0.02 || (tint.hue.g - tint.hue.b).abs() >= 0.02,
                "{token:?} {mode:?} hue carries color"
            );
        }
    }
}

/// The wire hex form is deterministic and equals the rendered color:
/// the snapshot's typed tint evidence is the actual render value.
#[test]
fn hex_form_is_deterministic_and_matches_the_resolved_color() {
    let theme = Theme::dark();
    let tint = space_tint(SpaceTheme::Cobalt, TintIntensity::Standard, &theme);
    assert_eq!(hex(tint.hue), "#6E8BFF", "cobalt dark identity tile");
    assert_eq!(hex(tint.chrome), "#151D32", "cobalt dark standard chrome");
    assert_eq!(
        hex(tint.chip_active),
        "#303954",
        "cobalt dark standard chip"
    );
    // The light identity hue is the §12 light cobalt tile.
    let light = Theme::new(ThemeMode::Light, ContrastMode::Standard, Density::Normal);
    let light_tint = space_tint(SpaceTheme::Cobalt, TintIntensity::Standard, &light);
    assert_eq!(hex(light_tint.hue), "#4062E8", "cobalt light identity tile");
    // S12: the Paper cobalt rail (#DFE4F3 reference).
    assert_eq!(
        hex(light_tint.chrome),
        "#E0E5F4",
        "cobalt light standard chrome"
    );
    assert_eq!(
        hex(light_tint.chip_active),
        "#F0F2FD",
        "cobalt light standard chip"
    );
}
