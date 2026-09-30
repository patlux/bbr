//! Every window after a restart; a closed window from its toast or ⇧⌘T.

use super::*;

#[test]
#[ignore]
fn every_window_comes_back_after_a_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-restore");
    let marker = fixtures.url("/window-marker.html");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        // The first window: two tabs on the default space, the first one
        // selected.
        open_tab(&mut client, &mut since, &page_two, None);
        client
            .send_ok(&bbr_testkit::protocol::dsl::select_tab(0))
            .expect("select page one");
        let first = settle(&mut client, "page one", |s| s.active_tab_index == Some(0));
        let home = first.active_space;
        let profile = first.active_profile;

        // The second window: its own space with its own tab, rail hidden.
        let (second, _) = new_window(&mut client);
        client
            .send_ok(&bbr_testkit::protocol::dsl::create_space(profile, "Work"))
            .expect("create Work");
        let work = settle(&mut client, "Work", |s| {
            s.spaces.iter().any(|space| space.title == "Work")
        })
        .spaces
        .iter()
        .find(|space| space.title == "Work")
        .unwrap()
        .space;
        client
            .send_ok(&bbr_testkit::protocol::dsl::activate_space(work))
            .expect("activate Work");
        settle(&mut client, "Work active", |s| s.active_space == work);
        open_tab(&mut client, &mut since, &marker, None);
        keystroke(&mut client, "cmd-s");
        let saved = settle(&mut client, "the second window's rail hidden", |s| {
            !entry(s, second).sidebar_visible && entry(s, second).selected_index == Some(0)
        });
        let saved_first = entry(&saved, 0);
        let saved_second = entry(&saved, second);
        assert_eq!(saved_first.active_space, Some(home.0));
        assert_eq!(saved_first.selected_index, Some(0));
        assert!(saved_first.sidebar_visible);
        assert_eq!(saved_second.active_space, Some(work.0));
        assert!(saved_first.content_rect.is_some() && saved_second.content_rect.is_some());
        assert!(
            !close(saved_first.content_rect, saved_second.content_rect),
            "the windows sit apart"
        );

        // One window's save writes every window.
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush");
        (saved_first, saved_second)
    }));
    let (saved_first, saved_second) = match result {
        Ok(saved) => {
            harness.success = false;
            saved
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    };
    drop(harness);

    let (mut harness, socket, token) =
        spawn_harness("windows-restore", &fixtures.url("/page1.html"));
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let restored = settle(&mut client, "both windows", |s| {
            s.windows.len() == 2 && s.windows.iter().all(|w| w.selected_view.is_some())
        });
        assert_eq!(current(&restored), 0, "the first window is current");
        let first = entry(&restored, 0);
        let second = restored
            .windows
            .iter()
            .find(|w| w.window != 0)
            .cloned()
            .unwrap();
        assert_eq!(first.active_space, saved_first.active_space);
        assert_eq!(first.selected_index, saved_first.selected_index);
        assert!(first.sidebar_visible);
        assert!(
            close(first.content_rect, saved_first.content_rect),
            "{:?} vs {:?}",
            first.content_rect,
            saved_first.content_rect
        );
        assert_eq!(second.active_space, saved_second.active_space);
        assert_eq!(second.selected_index, saved_second.selected_index);
        assert!(!second.sidebar_visible, "its own rail came back");
        assert_eq!(
            second.selected_root,
            Some(second.window),
            "its page is in its root"
        );
        assert!(
            close(second.content_rect, saved_second.content_rect),
            "{:?} vs {:?}",
            second.content_rect,
            saved_second.content_rect
        );
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_closed_window_comes_back_from_its_toast_or_cmd_shift_t() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-reopen");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let (second, _) = new_window(&mut client);
        let view = open_tab(&mut client, &mut since, &page_two, None);
        let before = settle(&mut client, "the second window's page", |s| {
            entry(s, second).selected_view == Some(view)
        });
        let rect = entry(&before, second).content_rect;

        // ⇧⌘W: the window closes, the first window says so and offers
        // Reopen.
        keystroke(&mut client, "cmd-shift-w");
        let closed = settle(&mut client, "the toast", |s| {
            s.windows.len() == 1
                && s.toast
                    .as_ref()
                    .is_some_and(|t| t.message == "Window closed" && !t.leaving)
        });
        assert_eq!(closed.toast.unwrap().action.as_deref(), Some("Reopen"));
        assert_eq!(closed.tabs.len(), 2, "no tab closed");

        // Reopen: the same page in a new window at the old place.
        client
            .send_ok(&bbr_testkit::Request::ToastAction)
            .expect("Reopen");
        let back = settle(&mut client, "the reopened window", |s| {
            s.windows.len() == 2 && s.windows.iter().any(|w| w.selected_view == Some(view))
        });
        let reopened = back
            .windows
            .iter()
            .find(|w| w.selected_view == Some(view))
            .cloned()
            .unwrap();
        assert!(reopened.current, "the reopened window takes over");
        assert_eq!(
            reopened.selected_root,
            Some(reopened.window),
            "never reloaded, moved in"
        );
        assert!(
            close(reopened.content_rect, rect),
            "{:?} vs {rect:?}",
            reopened.content_rect
        );
        assert!(back.toast.is_none(), "the toast has done its part");

        // ⇧⌘W then ⇧⌘T: nothing was closed after the window, so the window
        // comes back.
        keystroke(&mut client, "cmd-shift-w");
        settle(&mut client, "closed again", |s| s.windows.len() == 1);
        keystroke(&mut client, "cmd-shift-t");
        settle(&mut client, "back by cmd-shift-t", |s| {
            s.windows.len() == 2 && s.windows.iter().any(|w| w.selected_view == Some(view))
        });

        // ⇧⌘W, then a tab closes: ⇧⌘T reopens the tab, not the window.
        keystroke(&mut client, "cmd-shift-w");
        let one = settle(&mut client, "closed a third time", |s| s.windows.len() == 1);
        let tabs = one.tabs.len();
        client
            .send_ok(&bbr_testkit::protocol::dsl::close_tab(1))
            .expect("close a tab");
        settle(&mut client, "a tab closed", |s| s.tabs.len() == tabs - 1);
        keystroke(&mut client, "cmd-shift-t");
        let reopened_tab = settle(&mut client, "the tab back", |s| s.tabs.len() == tabs);
        assert_eq!(reopened_tab.windows.len(), 1, "the tab, not the window");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
