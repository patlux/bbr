//! BBR Chassis — the typed color-token system (DESIGN.md §2–§5).
//!
//! Single source of truth for the shell's colors: the neutral scale, the
//! Phosphor accent, the status/indicator hues, the materials, the washes,
//! the §7.2 favicon slate ramp, and the §12 profile-hue tiles. Typography,
//! spacing, radius, density, motion, and elevation live in
//! [`crate::theme`]; tests and E2E assert the §15 mode fields and the §3.1
//! contrast contract, which is **derived from these tokens** — never copied
//! as a literal ratio.
//!
//! Token names intentionally mirror `DESIGN.md`:
//!
//! ```text
//! color:ink            → Palette::ink            color:accent       → accent
//! color:ink.secondary  → Palette::ink_secondary  color:accent.ink  → accent_ink
//! color:ink.faint      → Palette::ink_faint      color:tint.selected → tint_selected
//! color:hairline       → Palette::hairline       material:sidebar   → surface_sidebar
//! color:border.strong  → Palette::border_strong  material:canvas    → surface_canvas
//! material:field       → surface_field           material:glass     → surface_glass
//! status.error/attention/ok/info + .surface       → status_* / status_*_surface
//! favicon slate ramp (monogram)                   → FaviconSlate
//! profile hue tiles (§12)                         → ProfileHueRamp
//! ```

use bbr_domain::ProfileHue;
use gpui::{rgb, Rgba};

use crate::theme::ThemeMode;

/// A hex string `RRGGBB` palette token (kept as a string until the GPUI
/// color is requested so the WCAG gate can compute on the raw channels).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Hex(&'static str);

impl Hex {
    /// The color used by GPUI rendering.
    fn rgba(self) -> Rgba {
        rgb(u32::from_str_radix(self.0, 16).expect("token hex literal"))
    }

