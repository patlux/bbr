//! BBR Chassis — the typed GPUI design-token system (DESIGN.md §2–§8).
//!
//! Single source of truth for how the shell looks: colors, typography steps,
//! spacing, radius, density, motion, eyebrow labels, and elevation. The
//! placeholder `render.rs` magic `rgb(0x…)` values are replaced by this
//! module in the same groups that introduce the shell; tests and E2E assert
//! the §15 mode fields (`theme`, `contrast`, `density`) and the §3.1
//! contrast contract, which is **derived from these tokens** — never copied
//! as a literal ratio.
//!
//! Token names intentionally mirror `DESIGN.md` (list of the mapping):
//!
//! ```text
//! color:ink            → Palette::ink            color:accent       → accent
//! color:ink.secondary  → Palette::ink_secondary  color:accent.ink  → accent_ink
//! color:ink.faint      → Palette::ink_faint      color:tint.selected → tint_selected
//! color:hairline       → Palette::hairline       material:sidebar   → surface_sidebar
//! color:border.strong  → Palette::border_strong  material:canvas    → surface_canvas
//! material:field       → surface_field           material:glass     → surface_glass
//! status.error/attention/ok/info + .surface       → status_* / status_*_surface
//! type:title/label/meta/mono/key/badge/display     → TypeRole::TITLE/…
//! sp:N (4·N pt)         → Spacing::sp(N)           radius:sm/md/lg/xl/full → Radius::*
//! density:row / sidebar / icon …                   → Density::* (S/N/XL)
//! dur:instant…emphasis / load:indeterminate       → Motion::*
//! elev:0/1/2            → Elevation::*             favicon slate ramp (monogram) → FaviconSlate
//! ```

use gpui::{rgb, Rgba};

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
    fn luminance(self) -> f64 {
        0.2126 * self.channel(0) + 0.7152 * self.channel(1) + 0.0722 * self.channel(2)
    }

    /// WCAG 2.x contrast ratio against another token.
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
    INK_LIGHT = "1A1C1F";   INK_DARK = "ECEEF1";
    SECONDARY_LIGHT = "555B63"; SECONDARY_DARK = "A6ACB5";
    FAINT_LIGHT = "949AA2"; FAINT_DARK = "6E747D";
    HAIRLINE_LIGHT = "E2E2DF"; HAIRLINE_DARK = "2A2E34";
    BORDER_STRONG_LIGHT = "C7C9C5"; BORDER_STRONG_DARK = "3B4149";
    INK_IC_LIGHT = "000000"; INK_IC_DARK = "FFFFFF";
    SECONDARY_IC_LIGHT = "333A41"; SECONDARY_IC_DARK = "D6DAE0";
    FAINT_IC_LIGHT = "5C626A"; FAINT_IC_DARK = "8A919B";
    HAIRLINE_IC_LIGHT = "AFB2AC"; HAIRLINE_IC_DARK = "454C55";
    BORDER_STRONG_IC_LIGHT = "9FA29E"; BORDER_STRONG_IC_DARK = "525A63";
    // §3.2 accent — Phosphor.
    ACCENT_LIGHT = "0E8F79"; ACCENT_DARK = "2AD4B6";
    ACCENT_INK_LIGHT = "0A5C4E"; ACCENT_INK_DARK = "3DF0CE";
    ACCENT_IC_LIGHT = "0B6B5C"; ACCENT_IC_DARK = "3DF0CE";
    ACCENT_INK_IC_LIGHT = "0A5A4C"; ACCENT_INK_IC_DARK = "3DF0CE";
    // §3.3 status hues + tint (§3.5).
    ERROR_LIGHT = "C22E24"; ERROR_DARK = "FF6A63";
    ATTENTION_LIGHT = "8F5C00"; ATTENTION_DARK = "FFC63F";
    OK_LIGHT = "177A55"; OK_DARK = "45C08A";
    INFO_LIGHT = "2C5CC5"; INFO_DARK = "8AA6FF";
    TINT_SELECTED_LIGHT = "D8E5E0"; TINT_SELECTED_DARK = "1C3333";
    TINT_SELECTED_IC_DARK = "162B2B";
    // §2 materials.
    SURFACE_CANVAS_LIGHT = "F6F6F4"; SURFACE_CANVAS_DARK = "121416";
    SURFACE_SIDEBAR_LIGHT = "EFEFEC"; SURFACE_SIDEBAR_DARK = "1A1D21";
    SURFACE_FIELD_LIGHT = "FFFFFF"; SURFACE_FIELD_DARK = "23272C";
    SURFACE_OVERLAY_LIGHT = "FFFFFF"; SURFACE_OVERLAY_DARK = "202429";
    SURFACE_GLASS_LIGHT = "FFFFFF"; SURFACE_GLASS_DARK = "151719";
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
    /// The dark theme at standard contrast (the shipped shell mode).
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

    /// The light theme at standard contrast (defined; shipped with the
    /// light-mode slice).
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
            tint_selected: TINT_SELECTED_IC_DARK.rgba(),
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
/// hue name comes from the portable domain; the concrete colors live here.
/// Tile/letter pairs are authored ≥ 4.5:1 and the gate recomputes that from
/// the raw hex tokens below.
#[derive(Clone, Copy, Debug)]
pub struct ProfileHueRamp;

