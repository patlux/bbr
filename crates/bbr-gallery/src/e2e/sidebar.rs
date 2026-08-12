//! G3 Sidebar composition interaction scenarios (packet
//! `component-gallery-compositions.md`): the Sidebar story renders the
//! selected matrix family through the **exact production
//! `bbr_ui::sidebar::sidebar_surface`** with the one production
//! [`bbr_ui::sidebar::Callbacks`] set filled by recorder callbacks. Real
//! pointer presses and real keyboard events on the production targets prove
//! which production callback fired (the recorder path) and that the
//! propagation contract matches the app's wiring (the stop-propagation
//! slots never leak). The every-family sweep covers the matrix families
//! through the story's state selector; the dense family proves the bounded
//! 500-tab windowed render. No product behavior is asserted here — the
//! gallery proves renderability and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{Entity, VisualTestContext};

use crate::app::GalleryApp;
use crate::harness::{
    bounds_of, click_selector, dispatch, flush, move_to_story, run_scenario, snapshot,
};
use crate::sidebar_matrix::{SIDEBAR_REJECTED, SIDEBAR_VALID};
use crate::stories::sidebar::SIDEBAR_FAMILY_SELECTOR;

/// Navigate to the Sidebar story by its label.
fn open_sidebar(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Sidebar")
            .expect("Sidebar story registered")
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

/// Focus one of the fixture handles (the production keyboard paths need the
/// element focused; the gallery never focuses them itself).
fn focus_handle(window: &mut VisualTestContext, handle: &gpui::FocusHandle) {
    window.update(|window, cx| {
        window.focus(handle, cx);
    });
    flush(window);
}

#[test]
#[ignore]
fn sidebar_story_renders_every_matrix_family_with_stable_ids() {
    run_scenario(|window, app| {
        open_sidebar(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Sidebar");
        assert!(snap.sidebar.select_rows.is_empty());

        // The default family renders the production rail. GPUI's
        // `debug_bounds` exposes only explicit `debug_selector` values, so
        // this sweep asserts exactly the production elements that expose
        // one (the G2 precedent: tab rows, favorites, folders, footer
        // controls). The remaining §15 ids (`bbr-sidebar`, `bbr-brand`,
        // `bbr-address`, section labels, `bbr-status`, `bbr-sidebar-toggle`,
        // `bbr-space-strip`, `bbr-sidebar-divider`) are plain `.id()`
        // elements — not debug targets — and are proven by the matrix
        // model/contract tests and the story's value rows, never claimed
        // as pointer targets here.
        let family = bounds_of(window, SIDEBAR_FAMILY_SELECTOR);
        assert!(
            family.size.width > gpui::px(0.0) && family.size.height > gpui::px(0.0),
            "the default family renders"
        );
        for id in [
            "bbr-favorites",
            "bbr-fav-0",
            "bbr-tab-row-0",
            "bbr-folder-row-0",
            "bbr-folder-item-0-0",
            "bbr-footer",
            "bbr-downloads",
            "bbr-plus",
        ] {
            assert!(
                window.debug_bounds(id).is_some(),
                "production selector {id} renders in the default family"
            );
        }

        // Every matrix family renders through the state selector.
        let family_count = window.read(|_| SIDEBAR_VALID.len());
        assert_eq!(family_count, 14);
        for (index, case) in SIDEBAR_VALID.iter().enumerate() {
            select_family(window, app, index);
            let snap = snapshot(window, app);
            assert_eq!(
                snap.story_state, case.label,
                "state selector switches the family"
            );
            let family = bounds_of(window, SIDEBAR_FAMILY_SELECTOR);
            assert!(
                family.size.width > gpui::px(0.0) && family.size.height > gpui::px(0.0),
                "family {index} ({}) renders",
                case.label
            );
        }

        // The dense 500-tab family renders the bounded window: exactly the
        // pinned block plus the 64-row window — rows outside the window are
        // never exposed.
        select_family(window, app, 1);
        assert!(window.debug_bounds("bbr-tab-row-10").is_some());
        assert!(window.debug_bounds("bbr-tab-row-73").is_some());
        assert!(
            window.debug_bounds("bbr-tab-row-100").is_none(),
            "rows outside the bounded window are never rendered"
        );
        assert!(
            window.debug_bounds("bbr-tab-row-500").is_none(),
            "the 500th row is never rendered"
        );

        // The collapsed family renders the rail at width 0: the rail is a
        // 0-wide flex container, so its production rows lay out at 0 width
        // (the matrix unit test proves the model's `sidebar_width` is 0;
        // here the rendered row reports the same 0 width through the
        // production selector).
        select_family(window, app, 4);
        let collapsed = bounds_of(window, "bbr-tab-row-0");
        assert_eq!(
            collapsed.size.width,
            gpui::px(0.0),
            "collapsed rail is 0 wide"
        );

        // The rejected combinations are declared and refuse construction
        // (the matrix unit tests prove the refusal; the story shows them).
        let rejected_count = window.read(|_| SIDEBAR_REJECTED.len());
        assert_eq!(rejected_count, 4);

        // Mode/state fields are untouched by story selection.
        let snap = snapshot(window, app);
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_pointer_presses_report_through_the_sidebar_recorders() {
    run_scenario(|window, app| {
        open_sidebar(window, app);
        // The default family: 1 pinned + 3 regular rows, 3 favorites, 1
        // folder with 2 members, 2 spaces, address command, footer, toggle,
        // divider.
        //
        // Pointer targets are exactly the production elements that expose
        // an explicit `debug_selector` (the G2 precedent). The remaining
        // §15 ids (`bbr-space-chip-<n>`, `bbr-sidebar-toggle`,
        // `bbr-sidebar-divider`, `bbr-address`, `bbr-plus-menu` and its
        // items) are plain `.id()` elements in `bbr-ui` — GPUI's
        // `debug_bounds` cannot target them, and this lane must not add
        // production debug selectors or claim non-selector ids as pointer
        // targets. Their wiring stays proven by the observer contract test
        // and the G2 component stories; the divider and space-strip
        // keyboard paths are covered below.

        // §11.1 row body: press + select + drag begin + the rail chrome
        // focus entry (the production pointer path, no stop-propagation).
        click_selector(window, "bbr-tab-row-1");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.press_rows, vec![1], "row press");
        assert_eq!(snap.sidebar.select_rows, vec![1], "row select");
        assert_eq!(snap.sidebar.drag_begins, vec![1], "row drag begin");
        assert_eq!(snap.sidebar.rail_mouse_downs, 1, "rail chrome focus entry");

        // §11.1 close control: stops propagation — only the close fires.
        click_selector(window, "bbr-tab-close-1");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.close_rows, vec![1], "row close");
        assert_eq!(
            snap.sidebar.select_rows,
            vec![1],
            "close stops propagation — no select after a close press"
        );
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 1,
            "close never reaches the rail"
        );

        // §11.2 pin control: stops propagation — only the pin fires.
        click_selector(window, "bbr-tab-pin-1");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.toggle_pins, vec![1], "pin toggle");
        assert_eq!(snap.sidebar.select_rows, vec![1], "pin stops propagation");
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 1,
            "pin never reaches the rail"
        );

        // §11.10 favorites tile: the activation fires and the rail chrome
        // entry follows (no stop-propagation).
        click_selector(window, "bbr-fav-0");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.favorite_activations,
            vec![0],
            "favorite activation"
        );
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 2,
            "tile press reaches the rail"
        );

        // §11.11 folder row: the row body fires the focus entry (the
        // disclosure toggle is the separate `bbr-folder-toggle-<n>`
        // control, covered by the G2 folders story) and the rail entry.
        click_selector(window, "bbr-folder-row-0");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.focus_folders, vec![1], "folder focus");
        assert!(
            snap.sidebar.toggle_folders.is_empty(),
            "the row body never fires the disclosure toggle"
        );
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 3,
            "folder press reaches the rail"
        );

        // §11.11 nested member: select + the rail entry.
        click_selector(window, "bbr-folder-item-0-0");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.folder_member_selects,
            vec![(1, 0)],
            "member select"
        );
        assert!(snap.sidebar.folder_member_closes.is_empty());
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 4,
            "member press reaches the rail"
        );

        // §11.11 nested member close: stops propagation — only the close
        // fires, never a select.
        click_selector(window, "bbr-folder-item-0-0-close");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.folder_member_closes,
            vec![(1, 0)],
            "member close"
        );
        assert_eq!(
            snap.sidebar.folder_member_selects,
            vec![(1, 0)],
            "close stops propagation (§11.11)"
        );

        // §11.4 footer controls: both stop propagation.
        click_selector(window, "bbr-downloads");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.downloads, 1, "downloads press");
        assert_eq!(
            snap.sidebar.rail_mouse_downs, 4,
            "downloads stops propagation"
        );
        click_selector(window, "bbr-plus");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.plus, 1, "plus press");
        assert_eq!(snap.sidebar.downloads, 1, "plus never touches downloads");
        assert_eq!(snap.sidebar.rail_mouse_downs, 4, "plus stops propagation");
    });
}

