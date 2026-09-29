//! Blueprint D14 (DESIGN.md §11.14): the Back/Forward history menu over a
//! real WKWebView session history. A secondary click on Back lists the
//! back entries nearest first from WebKit's own back-forward list; a row
//! jumps straight to its entry (two steps in one); Forward lists the
//! other side; holding the primary button on Back opens the menu without
//! stepping; a short press-and-release over Back steps once. No sleeps:
//! every step synchronizes on real WebKit titles or snapshot state under a
//! deadline.

use super::*;

const PAGE_THREE_TITLE: &str = "BBR Background X";

fn menu(client: &mut Client) -> Option<bbr_testkit::protocol::NavHistorySnapshot> {
    flush_frame(client);
    client.snapshot().expect("menu snapshot").nav_history
}

#[test]
#[ignore]
fn back_and_forward_list_the_session_history_and_jump_in_one_step() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let page_three = fixtures.url("/bg-x.html");
    let (mut harness, socket, token) = spawn_harness("titlebar-history", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate two");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        client
            .send_ok(&dsl::navigate(page_three.clone()))
            .expect("navigate three");
        since = wait_title(&mut client, view, PAGE_THREE_TITLE, since);

        // A secondary click on Back lists the back side, nearest first.
        client
            .send_ok(&bbr_testkit::Request::RightClickTitlebarControl {
                control: "back".into(),
                x: 200.0,
                y: 30.0,
            })
            .expect("right-click back");
        let back = menu(&mut client).expect("history menu open");
        assert_eq!(back.direction, "back");
        assert_eq!(back.labels, vec![PAGE_TWO_TITLE, PAGE_ONE_TITLE]);
        assert_eq!(back.urls, vec![page_two.clone(), page_one.clone()]);
        assert_eq!(back.selected, Some(0));
        let snapshot = client.snapshot().expect("portal snapshot");
        assert!(snapshot.portal.visible, "the menu renders in the portal");
        assert!(snapshot
            .portal
            .ids
            .iter()
            .any(|id| id == "bbr-nav-history-menu"));
        assert_eq!(
            snapshot.active_web_view,
            Some(view),
            "opening never navigates"
        );

        // The second row jumps two entries back in one step.
        client
            .send_ok(&bbr_testkit::Request::NavHistoryActivate { index: 1 })
            .expect("activate page one");
        since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        assert!(menu(&mut client).is_none(), "the jump closes the menu");

        // Forward lists the other side, nearest first; Escape closes it.
        client
            .send_ok(&bbr_testkit::Request::RightClickTitlebarControl {
                control: "forward".into(),
                x: 230.0,
                y: 30.0,
            })
            .expect("right-click forward");
        let forward = menu(&mut client).expect("forward menu open");
        assert_eq!(forward.direction, "forward");
        assert_eq!(forward.labels, vec![PAGE_TWO_TITLE, PAGE_THREE_TITLE]);
        keystroke(&mut client, "escape");
        assert!(menu(&mut client).is_none(), "Escape closes the menu");

        // Holding the primary button on Forward opens the menu without
        // stepping; the release over the control does not navigate.
        client
            .send_ok(&dsl::hover_titlebar_control("forward"))
            .expect("hover forward");
        client
            .send_ok(&bbr_testkit::Request::PressTitlebarControlAt {
                control: "forward".into(),
                x: 230.0,
                y: 30.0,
            })
            .expect("press forward");
        let until = Instant::now() + Duration::from_secs(10);
        let held = loop {
            if let Some(held) = menu(&mut client) {
                break held;
            }
            assert!(Instant::now() < until, "the hold never opened the menu");
        };
        assert_eq!(held.direction, "forward");
        client
            .send_ok(&dsl::release_titlebar_control())
            .expect("release forward");
        flush_frame(&mut client);
        let after_hold = client.snapshot().expect("after hold snapshot");
        assert_eq!(after_hold.title.as_deref(), Some(PAGE_ONE_TITLE), "no step");
        client
            .send_ok(&bbr_testkit::Request::NavHistoryClose)
            .expect("close");

        // A short press released over Forward steps once.
        client
            .send_ok(&dsl::hover_titlebar_control("forward"))
            .expect("hover forward again");
        client
            .send_ok(&bbr_testkit::Request::PressTitlebarControlAt {
                control: "forward".into(),
                x: 230.0,
                y: 30.0,
            })
            .expect("press forward");
        client
            .send_ok(&dsl::release_titlebar_control())
            .expect("release forward");
        wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        assert!(menu(&mut client).is_none(), "a click never opens the menu");

        // Blueprint D10: the last row, Show Full History, opens History.
        client
            .send_ok(&bbr_testkit::Request::RightClickTitlebarControl {
                control: "back".into(),
                x: 200.0,
                y: 30.0,
            })
            .expect("right-click back again");
        let back = menu(&mut client).expect("back menu open");
        assert!(
            back.show_full_history,
            "Show Full History follows the entries"
        );
        client
            .send_ok(&bbr_testkit::Request::NavHistoryActivate {
                index: back.labels.len(),
            })
            .expect("show full history");
        assert!(menu(&mut client).is_none(), "the row closes the menu");
        let history = client.snapshot().expect("history snapshot").history;
        assert!(history.is_some(), "Show Full History opens History");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