    /// Linearized sRGB channel for WCAG relative luminance (§3.1 gate).
    #[allow(dead_code)] // §3.1 gate primitives; consumed by the test suite
    fn channel(self, idx: usize) -> f64 {
        let value =
            u8::from_str_radix(&self.0[idx * 2..idx * 2 + 2], 16).expect("hex byte") as f64 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG 2.x relative luminance.
    #[allow(dead_code)] // §3.1 gate primitives; consumed by the test suite
    fn luminance(self) -> f64 {
        0.2126 * self.channel(0) + 0.7152 * self.channel(1) + 0.0722 * self.channel(2)
    }

    /// WCAG 2.x contrast ratio against another token.
    #[allow(dead_code)] // §3.1 gate primitives; consumed by the test suite
    fn contrast(self, other: Hex) -> f64 {
        let (hi, lo) = if self.luminance() >= other.luminance() {
            (self.luminance(), other.luminance())
        } else {
            (other.luminance(), self.luminance())
        };
        (hi + 0.05) / (lo + 0.05)
    }
}

macro_rules! hex {
    ($($name:ident = $value:literal;)*) => {
        $(const $name: Hex = Hex($value);)*
    };
}

// §3.1 neutral scale and text (light / dark / increased-contrast columns).
hex! {
    INK_LIGHT = "111315";   INK_DARK = "ECEEF1";
    SECONDARY_LIGHT = "4A4F55"; SECONDARY_DARK = "A6ACB5";
    FAINT_LIGHT = "949AA2"; FAINT_DARK = "6E747D";
    HAIRLINE_LIGHT = "C7C7C7"; HAIRLINE_DARK = "2A2E34";
    BORDER_STRONG_LIGHT = "C7C9C5"; BORDER_STRONG_DARK = "3B4149";
    INK_IC_LIGHT = "000000"; INK_IC_DARK = "FFFFFF";
    SECONDARY_IC_LIGHT = "292E34"; SECONDARY_IC_DARK = "D6DAE0";
    FAINT_IC_LIGHT = "5C626A"; FAINT_IC_DARK = "8A919B";
    HAIRLINE_IC_LIGHT = "9FA29E"; HAIRLINE_IC_DARK = "454C55";
    BORDER_STRONG_IC_LIGHT = "9FA29E"; BORDER_STRONG_IC_DARK = "525A63";
    // §3.2 accent — Phosphor.
    ACCENT_LIGHT = "0A7967"; ACCENT_DARK = "2AD4B6";
    ACCENT_INK_LIGHT = "0A5C4E"; ACCENT_INK_DARK = "3DF0CE";
    ACCENT_IC_LIGHT = "0B6B5C"; ACCENT_IC_DARK = "3DF0CE";
    ACCENT_INK_IC_LIGHT = "0A5A4C"; ACCENT_INK_IC_DARK = "3DF0CE";
    // §3.3 status hues + tint (§3.5).
    ERROR_LIGHT = "B5261E"; ERROR_DARK = "FF6A63";
    ATTENTION_LIGHT = "805100"; ATTENTION_DARK = "FFC63F";
    OK_LIGHT = "177A55"; OK_DARK = "45C08A";
    INFO_LIGHT = "2C5CC5"; INFO_DARK = "8AA6FF";
    TINT_SELECTED_LIGHT = "EEF2F9"; TINT_SELECTED_DARK = "0D2020";
    TINT_SELECTED_IC_LIGHT = "E1E8F3"; TINT_SELECTED_IC_DARK = "091818";
    // §2 materials.
    SURFACE_CANVAS_LIGHT = "F6F6F4"; SURFACE_CANVAS_DARK = "121416";
    SURFACE_SIDEBAR_LIGHT = "E0E0E0"; SURFACE_SIDEBAR_DARK = "1A1D21";
    SURFACE_FIELD_LIGHT = "FFFFFF"; SURFACE_FIELD_DARK = "23272C";
    SURFACE_OVERLAY_LIGHT = "FFFFFF"; SURFACE_OVERLAY_DARK = "202429";
    SURFACE_GLASS_LIGHT = "FFFFFF"; SURFACE_GLASS_DARK = "151719";
    SURFACE_FAVORITE_LIGHT = "D2D2D2"; SURFACE_FAVORITE_DARK = "23272C";
    SURFACE_FOOTER_LIGHT = "F3F8FE"; SURFACE_FOOTER_DARK = "202429";
}

/// The mode-selectable color palette for one theme/contrast combination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    // Neutral text scale (§3.1).
    pub ink: Rgba,
    pub ink_secondary: Rgba,
    pub ink_faint: Rgba,
    pub hairline: Rgba,
    pub border_strong: Rgba,
    /// Phosphor accent (§3.2): rails, rings, drag ticks.
    pub accent: Rgba,
    /// Accent-colored text (match highlight, mode chip, links).
    pub accent_ink: Rgba,
    // Status hues (§3.3): only on small glyph/dot surfaces with a
    // non-color companion.
    pub status_error: Rgba,
    pub status_attention: Rgba,
    pub status_ok: Rgba,
    pub status_info: Rgba,
    /// Status color at 10% over the owning material (12% in IC).
    pub status_error_surface: Rgba,
    pub status_attention_surface: Rgba,
    /// Accent at 10% over the material (20% in IC): the selection confirmation.
    pub tint_selected: Rgba,
    // Materials (§2): canvas behind glass, the sidebar rail, raised fields,
    // floating overlays, and what sits behind native web views.
    pub surface_canvas: Rgba,
    pub surface_sidebar: Rgba,
    pub surface_field: Rgba,
    pub surface_overlay: Rgba,
    pub surface_glass: Rgba,
    /// Reference-calibrated Favorite destination tile surface.
    pub surface_favorite: Rgba,
    /// Reference-calibrated footer transport/Space strip surface.
    pub surface_footer: Rgba,
    /// Row hover wash (§10.2) blended over the sidebar material; IC deepens.
    pub wash_hover: Rgba,
    /// Pointer-down / pressed wash (§10.3).
    pub wash_active: Rgba,
    /// §11.5 address-control fill: white-translucent on dark chrome,
    /// black-translucent on light chrome — an alpha wash over the sidebar
    /// material, so the token pair follows the theme and the space tint
    /// through the material beneath (never a hardcoded color, never a
    /// bordered field style).
    pub address_fill: Rgba,
    /// §11.5 address-control hover: one step brighter on dark chrome, one
    /// step darker on light chrome (§10.2 wash mechanics at `dur:fine`).
    pub address_fill_hover: Rgba,
    /// The two status surface washes at the IC level.
    pub surface_error_ic: Rgba,
    pub surface_attention_ic: Rgba,
}

fn blend(base: Hex, over: Hex, alpha: f64) -> Rgba {
    let mix = |b: usize, o: usize| {
        let base_ch = u8::from_str_radix(&base.0[b * 2..b * 2 + 2], 16).unwrap() as f32;
        let over_ch = u8::from_str_radix(&over.0[o * 2..o * 2 + 2], 16).unwrap() as f32;
        (base_ch * (1.0 - alpha as f32) + over_ch * alpha as f32) / 255.0
    };
    Rgba {
        r: mix(0, 0),
        g: mix(1, 1),
        b: mix(2, 2),
        a: 1.0,
    }
}

const fn rgba_8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

