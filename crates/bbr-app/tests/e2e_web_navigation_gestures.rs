//! Web-content navigation gestures real-app E2E (N1 packet
//! `web-content-navigation-gestures`).
//!
//! Drives the real `bbr` app and real WKWebViews through the exact
//! productive content-gesture transport: the typed `ContentGestureCommit`
//! request pushes into the exact channel the native content-gesture bridge
//! reporter uses, and the pump drains it into
//! `BbrApp::on_content_gesture_commit` — the same production method a real
//! committed horizontal trackpad swipe reaches. The hidden harness cannot
//! synthesize a trustworthy NSEvent scroll gesture, so committed logical
//! commits are injected typed (the accepted seam pattern; native phase/
//! direction/cancel mapping stays unit-proven in `bbr-web-macos`).
//!
//! Every scenario uses real WebKit history state (`canGoBack()`/
//! `canGoForward()` events) and deterministic fixtures. No sleeps; every
//! step synchronizes on typed events or snapshot state under deadlines.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_web::{ContentGestureDirection, WebEvent, WebViewId};
use e2e_common::*;

const CHILD_TARGET_TITLE: &str = "BBR Popup Target";

/// Send one committed logical content gesture through the exact production
/// transport (native bridge channel → pump → `on_content_gesture_commit`).
fn gesture(client: &mut Client, view: WebViewId, direction: ContentGestureDirection) {
    client
        .send_ok(&Request::ContentGestureCommit { view, direction })
        .expect("inject committed content gesture");
}

fn wait_history(
    client: &mut Client,
    view: WebViewId,
    can_back: bool,
    can_forward: bool,
    since: u64,
) -> u64 {
    let mut cursor = since;
    if can_back {
        cursor = wait_web_event(
            client,
            |e| matches!(e, WebEvent::CanGoBackChanged(id, true) if *id == view),
            cursor,
        );
    }
    if can_forward {
        cursor = wait_web_event(
            client,
            |e| matches!(e, WebEvent::CanGoForwardChanged(id, true) if *id == view),
            cursor,
        );
    }
    cursor
}

/// Wait until the initial fixture page finished loading (its title is
/// known). A later real navigation must supersede a *committed* first
/// document, otherwise WebKit cancels the still-provisional initial load
/// and the history has only one entry — the exact-pane Back/Forward
/// scenarios need a genuine two-entry history.
fn wait_fixture_loaded(client: &mut Client, view: WebViewId, title: &str, since: u64) -> u64 {
    let snapshot = client.snapshot().expect("fixture title snapshot");
    if snapshot.title.as_deref() == Some(title) {
        return since;
    }
    wait_title(client, view, title, since)
}

/// The persistent `TabId` of the tab currently backed by `view` (identity
/// evidence: the return names stable ids, never indices or web views).
fn tab_id(snapshot: &bbr_testkit::protocol::Snapshot, view: WebViewId) -> u64 {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.web_view == Some(view))
        .unwrap_or_else(|| panic!("no tab owns {view:?}"))
        .id
}

/// Assert the opener-return outcome named the exact child/opener pair and
/// return the presented opener view.
fn assert_return(pair: BrowserChange, child: u64, opener: u64, recreated: bool) -> WebViewId {
    match pair {
        BrowserChange::OpenerReturn {
            child: c,
            opener: o,
            presented_view,
            recreated: r,
            ..
        } => {
            assert_eq!(c, child, "return names the exact closed child");
            assert_eq!(o, opener, "return names the exact opener");
            assert_eq!(r, recreated, "recreation flag matches lifecycle");
            presented_view
        }
        other => panic!("expected OpenerReturn, got {other:?}"),
    }
}

/// A child opened from the active tab through the productive new-window
/// path: returns the child's `WebViewId` and advances the event cursor past
/// the child's target title.
fn open_child(
    client: &mut Client,
    since: &mut u64,
    opener_view: WebViewId,
    profile: bbr_testkit::protocol::TestProfileId,
    target_url: &str,
    target_title: &str,
) -> WebViewId {
    client
        .send_ok(&Request::NewWindow {
            source: opener_view,
            source_profile: profile,
            url: target_url.into(),
            intent: bbr_web::new_window::NewWindowIntent::OpenTab,
            user_initiated: true,
        })
        .expect("inject productive new-window delivery");
    let (seq, change) = wait_browser_change_matching(
        client,
        |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
        *since,
    );
    *since = seq;
    let child_view = match change {
        BrowserChange::NewWindowOpened { view, .. } => view,
        _ => unreachable!(),
    };
    let snapshot = client.snapshot().expect("child snapshot");
    if snapshot.title.as_deref() != Some(target_title) {
        *since = wait_title(client, child_view, target_title, *since);
    }
    child_view
}

