//! DESIGN.md §11.10 Favorites affordances: the zero-favorites drop band,
//! the glyph-only slot in the next free cell, and the drag-time drop cell
//! (`bbr-favorites-drop-cell`) a populated grid offers once a tab drag is
//! active — plus the §15 id derivation that mirrors which one renders.

use gpui::{div, prelude::*, Div, Stateful, Styled};

use super::{
    favorite_tile_icon_id, favorite_tile_id, FavoriteTileSpec, FAVORITES_ID, FAVORITES_SECTION_ID,
    GRID_COLUMNS,
};
use crate::theme::{Radius, Spacing, Theme};

/// The §11.10 empty drop affordance (`bbr-favorites-empty`): the full-width
/// band at zero favorites, then a glyph-only slot in the next free cell.
pub const FAVORITES_EMPTY_ID: &str = "bbr-favorites-empty";
/// The band's region name — present only in the zero-favorites band.
pub const FAVORITES_EMPTY_LABEL_ID: &str = "bbr-favorites-empty-label";
/// The band's gesture hint — present only in the zero-favorites band.
pub const FAVORITES_EMPTY_HINT_ID: &str = "bbr-favorites-empty-hint";

/// Whether the empty affordance renders for `count` favorites: the full
/// band at zero, a trailing glyph slot while the last row has a free cell,
/// and nothing when the last row is exactly full (an affordance never costs
/// a whole row).
pub fn empty_slot_shown(count: usize) -> bool {
    count == 0 || !count.is_multiple_of(GRID_COLUMNS)
}

/// The zero-favorites band height (S 48 / N 56 / XL 64): identical at rest
/// and as a drag target, so nothing in the rail reflows mid-drag.
pub fn favorites_empty_band_height(theme: &Theme) -> f32 {
    f32::from(theme.density.favorite_tile_height()) + 12.0
}

/// The §11.10 zero-favorites band: a dashed hairline outline naming the
/// region and the gesture. As a drag target it takes the §10.9 language —
/// solid accent outline, `tint.selected` fill, the label commits to "Add to
/// Favorites" and the hint states the drop-time fact — without changing its
/// geometry.
pub fn favorites_empty_band(theme: &Theme, targeted: bool) -> Stateful<Div> {
    let p = theme.palette;
    let (label, hint, label_color) = if targeted {
        (
            "Add to Favorites",
            "Reachable from every space and profile",
            p.accent_ink,
        )
    } else {
        (
            "Favorites",
            "Drop a tab here to favorite it",
            p.ink_secondary,
        )
    };
    let band = div()
        .id(FAVORITES_EMPTY_ID)
        .debug_selector(|| FAVORITES_EMPTY_ID.to_string())
        .mx(Spacing::sp2())
        .h(gpui::px(favorites_empty_band_height(theme)))
        .px(Spacing::sp3())
        .rounded(Radius.medium())
        .border_1()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .child(crate::icons::Icon::Downloads.canvas(theme, gpui::px(16.0)))
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .child(
                    div()
                        .id(FAVORITES_EMPTY_LABEL_ID)
                        .text_size(theme.type_.meta.size)
                        .font_weight(gpui::FontWeight(600.0))
                        .text_color(label_color)
                        .child(label),
                )
                .child(
                    div()
                        .id(FAVORITES_EMPTY_HINT_ID)
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(hint),
                ),
        );
    if targeted {
        band.border_color(p.accent).bg(p.tint_selected)
    } else {
        band.border_dashed().border_color(p.hairline)
    }
}

/// The glyph-only slot in the next free cell once favorites exist: tile
/// geometry, dashed hairline outline, the download-into glyph, no text.
pub fn favorites_empty_slot(theme: &Theme, width: f32) -> Stateful<Div> {
    div()
        .id(FAVORITES_EMPTY_ID)
        .debug_selector(|| FAVORITES_EMPTY_ID.to_string())
        .w(gpui::px(width))
        .h(theme.density.favorite_tile_height())
        .rounded(Radius.small())
        .border_1()
        .border_dashed()
        .border_color(theme.palette.hairline)
        .flex()
        .items_center()
        .justify_center()
        .child(crate::icons::Icon::Downloads.canvas(theme, gpui::px(16.0)))
}

/// The §11.10 drop cell id: present only while a tab drag is in flight over
/// a populated grid.
pub const FAVORITES_DROP_CELL_ID: &str = "bbr-favorites-drop-cell";

/// The rail growth the drop cell commits at drag activation: one more tile
/// row (tile height plus the row gap) exactly when the last row is full;
/// otherwise the cell takes the next free cell (replacing the rest slot)
/// and the zero-favorites band keeps its geometry. Pure in density and
/// count, so the render and the drag resolver agree by construction.
pub fn favorites_offer_growth(theme: &Theme, count: usize) -> f32 {
    if count == 0 || !count.is_multiple_of(GRID_COLUMNS) {
        return 0.0;
    }
    f32::from(theme.density.favorite_tile_height()) + f32::from(theme.density.favorites_gap())
}

