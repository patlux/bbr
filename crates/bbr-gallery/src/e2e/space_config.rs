//! Space-configuration interaction scenarios (packet
//! `space-creation-appearance-ui.md`): the story renders every valid §11.3
//! matrix family through the production
//! `bbr_ui::surfaces::space_config::space_config_surface` with recorder
//! callbacks. A real pointer press on a curated theme swatch proves the
//! production `on_select_theme` fires, a press on an emoji/glyph option
//! proves `on_select_identity` fires, a press on Done/Cancel proves
//! `on_done`/`on_cancel`, and a press inside the surface proves the §13
//! inside-press marker fires. No product behavior is asserted here — the
//! gallery proves renderability and callback wiring only.
//!
//! Option targeting uses the production surface's stable §15 ids
//! (`bbr-space-config-theme-<name>`, `bbr-space-config-emoji-<index>`,
//! `bbr-space-config-glyph-<name>`, `bbr-space-config-done`,
//! `bbr-space-config-cancel`) — each family renders exactly once at a time
//! (the story's state selector switches the family), so the ids resolve
//! deterministically through `debug_bounds`.

use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::app::GalleryApp;
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::space_config_matrix::space_config_cases;
use crate::stories::space_config::SPACE_CONFIG_FAMILY_SELECTOR;

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

/// Switch the story's state selector to the family at `index` (the
/// production control path, exactly like the Sidebar story's sweep).
fn select_family(window: &mut VisualTestContext, app: &Entity<GalleryApp>, index: usize) {
    window.update(|_window, cx| {
        app.update(cx, |app, cx| {
            app.apply_control(crate::controls::ControlAction::State(index), cx)
        });
    });
    flush(window);
}

/// The center of the first option row of the matrix family: the family
/// wrapper is the `.flex` containing block; the surface's Done control is
/// the rightmost element of the actions row. The press targets the
/// surface's stable §15 id directly through its measured bounds.
fn center_of(window: &mut VisualTestContext, selector: &'static str) -> gpui::Point<gpui::Pixels> {
    let bounds = bounds_of(window, selector);
    point(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    )
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[test]
#[ignore]
fn space_config_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_story(window, app, "Space configuration");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Space configuration");
        assert_eq!(snap.space_config.events, Vec::new());
        // Every valid matrix family is registered and renders through the
        // state selector (one family at a time, exactly like the Sidebar
        // story's sweep): each family wrapper has non-empty bounds.
        let family_count = space_config_cases().len();
        assert_eq!(family_count, 4);
        for index in 0..family_count {
            select_family(window, app, index);
            let snap = snapshot(window, app);
            assert_eq!(
                snap.story_state,
                space_config_cases()[index].label,
                "state selector switches the family"
            );
            let bounds = bounds_of(
                window,
                static_selector(format!("{SPACE_CONFIG_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "space-configuration family {index} renders"
            );
        }
        // The production surface semantic id renders in the story: the
        // surface registers `bbr-space-config` as a `debug_selector`
        // target, so GPUI's `debug_bounds` resolves it.
        let surface = bounds_of(window, static_selector("bbr-space-config".into()));
        assert!(
            surface.size.width > px(0.0) && surface.size.height > px(0.0),
            "production bbr-space-config surface renders"
        );
        // The full curated option set renders (theme swatches, emoji and
        // glyph options, the action controls) with non-empty bounds.
        for selector in [
            "bbr-space-config-theme-cobalt",
            "bbr-space-config-theme-pine",
            "bbr-space-config-emoji-0",
            "bbr-space-config-glyph-code",
            "bbr-space-config-intensity-subtle",
            "bbr-space-config-intensity-rich",
            "bbr-space-config-grain",
            "bbr-space-config-done",
            "bbr-space-config-cancel",
            "bbr-space-config-name",
            "bbr-space-config-title",
        ] {
            let bounds = bounds_of(window, static_selector(selector.into()));
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "production id {selector} renders"
            );
        }
    });
}

#[test]
#[ignore]
fn pointer_presses_fire_the_production_recorder_callbacks() {
    run_scenario(|window, app| {
        open_story(window, app, "Space configuration");
        // A real pointer press on the curated cobalt swatch fires the
        // production `on_select_theme` (the recorder path). The press
        // bubbles to the surface root so the §13 inside-press marker also
        // fires (the app's outside-press handler must never dismiss the
        // surface mid-configuration) — exactly the downloads-panel
        // row-press precedent.
        let swatch = center_of(
            window,
            static_selector("bbr-space-config-theme-cobalt".into()),
        );
        window.simulate_click(swatch, Modifiers::default());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.space_config.events,
            vec![
                crate::space_config_recorder::SpaceConfigObservation::ThemeSelected(
                    bbr_domain::SpaceTheme::Cobalt
                ),
                crate::space_config_recorder::SpaceConfigObservation::InsidePress,
            ],
            "the production on_select_theme reported Cobalt and the §13 marker fired"
        );
        // A real pointer press on a curated emoji option fires the
        // production `on_select_identity` (plus the §13 marker).
        let emoji = center_of(window, static_selector("bbr-space-config-emoji-0".into()));
        window.simulate_click(emoji, Modifiers::default());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.space_config.events.len(),
            4,
            "identity selection plus the §13 marker appended"
        );
        assert!(
            matches!(
                &snap.space_config.events[2],
                crate::space_config_recorder::SpaceConfigObservation::IdentitySelected(_)
            ),
            "the production on_select_identity reported"
        );
        // v2 D6/D7: real presses on an intensity step and the grain
        // toggle fire the production paper callbacks (plus the marker).
        for (selector, expected) in [
            (
                "bbr-space-config-intensity-rich",
                crate::space_config_recorder::SpaceConfigObservation::IntensitySelected(
                    bbr_domain::TintIntensity::Rich,
                ),
            ),
            (
                "bbr-space-config-grain",
                crate::space_config_recorder::SpaceConfigObservation::GrainToggled,
            ),
        ] {
            let point = center_of(window, static_selector(selector.into()));
            window.simulate_click(point, Modifiers::default());
            flush(window);
            let events = snapshot(window, app).space_config.events;
            assert_eq!(
                events[events.len() - 2..],
                [
                    expected,
                    crate::space_config_recorder::SpaceConfigObservation::InsidePress
                ],
                "{selector} fired its production callback"
            );
        }
        // A real pointer press on Done fires `on_done`.
        let done = center_of(window, static_selector("bbr-space-config-done".into()));
        window.simulate_click(done, Modifiers::default());
        flush(window);
        let snap = snapshot(window, app);
        assert!(
            snap.space_config.events.iter().any(|e| matches!(
                e,
                crate::space_config_recorder::SpaceConfigObservation::Done
            )),
            "the production on_done fired"
        );
        // A real pointer press on Cancel fires `on_cancel`.
        let cancel = center_of(window, static_selector("bbr-space-config-cancel".into()));
        window.simulate_click(cancel, Modifiers::default());
        flush(window);
        let snap = snapshot(window, app);
        assert!(
            snap.space_config.events.iter().any(|e| matches!(
                e,
                crate::space_config_recorder::SpaceConfigObservation::Cancel
            )),
            "the production on_cancel fired"
        );
    });
}
