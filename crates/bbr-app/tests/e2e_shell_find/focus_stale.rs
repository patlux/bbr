//! Focus restoration + staleness scenarios (task packet
//! `find-in-page-portal-bar`).

use crate::e2e_common::*;
use crate::support::*;

/// Escape (and the production close path) restores the captured origin focus
/// domain deterministically: web-focused at open returns focus to the web
/// view; chrome-focused at open (an open Cmd-L location surface) returns to
/// the command bar with the origin mode intact. Highlights are cleared with
/// the presentation; identity is untouched.
#[test]
#[ignore]
fn find_escape_restores_origin_focus_domain_and_clears() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, repeated, _zero, _mutate, _isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness("find-escape", &repeated);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(!settled.chrome_focused, "web-focused origin");
        let view = settled.active_web_view.expect("first native view");
        since = wait_fixture_loaded(&mut client, view, FIND_REPEATED_TITLE, since);

        // Web origin: open, search, Escape closes and returns to the web.
        client.send_ok(&dsl::find_open()).expect("find open");
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        wait_find_report(&mut client, 3, Some(1), deadline());
        key(&mut client, "escape");
        let closed = wait_for_find_closed(&mut client, deadline());
        assert!(!closed.chrome_focused, "focus returned to the web view");
        assert_ownership_preserved(&closed, &settled);

        // Chrome origin: Cmd-L opens the location surface (chrome focus),
        // Cmd-F replaces it, Escape returns to the command bar with the
        // origin mode intact.
        key(&mut client, "cmd-l");
        let _ = wait_chrome_focus(&mut client, true, since);
        let chrome = client.snapshot().expect("chrome snapshot");
        assert_eq!(
            chrome.command_bar_mode,
            bbr_testkit::protocol::CommandBarMode::Location,
            "Cmd-L opened the location surface"
        );
        key(&mut client, "cmd-f");
        let find = wait_find_open(&mut client, deadline());
        assert!(find.find.focused, "the find field owns focus");
        assert!(
            !find.chrome_focused,
            "the find field replaced the chrome focus while open"
        );
        key(&mut client, "escape");
        let restored = wait_for_find_closed(&mut client, deadline());
        assert!(
            restored.chrome_focused,
            "focus returned to the command bar (chrome origin)"
        );
        assert_eq!(
            restored.command_bar_mode,
            bbr_testkit::protocol::CommandBarMode::Location,
            "the origin command mode survived the find cycle"
        );
        assert_ownership_preserved(&restored, &settled);

        // The explicit close path (FindClose = the close-button seam) clears
        // identically from a fresh open. The origin is now chrome (the
        // previous cycle restored the command bar), so close returns there.
        client.send_ok(&dsl::find_open()).expect("find open again");
        client
            .send_ok(&dsl::find_set_query("bob"))
            .expect("set query");
        wait_find_report(&mut client, 1, Some(1), deadline());
        client.send_ok(&dsl::find_close()).expect("find close");
        let closed = wait_for_find_closed(&mut client, deadline());
        assert!(closed.chrome_focused, "close restores the chrome origin");
        assert_ownership_preserved(&closed, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// A navigation of the bound view invalidates the reported results in place
/// (stale) without closing the bar; a fresh query restarts deterministically
/// on the new document, including the zero-match report on the zero page. A
/// page content mutation is observed by the strategy and reported stale on
/// the next request, with the fresh count from the mutated DOM.
#[test]
#[ignore]
fn find_stale_navigation_and_page_mutation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, repeated, zero, mutate, _isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness("find-stale", &repeated);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        since = wait_fixture_loaded(&mut client, view, FIND_REPEATED_TITLE, since);

        // Part A — navigation of the bound view invalidates in place.
        client.send_ok(&dsl::find_open()).expect("find open");
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        wait_find_report(&mut client, 3, Some(1), deadline());
        client
            .send_ok(&dsl::navigate(&zero))
            .expect("navigate bound view");
        let stale = wait_for_find_stale(&mut client, deadline());
        assert!(stale.find.open, "navigation invalidates, never closes");
        assert_eq!(
            stale.portal.presentation,
            Some(FIND_PRESENTATION_ID),
            "the bar stays presented after navigation"
        );
        assert_eq!(
            stale.find.match_count,
            Some(3),
            "the stale report keeps the old count"
        );
        assert_eq!(
            stale.active_web_view,
            Some(view),
            "navigation keeps the session"
        );

        // Wait for the new document to finish loading before the fresh
        // query: a query issued mid-navigation runs against the old or
        // transitional document and reports a misleading count.
        if stale.title.as_deref() != Some(FIND_ZERO_TITLE) {
            since = wait_title(&mut client, view, FIND_ZERO_TITLE, since);
        }

        // A fresh query on the new document restarts the search; the zero
        // page reports count 0 for an absent query.
        client
            .send_ok(&dsl::find_set_query("quiet"))
            .expect("fresh query");
        let fresh = wait_find_report(&mut client, 1, Some(1), deadline());
        assert!(!fresh.find.stale, "a fresh query clears staleness");
        client
            .send_ok(&dsl::find_set_query("xyzzy"))
            .expect("zero query on zero page");
        wait_find_report(&mut client, 0, None, deadline());
        client.send_ok(&dsl::find_close()).expect("close find");
        wait_for_find_closed(&mut client, deadline());
        assert_ownership_preserved(&stale, &settled);

        // Part B — page mutation is observed by the strategy and reported.
        client
            .send_ok(&bbr_testkit::Request::OpenTab { url: mutate })
            .expect("open mutate tab");
        let opened = client.snapshot().expect("opened mutate tab");
        let mutate_view = opened.active_web_view.expect("mutate view");
        if opened.title.as_deref() != Some(FIND_MUTATE_TITLE) {
            since = wait_title(&mut client, mutate_view, FIND_MUTATE_TITLE, since);
        }
        flush_frame(&mut client);
        client
            .send_ok(&dsl::find_open())
            .expect("find open on mutate");
        client
            .send_ok(&dsl::find_set_query("target"))
            .expect("set query");
        wait_find_report(&mut client, 2, Some(1), deadline());

        // The fixture mutates its own DOM after a fixed delay and signals it
        // through the title; then the next strategy request observes the
        // mutation and reports the fresh count as stale.
        if client
            .snapshot()
            .expect("mutate title check")
            .title
            .as_deref()
            != Some(FIND_MUTATE_CHANGED_TITLE)
        {
            let _ = wait_title(&mut client, mutate_view, FIND_MUTATE_CHANGED_TITLE, since);
        }
        key(&mut client, "enter");
        let mutated = wait_for_find_stale(&mut client, deadline());
        assert_eq!(
            mutated.find.match_count,
            Some(3),
            "the strategy recomputed the mutated DOM"
        );
        assert_eq!(
            mutated.find.current_match,
            Some(2),
            "the step landed on the fresh third match"
        );
        assert!(mutated.find.open, "the bar stays open while stale");
        assert_eq!(mutated.active_web_view, Some(mutate_view));
        assert_eq!(mutated.engine_sessions, opened.engine_sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
