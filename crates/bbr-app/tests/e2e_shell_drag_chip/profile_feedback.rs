//! Blueprint D07 (DESIGN §11.3/§12): a cross-profile move is announced. The
//! drag names the destination profile before the drop; after the drop a
//! toast says the page reloaded in that profile, and Undo moves the tab
//! back — recreating it once more, in the source profile's store.

use super::*;
use super::{chip_center, chip_index, drag_onto};
use bbr_testkit::protocol::{dsl, BrowserChange};

#[test]
#[ignore]
fn cross_profile_drop_names_the_destination_and_undo_moves_the_tab_back() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-chip-feedback", &page_one);
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
        let _second = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::create_profile("Work", "Clients"))
            .expect("create the Work profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let work_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };

        // Over the Work chip, the drag names the destination profile.
        let seeded = strip_presence(&mut client);
        let target = chip_center(&seeded.space_strip_scroll, chip_index(&seeded, work_space));
        let during = drag_onto(&mut client, 1, target.0, target.1);
        assert_eq!(during.drag_chip, Some(work_space));
        assert_eq!(
            during.drag_destination.as_deref(),
            Some("Move to Clients · Profile Work")
        );

        // The drop recreates the page in Work; the toast says so.
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: work_space,
                recreated: true,
            },
            since,
        );
        let until = deadline();
        let moved = loop {
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush");
            let snapshot = client.snapshot().expect("toast snapshot");
            if snapshot.toast.is_some() {
                break snapshot;
            }
            assert!(Instant::now() < until, "no move toast");
        };
        assert_eq!(
            moved.drag_destination, None,
            "the label leaves with the drag"
        );
        let toast = moved.toast.expect("toast");
        assert_eq!(
            toast.message,
            "Reloaded in Profile · Work — sign-ins may differ"
        );
        assert_eq!(toast.action.as_deref(), Some("Undo"));
        assert_eq!(moved.tabs.len(), 1, "the source keeps page one");

        // Undo moves the tab back and recreates it in the source profile.
        client
            .send_ok(&bbr_testkit::Request::ToastAction)
            .expect("undo");
        wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: default_space,
                recreated: true,
            },
            since,
        );
        let back = client.snapshot().expect("undone snapshot");
        assert_eq!(back.tabs.len(), 2, "the tab is back in its space");
        assert!(back.toast.is_none(), "Undo removes the toast");
        assert_eq!(back.active_space, default_space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
