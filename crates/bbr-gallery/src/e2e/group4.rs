//! Group 4 sidebar-component interaction scenarios (packet
//! `component-gallery-browser-components.md`): the five stories render
//! every valid matrix family through the production `bbr-ui` symbols
//! (`favorites_grid`/`favorite_tile`, `folder_row`/`folder_item_row`,
//! `footer_surface`, `profile_chip`, `plus_menu_items`/`PlusMenuState`)
//! with recorder callbacks. Real pointer presses on the production
//! targets prove which production callback fired (the recorder path); the
//! every-story sweep covers the five new stories automatically. No product
//! behavior is asserted here — the gallery proves renderability and
//! callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, Modifiers, VisualTestContext};

use crate::app::GalleryApp;
use crate::favorites_matrix::{FAVORITES_REJECTED, FAVORITES_VALID};
use crate::folders_matrix::{FOLDERS_REJECTED, FOLDERS_VALID};
use crate::footer_matrix::{FOOTER_REJECTED, FOOTER_VALID};
use crate::harness::{bounds_of, click_selector, flush, move_to_story, run_scenario, snapshot};
use crate::menu_matrix::{PLUS_MENU_REJECTED, PLUS_MENU_VALID};
use crate::profile_chip_matrix::{PROFILE_CHIP_REJECTED, PROFILE_CHIP_VALID};
use crate::stories::favorites::FAVORITES_FAMILY_SELECTOR;
use crate::stories::folders::FOLDERS_FAMILY_SELECTOR;
use crate::stories::footer::FOOTER_FAMILY_SELECTOR;
use crate::stories::menu::MENU_FAMILY_SELECTOR;
use crate::stories::profile_chip::PROFILE_CHIP_FAMILY_SELECTOR;

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

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[test]
#[ignore]
fn group4_stories_render_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        // §11.10 Favorites grid: every valid family renders through the
        // production grid + tiles; the empty family hides the label.
        open_story(window, app, "Favorites grid");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Favorites grid");
        assert!(snap.group4.favorite_activations.is_empty());
        let family_count = window.read(|_| FAVORITES_VALID.len());
        assert_eq!(family_count, 6);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{FAVORITES_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "favorites family {index} renders"
            );
        }
        // The production tile ids render (the E2E targets `bbr-fav-0`).
        assert!(window.debug_bounds("bbr-fav-0").is_some());
        assert!(window.debug_bounds("bbr-fav-icon-0").is_some());
        let rejected_count = window.read(|_| FAVORITES_REJECTED.len());
        assert_eq!(rejected_count, 2);

        // §11.11 Folders: every valid family renders; the expanded family
        // exposes the stable nested ids (bbr-folder-item-10-0 …).
        open_story(window, app, "Folders");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Folders");
        assert!(snap.group4.folder_toggles.is_empty());
        let family_count = window.read(|_| FOLDERS_VALID.len());
        assert_eq!(family_count, 5);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{FOLDERS_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "folders family {index} renders"
            );
        }
        assert!(bounds_of(window, "bbr-folder-row-0").size.height > px(0.0));
        assert!(bounds_of(window, "bbr-folder-toggle-10").size.width > px(0.0));
        assert!(bounds_of(window, "bbr-folder-item-10-0").size.height > px(0.0));
        assert!(bounds_of(window, "bbr-folder-item-10-0-close").size.width > px(0.0));
        let rejected_count = window.read(|_| FOLDERS_REJECTED.len());
        assert_eq!(rejected_count, 2);

        // §11.4 Footer: every valid family renders the production surface
        // with its stable ids (downloads · strip · plus).
        open_story(window, app, "Footer");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Footer");
        assert_eq!(snap.group4.downloads, 0);
        assert_eq!(snap.group4.plus, 0);
        let family_count = window.read(|_| FOOTER_VALID.len());
        assert_eq!(family_count, 8);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{FOOTER_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "footer family {index} renders"
            );
        }
        // The production §15 ids ride every footer: the fixed controls and
        // the allowed strip in context.
        assert!(window.debug_bounds("bbr-footer").is_some());
        assert!(window.debug_bounds("bbr-downloads").is_some());
        assert!(window.debug_bounds("bbr-space-strip").is_some());
        assert!(window.debug_bounds("bbr-plus").is_some());
        let rejected_count = window.read(|_| FOOTER_REJECTED.len());
        assert_eq!(rejected_count, 2);

        // §12 Profile chip (S/N/XL): every valid family renders; the
        // profile-keyed id family carries the unique §12 key.
        open_story(window, app, "Profile chip (S/N/XL)");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Profile chip (S/N/XL)");
        let family_count = window.read(|_| PROFILE_CHIP_VALID.len());
        assert_eq!(family_count, 12);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{PROFILE_CHIP_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "profile chip family {index} renders"
            );
        }
        assert!(
            window.debug_bounds("bbr-profile-chip-8").is_some(),
            "the profile-keyed id family renders bbr-profile-chip-8"
        );
        let rejected_count = window.read(|_| PROFILE_CHIP_REJECTED.len());
        assert_eq!(rejected_count, 2);

        // §11.4 Plus menu: the state families render; the two production
        // v1 items expose their production §15 ids.
        open_story(window, app, "Plus menu");
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Plus menu");
        assert!(snap.group4.menu_items.is_empty());
        let family_count = window.read(|_| PLUS_MENU_VALID.len());
        assert_eq!(family_count, 3);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{MENU_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "plus menu family {index} renders"
            );
        }
        assert!(window.debug_bounds("bbr-plus-menu").is_some());
        assert!(window.debug_bounds("bbr-plus-new-space").is_some());
        assert!(window.debug_bounds("bbr-plus-new-folder").is_some());
        let rejected_count = window.read(|_| PLUS_MENU_REJECTED.len());
        assert_eq!(rejected_count, 2);

        // Mode/state fields are untouched by story selection.
        let snap = snapshot(window, app);
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_pointer_presses_report_through_the_group4_recorders() {
    run_scenario(|window, app| {
        // §11.10: a real pointer press on the first tile of the N
        // family fires the production `on_activate` for that exact tile.
        open_story(window, app, "Favorites grid");
        flush(window);
        let family = bounds_of(
            window,
            static_selector(format!("{FAVORITES_FAMILY_SELECTOR}0")),
        );
        // Tile 0 (bbr-fav-0): 36 × 36 at the grid's sp2 inset.
        let tile = point(family.origin.x + px(26.0), family.origin.y + px(26.0));
        window.simulate_click(tile, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.group4.favorite_activations,
            vec![0],
            "real tile press records on_activate"
        );

        // §11.11: the disclosure toggle, the nested-row select, and the
        // close (which stops propagation — no select after a close press).
        open_story(window, app, "Folders");
        flush(window);
        click_selector(window, "bbr-folder-toggle-10");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.group4.folder_toggles, vec![10], "real toggle press");
        let member = bounds_of(window, "bbr-folder-item-10-0");
        // The §11.1 monogram tile (sp3 inset + icon) — the row body left
        // of the close control: select fires, close does not.
        let body = point(member.origin.x + px(21.0), member.center().y);
        window.simulate_click(body, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.group4.folder_selects, vec![(10, 0)], "member select");
        assert!(snap.group4.folder_closes.is_empty());
        click_selector(window, "bbr-folder-item-10-0-close");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.group4.folder_closes,
            vec![(10, 0)],
            "real close press records on_close"
        );
        assert_eq!(
            snap.group4.folder_selects,
            vec![(10, 0)],
            "close stops propagation (§11.11)"
        );

        // §11.4: real presses on the footer fixed controls fire
        // `on_downloads` and `on_plus`.
        open_story(window, app, "Footer");
        flush(window);
        let family = bounds_of(
            window,
            static_selector(format!("{FOOTER_FAMILY_SELECTOR}0")),
        );
        // The [Downloads] glyph (sp2 inset + half the 18 pt icon) and the
        // glyph-only [+] at the row's trailing sp2 inset.
        let downloads = point(family.origin.x + px(17.0), family.origin.y + px(18.0));
        window.simulate_click(downloads, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.group4.downloads, 1, "real downloads press");
        let plus = point(family.right() - px(17.0), family.origin.y + px(18.0));
        window.simulate_click(plus, Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.group4.plus, 1, "real plus press");
        assert_eq!(snap.group4.downloads, 1, "plus never touches downloads");

        // §11.4: real presses on the open menu's production item ids
        // commit exactly those items through the recorder (the production
        // §15 ids ride the debug-selector map, the tab-row pattern).
        open_story(window, app, "Plus menu");
        flush(window);
        click_selector(window, "bbr-plus-new-space");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.group4.menu_items,
            vec!["bbr-plus-new-space"],
            "New Space commit"
        );
        click_selector(window, "bbr-plus-new-folder");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.group4.menu_items,
            vec!["bbr-plus-new-space", "bbr-plus-new-folder"],
            "New Folder commit"
        );
    });
}
