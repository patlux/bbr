//! Real-app external-URL open E2E (queue item 13, native slice).
//!
//! Proves the macOS delivery path end to end over the real app process and
//! real WKWebViews: a delivered link opens a fresh tab in the frontmost
//! eligible window through the merged portable contract and the existing
//! controller — without disturbing identity (existing view, profile, space,
//! tab order all unchanged); burst duplicates open exactly one tab; rejected
//! schemes open nothing; and a link delivered while the app is still
//! launching (cold start) lands once the app finishes launching.
//!
//! Delivery goes through the exact production queue: the testkit
//! `DeliverExternalUrl` request pushes into the same channel the AppKit
//! delegate path (`application:openURLs:` via GPUI's `on_open_urls`) uses,
//! and the pump drains that channel into `BbrApp::on_external_url`.

#![cfg(target_os = "macos")]

mod e2e_common;

use std::time::{Duration, Instant};

use bbr_testkit::protocol::{dsl, BrowserChange};
use bbr_testkit::Client;
use bbr_web::WebViewId;
use e2e_common::*;

/// Deliver a URL through the testkit seam into the production external-URL
/// queue (the same channel the AppKit delegate path pushes into).
fn deliver(client: &mut Client, url: &str) {
    client
        .send_ok(&bbr_testkit::Request::DeliverExternalUrl {
            url: url.to_string(),
        })
        .expect("deliver external url");
}

/// A delivered link opens a fresh tab in the frontmost eligible window,
/// routed through the merged contract: the existing tab keeps its view
/// identity, profile and space are untouched, and the tab order is
/// preserved — the new tab joins the end of the active space.
#[test]
#[ignore]
fn delivered_link_opens_in_frontmost_window_without_disturbing_identity() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("exturl-identity", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        assert_eq!(initial.tabs.len(), 1);
        assert_eq!(initial.engine_sessions, 1);
        let first_view = initial.active_web_view.expect("first view");
        let initial_profile = initial.active_profile;
        let initial_space = initial.active_space;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Deliver a link through the production queue: exactly one fresh tab
        // opens in the same window, same profile, same space.
        deliver(&mut client, &page_two);
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        let delivered_view = wait_for_external_tab(&mut client, &page_two, &page_one);

        let after = client.snapshot().expect("after external delivery");
        assert_eq!(after.tabs.len(), 2, "a fresh tab was opened");
        assert_eq!(after.engine_sessions, 2, "the fresh tab owns a new session");
        // Identity preservation: the existing tab keeps its exact view id,
        // position (index 0), URL, and title; nothing was reordered.
        assert_eq!(after.tabs[0].web_view, Some(first_view));
        assert_eq!(after.tabs[0].url, page_one);
        assert_eq!(after.tabs[1].web_view, Some(delivered_view));
        assert_eq!(after.tabs[1].url, page_two);
        assert_eq!(after.active_web_view, Some(delivered_view));
        assert_eq!(after.active_profile, initial_profile, "profile untouched");
        assert_eq!(after.active_space, initial_space, "space untouched");

        finish(Ok(()), &mut harness, &socket, &token);
    }));
    finish(result, &mut harness, &socket, &token);
}

/// While the user is inside another profile's space, a delivered link joins
/// exactly that window's current profile and space — never the default
/// profile and never a different space. The other profile's tabs keep their
/// sessions untouched.
#[test]
#[ignore]
fn delivered_link_respects_the_frontmost_windows_profile_and_space() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("exturl-profile", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Create a second profile with its own space and make it active —
        // the frontmost (only) window now presents that profile/space.
        client
            .send_ok(&dsl::create_profile("Work", "Focus"))
            .expect("create profile");
        let (created_seq, change) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = created_seq;
        let (work_profile, work_space) = match change {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            _ => unreachable!("profile created"),
        };
        client
            .send_ok(&dsl::activate_space(work_space))
            .expect("activate work space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: work_space },
            since,
        );
        let work = client.snapshot().expect("work space snapshot");
        assert_eq!(work.active_profile, work_profile);
        assert_eq!(work.active_space, work_space);
        assert!(work.tabs.is_empty(), "fresh space has no tabs");

        // A delivered link now opens inside the window's current profile and
        // space — the fresh tab belongs to the work space.
        deliver(&mut client, &page_two);
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 0 }, since);
        let after = client.snapshot().expect("after delivery into work space");
        assert_eq!(
            after.active_profile, work_profile,
            "stays in the work profile"
        );
        assert_eq!(after.active_space, work_space, "stays in the work space");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.tabs[0].url, page_two);
        let delivered_view = after.active_web_view.expect("delivered view");

        // Identity preservation across profiles: the default profile's tab
        // keeps its exact view id and session; nothing moved or recreated.
        let original = after
            .all_tabs
            .iter()
            .find(|tab| tab.web_view == Some(first_view))
            .expect("default-profile tab untouched");
        assert_eq!(
            original.profile, initial.active_profile.0,
            "default profile untouched"
        );
        assert_eq!(original.url, page_one);
        assert_eq!(after.engine_sessions, 2, "both sessions stay live");
        assert_ne!(delivered_view, first_view);

        finish(Ok(()), &mut harness, &socket, &token);
    }));
    finish(result, &mut harness, &socket, &token);
}

