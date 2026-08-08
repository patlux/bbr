//! BBR Chassis — the typed GPUI design-token system (DESIGN.md §2–§8).
//!
//! Color tokens and their WCAG contrast gate live in [`crate::palette`]; this
//! module owns the shell modes (dark/light, standard/increased contrast,
//! density), the typography steps, spacing, radius, motion, and elevation
//! scales. The placeholder `render.rs` magic `rgb(0x…)` values are replaced
//! by these tokens in the same groups that introduce the shell; tests and E2E
//! assert the §15 mode fields and the §3.1 contrast contract **derived from
//! the tokens** — never copied as literal ratios.
//!
//! Token names intentionally mirror `DESIGN.md`:
//!
//! ```text
//! type:title/label/meta/mono/key/badge/display → TypeRole::TITLE/…
//! sp:N (4·N pt)            → Spacing::spN            radius:sm/md/lg/xl/full → Radius::*
//! density:row / row-tile / space-chip / sidebar / icon / hit-min /
//! hit-min.close / row-inset / section-gap / favorites-gap / state-slot /
//! row-padding-x / profile-chip / density-radius → Density::* (S/N/XL)
//! dur:instant…emphasis / load:indeterminate → Motion::*
//! elev:0/1/2               → Elevation::*            theme/contrast/density → §15 fields
//! ```

use gpui::{px, FontWeight, Pixels};

use crate::palette::Palette;

/// The §8 density enum and its geometry tokens (moved into a submodule to
/// keep this file under the ~500-line budget; re-exported unchanged so
/// `crate::theme::Density` stays the single naming authority).
mod density;
pub use density::Density;

/// Typography steps (DESIGN.md §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeRole {
    /// 24 pt / 600 — empty-state headline only (26 at XL).
    Display,
    /// 14 pt / 500 — tab title, space name.
    Title,
    /// 13 pt / 500 — buttons, nav labels, badges' base.
    Label,
    /// 12 pt / 400 — status, counts, timestamps (tabular).
    Meta,
    /// 13 pt / 400 — URLs in location mode and results.
    Mono,
    /// 12 pt / 500 — shortcut hints.
    Key,
    /// 11 pt / 600 — section labels "Pinned"/"Tabs", uppercase.
    Badge,
}

/// One typographic step with the density-adjusted size (DESIGN.md §4, §8).
#[derive(Clone, Copy, Debug)]
pub struct TypeStep {
    pub size: Pixels,
    pub weight: FontWeight,
    /// Line height as a leading multiplier.
    pub leading: f32,
}

/// The active theme: palette, mode fields for E2E snapshots (§15), and the
/// typography/spacing/radius/density/motion/elevation constants.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    /// §15 mode field: light or dark.
    pub mode: ThemeMode,
    /// §15 mode field: standard or increased contrast.
    pub contrast: ContrastMode,
    /// §15 mode field: row/sidebar density for the tab collection.
    pub density: Density,
    pub palette: Palette,
    pub type_: TypeScale,
    pub spacing: Spacing,
    pub radius: Radius,
    pub motion: Motion,
    pub elevation: Elevation,
}

/// §15 `theme` field on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// §15 `contrast` field on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastMode {
    Standard,
    Increased,
}

/// Typography steps resolved for a density (DESIGN.md §4 table).
#[derive(Clone, Copy, Debug)]
pub struct TypeScale {
    pub display: TypeStep,
    pub title: TypeStep,
    pub label: TypeStep,
    pub meta: TypeStep,
    pub mono: TypeStep,
    pub key: TypeStep,
    pub badge: TypeStep,
}

const fn step(size: f32, weight: f32, leading: f32) -> TypeStep {
    TypeStep {
        size: px(size),
        weight: FontWeight(weight),
        leading,
    }
}

fn type_scale(density: Density) -> TypeScale {
    let s = match density {
        Density::Small => (12.0, 12.0, 11.0, 12.0, 11.0, 11.0),
        Density::Normal => (14.0, 13.0, 12.0, 13.0, 12.0, 11.0),
        Density::ExtraLarge => (16.0, 14.0, 13.0, 14.0, 13.0, 12.0),
    };
    TypeScale {
        // display: 24/24/26, title/label/meta/mono/key/badge per the §8 row.
        display: step(
            match density {
                Density::Small => 24.0,
                Density::Normal => 24.0,
                Density::ExtraLarge => 26.0,
            },
            600.0,
            1.2,
        ),
        title: step(s.0, 500.0, 1.2),
        label: step(s.1, 500.0, 1.2),
        meta: step(s.2, 400.0, 1.2),
        mono: step(s.3, 400.0, 1.2),
        key: step(s.4, 500.0, 1.2),
        badge: step(s.5, 600.0, 1.2),
    }
}

