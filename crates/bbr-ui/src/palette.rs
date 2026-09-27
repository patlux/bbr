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
//! color:tint.accent    → Palette::tint_accent
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
    HAIRLINE_LIGHT = "D0D0CC"; HAIRLINE_DARK = "2C2F35";
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
    // §3.5/§18.2 (D1): the selection lifts *above* the rail — opaque white
    // on light; the dark standard selection is a white veil (see
    // `SELECTED_VEIL_DARK`), the dark IC selection this opaque step.
    TINT_SELECTED_LIGHT = "FFFFFF"; TINT_SELECTED_IC_DARK = "35383F";
    // §18.2 `tint.accent`: the one accent moment on white/overlay surfaces.
    TINT_ACCENT_LIGHT = "E6F1EE"; TINT_ACCENT_DARK = "26403F";
    TINT_ACCENT_IC_LIGHT = "D9EAE5"; TINT_ACCENT_IC_DARK = "2B4A48";
    // §2/§18.2 materials.
    SURFACE_CANVAS_LIGHT = "F6F6F4"; SURFACE_CANVAS_DARK = "131517";
    SURFACE_SIDEBAR_LIGHT = "E5E5E2"; SURFACE_SIDEBAR_DARK = "1C1E22";
    SURFACE_FIELD_LIGHT = "FFFFFF"; SURFACE_FIELD_DARK = "2A2D33";
    SURFACE_OVERLAY_LIGHT = "FFFFFF"; SURFACE_OVERLAY_DARK = "25282D";
    SURFACE_GLASS_LIGHT = "FFFFFF"; SURFACE_GLASS_DARK = "151719";
}

/// §18.2 translucent tokens are veils painted over the (possibly
/// space-tinted) rail, so one value follows every tint. Alphas are the
/// DESIGN percentages on the 0–255 scale.
const fn white(alpha: u8) -> Rgba {
    rgba_8(255, 255, 255, alpha)
}

/// `color:ink` (light) as a translucent wash/border colour.
const fn ink_veil(alpha: u8) -> Rgba {
    rgba_8(0x11, 0x13, 0x15, alpha)
}

