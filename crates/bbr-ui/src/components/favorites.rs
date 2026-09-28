//! DESIGN.md §11.10 Favorites grid: a `type:badge` "Favorites" section over
//! a grid of square destination tiles.
//!
//! The grid renders the §7.2 favicon/monogram tile centered in a
//! `radius:sm` tile, `density:favorites-gap` apart, in **four columns at
//! every density and width** ([`GRID_COLUMNS`]) so the tile → index mapping
//! is deterministic. The tile width derives from the rail
//! ([`favorite_tile_width`]) so the grid always spans the full content
//! width; the tile height stays fixed per density, keeping
//! [`favorites_region_height`] a pure function of density, count and the
//! drag offer. Favorites are destinations
//! (title + url identities from the L5 session-local grid), never tab
//! references — tab lifecycle states do not apply.
//!
//! This module owns the tile identity contract (ids, per-density grid
//! columns, the tile spec) and the grid anatomy. The snapshot→spec
//! derivation (`favorite_tiles`) stays an app-owned adapter that builds
//! [`FavoriteTileSpec`] from the controller read model; the rail
//! composition wires the per-tile activation listeners and the
//! empty-region hiding (the section label renders only while tiles exist).

use gpui::{
    div, img, prelude::*, App, Div, Image, ImageSource, MouseButton, MouseDownEvent, ObjectFit,
    SharedString, Stateful, Styled, Window,
};
use std::sync::Arc;

use crate::palette::FaviconSlate;
use crate::theme::{Radius, Spacing, Theme, ThemeMode};

/// The §15 id of the favorites grid container.
pub const FAVORITES_ID: &str = "bbr-favorites";
/// The §15 id of the "Favorites" section label (hidden when the grid is
/// empty — the §11.4 region rule).
#[allow(dead_code)] // §15 visible-ids seam; consumed by the app's snapshot
pub const FAVORITES_SECTION_ID: &str = "bbr-section-favorites";

/// The stable tile id `bbr-fav-<n>` (DESIGN.md §11.10).
pub fn favorite_tile_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-fav-{index}"))
}

/// The stable tile-icon id `bbr-fav-icon-<n>` (DESIGN.md §11.10).
pub fn favorite_tile_icon_id(index: usize) -> SharedString {
    SharedString::from(format!("bbr-fav-icon-{index}"))
}

/// DESIGN.md §11.10: the grid is four columns at every density and width.
pub const GRID_COLUMNS: usize = 4;

/// The constant column count (tile `n` always lands in the same cell).
pub fn grid_columns() -> usize {
    GRID_COLUMNS
}

/// The derived tile width for a rail of `rail_width` (points): the rail
/// column (minus the 4 pt resize handle) inside the region's `sp:2` inline
/// padding, split into [`GRID_COLUMNS`] tiles with the density gap. Widening
/// the rail widens the tiles; it never adds a column or leaves slack.
pub fn favorite_tile_width(theme: &Theme, rail_width: f32) -> f32 {
    let content = (rail_width
        - crate::sidebar::geometry::RESIZE_HANDLE_WIDTH
        - 2.0 * f32::from(Spacing::sp2()))
    .max(0.0);
    let gap = f32::from(theme.density.favorites_gap());
    let columns = GRID_COLUMNS as f32;
    ((content - (columns - 1.0) * gap) / columns).max(0.0)
}

/// One favorite destination tile's rendered identity: the §7.2 monogram
/// fallback derives deterministically from the hostname, so the tile
/// identity never needs the favicon pipeline.
///
/// The tile also carries the concrete `TabId + SpaceId + ProfileId` the L5
/// read model reports (DESIGN.md §11.10 data source): a favorite is an
/// additional presentation of one exact tab, and the semantic name includes
/// the owning space/profile when they differ from the current context. The
/// ids stay opaque `u64` (the raw tuple fields) so the app layer never
/// needs the domain crate nameable outside its dev-dependency scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteTileSpec {
    /// Position in deterministic insertion order — the `bbr-fav-<n>` id.
    pub index: usize,
    /// The exact concrete tab this tile additionally presents (`TabId.0`).
    pub tab: u64,
    /// Current owner of `tab`; changes if that same tab is moved
    /// (`SpaceId.0`).
    pub space: u64,
    /// Derived from `space`; identifies the isolated container to activate
    /// (`ProfileId.0`).
    pub profile: u64,
    pub title: String,
    pub url: String,
    /// §7.2 monogram letter (first hostname character, uppercase).
    pub letter: char,
    /// §7.2 FNV slate-hue ramp index (0..6).
    pub hue: usize,
    /// The resolved favicon image for the tile's host in the owning
    /// profile's bounded store, if acquired (task packet
    /// `real-favicon-pipeline.md`): the tile renders it instead of the
    /// monogram — the same record the tab row derives from, one
    /// acquisition, every presentation.
    pub image: Option<Arc<Image>>,
}

