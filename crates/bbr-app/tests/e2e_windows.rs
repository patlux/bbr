//! Real-app several-windows E2E (blueprint D19, task packet
//! `multi-window.md`): ⌘N opens a window on the current space with New Tab
//! ready; each window keeps its own selection; selecting another window's
//! page moves the live WKWebView into this window's root without a reload;
//! ⇧⌘W closes a window and keeps its tabs, and closing the last window
//! quits; the menu bar and the Dock menu open windows; another window's
//! change of shared state re-renders a window, an idle one does not. Every
//! step waits on typed snapshot state under a deadline.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{CommandSurface, Snapshot, WindowEntry};
use e2e_common::*;

fn settle(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        let snapshot = client.snapshot().expect("windows snapshot");
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "{what} never settled; windows: {:?}",
            snapshot.windows
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn keystroke(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .unwrap_or_else(|error| panic!("{keystroke}: {error}"));
}

fn invoke(client: &mut Client, menu: &str, item: &str) {
    client
        .send_ok(&bbr_testkit::Request::InvokeMenuItem {
            menu: menu.into(),
            item: item.into(),
        })
        .unwrap_or_else(|error| panic!("{menu} › {item}: {error}"));
}

fn select_window(client: &mut Client, window: u64) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::SelectWindow { window })
        .expect("select window");
    settle(client, "the selected window", |s| {
        s.windows.iter().any(|w| w.window == window && w.current)
    })
}

fn entry(snapshot: &Snapshot, window: u64) -> WindowEntry {
    snapshot
        .windows
        .iter()
        .find(|w| w.window == window)
        .cloned()
        .unwrap_or_else(|| panic!("window {window} in {:?}", snapshot.windows))
}

fn current(snapshot: &Snapshot) -> u64 {
    snapshot
        .windows
        .iter()
        .find(|w| w.current)
        .expect("a current window")
        .window
}

/// Open a window with ⌘N and close its New Tab palette; returns its key.
fn new_window(client: &mut Client) -> (u64, Snapshot) {
    let before = client.snapshot().expect("before new window").windows.len();
    keystroke(client, "cmd-n");
    let opened = settle(client, "the new window", |s| {
        s.windows.len() == before + 1 && s.command_surface == CommandSurface::Palette
    });
    let key = current(&opened);
    keystroke(client, "escape");
    settle(client, "the palette closed", |s| {
        s.command_surface == CommandSurface::None
    });
    (key, opened)
}

fn start(
    tag: &str,
) -> (
    FixtureServer,
    String,
    String,
    Harness,
    std::path::PathBuf,
    String,
) {
    let (fixtures, page_one, page_two) = fixtures();
    let (harness, socket, token) = spawn_harness(tag, &page_one);
    (fixtures, page_one, page_two, harness, socket, token)
}

fn ready(client: &mut Client, token: &str) -> u64 {
    client.hello(token).expect("hello");
    let replay = client.attach(None).expect("attach");
    let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
    // The first page has finished loading: what follows starts from rest.
    settle(client, "the first window", |s| {
        s.windows.len() == 1
            && s.active_web_view.is_some()
            && s.tabs
                .first()
                .is_some_and(|tab| tab.title == PAGE_ONE_TITLE && !tab.loading)
    });
    since
}

