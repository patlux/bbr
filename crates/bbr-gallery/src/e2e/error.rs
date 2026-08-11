//! Error/Crash interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 7): the story renders
//! every valid §11.8 matrix family through the production
//! `bbr_ui::surfaces::error` surface and the production
//! `bbr_ui::components::pane` panels with recorder callbacks. Real pointer
//! presses on the surface's action/dismiss buttons and the panes'
//! reload/close/new-tab buttons prove the production callbacks fire (the
//! recorder path); the every-story sweep covers the two new stories
//! automatically. No product behavior is asserted here — the gallery
//! proves renderability and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::app::GalleryApp;
use crate::error_matrix::{ERROR_REJECTED, ERROR_VALID};
use crate::error_panel_matrix::PANEL_REJECTED;
use crate::harness::{bounds_of, click_selector, flush, move_to_story, run_scenario, snapshot};
use crate::stories::error::ERROR_FAMILY_SELECTOR;

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

/// The §11.8 action/dismiss button centers of the first matrix family:
/// the gallery wraps each family in a test-target container whose first
/// child is a fixed-height frame holding the production surface. The
/// surface content is vertically centered in the frame, so the action row
/// (the last content row) sits a fixed offset above the frame's bottom
/// edge; the action button is the left half of the centered row and the
/// dismiss button the right half. The production §15 ids repeat per
/// family, so `debug_bounds` cannot target one family's button — the
/// frame test-target is the stable click anchor (the same pattern the
/// Address story uses for its pill center).
fn first_action_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let frame = bounds_of(
        window,
        static_selector(format!("{ERROR_FAMILY_SELECTOR}0-frame")),
    );
    point(
        frame.center().x - px(50.0),
        frame.origin.y + frame.size.height - px(50.0),
    )
}

/// The dismiss button center of the first matrix family (the right half
/// of the centered action row — see [`first_action_center`]).
fn first_dismiss_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let frame = bounds_of(
        window,
        static_selector(format!("{ERROR_FAMILY_SELECTOR}0-frame")),
    );
    point(
        frame.center().x + px(55.0),
        frame.origin.y + frame.size.height - px(50.0),
    )
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

/// The production §15 ids the Error surface story renders (each matrix
/// family repeats the ids; `debug_bounds` resolves the repeated id to one
/// instance — the tests only assert presence, never a specific family).
const ERROR_SURFACE_IDS: [&str; 6] = [
    "bbr-error-panel",
    "bbr-error-title",
    "bbr-error-copy",
    "bbr-error-url",
    "bbr-error-action",
    "bbr-error-dismiss",
];

/// The production §15 ids the Crash / Empty panes story renders.
const PANES_IDS: [&str; 5] = [
    "bbr-crash-panel",
    "bbr-crash-reload",
    "bbr-crash-close",
    "bbr-empty-new-tab",
    "bbr-empty-new-tab-button",
];

#[test]
#[ignore]
fn error_stories_render_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_story(window, app, "Error surface");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Error surface");
        assert_eq!(snap.error.actions, 0);
        assert_eq!(snap.error.dismissals, 0);
        // Every valid error family is registered and renders through the
        // production surface (each family wrapper has non-empty bounds).
        let family_count = window.read(|_| ERROR_VALID.len());
        assert_eq!(family_count, 16);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{ERROR_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "error family {index} renders"
            );
        }
        // The production §15 ids render through the first family.
        for id in ERROR_SURFACE_IDS {
            assert!(
                window.debug_bounds(id).is_some(),
                "production error id {id} renders"
            );
        }
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| ERROR_REJECTED.len());
        assert_eq!(rejected_count, 2);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);

        // The Crash / Empty panes story renders the §11.8 glass panes
        // through the production pane components.
        open_story(window, app, "Crash / Empty panes");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Crash / Empty panes");
        for id in PANES_IDS {
            assert!(
                window.debug_bounds(id).is_some(),
                "production pane id {id} renders"
            );
        }
        let rejected_panels = window.read(|_| PANEL_REJECTED.len());
        assert_eq!(rejected_panels, 1);
    });
}

#[test]
#[ignore]
fn real_presses_report_through_the_recorder() {
    run_scenario(|window, app| {
        // The error surface: a real pointer press on the primary action
        // button fires the production `on_action`; the dismiss button
        // fires `on_dismiss` (the recorder path). The production §15 ids
        // repeat per matrix family, so the press targets the first
        // family's buttons through the frame test-target anchor.
        open_story(window, app, "Error surface");
        flush(window);
        let action = first_action_center(window);
        window.simulate_click(action, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.error.actions, 1, "real press records on_action");
        assert_eq!(snap.error.dismissals, 0, "an action never dismisses");
        let dismiss = first_dismiss_center(window);
        window.simulate_click(dismiss, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.error.dismissals, 1, "real press records on_dismiss");
        assert_eq!(snap.error.actions, 1, "dismiss never fires an action");

        // The panes: crash panel Reload/Close and empty pane New Tab ⌘T.
        open_story(window, app, "Crash / Empty panes");
        flush(window);
        click_selector(window, "bbr-crash-reload");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.error.reloads, 1, "real press records on_reload");
        assert_eq!(snap.error.closes, 0, "reload never closes");
        click_selector(window, "bbr-crash-close");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.error.closes, 1, "real press records on_close");
        click_selector(window, "bbr-empty-new-tab-button");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.error.new_tabs, 1, "real press records on_new_tab");
    });
}
