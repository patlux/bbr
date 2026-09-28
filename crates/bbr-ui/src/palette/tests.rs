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
/// Asserts the documented pairs on the untinted rail; the composited v2
/// surfaces are gated per space tint below.
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

/// The four shell palettes with their mode/contrast, for the composited
/// v2 gates.
fn palettes() -> [(ThemeMode, crate::theme::ContrastMode, Palette); 4] {
    use crate::theme::ContrastMode::{Increased, Standard};
    [
        (ThemeMode::Light, Standard, Palette::light_standard()),
        (ThemeMode::Light, Increased, Palette::light_increased()),
        (ThemeMode::Dark, Standard, Palette::dark_standard()),
        (ThemeMode::Dark, Increased, Palette::dark_increased()),
    ]
}

/// The untinted rail plus the rail under every curated space tint and
/// intensity step — the
/// surfaces a §18.2 veil can actually be composited over.
fn rails(mode: ThemeMode, contrast: crate::theme::ContrastMode) -> Vec<(String, Rgba)> {
    let theme = Theme::new(mode, contrast, crate::theme::Density::Normal);
    let mut rails = vec![("untinted".to_string(), theme.palette.surface_sidebar)];
    // v2 D7: every curated intensity step is a rail the veils sit on.
    for token in bbr_domain::SpaceTheme::ALL {
        for step in bbr_domain::TintIntensity::ALL {
            rails.push((
                format!("{token:?}/{step:?}"),
                crate::theme::space_tint(token, step, &theme).chrome,
            ));
        }
    }
    rails
}

/// CIE L* of a rendered colour (for "visibly above the rail" checks).
fn lightness(color: Rgba) -> f64 {
    let y = contrast(color, gpui::rgb(0x000000)) * 0.05 - 0.05;
    if y > 0.008856 {
        116.0 * y.cbrt() - 16.0
    } else {
        903.3 * y
    }
}

/// D1 (§18.1/§18.2): the selected row lifts *above* the rail and its text
/// holds `ink ≥ 10.5:1` (IC ≥ 11:1) and `ink.secondary ≥ 4.5:1` (IC ≥ 7:1)
/// on the composited selection over the untinted and every tinted rail.
#[test]
fn selection_lifts_above_every_rail_and_keeps_the_d1_text_bars() {
    use crate::theme::ContrastMode;
    for (mode, contrast_mode, p) in palettes() {
        let (ink_min, secondary_min) = match contrast_mode {
            ContrastMode::Standard => (10.5, 4.5),
            ContrastMode::Increased => (11.0, 7.0),
        };
        for (name, rail) in rails(mode, contrast_mode) {
            let selected = over(p.tint_selected, rail);
            let ink = contrast(p.ink, selected);
            let secondary = contrast(p.ink_secondary, selected);
            assert!(
                ink >= ink_min,
                "{mode:?} {contrast_mode:?} {name}: selected ink {ink:.2} ≥ {ink_min}"
            );
            assert!(
                secondary >= secondary_min,
                "{mode:?} {contrast_mode:?} {name}: selected meta {secondary:.2} ≥ {secondary_min}"
            );
            let lift = lightness(selected) - lightness(rail);
            assert!(
                lift >= 4.0,
                "{mode:?} {contrast_mode:?} {name}: selection sits {lift:.2} L* above the rail"
            );
        }
    }
    // Dark IC replaces the veil with an opaque, stronger step.
    let (standard, increased) = (Palette::dark_standard(), Palette::dark_increased());
    assert!(standard.tint_selected.a < 1.0, "dark standard is a veil");
    assert_eq!(increased.tint_selected.a, 1.0);
    let rail = standard.surface_sidebar;
    assert!(
        lightness(increased.tint_selected) > lightness(over(standard.tint_selected, rail)),
        "increased contrast strengthens the dark selection"
    );
}