/// Dark standard selection: white 9 % over the rail (D1).
const SELECTED_VEIL_DARK: Rgba = white(23);

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
    /// Text and glyphs painted *on* an accent fill (§3.2 `color:on-accent`):
    /// primary buttons such as Reload, New Tab, Log In and Done. Never
    /// `accent_ink`, which is text on a neutral surface.
    pub on_accent: Rgba,
    // Status hues (§3.3): only on small glyph/dot surfaces with a
    // non-color companion.
    pub status_error: Rgba,
    pub status_attention: Rgba,
    pub status_ok: Rgba,
    pub status_info: Rgba,
    /// Status color at 10% over the owning material (12% in IC).
    pub status_error_surface: Rgba,
    pub status_attention_surface: Rgba,
    /// §3.5/§18.2 (D1) selected-row surface: it lifts *above* the rail —
    /// opaque white on light, a white veil on dark (opaque in dark IC).
    /// Composite over the rail before computing contrast ([`over`]).
    pub tint_selected: Rgba,
    /// §18.2 `color:tint.accent`: the single accent moment on white and
    /// overlay surfaces — command results, menu and Library cursors, drop
    /// targets. Text on it is `ink` / `accent.ink`.
    pub tint_accent: Rgba,
    // Materials (§2): canvas behind glass, the sidebar rail, raised fields,
    // floating overlays, and what sits behind native web views.
    pub surface_canvas: Rgba,
    pub surface_sidebar: Rgba,
    pub surface_field: Rgba,
    pub surface_overlay: Rgba,
    pub surface_glass: Rgba,
    /// §18.2 Favorite destination tile: a white veil over the rail.
    pub surface_favorite: Rgba,
    /// §18.2 footer / media-card surface: a white veil over the rail.
    pub surface_footer: Rgba,
    /// Row hover wash (§10.2): a translucent ink (light) / white (dark)
    /// veil over whatever surface the row sits on; IC deepens.
    pub wash_hover: Rgba,
    /// Pointer-down / pressed wash (§10.3), translucent like the hover.
    pub wash_active: Rgba,
    /// §11.5 address field fill: a white veil over the rail material — one
    /// step lighter than the rail in both modes — so the field follows the
    /// theme and the space tint through the material beneath (never a
    /// hardcoded surface color).
    pub address_fill: Rgba,
    /// §11.5 address-field hover: one step darker on light chrome, one step
    /// brighter on dark chrome (§10.2 wash mechanics at `dur:fine`); the
    /// border never changes.
    pub address_fill_hover: Rgba,
    /// §11.5 address-field border: hairline-strength presence, not contrast.
    pub address_border: Rgba,
    /// §11.5 trailing accessory (zoom chip) inset wash.
    pub address_accessory_fill: Rgba,
    /// The two status surface washes at the IC level.
    pub surface_error_ic: Rgba,
    pub surface_attention_ic: Rgba,
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
            on_accent: SURFACE_GLASS_DARK.rgba(),
            status_error: ERROR_DARK.rgba(),
            status_attention: ATTENTION_DARK.rgba(),
            status_ok: OK_DARK.rgba(),
            status_info: INFO_DARK.rgba(),
            status_error_surface: rgba_8(255, 106, 99, 25),
            status_attention_surface: rgba_8(255, 198, 63, 25),
            tint_selected: SELECTED_VEIL_DARK,
            tint_accent: TINT_ACCENT_DARK.rgba(),
            surface_canvas: SURFACE_CANVAS_DARK.rgba(),
            surface_sidebar: SURFACE_SIDEBAR_DARK.rgba(),
            surface_field: SURFACE_FIELD_DARK.rgba(),
            surface_overlay: SURFACE_OVERLAY_DARK.rgba(),
            surface_glass: SURFACE_GLASS_DARK.rgba(),
            surface_favorite: white(15),
            // Paper's 7 % missed `ink.secondary ≥ 5.9:1` on the Slate-tinted
            // rail (5.73); 5 % holds it (§18.2 records the correction).
            surface_footer: white(13),
            wash_hover: white(13),
            wash_active: white(20),
            // §11.5: white veil over the dark rail; the hover steps one
            // fill brighter (a higher white alpha).
            address_fill: white(15),
            address_fill_hover: white(26),
            address_border: white(10),
            address_accessory_fill: white(15),
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
            tint_accent: TINT_ACCENT_IC_DARK.rgba(),
            wash_hover: white(20),
            wash_active: white(31),
            // Increased contrast hardens the address veils the same way it
            // hardens the row washes and hairlines.
            address_fill: white(20),
            address_fill_hover: white(31),
            address_border: white(26),
            address_accessory_fill: white(20),
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
            on_accent: SURFACE_GLASS_LIGHT.rgba(),
            status_error: ERROR_LIGHT.rgba(),
            status_attention: ATTENTION_LIGHT.rgba(),
            status_ok: OK_LIGHT.rgba(),
            status_info: INFO_LIGHT.rgba(),
            status_error_surface: rgba_8(194, 46, 36, 25),
            status_attention_surface: rgba_8(143, 92, 0, 25),
            tint_selected: TINT_SELECTED_LIGHT.rgba(),
            tint_accent: TINT_ACCENT_LIGHT.rgba(),
            surface_canvas: SURFACE_CANVAS_LIGHT.rgba(),
            surface_sidebar: SURFACE_SIDEBAR_LIGHT.rgba(),
            surface_field: SURFACE_FIELD_LIGHT.rgba(),
            surface_overlay: SURFACE_OVERLAY_LIGHT.rgba(),
            surface_glass: SURFACE_GLASS_LIGHT.rgba(),
            surface_favorite: white(128),
            surface_footer: white(158),
            wash_hover: ink_veil(13),
            wash_active: ink_veil(23),
            // §11.5: a white veil over the light rail; the hover steps one
            // fill darker (a lower white alpha lets more rail through).
            address_fill: white(140),
            address_fill_hover: white(102),
            address_border: ink_veil(10),
            address_accessory_fill: ink_veil(15),
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
            // The light selection is already opaque white + lift; IC keeps it
            // and hardens everything around it.
            tint_accent: TINT_ACCENT_IC_LIGHT.rgba(),
            // §3.4/§10.2: increased contrast deepens the row washes (8 % /
            // 12 %) exactly like the dark theme.
            wash_hover: ink_veil(20),
            wash_active: ink_veil(31),
            address_border: ink_veil(26),
            address_accessory_fill: ink_veil(20),
            ..base
        }
    }
}

/// Source-over composite of a (possibly translucent) token onto an opaque
/// surface — the colour the user sees, and the one every contrast gate on a
/// §18.2 veil must compute against.
pub fn over(top: Rgba, base: Rgba) -> Rgba {
    let mix = |t: f32, b: f32| top.a * t + (1.0 - top.a) * b;
    Rgba {
        r: mix(top.r, base.r),
        g: mix(top.g, base.g),
        b: mix(top.b, base.b),
        a: 1.0,
    }
}

/// WCAG 2.x contrast ratio between two rendered colors (alpha ignored).
///
/// The §3.1 token gate computes on the raw hex tokens; this helper lets
/// component tests prove that the *pair a component actually paints*
/// (e.g. primary-button text on its fill) meets the same bars.
pub fn contrast(a: Rgba, b: Rgba) -> f64 {
    fn linear(channel: f32) -> f64 {
        let value = f64::from(channel);
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(color: Rgba) -> f64 {
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
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

    /// The monogram letter color for the mode's ramp: ink on the light
    /// ramp, white on the dark ramp. One authority for every monogram tile
    /// (tab rows, Favorites, folders, Library) so the §12/§7.2 letter bar
    /// holds everywhere.
    pub fn letter(mode: ThemeMode) -> Rgba {
        match mode {
            ThemeMode::Light => INK_LIGHT.rgba(),
            ThemeMode::Dark => rgb(0xFFFFFF),
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
