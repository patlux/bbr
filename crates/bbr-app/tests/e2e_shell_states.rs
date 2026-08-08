//! Real-app tab-row state E2E (Phase-4 first vertical slice, part 2).
//!
//! The pinned/regular regions, per-row visual states (loading/error), long
//! title and missing-favicon fallbacks, and crash recovery — everything that
//! renders from controller snapshots on the visible tab surface. Split out of
//! the interaction suite so each binary stays focused and bounded.

#![cfg(target_os = "macos")]

mod e2e_common;

use e2e_common::*;

#[test]
#[ignore]
fn shell_renders_regions_fallbacks_and_crash_recovery() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let long_title = fixtures.url("/long-title.html");
    let separator_title = fixtures.url("/separator-title.html");
    let slow = fixtures.url("/__fx/slow");
    let blank = "about:blank";

    let (mut harness, socket, token) = spawn_harness("states", &page_one);

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

        // about:blank has no hostname — the §7.2 monogram falls back to the
        // scheme and the title fallback chain still yields a row label, so
        // the missing-favicon tab renders like any other row.
        let blank_view = open_tab(&mut client, &mut since, blank, None);
        let blank_deadline = deadline();
        loop {
            client
                .send_ok(&bbr_testkit::Request::ProbeWebContent { view: blank_view })
                .expect("probe dark blank page");
            let snapshot = client.snapshot().expect("blank content snapshot");
            if let Some(probe) = snapshot.web_content_probe.as_ref() {
                if probe.view == blank_view
                    && probe.ready_state == "complete"
                    && probe.dom_url == blank
                    && probe.has_body
                    && probe.root_background_color == "rgb(21, 23, 25)"
                    && probe.body_background_color == "rgb(21, 23, 25)"
                {
                    break;
                }
            }
            assert!(
                Instant::now() < blank_deadline,
                "about:blank never committed the dark document background; last probe: {:?}",
                snapshot.web_content_probe
            );
            std::thread::yield_now();
        }
        open_tab(&mut client, &mut since, &long_title, Some(LONG_TITLE));
        open_tab(
            &mut client,
            &mut since,
            &separator_title,
            Some(SEPARATOR_TITLE),
        );
        let slow_view = open_tab(&mut client, &mut since, &slow, Some(SLOW_TITLE));

        let five = client.snapshot().expect("five-tab snapshot");
        assert_eq!(five.tabs.len(), 5);
        assert_eq!(five.engine_sessions, 5);
        assert_eq!(five.tabs[0].url, page_one);
        assert_eq!(five.tabs[1].url, blank);
        assert_eq!(five.tabs[2].url, long_title, "long-title tab holds its URL");
        assert_eq!(five.tabs[2].title, LONG_TITLE, "long title survives");
        assert_eq!(
            five.tabs[3].title, SEPARATOR_TITLE,
            "separator-title tab holds its URL title"
        );
        assert_eq!(five.active_tab_index, Some(4));
        assert_eq!(five.active_web_view, Some(slow_view));
        // §4 truncation is applied to the rendered row titles from the same
        // row_visuals source the shell renders: the late separator falls back
        // to the full title (GPUI text_ellipsis cuts visually), the early
        // separator cuts so the site name survives.
        assert_eq!(five.row_titles[2], LONG_TITLE);
        assert_eq!(five.row_titles[3], SEPARATOR_ROW_TITLE);

        // Pinned region: pinning the second row moves it to the pinned head
        // without touching sessions (testkit seam; pin commands are a later
        // slice). The shell renders the pinned region separately (§11.2).
        client.send_ok(&dsl::pin_tab(1)).expect("pin second tab");
        let _ = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        let pinned = client.snapshot().expect("pinned snapshot");
        assert_eq!(pinned.tabs.len(), 5);
        assert!(pinned.tabs[0].pinned, "pinned region leads the space");
        assert!(
            pinned.tabs[1..].iter().all(|tab| !tab.pinned),
            "regular region follows the pinned head"
        );
        assert_eq!(pinned.tabs[0].url, blank, "pinned row is the blank tab");
        assert_eq!(pinned.engine_sessions, 5, "pin never recreates sessions");
        assert_eq!(pinned.active_tab_index, Some(4));
        assert_eq!(
            pinned.tab_states[0],
            TabVisualState::Default,
            "pinned rows carry the same state vocabulary"
        );

        // Loading state: navigate the slow fixture and observe the §10.5
        // row state while the deterministic response is still pending.
        client
            .send_ok(&dsl::navigate(&slow))
            .expect("navigate slow page");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, true) if *view == slow_view),
            since,
        );
        let loading = client.snapshot().expect("loading snapshot");
        assert_eq!(loading.active_tab_index, Some(4));
        assert!(loading.tabs[4].loading, "session loading flag observable");
        assert_eq!(loading.tab_states[4], TabVisualState::Loading);
        since = wait_title(&mut client, slow_view, SLOW_TITLE, since);
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(view, false) if *view == slow_view),
            since,
        );
        let loaded = client.snapshot().expect("loaded snapshot");
        assert!(!loaded.tabs[4].loading);
        assert_eq!(loaded.tab_states[4], TabVisualState::Selected);

        // Crash state: the engine event rides the real event transport; the
        // shell must show the §10.7 error state for the row and the crash
        // panel surface, with the session destroyed.
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(slow_view)))
            .expect("inject crash");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::PageCrashed(view) if *view == slow_view),
            since,
        );
        let crashed = client.snapshot().expect("crashed snapshot");
        assert!(
            crashed.tabs[4].crashed,
            "crash flag survives session teardown"
        );
        assert_eq!(crashed.tabs[4].web_view, None, "session destroyed");
        assert_eq!(crashed.active_web_view, None, "no native surface presented");
        assert_eq!(crashed.engine_sessions, 4);
        assert_eq!(crashed.tab_states[4], TabVisualState::Error);
        // §11.8 crash panel + §15 ids: the crashed row stays listed with its
        // error state while the panel replaces the glass area.
        assert_ids(
            &crashed,
            &[
                "bbr-crash-panel",
                "bbr-crash-reload",
                "bbr-crash-close",
                "bbr-tab-row-4",
                "bbr-tab-state-4",
            ],
        );

        // Recovery: reload reactivates the tab with a fresh session and the
        // preserved URL; the crash state clears. The mutation is synchronous,
        // so the immediate snapshot observes the fresh incarnation.
        client.send_ok(&dsl::reload()).expect("reload crashed tab");
        let recovered = client.snapshot().expect("recovered snapshot");
        assert_eq!(recovered.tabs.len(), 5);
        assert_eq!(recovered.engine_sessions, 5);
        assert_eq!(recovered.active_tab_index, Some(4));
        assert!(
            !recovered.tabs[4].crashed,
            "fresh session clears crash state"
        );
        assert_eq!(
            recovered.tabs[4].url, slow,
            "preserved URL drives recreation"
        );
        let fresh_view = recovered.active_web_view.expect("fresh view");
        assert_ne!(fresh_view, slow_view, "fresh incarnation");
        since = wait_title(&mut client, fresh_view, SLOW_TITLE, since);

        // Back to the first tab: warm identity must still hold everywhere.
        client
            .send_ok(&dsl::click_tab(1))
            .expect("click page-one row");
        let _ = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 1 }, since);
        let back = client.snapshot().expect("back snapshot");
        assert_eq!(back.active_web_view, Some(first_view));
        assert_eq!(back.tabs[1].web_view, Some(first_view));
        assert_eq!(back.engine_sessions, 5);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
