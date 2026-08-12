//! Real-app favicon acquisition E2E (task packet `real-favicon-pipeline.md`).
//!
//! Covers the production acquisition path over a real WKWebView against the
//! deterministic loopback fixtures: a host with a declared
//! `<link rel="icon">` must reach the rendered tab row AND its Favorite
//! tile as `favicon == image` after load, and a host that declares no icon
//! (and serves no `/favicon.ico`) must stay on the deterministic §7.2
//! monogram through the bounded acquisition window. Every step synchronizes
//! on typed events or snapshot state under a deadline — no sleeps.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::TestProfileId;
use bbr_web::FaviconContent;
use e2e_common::*;

/// The declared-icon fixture's `<title>`.
const FAVICON_TITLE: &str = "BBR Favicon Fixture";
/// The no-icon fixture's `<title>`.
const NO_FAVICON_TITLE: &str = "BBR No-Favicon Fixture";

/// Poll the snapshot until the active-space tab at `index` reports the
/// expected `favicon` presentation (`"image"` / `"monogram"`), under the
/// shared deadline. The mirror refresh happens on the frame boundary, so a
/// store mutation lands in the next snapshot — never asserted immediately.
fn wait_favicon(
    client: &mut Client,
    index: usize,
    expected: &str,
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    loop {
        let snapshot = client.snapshot().expect("favicon snapshot");
        if snapshot
            .tabs
            .get(index)
            .is_some_and(|tab| tab.favicon == expected)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "tab {index} favicon did not reach {expected:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A host with a declared favicon: the real image reaches the tab row and
/// the Favorite tile (one acquisition, every presentation — the same record
/// both derive from).
#[test]
#[ignore]
fn declared_favicon_reaches_tab_row_and_favorite_tile() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page = fixtures.url("/favicon.html");

    let (mut harness, socket, token) = spawn_harness("favicons-declared", &page);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(FAVICON_TITLE) {
            since = wait_title(&mut client, first_view, FAVICON_TITLE, since);
        }

        // The declared icon downloads through the owning profile's isolated
        // session; the adapter emits exactly one `FaviconChanged` with
        // resolved image bytes.
        client
            .wait_for_event_since(
                |event| {
                    matches!(
                        event,
                        Event::Web(WebEvent::FaviconChanged(_, FaviconContent::Image(_)))
                    )
                },
                since,
                deadline(),
            )
            .expect("favicon image event");

        // The typed snapshot reports the image on the rendered tab row, and
        // the fixture served the declared candidate exactly once.
        let after = wait_favicon(&mut client, 0, "image", deadline());
        assert_eq!(after.tabs[0].favicon, "image");
        assert_eq!(
            after.tabs[0].url, page,
            "the favicon tab is the declared-icon host"
        );
        assert_eq!(
            fixtures.hits("/favicon.png"),
            1,
            "the declared candidate is fetched exactly once"
        );
        // The §15 rendered slot is present (the row renders the tile).
        assert!(after.visible_ids.iter().any(|id| id == "bbr-tab-favicon-0"));

        // The Favorite tile derives from the same store record: toggling a
        // favorite for the tab renders the tile with the image, never a
        // second acquisition.
        client
            .send_ok(&bbr_testkit::Request::ToggleFavoriteTab { index: 0 })
            .expect("favorite the declared-icon tab");
        let with_favorite = client.snapshot().expect("favorite snapshot");
        assert_eq!(with_favorite.favorites.len(), 1);
        assert_eq!(
            with_favorite.favorites[0].favicon, "image",
            "the Favorite tile presents the same record the tab row derives from"
        );
        assert_eq!(
            fixtures.hits("/favicon.png"),
            1,
            "one acquisition serves every presentation of the host"
        );

        // §10 state precedence over an acquired image: the stored favicon
        // identity stays `image`, while the row's typed visual state moves
        // to Loading and Error — the production tile then renders the state
        // glyph branch proven by the component decision test.
        client
            .send_ok(&dsl::inject_web_event(WebEvent::LoadingChanged(
                first_view, true,
            )))
            .expect("inject loading over image");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, true) if *view == first_view),
            since,
        );
        let loading = client.snapshot().expect("loading favicon snapshot");
        assert_eq!(loading.tabs[0].favicon, "image");
        assert_eq!(loading.tab_states[0], TabVisualState::Loading);

        client
            .send_ok(&dsl::inject_web_event(WebEvent::LoadingChanged(
                first_view, false,
            )))
            .expect("clear loading over image");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, false) if *view == first_view),
            since,
        );
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(first_view)))
            .expect("inject crash over image");
        wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::PageCrashed(view) if *view == first_view),
            since,
        );
        let crashed = client.snapshot().expect("crashed favicon snapshot");
        assert_eq!(crashed.tabs[0].favicon, "image");
        assert_eq!(crashed.tab_states[0], TabVisualState::Error);

        client.quit().expect("quit");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The same host in two profiles performs one acquisition through each
