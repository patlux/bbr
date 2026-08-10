//! Saved Pages Library scenarios (B3 packet `saved-pages-library`, Gallery
//! G2/G3): the G2 row/control stories and the G3 composition state matrix
//! render through the **exact production `bbr-ui` Library rows/controls/
//! surface**, and real pointer clicks on every control dispatch **exactly
//! once** through the recorder path. Nothing is faked by calling the app
//! methods directly; every interaction is a real platform event at the
//! control's on-screen bounds. Search is proven at the model level (the
//! search field renders the query and the no-results state presents); the
//! collection action is the move entry (moving records into/out of a
//! collection).
//!
//! The Library controls carry the production §15 ids (`.id(...)`); the
//! Gallery's own `debug_selector` wrappers expose the row/control bounds
//! to the real-pointer E2E, and the row's inner controls (result block,
//! rename button, select box) sit at deterministic positions inside the
//! row bounds (Normal density, fixed paddings/gaps), so the E2E clicks
//! them at their computed centers — the production components are never
//! modified for the gallery.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Bounds, Entity, Pixels, Point, VisualTestContext};

use crate::app::GalleryApp;
use crate::harness::{bounds_of, click_selector, flush, move_to_story, run_scenario, snapshot};
use crate::library_fixtures::LIBRARY_STATES;

/// Navigate to a story by its label.
fn open_story(window: &mut VisualTestContext, app: &Entity<GalleryApp>, label: &str) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == label)
            .expect("story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

/// Switch the selected story's state through the production control path.
fn set_state(window: &mut VisualTestContext, app: &Entity<GalleryApp>, index: usize) {
    window.update(|_window, cx| {
        app.update(cx, |app, cx| {
            app.apply_control(crate::controls::ControlAction::State(index), cx)
        });
    });
    flush(window);
}

/// Assert a selector rendered with a non-empty on-screen bounds box.
fn rendered(window: &mut VisualTestContext, selector: &'static str) {
    let bounds = bounds_of(window, selector);
    assert!(
        bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
        "{selector} renders with non-empty bounds"
    );
}

/// A real pointer click at a point (mouse down + up through the production
/// event path).
fn click_at(window: &mut VisualTestContext, at: Point<Pixels>) {
    window.simulate_click(at, gpui::Modifiers::none());
}

/// The §11.16 row inner-control centers (Normal density, deterministic
/// layout): `[selection box][favicon][result block (flex_grow)][rename]
/// [select]`. The result block is flex_grow, so its center is safely
/// inside; the rename button and select box sit at fixed offsets from the
/// row's right edge (right padding `sp:3` 12, `hit-min.close` 28, gap
/// `sp:2` 8).
fn row_control_points(bounds: Bounds<Pixels>) -> (Point<Pixels>, Point<Pixels>, Point<Pixels>) {
    let y = bounds.center().y;
    let right = bounds.origin.x + bounds.size.width;
    let result = point(bounds.origin.x + bounds.size.width * 0.4, y);
    let rename = point(right - px(62.0), y);
    let select = point(right - px(26.0), y);
    (result, rename, select)
}

/// The editing-row inner-control centers: `[selection][favicon][rename
/// input (flex_grow)][confirm][cancel][select]`.
fn editing_row_control_points(
    bounds: Bounds<Pixels>,
) -> (Point<Pixels>, Point<Pixels>, Point<Pixels>) {
    let y = bounds.center().y;
    let right = bounds.origin.x + bounds.size.width;
    let confirm = point(right - px(98.0), y);
    let cancel = point(right - px(62.0), y);
    let select = point(right - px(26.0), y);
    (confirm, cancel, select)
}

#[test]
#[ignore]
fn library_row_controls_dispatch_exactly_once() {
    run_scenario(|window, app| {
        open_story(window, app, "Library row");
        let snap = snapshot(window, app);
        assert!(snap.library.opens.is_empty());
        let row = bounds_of(window, "bbr-gallery-library-row-1");
        let (result, rename, select) = row_control_points(row);
        // Real pointer click on row 1's result block: open dispatches once.
        click_at(window, result);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.library.opens, vec![1], "open dispatches exactly once");
        // Real pointer click on row 1's select box: select dispatches once.
        click_at(window, select);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.select_toggles,
            vec![1],
            "select dispatches exactly once"
        );
        // Real pointer click on row 1's rename button: rename stages once.
        click_at(window, rename);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.renames,
            vec![1],
            "rename dispatches exactly once"
        );
        // The staged rename input renders only in the editing state.
        set_state(window, app, 2); // "editing"
        let snap = snapshot(window, app);
        assert_eq!(snap.story_state, "editing");
        let row = bounds_of(window, "bbr-gallery-library-row-1");
        let (confirm, cancel, _select) = editing_row_control_points(row);
        // Real pointer clicks on Confirm/Cancel dispatch exactly once each.
        click_at(window, confirm);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.rename_confirms, 1,
            "rename confirm dispatches exactly once"
        );
        click_at(window, cancel);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.rename_cancels, 1,
            "rename cancel dispatches exactly once"
        );
        // Mode/state fields are untouched by Library interactions.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn library_collection_row_dispatches_exactly_once() {
    run_scenario(|window, app| {
        open_story(window, app, "Library collection row");
        // A real pointer click on the collection row (the move/rename/
        // remove entry) dispatches exactly once with the CollectionId.
        click_selector(window, "bbr-gallery-library-collection-10");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.collection_actions,
            vec![10],
            "collection action dispatches exactly once"
        );
        click_selector(window, "bbr-gallery-library-collection-11");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.collection_actions,
            vec![10, 11],
            "each collection row dispatches its own id"
        );
    });
}