/// 4 pt grid spacing scale (DESIGN.md §5): `sp(N)` = N·4 pt.
#[derive(Clone, Copy, Debug)]
pub struct Spacing;

impl Spacing {
    pub const fn sp1() -> Pixels {
        px(4.0)
    }
    pub const fn sp2() -> Pixels {
        px(8.0)
    }
    pub const fn sp3() -> Pixels {
        px(12.0)
    }
    pub const fn sp4() -> Pixels {
        px(16.0)
    }
    pub const fn sp5() -> Pixels {
        px(20.0)
    }
    pub const fn sp6() -> Pixels {
        px(24.0)
    }
    pub const fn sp7() -> Pixels {
        px(32.0)
    }
    pub const fn sp8() -> Pixels {
        px(40.0)
    }
}

/// Corner radii (DESIGN.md §5): chrome stays in 6–10 px, never pill-cards;
/// `radius:xl` (14) is reserved for the XL palette sheet.
#[derive(Clone, Copy, Debug)]
pub struct Radius;

impl Radius {
    /// §5 `radius:sm` 6 — small tile, profile chip, S-density rows.
    pub const fn small(self) -> Pixels {
        px(6.0)
    }
    /// §5 `radius:md` 8 — N/XL row tiles, buttons.
    pub const fn medium(self) -> Pixels {
        px(8.0)
    }
    /// §5 `radius:lg` 10 — command field, N palette, crash panel.
    pub const fn large(self) -> Pixels {
        px(10.0)
    }
    /// §5 `radius:xl` 14 — reserved for the XL palette sheet.
    pub const fn xlarge(self) -> Pixels {
        px(14.0)
    }
    pub const fn full(self) -> Pixels {
        px(9999.0)
    }
}

/// Motion durations (DESIGN.md §6). State changes are `instant`; the rest
/// gate transient indication only.
#[derive(Clone, Copy, Debug)]
pub struct Motion;

impl Motion {
    pub const fn instant() -> std::time::Duration {
        std::time::Duration::ZERO
    }
    pub const fn micro() -> std::time::Duration {
        std::time::Duration::from_millis(70)
    }
    pub const fn fine() -> std::time::Duration {
        std::time::Duration::from_millis(120)
    }
    pub const fn standard() -> std::time::Duration {
        std::time::Duration::from_millis(180)
    }
    pub const fn emphasis() -> std::time::Duration {
        std::time::Duration::from_millis(260)
    }
    /// Indeterminate loading loop (DESIGN.md §10.5).
    pub const fn load_loop() -> std::time::Duration {
        std::time::Duration::from_millis(580)
    }
}

/// Elevation (DESIGN.md §2). `elev:1` is a hairline border on the same
/// material; rows never shadow (list paint budget).
#[derive(Clone, Copy, Debug)]
pub struct Elevation;

impl Elevation {
    pub const fn base() -> u8 {
        0
    }
    pub const fn raised() -> u8 {
        1
    }
    pub const fn floating() -> u8 {
        2
    }
}

