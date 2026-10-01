//! A real drag of a Favorite tile back into the tab list (task packet
//! `drag-back-out-of-pins`).

use bbr_testkit::protocol::{RealMouseButton, RealMousePhase};

use crate::e2e_common::real_input::*;
use crate::e2e_common::*;
use crate::{quit, start};

#[test]
#[ignore]
fn a_real_drag_takes_a_favorite_back_into_the_tabs() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-favorite-drag", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (_view, mut since) = start(&mut client, &token);
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        // [page_two★ | page_one]
        client
            .send_ok(&bbr_testkit::protocol::dsl::toggle_favorite_tab(1))
            .expect("favorite page two");
        let before = wait_for(&mut client, "the Favorite tile", |snapshot| {
            snapshot.favorites.len() == 1 && rect_of(snapshot, "bbr-fav-0").is_some()
        });
        assert_eq!(before.pinned_count, 1);
        let favorite_view = before.tabs[0].web_view;
        let from = center(rect_of(&before, "bbr-fav-0").expect("tile drawn"));

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
        // Past the activation distance: the ghost row opens above the
        // regular rows, so the rows move.
        let mut at = from;
        for _ in 0..3 {
            at.1 += 6.0;
            real_mouse(
                &mut client,
                RealMousePhase::Dragged,
                RealMouseButton::Left,
                at,
                &[],
            );
        }
        // Aim at the lower quarter of the regular row where it is drawn now:
        // the drop lands after it, at the end of the list.
        let moved = drawn(&mut client);
        assert!(moved.drag_from_favorite, "the tile press started the drag");
        let target = moved
            .tab_row_rects
            .iter()
            .find(|(row, _)| *row == 1)
            .map(|(_, rect)| *rect)
            .expect("the regular row is drawn");
        let to = (from.0, f64::from(target[1] + target[3] * 3.0 / 4.0));
        for step in 1..=6 {
            let t = f64::from(step) / 6.0;
            real_mouse(
                &mut client,
                RealMousePhase::Dragged,
                RealMouseButton::Left,
                (at.0, at.1 + (to.1 - at.1) * t),
                &[],
            );
        }
        // Release only once the insertion mark shows the end of the list,
        // as a hand does.
        let until = deadline();
        let mut nudge = 0.0;
        loop {
            let shown = drawn(&mut client);
            if shown.drag_unpin_slot == Some(2) && shown.drag_valid {
                break;
            }
            assert!(
                Instant::now() < until,
                "the drag never showed the end of the list: unpin {:?}, slot {:?}, rows {:?}",
                shown.drag_unpin_slot,
                shown.drag_slot,
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
        let after = wait_for(&mut client, "the Favorite back in the tabs", |snapshot| {
            snapshot.favorites.is_empty() && snapshot.pinned_count == 0
        });
        assert_eq!(after.tabs.len(), 2);
        assert_eq!(after.tabs[1].url, page_two, "dropped after page one");
        assert_eq!(after.tabs[1].web_view, favorite_view, "the session stays");

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