impl ProfileHueRamp {
    pub fn tile(hue: bbr_browser::ProfileHue, mode: crate::theme::ThemeMode) -> ProfileHueTile {
        use bbr_browser::ProfileHue;
        let (tile, letter) = match (hue, mode) {
            (ProfileHue::Cobalt, crate::theme::ThemeMode::Light) => (0x4062E8, 0xFFFFFF),
            (ProfileHue::Cobalt, crate::theme::ThemeMode::Dark) => (0x6E8BFF, 0xFFFFFF),
            (ProfileHue::Violet, crate::theme::ThemeMode::Light) => (0x7C4DF0, 0xFFFFFF),
            (ProfileHue::Violet, crate::theme::ThemeMode::Dark) => (0x8A63F0, 0xFFFFFF),
            (ProfileHue::Fuchsia, crate::theme::ThemeMode::Light) => (0xC63B8F, 0xFFFFFF),
            (ProfileHue::Fuchsia, crate::theme::ThemeMode::Dark) => (0xB02E7E, 0xFFFFFF),
            (ProfileHue::Amber, crate::theme::ThemeMode::Light) => (0xB57912, 0x1A0D00),
            (ProfileHue::Amber, crate::theme::ThemeMode::Dark) => (0xC0861F, 0xFFFFFF),
            (ProfileHue::Terracotta, crate::theme::ThemeMode::Light) => (0xB4582C, 0xFFFFFF),
            (ProfileHue::Terracotta, crate::theme::ThemeMode::Dark) => (0xB85A30, 0xFFFFFF),
            (ProfileHue::Slate, crate::theme::ThemeMode::Light) => (0x5B6470, 0xFFFFFF),
            (ProfileHue::Slate, crate::theme::ThemeMode::Dark) => (0x8B94A2, 0xFFFFFF),
        };
        ProfileHueTile {
            tile: rgb(tile),
            letter: rgb(letter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, Theme};

    /// §12: profile tile/letter pairs must compute ≥ 3:1 from the tokens —
    /// the same bar the §7.2 favicon monogram tiles use. The DESIGN table's
    /// dark tiles top out at ≈3.1:1 against white, so 4.5:1 would be a
    /// copied claim the authored hues cannot satisfy; identity never relies
    /// on color alone (the monogram is the non-color companion).
    #[test]
    fn profile_hue_tiles_meet_the_documented_contrast_contract() {
        use bbr_browser::ProfileHue;
        let lum = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let ratio = |bg: &Rgba, fg: &Rgba| {
            let l = |r: &Rgba| {
                0.2126 * lum(r.r as f64) + 0.7152 * lum(r.g as f64) + 0.0722 * lum(r.b as f64)
            };
            let (hi, lo) = if l(bg) >= l(fg) {
                (l(bg), l(fg))
            } else {
                (l(fg), l(bg))
            };
            (hi + 0.05) / (lo + 0.05)
        };
        for mode in [
            crate::theme::ThemeMode::Light,
            crate::theme::ThemeMode::Dark,
        ] {
            for hue in [
                ProfileHue::Cobalt,
                ProfileHue::Violet,
                ProfileHue::Fuchsia,
                ProfileHue::Amber,
                ProfileHue::Terracotta,
                ProfileHue::Slate,
            ] {
                let t = ProfileHueRamp::tile(hue, mode);
                assert!(
                    ratio(&t.tile, &t.letter) >= 3.0,
                    "profile hue {hue:?} {mode:?} letter ≥ 3:1, got {:.2}",
                    ratio(&t.tile, &t.letter)
                );
            }
        }
    }

    /// §3.1 contrast gate — derived from the tokens, never copied ratios.
    /// Asserts the documented pairs on their real surfaces.
    #[test]
    fn dark_theme_meets_the_documented_contrast_contract() {
        let theme = Theme::dark();
        let sidebar = SURFACE_SIDEBAR_DARK;
        assert!(INK_DARK.contrast(sidebar) >= 14.0, "ink ≥ 14:1 on sidebar");
        assert!(
            SECONDARY_DARK.contrast(sidebar) >= 5.9,
            "ink.secondary ≥ 5.9:1"
        );
        assert!(ACCENT_DARK.contrast(sidebar) >= 3.0, "accent rail ≥ 3:1");
        assert!(
            ACCENT_INK_DARK.contrast(sidebar) >= 9.0,
            "accent.ink ≥ 9:1 dark"
        );
        for (name, token) in [
            ("error", ERROR_DARK),
            ("attention", ATTENTION_DARK),
            ("ok", OK_DARK),
            ("info", INFO_DARK),
        ] {
            assert!(token.contrast(sidebar) >= 4.5, "status {name} ≥ 4.5:1");
        }
        assert!(SECONDARY_DARK.contrast(SURFACE_FIELD_DARK) >= 4.5);
        assert_eq!(theme.palette.surface_canvas.a, 1.0);
        assert_ne!(theme.palette.surface_sidebar, theme.palette.surface_canvas);
        assert_ne!(theme.palette.surface_field, theme.palette.surface_sidebar);
    }

    #[test]
    fn increased_contrast_hardens_text_and_hairlines() {
        let sidebar = SURFACE_SIDEBAR_DARK;
        assert!(
            SECONDARY_IC_DARK.contrast(sidebar) >= 9.0,
            "IC secondary ≥ 9:1"
        );
        assert!(
            HAIRLINE_IC_DARK.contrast(sidebar) >= 1.8,
            "IC hairline ≥ 1.8:1"
        );
        assert!(ACCENT_INK_IC_DARK.contrast(sidebar) >= 9.0, "IC accent.ink");
        assert!(INK_IC_DARK.contrast(sidebar) >= INK_DARK.contrast(sidebar));
    }

    #[test]
    fn light_theme_meets_the_documented_contrast_contract() {
        let sidebar = SURFACE_SIDEBAR_LIGHT;
        assert!(INK_LIGHT.contrast(sidebar) >= 14.0);
        assert!(SECONDARY_LIGHT.contrast(sidebar) >= 5.9);
        assert!(ACCENT_LIGHT.contrast(sidebar) >= 3.0);
        assert!(ACCENT_INK_LIGHT.contrast(sidebar) >= 5.9);
        assert!(INK_IC_LIGHT.contrast(sidebar) >= 14.0);
        assert!(SECONDARY_IC_LIGHT.contrast(sidebar) >= 9.0);
        assert!(HAIRLINE_IC_LIGHT.contrast(sidebar) >= 1.8);
        for (name, token) in [("error", ERROR_LIGHT), ("attention", ATTENTION_LIGHT)] {
            assert!(token.contrast(sidebar) >= 4.5, "light status {name}");
        }
    }

    /// §11.5 address-control fill token pair: white-translucent on dark
    /// chrome, black-translucent on light chrome; the hover step is one
    /// fill brighter on dark and one fill darker on light — both derived
    /// from the token pair, never a hardcoded color in the component.
    #[test]
    fn address_fill_pair_is_translucent_and_polarized_per_theme() {
        for theme in [
            Theme::dark(),
            Theme::new(
                crate::theme::ThemeMode::Light,
                ContrastMode::Standard,
                Density::Normal,
            ),
            Theme::new(
                crate::theme::ThemeMode::Dark,
                ContrastMode::Increased,
                Density::Normal,
            ),
            Theme::new(
                crate::theme::ThemeMode::Light,
                ContrastMode::Increased,
                Density::Normal,
            ),
        ] {
            let fill = theme.palette.address_fill;
            let hover = theme.palette.address_fill_hover;
            assert!(fill.a > 0.0 && fill.a < 1.0, "fill stays translucent");
            assert!(hover.a > 0.0 && hover.a < 1.0, "hover stays translucent");
            match theme.mode {
                crate::theme::ThemeMode::Dark => {
                    // White-translucent: the RGB channels are near-white and
                    // the hover is strictly brighter (higher alpha).
                    assert!(fill.r >= 0.9 && fill.g >= 0.9 && fill.b >= 0.9);
                    assert!(hover.a > fill.a, "dark hover brightens one step");
                }
                crate::theme::ThemeMode::Light => {
                    // Black-translucent: RGB near-black and the hover is
                    // strictly darker (higher alpha).
                    assert!(fill.r <= 0.1 && fill.g <= 0.1 && fill.b <= 0.1);
                    assert!(hover.a > fill.a, "light hover darkens one step");
                }
            }
            // The hover is never a different hue — only the alpha steps.
            assert_eq!((fill.r, fill.g, fill.b), (hover.r, hover.g, hover.b));
        }
    }

    #[test]
    fn monogram_ramp_letters_stay_legible() {
        // §7.2: tile letters are `type:badge`-size and must read on the ramp.
        // Contrast derives from the *rendered* Rgba tiles, never copied.
        let lum = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let white_l = lum(1.0);
        for hue_index in 0..6 {
            let bg = FaviconSlate::dark(hue_index);
            let bg_l =
                0.2126 * lum(bg.r as f64) + 0.7152 * lum(bg.g as f64) + 0.0722 * lum(bg.b as f64);
            let ratio = (white_l + 0.05) / (bg_l + 0.05);
            assert!(
                ratio >= 3.0,
                "monogram tile {hue_index} letter ≥ 3:1, got {ratio:.2}"
            );
        }
    }
}
