//! Find-bar interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 6): the story renders
//! every valid §11.4 matrix family through the production
//! `bbr_ui::surfaces::find` symbols (`find_bar`, `FindSurface`,
//! `FindBarRect`) with recorder callbacks. Real pointer presses on the
//! previous/next/close controls prove which production callback fired (the
//! recorder path); the every-story sweep covers the new story
//! automatically. No product behavior is asserted here — the gallery
//! proves renderability and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, Theme, ThemeMode};
use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::app::GalleryApp;
use crate::find_matrix::{FIND_FAMILY_WIDTH, FIND_REJECTED, FIND_VALID};
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::stories::find::FIND_FAMILY_SELECTOR;

/// Navigate to the Find bar story by its stable label.
fn open_find_bar(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Find bar")
            .expect("Find bar story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[test]
#[ignore]
fn find_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_find_bar(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Find bar");
        assert_eq!(snap.find.previous, 0);
        assert_eq!(snap.find.next, 0);
        assert_eq!(snap.find.closes, 0);
        // Every valid family is registered — the story renders them all
        // through the production component (each family wrapper has
        // non-empty bounds).
        let family_count = window.read(|_| FIND_VALID.len());
        assert_eq!(family_count, 6);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{FIND_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "find family {index} renders"
            );
        }
        // The production component renders its stable §15 ids.
        assert!(window.debug_bounds("bbr-find-bar").is_some());
        assert!(window.debug_bounds("bbr-find-field").is_some());
        assert!(window.debug_bounds("bbr-find-text").is_some());
        assert!(window.debug_bounds("bbr-find-count").is_some());
        assert!(window.debug_bounds("bbr-find-previous").is_some());
        assert!(window.debug_bounds("bbr-find-next").is_some());
        assert!(window.debug_bounds("bbr-find-close").is_some());
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| FIND_REJECTED.len());
        assert_eq!(rejected_count, 2);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

/// The bar bounds of the first family: the gallery wraps each family in a
/// test-target container whose first child is the bar at the §11.4 compact
/// clamp width ([`FIND_FAMILY_WIDTH`]) and `density:row` height, so the
/// bar's bounds derive from the wrapper's origin plus the settled bar
/// size (the wrapper itself may be wider than the bar because the meta
/// line below can exceed 360 pt, and taller because of that line).
fn first_family_bar_bounds(window: &mut VisualTestContext) -> (gpui::Point<gpui::Pixels>, f32) {
    let family = bounds_of(window, static_selector(format!("{FIND_FAMILY_SELECTOR}0")));
    let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let row = f32::from(theme.density.row());
    let bar_left = family.origin.x;
    let bar_right = f32::from(family.origin.x) + FIND_FAMILY_WIDTH;
    let bar_center_y = family.origin.y + px(row / 2.0);
    (point(bar_left, bar_center_y), bar_right)
}

/// The center of the first family's "next" control, derived from the
/// production layout tokens (the §11.4 bar: `px:sp2`, gap `sp1`, the
/// `density:hit-min` step controls, `density:hit-min.close` close square,
/// right-anchored after the flex field + `min-w` count).
fn next_control_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let (origin, bar_right) = first_family_bar_bounds(window);
    let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let density = theme.density;
    let gap = f32::from(bbr_ui::theme::Spacing::sp1());
    let pad = f32::from(bbr_ui::theme::Spacing::sp2());
    let close_w = f32::from(density.hit_min_close());
    let next_w = f32::from(density.hit_min());
    let next_center_x = px(bar_right - pad - close_w - gap - next_w / 2.0);
    point(next_center_x, origin.y)
}

/// The center of the first family's "previous" control (right-anchored,
/// one step-control width and one gap left of the next control).
fn previous_control_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let (origin, bar_right) = first_family_bar_bounds(window);
    let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let density = theme.density;
    let gap = f32::from(bbr_ui::theme::Spacing::sp1());
    let pad = f32::from(bbr_ui::theme::Spacing::sp2());
    let close_w = f32::from(density.hit_min_close());
    let next_w = f32::from(density.hit_min());
    let prev_w = f32::from(density.hit_min());
    let prev_center_x = px(bar_right - pad - close_w - gap - next_w - gap - prev_w / 2.0);
    point(prev_center_x, origin.y)
}

/// The center of the first family's "close" control (right-anchored at
/// the bar's trailing edge inside the `px:sp2` padding).
fn close_control_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let (origin, bar_right) = first_family_bar_bounds(window);
    let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let density = theme.density;
    let pad = f32::from(bbr_ui::theme::Spacing::sp2());
    let close_w = f32::from(density.hit_min_close());
    let close_center_x = px(bar_right - pad - close_w / 2.0);
    point(close_center_x, origin.y)
}

#[test]
#[ignore]
fn real_pointer_presses_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_find_bar(window, app);
        flush(window);
        // A real pointer press on the first family's next control: the
        // production on_next callback fires and records alone.
        let next_center = next_control_center(window);
        window.simulate_click(next_center, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.find.next, 1, "real next press recorded");
        assert_eq!(snap.find.previous, 0, "next records no previous");
        assert_eq!(snap.find.closes, 0, "next records no close");
        // A real pointer press on the previous control.
        let previous_center = previous_control_center(window);
        window.simulate_click(previous_center, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.find.next, 1, "previous records no next");
        assert_eq!(snap.find.previous, 1, "real previous press recorded");
        assert_eq!(snap.find.closes, 0);
        // A real pointer press on the close control.
        let close_center = close_control_center(window);
        window.simulate_click(close_center, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.find.next, 1, "close records no next");
        assert_eq!(snap.find.previous, 1, "close records no previous");
        assert_eq!(snap.find.closes, 1, "real close press recorded");
    });
}
