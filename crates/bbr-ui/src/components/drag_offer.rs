//! DESIGN.md §10.9/§11.2 drag offers: the organized area's "Pin tab here"
//! ghost row (`bbr-pinned-drop-row`), the lifted-row drag proxy
//! (`bbr-tab-drag-proxy`), and the one geometry function both the render
//! and the drag resolver use for the growth a drag commits at activation.
//!
//! Offers exist only while a tab drag is active (past the 4 pt activation
//! distance). They appear together in one frame and disappear on drop,
//! cancel, or Escape, so the rail grows at most once per gesture and never
//! reflows under the pointer mid-drag.

use gpui::{div, prelude::*, px, AnyElement, Div, SharedString, Stateful, Styled};

use crate::components::favorites::favorites_offer_growth;
use crate::components::{favicon_tile, RowVisuals};
use crate::icons::Icon;
use crate::state::TabVisualState;
use crate::theme::{Spacing, Theme};

/// The §11.2 ghost row id (present only while a tab drag offers pinning).
pub const PINNED_DROP_ROW_ID: &str = "bbr-pinned-drop-row";
/// The §10.9 drag proxy id (the lifted row following the pointer).
pub const TAB_DRAG_PROXY_ID: &str = "bbr-tab-drag-proxy";
/// The ghost row's visible label.
pub const PINNED_DROP_ROW_LABEL: &str = "Pin tab here";

/// The ghost row's layout height: one row pitch (`density:row` surface plus
/// the row gap), so the offer grows the rail by exactly one pitch.
pub fn pinned_drop_row_height(theme: &Theme) -> f32 {
    f32::from(theme.density.pitch())
}

/// The rail growth a drag commits at activation **above** its source row:
/// the Favorites drop cell's extra row (only when the last grid row is
/// full) plus, for an unpinned source, the ghost row that the organized
/// area appends above the regular tabs. Pinned sources sit above the ghost
/// row, and a pinned tab is never offered pinning, so only the Favorites
/// growth moves them. The drag resolver subtracts this from the pointer
/// displacement, so slot targets stay on the rows the render shows.
pub fn activation_growth(theme: &Theme, favorites: usize, unpinned_source: bool) -> f32 {
    let ghost = if unpinned_source {
        pinned_drop_row_height(theme)
    } else {
        0.0
    };
    favorites_offer_growth(theme, favorites) + ghost
}

/// The §11.2 ghost row: dashed `accent` outline on the §10.2 inset row
/// surface, no fill, a leading plus glyph and `type:meta` "Pin tab here" in
/// `accent.ink`. As the live target (pointer over it) the outline turns
/// solid and the surface takes `tint.accent` (§10.9, §18.2). `anchor` is the
/// app-owned rect recorder the drag resolver hit-tests against, so the
/// resolver never keeps a parallel layout model.
pub fn pinned_drop_row(theme: &Theme, targeted: bool, anchor: AnyElement) -> Stateful<Div> {
    let p = theme.palette;
    let surface = div()
        .absolute()
        .left(theme.density.row_inset())
        .right(theme.density.row_inset())
        .top(px(0.0))
        .bottom(px(0.0))
        .rounded(theme.density.density_radius())
        .border_1()
        .border_color(p.accent);
    let surface = if targeted {
        surface.bg(p.tint_accent)
    } else {
        surface.border_dashed()
    };
    div()
        .id(PINNED_DROP_ROW_ID)
        .debug_selector(|| PINNED_DROP_ROW_ID.to_string())
        .relative()
        .flex_shrink_0()
        .h(theme.density.row())
        .mb(theme.density.gap())
        .px(theme.density.row_inset() + theme.density.row_padding_x())
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .child(surface)
        .child(Icon::Plus.canvas_with(theme, theme.density.icon(), p.accent))
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .font_weight(theme.type_.meta.weight)
                .text_color(p.accent_ink)
                .child(PINNED_DROP_ROW_LABEL),
        )
        .child(anchor)
}

/// The drag proxy's window rect `[x, y, width, height]`: the lifted row
/// keeps the press point under the pointer. `dx` is the pointer's
/// horizontal travel from the press, `press_in_row` the press's y inside the
/// row surface. The width is the row surface inside the rail column.
pub fn drag_proxy_rect(
    theme: &Theme,
    rail_width: f32,
    dx: f32,
    pointer_y: f32,
    press_in_row: f32,
) -> [f32; 4] {
    let inset = f32::from(theme.density.row_inset());
    let width = (rail_width - crate::sidebar::geometry::RESIZE_HANDLE_WIDTH - 2.0 * inset).max(0.0);
    [
        inset + dx,
        pointer_y - press_in_row,
        width,
        f32::from(theme.density.row()),
    ]
}

/// Paper S-9 (move to space): while the pointer targets a space chip the
/// proxy floats above the strip — its bottom edge 4 pt above `strip_top` —
/// so the live chip and its destination tag stay visible. Above that line
/// the proxy keeps following the pointer unchanged.
pub fn lift_proxy_above_strip(rect: [f32; 4], strip_top: f32) -> [f32; 4] {
    let [x, y, width, height] = rect;
    [x, y.min(strip_top - 4.0 - height), width, height]
}

