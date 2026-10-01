//! Kept places across a restart (schema v5, task packet
//! `closed-pins-and-favorites`): a pinned tab's pinned page, a pinned tab
//! closed in place with the page it closed on (its ⇧⌘T record), the
//! Favorites grid and a folder with its member all survive a graceful
//! restart through the production keyboard and organization paths. After
//! the restart the closed row stays closed without a session, ⌘W on the
//! navigated pinned tab resets it to its persisted pinned page, and ⇧⌘T
//! reopens first that row at the page it closed on, then the row closed
//! before the restart at its page.

use bbr_persistence::{
    BrowserState, GlyphName, Profile, ProfileHue, SidebarState, Space, SpaceGlyph, SpaceTint, Tab,
    WindowGeometry, WindowState,
};
use bbr_testkit::protocol::Snapshot;

use crate::e2e_common::real_input::wait_for;
use crate::*;

/// [100 pinned page one | 101 page one (active), 102 page two].
fn seed(page_one: &str, page_two: &str) -> BrowserState {
    let tab = |id: u64, title: &str, url: &str, pinned: bool| Tab {
        id,
        title: title.into(),
        url: url.into(),
        pinned,
        pinned_page: None,
        closed: false,
    };
    BrowserState {
        profiles: vec![Profile {
            id: 1,
            title: "Work".into(),
            hue: ProfileHue::Cobalt,
            spaces: vec![Space {
                id: 10,
                title: "Main".into(),
                tint: SpaceTint::Cobalt,
                paper: Default::default(),
                glyph: SpaceGlyph::Glyph(GlyphName::Code),
                tabs: vec![
                    tab(100, "Home", page_one, true),
                    tab(101, "One", page_one, false),
                    tab(102, "Two", page_two, false),
                ],
                active_tab: Some(101),
                recently_closed: Vec::new(),
                split: None,
                folders: Vec::new(),
            }],
        }],
        windows: vec![WindowState {
            id: 50,
            geometry: Some(WindowGeometry {
                x: -200.0,
                y: 10.0,
                width: 1080.0,
                height: 700.0,
            }),
            sidebar: SidebarState {
                width: 300.0,
                collapsed: false,
                last_expanded_width: 300.0,
            },
        }],
        favorites: Vec::new(),
    }
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&dsl::keystroke(keystroke))
        .unwrap_or_else(|error| panic!("{keystroke}: {error:?}"));
}

fn tab(snapshot: &Snapshot, index: usize) -> &bbr_testkit::protocol::TabSnapshot {
    &snapshot.tabs[index]
}

/// The live view of the tab at `index` once drawn.
fn view_at(client: &mut Client, index: usize) -> WebViewId {
    wait_for(client, "a live view", |s| s.tabs[index].web_view.is_some()).tabs[index]
        .web_view
        .expect("live view")
}