/// profile's isolated WebKit data store and keeps two separate cache records.
/// A process-global host cache would serve the second Space without another
/// request, so the fixture's exact hit count is the real-app isolation proof.
#[test]
#[ignore]
fn same_host_in_two_profiles_acquires_once_per_profile() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page = fixtures.url("/favicon.html");

    let (mut harness, socket, token) = spawn_harness("favicons-profiles", &page);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(FAVICON_TITLE) {
            since = wait_title(&mut client, first_view, FAVICON_TITLE, since);
        }
        let first_profile = wait_favicon(&mut client, 0, "image", deadline());
        assert_eq!(first_profile.tabs[0].favicon, "image");
        assert_eq!(fixtures.hits("/favicon.png"), 1);

        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (created_seq, created) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = created_seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        assert_ne!(private_profile, TestProfileId(1));

        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &page, Some(FAVICON_TITLE));
        let private_snapshot = wait_favicon(&mut client, 0, "image", deadline());
        assert_eq!(private_snapshot.active_profile, private_profile);
        assert_eq!(private_snapshot.active_web_view, Some(private_view));
        assert_eq!(private_snapshot.tabs[0].favicon, "image");
        assert_eq!(
            fixtures.hits("/favicon.png"),
            2,
            "each profile's isolated store performs its own acquisition"
        );

        client.quit().expect("quit");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A host that declares no icon (and serves no `/favicon.ico`) stays on the
/// deterministic §7.2 monogram through the bounded acquisition window: the
/// fallback fails silently and no image record ever lands.
#[test]
#[ignore]
fn host_without_declared_icon_stays_monogram() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page = fixtures.url("/no-favicon.html");

    let (mut harness, socket, token) = spawn_harness("favicons-none", &page);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(NO_FAVICON_TITLE) {
            wait_title(&mut client, first_view, NO_FAVICON_TITLE, since);
        }

        // The no-icon host stays on the monogram across the bounded
        // acquisition window: the candidate collection yields nothing, the
        // `/favicon.ico` fallback fails silently (the fixture serves none),
        // and no image record lands. Deadline-bounded snapshot polling —
        // every observed snapshot must keep the monogram.
        let window = Instant::now() + Duration::from_secs(4);
        while Instant::now() < window {
            let snapshot = client.snapshot().expect("monogram snapshot");
            assert_eq!(
                snapshot.tabs[0].favicon, "monogram",
                "a host without a declared icon never presents an image"
            );
            assert!(snapshot.tabs[0].url.ends_with("/no-favicon.html"));
            std::thread::sleep(Duration::from_millis(200));
        }
        // The fixture never served any icon for the no-icon host.
        assert_eq!(fixtures.hits("/favicon.png"), 0);

        client.quit().expect("quit");
    }));

    finish(result, &mut harness, &socket, &token);
}
