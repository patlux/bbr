//! Drag-to-chip no-op evidence (DESIGN.md §10.9): dropping on the dragged
//! tab's own space, into the gap between chips, or onto the fixed
//! `[Downloads]`/`[+]` controls commits nothing (the drop is rejected
//! whole), and Escape cancels a targeted drag without committing. No
//! spurious move/reorder event may leave the app.

use super::*;
use super::{chip_center, chip_index, chip_span, drag_onto};
use bbr_testkit::protocol::{dsl, BrowserChange, Event};

fn move_events(client: &mut Client, since: u64) -> usize {
    client
        .attach(Some(since))
        .expect("no-op replay")
        .into_iter()
        .filter(|(_, event)| {
            matches!(
                event,
                Event::BrowserChanged {
                    kind: BrowserChange::TabMoved { .. } | BrowserChange::TabReordered { .. }
                }
            )
        })
        .count()
}

/// Own-space, gap, fixed-footer, and cancel drops all commit nothing: the
/// tab order, sessions, and selection stay byte-identical and no
/// move/reorder event leaves the app.
#[test]
#[ignore]
fn own_space_gap_fixed_footer_and_cancel_drops_are_noops() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-chip-noops", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        let default_space = initial.active_space;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(0)).expect("select row 0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);

        // A second space so the strip renders two chips with a real gap.
        client
            .send_ok(&dsl::create_space(initial.active_profile, "Reading"))
            .expect("create reading space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        let seeded = strip_presence(&mut client);
        assert_eq!(seeded.space_chips.len(), 2);
        let scroll = &seeded.space_strip_scroll;
        let default_center = chip_center(scroll, chip_index(&seeded, default_space));
        let reading_center = chip_center(scroll, chip_index(&seeded, reading));

        // 1. Own-space drop: the pointer targets the active space's own
        //    chip (a legal target), but the release commits nothing.
        let during_own = drag_onto(&mut client, 0, default_center.0, default_center.1);
        assert_eq!(
            during_own.drag_chip,
            Some(default_space),
            "the own chip is a typed target during the drag"
        );
        assert!(during_own.drag_valid);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end own-space drag");
        let after_own = client.snapshot().expect("own-space snapshot");
        assert_eq!(after_own.drag_source, None, "drag state cleared");
        assert_eq!(after_own.tabs.len(), 2, "nothing moved");
        assert_eq!(after_own.engine_sessions, 2, "no session mutation");
        assert_eq!(after_own.active_web_view, Some(first_view));

        // 2. Gap drop: between the two chips there is no target at all —
        //    the drop is rejected whole.
        let gap_x = (chip_span(scroll, 0)[1] + chip_span(scroll, 1)[0]) / 2.0;
        let during_gap = drag_onto(&mut client, 0, gap_x, reading_center.1);
        assert_eq!(during_gap.drag_chip, None, "the gap is a no-target zone");
        assert_eq!(during_gap.drag_slot, None);
        assert!(!during_gap.drag_valid, "the gap drop is rejected whole");
        client.send_ok(&dsl::end_tab_drag()).expect("end gap drag");
        let after_gap = client.snapshot().expect("gap snapshot");
        assert_eq!(after_gap.tabs.len(), 2, "gap drop moved nothing");

        // 3. Fixed-footer drop: the `[Downloads]` control left of the strip
        //    viewport is chrome, never a chip target.
        let downloads_x = scroll.viewport[0] - 20.0;
        let during_fixed = drag_onto(&mut client, 0, downloads_x, reading_center.1);
        assert_eq!(during_fixed.drag_chip, None, "fixed chrome is no target");
        assert!(!during_fixed.drag_valid);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end fixed drag");
        let after_fixed = client.snapshot().expect("fixed snapshot");
        assert_eq!(after_fixed.tabs.len(), 2, "fixed-chrome drop moved nothing");
        assert_eq!(after_fixed.engine_sessions, 2);

        // 4. Escape cancels a targeted drag: the chip resolves, but the
        //    cancel commits nothing and restores chrome focus.
        let during_cancel = drag_onto(&mut client, 0, reading_center.0, reading_center.1);
        assert_eq!(
            during_cancel.drag_chip,
            Some(reading),
            "the destination chip resolves before the cancel"
        );
        client
            .send_ok(&bbr_testkit::Request::CancelTabDrag)
            .expect("cancel drag");
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_eq!(cancelled.drag_source, None);
        assert_eq!(cancelled.drag_chip, None);
        assert_eq!(cancelled.tabs.len(), 2, "cancel moved nothing");
        assert!(
            cancelled.chrome_focused,
            "cancel restores the chrome domain"
        );

        // Across the whole no-op sequence exactly zero move/reorder events.
        let moved = move_events(&mut client, since);
        assert_eq!(moved, 0, "no-op drops emit no move/reorder event");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