impl Theme {
    /// The shipped shell mode: dark, standard contrast, normal density.
    pub fn dark() -> Self {
        Self::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    /// Automatic density from the regular-tab collection size (DESIGN.md §8:
    /// "Density is a product of the machine, not a slider" — the shell
    /// crosses to Small at 200 regular tabs and ExtraLarge at 600). Row
    /// heights are uniform per density, so a density change never happens
    /// mid-scroll; the collection crossing a threshold at seed time flips the
    /// geometry once for the whole session view.
    pub fn for_collection(mode: ThemeMode, contrast: ContrastMode, regular_tabs: usize) -> Self {
        let density = if regular_tabs >= 600 {
            Density::ExtraLarge
        } else if regular_tabs >= 200 {
            Density::Small
        } else {
            Density::Normal
        };
        Self::new(mode, contrast, density)
    }

    pub fn new(mode: ThemeMode, contrast: ContrastMode, density: Density) -> Self {
        let palette = match (mode, contrast) {
            (ThemeMode::Dark, ContrastMode::Standard) => Palette::dark_standard(),
            (ThemeMode::Dark, ContrastMode::Increased) => Palette::dark_increased(),
            (ThemeMode::Light, ContrastMode::Standard) => Palette::light_standard(),
            (ThemeMode::Light, ContrastMode::Increased) => Palette::light_increased(),
        };
        Theme {
            mode,
            contrast,
            density,
            palette,
            type_: type_scale(density),
            spacing: Spacing,
            radius: Radius,
            motion: Motion,
            elevation: Elevation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_geometry_is_uniform_per_density() {
        // §8: uniform row heights per density keep scroll math deterministic;
        // normal density is the shipped 280 pt sidebar.
        assert_eq!(Density::Normal.row(), px(36.0));
        assert_eq!(Density::Small.row(), px(32.0));
        assert_eq!(Density::ExtraLarge.row(), px(44.0));
        assert_eq!(Density::Small.row_tile(), px(28.0));
        assert_eq!(Density::Normal.row_tile(), px(36.0));
        assert_eq!(Density::ExtraLarge.row_tile(), px(44.0));
        assert_eq!(Density::Normal.sidebar_width(), px(280.0));
        assert_eq!(Density::Small.sidebar_width(), px(232.0));
        assert_eq!(Density::ExtraLarge.sidebar_width(), px(340.0));
        // §11.4 `win:palette`: S 440 / N 520 / XL 620 pt.
        assert_eq!(Density::Small.palette_width(), px(440.0));
        assert_eq!(Density::Normal.palette_width(), px(520.0));
        assert_eq!(Density::ExtraLarge.palette_width(), px(620.0));
        assert_eq!(type_scale(Density::Small).title.size, px(12.0));
        assert_eq!(type_scale(Density::Normal).title.size, px(14.0));
        assert_eq!(type_scale(Density::ExtraLarge).title.size, px(16.0));
        assert_eq!(type_scale(Density::Normal).badge.size, px(11.0));
        assert!(f32::from(Density::ExtraLarge.hit_min()) > f32::from(Density::Small.hit_min()));
    }

    #[test]
    fn density_scale_tokens_match_the_settled_table() {
        // §8/§11/§12: the S1 scale's new geometry tokens, exact per density.
        assert_eq!(Density::Small.hit_min_close(), px(24.0));
        assert_eq!(Density::Normal.hit_min_close(), px(28.0));
        assert_eq!(Density::ExtraLarge.hit_min_close(), px(32.0));
        assert_eq!(Density::Small.row_inset(), px(4.0));
        assert_eq!(Density::Normal.row_inset(), px(8.0));
        assert_eq!(Density::ExtraLarge.row_inset(), px(12.0));
        assert_eq!(Density::Small.section_gap(), px(12.0));
        assert_eq!(Density::Normal.section_gap(), px(16.0));
        assert_eq!(Density::ExtraLarge.section_gap(), px(20.0));
        assert_eq!(Density::Small.favorites_gap(), px(4.0));
        assert_eq!(Density::Normal.favorites_gap(), px(8.0));
        assert_eq!(Density::ExtraLarge.favorites_gap(), px(8.0));
        assert_eq!(Density::Small.state_slot(), px(8.0));
        assert_eq!(Density::Normal.state_slot(), px(10.0));
        assert_eq!(Density::ExtraLarge.state_slot(), px(10.0));
        assert_eq!(Density::Small.row_padding_x(), px(8.0));
        assert_eq!(Density::Normal.row_padding_x(), px(12.0));
        assert_eq!(Density::ExtraLarge.row_padding_x(), px(16.0));
        assert_eq!(Density::Small.profile_chip(), px(20.0));
        assert_eq!(Density::Normal.profile_chip(), px(24.0));
        assert_eq!(Density::ExtraLarge.profile_chip(), px(28.0));
        assert_eq!(Density::Small.density_radius(), Radius.small());
        assert_eq!(Density::Normal.density_radius(), Radius.medium());
        assert_eq!(Density::ExtraLarge.density_radius(), Radius.large());
        // §8: `density:space-chip` and `density:row` share one surface family.
        assert_eq!(Density::Small.space_row(), Density::Small.row());
        assert_eq!(Density::Normal.space_row(), Density::Normal.row());
        assert_eq!(Density::ExtraLarge.space_row(), Density::ExtraLarge.row());
    }

    #[test]
    fn spacing_and_radius_are_4pt_multiples() {
        for sp in [
            Spacing::sp1(),
            Spacing::sp2(),
            Spacing::sp3(),
            Spacing::sp4(),
        ] {
            let pts = f32::from(sp);
            assert_eq!(
                pts,
                (pts / 4.0).round() * 4.0,
                "spacing stays on the 4 pt grid"
            );
        }
        for r in [
            Radius.small(),
            Radius.medium(),
            Radius.large(),
            Radius.xlarge(),
        ] {
            let pts = f32::from(r);
            assert!(
                (6.0..=14.0).contains(&pts),
                "chrome radii stay 6–14 px, got {pts}"
            );
        }
    }
}