impl Palette {
    /// The supported dark theme at standard contrast.
    pub fn dark_standard() -> Self {
        Self {
            ink: INK_DARK.rgba(),
            ink_secondary: SECONDARY_DARK.rgba(),
            ink_faint: FAINT_DARK.rgba(),
            hairline: HAIRLINE_DARK.rgba(),
            border_strong: BORDER_STRONG_DARK.rgba(),
            accent: ACCENT_DARK.rgba(),
            accent_ink: ACCENT_INK_DARK.rgba(),
            status_error: ERROR_DARK.rgba(),
            status_attention: ATTENTION_DARK.rgba(),
            status_ok: OK_DARK.rgba(),
            status_info: INFO_DARK.rgba(),
            status_error_surface: rgba_8(255, 106, 99, 25),
            status_attention_surface: rgba_8(255, 198, 63, 25),
            tint_selected: TINT_SELECTED_DARK.rgba(),
            surface_canvas: SURFACE_CANVAS_DARK.rgba(),
            surface_sidebar: SURFACE_SIDEBAR_DARK.rgba(),
            surface_field: SURFACE_FIELD_DARK.rgba(),
            surface_overlay: SURFACE_OVERLAY_DARK.rgba(),
            surface_glass: SURFACE_GLASS_DARK.rgba(),
            surface_favorite: SURFACE_FAVORITE_DARK.rgba(),
            surface_footer: SURFACE_FOOTER_DARK.rgba(),
            wash_hover: blend(SURFACE_SIDEBAR_DARK, INK_DARK, 0.05),
            wash_active: blend(SURFACE_SIDEBAR_DARK, INK_DARK, 0.08),
            // §11.5: white-translucent over the dark sidebar; the hover
            // steps one fill brighter (a higher white alpha).
            address_fill: rgba_8(255, 255, 255, 25),
            address_fill_hover: rgba_8(255, 255, 255, 41),
            surface_error_ic: rgba_8(255, 106, 99, 30),
            surface_attention_ic: rgba_8(255, 198, 63, 30),
        }
    }

    /// The dark theme at increased contrast (§3.4).
    pub fn dark_increased() -> Self {
        let base = Self::dark_standard();
        Palette {
            ink: INK_IC_DARK.rgba(),
            ink_secondary: SECONDARY_IC_DARK.rgba(),
            ink_faint: FAINT_IC_DARK.rgba(),
            hairline: HAIRLINE_IC_DARK.rgba(),
            border_strong: BORDER_STRONG_IC_DARK.rgba(),
            accent: ACCENT_IC_DARK.rgba(),
            accent_ink: ACCENT_INK_IC_DARK.rgba(),
            tint_selected: TINT_SELECTED_IC_DARK.rgba(),
            wash_hover: blend(SURFACE_SIDEBAR_DARK, INK_DARK, 0.07),
            wash_active: blend(SURFACE_SIDEBAR_DARK, INK_DARK, 0.10),
            // Increased contrast hardens the translucent address fill the
            // same way it hardens the row washes.
            address_fill: rgba_8(255, 255, 255, 30),
            address_fill_hover: rgba_8(255, 255, 255, 48),
            ..base
        }
    }

    /// The light theme at standard contrast.
    pub fn light_standard() -> Self {
        Self {
            ink: INK_LIGHT.rgba(),
            ink_secondary: SECONDARY_LIGHT.rgba(),
            ink_faint: FAINT_LIGHT.rgba(),
            hairline: HAIRLINE_LIGHT.rgba(),
            border_strong: BORDER_STRONG_LIGHT.rgba(),
            accent: ACCENT_LIGHT.rgba(),
            accent_ink: ACCENT_INK_LIGHT.rgba(),
            status_error: ERROR_LIGHT.rgba(),
            status_attention: ATTENTION_LIGHT.rgba(),
            status_ok: OK_LIGHT.rgba(),
            status_info: INFO_LIGHT.rgba(),
            status_error_surface: rgba_8(194, 46, 36, 25),
            status_attention_surface: rgba_8(143, 92, 0, 25),
            tint_selected: TINT_SELECTED_LIGHT.rgba(),
            surface_canvas: SURFACE_CANVAS_LIGHT.rgba(),
            surface_sidebar: SURFACE_SIDEBAR_LIGHT.rgba(),
            surface_field: SURFACE_FIELD_LIGHT.rgba(),
            surface_overlay: SURFACE_OVERLAY_LIGHT.rgba(),
            surface_glass: SURFACE_GLASS_LIGHT.rgba(),
            surface_favorite: SURFACE_FAVORITE_LIGHT.rgba(),
            surface_footer: SURFACE_FOOTER_LIGHT.rgba(),
            wash_hover: blend(SURFACE_SIDEBAR_LIGHT, INK_LIGHT, 0.045),
            wash_active: blend(SURFACE_SIDEBAR_LIGHT, INK_LIGHT, 0.08),
            // §11.5: black-translucent over the light sidebar; the hover
            // steps one fill darker (a higher black alpha).
            address_fill: rgba_8(0, 0, 0, 15),
            address_fill_hover: rgba_8(0, 0, 0, 26),
            surface_error_ic: rgba_8(194, 46, 36, 30),
            surface_attention_ic: rgba_8(143, 92, 0, 30),
        }
    }