/// The §11.10 drop cell: one additional cell at the end of the wrap order
/// with tile geometry, a dashed `accent` outline, no fill, and a centered
/// plus glyph in `accent`. As the live target (pointer over the grid) it
/// takes the §10.9 `drag-target` language: solid `accent` outline and
/// `tint.selected` fill.
pub fn favorites_drop_cell(theme: &Theme, width: f32, targeted: bool) -> Stateful<Div> {
    let p = theme.palette;
    let cell = div()
        .id(FAVORITES_DROP_CELL_ID)
        .debug_selector(|| FAVORITES_DROP_CELL_ID.to_string())
        .w(gpui::px(width))
        .h(theme.density.favorite_tile_height())
        .rounded(Radius.small())
        .border_1()
        .border_color(p.accent)
        .flex()
        .items_center()
        .justify_center()
        .child(crate::icons::Icon::Plus.canvas_with(theme, gpui::px(16.0), p.accent));
    if targeted {
        cell.bg(p.tint_selected)
    } else {
        cell.border_dashed()
    }
}

/// The §15 id set for the `visible_ids` derivation, mirroring the render:
/// the region wrapper always; the zero-favorites band with its label and
/// hint (also the drag target at zero favorites), or the grid with its
/// tiles and icons plus the trailing cell — the drop cell while a tab drag
/// offers it (`drag_offer`), else the glyph slot while the last row has a
/// free cell.
pub fn favorites_ids(tiles: &[FavoriteTileSpec], drag_offer: bool) -> Vec<String> {
    let mut ids = vec![FAVORITES_SECTION_ID.to_string()];
    if tiles.is_empty() {
        ids.push(FAVORITES_EMPTY_ID.to_string());
        ids.push(FAVORITES_EMPTY_LABEL_ID.to_string());
        ids.push(FAVORITES_EMPTY_HINT_ID.to_string());
        return ids;
    }
    ids.push(FAVORITES_ID.to_string());
    for tile in tiles {
        ids.push(favorite_tile_id(tile.index).to_string());
        ids.push(favorite_tile_icon_id(tile.index).to_string());
    }
    if drag_offer {
        ids.push(FAVORITES_DROP_CELL_ID.to_string());
    } else if empty_slot_shown(tiles.len()) {
        ids.push(FAVORITES_EMPTY_ID.to_string());
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, ThemeMode};

    fn tiles(n: usize) -> Vec<FavoriteTileSpec> {
        (0..n)
            .map(|index| FavoriteTileSpec {
                index,
                tab: index as u64 + 10,
                space: 1,
                profile: 1,
                title: format!("Tile {index}"),
                url: format!("https://t{index}.example"),
                letter: 'T',
                hue: index % 6,
                image: None,
            })
            .collect()
    }

    #[test]
    fn the_empty_affordance_follows_the_wrap_order() {
        assert!(empty_slot_shown(0), "the full band at zero favorites");
        for n in [1, 2, 3, 5, 6, 7, 9] {
            assert!(empty_slot_shown(n), "{n}: a free cell in the last row");
        }
        for n in [4, 8, 12] {
            assert!(
                !empty_slot_shown(n),
                "{n}: a full row never opens a new row"
            );
        }
        let empty = favorites_ids(&[], false);
        assert!(empty.contains(&FAVORITES_EMPTY_LABEL_ID.to_string()));
        assert!(empty.contains(&FAVORITES_EMPTY_HINT_ID.to_string()));
        assert!(
            !empty.contains(&FAVORITES_ID.to_string()),
            "no grid without tiles"
        );
    }

    #[test]
    fn favorites_ids_mirror_the_populated_grid() {
        let tiles = tiles(2);
        let ids = favorites_ids(&tiles, false);
        assert!(ids.contains(&"bbr-section-favorites".to_string()));
        assert!(ids.contains(&"bbr-favorites".to_string()));
        for tile in &tiles {
            assert!(ids.contains(&favorite_tile_id(tile.index).to_string()));
            assert!(ids.contains(&favorite_tile_icon_id(tile.index).to_string()));
        }
        // Two tiles leave free cells: the glyph-only slot, without text.
        assert!(ids.contains(&FAVORITES_EMPTY_ID.to_string()));
        assert!(!ids.contains(&FAVORITES_EMPTY_LABEL_ID.to_string()));
    }

    #[test]
    fn the_drop_cell_replaces_the_rest_slot_only_on_a_populated_grid() {
        // Zero favorites: the band itself is the target — no drop cell.
        let empty = favorites_ids(&[], true);
        assert!(empty.contains(&FAVORITES_EMPTY_ID.to_string()));
        assert!(!empty.contains(&FAVORITES_DROP_CELL_ID.to_string()));
        for n in [1, 3, 4, 5, 8] {
            let ids = favorites_ids(&tiles(n), true);
            assert!(ids.contains(&FAVORITES_DROP_CELL_ID.to_string()), "{n}");
            assert!(
                !ids.contains(&FAVORITES_EMPTY_ID.to_string()),
                "{n}: one trailing cell, never slot and cell together"
            );
            let rest = favorites_ids(&tiles(n), false);
            assert!(!rest.contains(&FAVORITES_DROP_CELL_ID.to_string()), "{n}");
        }
    }

    #[test]
    fn the_drop_cell_grows_the_region_only_for_a_full_last_row() {
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, density);
            let row =
                f32::from(density.favorite_tile_height()) + f32::from(density.favorites_gap());
            for n in 0..=9usize {
                let expected = if n > 0 && n % 4 == 0 { row } else { 0.0 };
                assert_eq!(
                    favorites_offer_growth(&theme, n),
                    expected,
                    "{n} ({density:?})"
                );
            }
        }
        assert_eq!(FAVORITES_DROP_CELL_ID, "bbr-favorites-drop-cell");
    }
}
