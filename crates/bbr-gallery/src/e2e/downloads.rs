//! Downloads panel interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 5): the story renders
//! every valid §11.12 matrix family through the production
//! `bbr_ui::surfaces::downloads` surface (`downloads_panel_surface`) with
//! recorder callbacks. Real pointer hover/press on a row, a real press on
//! the contextual action, the §17.8 Delete control, and the §17.7 Clear
//! control prove which production callback fired (the recorder path); the
//! bounded `>6-row` scroll window provably renders only its window slice.
//! No product behavior is asserted here — the gallery proves renderability
//! and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, size, Entity, Modifiers, MouseButton, VisualTestContext};

use crate::app::GalleryApp;
use crate::downloads_matrix::{
    CANCELLED_FAMILY_INDEX, DOWNLOAD_FAMILY_SELECTOR, DOWNLOAD_REJECTED, DOWNLOAD_VALID,
    DOWNLOAD_WASH_FAMILY_SELECTOR, DOWNLOAD_WASH_VALID,
};
use crate::downloads_panels::{
    scroll_window_start, DOWNLOAD_PANEL_FAMILY_SELECTOR, PANEL_VALID, SCROLL_FAMILY_ANCHOR,
    SCROLL_FAMILY_ROWS,
};
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};

/// Navigate to the Downloads panel story by its stable label.
fn open_downloads(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Downloads panel")
            .expect("downloads panel story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

/// The stable `bbr-download-row-<n>` id of the scroll family's row `n`.
/// The family's rows sit at list positions `0..SCROLL_FAMILY_ROWS`, and
/// positions ≥ 7 are painted by no other element on the story page, so
/// the bounded-window E2E can observe the window rows and the missing
/// before-window row unambiguously.
fn scroll_row_id(row: usize) -> &'static str {
    static_selector(format!("bbr-download-row-{row}"))
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[test]
#[ignore]
fn downloads_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_downloads(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Downloads panel");
        assert_eq!(snap.downloads.hovers, Vec::<(usize, bool)>::new());
        assert_eq!(snap.downloads.presses, Vec::<(usize, bool)>::new());
        assert_eq!(snap.downloads.actions, Vec::<usize>::new());
        assert_eq!(snap.downloads.deletes, Vec::<u64>::new());
        assert_eq!(snap.downloads.clears, 0);
        assert_eq!(snap.downloads.inside_presses, 0);
        assert_eq!(snap.downloads.key_downs, 0);
        // Every valid row family is registered and renders through the
        // production surface (each family wrapper has non-empty bounds).
        let row_count = window.read(|_| DOWNLOAD_VALID.len());
        assert_eq!(row_count, 7);
        for index in 0..row_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{DOWNLOAD_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "row family {index} renders"
            );
        }
        // The wash families render their production rows in-window: each
        // family's row (`bbr-download-row-<index>`) must be laid out inside
        // that family's own wrapper — not merely somewhere on the page. The
        // row families and panel families share the same
        // `bbr-download-row-<n>` id space, so a global bounds check would
        // pass even if a wash panel were empty (a sibling family at the
        // same index would satisfy the selector); the production window
        // filter `model.window.contains(&row.index)` must include every
        // wash enumerate index, and the story renders the washes after the
        // panels so each wash row is the last painted instance of its id
        // and resolves to its own wrapper.
        let wash_count = window.read(|_| DOWNLOAD_WASH_VALID.len());
        assert_eq!(wash_count, 2);
        for index in 0..wash_count {
            let wrapper = bounds_of(
                window,
                static_selector(format!("{DOWNLOAD_WASH_FAMILY_SELECTOR}{index}")),
            );
            let row = bounds_of(window, static_selector(format!("bbr-download-row-{index}")));
            assert!(
                row.is_contained_within(&wrapper),
                "wash family {index} renders its production row in-window (§10.2/§10.3)"
            );
        }
        // The five panel families render.
        let panel_count = window.read(|_| PANEL_VALID.len());
        assert_eq!(panel_count, 5);
        for index in 0..panel_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{DOWNLOAD_PANEL_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "panel family {index} renders"
            );
        }
        // The empty family renders the production empty hint.
        assert!(
            window.debug_bounds("bbr-downloads-empty").is_some(),
            "the empty panel family renders the production empty hint"
        );
        // The Active+History family renders both group labels.
        assert!(
            window.debug_bounds("bbr-downloads-active").is_some(),
            "the both-groups family renders the Active label"
        );
        assert!(
            window.debug_bounds("bbr-downloads-history").is_some(),
            "the both-groups family renders the History label"
        );
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| DOWNLOAD_REJECTED.len());
        assert_eq!(rejected_count, 4);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn the_bounded_scroll_window_renders_only_its_window_slice() {
    run_scenario(|window, app| {
        open_downloads(window, app);
        flush(window);
        // The scroll family owns 22 history rows at list positions 0..21
        // at a scroll anchor of 8; the production window clamp derives
        // the rendered slice 8..22. Only the window's rows have bounds —
        // the rows before the window are never laid out (the §11.6
        // bounded window, never the full history).
        let start = scroll_window_start();
        assert!(start > 0, "the scroll family is scrolled off the top");
        assert!(
            start < SCROLL_FAMILY_ROWS - 1,
            "the window is a true slice of the {SCROLL_FAMILY_ROWS}-row list"
        );
        // The window's rows render through the production surface: the
        // first window row, a middle row, and the last list row all have
        // bounds (positions ≥ 7 are scroll-unique, so these ids resolve
        // to the scroll family's rows).
        let last = SCROLL_FAMILY_ROWS - 1;
        assert!(window.debug_bounds(scroll_row_id(start)).is_some());
        assert!(window.debug_bounds(scroll_row_id(start + 3)).is_some());
        assert!(window.debug_bounds(scroll_row_id(last)).is_some());
        // The bounded window never lays out rows before it: the
        // scroll-unique row just before the window has no bounds (§11.6
        // bounded window — the top spacer absorbs it, it is never a row).
        assert!(
            window.debug_bounds(scroll_row_id(start - 1)).is_none(),
            "row {} is outside the bounded window and must not render",
            start - 1
        );
        // The scroll anchor const is consistent with the fixture.
        assert_eq!(SCROLL_FAMILY_ANCHOR, 8);
    });
}

