//! Shared harness support and typed assertions for the Tab/Favorite menu
//! E2E (task packet `tab-favorite-context-menus`).
//!
//! The menu is ordinary GPUI chrome over the sidebar (never a portal
//! surface), so the harness drives the exact production seams the sidebar
//! rows and the menu card listeners call: `RightClickRow` /
//! `RightClickFavorite` construct the real secondary-button `MouseDownEvent`
//! the row listeners would receive (anchored at the pointer), and the
//! `TabMenu*` requests call the exact app methods the card's hover/press
//! listeners and the Escape/outside-press close paths call. Keyboard input
//! goes through GPUI's real `DispatchKeystroke` path.

use std::time::Instant;

use crate::e2e_common::*;
use bbr_testkit::protocol::Snapshot;

/// The §15 presentation source keys the menu registers (`bbr-tab-menu` for
/// a tab-row open, `bbr-favorite-menu` for a Favorite tile open).
pub(super) const MENU_SOURCE_TAB: &str = "bbr-tab-menu";
pub(super) const MENU_SOURCE_FAVORITE: &str = "bbr-favorite-menu";

/// The first enabled row of the deterministic row model (render order):
/// `Move to Folder` when the source space has folders, else `Move to
/// Space`, else `Reload Tab`. The keyboard cursor lands here at open.
pub(super) fn first_enabled_row(snapshot: &Snapshot) -> usize {
    snapshot
        .tab_menu
        .rows
        .iter()
        .position(|row| row.enabled)
        .expect("the menu always has an enabled row")
}

/// The exact `TabId` of the active space's tab at `index` (the row the
/// right-click opens on).
pub(super) fn active_space_tab_id(snapshot: &Snapshot, index: usize) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.space == snapshot.active_space && tab.index == index)
        .expect("active-space tab exists")
        .id
}

/// The current space-local index of `tab` in the active space (the row a
/// later right-click must open on — mutations like pin/unpin reorder rows,
/// so callers re-resolve the row from the latest snapshot instead of
/// assuming a stale index).
pub(super) fn active_space_row_of(snapshot: &Snapshot, tab: u64) -> usize {
    snapshot
        .all_tabs
        .iter()
        .find(|candidate| candidate.id == tab && candidate.space == snapshot.active_space)
        .expect("active-space tab exists")
        .index
}

/// Poll until the space collection reaches `count` (deterministic mutation
/// evidence for the setup that makes the `Move to Space` submenu genuinely
/// present).
pub(super) fn wait_for_space_count(client: &mut Client, count: usize, deadline: Instant) {
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush while waiting for spaces");
        let snapshot = client.snapshot().expect("space count snapshot");
        if snapshot.spaces.len() == count {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "space collection never reached {count}: {:?}",
            snapshot.spaces.len()
        );
        std::thread::yield_now();
    }
}

/// Create a second space in the active profile (the setup that makes the
/// `Move to Space` submenu and its same-profile destination row present).
pub(super) fn create_second_space(client: &mut Client, deadline: Instant) {
    let profile = client.snapshot().expect("profile snapshot").active_profile;
    client
        .send_ok(&bbr_testkit::Request::CreateSpace {
            profile,
            title: "Second".into(),
        })
        .expect("create second space");
    wait_for_space_count(client, 2, deadline);
}

/// Poll until the folder collection reaches `count` (deterministic mutation
/// evidence for the setup that makes the `Move to Folder` submenu present).
pub(super) fn wait_for_folder_count(client: &mut Client, count: usize, deadline: Instant) {
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush while waiting for folders");
        let snapshot = client.snapshot().expect("folder count snapshot");
        if snapshot.folders.len() == count {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "folder collection never reached {count}: {:?}",
            snapshot.folders.len()
        );
        std::thread::yield_now();
    }
}

/// Dispatch one keystroke through GPUI's real input path: `up`/`down`/
/// `left`/`right`/`enter` reach the menu's key handler while the menu owns
/// keyboard focus; `escape` routes through the global binding that closes an
/// open menu.
pub(super) fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

pub(super) fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// Secondary-click on the tab row at `index` (the production pointer path).
pub(super) fn right_click_row(client: &mut Client, index: usize, x: f32, y: f32) {
    client
        .send_ok(&bbr_testkit::Request::RightClickRow { index, x, y })
        .expect("right click row");
    flush_frame(client);
}

/// Secondary-click on the Favorite tile at `index` (the production pointer
/// path).
pub(super) fn right_click_favorite(client: &mut Client, index: usize, x: f32, y: f32) {
    client
        .send_ok(&bbr_testkit::Request::RightClickFavorite { index, x, y })
        .expect("right click favorite");
    flush_frame(client);
}

