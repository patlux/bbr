//! Blueprint D21 (DESIGN §9): ⌥⌘S moves keyboard focus from the page into
//! the rail on the selected row; ↑↓ move, Return selects and returns focus
//! to the page. With the rail hidden, ⌥⌘S shows it first. Every step
//! synchronizes on semantic events or snapshot state under a deadline.

use super::*;

fn keystroke(client: &mut Client, key: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: key.into(),
        })
        .expect("keystroke");
}

#[test]
#[ignore]
fn option_command_s_moves_focus_into_the_rail_and_return_goes_back_to_the_page() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("tabs-rail-focus", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = wait_for_window_settled(&mut client, deadline());
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FocusWeb)
            .expect("page focus");
        since = wait_chrome_focus(&mut client, false, since);

        // ⌥⌘S: the rail takes keyboard focus on the selected row.
        keystroke(&mut client, "alt-cmd-s");
        since = wait_chrome_focus(&mut client, true, since);
        let rail = client.snapshot().expect("rail focus snapshot");
        assert_eq!(
            rail.active_tab_index,
            Some(1),
            "focus lands on the selected row"
        );
        assert_eq!(rail.command_bar_mode, CommandBarMode::Inactive);

        // ↑ moves to the first row; Return selects it and returns focus to
        // the page.
        keystroke(&mut client, "up");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        keystroke(&mut client, "enter");
        since = wait_chrome_focus(&mut client, false, since);
        let back = client.snapshot().expect("page focus snapshot");
        assert_eq!(back.active_tab_index, Some(0));
        assert_eq!(back.active_web_view, Some(first_view));
        assert_eq!(back.engine_sessions, 2, "no session was recreated");

        // With the rail hidden, ⌥⌘S shows it and focuses the selected row.
        keystroke(&mut client, "cmd-s");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush collapse");
        assert!(!client.snapshot().expect("collapsed").sidebar_visible);
        keystroke(&mut client, "alt-cmd-s");
        let _ = wait_chrome_focus(&mut client, true, since);
        let shown = client.snapshot().expect("shown snapshot");
        assert!(shown.sidebar_visible, "the rail shows to take focus");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
