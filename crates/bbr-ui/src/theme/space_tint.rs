//! Per-space curated theme tokens → contrast-safe chrome tint
//! (DESIGN.md §11.3: "The active token tints the BBR-owned sidebar material
//! through a contrast-safe, desaturated chrome variant").
//!
//! This is the **single central presentation model** for the space
//! appearance: the portable domain carries only the stable [`SpaceTheme`]
//! token name; every concrete light/dark × standard/increased-contrast
//! color a consumer renders comes from [`space_tint`] — never from a raw
//! literal scattered in a consumer. The tint is applied only to BBR-owned
//! chrome (the sidebar material and the active space chip); text,
//! hairlines, accent rails, icons, and web content keep their own tokens
//! and are never sampled or tinted (§13).
//!
//! # Derivation (never copied)
//!
//! - The **base material** is the shell's own [`Theme::palette`]
//!   `surface_sidebar` token, so the tint always follows the active
//!   mode/contrast theme.
//! - The six identity tokens reuse the DESIGN.md §12 profile-hue tiles
//!   verbatim ([`ProfileHueRamp::tile`]) — the existing profile identity
//!   hues "go into" the theming mechanism without a second color
//!   vocabulary. Pine/Plum/Ink are the curated space-only tokens.
//! - The "chrome variant" is a second hue family ([`chrome_wash_hue`])
//!   blended into the base material at 12 % (IC 14 %, DESIGN §18.1 D3) —
//!   never the saturated identity hue itself — calibrated so the D3
//!   tinted-rail bars and the D1 selected-row and footer bars hold on every
//!   tinted surface, veils composited.
//! - The active chip (§18.2) is the selected-row surface (`tint.selected`
//!   composited over the tinted rail) carrying 8 % of the identity hue; the
//!   strip paints it with `elev:lift`.

use bbr_domain::SpaceTheme;
use gpui::{rgb, Rgba};

use crate::palette::ProfileHueRamp;
use crate::theme::{ContrastMode, Theme, ThemeMode};

/// The blend alpha of the chrome wash into the base sidebar material
/// (DESIGN §18.1 D3): 12 % so the space is felt on the whole window body,
/// 14 % under increased contrast (the §3.4 deepening spirit) — the gate
/// proves every text bar on the result.
const TINT_ALPHA_STANDARD: f64 = 0.12;
const TINT_ALPHA_INCREASED: f64 = 0.14;
/// The active chip (§18.2): the lifted selection surface carrying this
/// share of the identity hue, so each space's chip stays distinguishable
/// in both contrast modes.
const CHIP_HUE_ALPHA: f64 = 0.08;

/// The resolved per-space chrome presentation for one theme/mode/contrast
/// combination: the saturated identity hue, the desaturated rail wash
/// blended over the base sidebar material, and the active chip surface.
/// All three derive from the same central [`space_tint`] resolver — no
/// consumer ever authors a render color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpaceTint {
    /// The hue's own saturated color (the identity tile family).
    pub hue: Rgba,
    /// The chrome tint — the *desaturated wash* blended into the base
    /// sidebar material (DESIGN.md §11.3 "desaturated chrome variant").
    pub chrome: Rgba,
    /// The active chip surface (§18.2): the lifted selection surface over
    /// the tinted rail carrying 8 % of `hue`.
    pub chip_active: Rgba,
}

/// Resolve one curated [`SpaceTheme`] token to its chrome presentation for
/// the shell's active theme. The base material is the shell's own
/// `surface_sidebar` token (so mode/contrast follow the theme); the six
/// identity tokens map to the §12 profile tiles; the three space-only
/// tokens have curated hue pairs.
pub fn space_tint(token: SpaceTheme, theme: &Theme) -> SpaceTint {
    let base = theme.palette.surface_sidebar;
    let hue = identity_hue(token, theme.mode);
    let wash = chrome_wash_hue(token, theme.mode);
    let tint_alpha = match theme.contrast {
        ContrastMode::Standard => TINT_ALPHA_STANDARD,
        ContrastMode::Increased => TINT_ALPHA_INCREASED,
    };
    let chrome = blend(base, wash, tint_alpha);
    let selection = crate::palette::over(theme.palette.tint_selected, chrome);
    SpaceTint {
        hue,
        chrome,
        chip_active: blend(selection, hue, CHIP_HUE_ALPHA),
    }
}

