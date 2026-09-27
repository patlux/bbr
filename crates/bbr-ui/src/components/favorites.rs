//! DESIGN.md §11.10 Favorites grid: a `type:badge` "Favorites" section over
//! a grid of square destination tiles.
//!
//! The grid renders the §7.2 favicon/monogram tile centered in a
//! `radius:sm` tile at `density:row-tile` size, `density:favorites-gap`
//! (S `sp:1` / N `sp:2` / XL `sp:2`) apart, with fixed per-density column
//! counts (S 5 / N 4 / XL 3) so the tile → index mapping is deterministic
//! at every sidebar width. Favorites are destinations
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
use crate::theme::{Density, Radius, Spacing, Theme, ThemeMode};

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

/// Fixed per-density column count (DESIGN.md §11.10): S 5 / N 4 / XL 3, so
/// the tile → index mapping is deterministic at every sidebar width.
#[allow(dead_code)] // consumed by the rail composition step's grid layout
pub fn grid_columns(density: Density) -> usize {
    match density {
        Density::Small => 5,
        Density::Normal => 4,
        Density::ExtraLarge => 3,
    }
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
/// (open-vs-focus semantics is an open decision).
pub fn favorite_tile(
    theme: &Theme,
    tile: &FavoriteTileSpec,
    on_activate: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let width = theme.density.favorite_tile_width();
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
        .rounded(Radius.small())
        .bg(theme.palette.surface_favorite)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_activate)
        .child(icon_slot)
}

/// The favorites region's deterministic top-down height (points): the
/// `type:badge` label (padding + line) plus the wrapped tile rows at the
/// fixed per-density column count with `sp:1` gaps. The drag resolver and
/// the render both derive the band from this same math, so a drop target
/// above the tab list lines up exactly with the rendered grid.
pub fn favorites_region_height(theme: &Theme, tile_count: usize) -> f32 {
    if tile_count == 0 {
        return 0.0;
    }
    let columns = grid_columns(theme.density);
    let tile = f32::from(theme.density.favorite_tile_height());
    let gap = f32::from(theme.density.favorites_gap());
    let rows = tile_count.div_ceil(columns);
    let grid = rows as f32 * tile + (rows.saturating_sub(1)) as f32 * gap;
    // Label: `py:sp1` on both sides + the badge line (size × 1.2 lh).
    let label = f32::from(Spacing::sp1()) * 2.0 + f32::from(theme.type_.badge.size) * 1.2;
    // The surface wrapper adds `pb:sp1` (see `favorites_surface`).
    label + grid + f32::from(Spacing::sp1())
}

/// The §15 id set for the `visible_ids` derivation: the section label only
/// while tiles exist, then the grid, the tiles, and their icon slots. The
/// caller decides the empty-region hiding exactly like the render.
#[allow(dead_code)] // §15 visible-ids seam; consumed by the app's snapshot
pub fn favorites_ids(tiles: &[FavoriteTileSpec]) -> Vec<String> {
    let mut ids = Vec::new();
    if tiles.is_empty() {
        return ids;
    }
    ids.push(FAVORITES_SECTION_ID.to_string());
    ids.push(FAVORITES_ID.to_string());
    for tile in tiles {
        ids.push(favorite_tile_id(tile.index).to_string());
        ids.push(favorite_tile_icon_id(tile.index).to_string());
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn grid_columns_follow_the_density_contract() {
        assert_eq!(grid_columns(Density::Small), 5);
        assert_eq!(grid_columns(Density::Normal), 4);
        assert_eq!(grid_columns(Density::ExtraLarge), 3);
    }

    #[test]
    fn tile_ids_are_deterministic_per_index() {
        assert_eq!(favorite_tile_id(0).to_string(), "bbr-fav-0");
        assert_eq!(favorite_tile_id(12).to_string(), "bbr-fav-12");
        assert_eq!(favorite_tile_icon_id(3).to_string(), "bbr-fav-icon-3");
    }

    #[test]
    fn region_height_is_deterministic_and_zero_when_empty() {
        let theme = crate::theme::Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Normal,
        );
        assert_eq!(favorites_region_height(&theme, 0), 0.0);
        let one = favorites_region_height(&theme, 1);
        let two = favorites_region_height(&theme, 2);
        assert!(one > 0.0);
        // One wrapped row at N: the tiles sit on one line; two tiles share
        // the same line, so the heights agree below the first wrap.
        assert_eq!(one, two);
        let columns = grid_columns(theme.density);
        let wrapped = favorites_region_height(&theme, columns + 1);
        assert!(wrapped > two, "a wrapped second row adds tile + gap height");
    }

    #[test]
    fn favorites_ids_hide_the_empty_region() {
        assert!(favorites_ids(&[]).is_empty());
        let tiles = tiles();
        let ids = favorites_ids(&tiles);
        assert!(ids.contains(&"bbr-section-favorites".to_string()));
        assert!(ids.contains(&"bbr-favorites".to_string()));
        for tile in &tiles {
            assert!(ids.contains(&favorite_tile_id(tile.index).to_string()));
            assert!(ids.contains(&favorite_tile_icon_id(tile.index).to_string()));
        }
    }
}
