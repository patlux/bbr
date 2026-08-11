//! Context-menu interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 9): the story renders
//! every valid §11.4/§11.13 page-menu family through the production
//! `bbr_ui::surfaces::menu::context_menu_card` with recorder callbacks. A
//! real pointer hover on an enabled row proves the production
//! `on_row_hover` fires, and a real pointer press proves `on_row_press`
//! fires (the recorder path); a disabled row never reports either (the
//! production card attaches no listeners to disabled rows). The
//! every-story sweep covers the new story automatically. No product
//! behavior is asserted here — the gallery proves renderability and
//! callback wiring only.
//!
//! Row targeting uses the DESIGN §8/§11.4 row pitch (`theme.density.row()`,
//! the same formula the Address pill E2E uses) rather than the production
//! row ids, because the card's fixed `bbr-context-menu-row-<n>` ids repeat
//! per matrix family (GPUI's `debug_bounds` map keeps the last painted
//! family, so a row id is not family-deterministic). The production card
//! registers its card id (`bbr-context-menu`) as a `debug_selector`
//! target, so the semantic-id assertion resolves through `debug_bounds`.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::app::GalleryApp;
use crate::context_menu_matrix::{
    menu_rows, CONTEXT_MENU_FIXTURE_RECT, CONTEXT_MENU_REJECTED, CONTEXT_MENU_VALID,
};
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::stories::context_menu::CONTEXT_MENU_FAMILY_SELECTOR;

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

/// The center of one row of one matrix family's card. The family wrapper
/// is the `.relative()` containing block; the card sits at the fixture
/// rect (top-left origin) and the `sp:1` card padding sits above the
/// first row. The row pitch is the DESIGN §8 `density:row` height (36 pt
/// at Normal) — the §11.4 command-surface family's row contract — not the
/// text-line box, so the pointer lands inside the row's real hitbox
/// (the settled production row geometry the Address pill E2E derives the
/// same way).
fn family_row_center(
    window: &mut VisualTestContext,
    family: usize,
    row: usize,
) -> gpui::Point<gpui::Pixels> {
    let family_bounds = bounds_of(
        window,
        static_selector(format!("{CONTEXT_MENU_FAMILY_SELECTOR}{family}")),
    );
    let theme = bbr_ui::theme::Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let row_height = f32::from(theme.density.row());
    let card_padding = f32::from(bbr_ui::theme::Spacing::sp1());
    let x = family_bounds.origin.x
        + px(CONTEXT_MENU_FIXTURE_RECT.x + CONTEXT_MENU_FIXTURE_RECT.width / 2.0);
    let y = family_bounds.origin.y
        + px(CONTEXT_MENU_FIXTURE_RECT.y
            + card_padding
            + row_height / 2.0
            + row as f32 * row_height);
    point(x, y)
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[test]
#[ignore]
fn context_menu_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_story(window, app, "Context menu");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Context menu");
        assert_eq!(snap.context_menu.hovers, Vec::<(usize, bool)>::new());
        assert_eq!(snap.context_menu.presses, Vec::<usize>::new());
        // Every valid page-menu family is registered and renders through
        // the production card (each family wrapper has non-empty bounds).
        let family_count = window.read(|_| CONTEXT_MENU_VALID.len());
        assert_eq!(family_count, 7);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{CONTEXT_MENU_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "page-menu family {index} renders"
            );
        }
        // The production card semantic id renders in the story: the card
        // registers `bbr-context-menu` as a `debug_selector` target, so
        // GPUI's `debug_bounds` resolves it (the semantic `.id()` alone
        // never enters `debug_bounds`).
        assert!(
            window
                .debug_bounds(bbr_ui::surfaces::menu::CONTEXT_MENU_ID)
                .is_some(),
            "the story includes the production bbr-context-menu semantic id"
        );
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| CONTEXT_MENU_REJECTED.len());
        assert_eq!(rejected_count, 4);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_hover_and_press_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_story(window, app, "Context menu");
        flush(window);
        // Family 0 is the fresh presentation: Back (0) / Forward (1)
        // disabled, Reload (2) enabled, Copy Link (3) / Open in New Tab
        // (4) disabled — the production row model.
        let rows = window.read(|_| menu_rows(false, false, None));
        assert!(rows[2].enabled, "Reload is the first enabled row");
        assert!(!rows[0].enabled && !rows[1].enabled);
        assert!(!rows[3].enabled && !rows[4].enabled);

        // A real pointer move onto the enabled Reload row: the production
        // `on_row_hover` callback fires with `(index, hovered: true)` —
        // the pointer entered the row (§11.13 shared cursor).
        let reload = family_row_center(window, 0, 2);
        window.simulate_mouse_move(reload, None, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.context_menu.hovers,
            vec![(2, true)],
            "real pointer move records the hovered row (§11.13 shared cursor)"
        );
        assert_eq!(
            snap.context_menu.presses,
            Vec::<usize>::new(),
            "hover never activates"
        );

        // A real pointer press on the same row: the production
        // `on_row_press` callback fires (the app's menu_activate path). A
        // mouse down/up dispatches no hover event, so the hover history
        // stays exactly the single enter.
        window.simulate_click(reload, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.context_menu.presses,
            vec![2],
            "real press records on_row_press"
        );
        assert_eq!(
            snap.context_menu.hovers,
            vec![(2, true)],
            "the press dispatches no additional hover entry"
        );

        // A disabled row (Back, disabled without history) never reports:
        // the production card attaches no hover/press listeners to it. The
        // move away from Reload fires its `on_hover` exit (`(2, false)`),
        // but nothing ever reports the disabled row's own index.
        let back = family_row_center(window, 0, 0);
        window.simulate_mouse_move(back, None, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert!(
            snap.context_menu.hovers.contains(&(2, false)),
            "leaving the enabled row fires the hover exit"
        );
        assert!(
            !snap
                .context_menu
                .hovers
                .iter()
                .any(|(index, _hovered)| *index == 0),
            "disabled rows never report hover (no listener attached)"
        );
        assert_eq!(
            snap.context_menu.presses,
            vec![2],
            "hover on the disabled row never activates"
        );
        // A real press on the disabled row: no `on_row_press` fires (the
        // disabled row carries no activation listener).
        window.simulate_click(back, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.context_menu.presses,
            vec![2],
            "disabled rows never report press (no listener attached)"
        );
    });
}