/// The deterministic `#RRGGBB` wire form of a resolved chrome color (the
/// §15 snapshot exposes the resolved tint state as typed hex evidence; it
/// never exposes native/web pixels). Rounding is the same per-channel
/// rounding the render applies when blending, so the wire value equals the
/// rendered color.
///
/// Consumed by the app's testkit snapshot seam; `bbr-ui` itself uses it
/// only from its unit tests, so the lint is relaxed.
#[allow(dead_code)]
pub fn hex(rgba: Rgba) -> String {
    let channel = |value: f32| (value * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b)
    )
}

/// The saturated identity hue pair for a token (light/dark). The six
/// identity tokens reuse the §12 profile tiles verbatim
/// ([`ProfileHueRamp::tile`]) so profile identity "goes into" the theming
/// mechanism without changing the authored values; the space-only tokens
/// are curated deep-pine, deep-plum, and neutral-ink hues. This is the
/// *identity* color — the chip surface and the identity tile derive from
/// it.
fn identity_hue(token: SpaceTheme, mode: ThemeMode) -> Rgba {
    if let Some(hue) = token.profile_hue() {
        return ProfileHueRamp::tile(hue, mode).tile;
    }
    use SpaceTheme as T;
    let (r, g, b) = match (token, mode) {
        // Curated space-only identity hues (provisional S1 calibration,
        // Patrick accepted): deep pine green, deep plum, neutral ink.
        (T::Pine, ThemeMode::Light) => (0x0E, 0x6E, 0x52),
        (T::Pine, ThemeMode::Dark) => (0x2F, 0xBF, 0x8F),
        (T::Plum, ThemeMode::Light) => (0x6E, 0x3A, 0x8F),
        (T::Plum, ThemeMode::Dark) => (0x9A, 0x63, 0xC9),
        (T::Ink, ThemeMode::Light) => (0x6A, 0x6A, 0x6A),
        (T::Ink, ThemeMode::Dark) => (0x8A, 0x8A, 0x8A),
        // The six identity tokens never reach here (`profile_hue()` above
        // resolves them through the §12 tiles).
        _ => unreachable!("identity token resolves through the §12 tiles"),
    };
    rgb((r << 16) | (g << 8) | b)
}

/// The *chrome wash* hue pair for a token (light/dark): the color the rail
/// material is blended toward at 12 % (IC 14 %). This is deliberately
/// **not** the saturated identity hue — it is the variant the gates prove
/// against (DESIGN §18.1 D1/D3, calibrated with v2 S4):
///
/// - Dark mode: the identity hue at 1.3× the v1 wash saturation and the
///   highest value that keeps `ink ≥ 12:1` on the rail, the composited
///   selection at `ink ≥ 10.5:1` and the footer veil at
///   `ink.secondary ≥ 5.9:1` (IC: 14:1 / 9:1 at 14 %). Luminance sits near
///   the rail's own, so the tint reads as hue, not as a lighter rail.
/// - Light mode: the identity hue at value 1 and HSV saturation 0.45 (Slate
///   half that, Ink a neutral grey), lowered until `accent.ink ≥ 5.9:1` —
///   the binding light bar — holds on the rail; the tint may darken the
///   rail slightly while `ink.secondary` keeps ≥ 5.9:1 (IC ≥ 9:1).
fn chrome_wash_hue(token: SpaceTheme, mode: ThemeMode) -> Rgba {
    use SpaceTheme as T;
    let (r, g, b) = match (token, mode) {
        (T::Cobalt, ThemeMode::Dark) => (0x14, 0x36, 0x95),
        (T::Violet, ThemeMode::Dark) => (0x3E, 0x28, 0x9F),
        (T::Fuchsia, ThemeMode::Dark) => (0x87, 0x19, 0x59),
        (T::Amber, ThemeMode::Dark) => (0x4F, 0x36, 0x00),
        (T::Terracotta, ThemeMode::Dark) => (0x71, 0x2A, 0x06),
        (T::Slate, ThemeMode::Dark) => (0x30, 0x38, 0x43),
        (T::Pine, ThemeMode::Dark) => (0x02, 0x46, 0x2F),
        (T::Plum, ThemeMode::Dark) => (0x5C, 0x24, 0x79),
        (T::Ink, ThemeMode::Dark) => (0x37, 0x37, 0x37),
        (T::Cobalt, ThemeMode::Light) => (0xA3, 0xB6, 0xFF),
        (T::Violet, ThemeMode::Light) => (0xC5, 0xAD, 0xFF),
        (T::Fuchsia, ThemeMode::Light) => (0xFF, 0x9E, 0xD9),
        (T::Amber, ThemeMode::Light) => (0xFF, 0xD5, 0x8C),
        (T::Terracotta, ThemeMode::Light) => (0xFF, 0xB1, 0x8C),
        (T::Slate, ThemeMode::Light) => (0xC6, 0xDE, 0xFF),
        (T::Pine, ThemeMode::Light) => (0x8C, 0xFF, 0xDE),
        (T::Plum, ThemeMode::Light) => (0xDC, 0xA6, 0xFF),
        (T::Ink, ThemeMode::Light) => (0xB8, 0xB8, 0xB8),
    };
    rgb((r << 16) | (g << 8) | b)
}