#[test]
#[ignore]
fn a_new_window_selects_its_own_tab_and_takes_pages_without_reload() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, _page_two, mut harness, socket, token) = start("windows-select");
    let marker = fixtures.url("/window-marker.html");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let first_view = client.snapshot().unwrap().active_web_view.unwrap();
        let marker_view = open_tab(&mut client, &mut since, &marker, None);
        let loaded = settle(&mut client, "the marker title", |s| {
            s.tabs
                .get(1)
                .is_some_and(|tab| tab.title.starts_with("Window marker "))
        });
        let title = loaded.tabs[1].title.clone();
        let space = loaded.active_space;

        // ⌘N: a window on the same space, nothing selected, New Tab ready.
        let (second, opened) = new_window(&mut client);
        assert_ne!(second, 0);
        assert_eq!(opened.active_space, space);
        assert_eq!(opened.active_tab_index, None, "the empty state");
        assert_eq!(opened.tabs.len(), 2, "the space's tabs are shared");
        let first = entry(&opened, 0);
        assert_eq!(first.selected_index, Some(1));
        assert_eq!(first.selected_view, Some(marker_view));
        assert_eq!(first.selected_root, Some(0));

        // Selecting the first window's page moves it here: the same
        // session, re-parented into this window's root, never reloaded.
        client
            .send_ok(&dsl::select_tab(1))
            .expect("select the marker");
        let moved = settle(&mut client, "the moved page", |s| {
            s.active_web_view == Some(marker_view)
        });
        assert_eq!(entry(&moved, second).selected_root, Some(second));
        assert_eq!(
            entry(&moved, 0).selected_index,
            None,
            "the first window lost it"
        );
        let until = deadline();
        loop {
            client
                .send_ok(&bbr_testkit::Request::ProbeWebContent { view: marker_view })
                .expect("probe the moved page");
            let snapshot = client.snapshot().expect("probe snapshot");
            if let Some(probe) = snapshot.web_content_probe.as_ref() {
                if probe.view == marker_view && probe.ready_state == "complete" {
                    assert_eq!(probe.dom_title, title, "moved without a reload");
                    break;
                }
            }
            assert!(
                Instant::now() < until,
                "the moved page never answered the probe"
            );
            std::thread::sleep(Duration::from_millis(20));
        }

        // The first window shows its space's empty state, then selects its
        // own tab; the second keeps the marker.
        let first_now = select_window(&mut client, 0);
        assert_eq!(first_now.active_tab_index, None);
        assert_eq!(first_now.active_web_view, None);
        client
            .send_ok(&dsl::select_tab(0))
            .expect("select page one");
        let own = settle(&mut client, "page one in the first window", |s| {
            s.active_web_view == Some(first_view)
        });
        assert_eq!(entry(&own, 0).selected_root, Some(0));
        let second_now = select_window(&mut client, second);
        assert_eq!(second_now.active_web_view, Some(marker_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn closing_a_window_keeps_its_tabs_and_the_last_close_quits() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-close");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let (second, _) = new_window(&mut client);
        let opened = open_tab(&mut client, &mut since, &page_two, None);
        let two = settle(&mut client, "the second window's tab", |s| {
            s.active_web_view == Some(opened) && s.tabs.len() == 2
        });
        assert_eq!(entry(&two, second).selected_view, Some(opened));

        // ⇧⌘W closes the second window: every tab stays, BBR keeps
        // running in the first window.
        keystroke(&mut client, "cmd-shift-w");
        let one = settle(&mut client, "one window left", |s| s.windows.len() == 1);
        assert_eq!(current(&one), 0);
        assert_eq!(one.tabs.len(), 2, "closing a window closes no tab");

        // Closing the last window saves and quits.
        client
            .send_ok(&bbr_testkit::Request::CloseWindow)
            .expect("CloseWindow response must be flushed before teardown");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "the last window's close quits: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn the_menu_bar_and_the_dock_open_and_close_windows() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, _page_one, _page_two, mut harness, socket, token) = start("windows-menus");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        ready(&mut client, &token);
        let menus = settle(&mut client, "menus", |s| !s.menu_bar.is_empty());
        let file = menus
            .menu_bar
            .iter()
            .find(|menu| menu.name == "File")
            .expect("File menu");
        let names: Vec<&str> = file
            .items
            .iter()
            .filter_map(|item| item.name.as_deref())
            .collect();
        assert!(names.contains(&"New Window") && names.contains(&"Close Window"));
        let dock: Vec<&str> = menus
            .dock_menu
            .iter()
            .filter_map(|item| item.name.as_deref())
            .collect();
        assert!(dock.contains(&"New Window"));

        invoke(&mut client, "File", "New Window");
        settle(&mut client, "two windows", |s| s.windows.len() == 2);
        invoke(&mut client, "Dock", "New Window");
        let three = settle(&mut client, "three windows", |s| s.windows.len() == 3);
        let third = current(&three);
        invoke(&mut client, "File", "Close Window");
        let two = settle(&mut client, "two windows again", |s| s.windows.len() == 2);
        assert!(two.windows.iter().all(|w| w.window != third));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_window_redraws_for_another_windows_change_and_stays_quiet_when_idle() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-refresh");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let (second, _) = new_window(&mut client);
        let page_one = client.snapshot().expect("tabs").tabs.len();
        assert_eq!(page_one, 1);
        let before = select_window(&mut client, 0).window_refreshes;

        // Idle: the second window's cadence keeps ticking (and notifying),
        // but nothing the first window draws changes.
        select_window(&mut client, second);
        std::thread::sleep(Duration::from_millis(1500));
        let idle = select_window(&mut client, 0).window_refreshes;
        assert_eq!(idle, before, "an idle window never redraws the other");

        // A tab opened in the second window is shared state the first
        // window draws.
        select_window(&mut client, second);
        open_tab(&mut client, &mut since, &page_two, None);
        let first = select_window(&mut client, 0);
        assert!(
            first.window_refreshes > idle,
            "the change redrew the first window"
        );
        assert_eq!(first.tabs.len(), 2);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