#[test]
#[ignore]
fn back_and_forward_navigate_the_exact_pane_and_never_touch_other_tabs() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("gest-back-fwd", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let pane = settled.active_web_view.expect("first native view");
        since = wait_fixture_loaded(&mut client, pane, PAGE_ONE_TITLE, since);
        since = wait_history(&mut client, pane, false, false, since);

        // Real navigation arms the real history: page1 → page2.
        client
            .send_ok(&Request::Navigate {
                url: page_two.clone(),
            })
            .expect("navigate to page two");
        since = wait_title(&mut client, pane, PAGE_TWO_TITLE, since);
        since = wait_history(&mut client, pane, true, false, since);

        // A second unrelated tab (page one, no history) stays active and
        // untouched while the gesture targets the warm first pane by id.
        let other = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        assert_ne!(other, pane, "second tab owns its own view");
        let before = client.snapshot().expect("two-tab snapshot");
        assert_eq!(before.tabs.len(), 2);
        assert_eq!(before.engine_sessions, 2);

        // Gesture Back targets the exact warm pane, not the active tab. The
        // typed change is emitted before WebKit's async title lands, so the
        // change is waited for first.
        gesture(&mut client, pane, ContentGestureDirection::Back);
        let (seq, routed) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::GestureHistoryRouted { .. }),
            since,
        );
        since = seq;
        assert_eq!(
            routed,
            BrowserChange::GestureHistoryRouted { view: pane },
            "one history dispatch names the exact bound pane"
        );
        since = wait_title(&mut client, pane, PAGE_ONE_TITLE, since);
        since = wait_history(&mut client, pane, false, true, since);

        // Gesture Forward returns the pane to page two.
        gesture(&mut client, pane, ContentGestureDirection::Forward);
        let (seq, routed) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::GestureHistoryRouted { view } if *view == pane),
            since,
        );
        since = seq;
        assert_eq!(
            routed,
            BrowserChange::GestureHistoryRouted { view: pane },
            "forward dispatch names the same pane"
        );
        since = wait_title(&mut client, pane, PAGE_TWO_TITLE, since);
        wait_history(&mut client, pane, true, false, since);

        let after = client.snapshot().expect("after forward snapshot");
        assert_eq!(after.tabs.len(), 2, "no tab mutated");
        assert_eq!(after.engine_sessions, 2, "no session created or destroyed");
        assert_eq!(after.active_web_view, Some(other), "active tab untouched");
        assert_eq!(
            after.tabs[0].web_view,
            Some(pane),
            "target pane keeps its identity"
        );
        assert_eq!(after.content_gesture.history_routed, 2);
        assert_eq!(after.content_gesture.opener_returns, 0);
        assert_eq!(after.content_gesture.no_ops, 0);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn back_at_settled_child_root_closes_only_child_and_presents_exact_opener() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-child-root", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        let opener_space = settled.active_space;
        since = wait_history(&mut client, opener_view, false, false, since);

        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &target,
            CHILD_TARGET_TITLE,
        );
        let child = client.snapshot().expect("child loaded snapshot");
        assert_eq!(child.tabs.len(), 2);
        assert_eq!(child.engine_sessions, 2);
        assert_eq!(child.active_web_view, Some(child_view));
        assert_eq!(child.content_gesture.opener_relations.len(), 1);
        assert!(!child.can_go_back, "child sits at its root");
        let child_id = tab_id(&child, child_view);
        let opener_id = tab_id(&child, opener_view);

        // The one committed Back runs the atomic close-child/activate-opener
        // transaction: exactly one causal typed outcome.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The opener became visible again: its suspended load resumes and the
        // presentation settles on its fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        assert_eq!(presented, opener_view, "warm opener session is reused");

        let after = client.snapshot().expect("opener return snapshot");
        assert_eq!(after.tabs.len(), 1, "exactly the child closed");
        assert_eq!(
            after.engine_sessions, 1,
            "child session destroyed, none created"
        );
        assert_eq!(after.active_web_view, Some(opener_view));
        assert_eq!(after.active_space, opener_space);
        assert_eq!(after.active_profile, opener_profile);
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert!(
            after.content_gesture.opener_relations.is_empty(),
            "the closed child's relation is invalidated"
        );
        assert_eq!(after.content_gesture.opener_returns, 1);
        assert_eq!(after.content_gesture.history_routed, 0);
        assert_eq!(after.content_gesture.no_ops, 0);
        // The opener presents and receives native web focus.
        assert!(!after.chrome_focused, "web content owns focus after return");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn child_with_older_history_navigates_first_then_back_at_root_returns() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-child-hist", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_history(&mut client, opener_view, false, false, since);

        // Child opens at page two (its root), then navigates within its own
        // history to the popup target.
        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &page_two,
            PAGE_TWO_TITLE,
        );
        client
            .send_ok(&Request::Navigate {
                url: target.clone(),
            })
            .expect("child navigates to target");
        since = wait_title(&mut client, child_view, CHILD_TARGET_TITLE, since);
        since = wait_history(&mut client, child_view, true, false, since);

        // First Back uses the child's older history: exactly one history
        // dispatch, the child stays open. The typed change is emitted before
        // WebKit's async title lands, so the change is waited for first.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, routed) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::GestureHistoryRouted { view } if *view == child_view),
            since,
        );
        since = seq;
        assert_eq!(
            routed,
            BrowserChange::GestureHistoryRouted { view: child_view }
        );
        since = wait_title(&mut client, child_view, PAGE_TWO_TITLE, since);
        let mid = client.snapshot().expect("after first back snapshot");
        assert_eq!(mid.tabs.len(), 2, "child stayed open");
        assert_eq!(mid.engine_sessions, 2);
        assert_eq!(mid.content_gesture.history_routed, 1);
        assert_eq!(mid.content_gesture.opener_returns, 0);
        let child_id = tab_id(&mid, child_view);
        let opener_id = tab_id(&mid, opener_view);

        // The child is now at its root again; a second committed Back
        // returns to the exact opener.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The opener became visible again: its suspended load resumes and the
        // presentation settles on its fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("return snapshot");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.engine_sessions, 1);
        assert_eq!(after.active_web_view, Some(opener_view));
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_eq!(after.content_gesture.history_routed, 1);
        assert_eq!(after.content_gesture.opener_returns, 1);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn no_op_matrix_preserves_tabs_sessions_and_focus() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-noops", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_fixture_loaded(&mut client, opener_view, PAGE_ONE_TITLE, since);
        since = wait_history(&mut client, opener_view, false, false, since);

        // 1. Disabled Forward at a root with no forward history: no-op.
        gesture(&mut client, opener_view, ContentGestureDirection::Forward);
        // 2. Root Back without provenance: no-op, never closes the tab.
        gesture(&mut client, opener_view, ContentGestureDirection::Back);
        // 3. A stale view id: no-op.
        gesture(
            &mut client,
            WebViewId(u64::MAX),
            ContentGestureDirection::Back,
        );
        let baseline = client.snapshot().expect("no-op baseline snapshot");
        assert_eq!(baseline.tabs.len(), 1);
        assert_eq!(baseline.engine_sessions, 1);
        assert_eq!(baseline.active_web_view, Some(opener_view));
        assert_eq!(baseline.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_eq!(baseline.content_gesture.no_ops, 3);
        assert_eq!(baseline.content_gesture.history_routed, 0);
        assert_eq!(baseline.content_gesture.opener_returns, 0);

        // 4. Stale opener: a child whose opener was closed is a no-op and
        //    must never close an arbitrary tab.
        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &target,
            CHILD_TARGET_TITLE,
        );
        let child = client.snapshot().expect("child loaded snapshot");
        let child_id = tab_id(&child, child_view);
        client
            .send_ok(&Request::CloseTab { index: 0 })
            .expect("close the opener tab");
        let (close_seq, _) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::TabClosed { .. }),
            since,
        );
        let _ = close_seq;
        // The child is now active at its root but its opener is gone.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let after = client.snapshot().expect("stale opener snapshot");
        assert_eq!(after.tabs.len(), 1, "no tab was closed by the no-op");
        assert_eq!(after.active_web_view, Some(child_view));
        assert_eq!(after.title.as_deref(), Some(CHILD_TARGET_TITLE));
        assert_eq!(after.content_gesture.no_ops, 4);
        assert!(after.content_gesture.opener_relations.is_empty());
        assert_eq!(after.content_gesture.opener_returns, 0);
        assert_eq!(
            tab_id(&after, child_view),
            child_id,
            "the child identity survives the stale-opener no-op"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn provisional_child_load_is_a_no_op_then_returns_once_settled() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let slow = fixtures.url("/__fx/slow");
    let (mut harness, socket, token) = spawn_harness("gest-provisional", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_history(&mut client, opener_view, false, false, since);

        // The child's initial document is the deterministic slow route: the
        // load stays provisional (uncommitted) for the fixture's fixed
        // server-side delay, so a committed Back during that window is a
        // whole no-op — never an ambiguous provisional-load return.
        client
            .send_ok(&Request::NewWindow {
                source: opener_view,
                source_profile: opener_profile,
                url: slow.clone(),
                intent: bbr_web::new_window::NewWindowIntent::OpenTab,
                user_initiated: true,
            })
            .expect("open child on the slow route");
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        since = seq;
        let child_view = match change {
            BrowserChange::NewWindowOpened { view, .. } => view,
            _ => unreachable!(),
        };
        // Synchronize on the observable in-flight load: the production
        // `loading` flag flips true before the fixture commits.
        let provisional = loop {
            let snapshot = client.snapshot().expect("provisional snapshot");
            if snapshot.loading && snapshot.active_web_view == Some(child_view) {
                break snapshot;
            }
            assert!(
                Instant::now() < deadline(),
                "child provisional load did not start"
            );
            std::thread::yield_now();
        };
        assert!(
            !provisional.can_go_back,
            "provisional child has no back history"
        );
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let still = client.snapshot().expect("provisional no-op snapshot");
        assert_eq!(still.tabs.len(), 2, "child was not closed during load");
        assert_eq!(still.active_web_view, Some(child_view));
        assert_eq!(still.content_gesture.no_ops, 1);
        assert_eq!(still.content_gesture.opener_returns, 0);

        // Once the slow document commits the child is a settled root: a
        // later committed Back returns to the exact opener.
        since = wait_title(&mut client, child_view, SLOW_TITLE, since);
        let child = client.snapshot().expect("settled child snapshot");
        let child_id = tab_id(&child, child_view);
        let opener_id = tab_id(&child, opener_view);
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The opener became visible again: its suspended load resumes and the
        // presentation settles on its fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("settled return snapshot");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.engine_sessions, 1);
        assert_eq!(after.active_web_view, Some(opener_view));
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_eq!(after.content_gesture.opener_returns, 1);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn return_from_split_child_dissolves_the_split_and_presents_opener() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-split", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_history(&mut client, opener_view, false, false, since);
        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &target,
            CHILD_TARGET_TITLE,
        );

        // Two-pane split: the child (active, index 1) is the focused pane,
        // the opener (index 0) the secondary pane.
        client
            .send_ok(&Request::SetSplit {
                primary: 1,
                secondary: 0,
            })
            .expect("open split over child + opener");
        let split = client.snapshot().expect("split snapshot");
        assert_eq!(
            split.pane_layout.as_ref().map(|p| (p.primary, p.secondary)),
            Some((child_view, Some(opener_view))),
            "child is the focused split pane under the pointer"
        );
        assert_eq!(split.engine_sessions, 2);
        let child_id = tab_id(&split, child_view);
        let opener_id = tab_id(&split, opener_view);

        // The committed Back over the child pane runs the atomic return:
        // the child closes, the split dissolves, the opener presents in
        // single-pane mode.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The opener became visible again: its suspended load resumes and the
        // presentation settles on its fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("split dissolved snapshot");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.engine_sessions, 1);
        assert_eq!(
            after.pane_layout.as_ref().map(|p| p.secondary),
            Some(None),
            "the split dissolved with the child"
        );
        assert_eq!(after.active_web_view, Some(opener_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn return_activates_opener_moved_across_space_and_profile() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-moved-opener", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let home_space = settled.active_space;
        let home_profile = settled.active_profile;
        let opener_view = settled.active_web_view.expect("opener view");
        since = wait_history(&mut client, opener_view, false, false, since);

        // A second profile with its own space: the destination of the move.
        client
            .send_ok(&dsl::create_profile("Work", "Source"))
            .expect("create work profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (work_profile, work_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            _ => unreachable!(),
        };
        // A tab lives in the work space so the moved opener's destination
        // space is non-empty (the opener keeps its identity at the end).
        client
            .send_ok(&dsl::activate_space(work_space))
            .expect("activate work space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: work_space },
            since,
        );
        let _work_tab = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::activate_space(home_space))
            .expect("activate home space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: home_space },
            since,
        );

        // Child in the home space; the opener then moves across the profile
        // boundary (its session is destroyed and rebuilt in the destination
        // store at move time — reported by the TabMoved recreation flag).
        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            home_profile,
            &target,
            CHILD_TARGET_TITLE,
        );
        let before_move = client.snapshot().expect("pre-move snapshot");
        assert_eq!(before_move.tabs.len(), 2, "opener + child in home space");
        let child_id = tab_id(&before_move, child_view);
        let opener_id = tab_id(&before_move, opener_view);
        client
            .send_ok(&Request::MoveTab {
                index: 0,
                to_space: work_space,
            })
            .expect("move the opener to the work profile");
        let (seq, moved) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::TabMoved { .. }),
            since,
        );
        since = seq;
        assert_eq!(
            moved,
            BrowserChange::TabMoved {
                to_space: work_space,
                recreated: true,
            },
            "cross-profile move recreates the opener session at move time"
        );

        // Back at the child root activates the exact opener at its current
        // Space/Profile, isolated from the home profile. The opener's warm
        // destination session is reused — no recreation at return time.
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The recreated destination session loads the opener's URL; its
        // presentation settles on the fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("moved opener snapshot");
        assert_eq!(
            after.active_space, work_space,
            "opener's current space wins"
        );
        assert_eq!(after.active_profile, work_profile);
        assert_eq!(after.active_web_view, Some(presented));
        let opener_row = after
            .all_tabs
            .iter()
            .find(|tab| tab.id == opener_id)
            .expect("opener in the global snapshot");
        assert_eq!(opener_row.space, work_space);
        assert_eq!(opener_row.profile, work_profile.0);
        assert_eq!(after.content_gesture.opener_returns, 1);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn cmd_w_closes_the_active_child_through_normal_close_never_opener_selection() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-cmd-w", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_history(&mut client, opener_view, false, false, since);
        let _child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &target,
            CHILD_TARGET_TITLE,
        );

        // Cmd-W is the ordinary close seam: it closes the ACTIVE child and
        // selects a neighbor — it never runs the gesture opener-return
        // transaction (the provenance-based selection stays exclusive to
        // the committed Back fallback).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-w".into(),
            })
            .expect("dispatch cmd-w");
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::TabClosed { .. }),
            since,
        );
        let _ = seq;
        assert!(
            !matches!(change, BrowserChange::OpenerReturn { .. }),
            "Cmd-W is a normal close, never an opener return"
        );
        // The opener becomes the active tab again; its suspended load resumes
        // and the presentation settles on its fixture title.
        let _ = wait_fixture_loaded(&mut client, opener_view, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("cmd-w snapshot");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.engine_sessions, 1);
        assert_eq!(after.active_web_view, Some(opener_view));
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_eq!(
            after.content_gesture.opener_returns, 0,
            "no gesture transaction"
        );
        assert_eq!(
            after.content_gesture.commits, 0,
            "no gesture commits at all"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn settled_failure_state_at_child_root_still_returns_to_opener() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("gest-failure", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let opener_view = settled.active_web_view.expect("opener view");
        let opener_profile = settled.active_profile;
        since = wait_history(&mut client, opener_view, false, false, since);
        let child_view = open_child(
            &mut client,
            &mut since,
            opener_view,
            opener_profile,
            &target,
            CHILD_TARGET_TITLE,
        );

        // A settled navigation failure (the child's live session still owns
        // its view) makes the root a settled failure state: the committed
        // Back returns to the opener instead of hanging on a broken page.
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::NavigationFailed(child_view, "fixture failure".into()),
            })
            .expect("inject settled navigation failure");
        since = wait_web_event(
            &mut client,
            |e| matches!(e, WebEvent::NavigationFailed(view, _) if *view == child_view),
            since,
        );
        let failed = client.snapshot().expect("failed child snapshot");
        assert_eq!(failed.tabs.len(), 2);
        let child_id = tab_id(&failed, child_view);
        let opener_id = tab_id(&failed, opener_view);
        gesture(&mut client, child_view, ContentGestureDirection::Back);
        let (seq, change) = wait_browser_change_matching(
            &mut client,
            |c| matches!(c, BrowserChange::OpenerReturn { .. }),
            since,
        );
        let presented = assert_return(change, child_id, opener_id, false);
        let _ = seq;
        // The opener became visible again: its suspended load resumes and the
        // presentation settles on its fixture title before the state asserts.
        let _ = wait_fixture_loaded(&mut client, presented, PAGE_ONE_TITLE, since);
        let after = client.snapshot().expect("failure return snapshot");
        assert_eq!(after.tabs.len(), 1);
        assert_eq!(after.engine_sessions, 1);
        assert_eq!(after.active_web_view, Some(opener_view));
        assert_eq!(after.title.as_deref(), Some(PAGE_ONE_TITLE));
        assert_eq!(after.content_gesture.opener_returns, 1);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
