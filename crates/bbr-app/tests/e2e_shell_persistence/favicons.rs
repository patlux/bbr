//! Favicons across a restart (task packet `favicon-disk-cache`): an icon
//! acquired before a graceful restart still shows on the Favorite tile of a
//! tab that owns no session after the relaunch, without fetching it again.

use bbr_persistence::{
    BrowserState, GlyphName, Profile, ProfileHue, SidebarState, Space, SpaceGlyph, SpaceTint, Tab,
    WindowGeometry, WindowState,
};

use crate::e2e_common::real_input::wait_for;
use crate::*;

/// [100 the declared-icon page (active), 101 page one].
fn seed(favicon_page: &str, page_one: &str) -> BrowserState {
    let tab = |id: u64, title: &str, url: &str| Tab {
        id,
        title: title.into(),
        url: url.into(),
        pinned: false,
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
                tabs: vec![tab(100, "Icon", favicon_page), tab(101, "One", page_one)],
                active_tab: Some(100),
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

#[test]
#[ignore]
fn an_acquired_favicon_shows_after_a_restart_without_a_session() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let favicon_page = fixtures.url("/favicon.html");
    let pid = std::process::id();
    let profile = harness_root("persist-favicons", pid).join("profile");
    seed_browser_state(&profile, &seed(&favicon_page, &page_one));

    // Leg 1: the icon is acquired, the tab becomes a Favorite, and page one
    // is selected so the Favorite owns no session after the relaunch.
    let (mut harness, socket, token) = spawn_harness("persist-favicons", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        wait_for_window_settled(&mut client, deadline());
        wait_for(&mut client, "the acquired icon", |s| {
            s.tabs[0].favicon == "image"
        });
        assert_eq!(fixtures.hits("/favicon.png"), 1);
        client
            .send_ok(&dsl::toggle_favorite_tab(0))
            .expect("favorite the icon tab");
        client
            .send_ok(&dsl::select_tab(1))
            .expect("select page one");
        wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let favorite = wait_for(&mut client, "the Favorite tile", |s| s.favorites.len() == 1);
        assert_eq!(favorite.favorites[0].favicon, "image");
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);
    assert!(
        profile.join("favicons").join("1").join("index").exists(),
        "the profile's favicons are kept in the profile directory"
    );

    // Leg 2: the Favorite has no session, yet its tile shows the icon kept
    // on disk; nothing fetched it again.
    let (mut harness2, socket2, token2) = spawn_harness("persist-favicons", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let restored = wait_for_window_settled(&mut client, deadline());
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "restored from the store: {:?}",
            restored.restore
        );
        let tile = wait_for(&mut client, "the restored Favorite", |s| {
            s.favorites.len() == 1
        });
        let favorite_tab = tile
            .all_tabs
            .iter()
            .find(|tab| tab.id == tile.favorites[0].tab)
            .expect("the Favorite's tab");
        assert_eq!(favorite_tab.url, favicon_page);
        assert_eq!(
            tile.tabs[favorite_tab.index].web_view, None,
            "the Favorite owns no session after the relaunch"
        );
        assert_eq!(
            tile.favorites[0].favicon, "image",
            "the kept icon shows without a session"
        );
        assert_eq!(fixtures.hits("/favicon.png"), 1, "no second fetch");
        client.quit().expect("quit");
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}

/// Keep the profile directory for the next launch and rethrow a failed leg.
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
