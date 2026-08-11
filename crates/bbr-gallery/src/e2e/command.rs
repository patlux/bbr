//! G3 Command composition interaction scenarios (packet
//! `component-gallery-compositions.md`): the Command story renders the
//! selected matrix family through the **exact production
//! `bbr_ui::surfaces::command::command_card`** with the plain fixture
//! model. The every-family sweep covers the matrix families through the
//! story's state selector and asserts the production debug-selector targets
//! (the explicit `debug_selector` values the production renderer exposes —
//! plain `.id()` elements are never claimed as debug targets here); the
//! observation scenario asserts the production presentation summary the
//! app recorded. No product behavior is asserted here — the gallery proves
//! renderability and presentation observation only; activation stays
//! app/Portal-owned and never exists in the gallery.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{Entity, VisualTestContext};

use crate::app::GalleryApp;
use crate::command_matrix::{COMMAND_REJECTED, COMMAND_VALID};
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::stories::command::COMMAND_FAMILY_SELECTOR;

/// Navigate to the Command story by its label.
fn open_command(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Command palette / overlay")
            .expect("Command story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

/// Switch the story's state selector to the family at `index` (the
/// production control path).
fn select_family(window: &mut VisualTestContext, app: &Entity<GalleryApp>, index: usize) {
    window.update(|_window, cx| {
        app.update(cx, |app, cx| {
            app.apply_control(crate::controls::ControlAction::State(index), cx)
        });
    });
    flush(window);
}

#[test]
#[ignore]
fn command_story_renders_every_matrix_family_with_stable_ids() {
    run_scenario(|window, app| {
        open_command(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Command palette / overlay");
        // The Command story is the append-only StoryId 45 (the catalog now
        // holds 46 stories; ids 0–44 are pinned).
        assert_eq!(snap.selected_story, 45);

        // The default family (closed / inactive) renders the production
        // closed card and nothing else.
        let family = bounds_of(window, COMMAND_FAMILY_SELECTOR);
        assert!(
            family.size.width > gpui::px(0.0) && family.size.height > gpui::px(0.0),
            "the default family renders"
        );
        assert!(
            window.debug_bounds("bbr-command-portal-closed").is_some(),
            "the closed family renders the production closed card"
        );
        assert!(
            window.debug_bounds("bbr-palette").is_none(),
            "the closed family never renders the palette"
        );

        // Every matrix family renders through the state selector with its
        // exact production debug targets (one surface at a time).
        assert_eq!(COMMAND_VALID.len(), 12);
        for (index, case) in COMMAND_VALID.iter().enumerate() {
            select_family(window, app, index);
            let snap = snapshot(window, app);
            assert_eq!(
                snap.story_state, case.label,
                "state selector switches the family"
            );
            let family = bounds_of(window, COMMAND_FAMILY_SELECTOR);
            assert!(
                family.size.width > gpui::px(0.0) && family.size.height > gpui::px(0.0),
                "family {index} ({}) renders",
                case.label
            );
            let palette = window.debug_bounds("bbr-palette").is_some();
            let overlay = window.debug_bounds("bbr-command-overlay").is_some();
            let closed = window.debug_bounds("bbr-command-portal-closed").is_some();
            // The mutual exclusion: exactly one surface renders.
            assert_eq!(
                [palette, overlay, closed]
                    .into_iter()
                    .filter(|b| *b)
                    .count(),
                1,
                "family {index} renders exactly one presentation"
            );
            match case.presentation {
                bbr_ui::surfaces::command::CommandPresentation::Closed => {
                    assert!(closed, "family {index} renders the closed card");
                }
                bbr_ui::surfaces::command::CommandPresentation::CenteredPalette => {
                    assert!(palette, "family {index} renders the palette");
                    assert!(
                        window.debug_bounds("bbr-palette-field").is_some(),
                        "family {index} renders the palette field"
                    );
                }
                bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay => {
                    assert!(overlay, "family {index} renders the overlay");
                    assert!(
                        window.debug_bounds("bbr-command-overlay-field").is_some(),
                        "family {index} renders the overlay field"
                    );
                }
            }
            // The typed state rows render their exact §11.5 ids.
            let (state_target, state_targets) = match (case.presentation, case.state) {
                (
                    bbr_ui::surfaces::command::CommandPresentation::CenteredPalette,
                    bbr_ui::surfaces::command::CommandResultState::Loading,
                ) => ("bbr-palette-loading", &["bbr-palette-loading"][..]),
                (
                    bbr_ui::surfaces::command::CommandPresentation::CenteredPalette,
                    bbr_ui::surfaces::command::CommandResultState::Empty,
                ) => ("bbr-palette-empty", &["bbr-palette-empty"][..]),
                (
                    bbr_ui::surfaces::command::CommandPresentation::CenteredPalette,
                    bbr_ui::surfaces::command::CommandResultState::ActivationError,
                ) => ("bbr-palette-error", &["bbr-palette-error"][..]),
                (
                    bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay,
                    bbr_ui::surfaces::command::CommandResultState::Loading,
                ) => ("bbr-command-loading", &["bbr-command-loading"][..]),
                (
                    bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay,
                    bbr_ui::surfaces::command::CommandResultState::Empty,
                ) => ("bbr-command-empty", &["bbr-command-empty"][..]),
                (
                    bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay,
                    bbr_ui::surfaces::command::CommandResultState::ActivationError,
                ) => ("bbr-command-error", &["bbr-command-error"][..]),
                _ => ("", &[][..]),
            };
            if !state_targets.is_empty() {
                assert!(
                    window.debug_bounds(state_target).is_some(),
                    "family {index} renders the state row {}",
                    state_target
                );
            }
            // The results families render their deterministic §11.5 row ids
            // under the surface's own prefix (the row specs are pinned by
            // [`RowSpec::results`], so the ids are literal and static).
            if case.state == bbr_ui::surfaces::command::CommandResultState::Results {
                let row_ids: &[&str] = match case.presentation {
                    bbr_ui::surfaces::command::CommandPresentation::CenteredPalette => &[
                        "bbr-palette-result-tab:1",
                        "bbr-palette-result-tab:2",
                        "bbr-palette-result-space:3",
                        "bbr-palette-result-action:1",
                        "bbr-palette-result-action:2",
                        "bbr-palette-result-fallback",
                    ],
                    bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay => &[
                        "bbr-command-result-tab:1",
                        "bbr-command-result-tab:2",
                        "bbr-command-result-space:3",
                        "bbr-command-result-action:1",
                        "bbr-command-result-action:2",
                        "bbr-command-result-fallback",
                    ],
                    bbr_ui::surfaces::command::CommandPresentation::Closed => unreachable!(),
                };
                for id in row_ids {
                    assert!(
                        window.debug_bounds(id).is_some(),
                        "family {index} renders the result row {}",
                        id
                    );
                }
            }
        }

        // The rejected combinations are declared and refuse construction
        // (the matrix unit tests prove the refusal; the story shows them).
        assert_eq!(COMMAND_REJECTED.len(), 2);

        // Mode/state fields are untouched by story selection.
        let snap = snapshot(window, app);
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn command_story_observes_the_production_presentation_model() {
    run_scenario(|window, app| {
        open_command(window, app);
        // The closed family observes the production model summary (the
        // "observer-or-snapshot" evidence for the callback-free surface).
        let snap = snapshot(window, app);
        assert_eq!(snap.command.presentation.as_deref(), Some("closed"));
        assert_eq!(snap.command.state.as_deref(), Some("inactive"));
        assert!(!snap.command.chrome_focused);
        assert!(snap.command.rows.is_empty());

        // The results family observes the deterministic rendered keys in
        // render order and the selection.
        select_family(window, app, 1); // centered palette — results (selected 1)
        let snap = snapshot(window, app);
        assert_eq!(snap.command.presentation.as_deref(), Some("palette"));
        assert_eq!(snap.command.state.as_deref(), Some("results"));
        assert!(snap.command.chrome_focused);
        assert_eq!(snap.command.selected, 1);
        assert_eq!(
            snap.command.rows,
            vec!["tab:1", "tab:2", "space:3", "action:1", "action:2", "fallback",]
        );

        // The overlay family observes its own discriminant.
        select_family(window, app, 5); // anchored overlay — results (selected 0)
        let snap = snapshot(window, app);
        assert_eq!(snap.command.presentation.as_deref(), Some("overlay"));
        assert_eq!(snap.command.state.as_deref(), Some("results"));
        assert_eq!(snap.command.selected, 0);

        // The activation-error family observes the typed state.
        select_family(window, app, 8); // anchored overlay — activation error
        let snap = snapshot(window, app);
        assert_eq!(snap.command.presentation.as_deref(), Some("overlay"));
        assert_eq!(snap.command.state.as_deref(), Some("activation-error"));

        // The renders counter proves the story re-observes on each
        // state-selector switch through the production control path.
        assert!(snap.command.renders >= 5);
    });
}
