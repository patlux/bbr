//! Palette contrast-gate tests (DESIGN.md §3.1/§3.5, §12): every pair is
//! recomputed from the stored hex tokens — never a copied ratio.

use super::*;
use crate::theme::Theme;

/// §12: profile tile/letter pairs must compute ≥ 3:1 from the tokens —
/// the same bar the §7.2 favicon monogram tiles use. The DESIGN table's
/// dark tiles top out at ≈3.1:1 against white, so 4.5:1 would be a
/// copied claim the authored hues cannot satisfy; identity never relies
/// on color alone (the monogram is the non-color companion).
#[test]
fn profile_hue_tiles_meet_the_documented_contrast_contract() {
    use bbr_domain::ProfileHue;
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
    // Reference-calibrated dark surfaces (task packet
    // `system-appearance-theme`): the selected row renders ink title +
    // ink.secondary meta on `tint.selected` (§10.4), and the footer strip
    // renders its glyphs at ink.secondary on `material:footer` (§11.4).
    assert!(
        INK_DARK.contrast(TINT_SELECTED_DARK) >= 14.0,
        "selected-row ink ≥ 14:1 on the dark selection tint"
    );
    assert!(
        SECONDARY_DARK.contrast(TINT_SELECTED_DARK) >= 5.9,
        "selected-row meta ≥ 5.9:1 on the dark selection tint"
    );
    assert!(
        SECONDARY_DARK.contrast(SURFACE_FOOTER_DARK) >= 5.9,
        "footer glyphs ≥ 5.9:1 on the dark footer material"
    );
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
    // IC deepens the selection tint (§3.4): the IC dark tint is deeper
    // than the standard dark tint and still holds the §3.1 text bars
    // (ink ≥ 14:1, IC secondary ≥ 9:1 on the tint; footer ≥ 9:1).
    assert!(
        INK_IC_DARK.contrast(TINT_SELECTED_IC_DARK) >= 14.0,
        "IC selected-row ink ≥ 14:1"
    );
    assert!(
        SECONDARY_IC_DARK.contrast(TINT_SELECTED_IC_DARK) >= 9.0,
        "IC selected-row meta ≥ 9:1"
    );
    assert!(
        SECONDARY_IC_DARK.contrast(SURFACE_FOOTER_DARK) >= 9.0,
        "IC footer glyphs ≥ 9:1"
    );
    assert!(
        TINT_SELECTED_IC_DARK.luminance() < TINT_SELECTED_DARK.luminance(),
        "increased contrast deepens the selection tint"
    );
}

#[test]
fn dark_selection_tint_is_visible_on_the_rail() {
    // Regression: #0D2020 sat 0.1 L* *above* the dark rail — the selected
    // surface was invisible and only the 600 title weight marked selection.
    // The §3.1 ink bar caps the tint just above the rail's luminance, so the
    // visible surface sits below it: at least 4 CIE L* units, like the hover
    // wash above it.
    fn lightness(hex: Hex) -> f64 {
        let y = hex.luminance();
        if y > 0.008856 {
            116.0 * y.cbrt() - 16.0
        } else {
            903.3 * y
        }
    }
    let rail = lightness(SURFACE_SIDEBAR_DARK);
    for (name, tint) in [
        ("standard", TINT_SELECTED_DARK),
        ("increased contrast", TINT_SELECTED_IC_DARK),
    ] {
        let delta = rail - lightness(tint);
        assert!(
            delta >= 4.0,
            "{name} dark selection is {delta:.2} L* below the rail"
        );
    }
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
    // Reference-calibrated light surfaces (task packet
    // `system-appearance-theme`): the same §3.1 pairs the dark side
    // asserts — selected-row ink/meta on `tint.selected` (§10.4) and
    // footer glyphs on `material:footer` (§11.4). Light stays the
    // calibrated reference and must keep passing.
    assert!(INK_LIGHT.contrast(TINT_SELECTED_LIGHT) >= 14.0);
    assert!(SECONDARY_LIGHT.contrast(TINT_SELECTED_LIGHT) >= 5.9);
    assert!(SECONDARY_LIGHT.contrast(SURFACE_FOOTER_LIGHT) >= 5.9);
    assert!(INK_IC_LIGHT.contrast(TINT_SELECTED_IC_LIGHT) >= 14.0);
    assert!(SECONDARY_IC_LIGHT.contrast(TINT_SELECTED_IC_LIGHT) >= 9.0);
    assert!(SECONDARY_IC_LIGHT.contrast(SURFACE_FOOTER_LIGHT) >= 9.0);
}