/// The favorites grid anatomy: a `flex_wrap` grid of pre-built tiles at the
/// §11.10 `density:favorites-gap`. The rail composition builds the
/// per-tile listeners (exactly like the tab/space surfaces build per-row
/// listeners) and calls this only while tiles exist so the empty region
/// hides its section label.
pub fn favorites_grid(theme: &Theme, children: Vec<gpui::AnyElement>) -> Stateful<Div> {
    div()
        .id(FAVORITES_ID)
        .debug_selector(|| FAVORITES_ID.to_string())
        .flex()
        .flex_wrap()
        .gap(theme.density.favorites_gap())
        .px(Spacing::sp2())
        .children(children)
}

/// One §11.10 destination tile: the §7.2 monogram tile centered in a
/// `radius:sm` square at `density:row-tile` size, `density:favorites-gap`
/// apart. The tile id and the inner icon id are the stable §15 identity
/// pair. `on_activate` is the production activation listener
/// (open-vs-focus semantics is an open decision). `pop` carries the §18.5
/// favorite-pop moment key while this tile's favorite was just committed.
pub fn favorite_tile(
    theme: &Theme,
    tile: &FavoriteTileSpec,
    width: f32,
    pop: Option<u64>,
    on_activate: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let width = gpui::px(width);
    let height = theme.density.favorite_tile_height();
    let slate = match theme.mode {
        ThemeMode::Dark => FaviconSlate::dark(tile.hue),
        ThemeMode::Light => FaviconSlate::light(tile.hue),
    };
    let size = theme.density.icon();
    let radius = Radius.small();
    let badge_size = theme.type_.badge.size;
    let badge_weight = theme.type_.badge.weight;
    let glass = FaviconSlate::letter(theme.mode);
    let letter = tile.letter;
    let monogram = div()
        .id(favorite_tile_icon_id(tile.index))
        .debug_selector(move || favorite_tile_icon_id(tile.index).to_string())
        .size(size)
        .rounded(radius)
        .bg(slate)
        .flex()
        .items_center()
        .justify_center()
        .text_size(badge_size)
        .font_weight(badge_weight)
        .text_color(glass)
        .child(SharedString::from(tile.letter.to_string()));
    // The tile's inner icon slot: the §7.2 monogram, or the resolved
    // favicon image when the owning profile's store holds one (the same
    // record the tab row derives from). A decode failure falls back to the
    // monogram silently (the acquisition contract's fallback rule).
    let icon_slot: gpui::AnyElement = match tile.image.clone() {
        Some(image) => img(ImageSource::Image(image))
            .id(favorite_tile_icon_id(tile.index))
            .size(size)
            .rounded(radius)
            .object_fit(ObjectFit::Contain)
            .with_fallback(move || {
                div()
                    .size(size)
                    .rounded(radius)
                    .bg(slate)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(badge_size)
                    .font_weight(badge_weight)
                    .text_color(glass)
                    .child(SharedString::from(letter.to_string()))
                    .into_any()
            })
            .into_any(),
        None => monogram.into_any(),
    };
    div()
        .id(favorite_tile_id(tile.index))
        .debug_selector(move || favorite_tile_id(tile.index).to_string())
        .w(width)
        .h(height)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_activate)
        // §18.2: a white veil over the (space-tinted) rail — the surface
        // layer, which pops (§18.5) without moving the tile's box.
        .child(pop::surface_layer(
            theme,
            favorite_tile_radius(theme.density),
            pop,
        ))
        .child(icon_slot)
        .when_some(pop::speck_layer(theme, pop), |el, specks| el.child(specks))
}

/// §5/§18.3 Favorite tile corner radius: `radius:md` (10), `radius:sm` at
/// S density. The empty slot and the drop cell share it so every cell in the
/// grid reads as the same tile.
pub fn favorite_tile_radius(density: crate::theme::Density) -> gpui::Pixels {
    match density {
        crate::theme::Density::Small => Radius.small(),
        _ => Radius.medium(),
    }
}

/// The favorites region's deterministic height (points) — a pure function
/// of density, favorite count and whether a tab drag offers the drop cell
/// (never of the live rail width). It mirrors the render in
/// `sidebar::regions::favorites_surface`: the wrapper's `pt:sp1`/`pb:sp2`
/// around either the zero-favorites band or the wrapped grid, whose last
/// row also holds the rest slot (no extra row) or the drop cell (one more
/// row only when the tiles fill the last row). The E2E compares the band
/// anchor's measured rect with this value, and the offer growth is its
/// difference.
pub fn favorites_region_height(theme: &Theme, tile_count: usize, offer: bool) -> f32 {
    let body = if tile_count == 0 {
        favorites_empty_band_height(theme)
    } else {
        let cells = tile_count + usize::from(offer);
        let rows = cells.div_ceil(GRID_COLUMNS);
        let tile = f32::from(theme.density.favorite_tile_height());
        let gap = f32::from(theme.density.favorites_gap());
        rows as f32 * tile + (rows - 1) as f32 * gap
    };
    f32::from(Spacing::sp1()) + body + f32::from(Spacing::sp2())
}

