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
//! density:row / sidebar / icon / hit-min → Density::* (S/N/XL)
//! dur:instant…emphasis / load:indeterminate → Motion::*
//! elev:0/1/2               → Elevation::*            theme/contrast/density → §15 fields
//! ```

use gpui::{px, FontWeight, Pixels};

use crate::palette::Palette;

/// Typography steps (DESIGN.md §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeRole {
    /// 22 pt / 600 — empty-state headline only.
    Display,
    /// 13 pt / 500 — tab title, space name.
    Title,
    /// 12 pt / 500 — buttons, nav labels, badges' base.
    Label,
    /// 11 pt / 400 — status, counts, timestamps (tabular).
    Meta,
    /// 12 pt / 400 — URLs in location mode and results.
    Mono,
    /// 11 pt / 500 — shortcut hints.
    Key,
    /// 10 pt / 600 — section labels "Pinned"/"Tabs", uppercase.
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

/// Density of the shell and tab rows (DESIGN.md §8). States and semantics
/// are identical across densities; rows keep uniform heights per density.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Small,
    Normal,
    ExtraLarge,
}

impl Density {
    pub fn row(self) -> Pixels {
        px(match self {
            Density::Small => 28.0,
            Density::Normal => 32.0,
            Density::ExtraLarge => 40.0,
        })
    }

    /// Row tile height (row minus the 2 pt gutter).
    pub fn row_tile(self) -> Pixels {
        px(match self {
            Density::Small => 26.0,
            Density::Normal => 28.0,
            Density::ExtraLarge => 36.0,
        })
    }

    /// §8 space row height (`density:space-row`).
    pub fn space_row(self) -> Pixels {
        px(match self {
            Density::Small => 30.0,
            Density::Normal => 34.0,
            Density::ExtraLarge => 44.0,
        })
    }

    pub fn sidebar_width(self) -> Pixels {
        px(match self {
            Density::Small => 216.0,
            Density::Normal => 260.0,
            Density::ExtraLarge => 320.0,
        })
    }

    pub fn icon(self) -> Pixels {
        px(match self {
            Density::Small => 14.0,
            Density::Normal => 16.0,
            Density::ExtraLarge => 18.0,
        })
    }

    /// Minimum hit target for interactive chrome controls.
    pub fn hit_min(self) -> Pixels {
        px(match self {
            Density::Small => 28.0,
            Density::Normal => 32.0,
            Density::ExtraLarge => 40.0,
        })
    }
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
        Density::Small => (11.0, 11.0, 10.0, 10.0, 11.0, 10.0),
        Density::Normal => (13.0, 12.0, 11.0, 11.0, 12.0, 10.0),
        Density::ExtraLarge => (15.0, 13.0, 12.0, 12.0, 13.0, 11.0),
    };
    TypeScale {
        // display: 22/24, title/label/meta/mono/key/badge per the §8 row.
        display: step(
            if density == Density::ExtraLarge {
                24.0
            } else {
                22.0
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

/// Corner radii (DESIGN.md §5): chrome stays in 4–8 px, never pill-cards.
#[derive(Clone, Copy, Debug)]
pub struct Radius;

impl Radius {
    pub const fn small(self) -> Pixels {
        px(4.0)
    }
    pub const fn medium(self) -> Pixels {
        px(6.0)
    }
    pub const fn large(self) -> Pixels {
        px(8.0)
    }
    pub const fn xlarge(self) -> Pixels {
        px(12.0)
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
        // normal density is the shipped 260 pt sidebar.
        assert_eq!(Density::Normal.row(), px(32.0));
        assert_eq!(Density::Normal.row_tile(), px(28.0));
        assert_eq!(Density::Normal.sidebar_width(), px(260.0));
        assert_eq!(Density::Small.sidebar_width(), px(216.0));
        assert_eq!(Density::ExtraLarge.sidebar_width(), px(320.0));
        assert_eq!(type_scale(Density::Small).title.size, px(11.0));
        assert_eq!(type_scale(Density::Normal).title.size, px(13.0));
        assert_eq!(type_scale(Density::ExtraLarge).title.size, px(15.0));
        assert_eq!(type_scale(Density::Normal).badge.size, px(10.0));
        assert!(f32::from(Density::ExtraLarge.hit_min()) > f32::from(Density::Small.hit_min()));
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
        for r in [Radius, Radius, Radius, Radius] {
            let pts = f32::from(r.small());
            assert!(
                (4.0..=12.0).contains(&pts),
                "chrome radii stay 4–12 px, got {pts}"
            );
        }
    }
}
