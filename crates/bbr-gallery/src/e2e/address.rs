//! Address/Input interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 3): the story renders
//! every valid §11.5 matrix family through the production
//! `bbr-ui::components::address` symbols (`address_control`,
//! `staging_surface`) with recorder callbacks. A real pointer hover on the
//! pill proves the production `on_hover` fires, and a real pointer press
//! proves `on_open` fires (the recorder path); the every-story sweep covers
//! the two new stories automatically. No product behavior is asserted
//! here — the gallery proves renderability and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::address_matrix::{ADDRESS_REJECTED, ADDRESS_VALID, STAGING_VALID};
use crate::app::GalleryApp;
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::stories::address::{ADDRESS_FAMILY_SELECTOR, STAGING_FAMILY_SELECTOR};

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

/// The §11.5 pill center of the first matrix family: the gallery wraps
/// each family in a test-target container; the pill is the first child at
/// `density:row` height (36 pt at Normal — the canvas default), so its
/// center sits half that height below the wrapper's top edge.
fn first_pill_center(window: &mut VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let family = bounds_of(
        window,
        static_selector(format!("{ADDRESS_FAMILY_SELECTOR}0")),
    );
    point(
        family.center().x,
        family.origin.y
            + px(f32::from(
                bbr_ui::theme::Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
                    .density
                    .row(),
            ) / 2.0),
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
fn address_stories_render_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_story(window, app, "Address control");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Address control");
        assert_eq!(snap.address.hovers, Vec::<bool>::new());
        assert_eq!(snap.address.opens, 0);
        // Every valid pill family is registered and renders through the
        // production component (each family wrapper has non-empty bounds).
        let family_count = window.read(|_| ADDRESS_VALID.len());
        assert_eq!(family_count, 6);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{ADDRESS_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "pill family {index} renders"
            );
        }
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| ADDRESS_REJECTED.len());
        assert_eq!(rejected_count, 3);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);

        // The Command staging story renders every staging family.
        open_story(window, app, "Command staging");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Command staging");
        let staging_count = window.read(|_| STAGING_VALID.len());
        assert_eq!(staging_count, 7);
        for index in 0..staging_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{STAGING_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "staging family {index} renders"
            );
        }
        // The staging story includes the production profile-hint semantic
        // id: the profile-hint families render the production
        // `staging_surface` with `StagingModel.profile_hint: Some(...)`
        // (the production path that renders `bbr-command-profile-hint`),
        // and the story registers that production id as a test-target key
        // on those families.
        let profile_hint_families: Vec<usize> = window.read(|_| {
            STAGING_VALID
                .iter()
                .enumerate()
                .filter(|(_, case)| case.profile_hint)
                .map(|(index, _)| index)
                .collect()
        });
        assert!(
            !profile_hint_families.is_empty(),
            "the matrix declares profile-hint staging families"
        );
        for index in profile_hint_families {
            let bounds = bounds_of(
                window,
                static_selector(format!("{STAGING_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "profile-hint staging family {index} renders"
            );
        }
        assert!(
            window.debug_bounds("bbr-command-profile-hint").is_some(),
            "the staging story includes the production profile-hint semantic id"
        );
    });
}

#[test]
#[ignore]
fn real_hover_and_press_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_story(window, app, "Address control");
        flush(window);
        // A real pointer move onto the pill: the production `on_hover`
        // callback fires with `true` (§10.2 wash follows the pointer).
        let pill = first_pill_center(window);
        window.simulate_mouse_move(pill, None, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.address.hovers,
            vec![true],
            "real pointer move records the hover entry (§10.2)"
        );
        assert_eq!(snap.address.opens, 0, "hover never opens the overlay");
        // A real pointer press on the pill: the production `on_open`
        // callback fires (the press that opens the §11.4 overlay over the
        // control) and stops propagation (§11.5/§13).
        window.simulate_click(pill, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.address.opens, 1, "real press records on_open");
        assert_eq!(
            snap.address.hovers,
            vec![true],
            "the press dispatches no additional hover entry"
        );
    });
}
