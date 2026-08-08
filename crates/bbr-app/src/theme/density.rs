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
    /// §8 `density:row` — uniform row height per density (S 32 / N 36 /
    /// XL 44), so scroll math stays deterministic.
    pub fn row(self) -> Pixels {
        px(match self {
            Density::Small => 32.0,
            Density::Normal => 36.0,
            Density::ExtraLarge => 44.0,
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

    /// §8/§5 `win:sidebar` (S 232 / N 280 / XL 340 pt).
    pub fn sidebar_width(self) -> Pixels {
        px(match self {
            Density::Small => 232.0,
            Density::Normal => 280.0,
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
    /// surface from the rail edges (S `sp:1` 4 / N `sp:2` 8 / XL `sp:3` 12).
    pub fn row_inset(self) -> Pixels {
        px(match self {
            Density::Small => 4.0,
            Density::Normal => 8.0,
            Density::ExtraLarge => 12.0,
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

    /// §11.10 favorites grid gap (S `sp:1` 4 / N `sp:2` 8 / XL `sp:2` 8).
    pub fn favorites_gap(self) -> Pixels {
        px(match self {
            Density::Small => 4.0,
            Density::Normal => 8.0,
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