#[test]
#[ignore]
fn real_pointer_interactions_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_downloads(window, app);
        // The downloads story page is a tall scrollable catalog (7 row
        // families + 2 washes + 5 panel families + rejected rows); the
        // §17.7 Clear control of the last history panel sits below the
        // default test-window fold. Make the whole page genuinely
        // on-window before the real pointer interactions, so every
        // production target (the cancelled row, its Restart/Delete
        // controls, and the panel Clear) is visible and hit-testable.
        window.simulate_resize(size(px(1920.0), px(4000.0)));
        flush(window);
        // The cancelled row family (index 6) renders the §17.8 Restart +
        // Delete pair: the row body, the action slot, and the Delete
        // control all live inside the production row at list position 6
        // (the family's row keeps its declaration index, so
        // `bbr-download-row/-action/-delete-6` resolve to this family's
        // row — the scroll family never renders position 6).
        let row = bounds_of(
            window,
            static_selector(format!("bbr-download-row-{CANCELLED_FAMILY_INDEX}")),
        );
        assert!(row.size.width > px(0.0) && row.size.height > px(0.0));
        // A real pointer move onto the row body: the production
        // `on_row_hover` callback fires with `true` (§10.2 wash follows
        // the pointer).
        let body = point(row.origin.x + px(30.0), row.center().y);
        window.simulate_mouse_move(body, None, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.downloads.hovers,
            vec![(CANCELLED_FAMILY_INDEX, true)],
            "real pointer move records the hover entry (§10.2)"
        );
        assert_eq!(snap.downloads.presses, Vec::<(usize, bool)>::new());
        assert_eq!(snap.downloads.actions, Vec::<usize>::new());
        // A real pointer press on the row body: the production `on_row_press`
        // fires `(index, true)`; the press bubbles to the panel root so the
        // §13 inside-press marker also fires (the app's outside-press handler
        // must never dismiss the panel).
        window.simulate_mouse_down(body, MouseButton::Left, Modifiers::none());
        window.simulate_mouse_up(body, MouseButton::Left, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.downloads.presses,
            vec![
                (CANCELLED_FAMILY_INDEX, true),
                (CANCELLED_FAMILY_INDEX, false)
            ],
            "real press records the pressed wash (§10.3)"
        );
        assert_eq!(snap.downloads.inside_presses, 1, "the §13 marker fires");
        assert_eq!(snap.downloads.actions, Vec::<usize>::new());
        // A real pointer click on the contextual action slot: the production
        // `on_row_action` fires with the panel index and stops propagation
        // at the surface (no inside-press marker for an action press).
        let action = bounds_of(
            window,
            static_selector(format!("bbr-download-action-{CANCELLED_FAMILY_INDEX}")),
        );
        window.simulate_click(action.center(), Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.downloads.actions, vec![CANCELLED_FAMILY_INDEX]);
        assert_eq!(
            snap.downloads.inside_presses, 1,
            "the action activation stops propagation (§11.12)"
        );
        // A real pointer click on the §17.8 Delete control: `on_row_delete`
        // fires with the attempt id (and stops propagation).
        let delete = bounds_of(
            window,
            static_selector(format!("bbr-download-delete-{CANCELLED_FAMILY_INDEX}")),
        );
        window.simulate_click(delete.center(), Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.downloads.deletes,
            vec![crate::downloads_matrix::FIXTURE_ID_BASE + CANCELLED_FAMILY_INDEX as u64],
            "real delete click records the attempt id (§17.8)"
        );
        assert_eq!(
            snap.downloads.inside_presses, 1,
            "the delete activation stops propagation (§11.12)"
        );
        // A real pointer click on the §17.7 Clear control: `on_clear` fires
        // (the History family renders the Clear control below the scroll
        // area) and stops propagation.
        let clear = bounds_of(window, "bbr-downloads-clear");
        window.simulate_click(clear.center(), Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.downloads.clears, 1, "real clear click recorded");
        assert_eq!(
            snap.downloads.inside_presses, 1,
            "the clear activation stops propagation (§11.12)"
        );
    });
}
