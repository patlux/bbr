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
//! The §11.10 affordances (zero-favorites drop band, glyph slot, drag-time
//! drop cell) and the §11.2 ghost row render through their production
//! components at rest and as §10.9 drag targets. The §10 tile states
//! (default/hover/active/focused) are not expressible through the current
//! production component (`FavoriteTileSpec` carries no state field) — the
//! story documents them rather than inventing families.

use std::rc::Rc;

use bbr_ui::components::drag_offer::pinned_drop_row;
use bbr_ui::components::favorites::{
    favorite_tile_width, favorites_drop_cell, favorites_empty_band, favorites_empty_slot,
    favorites_ids, grid_columns,
};
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
/// `favorite_tile` render every valid matrix family — the four-column
/// contract at every density, the §7.2 monogram tile anatomy, the resolved
/// cross-context tab/space/profile identity, and the empty region (the
/// drop band) — with recorder `on_activate` callbacks. The
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
            let case_theme = Theme::new(theme.mode, theme.contrast, case.density)
                .with_motion_mode(theme.motion_mode);
            let tiles = build_favorites_fixture(case).expect("valid matrix family builds");
            let body: AnyElement = if tiles.is_empty() {
                // §11.10: the empty region renders the drop band that names
                // the region and the gesture.
                favorites_empty_band(&case_theme, false).into_any()
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
                        favorite_tile(
                            &case_theme,
                            tile,
                            bbr_ui::components::favorites::favorite_tile_width(&case_theme, 300.0),
                            None,
                            activate,
                        )
                        .into_any()
                    })
                    .collect();
                favorites_grid(&case_theme, children).into_any()
            };
            let columns = grid_columns();
            let meta = if tiles.is_empty() {
                "0 tiles · drop band".to_string()
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
            "4 at every density and width; tile width derives from the rail, height from the density (§11.10)",
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
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "§11.10 / §11.2 drag offers (rest · offered · live target)",
    ));
    children.extend(drag_offer_samples(theme));
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    children.push(section_heading(theme, "§15 id derivation"));
    let empty_ids = favorites_ids(&[], false).join(" · ");
    children.push(value_row(theme, "empty region", &empty_ids));
    story_page(theme, "Favorites grid", children)
}

/// The production affordance components in each state: the drop band at
/// rest and as the live target, the glyph slot, the drop cell offered and
/// targeted, and the ghost row offered and targeted (its rect anchor is an
/// empty placeholder here — the app owns the recorder).
fn drag_offer_samples(theme: &Theme) -> Vec<AnyElement> {
    let width = favorite_tile_width(theme, 300.0);
    let labeled = |label: &str, element: AnyElement| -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(element)
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(theme.palette.ink_faint)
                    .child(SharedString::from(label.to_string())),
            )
            .into_any()
    };
    vec![
        labeled(
            "bbr-favorites-empty — rest",
            favorites_empty_band(theme, false).into_any(),
        ),
        labeled(
            "bbr-favorites-empty — drag target",
            favorites_empty_band(theme, true).into_any(),
        ),
        labeled(
            "glyph slot · drop cell offered · drop cell targeted",
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(favorites_empty_slot(theme, width))
                .child(favorites_drop_cell(theme, width, false).id("drop-offered"))
                .child(favorites_drop_cell(theme, width, true).id("drop-targeted"))
                .into_any(),
        ),
        labeled(
            "bbr-pinned-drop-row — offered",
            pinned_drop_row(theme, false, div().into_any()).into_any(),
        ),
        labeled(
            "bbr-pinned-drop-row — live target",
            pinned_drop_row(theme, true, div().into_any()).into_any(),
        ),
    ]
}
