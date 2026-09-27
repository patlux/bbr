//! The shell density enum and its geometry tokens (DESIGN.md §8).
//!
//! Three fixed densities are chosen by the shell from tab-collection size;
//! geometry is per-density, **states and semantics are identical across
//! densities** (§8). Rows keep uniform heights per density so scroll math
//! stays deterministic. Every accessor below is the §8/§11/§12 token —
//! production size literals must derive from these, never be reintroduced.

use gpui::{px, Pixels};

/// Density of the shell and tab rows (DESIGN.md §8). States and semantics
/// are identical across densities; rows keep uniform heights per density.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Small,
    Normal,
    ExtraLarge,
}

impl Density {
    /// §8 `density:row` — uniform row surface height per density (S 32 /
    /// N 36 / XL 44), so scroll math stays deterministic.
    pub fn row(self) -> Pixels {
        px(match self {
            Density::Small => 32.0,
            Density::Normal => 36.0,
            Density::ExtraLarge => 44.0,
        })
    }

    /// §8 `density:gap` — the reference-calibrated inter-row gap between
    /// row surfaces. Normal density ships the reference's 4 pt gap; the
    /// S/XL families keep the same 4 pt gap so the pitch follows the §8
    /// row scale proportionally (S 36 / N 40 / XL 48). The gap is the
    /// interstitial rhythm rows step on; hit minima and row surfaces are
    /// unchanged (§8 constraints).
    pub fn gap(self) -> Pixels {
        px(match self {
            Density::Small | Density::Normal | Density::ExtraLarge => 4.0,
        })
    }

    /// §8 `density:pitch` — the row step = surface + gap (S 36 / N 40 /
    /// XL 48): the layout pitch every regular/pinned/folder/nested/New Tab
    /// row steps on, and the exact step the dense scroll math, scroll
    /// offsets, drag-target geometry and the testkit snapshot use.
    /// [`Density::row`] stays the painted row surface; [`Density::pitch`]
    /// is the row pitch. The dense bounded-window rendering keeps its
    /// exact math under the new constant pitch without weakening any
    /// bound (§11.6).
    pub fn pitch(self) -> Pixels {
        px(match self {
            Density::Small => 36.0,
            Density::Normal => 40.0,
            Density::ExtraLarge => 48.0,
        })
    }

    /// §8 `density:row-tile` (S 28 / N 36 / XL 44). Equal to the row at
    /// N/XL; at S the 4 pt difference is the interstitial gutter that
    /// lets icon-only pinned rows breathe.
    pub fn row_tile(self) -> Pixels {
        px(match self {
            Density::Small => 28.0,
            Density::Normal => 36.0,
            Density::ExtraLarge => 44.0,
        })
    }

    /// §8/§5 `win:sidebar` (S 232 / N 300 / XL 340 pt).
    pub fn sidebar_width(self) -> Pixels {
        px(match self {
            Density::Small => 232.0,
            Density::Normal => 300.0,
            Density::ExtraLarge => 340.0,
        })
    }

    /// `win:palette` (DESIGN.md §11.4): the centered Cmd-T palette width
    /// (S 440 / N 520 / XL 620 pt) and the field-anchored overlay's upper
    /// clamp.
    pub fn palette_width(self) -> Pixels {
        px(match self {
            Density::Small => 440.0,
            Density::Normal => 520.0,
            Density::ExtraLarge => 620.0,
        })
    }

    /// §7/§8 `icon:` — row glyphs take the owning density's size
    /// (S 16 / N 18 / XL 20).
    pub fn icon(self) -> Pixels {
        px(match self {
            Density::Small => 16.0,
            Density::Normal => 18.0,
            Density::ExtraLarge => 20.0,
        })
    }

    /// §8 `density:hit-min` — minimum hit target for interactive chrome
    /// controls (S 28 / N 32 / XL 40).
    pub fn hit_min(self) -> Pixels {
        px(match self {
            Density::Small => 28.0,
            Density::Normal => 32.0,
            Density::ExtraLarge => 40.0,
        })
    }

    /// §8 `density:hit-min.close` — the close-control hit square
    /// (S 24 / N 28 / XL 32); the §7 close glyph renders at
    /// [`Density::icon`] optically centered in it.
    pub fn hit_min_close(self) -> Pixels {
        px(match self {
            Density::Small => 24.0,
            Density::Normal => 28.0,
            Density::ExtraLarge => 32.0,
        })
    }

    /// §8 `density:row-inset` — the inset of the §10.2–§10.4 rounded row
    /// surface from the rail edges (S `sp:1` 4 / N `sp:1` 4 / XL `sp:2` 8).
    pub fn row_inset(self) -> Pixels {
        px(match self {
            Density::Small => 4.0,
            Density::Normal => 4.0,
            Density::ExtraLarge => 8.0,
        })
    }

