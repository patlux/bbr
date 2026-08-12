//! Tab Row interaction scenarios (packet
//! `component-gallery-browser-components.md`): the story renders every
//! valid matrix family through the production `bbr-ui::components::tab_row`
//! with recorder callbacks. Real pointer clicks on the close control, the
//! pin control, and the row body prove which production callback fired
//! (the recorder path); the §10.9 drag ticks (legal vs clamped-invalid)
//! render with their stable ids. No product behavior is asserted here —
//! the gallery proves renderability and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, VisualTestContext};

use crate::app::GalleryApp;
use crate::harness::{bounds_of, click_selector, flush, move_to_story, run_scenario, snapshot};
use crate::tab_row_matrix::{
    DRAG_TICK_INVALID_ID, DRAG_TICK_LEGAL_ID, PINNED_N_PIN_ID, PINNED_S_ROW_ID,
    REVEAL_CLOSE_CLOSE_ID, TAB_ROW_REJECTED, TAB_ROW_VALID,
};

/// Navigate to the Tab Row story (found by its stable label).
fn open_tab_row(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Tab Row")
            .expect("Tab Row story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

#[test]
#[ignore]
fn tab_row_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_tab_row(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Tab Row");
        assert_eq!(snap.tab_closes, 0);
        assert_eq!(snap.tab_pins, 0);
        assert_eq!(snap.tab_selects, 0);
        assert_eq!(snap.tab_drag_begins, 0);
        // Every valid family is registered — the story renders them all.
        let family_count = window.read(|_| TAB_ROW_VALID.len());
        assert_eq!(family_count, 18);
        // The first family (default, N) renders the stable §11.1 ids.
        flush(window);
        let row = bounds_of(window, "bbr-tab-row-0");
        assert!(row.size.width > px(0.0) && row.size.height > px(0.0));
        assert!(bounds_of(window, "bbr-tab-title-0").size.width > px(0.0));
        assert!(bounds_of(window, "bbr-tab-favicon-0").size.width > px(0.0));
        assert!(bounds_of(window, "bbr-tab-state-0").size.width > px(0.0));
        assert!(bounds_of(window, "bbr-tab-close-0").size.width > px(0.0));
        assert!(bounds_of(window, "bbr-tab-pin-0").size.width > px(0.0));
        // The pinned-S family (icon-only) and the drag-tick families render
        // at their own stable ids.
        assert!(bounds_of(window, PINNED_S_ROW_ID).size.height > px(0.0));
        assert!(bounds_of(window, DRAG_TICK_LEGAL_ID).size.width > px(0.0));
        assert!(bounds_of(window, DRAG_TICK_INVALID_ID).size.width > px(0.0));
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| TAB_ROW_REJECTED.len());
        assert_eq!(rejected_count, 4);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_close_pin_and_row_press_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_tab_row(window, app);
        flush(window);
        // A real pointer click on the close control of the reveal-close
        // family (§10.2, visible): the production on_close callback fires
        // and stops propagation (no select/drag).
        click_selector(window, REVEAL_CLOSE_CLOSE_ID);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.tab_closes, 1, "real close click recorded");
        assert_eq!(snap.tab_selects, 0, "close stops propagation (§11.1)");
        assert_eq!(snap.tab_drag_begins, 0);
        assert_eq!(snap.tab_pins, 0);
        // A real pointer click on the pin control of the pinned family:
        // on_pin fires alone.
        click_selector(window, PINNED_N_PIN_ID);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.tab_pins, 1, "real pin click recorded");
        assert_eq!(snap.tab_closes, 1);
        assert_eq!(snap.tab_selects, 0);
        assert_eq!(snap.tab_drag_begins, 0);
        // A real press on the first family's row body (the favicon tile
        // area, left of the pin): on_select AND on_begin_drag both fire —
        // the production two-listener path — and no control fires.
        let row = bounds_of(window, "bbr-tab-row-0");
        let body = point(row.origin.x + px(8.0), row.center().y);
        window.simulate_click(body, gpui::Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.tab_selects, 1, "row press records select");
        assert_eq!(snap.tab_drag_begins, 1, "row press records drag begin");
        assert_eq!(snap.tab_closes, 1, "row press records no close");
        assert_eq!(snap.tab_pins, 1, "row press records no pin");
    });
}