/// The §10.9 drag proxy: the dragged row lifted off the list (92% opacity,
/// `elev:2` shadow, 1 pt `border.strong` outline) at `rect` in window
/// coordinates. It paints the row's favicon and title only; the row left in
/// the list keeps its `dragging` state as the placeholder. The positioned
/// outer element carries no border, so a rect anchor appended to it records
/// exactly `rect`; the bordered surface fills it.
pub fn tab_drag_proxy(theme: &Theme, row: &RowVisuals, rect: [f32; 4]) -> Stateful<Div> {
    let p = theme.palette;
    let surface = div()
        .size_full()
        .px(theme.density.row_padding_x())
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .rounded(theme.density.density_radius())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.border_strong)
        .shadow(crate::theme::elevation::elev2(theme))
        .opacity(0.92)
        .child(favicon_tile(
            theme,
            row.index,
            row.favicon.clone(),
            TabVisualState::Dragging,
        ))
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(SharedString::from(row.title.to_string())),
        );
    div()
        .id(TAB_DRAG_PROXY_ID)
        .debug_selector(|| TAB_DRAG_PROXY_ID.to_string())
        .absolute()
        .left(px(rect[0]))
        .top(px(rect[1]))
        .w(px(rect[2]))
        .h(px(rect[3]))
        .child(surface)
        .children(crate::rect_probe::rect_probe(TAB_DRAG_PROXY_ID))
}

/// The §15 id of the blueprint D07 destination-profile label.
pub const DRAG_DESTINATION_ID: &str = "bbr-drag-destination";

/// Blueprint D07 (board P-7): while a drag targets a chip of another
/// profile, a label above the footer names the destination — the profile
/// chip plus "Move to Work · Profile Work" — before the drop recreates the
/// page. `material:overlay`, hairline + `elev:2`, 28 pt tall.
pub fn drag_destination_label(
    theme: &Theme,
    destination: &crate::state::DragDestination,
) -> Stateful<Div> {
    let p = theme.palette;
    let spec = crate::components::ProfileChipSpec {
        letter: destination.companion.letter.clone(),
        hue: destination.companion.hue,
    };
    div()
        .id(DRAG_DESTINATION_ID)
        .absolute()
        .left(crate::theme::Spacing::sp3())
        .bottom(theme.density.row() + crate::theme::Spacing::sp2())
        .h(px(28.0))
        .pl(px(4.0))
        .pr(px(10.0))
        .gap(px(6.0))
        .flex()
        .flex_row()
        .items_center()
        .rounded(px(8.0))
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(crate::theme::elevation::elev2(theme))
        .child(crate::components::profile_chip(
            theme,
            destination.companion.profile_index,
            &spec,
        ))
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink)
                .whitespace_nowrap()
                .child(destination.text.clone()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, ThemeMode};

    fn theme(density: Density) -> Theme {
        Theme::new(ThemeMode::Light, ContrastMode::Standard, density)
    }

    #[test]
    fn the_ghost_row_grows_the_rail_by_exactly_one_pitch() {
        for density in [Density::Small, Density::Normal, Density::ExtraLarge] {
            let theme = theme(density);
            assert_eq!(
                pinned_drop_row_height(&theme),
                f32::from(density.pitch()),
                "{density:?}"
            );
        }
    }

    #[test]
    fn activation_growth_counts_only_offers_above_the_source() {
        let theme = theme(Density::Normal);
        let pitch = f32::from(Density::Normal.pitch());
        let row = f32::from(Density::Normal.favorite_tile_height())
            + f32::from(Density::Normal.favorites_gap());
        // Zero favorites: the band keeps its geometry; partial rows: the
        // drop cell takes the free cell. Only the ghost row moves an
        // unpinned source, and nothing moves a pinned one.
        for favorites in [0, 1, 3, 5] {
            assert_eq!(activation_growth(&theme, favorites, true), pitch);
            assert_eq!(activation_growth(&theme, favorites, false), 0.0);
        }
        // A full last row: the drop cell opens one more row, once.
        for favorites in [4, 8] {
            assert_eq!(activation_growth(&theme, favorites, true), pitch + row);
            assert_eq!(activation_growth(&theme, favorites, false), row);
        }
    }

    #[test]
    fn the_proxy_keeps_the_press_point_under_the_pointer() {
        let theme = theme(Density::Normal);
        let rest = drag_proxy_rect(&theme, 260.0, 0.0, 400.0, 18.0);
        let moved = drag_proxy_rect(&theme, 260.0, 12.0, 480.0, 18.0);
        assert_eq!(rest[1], 382.0, "the press sits 18 pt into the proxy");
        assert_eq!(moved[0] - rest[0], 12.0, "horizontal travel follows");
        assert_eq!(moved[1] - rest[1], 80.0, "vertical travel follows");
        assert_eq!(rest[2], moved[2], "width is the rail row surface");
        assert_eq!(rest[3], f32::from(Density::Normal.row()));
    }

    #[test]
    fn a_chip_target_lifts_the_proxy_above_the_strip() {
        let theme = theme(Density::Normal);
        let over_chip = drag_proxy_rect(&theme, 300.0, 0.0, 1381.0, 18.0);
        let lifted = lift_proxy_above_strip(over_chip, 1365.0);
        assert_eq!(lifted[1] + lifted[3], 1361.0, "bottom 4 pt above the strip");
        assert_eq!(lifted[0], over_chip[0], "horizontal travel still follows");
        assert_eq!(lifted[2..], over_chip[2..], "same row surface");
        let high = drag_proxy_rect(&theme, 300.0, 0.0, 600.0, 18.0);
        assert_eq!(
            lift_proxy_above_strip(high, 1365.0),
            high,
            "far above: unchanged"
        );
    }

    #[test]
    fn offer_ids_are_the_design_contract() {
        assert_eq!(PINNED_DROP_ROW_ID, "bbr-pinned-drop-row");
        assert_eq!(TAB_DRAG_PROXY_ID, "bbr-tab-drag-proxy");
    }
}