/// The contract's burst dedup: two immediate deliveries of the same URL open
/// exactly one tab, and a distinct URL still opens on its own.
#[test]
#[ignore]
fn burst_duplicate_deliveries_open_exactly_one_tab() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("exturl-burst", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // The OS can deliver the same URL several times in a burst (double
        // click on a link in another app): the contract routes it once.
        deliver(&mut client, &page_two);
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        deliver(&mut client, &page_two);
        // No second TabOpened may arrive; give the queue a bounded window to
        // prove the duplicate was suppressed rather than merely slow.
        std::thread::sleep(Duration::from_millis(250));
        let deduped = client.snapshot().expect("after duplicate delivery");
        assert_eq!(deduped.tabs.len(), 2, "the burst opened exactly one tab");
        assert_eq!(deduped.engine_sessions, 2, "no second session was created");

        // A distinct URL is a fresh key: it routes on its own.
        deliver(&mut client, &page_one);
        let _ = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 2 }, since);
        let after = client.snapshot().expect("after distinct delivery");
        assert_eq!(after.tabs.len(), 3);
        assert_eq!(after.engine_sessions, 3);
        assert_eq!(after.tabs[1].url, page_two);
        assert_eq!(after.tabs[2].url, page_one);

        finish(Ok(()), &mut harness, &socket, &token);
    }));
    finish(result, &mut harness, &socket, &token);
}

/// The contract rejects dangerous and malformed schemes: nothing opens, no
/// session is created, and the existing identity is untouched.
#[test]
#[ignore]
fn rejected_schemes_open_nothing() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("exturl-reject", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        deliver(&mut client, "javascript:alert(1)");
        deliver(&mut client, "file:///etc/passwd");
        deliver(&mut client, "mailto:someone@example.com");
        deliver(&mut client, "no scheme here");
        std::thread::sleep(Duration::from_millis(250));
        let after = client.snapshot().expect("after rejected deliveries");
        assert_eq!(after.tabs.len(), 1, "no tab opened");
        assert_eq!(after.engine_sessions, 1, "no session was created");
        assert_eq!(after.active_web_view, Some(first_view));
        assert_eq!(after.tabs[0].url, page_one);
        let _ = since;

        finish(Ok(()), &mut harness, &socket, &token);
    }));
    finish(result, &mut harness, &socket, &token);
}

/// Cold start: a URL delivered while the app is still launching lands once
/// the app finishes launching — the link is buffered in the production
/// external-URL queue and drained by the pump, never dropped.
///
/// The harness connects the instant the gateway socket appears (the socket
/// is created mid-launch, before the event pump starts and before `AppReady`
/// is broadcast), so the delivery rides the exact production channel the
/// AppKit delegate path (`application:openURLs:` via GPUI's `on_open_urls`)
/// pushes into. The E2E harness spawns the raw debug binary, which cannot
/// receive real Apple Events on this host (LaunchServices resolves the app
/// name to other lanes' registered BBR.app bundles), so the OS-side
/// launch-time queueing itself is not exercised here — the buffering the app
/// owns (channel created before the run loop, callback registered before
/// `run`, pump draining buffered URLs once the entity exists) is.
#[test]
#[ignore]
fn cold_start_link_lands_after_launch() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("exturl-cold", &page_one);

    // Connect at the earliest possible moment: `connect` retries until the
    // gateway socket exists, which happens mid-launch — before the event
    // pump is spawned and before `AppReady`. The URL therefore arrives
    // while the app is still launching and must be buffered until the pump
    // starts draining.
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        deliver(&mut client, &page_two);
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // The cold-start link may land before or after the attach; either
        // way it opens exactly one extra tab with page_two loaded.
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            let snapshot = client.snapshot().expect("snapshot while waiting");
            if snapshot.tabs.iter().any(|tab| tab.url == page_two) {
                break;
            }
            assert!(
                Instant::now() < until,
                "the cold-start link never opened a tab"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        let after = client.snapshot().expect("after cold-start delivery");
        assert_eq!(after.tabs.len(), 2, "initial tab + delivered tab");
        let cold_view = after
            .tabs
            .iter()
            .find(|tab| tab.url == page_two)
            .expect("delivered tab present")
            .web_view
            .expect("delivered tab has a live session");
        if after.title.as_deref() != Some(PAGE_TWO_TITLE) {
            let _ = wait_title(&mut client, cold_view, PAGE_TWO_TITLE, since);
        }
        // The initial tab kept its identity; only the delivered tab is new.
        assert_eq!(after.tabs[0].url, page_one);

        finish(Ok(()), &mut harness, &socket, &token);
    }));
    finish(result, &mut harness, &socket, &token);
}

/// Wait until the external-delivery tab (`expected_url`) is present with
/// page_two's title loaded and the original tab still at index 0; return the
/// delivered tab's fresh view id.
fn wait_for_external_tab(client: &mut Client, expected_url: &str, original_url: &str) -> WebViewId {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        let snapshot = client.snapshot().expect("snapshot while waiting");
        let Some(tab) = snapshot.tabs.iter().find(|tab| tab.url == expected_url) else {
            assert!(Instant::now() < until, "external tab never appeared");
            std::thread::sleep(Duration::from_millis(50));
            continue;
        };
        if snapshot.tabs.len() < 2 || snapshot.tabs[0].url != original_url {
            assert!(Instant::now() < until, "tab order never settled");
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        let view = tab.web_view.expect("external tab has a live session");
        if snapshot.title.as_deref() != Some(PAGE_TWO_TITLE) {
            assert!(Instant::now() < until, "delivered page never loaded");
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        return view;
    }
}
