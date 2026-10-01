//! The Favorites-grid matrix authority (packet
//! `component-gallery-browser-components.md`, Group 4): **one declaration
//! source** for every valid §11.10 favorites family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::components::favorites` symbols
//! (`favorites_grid`, `favorite_tile`), and the fixture builder refuses
//! (`None`) any case the validity authority rejects — so the story, the
//! fixture builder, the matrix table, and the tests cannot drift. The
//! rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Favorites row: S / N / XL
//! grids (four columns at every density — §11.10 — so the tile → index
//! mapping is stable at every sidebar width); the
//! §7.2 monogram tile anatomy (distinct letters/hues through the
//! production `FavoriteTileSpec`); the resolved
//! `TabId + SpaceId + ProfileId` identity of every tile (a favorite is an
//! additional presentation of one exact tab — the semantic name includes
//! the owning space/profile when they differ from the current context);
//! the empty region (the §11.10 drop band); the tile that carries the
//! selection (its tab is the selected tab, §11.10); and the tile of a tab
//! closed in place (greyed, §10.12). Rejected: reorder during selection
//! (tiles never reorder, §11.10), a tile without a resolved
//! tab/space/profile identity (§11.10 data source), and a selected closed
//! tile (§10.12: the selection never lands on a closed tab).
//!
//! The §10 tile states hover/active/focused and the §10.9 drag-target
//! wash are **not expressible through the current production component**:
//! `FavoriteTileSpec` carries only the selection and the closed state, and
//! the drag-target wash is the app's composition wrap (`bbr-app`'s
//! `favorites_surface` border). Per the
//! packet's no-invented-variants rule, those DESIGN states are documented
//! in the story rather than fabricated as families.

use bbr_ui::components::FavoriteTileSpec;
use bbr_ui::theme::Density;

/// The tile-index offset per matrix family: family `n` renders its tiles
/// at `bbr-fav-<n * FAVORITES_FAMILY_OFFSET + tile>` so every tile id is
/// unique across the story (the E2E targets one exact tile). Pinned by
/// the matrix test so ids can never drift from the declaration.
pub const FAVORITES_FAMILY_OFFSET: usize = 10;

/// One matrix row: a family label, the production `FavoriteTileSpec`
/// dimensions, and the matrix-only claims that name the rejected
/// combinations in the same case shape. `cross_context` renders the
/// §11.10 cross-context identity family (the tile's semantic name
/// includes the owning space/profile when they differ from the current
/// context).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FavoriteCase {
    pub label: &'static str,
    /// The §8 density the family renders at (drives the §11.10 tile
    /// height; the column count is four at every density).
    pub density: Density,
    /// The number of destination tiles in this family's grid.
    pub tile_count: usize,
    /// §11.10 cross-context identity: the owning space/profile differ
    /// from the current context (the semantic name includes both).
    pub cross_context: bool,
    /// Matrix-only: this case claims the grid reorders tiles during
    /// selection (§11.10 rejects it — tiles never reorder).
    pub claims_reorder: bool,
    /// Matrix-only: this case claims a tile without a resolved
    /// `TabId + SpaceId + ProfileId` semantic name (§11.10 data source).
    pub missing_identity: bool,
    /// The family's second tile carries the selection (its tab is the
    /// selected tab, §11.10).
    pub selected: bool,
    /// The family's second tile's tab is closed in place (§10.12).
    pub closed: bool,
}

impl FavoriteCase {
    /// The valid-family constructor: the production fields with the
    /// matrix-only flags defaulting to false.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        density: Density,
        tile_count: usize,
        cross_context: bool,
    ) -> Self {
        FavoriteCase {
            label,
            density,
            tile_count,
            cross_context,
            claims_reorder: false,
            missing_identity: false,
            selected: false,
            closed: false,
        }
    }

    /// The family whose second tile carries the selection.
    const fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    /// The family whose second tile's tab is closed in place.
    const fn closed(mut self) -> Self {
        self.closed = true;
        self
    }
}