mod affordance;
mod pop;
pub use affordance::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Density;

    #[test]
    fn favorite_tiles_use_radius_md_and_radius_sm_at_small_density() {
        // §5/§18.3: Favorite tiles are `radius:md` (10); S keeps `radius:sm`.
        assert_eq!(favorite_tile_radius(Density::Small), Radius.small());
        assert_eq!(favorite_tile_radius(Density::Normal), gpui::px(10.0));
        assert_eq!(favorite_tile_radius(Density::ExtraLarge), gpui::px(10.0));
        // The tile surface is the §18.2 veil, never an opaque grey.
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = Theme::new(mode, crate::theme::ContrastMode::Standard, Density::Normal);
            assert!(theme.palette.surface_favorite.a < 1.0);
        }
    }

    fn tiles() -> Vec<FavoriteTileSpec> {
        vec![
            FavoriteTileSpec {
                index: 0,
                tab: 11,
                space: 1,
                profile: 1,
                title: "Example".into(),
                url: "https://example.com/a".into(),
                letter: 'E',
                hue: 0,
                image: None,
            },
            FavoriteTileSpec {
                index: 1,
                tab: 22,
                space: 1,
                profile: 1,
                title: "Local".into(),
                url: "http://localhost:8080/x".into(),
                letter: 'L',
                hue: 1,
                image: None,
            },
        ]
    }

    #[test]
    fn tile_specs_accept_the_shared_resolved_image_record() {
        let image = Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            b"not decoded until the asset pipeline".to_vec(),
        ));
        let tile = FavoriteTileSpec {
            index: 0,
            tab: 11,
            space: 1,
            profile: 1,
            title: "Example".into(),
            url: "https://example.com/a".into(),
            letter: 'E',
            hue: 0,
            image: Some(image.clone()),
        };
        assert!(tile
            .image
            .as_ref()
            .is_some_and(|stored| Arc::ptr_eq(stored, &image)));
    }

    #[test]
    fn tile_specs_keep_the_exact_tab_owner() {
        // A favorite is an additional presentation of one concrete `TabId`;
        // the resolved tile carries the same tab/space/profile the L5 read
        // model reports (DESIGN.md §11.10 data-source contract).
        let tiles = tiles();
        assert_eq!(tiles[0].index, 0);
        assert_eq!(tiles[0].tab, 11);
        assert_eq!(tiles[0].space, 1);
        assert_eq!(tiles[0].profile, 1);
        assert_eq!(tiles[0].title, "Example");
        assert_eq!(tiles[1].letter, 'L');
    }

    #[test]
    fn the_grid_is_four_columns_spanning_the_rail_at_every_width() {
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = crate::theme::Theme::new(
                ThemeMode::Light,
                crate::theme::ContrastMode::Standard,
                density,
            );
            let gap = f32::from(density.favorites_gap());
            let mut previous = 0.0;
            for rail in [200.0, 250.0, 300.0, 340.0, 400.0] {
                let tile = favorite_tile_width(&theme, rail);
                let content = rail - 4.0 - 16.0;
                assert!(tile > 0.0);
                assert!(
                    (tile * 4.0 + gap * 3.0 - content).abs() < 0.01,
                    "no trailing slack at {rail} pt ({density:?})"
                );
                assert!(tile > previous, "a wider rail widens the tiles");
                previous = tile;
            }
            // Height is a pure function of density and count: 0–9 tiles
            // wrap into `ceil(n / 4)` rows inside `pt:sp1` + `pb:sp2`.
            let tile_h = f32::from(density.favorite_tile_height());
            for n in 1..=9usize {
                let rows = n.div_ceil(4);
                let grid = rows as f32 * tile_h + (rows - 1) as f32 * gap;
                assert!(
                    (favorites_region_height(&theme, n, false) - (4.0 + grid + 8.0)).abs() < 0.01,
                    "{n} tiles wrap into {rows} rows ({density:?})"
                );
            }
            // Zero favorites: the band (S 48 / N 56 / XL 64) inside the
            // same padding, identical at rest and as the drag target.
            assert_eq!(
                favorites_region_height(&theme, 0, false),
                4.0 + favorites_empty_band_height(&theme) + 8.0
            );
            assert_eq!(
                favorites_region_height(&theme, 0, true),
                favorites_region_height(&theme, 0, false)
            );
        }
        assert_eq!(grid_columns(), 4);
    }

    #[test]
    fn tile_ids_are_deterministic_per_index() {
        assert_eq!(favorite_tile_id(0).to_string(), "bbr-fav-0");
        assert_eq!(favorite_tile_id(12).to_string(), "bbr-fav-12");
        assert_eq!(favorite_tile_icon_id(3).to_string(), "bbr-fav-icon-3");
    }

    #[test]
    fn the_offer_growth_is_the_region_height_difference() {
        // The drag resolver subtracts `favorites_offer_growth`; the render
        // grows by what `favorites_region_height` describes. One function
        // family, so the two can never disagree.
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = crate::theme::Theme::new(
                ThemeMode::Dark,
                crate::theme::ContrastMode::Standard,
                density,
            );
            for n in 0..=9usize {
                let grown = favorites_region_height(&theme, n, true)
                    - favorites_region_height(&theme, n, false);
                assert!(
                    (grown - favorites_offer_growth(&theme, n)).abs() < 0.01,
                    "{n} favorites ({density:?})"
                );
            }
        }
    }
}
