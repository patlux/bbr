//! Real-app several-windows E2E (blueprint D19, task packet
//! `multi-window.md`): ⌘N opens a window on the current space with New Tab
//! ready; each window keeps its own selection; selecting another window's
//! page moves the live WKWebView into this window's root without a reload;
//! ⇧⌘W closes a window and keeps its tabs, and closing the last window
//! quits; the menu bar and the Dock menu open windows; another window's
//! change of shared state re-renders a window, an idle one does not; every
//! window comes back after a restart; a closed window comes back from its
//! toast or ⇧⌘T; a tab dragged onto another window's rail or card moves
//! there. Every step waits on typed snapshot state under a deadline.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod drag;
mod lifecycle;
mod restore;

use bbr_testkit::protocol::{CommandSurface, Snapshot, WindowEntry};
use e2e_common::*;

pub(crate) fn settle(
    client: &mut Client,
    what: &str,
    ready: impl Fn(&Snapshot) -> bool,
) -> Snapshot {
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

pub(crate) fn keystroke(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .unwrap_or_else(|error| panic!("{keystroke}: {error}"));
}

pub(crate) fn invoke(client: &mut Client, menu: &str, item: &str) {
    client
        .send_ok(&bbr_testkit::Request::InvokeMenuItem {
            menu: menu.into(),
            item: item.into(),
        })
        .unwrap_or_else(|error| panic!("{menu} › {item}: {error}"));
}

pub(crate) fn select_window(client: &mut Client, window: u64) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::SelectWindow { window })
        .expect("select window");
    settle(client, "the selected window", |s| {
        s.windows.iter().any(|w| w.window == window && w.current)
    })
}

pub(crate) fn entry(snapshot: &Snapshot, window: u64) -> WindowEntry {
    snapshot
        .windows
        .iter()
        .find(|w| w.window == window)
        .cloned()
        .unwrap_or_else(|| panic!("window {window} in {:?}", snapshot.windows))
}

pub(crate) fn current(snapshot: &Snapshot) -> u64 {
    snapshot
        .windows
        .iter()
        .find(|w| w.current)
        .expect("a current window")
        .window
}

/// Open a window with ⌘N and close its New Tab palette; returns its key.
pub(crate) fn new_window(client: &mut Client) -> (u64, Snapshot) {
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

pub(crate) fn start(
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

pub(crate) fn ready(client: &mut Client, token: &str) -> u64 {
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

pub(crate) fn close(a: Option<[f64; 4]>, b: Option<[f64; 4]>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1.0),
        _ => false,
    }
}