/// §11.5 bordered address field: a white veil over the (tinted) rail —
/// one step lighter than the rail in both modes, so it follows the space
/// tint — with a hairline-strength border. The hover steps darker on light
/// chrome and brighter on dark chrome; domain ink keeps the §3.1 bar on the
/// composited field in every palette.
#[test]
fn address_field_is_a_bordered_veil_one_step_off_the_rail() {
    let over = |veil: gpui::Rgba, base: gpui::Rgba| gpui::Rgba {
        r: veil.a * veil.r + (1.0 - veil.a) * base.r,
        g: veil.a * veil.g + (1.0 - veil.a) * base.g,
        b: veil.a * veil.b + (1.0 - veil.a) * base.b,
        a: 1.0,
    };
    for (mode, p) in [
        (crate::theme::ThemeMode::Light, Palette::light_standard()),
        (crate::theme::ThemeMode::Light, Palette::light_increased()),
        (crate::theme::ThemeMode::Dark, Palette::dark_standard()),
        (crate::theme::ThemeMode::Dark, Palette::dark_increased()),
    ] {
        let (fill, hover) = (p.address_fill, p.address_fill_hover);
        for veil in [fill, hover] {
            assert!(veil.a > 0.0 && veil.a < 1.0, "a veil, never an opaque hex");
            assert!(veil.r >= 0.99 && veil.g >= 0.99 && veil.b >= 0.99);
            let field = over(veil, p.surface_sidebar);
            assert!(
                crate::palette::contrast(p.ink, field) >= 4.5,
                "domain ink on the field"
            );
        }
        match mode {
            crate::theme::ThemeMode::Light => assert!(hover.a < fill.a, "light hover darkens"),
            crate::theme::ThemeMode::Dark => assert!(hover.a > fill.a, "dark hover brightens"),
        }
        assert!(
            p.address_border.a > 0.0 && p.address_border.a < 0.2,
            "border is presence, not contrast"
        );
        assert!(p.address_accessory_fill.a > 0.0 && p.address_accessory_fill.a < 0.1);
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

#[test]
fn monogram_letters_read_on_both_ramps() {
    // Regression: the light ramp painted white letters (≈ 1.7 : 1). The
    // letter authority must hold the §12/§7.2 bar on every tile of both
    // ramps — computed from the rendered tile and letter colors.
    for hue_index in 0..6 {
        for (mode, bg) in [
            (ThemeMode::Light, FaviconSlate::light(hue_index)),
            (ThemeMode::Dark, FaviconSlate::dark(hue_index)),
        ] {
            let ratio = super::contrast(FaviconSlate::letter(mode), bg);
            assert!(
                ratio >= 3.0,
                "{mode:?} monogram tile {hue_index} letter ≥ 3:1, got {ratio:.2}"
            );
        }
    }
}

#[test]
fn increased_contrast_deepens_the_row_washes_in_both_modes() {
    // Regression: the light increased-contrast palette kept the standard
    // 4.5 % / 8 % washes although §3.4/§10.2 deepen them (7 % / 10 %).
    let pairs = [
        (
            Palette::light_standard(),
            Palette::light_increased(),
            SURFACE_SIDEBAR_LIGHT,
        ),
        (
            Palette::dark_standard(),
            Palette::dark_increased(),
            SURFACE_SIDEBAR_DARK,
        ),
    ];
    for (standard, increased, rail) in pairs {
        let rail = rail.rgba();
        for (name, base, deep) in [
            ("hover", standard.wash_hover, increased.wash_hover),
            ("active", standard.wash_active, increased.wash_active),
        ] {
            assert!(
                super::contrast(deep, rail) > super::contrast(base, rail),
                "increased contrast deepens the {name} wash"
            );
        }
    }
}