/// Every valid favorites family — the single declaration the story
/// renders. The density families cover the §11.10 four-column contract at
/// every density, the tile families cover the §7.2 monogram anatomy and the
/// resolved cross-context identity, and the empty family covers the §11.10
/// drop band.
#[rustfmt::skip]
pub const FAVORITES_VALID: &[FavoriteCase] = &[
    FavoriteCase::new("N grid", Density::Normal, 5, false),
    FavoriteCase::new("S grid", Density::Small, 6, false),
    FavoriteCase::new("XL grid", Density::ExtraLarge, 7, false),
    FavoriteCase::new("tile — default monogram + hue", Density::Normal, 3, false),
    FavoriteCase::new("tile — resolved tab/space/profile identity", Density::Normal, 2, true),
    FavoriteCase::new("empty region — drop band", Density::Normal, 0, false),
    FavoriteCase::new("tile — selected tab", Density::Normal, 3, false).selected(),
    FavoriteCase::new("tile — closed in place", Density::Normal, 3, false).closed(),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `FavoriteCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedFavorite {
    pub label: &'static str,
    pub case: FavoriteCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`favorite_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const FAVORITES_REJECTED: &[RejectedFavorite] = &[
    RejectedFavorite {
        label: "reorder during selection",
        case: FavoriteCase {
            label: "reorder during selection",
            density: Density::Normal,
            tile_count: 3,
            cross_context: false,
            claims_reorder: true,
            missing_identity: false,
            selected: false,
            closed: false,
        },
        reason: "§11.10: tiles never reorder during selection — the grid is a deterministic destination grid, not a draggable list",
    },
    RejectedFavorite {
        label: "a tile without a resolved TabId+SpaceId+ProfileId semantic name",
        case: FavoriteCase {
            label: "a tile without a resolved identity",
            density: Density::Normal,
            tile_count: 2,
            cross_context: false,
            claims_reorder: false,
            missing_identity: true,
            selected: false,
            closed: false,
        },
        reason: "§11.10 data source: every tile resolves TabId + SpaceId + ProfileId — a favorite is an additional presentation of one exact tab, and the semantic name includes the owning space/profile when they differ from the current context",
    },
    RejectedFavorite {
        label: "a selected closed tile",
        case: FavoriteCase::new("a selected closed tile", Density::Normal, 3, false).selected().closed(),
        reason: "§10.12: the selection never lands on a closed tab; activating the tile reopens it",
    },
];

/// The one favorites validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected grid.
pub fn favorite_case_valid(case: &FavoriteCase) -> bool {
    // §11.10: tiles never reorder during selection.
    if case.claims_reorder {
        return false;
    }
    // §11.10 data source: every tile resolves TabId + SpaceId + ProfileId.
    if case.missing_identity {
        return false;
    }
    // §10.12: a closed tab is never the selected tab.
    if case.selected && case.closed {
        return false;
    }
    true
}

/// Build the production `FavoriteTileSpec` list for a valid matrix family;
/// `None` for any rejected combination. Tiles are deterministic: family
/// `n` owns indices `n * FAVORITES_FAMILY_OFFSET..`, each tile resolves a
/// concrete tab/space/profile (the cross-context family reports a
/// different owning space/profile), and the §7.2 monogram letter + slate
/// hue derive deterministically from the tile position.
pub fn build_favorites_fixture(case: &FavoriteCase) -> Option<Vec<FavoriteTileSpec>> {
    if !favorite_case_valid(case) {
        return None;
    }
    let family = FAVORITES_VALID
        .iter()
        .position(|c| c.label == case.label)
        .unwrap_or(0);
    let offset = family * FAVORITES_FAMILY_OFFSET;
    let mut tiles = Vec::with_capacity(case.tile_count);
    for tile in 0..case.tile_count {
        let index = offset + tile;
        let letter = char::from(b'A' + (tile % 26) as u8);
        tiles.push(FavoriteTileSpec {
            index,
            // A favorite is an additional presentation of one exact tab.
            tab: 100 + index as u64,
            // The cross-context family reports an owning space/profile
            // that differs from the current context (§11.10 semantic
            // name); the rest stay in the current context.
            space: if case.cross_context { 7 } else { 1 },
            profile: if case.cross_context { 3 } else { 1 },
            title: format!("Site {letter}"),
            url: format!("https://site{}.example/", tile),
            letter,
            hue: tile % 6,
            image: None,
            closed: case.closed && tile == 1,
            selected: case.selected && tile == 1,
        });
    }
    Some(tiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::components::favorites::{favorites_ids, grid_columns};
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in FAVORITES_VALID {
            assert!(
                favorite_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let tiles = build_favorites_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(tiles.len(), case.tile_count, "{}", case.label);
            for tile in &tiles {
                // §11.10 data source: every tile resolves the concrete
                // tab/space/profile identity.
                assert_ne!(tile.tab, 0, "{}", case.label);
                assert_ne!(tile.space, 0, "{}", case.label);
                assert_ne!(tile.profile, 0, "{}", case.label);
                assert!(!tile.title.is_empty(), "{}", case.label);
                assert!(!tile.url.is_empty(), "{}", case.label);
                assert!(tile.letter.is_alphabetic(), "{}", case.label);
                assert!((0..6).contains(&tile.hue), "{}", case.label);
            }
        }
        for rejected in FAVORITES_REJECTED {
            assert!(
                !favorite_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_favorites_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = FAVORITES_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "N grid",
                "S grid",
                "XL grid",
                "tile — default monogram + hue",
                "tile — resolved tab/space/profile identity",
                "empty region — drop band",
                "tile — selected tab",
                "tile — closed in place",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in FAVORITES_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn the_grid_is_four_columns_at_every_density() {
        // §11.10: the grid is four columns at every density and width, so
        // the tile → index mapping is deterministic; density scales only
        // the tile height, the rail scales the tile width.
        assert_eq!(grid_columns(), 4);
        // The three density families wrap their tiles into those columns
        // through the production grid.
        let by_label = |label: &str| {
            FAVORITES_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        assert_eq!(by_label("S grid").tile_count, 6);
        assert_eq!(by_label("N grid").tile_count, 5);
        assert_eq!(by_label("XL grid").tile_count, 7);
    }

    #[test]
    fn tile_ids_are_unique_across_families_and_stable() {
        // Each family renders at `bbr-fav-<family * OFFSET + tile>`, so
        // the E2E targets one exact tile id without collisions.
        let by_label = |label: &str| {
            FAVORITES_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let first = build_favorites_fixture(by_label("N grid")).unwrap();
        let second = build_favorites_fixture(by_label("S grid")).unwrap();
        assert_eq!(first[0].index, 0, "family 0 starts at bbr-fav-0");
        assert_eq!(second[0].index, FAVORITES_FAMILY_OFFSET);
        assert_eq!(
            second[0].index, 10,
            "family 1 starts at bbr-fav-10 (the pinned offset)"
        );
        let all_indices: BTreeSet<usize> = FAVORITES_VALID
            .iter()
            .flat_map(|c| build_favorites_fixture(c).expect("valid family"))
            .map(|t| t.index)
            .collect();
        assert_eq!(
            all_indices.len(),
            FAVORITES_VALID.iter().map(|c| c.tile_count).sum::<usize>(),
            "every tile index is unique across the story"
        );
        // Stable: rebuilding yields the identical specs.
        assert_eq!(
            build_favorites_fixture(by_label("N grid")),
            build_favorites_fixture(by_label("N grid"))
        );
    }

    #[test]
    fn cross_context_family_carries_the_different_owning_identity() {
        // §11.10: the semantic name includes the owning space/profile when
        // they differ from the current context — the fixture reports the
        // different owning space/profile on every tile of the family.
        let case = FAVORITES_VALID
            .iter()
            .find(|c| c.cross_context)
            .expect("cross-context family declared");
        let tiles = build_favorites_fixture(case).unwrap();
        assert!(!tiles.is_empty());
        for tile in &tiles {
            assert_eq!(tile.space, 7, "{}", case.label);
            assert_eq!(tile.profile, 3, "{}", case.label);
        }
    }

    #[test]
    fn selected_and_closed_families_mark_exactly_their_second_tile() {
        let by_label = |label: &str| {
            FAVORITES_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let selected = build_favorites_fixture(by_label("tile — selected tab")).unwrap();
        let marks: Vec<(bool, bool)> = selected.iter().map(|t| (t.selected, t.closed)).collect();
        assert_eq!(marks, vec![(false, false), (true, false), (false, false)]);
        let closed = build_favorites_fixture(by_label("tile — closed in place")).unwrap();
        let marks: Vec<(bool, bool)> = closed.iter().map(|t| (t.selected, t.closed)).collect();
        assert_eq!(marks, vec![(false, false), (false, true), (false, false)]);
        // The closed family's second tile renders at its stable id.
        assert_eq!(closed[1].index, 7 * FAVORITES_FAMILY_OFFSET + 1);
    }

    #[test]
    fn empty_region_renders_the_drop_band() {
        // §11.10: the region is always present — with zero favorites it
        // renders the drop band (region name + gesture), never a grid.
        let empty = FAVORITES_VALID
            .iter()
            .find(|c| c.tile_count == 0)
            .expect("empty family declared");
        let tiles = build_favorites_fixture(empty).unwrap();
        assert!(tiles.is_empty());
        assert_eq!(
            favorites_ids(&tiles, false),
            vec![
                "bbr-section-favorites".to_string(),
                "bbr-favorites-empty".to_string(),
                "bbr-favorites-empty-label".to_string(),
                "bbr-favorites-empty-hint".to_string(),
            ]
        );
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = FAVORITES_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "reorder during selection",
                "a tile without a resolved TabId+SpaceId+ProfileId semantic name",
                "a selected closed tile",
            ]
        );
        for rejected in FAVORITES_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