/// Blend `hue` over the base material at `alpha` (per-channel linear
/// blend, the same math the palette uses for its material washes).
fn blend(base: Rgba, hue: Rgba, alpha: f64) -> Rgba {
    let mix = |b: f32, o: f32| b * (1.0 - alpha as f32) + o * alpha as f32;
    Rgba {
        r: mix(base.r, hue.r),
        g: mix(base.g, hue.g),
        b: mix(base.b, hue.b),
        a: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Density;

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
        let l = |r: &Rgba| {
            0.2126 * lum(r.r as f64) + 0.7152 * lum(r.g as f64) + 0.0722 * lum(r.b as f64)
        };
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
                for token in SpaceTheme::ALL {
                    let tint = space_tint(token, &theme);
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
                    let tint = space_tint(token, &theme);
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
                let tinted = lab(&space_tint(token, &theme).chrome);
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
    /// breaking the gate above. The deepened blend moves the chrome strictly
    /// further from the base material, so the measured distance (the WCAG
    /// ratio of the blended chrome against the base) strictly increases with
    /// contrast — `d_ic > d_std` — for every one of the nine tones in both
    /// modes, while the §3.1 text contracts still hold.
    #[test]
    fn increased_contrast_deepens_the_tint() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let standard = Theme::new(mode, ContrastMode::Standard, Density::Normal);
            let base = standard.palette.surface_sidebar;
            for token in SpaceTheme::ALL {
                let standard_tint = space_tint(token, &standard);
                let increased = space_tint(
                    token,
                    &Theme::new(mode, ContrastMode::Increased, Density::Normal),
                );
                let d_std = ratio(&base, &standard_tint.chrome);
                let d_ic = ratio(&base, &increased.chrome);
                assert!(
                    d_ic > d_std,
                    "{token:?} {mode:?} IC chrome moves strictly further from the base material (d_std {d_std:.6}, d_ic {d_ic:.6})"
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
                let tint = space_tint(token, &theme);
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
            let tint = space_tint(SpaceTheme::Ink, &theme);
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
                let tint = space_tint(token, &theme);
                assert!(
                    (tint.hue.r - tint.hue.g).abs() >= 0.02
                        || (tint.hue.g - tint.hue.b).abs() >= 0.02,
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
        let tint = space_tint(SpaceTheme::Cobalt, &theme);
        assert_eq!(hex(tint.hue), "#6E8BFF", "cobalt dark identity tile");
        assert_eq!(hex(tint.chrome), "#1B2130", "cobalt dark standard chrome");
        assert_eq!(
            hex(tint.chip_active),
            "#353C52",
            "cobalt dark standard chip"
        );
        // The light identity hue is the §12 light cobalt tile.
        let light = Theme::new(ThemeMode::Light, ContrastMode::Standard, Density::Normal);
        let light_tint = space_tint(SpaceTheme::Cobalt, &light);
        assert_eq!(hex(light_tint.hue), "#4062E8", "cobalt light identity tile");
        assert_eq!(
            hex(light_tint.chrome),
            "#DDDFE5",
            "cobalt light standard chrome"
        );
        assert_eq!(
            hex(light_tint.chip_active),
            "#F0F2FD",
            "cobalt light standard chip"
        );
    }
}
