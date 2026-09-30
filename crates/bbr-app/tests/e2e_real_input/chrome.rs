//! Real clicks on the browser chrome.

use crate::e2e_common::real_input::*;
use crate::e2e_common::*;
use crate::{quit, start};

#[test]
#[ignore]
fn chrome_controls_answer_real_clicks() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-chrome", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);

        // Titlebar Back.
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        wait_for(&mut client, "Back enabled", |s| s.titlebar.can_go_back);
        real_click(&mut client, "bbr-nav-back");
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);

        // Tab rows: select the other row, then hover a row and press its
        // close control.
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        assert_eq!(drawn(&mut client).active_tab_index, Some(1));
        real_click(&mut client, "bbr-tab-row-0");
        wait_for(&mut client, "row 0 selected", |s| {
            s.active_tab_index == Some(0)
        });
        let row = rect_of(&drawn(&mut client), "bbr-tab-row-1").expect("row 1 drawn");
        real_hover(&mut client, center(row));
        real_click(&mut client, "bbr-tab-close-1");
        wait_for(&mut client, "row 1 closed", |s| s.tabs.len() == 1);

        // Footer Downloads opens the panel; the next press outside it
        // closes it (DESIGN §13: one pointer action dismisses).
        real_click(&mut client, "bbr-downloads");
        wait_for(&mut client, "downloads open", |s| s.downloads.open);
        real_click(&mut client, "bbr-tab-row-0");
        wait_for(&mut client, "downloads closed", |s| !s.downloads.open);

        // Footer `+` opens its menu; Escape closes it.
        real_click(&mut client, "bbr-plus");
        wait_for(&mut client, "plus menu open", |s| s.plus_menu_open);
        real_key(&mut client, "escape");
        wait_for(&mut client, "plus menu closed", |s| !s.plus_menu_open);

        // A space chip activates its space.
        let before = drawn(&mut client);
        let first = before.active_space;
        let profile = before.active_profile;
        client
            .send_ok(&dsl::create_space(profile, "Second"))
            .expect("create space");
        let (next, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = next;
        let BrowserChange::SpaceCreated { space: second, .. } = change else {
            unreachable!("matched SpaceCreated");
        };
        client
            .send_ok(&dsl::activate_space(second))
            .expect("activate the new space");
        wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: second },
            since,
        );
        real_click(&mut client, &format!("bbr-space-chip-{}", first.0));
        wait_for(&mut client, "the first space active again", |s| {
            s.active_space == first
        });

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_real_drag_moves_a_tab_row() {
    use bbr_testkit::protocol::{RealMouseButton, RealMousePhase};

    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-drag", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (_view, mut since) = start(&mut client, &token);
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let before = drawn(&mut client);
        assert_eq!(before.tabs[1].url, page_two);
        let from = center(rect_of(&before, "bbr-tab-row-1").expect("row 1 drawn"));

        real_mouse(
            &mut client,
            RealMousePhase::Moved,
            RealMouseButton::Left,
            from,
            &[],
        );
        real_mouse(
            &mut client,
            RealMousePhase::Down,
            RealMouseButton::Left,
            from,
            &[],
        );
        // Past the activation distance: the drag starts and the rail grows
        // its offers above the pressed row, so the rows move.
        let mut at = from;
        for _ in 0..3 {
            at.1 -= 6.0;
            real_mouse(
                &mut client,
                RealMousePhase::Dragged,
                RealMouseButton::Left,
                at,
                &[],
            );
        }
        // Aim where row 0 is drawn now: its upper quarter drops before it.
        let moved = drawn(&mut client);
        let target = rect_of(&moved, "bbr-tab-row-0").expect("row 0 drawn");
        let to = (from.0, f64::from(target[1] + target[3] / 4.0));
        for step in 1..=6 {
            let t = f64::from(step) / 6.0;
            let point = (at.0, at.1 + (to.1 - at.1) * t);
            real_mouse(
                &mut client,
                RealMousePhase::Dragged,
                RealMouseButton::Left,
                point,
                &[],
            );
        }
        // Release only once the insertion mark shows the drop before row 0,
        // as a hand does: the target resolves against the rows the last
        // frame drew, so the pointer keeps moving there until it shows.
        let until = deadline();
        let mut nudge = 0.0;
        loop {
            let shown = drawn(&mut client);
            if shown.drag_slot == Some(0) && shown.drag_valid {
                break;
            }
            assert!(
                Instant::now() < until,
                "the drag never showed slot 0: slot {:?}, valid {}, rows {:?}",
                shown.drag_slot,
                shown.drag_valid,
                shown.tab_row_rects
            );
            nudge = if nudge == 0.0 { 1.0 } else { 0.0 };
            real_mouse(
                &mut client,
                RealMousePhase::Dragged,
                RealMouseButton::Left,
                (to.0, to.1 + nudge),
                &[],
            );
        }
        real_mouse(
            &mut client,
            RealMousePhase::Up,
            RealMouseButton::Left,
            to,
            &[],
        );
        let after = drawn(&mut client);
        assert!(
            after.tabs.len() == 2 && after.tabs[0].url == page_two,
            "the dragged row is first: tabs {:?}, rows before drop {:?}, target {target:?}",
            after
                .tabs
                .iter()
                .map(|tab| (&tab.url, tab.pinned))
                .collect::<Vec<_>>(),
            moved.tab_row_rects
        );

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