    /// The light theme at increased contrast.
    pub fn light_increased() -> Self {
        let base = Self::light_standard();
        Palette {
            ink: INK_IC_LIGHT.rgba(),
            ink_secondary: SECONDARY_IC_LIGHT.rgba(),
            ink_faint: FAINT_IC_LIGHT.rgba(),
            hairline: HAIRLINE_IC_LIGHT.rgba(),
            border_strong: BORDER_STRONG_IC_LIGHT.rgba(),
            accent: ACCENT_IC_LIGHT.rgba(),
            accent_ink: ACCENT_INK_IC_LIGHT.rgba(),
            tint_selected: TINT_SELECTED_IC_LIGHT.rgba(),
            // Increased contrast deepens the translucent address fill.
            address_fill: rgba_8(0, 0, 0, 18),
            address_fill_hover: rgba_8(0, 0, 0, 30),
            ..base
        }
    }
}

/// Deterministic monogram tile ramp for missing favicons (DESIGN.md §7.2):
/// six desaturated slate hues separate from accent/status/profile identity.
#[derive(Clone, Copy, Debug)]
pub struct FaviconSlate;
impl FaviconSlate {
    /// Tile background for `hue_index` (0..6) in the dark shell.
    pub fn dark(hue_index: usize) -> Rgba {
        match hue_index % 6 {
            0 => rgb(0x54606E),
            1 => rgb(0x5B6775),
            2 => rgb(0x4E5A68),
            3 => rgb(0x606B77),
            4 => rgb(0x59606A),
            _ => rgb(0x4A5662),
        }
    }

    /// Tile background in the light shell (lighter slate ramp).
    pub fn light(hue_index: usize) -> Rgba {
        match hue_index % 6 {
            0 => rgb(0xC3CBD4),
            1 => rgb(0xC9D0D8),
            2 => rgb(0xBDC6CF),
            3 => rgb(0xD0D6DD),
            4 => rgb(0xC6CCD3),
            _ => rgb(0xB6C0CA),
        }
    }
}

/// One §12 profile hue's rendered tile + letter pair for a theme mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileHueTile {
    pub tile: Rgba,
    pub letter: Rgba,
}

/// The six DESIGN.md §12 profile hue tiles (light/dark pairs). The identity
/// hue name comes from the portable domain ([`bbr_domain::ProfileHue`]);
/// the concrete colors live here. Tile/letter pairs are authored ≥ 4.5:1
/// and the gate recomputes that from the raw hex tokens below.
#[derive(Clone, Copy, Debug)]
pub struct ProfileHueRamp;

impl ProfileHueRamp {
    pub fn tile(hue: ProfileHue, mode: ThemeMode) -> ProfileHueTile {
        let (tile, letter) = match (hue, mode) {
            (ProfileHue::Cobalt, ThemeMode::Light) => (0x4062E8, 0xFFFFFF),
            (ProfileHue::Cobalt, ThemeMode::Dark) => (0x6E8BFF, 0xFFFFFF),
            (ProfileHue::Violet, ThemeMode::Light) => (0x7C4DF0, 0xFFFFFF),
            (ProfileHue::Violet, ThemeMode::Dark) => (0x8A63F0, 0xFFFFFF),
            (ProfileHue::Fuchsia, ThemeMode::Light) => (0xC63B8F, 0xFFFFFF),
            (ProfileHue::Fuchsia, ThemeMode::Dark) => (0xB02E7E, 0xFFFFFF),
            (ProfileHue::Amber, ThemeMode::Light) => (0xB57912, 0x1A0D00),
            (ProfileHue::Amber, ThemeMode::Dark) => (0xC0861F, 0xFFFFFF),
            (ProfileHue::Terracotta, ThemeMode::Light) => (0xB4582C, 0xFFFFFF),
            (ProfileHue::Terracotta, ThemeMode::Dark) => (0xB85A30, 0xFFFFFF),
            (ProfileHue::Slate, ThemeMode::Light) => (0x5B6470, 0xFFFFFF),
            (ProfileHue::Slate, ThemeMode::Dark) => (0x8B94A2, 0xFFFFFF),
        };
        ProfileHueTile {
            tile: rgb(tile),
            letter: rgb(letter),
        }
    }
}

#[cfg(test)]
mod tests;
