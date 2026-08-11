//! The G2 Group 4 Favorites-grid story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::favorites` symbols — the §11.10 deterministic
//! tile grid (`favorites_grid`) and the §7.2 monogram destination tiles
//! (`favorite_tile`) — render every valid matrix family from the one
//! matrix authority [`crate::favorites_matrix`] with recorder callbacks
//! proving which production callback fired (`on_activate`). Rejected
//! combinations (reorder during selection; a tile without a resolved
//! tab/space/profile identity) refuse fixture construction and are shown
//! with their DESIGN reasons. No gallery-local reimplementation of
//! anatomy, no copied tokens, no fake engine/controller/portal.
//!
//! The §10 tile states (default/hover/active/focused) and the §10.9
//! drag-target wash are not expressible through the current production
//! component (`FavoriteTileSpec` carries no state field; the drag-target
//! wash is the app's composition wrap) — the story documents them rather
//! than inventing families (packet rule: the matrix derives from DESIGN
//! and the current render modules).

use std::rc::Rc;

use bbr_ui::components::favorites::{favorites_ids, grid_columns};
use bbr_ui::components::{favorite_tile, favorites_grid};
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::favorites_matrix::{
    build_favorites_fixture, favorite_case_valid, FAVORITES_REJECTED, FAVORITES_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the favorites families
/// (`favorites-family-<n>`, where `<n>` is the family's declaration index
/// in [`FAVORITES_VALID`]). Test targets only — never product ids.
pub const FAVORITES_FAMILY_SELECTOR: &str = "favorites-family-";

/// §11.10 Favorites grid: the production `bbr-ui` `favorites_grid` +
/// `favorite_tile` render every valid matrix family — the per-density
/// column contract (S 5 / N 4 / XL 3), the §7.2 monogram tile anatomy,
/// the resolved cross-context tab/space/profile identity, and the empty
/// region (label hidden) — with recorder `on_activate` callbacks. The
/// rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_favorites(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let group4 = ctx
        .group4
        .as_ref()
        .expect("Favorites grid story renders with the recorder");
    let on_activate = Rc::clone(&group4.observer.on_favorite_activate);

    let families: Vec<AnyElement> = FAVORITES_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            // Each family renders at its own §8 density through the
            // production component (the canvas theme stays the caller's).
            let case_theme = Theme::new(theme.mode, theme.contrast, case.density);
            let tiles = build_favorites_fixture(case).expect("valid matrix family builds");
            let body: AnyElement = if tiles.is_empty() {
                // §11.4 region rule: the empty region hides its section
                // label — no section node, no grid (the production
                // `favorites_ids` derivation returns nothing).
                value_row(
                    theme,
                    "empty region",
                    "label hidden — bbr-section-favorites / bbr-favorites render nothing",
                )
            } else {
                let children: Vec<AnyElement> = tiles
                    .iter()
                    .map(|tile| {
                        let index = tile.index;
                        let activate = {
                            let on = Rc::clone(&on_activate);
                            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                on(index, window, cx)
                            }
                        };
                        favorite_tile(&case_theme, tile, activate).into_any()
                    })
                    .collect();
                favorites_grid(&case_theme, children).into_any()
            };
            let columns = grid_columns(case.density);
            let meta = if tiles.is_empty() {
                "0 tiles · label hidden".to_string()
            } else if case.cross_context {
                format!(
                    "{columns}-col · {} tiles · semantic name includes space 7 · profile 3",
                    tiles.len()
                )
            } else {
                format!("{columns}-col · {} tiles", tiles.len())
            };
            div()
                .debug_selector(move || format!("{FAVORITES_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(body)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(theme.palette.ink)
                                .child(SharedString::from(case.label)),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(theme.palette.ink_faint)
                                .child(SharedString::from(meta)),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `favorite_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = FAVORITES_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !favorite_case_valid(&r.case),
                "rejected favorites {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.10 favorites grid (production favorites_grid + favorite_tile)"),
        value_row(
            theme,
            "ids",
            "bbr-section-favorites · bbr-favorites · bbr-fav-<n> · bbr-fav-icon-<n>",
        ),
        value_row(
            theme,
            "callbacks",
            "on_activate (recorder) — the exact favorited tab activation",
        ),
        value_row(
            theme,
            "columns",
            "S 5 · N 4 · XL 3 (deterministic tile → index mapping, §11.10)",
        ),
        value_row(
            theme,
            "data source",
            "every tile resolves TabId + SpaceId + ProfileId; the semantic name includes the owning space/profile when they differ from the current context",
        ),
        value_row(
            theme,
            "§10 tile states",
            "default/hover/active/focused are DESIGN §10 states the current FavoriteTileSpec carries no field for — the production tile renders default only (no invented families)",
        ),
        value_row(
            theme,
            "§10.9 drag-target wash",
            "app composition (bbr-app favorites_surface border while a drag targets the grid) — not expressible through the production component here",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    children.push(section_heading(theme, "§15 id derivation"));
    let empty_ids = favorites_ids(&[]).len();
    children.push(value_row(
        theme,
        "empty region",
        &format!("favorites_ids(&[]) = {empty_ids} — the label hides with the grid"),
    ));
    story_page(theme, "Favorites grid", children)
}