/// §11.4/§18.2: footer glyphs render at `ink.secondary` on the footer veil
/// over the rail; the §3.1 bar (≥ 5.9:1, IC ≥ 9:1) holds on every rail.
#[test]
fn footer_veil_keeps_the_secondary_bar_on_every_rail() {
    use crate::theme::ContrastMode;
    for (mode, contrast_mode, p) in palettes() {
        let min = match contrast_mode {
            ContrastMode::Standard => 5.9,
            ContrastMode::Increased => 9.0,
        };
        assert!(p.surface_footer.a < 1.0, "the footer is a veil");
        assert!(p.surface_favorite.a < 1.0, "the Favorite tile is a veil");
        for (name, rail) in rails(mode, contrast_mode) {
            let footer = over(p.surface_footer, rail);
            let ratio = contrast(p.ink_secondary, footer);
            assert!(
                ratio >= min,
                "{mode:?} {contrast_mode:?} {name}: footer glyphs {ratio:.2} ≥ {min}"
            );
        }
    }
}

/// §18.2 `tint.accent`: `ink ≥ 9:1` and `accent.ink ≥ 4.5:1` in every mode,
/// and the surface stays visible on the overlay/field it marks.
#[test]
fn tint_accent_keeps_its_text_bars_and_reads_on_overlays() {
    for (mode, contrast_mode, p) in palettes() {
        assert_eq!(p.tint_accent.a, 1.0, "tint.accent is opaque");
        let ink = contrast(p.ink, p.tint_accent);
        let accent_ink = contrast(p.accent_ink, p.tint_accent);
        assert!(ink >= 9.0, "{mode:?} {contrast_mode:?} ink {ink:.2} ≥ 9");
        assert!(
            accent_ink >= 4.5,
            "{mode:?} {contrast_mode:?} accent.ink {accent_ink:.2} ≥ 4.5"
        );
        for surface in [p.surface_overlay, p.surface_field] {
            let delta = (lightness(p.tint_accent) - lightness(surface)).abs();
            assert!(
                delta >= 4.0,
                "{mode:?} {contrast_mode:?} tint.accent is {delta:.2} L* off its surface"
            );
        }
    }
}

/// §11.5 bordered address field: a white veil over the (tinted) rail —
/// one step lighter than the rail in both modes, so it follows the space
/// tint — with a hairline-strength border. The hover steps darker on light
/// chrome and brighter on dark chrome; domain ink keeps the §3.1 bar on the
/// composited field in every palette.
#[test]
fn address_field_is_a_bordered_veil_one_step_off_the_rail() {
    for (mode, contrast_mode, p) in palettes() {
        let (fill, hover) = (p.address_fill, p.address_fill_hover);
        for veil in [fill, hover] {
            assert!(veil.a > 0.0 && veil.a < 1.0, "a veil, never an opaque hex");
            assert!(veil.r >= 0.99 && veil.g >= 0.99 && veil.b >= 0.99);
            for (name, rail) in rails(mode, contrast_mode) {
                let field = over(veil, rail);
                assert!(
                    contrast(p.ink, field) >= 4.5,
                    "{mode:?} {contrast_mode:?} {name}: domain ink on the field"
                );
            }
        }
        match mode {
            ThemeMode::Light => assert!(hover.a < fill.a, "light hover darkens"),
            ThemeMode::Dark => assert!(hover.a > fill.a, "dark hover brightens"),
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
    // §3.4/§10.2/§18.2: the washes are translucent veils (ink on light,
    // white on dark) and increased contrast deepens them (8 % / 12 %). The
    // comparison composites each wash over the rail it paints on.
    let pairs = [
        (Palette::light_standard(), Palette::light_increased()),
        (Palette::dark_standard(), Palette::dark_increased()),
    ];
    for (standard, increased) in pairs {
        let rail = standard.surface_sidebar;
        for (name, base, deep) in [
            ("hover", standard.wash_hover, increased.wash_hover),
            ("active", standard.wash_active, increased.wash_active),
        ] {
            assert!(base.a < 1.0 && deep.a < 1.0, "{name} wash is a veil");
            assert!(
                super::contrast(over(deep, rail), rail) > super::contrast(over(base, rail), rail),
                "increased contrast deepens the {name} wash"
            );
        }
        assert!(
            standard.wash_active.a > standard.wash_hover.a,
            "pressed is stronger than hover"
        );
    }
}
