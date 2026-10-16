//! The Downloads panel **panel-level** matrix authority (packet
//! `component-gallery-browser-components.md`, Group 5): the §11.12 panel
//! composition families — Active/History groups, the empty hint, the
//! bounded `>6-row` scroll window, and the keyboard cursor focus — as the
//! single declaration the story renders. The row families and the
//! rejected-combination authority live in the sibling
//! [`crate::downloads_matrix`] module; together the two modules are the
//! one matrix declaration source split by responsibility (row vs panel),
//! so the story, the fixture builders, the matrix tables, and the tests
//! cannot drift.

use bbr_ui::state::DownloadVisual;
use bbr_ui::surfaces::downloads::{
    DownloadGroupKind, DownloadRowModel, PanelLayout, DOWNLOADS_MAX_VISIBLE, DOWNLOADS_WINDOW_ROWS,
};
use bbr_ui::theme::Theme;
use gpui::{point, px};

use crate::downloads_matrix::{build_download_fixture, DOWNLOAD_VALID};

/// A panel-level matrix family: the §11.12 panel composition (multiple
/// rows, group labels, the empty hint, the bounded scroll window, the
/// keyboard cursor focus). `rows` are declaration indices into
/// [`DOWNLOAD_VALID`] (active rows first, then history rows — the
/// production order). Each panel renders its own production surface, so
/// its rows use **list positions** (`0..rows.len()`) exactly like
/// `bbr-app` does — the production `downloads_panel_surface` derives the
/// bounded window and the spacer geometry from the same list-position
/// index space. The gallery story therefore shares the
/// `bbr-download-row-<n>` id space across its single-row families and
/// panels (every panel is an independent production surface); the E2E
/// targets the scroll family's scroll-unique rows (positions ≥ 7) and
/// the cancelled family's row (position 6), which no other family paints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelCase {
    pub label: &'static str,
    pub rows: &'static [usize],
    /// §9: whether the panel owns chrome focus (the ring + the keyboard
    /// cursor row).
    pub focused: bool,
    /// The keyboard-cursor row (rendered `DownloadVisual::Focused`).
    pub cursor: Option<usize>,
    /// The scroll anchor in rows (0 = top; the window derives from it,
    /// clamped to the deterministic list).
    pub scroll_anchor: usize,
}

/// Every valid panel family — the single declaration the story renders.
/// The `>6-row` family carries 22 history rows so the bounded window
/// (visible 6 + §11.6 overscan) provably renders only a slice. Its rows
/// sit at list positions `0..22`, so the window (anchored at 8) renders
/// positions 8..21 and never lays out 0..7 — and position 7 (the row
/// just before the window) is scroll-unique, which is exactly what lets
/// the E2E prove the bounded slice (`bbr-download-row-7` has no bounds
/// while the window rows 8..21 do).
#[rustfmt::skip]
pub const PANEL_VALID: &[PanelCase] = &[
    PanelCase { label: "Active + History groups", rows: &[0, 1, 2, 3, 5, 6], focused: false, cursor: None, scroll_anchor: 0 },
    PanelCase { label: "History-only (empty Active hides its label)", rows: &[3, 5, 6], focused: false, cursor: None, scroll_anchor: 0 },
    PanelCase { label: "Empty hint", rows: &[], focused: false, cursor: None, scroll_anchor: 0 },
    // 22 history rows (repeated succeeded/failed/cancelled families with
    // deterministic names) at a mid scroll anchor: only the bounded window
    // renders. The anchor (8) keeps the row just before the window (7)
    // scroll-unique, so the E2E can prove the slice on both sides.
    PanelCase { label: ">6-row scroll window (bounded)", rows: &[3, 5, 6, 3, 5, 6, 3, 5, 6, 3, 5, 6, 3, 5, 6, 3, 5, 6, 3, 5, 6, 3], focused: false, cursor: None, scroll_anchor: 8 },
    PanelCase { label: "Keyboard cursor focus", rows: &[0, 2, 3], focused: true, cursor: Some(1), scroll_anchor: 0 },
];

/// The panel fixture: the deterministic rows, the layout, the bounded
/// window, and the applied scroll offset the story passes to the surface.
pub struct PanelFixture {
    pub rows: Vec<DownloadRowModel>,
    pub layout: PanelLayout,
    pub window: std::ops::Range<usize>,
    /// The scroll offset applied to the production `ScrollHandle` so the
    /// window matches the scroll position (the app derives the window from
    /// exactly this handle — one authority).
    pub scroll_offset: gpui::Point<gpui::Pixels>,
    pub viewport_rows: usize,
}