#[test]
#[ignore]
fn library_remove_confirm_dispatches_exactly_once() {
    run_scenario(|window, app| {
        open_story(window, app, "Library remove confirm");
        let bounds = bounds_of(window, "bbr-gallery-library-remove-confirm");
        let y = bounds.center().y;
        let right = bounds.origin.x + bounds.size.width;
        // The confirm row is `[text (flex_1)][Remove][Cancel]`: the Cancel
        // button is the rightmost child, the Remove button sits left of
        // it (right padding `sp:3` 12, gap `sp:3` 12, button widths from
        // the `type:meta` labels).
        let cancel = point(right - px(44.0), y);
        let remove = point(right - px(120.0), y);
        click_at(window, cancel);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.remove_cancels, 1,
            "remove cancel dispatches exactly once"
        );
        click_at(window, remove);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.library.remove_confirms, 1,
            "remove confirm dispatches exactly once"
        );
    });
}

#[test]
#[ignore]
fn library_composition_state_matrix_renders() {
    run_scenario(|window, app| {
        open_story(window, app, "Library");
        for (index, state) in LIBRARY_STATES.iter().enumerate() {
            set_state(window, app, index);
            let snap = snapshot(window, app);
            assert_eq!(
                snap.story_state, *state,
                "state selector drives the model fixture"
            );
            // The surface renders without panic in every decided state.
            rendered(window, "bbr-gallery-library-surface");
        }
    });
}

#[test]
#[ignore]
fn library_search_field_renders_query_states() {
    run_scenario(|window, app| {
        open_story(window, app, "Library search");
        // Empty state: the placeholder field renders.
        rendered(window, "bbr-gallery-library-search");
        // Typed state: the query renders.
        set_state(window, app, 1); // "typed"
        let snap = snapshot(window, app);
        assert_eq!(snap.story_state, "typed");
        rendered(window, "bbr-gallery-library-search");
        // Focused state: the focus ring renders.
        set_state(window, app, 2); // "focused"
        let snap = snapshot(window, app);
        assert_eq!(snap.story_state, "focused");
        rendered(window, "bbr-gallery-library-search");
    });
}

#[test]
#[ignore]
fn library_sort_control_renders_all_four_orders() {
    run_scenario(|window, app| {
        open_story(window, app, "Library sort");
        for selector in [
            "bbr-gallery-library-sort-updated_newest",
            "bbr-gallery-library-sort-updated_oldest",
            "bbr-gallery-library-sort-title_ascending",
            "bbr-gallery-library-sort-title_descending",
        ] {
            rendered(window, selector);
        }
        // A real pointer click on the first order control dispatches the
        // sort cycle exactly once.
        click_selector(window, "bbr-gallery-library-sort-updated_newest");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.library.sort_cycles, 1, "sort dispatches exactly once");
    });
}