#[test]
#[ignore]
fn real_keyboard_events_report_through_the_sidebar_recorders() {
    run_scenario(|window, app| {
        open_sidebar(window, app);
        let handles = window.read(|cx| app.read(cx).sidebar.handles().clone_handles());

        // §11.1/§11.6 tab-list keyboard: focus the list and dispatch a real
        // keystroke through the production key path.
        focus_handle(window, &handles.tab_list);
        dispatch(window, "down");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.tab_list_keys,
            vec!["down"],
            "tab-list key down"
        );

        // §11.11 folders keyboard.
        focus_handle(window, &handles.folders);
        dispatch(window, "down");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.folders_keys, vec!["down"], "folders key down");

        // §11.3 space-strip keyboard.
        focus_handle(window, &handles.space_strip);
        dispatch(window, "right");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.space_strip_keys,
            vec!["right"],
            "space-strip key down"
        );

        // §9 divider keyboard.
        focus_handle(window, &handles.divider);
        dispatch(window, "right");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.sidebar.divider_keys, vec!["right"], "divider key down");

        // §11.4 plus-menu keyboard: the "plus menu open" family renders the
        // menu; focus it and dispatch a real keystroke.
        select_family(window, app, 8);
        focus_handle(window, &handles.plus_menu);
        dispatch(window, "down");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.sidebar.plus_menu_keys,
            vec!["down"],
            "plus-menu key down"
        );
    });
}