/// Build the panel fixture for a panel family: rows mapped from the
/// declared valid families at the family's unique index base, the
/// token-derived layout, the bounded window from the scroll anchor (the
/// production clamp), and the matching scroll offset.
pub fn build_panel_fixture(case: &PanelCase, theme: &Theme) -> PanelFixture {
    let rows: Vec<DownloadRowModel> = case
        .rows
        .iter()
        .enumerate()
        .map(|(offset, &family)| {
            let source = &DOWNLOAD_VALID[family];
            // The row index is the list position (0..rows.len()) — the
            // same index space the production app uses and the space the
            // production surface's bounded-window filter and spacer math
            // expect. A per-panel offset base would break the filter (no
            // row would ever be in the window).
            let index = offset;
            let mut model = build_download_fixture(source, index).expect("valid row family builds");
            // A repeated family in the scroll panel gets a deterministic
            // name so every row stays distinguishable.
            if case.rows.len() > 6 {
                model.filename = format!("{}-{offset}.bin", model.filename);
            }
            if case.cursor == Some(offset) {
                model.visual = DownloadVisual::Focused;
            }
            model
        })
        .collect();
    let active_count = rows
        .iter()
        .filter(|r| r.group == DownloadGroupKind::Active)
        .count();
    let history_count = rows.len() - active_count;
    let layout = PanelLayout::for_theme(theme, active_count, history_count);
    let total = layout.total();
    let anchor = total
        .saturating_sub(DOWNLOADS_WINDOW_ROWS)
        .min(case.scroll_anchor);
    let end = (anchor + DOWNLOADS_WINDOW_ROWS).min(total);
    let window = anchor..end;
    let scroll_offset = point(px(0.0), px(-layout.row_offset(anchor)));
    let viewport_rows = DOWNLOADS_MAX_VISIBLE.min(total);
    PanelFixture {
        rows,
        layout,
        window,
        scroll_offset,
        viewport_rows,
    }
}

/// The stable gallery test-target selector for the panel families
/// (`download-panel-family-<n>` in declaration order) — test targets only,
/// never product ids.
pub const DOWNLOAD_PANEL_FAMILY_SELECTOR: &str = "download-panel-family-";

/// The scroll family's deterministic row count (22 rows at list positions
/// 0..21 — the bounded window renders exactly `DOWNLOADS_WINDOW_ROWS` of
/// them).
pub const SCROLL_FAMILY_ROWS: usize = 22;
/// The scroll family's scroll anchor (the window starts here; with 22
/// rows the production clamp keeps the start at 8, so the row just before
/// the window — position 7 — is scroll-unique and proves the slice).
pub const SCROLL_FAMILY_ANCHOR: usize = 8;
/// The scroll family's first rendered window index (anchor clamped to
/// `total − WINDOW_ROWS`, then the window's start).
pub fn scroll_window_start() -> usize {
    SCROLL_FAMILY_ROWS
        .saturating_sub(DOWNLOADS_WINDOW_ROWS)
        .min(SCROLL_FAMILY_ANCHOR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::theme::{ContrastMode, Density, ThemeMode};

    fn fixture_theme() -> Theme {
        Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    #[test]
    fn scroll_family_proves_the_bounded_window() {
        let theme = fixture_theme();
        let scroll = PANEL_VALID
            .iter()
            .find(|c| c.label.starts_with(">6-row"))
            .expect("scroll family");
        let fixture = build_panel_fixture(scroll, &theme);
        assert_eq!(fixture.rows.len(), SCROLL_FAMILY_ROWS);
        // The window is a bounded slice, not the full list.
        assert!(fixture.window.len() <= DOWNLOADS_WINDOW_ROWS);
        assert!(fixture.window.len() < SCROLL_FAMILY_ROWS);
        assert_eq!(fixture.window.start, scroll_window_start());
        assert_eq!(fixture.viewport_rows, DOWNLOADS_MAX_VISIBLE);
        // The scroll offset matches the anchor so the window and the wheel
        // never disagree.
        assert!(fixture.scroll_offset.y < px(0.0));
        assert_eq!(
            fixture
                .layout
                .anchor_at(-f32::from(fixture.scroll_offset.y)),
            fixture.window.start
        );
        // A concrete scroll handle derived from the fixture window returns
        // the same anchor (production consistency).
        let handle = gpui::ScrollHandle::new();
        handle.set_offset(fixture.scroll_offset);
        assert_eq!(
            fixture.layout.anchor_at(-f32::from(handle.offset().y)),
            fixture.window.start
        );
    }

    #[test]
    fn panel_families_cover_groups_empty_and_cursor() {
        let theme = fixture_theme();
        let by_label = |label: &str| {
            PANEL_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        // Active + History: both groups present (labels + hairline render).
        let both = build_panel_fixture(by_label("Active + History groups"), &theme);
        assert!(both.layout.has_active());
        assert!(both.layout.has_history());
        assert_eq!(both.layout.active_count, 3);
        assert_eq!(both.layout.history_count, 3);
        // History-only: the active group is empty (its label is hidden).
        let history_only = build_panel_fixture(
            by_label("History-only (empty Active hides its label)"),
            &theme,
        );
        assert!(!history_only.layout.has_active());
        assert!(history_only.layout.has_history());
        // Empty: no rows, empty window, no groups.
        let empty = build_panel_fixture(by_label("Empty hint"), &theme);
        assert!(empty.rows.is_empty());
        assert_eq!(empty.window, 0..0);
        assert_eq!(empty.layout.total(), 0);
        // Keyboard cursor focus: the cursor row renders the focused visual
        // and the model reports the §9 ring.
        let cursor = build_panel_fixture(by_label("Keyboard cursor focus"), &theme);
        assert!(cursor
            .rows
            .iter()
            .any(|r| r.visual == DownloadVisual::Focused));
    }
}
