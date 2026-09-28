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

use bbr_domain::{SpaceTheme, TintIntensity};
use gpui::{rgb, Rgba};

use crate::palette::ProfileHueRamp;
use crate::theme::{ContrastMode, Theme, ThemeMode};

/// The blend alpha of the chrome wash into the base sidebar material
/// (DESIGN §18.1 D3): 12 % so the space is felt on the whole window body,
/// 14 % under increased contrast (the §3.4 deepening spirit) — the gate
/// proves every text bar on the result.
const TINT_ALPHA_STANDARD: f64 = 0.12;
const TINT_ALPHA_INCREASED: f64 = 0.14;
/// The curated intensity steps (v2 D7). `subtle` halves the wash
/// (IC 8 %); `rich` doubles it (IC 26 %) and then pins the result to the
/// standard step's relative luminance, so the step adds hue, never
/// lightness: every D3 bar is a luminance ratio and stays where the
/// standard step proved it (the near-achromatic Slate/Ink washes therefore
/// barely move).
const TINT_ALPHA_SUBTLE: f64 = 0.06;
const TINT_ALPHA_SUBTLE_INCREASED: f64 = 0.08;
const TINT_ALPHA_RICH: f64 = 0.24;
const TINT_ALPHA_RICH_INCREASED: f64 = 0.26;
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
///
/// `intensity` is the space's curated paper step (v2 D7); every step holds
/// the D3 bars in every mode and contrast.
pub fn space_tint(token: SpaceTheme, intensity: TintIntensity, theme: &Theme) -> SpaceTint {
    let base = theme.palette.surface_sidebar;
    let hue = identity_hue(token, theme.mode);
    let wash = chrome_wash_hue(token, theme.mode);
    let increased = theme.contrast == ContrastMode::Increased;
    let standard_alpha = if increased {
        TINT_ALPHA_INCREASED
    } else {
        TINT_ALPHA_STANDARD
    };
    let standard = blend(base, wash, standard_alpha);
    let chrome = match intensity {
        TintIntensity::Standard => standard,
        TintIntensity::Subtle => blend(
            base,
            wash,
            if increased {
                TINT_ALPHA_SUBTLE_INCREASED
            } else {
                TINT_ALPHA_SUBTLE
            },
        ),
        TintIntensity::Rich => pin_luminance(
            blend(
                base,
                wash,
                if increased {
                    TINT_ALPHA_RICH_INCREASED
                } else {
                    TINT_ALPHA_RICH
                },
            ),
            relative_luminance(standard),
        ),
    };
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

/// WCAG relative luminance of a rendered colour.
fn relative_luminance(color: Rgba) -> f64 {
    let channel = |value: f32| {
        let value = f64::from(value);
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// Move `color` toward black or white until its relative luminance equals
/// `target` (deterministic bisection; the hue direction is kept).
fn pin_luminance(color: Rgba, target: f64) -> Rgba {
    let darker = relative_luminance(color) > target;
    let toward = if darker { rgb(0x000000) } else { rgb(0xFFFFFF) };
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    for _ in 0..32 {
        let mid = (low + high) / 2.0;
        let luminance = relative_luminance(blend(color, toward, mid));
        let short = if darker {
            luminance > target
        } else {
            luminance < target
        };
        if short {
            low = mid;
        } else {
            high = mid;
        }
    }
    blend(color, toward, (low + high) / 2.0)
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
mod tests;
