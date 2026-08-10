//! Productive new-window seam real-app E2E (NW-SEAM).
//!
//! The script case is fully native: a real fixture executes `window.open`,
//! WebKit invokes the real WKUI delegate, portable policy blocks, and the app
//! records the report without creating a tab/session. The direct-activation
//! case uses typed injection through the exact `BbrApp::on_new_window_request`
//! production method the native delivery pump calls; the hidden harness cannot
//! synthesize a trustworthy `WKNavigationType::LinkActivated` click.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

use bbr_testkit::protocol::{BrowserChange, Request};
use e2e_common::*;

const POPUP_SOURCE_TITLE: &str = "BBR Popup Source";
const POPUP_TARGET_TITLE: &str = "BBR Popup Target";

fn wait_new_window(
    client: &mut Client,
    predicate: impl Fn(&bbr_testkit::protocol::Snapshot) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("new-window snapshot");
        if predicate(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "new-window state did not settle");
    }
}

#[test]
#[ignore]
fn real_script_window_open_is_blocked_and_reported_without_mutation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, _page_one, _page_two) = fixtures();
    let popup = fixtures.url("/popup.html");
    let (mut harness, socket, token) = spawn_harness("nw-script", &popup);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial popup snapshot");
        let source_view = initial.active_web_view.expect("source view");
        if initial.title.as_deref() != Some(POPUP_SOURCE_TITLE) {
            let _ = wait_title(&mut client, source_view, POPUP_SOURCE_TITLE, since);
        }
        let blocked = wait_new_window(&mut client, |snapshot| {
            snapshot.new_window.requests >= 1 && snapshot.new_window.blocked_reported >= 1
        });
        assert_eq!(blocked.tabs.len(), 1, "script popup opens no tab");
        assert_eq!(blocked.engine_sessions, 1, "script popup opens no session");
        assert_eq!(blocked.active_web_view, Some(source_view));
        assert_eq!(blocked.tabs[0].web_view, Some(source_view));
        assert_eq!(blocked.new_window.created_tabs, 0);
        assert_eq!(
            fixtures.hits("/popup-target.html"),
            0,
            "target never loaded"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn user_initiated_delivery_opens_one_regular_tab_in_source_profile_and_space() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("nw-user", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;
        let source_space = initial.active_space;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, source, PAGE_ONE_TITLE, since);
        }

        client
            .send_ok(&Request::NewWindow {
                source,
                source_profile,
                url: target.clone(),
                user_initiated: true,
                intent: bbr_web::new_window::NewWindowIntent::OpenTab,
            })
            .expect("inject productive new-window delivery");
        let (opened_seq, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let opened_view = match change {
            BrowserChange::NewWindowOpened {
                index,
                space,
                profile,
                view,
            } => {
                assert_eq!(index, 1);
                assert_eq!(space, source_space);
                assert_eq!(profile, source_profile);
                view
            }
            _ => unreachable!(),
        };
        let _ = wait_title(&mut client, opened_view, POPUP_TARGET_TITLE, opened_seq);
        let after = client.snapshot().expect("opened child snapshot");
        assert_eq!(after.tabs.len(), 2);
        assert_eq!(after.engine_sessions, 2);
        assert_eq!(after.active_space, source_space);
        assert_eq!(after.active_profile, source_profile);
        assert_eq!(after.tabs[0].web_view, Some(source));
        assert_eq!(after.tabs[1].web_view, Some(opened_view));
        assert_eq!(after.tabs[1].url, target);
        assert_eq!(after.active_web_view, Some(opened_view));
        assert_eq!(after.new_window.requests, 1);
        assert_eq!(after.new_window.created_tabs, 1);
        assert_eq!(after.new_window.blocked_reported, 0);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn stale_source_is_rejected_without_tab_or_session_mutation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("nw-stale", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let before = client.snapshot().expect("initial snapshot");
        let active = before.active_web_view.expect("active source view");
        if before.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, active, PAGE_ONE_TITLE, since);
        }
        let before = client.snapshot().expect("settled source snapshot");
        let response = client
            .request(&Request::NewWindow {
                source: bbr_web::WebViewId(u64::MAX),
                source_profile: before.active_profile,
                url: target,
                user_initiated: true,
                intent: bbr_web::new_window::NewWindowIntent::OpenTab,
            })
            .expect("stale-source response");
        assert!(
            matches!(response, bbr_testkit::protocol::Response::Error(_)),
            "stale source must fail closed: {response:?}"
        );

        let after = client.snapshot().expect("rejected request snapshot");
        assert_eq!(after.tabs, before.tabs);
        assert_eq!(after.all_tabs, before.all_tabs);
        assert_eq!(after.engine_sessions, before.engine_sessions);
        assert_eq!(after.active_space, before.active_space);
        assert_eq!(after.active_profile, before.active_profile);
        assert_eq!(after.active_web_view, before.active_web_view);
        assert_eq!(after.new_window.requests, before.new_window.requests + 1);
        assert_eq!(
            after.new_window.created_tabs,
            before.new_window.created_tabs
        );
        assert_eq!(
            after.new_window.blocked_reported,
            before.new_window.blocked_reported
        );
        assert_eq!(fixtures.hits("/popup-target.html"), 0);
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn inactive_source_space_stays_inactive_and_non_default_profile_is_preserved() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("nw-inactive", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        client
            .send_ok(&dsl::create_profile("Work", "Source"))
            .expect("create source profile");
        let (created_seq, created) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = created_seq;
        let (work_profile, source_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            _ => unreachable!(),
        };
        client
            .send_ok(&dsl::activate_space(source_space))
            .expect("activate source space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: source_space,
            },
            since,
        );
        let source = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        client
            .send_ok(&dsl::create_space(work_profile, "Active"))
            .expect("create active work space");
        let (space_seq, created_space) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = space_seq;
        let active_space = match created_space {
            BrowserChange::SpaceCreated { space, .. } => space,
            _ => unreachable!(),
        };
        client
            .send_ok(&dsl::activate_space(active_space))
            .expect("activate other space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: active_space,
            },
            since,
        );
        let active_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));

        client
            .send_ok(&Request::NewWindow {
                source,
                source_profile: work_profile,
                url: target.clone(),
                user_initiated: true,
                intent: bbr_web::new_window::NewWindowIntent::OpenTab,
            })
            .expect("open in inactive source space");
        let (_, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let child_view = match change {
            BrowserChange::NewWindowOpened {
                index,
                space,
                profile,
                view,
            } => {
                assert_eq!(index, 1);
                assert_eq!(space, source_space);
                assert_eq!(profile, work_profile);
                view
            }
            _ => unreachable!(),
        };
        let after = client.snapshot().expect("inactive source snapshot");
        assert_eq!(after.active_space, active_space, "active space unchanged");
        assert_eq!(after.active_profile, work_profile);
        assert_eq!(after.active_web_view, Some(active_view));
        assert_eq!(after.tabs.len(), 1, "active-space tabs unchanged");
        let child = after
            .all_tabs
            .iter()
            .find(|tab| tab.web_view == Some(child_view))
            .expect("child in global snapshot");
        assert_eq!(child.space, source_space);
        assert_eq!(child.profile, work_profile.0);
        assert_eq!(child.url, target);
        assert_eq!(after.new_window.created_tabs, 1);
    }));
    finish(result, &mut harness, &socket, &token);
}
