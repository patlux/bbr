//! Background link tabs real-app E2E (DD-LINK, task packet
//! `background-link-tabs`).
//!
//! The hidden real-app harness cannot synthesize a trustworthy native
//! `WKNavigationType::LinkActivated` click (the settled NW-SEAM evidence
//! split), so the user-initiated background path enters through the exact
//! `BbrApp::on_new_window_request` production method the native coordinator
//! delivery pump calls, with the explicit `NewWindowInput::Background`
//! disposition. The native modifier/button extraction (`classify_input`) is
//! unit-proven in `bbr-web-macos/tests/new_window.rs`; the script-created
//! `window.open` block path stays fully native (covered by
//! `e2e_shell_new_window.rs`). Every child target is a real deterministic
//! fixture loaded in a real WKWebView in the source profile's store.
//!
//! Coverage map (background-link-tabs.md, "Real-app E2E"):
//!
//! - The background disposition opens exactly one unselected regular child
//!   immediately after the opener; the opener stays selected, visible, and
//!   web-focused while the child loads in the background (real fixture title
//!   arrives for the hidden session; the opener's `WebViewId` and the app's
//!   active view never change; chrome focus is never stolen).
//! - Repeated X/Y/Z opens remain ordered `[B, X, Y, Z]` with an unrelated C
//!   after the child group.
//! - Child activation later reuses its already-created session; a committed
//!   Back gesture at the child's settled root returns to the exact opener.
//! - Stale source and invalid source profile reject without any mutation.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

use std::time::Instant;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_web::new_window::NewWindowInput;
use bbr_web::{ContentGestureDirection, WebViewId};
use e2e_common::*;

const OPENER_TITLE: &str = "BBR Background Links";
const X_TITLE: &str = "BBR Background X";

fn wait_snapshot(
    client: &mut Client,
    predicate: impl Fn(&bbr_testkit::protocol::Snapshot) -> bool,
    what: &str,
) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("background-link snapshot");
        if predicate(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} did not settle");
    }
}

/// Resolve the persistent `TabId` (u64) of the tab hosting `view`, from the
/// typed `all_tabs` global snapshot (provenance is identity-based, never
/// `WebViewId`-keyed by contract).
fn tab_id(snapshot: &bbr_testkit::protocol::Snapshot, view: WebViewId) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.web_view == Some(view))
        .unwrap_or_else(|| panic!("no tab owns {view:?}"))
        .id
}

fn inject_background(
    client: &mut Client,
    source: WebViewId,
    profile: bbr_web::ProfileId,
    url: &str,
) {
    client
        .send_ok(&Request::NewWindow {
            source,
            source_profile: profile,
            url: url.to_string(),
            user_initiated: true,
            intent: bbr_web::new_window::NewWindowIntent::Default,
            input: NewWindowInput::Background,
        })
        .expect("inject background-input new-window delivery");
}