#[test]
#[ignore]
fn closed_pins_pinned_pages_favorites_and_folders_survive_a_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let profile = harness_root("persist-kept", pid).join("profile");
    seed_browser_state(&profile, &seed(&page_one, &page_two));

    // Leg 1: pin, navigate, close in place, favorite and group through the
    // production paths; flush and quit gracefully.
    let (mut harness, socket, token) = spawn_harness("persist-kept", &page_one);
    let folder_id = std::cell::Cell::new(0);
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
        let first = wait_for_window_settled(&mut client, deadline());
        let one = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, one, PAGE_ONE_TITLE, since);
        }

        // 101 is pinned at page one, then navigates on to page two.
        key(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 1 }, since);
        client
            .send_ok(&dsl::navigate(page_two.clone()))
            .expect("navigate the pinned tab");
        since = wait_title(&mut client, one, PAGE_TWO_TITLE, since);

        // 102 is pinned at page two, navigates to page one and closes in
        // place: its row resets to page two, its record keeps page one.
        client.send_ok(&dsl::select_tab(2)).expect("select 102");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 2 }, since);
        let two = view_at(&mut client, 2);
        since = wait_title(&mut client, two, PAGE_TWO_TITLE, since);
        key(&mut client, "cmd-shift-p");
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 2 }, since);
        client
            .send_ok(&dsl::navigate(page_one.clone()))
            .expect("navigate the second pinned tab");
        since = wait_title(&mut client, two, PAGE_ONE_TITLE, since);
        key(&mut client, "cmd-w");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 2 }, since);
        let closed = wait_for(&mut client, "the closed row", |s| {
            tab(s, 2).lifecycle == "closed"
        });
        assert_eq!(tab(&closed, 2).url, page_two, "reset to its pinned page");
        assert_eq!(closed.active_tab_index, Some(1));

        // 100 joins the Favorites; 101 joins a folder.
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite 100");
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        let folder = wait_for(&mut client, "the folder", |s| s.folders.len() == 1).folders[0].id;
        folder_id.set(folder);
        client
            .send_ok(&dsl::move_tab_into_folder(1, folder))
            .expect("group 101");
        let organized = wait_for(&mut client, "the organization", |s| {
            s.favorites.len() == 1 && s.folders[0].members == vec![101]
        });
        assert_eq!(organized.favorites[0].tab, 100);

        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
        let _ = since;
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    // Leg 2: everything restores; the pinned page and the record work.
    let (mut harness2, socket2, token2) = spawn_harness("persist-kept", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        let restored = wait_for_window_settled(&mut client, deadline());
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "restored from the store: {:?}",
            restored.restore
        );
        assert_eq!(restored.pinned_count, 3);
        assert_eq!(restored.active_tab_index, Some(1));
        assert_eq!(tab(&restored, 1).url, page_two, "101 restores where it was");
        assert_eq!(tab(&restored, 2).lifecycle, "closed", "102 stays closed");
        assert_eq!(tab(&restored, 2).url, page_two);
        assert_eq!(tab(&restored, 2).web_view, None);
        assert_eq!(restored.engine_sessions, 1, "a closed row hydrates nothing");
        assert_eq!(restored.lifecycle_counts.closed, 1);
        let organized = wait_for(&mut client, "the restored organization", |s| {
            s.favorites.len() == 1 && s.folders.len() == 1
        });
        assert_eq!(organized.favorites[0].tab, 100);
        assert_eq!(organized.folders[0].id, folder_id.get(), "same folder id");
        assert_eq!(organized.folders[0].members, vec![101]);
        let one = restored.active_web_view.expect("restored active view");
        if restored.title.as_deref() != Some(PAGE_TWO_TITLE) {
            since = wait_title(&mut client, one, PAGE_TWO_TITLE, since);
        }

        // ⌘W resets 101 to its persisted pinned page (page one).
        key(&mut client, "cmd-w");
        since = wait_browser_change(&mut client, BrowserChange::TabClosed { index: 1 }, since);
        let closed = wait_for(&mut client, "101 closed", |s| {
            tab(s, 1).lifecycle == "closed"
        });
        assert_eq!(tab(&closed, 1).url, page_one, "the persisted pinned page");

        // ⇧⌘T reopens 101 at the page it closed on, in the same row.
        key(&mut client, "cmd-shift-t");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 1 }, since);
        let reopened = wait_for(&mut client, "101 reopened", |s| {
            s.active_tab_index == Some(1) && tab(s, 1).web_view.is_some()
        });
        assert_eq!(tab(&reopened, 1).url, page_two);
        assert_eq!(reopened.tabs.len(), 3, "no row was added");
        let view = tab(&reopened, 1).web_view.expect("live");
        since = wait_title(&mut client, view, PAGE_TWO_TITLE, since);

        // ⇧⌘T again reopens 102 at the page it closed on before the
        // restart (its persisted in-place record).
        key(&mut client, "cmd-shift-t");
        since = wait_browser_change(&mut client, BrowserChange::TabReopened { index: 2 }, since);
        let reopened = wait_for(&mut client, "102 reopened", |s| {
            s.active_tab_index == Some(2) && tab(s, 2).web_view.is_some()
        });
        assert_eq!(tab(&reopened, 2).url, page_one);
        assert_eq!(reopened.lifecycle_counts.closed, 0);
        let view = tab(&reopened, 2).web_view.expect("live");
        wait_title(&mut client, view, PAGE_ONE_TITLE, since);
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}

/// Keep the profile directory for the next launch (a successful finish lets
/// the harness Drop remove the root) and rethrow a failed leg.
fn keep_root(
    result: std::thread::Result<()>,
    harness: &mut Harness,
    socket: &std::path::Path,
    token: &str,
) {
    match result {
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}
