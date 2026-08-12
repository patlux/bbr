//! Open + dismiss scenarios (task packet `tab-favorite-context-menus`).
//!
//! Proves the no-open-mutation contract: opening the menu on an **inactive**
//! tab row never selects it (the selected neighbor stays selected and the
//! active web view stays presented); keyboard traversal and submenu
//! navigation preserve the bound source; Escape and an outside pointer press
//! dismiss with one action, restoring the origin focus domain; and the
//! complete controller snapshot and session registry stay byte-for-byte
//! unchanged across the whole open/traverse/dismiss cycle.

use crate::e2e_common::*;
use crate::support::*;

/// Open on an inactive tab row, traverse by keyboard (Up/Down, Right into
/// the space submenu, Left back), and dismiss by Escape — with the
/// snapshot identity untouched at every step.
#[test]
#[ignore]
fn open_traverse_and_escape_dismiss_never_mutate() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-open", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        // Open a second tab: it becomes active; the first tab stays hidden
        // warm at index 0 (the inactive row the menu opens on).
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        // A second space in the same profile makes the `Move to Space`
        // submenu genuinely present (an empty destination group is omitted
        // whole, so the submenu traversal below needs a real destination).
        create_second_space(&mut client, deadline());
        let before = client.snapshot().expect("before snapshot");
        assert_eq!(
            before.active_tab_index,
            Some(1),
            "the second tab is the selected neighbor"
        );
        let source_tab = active_space_tab_id(&before, 0);
        let before_identity = identity(&before);

        // §11.13: secondary-click the inactive row. The menu opens bound to
        // the exact source `TabId` under the tab source key; the selection,
        // the active view, the tab order, and the sessions stay untouched.
        let open = open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        assert_eq!(
            open.active_web_view,
            Some(second_view),
            "no selection change"
        );
        assert_eq!(
            open.active_tab_index,
            Some(1),
            "the neighbor stays selected"
        );
        assert_eq!(identity(&open), before_identity, "no mutation on open");
        assert!(
            !open.chrome_focused,
            "the settled origin is web focus; the menu restores it on close"
        );

        // Keyboard/pointer placement + focus: the open card is clamped fully
        // inside the window at the resolved rect, the menu owns the keyboard
        // focus handle, and the pointer-hover path shares the one cursor
        // with the keyboard (hovering a row selects it without activating).
        let [x, y, w, h] = open.tab_menu.anchor;
        assert!(w > 0.0 && h > 0.0, "the card has positive geometry");
        let [window_w, window_h] = open.window_content_size;
        if window_w > 0.0 {
            assert!(x >= 0.0 && x + w <= window_w + 0.5, "card clamped to width");
            assert!(
                y >= 0.0 && y + h <= window_h + 0.5,
                "card clamped to height"
            );
        }
        // Pointer hover onto the second enabled row moves the shared cursor
        // (keyboard cursor and hover share one selection); the bound source
        // and the snapshot identity stay untouched.
        let second_row = open
            .tab_menu
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.enabled)
            .nth(1)
            .map(|(i, _)| i)
            .expect("a second enabled row");
        menu_select(&mut client, second_row);
        flush_frame(&mut client);
        let hovered = client.snapshot().expect("hovered snapshot");
        assert_eq!(hovered.tab_menu.selected, Some(second_row));
        assert_eq!(hovered.tab_menu.bound_tab, Some(source_tab));
        assert_eq!(identity(&hovered), before_identity, "hover never mutates");
        // Keyboard traversal: Down/Up move the cursor skipping disabled
        // rows; Right opens the Move to Space submenu and Left closes it —
        // the bound tab never changes.
        let first_row = first_enabled_row(&open);
        assert_eq!(open.tab_menu.selected, Some(first_row));
        key(&mut client, "down");
        key(&mut client, "down");
        key(&mut client, "up");
        flush_frame(&mut client);
        let traversed = client.snapshot().expect("traversed snapshot");
        assert_eq!(
            traversed.tab_menu.bound_tab,
            Some(source_tab),
            "traversal preserves the source TabId"
        );
        assert_eq!(
            identity(&traversed),
            before_identity,
            "no mutation on traverse"
        );
        // The space submenu exists (two spaces), so the Move to Space row is
        // present; select it and open the submenu by Right.
        let space_row = traversed
            .tab_menu
            .rows
            .iter()
            .position(|row| row.id == "bbr-tab-menu-move-space")
            .expect("move-space row");
        menu_select(&mut client, space_row);
        key(&mut client, "right");
        flush_frame(&mut client);
        let sub = client.snapshot().expect("submenu snapshot");
        assert_eq!(sub.tab_menu.open_submenu, Some(space_row));
        assert_eq!(sub.tab_menu.submenu_selected, Some(0));
        assert_eq!(
            sub.tab_menu.bound_tab,
            Some(source_tab),
            "submenu traversal preserves the source TabId"
        );
        assert_eq!(
            identity(&sub),
            before_identity,
            "no mutation on submenu open"
        );
        // The destination card visibly renders: the production submenu_rect
        // is recomputed from the live window geometry (positive, fully on
        // screen, to the right of the parent card), and the submenu card +
        // destination row ids are part of the visible §15 id set.
        let submenu_rect = sub
            .tab_menu
            .submenu_rect
            .expect("the open submenu has a resolved placement");
        let [sx, sy, sw, sh] = submenu_rect;
        assert!(sw > 0.0 && sh > 0.0, "positive submenu geometry");
        // The destination card draws beside the parent card (right when the
        // window has room, flipped left at the right edge) — never on top of
        // it, never outside the window.
        let [ax, _, aw, _] = sub.tab_menu.anchor;
        let beside = (sx + sw <= ax + 0.5) || (sx >= ax + aw - 0.5);
        assert!(beside, "the submenu draws beside the parent card");
        let [window_w, window_h] = sub.window_content_size;
        if window_w > 0.0 {
            assert!(
                sx >= 0.0 && sx + sw <= window_w + 0.5,
                "submenu clamped to the window width"
            );
            assert!(
                sy >= 0.0 && sy + sh <= window_h + 0.5,
                "submenu clamped to the window height"
            );
        }
        assert!(
            sub.visible_ids
                .iter()
                .any(|id| id == "bbr-tab-menu-submenu"),
            "the submenu card id is visible"
        );
        assert!(
            sub.visible_ids
                .iter()
                .any(|id| id == "bbr-tab-menu-space-0"),
            "the destination row id is visible"
        );
        assert!(
            !sub.visible_ids
                .iter()
                .any(|id| id == "bbr-tab-menu-space-1"),
            "a same-profile second space has exactly one destination row"
        );
        key(&mut client, "left");
        flush_frame(&mut client);
        let back = client.snapshot().expect("back snapshot");
        assert_eq!(back.tab_menu.open_submenu, None, "Left closes the submenu");

        // Escape dismisses; the snapshot is byte-identical and web focus is
        // restored.
        key(&mut client, "escape");
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(identity(&closed), before_identity, "no mutation on dismiss");
        assert_eq!(closed.active_web_view, Some(second_view));
    }));

    finish(result, &mut harness, &socket, &token);
}

/// An outside pointer press dismisses with one action (the §13
/// one-pointer-action policy), leaving the source untouched.
#[test]
#[ignore]
fn outside_click_dismisses_with_one_action() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tab-menu-outside", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let before = client.snapshot().expect("before snapshot");
        let source_tab = active_space_tab_id(&before, 0);
        let before_identity = identity(&before);

        open_menu(&mut client, 0, source_tab, MENU_SOURCE_TAB, 140.0, 240.0);
        outside_click(&mut client);
        let closed = wait_menu_closed(&mut client, deadline());
        assert_eq!(
            identity(&closed),
            before_identity,
            "outside press never mutates"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}