#[test]
#[ignore]
fn background_disposition_opens_one_unselected_child_after_the_opener() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, _page_one, _page_two) = fixtures();
    let opener_url = fixtures.url("/links.html");
    let target = fixtures.url("/bg-x.html");
    let (mut harness, socket, token) = spawn_harness("bg-open", &opener_url);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        let opener_space = settled.active_space;
        if settled.title.as_deref() != Some(OPENER_TITLE) {
            since = wait_title(&mut client, opener, OPENER_TITLE, since);
        }
        let before = client.snapshot().expect("settled opener snapshot");
        assert!(!before.chrome_focused, "web focus before the open");

        inject_background(&mut client, opener, opener_profile, &target);
        let (opened_seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let child = match change {
            BrowserChange::NewWindowOpened {
                index,
                space,
                profile,
                view,
            } => {
                assert_eq!(index, 1, "first child inserts immediately after the opener");
                assert_eq!(space, opener_space);
                assert_eq!(profile, opener_profile);
                view
            }
            _ => unreachable!(),
        };
        // The child really loads in the background: the real fixture title
        // arrives for the hidden session.
        let _ = wait_title(&mut client, child, X_TITLE, opened_seq);
        let after = wait_snapshot(
            &mut client,
            |s| s.tabs.len() == 2 && s.active_web_view == Some(opener),
            "child settled with opener selected",
        );
        assert_eq!(after.tabs[0].web_view, Some(opener));
        assert_eq!(after.tabs[1].web_view, Some(child));
        assert_eq!(after.tabs[1].url, target);
        assert_eq!(after.active_web_view, Some(opener), "opener stays selected");
        assert!(
            !after.tabs[1].active,
            "the child is never selected/foregrounded"
        );
        assert_eq!(after.engine_sessions, 2);
        assert_eq!(after.active_space, opener_space);
        assert_eq!(after.active_profile, opener_profile);
        assert!(!after.chrome_focused, "the opener keeps web focus");
        assert_eq!(after.new_window.requests, 1);
        assert_eq!(after.new_window.created_tabs, 1);
        assert_eq!(after.new_window.created_background_tabs, 1);
        assert_eq!(after.new_window.blocked_reported, 0);
        assert_eq!(
            after.content_gesture.opener_relations,
            vec![(tab_id(&after, child), tab_id(&after, opener))],
            "exact identity-based opener provenance"
        );
        assert_eq!(
            fixtures.hits("/bg-x.html"),
            1,
            "the background child requested its real fixture"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn repeated_children_group_in_click_order_before_an_unrelated_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, _page_one, _page_two) = fixtures();
    let opener_url = fixtures.url("/links.html");
    let (x, y, z) = (
        fixtures.url("/bg-x.html"),
        fixtures.url("/bg-y.html"),
        fixtures.url("/bg-z.html"),
    );
    let (mut harness, socket, token) = spawn_harness("bg-group", &opener_url);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        if settled.title.as_deref() != Some(OPENER_TITLE) {
            since = wait_title(&mut client, opener, OPENER_TITLE, since);
        }

        // An unrelated tab C already exists after the opener: open it with a
        // plain command, then reselect the opener for the background gesture.
        client
            .send_ok(&Request::OpenTab {
                url: fixtures.url("/bg-z.html"),
            })
            .expect("open unrelated C");
        since = wait_browser_change(&mut client, BrowserChange::TabOpened { index: 1 }, since);
        client
            .send_ok(&Request::SelectTab { index: 0 })
            .expect("reselect the opener");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let mid = client.snapshot().expect("mid grouping snapshot");
        assert_eq!(mid.tabs.len(), 2, "opener + unrelated C");
        assert_eq!(mid.active_web_view, Some(opener));

        // X, then Y, then Z as background children of the opener.
        inject_background(&mut client, opener, opener_profile, &x);
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        since = seq;
        inject_background(&mut client, opener, opener_profile, &y);
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        since = seq;
        inject_background(&mut client, opener, opener_profile, &z);
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let _ = seq;
        let after = wait_snapshot(
            &mut client,
            |s| s.tabs.len() == 5 && s.active_web_view == Some(opener),
            "grouped children settled",
        );
        let x_view = after
            .tabs
            .iter()
            .find(|t| t.url == x)
            .and_then(|t| t.web_view)
            .expect("x view");
        let y_view = after
            .tabs
            .iter()
            .find(|t| t.url == y)
            .and_then(|t| t.web_view)
            .expect("y view");
        let z_view = after
            .tabs
            .iter()
            .find(|t| t.url == z)
            .and_then(|t| t.web_view)
            .expect("z view");
        let urls: Vec<&str> = after.tabs.iter().map(|t| t.url.as_str()).collect();
        assert_eq!(
            urls,
            vec![
                opener_url.as_str(),
                x.as_str(),
                y.as_str(),
                z.as_str(),
                fixtures.url("/bg-z.html").as_str()
            ],
            "grouped children stay ordered [B, X, Y, Z] with unrelated C after the group"
        );
        assert_eq!(after.tabs[1].web_view, Some(x_view));
        assert_eq!(after.tabs[2].web_view, Some(y_view));
        assert_eq!(after.tabs[3].web_view, Some(z_view));
        assert_eq!(after.active_web_view, Some(opener), "opener stays selected");
        assert_eq!(after.new_window.created_background_tabs, 3);
        for (i, tab) in after.tabs.iter().enumerate() {
            if (1..=3).contains(&i) {
                assert!(!tab.active, "child {i} never selected");
                assert!(tab.web_view.is_some(), "child {i} has a live session");
            }
        }
        assert!(!after.chrome_focused, "opener keeps web focus throughout");
        assert_eq!(after.engine_sessions, 5);
        assert_eq!(after.new_window.requests, 3);
        assert_eq!(after.content_gesture.opener_relations.len(), 3);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn child_activation_reuses_its_session_and_back_at_root_returns_to_the_opener() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, _page_one, _page_two) = fixtures();
    let opener_url = fixtures.url("/links.html");
    let target = fixtures.url("/bg-x.html");
    let (mut harness, socket, token) = spawn_harness("bg-return", &opener_url);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        if settled.title.as_deref() != Some(OPENER_TITLE) {
            since = wait_title(&mut client, opener, OPENER_TITLE, since);
        }

        inject_background(&mut client, opener, opener_profile, &target);
        let (opened_seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        since = opened_seq;
        let child = match change {
            BrowserChange::NewWindowOpened { view, .. } => view,
            _ => unreachable!(),
        };
        since = wait_title(&mut client, child, X_TITLE, since);

        // Activate the child later: the SAME already-created session is
        // reused (same WebViewId — no recreation) and it becomes the active,
        // focused pane.
        let child_index = client
            .snapshot()
            .expect("grouped snapshot")
            .tabs
            .iter()
            .position(|t| t.web_view == Some(child))
            .expect("child index");
        client
            .send_ok(&Request::SelectTab { index: child_index })
            .expect("activate the child");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::TabSelected { index: child_index },
            since,
        );
        let activated = client.snapshot().expect("activated child snapshot");
        assert_eq!(
            activated.active_web_view,
            Some(child),
            "same session reused"
        );
        assert_eq!(activated.engine_sessions, 2, "no session recreation");
        assert!(
            activated
                .content_gesture
                .opener_relations
                .contains(&(tab_id(&activated, child), tab_id(&activated, opener))),
            "provenance survives activation"
        );

        // The child's single-fixture document is a settled root with no
        // back history: a committed Back must close it and return to the
        // exact warm opener (Back-at-root contract).
        client
            .send_ok(&Request::ContentGestureCommit {
                view: child,
                direction: ContentGestureDirection::Back,
            })
            .expect("commit Back at the settled child root");
        let after = wait_snapshot(
            &mut client,
            |s| s.tabs.len() == 1 && s.active_web_view == Some(opener),
            "Back-at-root returned to the opener",
        );
        assert_eq!(after.active_web_view, Some(opener));
        assert_eq!(after.engine_sessions, 1, "child session destroyed");
        assert_eq!(after.active_space, settled.active_space);
        assert_eq!(after.active_profile, settled.active_profile);
        assert!(
            after.content_gesture.opener_returns >= 1,
            "Back-at-root committed an opener return"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn stale_source_and_invalid_profile_reject_without_mutation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, _page_one, _page_two) = fixtures();
    let opener_url = fixtures.url("/links.html");
    let target = fixtures.url("/bg-x.html");
    let (mut harness, socket, token) = spawn_harness("bg-stale", &opener_url);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        if settled.title.as_deref() != Some(OPENER_TITLE) {
            let _ = wait_title(&mut client, opener, OPENER_TITLE, since);
        }
        let before = client.snapshot().expect("before rejections");

        // Stale source view: fails closed with an error, zero mutation.
        let response = client
            .request(&Request::NewWindow {
                source: WebViewId(u64::MAX),
                source_profile: opener_profile,
                url: target.clone(),
                user_initiated: true,
                intent: bbr_web::new_window::NewWindowIntent::Default,
                input: NewWindowInput::Background,
            })
            .expect("stale-source response");
        assert!(
            matches!(response, bbr_testkit::protocol::Response::Error(_)),
            "stale source must fail closed: {response:?}"
        );
        // Invalid source profile: fails closed with an error, zero mutation.
        let response = client
            .request(&Request::NewWindow {
                source: opener,
                source_profile: bbr_web::ProfileId(4242),
                url: target.clone(),
                user_initiated: true,
                intent: bbr_web::new_window::NewWindowIntent::Default,
                input: NewWindowInput::Background,
            })
            .expect("profile-mismatch response");
        assert!(
            matches!(response, bbr_testkit::protocol::Response::Error(_)),
            "profile mismatch must fail closed: {response:?}"
        );

        let after = client.snapshot().expect("after rejections");
        assert_eq!(after.tabs, before.tabs, "tab order untouched");
        assert_eq!(after.all_tabs, before.all_tabs, "global tabs untouched");
        assert_eq!(after.engine_sessions, before.engine_sessions);
        assert_eq!(after.active_web_view, before.active_web_view);
        assert_eq!(after.active_space, before.active_space);
        assert_eq!(after.active_profile, before.active_profile);
        assert_eq!(after.new_window.requests, before.new_window.requests + 2);
        assert_eq!(
            after.new_window.created_tabs,
            before.new_window.created_tabs
        );
        assert_eq!(
            after.new_window.created_background_tabs,
            before.new_window.created_background_tabs
        );
        assert_eq!(
            fixtures.hits("/bg-x.html"),
            0,
            "no background load on rejection"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