    /// §8 `density:section-gap` — the breathing between sidebar regions
    /// (S `sp:3` 12 / N `sp:4` 16 / XL `sp:5` 20).
    pub fn section_gap(self) -> Pixels {
        px(match self {
            Density::Small => 12.0,
            Density::Normal => 16.0,
            Density::ExtraLarge => 20.0,
        })
    }

    /// Reference-calibrated Favorite tile height (36 / 44 / 52 pt).
    pub fn favorite_tile_height(self) -> Pixels {
        px(match self {
            Density::Small => 36.0,
            Density::Normal => 44.0,
            Density::ExtraLarge => 52.0,
        })
    }

    /// Favorite-grid gap. The normal four-tile band uses a 4 pt gap.
    pub fn favorites_gap(self) -> Pixels {
        px(match self {
            Density::Small => 4.0,
            Density::Normal => 4.0,
            Density::ExtraLarge => 8.0,
        })
    }

    /// §11.1 row state slot — the fixed trailing slot (S 8 / N 10 / XL 10).
    pub fn state_slot(self) -> Pixels {
        px(match self {
            Density::Small => 8.0,
            Density::Normal => 10.0,
            Density::ExtraLarge => 10.0,
        })
    }

    /// §11.1 row padding-x (S `sp:2` 8 / N `sp:3` 12 / XL `sp:4` 16).
    pub fn row_padding_x(self) -> Pixels {
        px(match self {
            Density::Small => 8.0,
            Density::Normal => 12.0,
            Density::ExtraLarge => 16.0,
        })
    }

    /// §12 profile chip square (S 20 / N 24 / XL 28).
    pub fn profile_chip(self) -> Pixels {
        px(match self {
            Density::Small => 20.0,
            Density::Normal => 24.0,
            Density::ExtraLarge => 28.0,
        })
    }

    /// §8 `density:radius` — the inset row surface's corner radius
    /// (S `radius:sm` 6 / N `radius:md` 8 / XL `radius:lg` 10).
    pub fn density_radius(self) -> Pixels {
        px(match self {
            Density::Small => 6.0,
            Density::Normal => 8.0,
            Density::ExtraLarge => 10.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Radius;
    use gpui::px;

    #[test]
    fn density_scale_tokens_match_the_settled_table() {
        // §8/§11/§12: the S1 scale's new geometry tokens, exact per density.
        assert_eq!(Density::Small.hit_min_close(), px(24.0));
        assert_eq!(Density::Normal.hit_min_close(), px(28.0));
        assert_eq!(Density::ExtraLarge.hit_min_close(), px(32.0));
        assert_eq!(Density::Small.row_inset(), px(4.0));
        assert_eq!(Density::Normal.row_inset(), px(4.0));
        assert_eq!(Density::ExtraLarge.row_inset(), px(8.0));
        assert_eq!(Density::Small.section_gap(), px(12.0));
        assert_eq!(Density::Normal.section_gap(), px(16.0));
        assert_eq!(Density::ExtraLarge.section_gap(), px(20.0));
        assert_eq!(Density::Small.favorites_gap(), px(4.0));
        assert_eq!(Density::Normal.favorites_gap(), px(4.0));
        assert_eq!(Density::ExtraLarge.favorites_gap(), px(8.0));
        assert_eq!(Density::Normal.favorite_tile_height(), px(44.0));
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
    }

    #[test]
    fn pitch_is_surface_plus_gap_for_every_density() {
        // §8 polish: `density:pitch` = `density:row` + `density:gap` for
        // every density — the normal rail ships the reference's ~40 pt
        // pitch (36 pt surfaces separated by 4 pt gaps); S/XL keep the
        // same 4 pt gap so the pitch follows the §8 row scale
        // proportionally (S 36 / N 40 / XL 48). The row surface and the
        // hit minima never change; only the inter-row pitch does.
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            assert_eq!(
                density.pitch(),
                px(f32::from(density.row()) + f32::from(density.gap())),
                "{density:?} pitch = surface + gap"
            );
        }
        assert_eq!(Density::Normal.gap(), px(4.0));
        assert_eq!(Density::Small.gap(), px(4.0));
        assert_eq!(Density::ExtraLarge.gap(), px(4.0));
        assert_eq!(Density::Small.pitch(), px(36.0));
        assert_eq!(Density::Normal.pitch(), px(40.0));
        assert_eq!(Density::ExtraLarge.pitch(), px(48.0));
        assert_eq!(Density::Normal.row(), px(36.0), "surface stays 36 pt");
        assert_eq!(Density::Small.row(), px(32.0));
        assert_eq!(Density::ExtraLarge.row(), px(44.0));
    }
}