pub(super) fn menu_select(client: &mut Client, index: usize) {
    client
        .send_ok(&bbr_testkit::Request::TabMenuSelect { index })
        .expect("tab menu select");
}

pub(super) fn menu_activate(client: &mut Client, index: usize) {
    client
        .send_ok(&bbr_testkit::Request::TabMenuActivate { index })
        .expect("tab menu activate");
}

/// An outside pointer press (the §13 one-pointer-action dismissal).
pub(super) fn outside_click(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::TabMenuOutsideClick)
        .expect("tab menu outside click");
}

/// Open the menu on the tab row at `index` and wait for the typed open
/// snapshot bound to the source `tab` with the given presentation source
/// key.
pub(super) fn open_menu(
    client: &mut Client,
    index: usize,
    tab: u64,
    source_key: &str,
    x: f32,
    y: f32,
) -> Snapshot {
    right_click_row(client, index, x, y);
    wait_menu_open(client, tab, source_key, deadline())
}

pub(super) fn wait_menu_open(
    client: &mut Client,
    tab: u64,
    source_key: &str,
    deadline: Instant,
) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("menu open snapshot");
        let menu = &snapshot.tab_menu;
        if menu.open && menu.bound_tab == Some(tab) && menu.source_key == source_key {
            assert_menu_open(&snapshot, tab, source_key);
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "tab menu did not open on tab {tab} ({source_key}); open={} bound={:?} source={:?}",
            menu.open,
            menu.bound_tab,
            menu.source_key
        );
        std::thread::yield_now();
    }
}

/// Poll until the menu is fully dismissed.
pub(super) fn wait_menu_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("menu closed snapshot");
        if !snapshot.tab_menu.open {
            assert_menu_closed(&snapshot);
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "tab menu did not close; selected={:?} open_submenu={:?}",
            snapshot.tab_menu.selected,
            snapshot.tab_menu.open_submenu
        );
        std::thread::yield_now();
    }
}

/// The menu is open, bound to `tab`, registered under `source_key`, with a
/// non-degenerate clamped placement fully inside the window.
pub(super) fn assert_menu_open(snapshot: &Snapshot, tab: u64, source_key: &str) {
    let menu = &snapshot.tab_menu;
    assert!(menu.open, "menu is open");
    assert!(menu.focused, "the open menu owns keyboard focus");
    assert_eq!(menu.bound_tab, Some(tab), "menu bound to the source tab");
    assert_eq!(menu.source_key, source_key, "presentation source key");
    assert!(!menu.rows.is_empty(), "the row model is never empty");
    let [x, y, w, h] = menu.anchor;
    assert!(w > 0.0 && h > 0.0, "the card has positive geometry");
    let [window_w, window_h] = snapshot.window_content_size;
    if window_w > 0.0 {
        assert!(
            x >= 0.0 && x + w <= window_w + 0.5,
            "card clamped to the window width"
        );
        assert!(
            y >= 0.0 && y + h <= window_h + 0.5,
            "card clamped to the window height"
        );
    }
}

/// The menu is closed with all presentation state reset.
pub(super) fn assert_menu_closed(snapshot: &Snapshot) {
    let menu = &snapshot.tab_menu;
    assert!(!menu.open, "menu is closed");
    assert!(!menu.focused);
    assert_eq!(menu.bound_tab, None);
    assert!(menu.rows.is_empty());
    assert_eq!(menu.selected, None);
    assert_eq!(menu.open_submenu, None);
    assert_eq!(menu.submenu_selected, None);
}

/// The complete tab/session identity of `snapshot`: the tab order, the
/// active selection, the live sessions, and the profile/space membership —
/// the mutation evidence for open/dismiss.
pub(super) type TabIdentity = (Vec<u64>, Option<usize>, usize, Vec<(u64, u64)>);

/// The complete tab/session identity of `snapshot` (see [`TabIdentity`]).
pub(super) fn identity(snapshot: &Snapshot) -> TabIdentity {
    let tabs: Vec<u64> = snapshot.all_tabs.iter().map(|tab| tab.id).collect();
    let sessions = snapshot.engine_sessions;
    let membership: Vec<(u64, u64)> = snapshot
        .all_tabs
        .iter()
        .map(|tab| (tab.id, tab.web_view.map(|view| view.0).unwrap_or(0)))
        .collect();
    (tabs, snapshot.active_tab_index, sessions, membership)
}
