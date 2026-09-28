//! v2 D7 intensity steps (S12 chroma model): ordered, luminance-pinned,
//! and deterministic.

use super::super::*;
use crate::theme::Density;

fn lab(c: &Rgba) -> [f64; 3] {
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
}

fn delta(a: &Rgba, b: &Rgba) -> f64 {
    let (a, b) = (lab(a), lab(b));
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn themes() -> Vec<Theme> {
    [ThemeMode::Dark, ThemeMode::Light]
        .into_iter()
        .flat_map(|mode| {
            [ContrastMode::Standard, ContrastMode::Increased]
                .map(|contrast| Theme::new(mode, contrast, Density::Normal))
        })
        .collect()
}

/// Subtle sits between the rail and the standard wash; rich carries more
/// hue than standard. The near-achromatic Slate/Ink washes are exempt
/// from the rich ordering (pinning the luminance leaves them almost no hue
/// to add).
#[test]
fn steps_order_from_subtle_to_rich() {
    for theme in themes() {
        let base = theme.palette.surface_sidebar;
        for token in SpaceTheme::ALL {
            let subtle = space_tint(token, TintIntensity::Subtle, &theme).chrome;
            let standard = space_tint(token, TintIntensity::Standard, &theme).chrome;
            let rich = space_tint(token, TintIntensity::Rich, &theme).chrome;
            let (d_subtle, d_standard) = (delta(&base, &subtle), delta(&base, &standard));
            assert!(
                d_subtle < d_standard,
                "{token:?} {:?} subtle ΔE {d_subtle:.2} < standard {d_standard:.2}",
                theme.mode
            );
            if matches!(token, SpaceTheme::Slate | SpaceTheme::Ink) {
                continue;
            }
            let step = delta(&standard, &rich);
            assert!(
                step >= 2.0,
                "{token:?} {:?} {:?} rich moves ΔE {step:.2} ≥ 2 beyond standard",
                theme.mode,
                theme.contrast
            );
        }
    }
}

/// S12: every step adds hue, never lightness — its relative luminance is
/// the untinted rail's, which is what keeps every D3 luminance bar.
#[test]
fn every_step_keeps_the_rail_luminance() {
    for theme in themes() {
        let rail = relative_luminance(theme.palette.surface_sidebar);
        for token in SpaceTheme::ALL {
            for step in TintIntensity::ALL {
                let chrome = space_tint(token, step, &theme).chrome;
                let luminance = relative_luminance(chrome);
                assert!(
                    (luminance - rail).abs() < 1e-4,
                    "{token:?} {step:?} {:?} {:?} luminance {luminance:.5} = rail {rail:.5}",
                    theme.mode,
                    theme.contrast
                );
            }
        }
    }
}

/// The resolver is a pure function of (token, step, theme), and the
/// identity hue does not depend on the step.
#[test]
fn steps_are_deterministic_and_keep_the_identity_hue() {
    let theme = Theme::dark();
    for token in SpaceTheme::ALL {
        for step in TintIntensity::ALL {
            let tint = space_tint(token, step, &theme);
            assert_eq!(tint, space_tint(token, step, &theme));
            assert_eq!(
                tint.hue,
                space_tint(token, TintIntensity::Standard, &theme).hue
            );
        }
    }
}
