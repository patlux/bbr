//! Open + navigation scenarios (task packet `find-in-page-portal-bar`).

use crate::e2e_common::*;
use crate::support::*;

/// The iteration-policy first proof: one `Cmd-F` open, one query, one
/// WebKit-reported count in a real WKWebView. The bar opens as the `0xC3`
/// `NativeAboveWeb` presentation over the real web view, wins AppKit hit
/// testing at the portal frame center (owner-classified Window Server
/// evidence), reports count 3 / ordinal 1 for the deterministic repeated
/// fixture (hidden text excluded), and never touches tab/session/WebView
/// identity, profile, space, or chrome focus. Uses a SHOWN window because
/// the frontmost evidence requires an on-screen parent.
#[test]
#[ignore]
fn find_bar_opens_over_real_webview_reports_count_and_keeps_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, repeated, _zero, _mutate, _isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("find-open", &repeated);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let first_view = settled.active_web_view.expect("first native view");
        assert!(first_view.0 > 0, "a real WKWebView backs the active tab");
        assert_eq!(settled.engine_kind, "native-wkwebview");
        assert!(settled.web_view_hosted);
        assert_find_closed(&settled);
        let _ = wait_fixture_loaded(&mut client, first_view, FIND_REPEATED_TITLE, since);

        // The production Cmd-F key binding through GPUI's real input path.
        key(&mut client, "cmd-f");
        let open = wait_find_open(&mut client, deadline());
        assert_find_open(&open);
        assert_eq!(open.find.query, "");
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&open, &settled);

        // One query, one reported count (the iteration-policy first proof).
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        let reported = wait_find_report(&mut client, 3, Some(1), deadline());
        assert_eq!(reported.find.query, "alice");
        assert!(reported.find.open, "the bar stays open after the report");
        assert_find_open(&reported);
        assert_ownership_preserved(&reported, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Enter/Shift-Enter navigation through the real GPUI key path, wrap-around
/// at both boundaries, a replacement query restarting the search, and the
/// zero-match report. Every step preserves the session identity.
#[test]
#[ignore]
fn find_navigation_wraps_and_zero_matches_report_deterministically() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, repeated, _zero, _mutate, _isolation) = find_fixtures();
    let (mut harness, socket, token) = spawn_harness("find-nav", &repeated);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        assert_find_closed(&settled);
        let view = settled.active_web_view.expect("first native view");
        let _ = wait_fixture_loaded(&mut client, view, FIND_REPEATED_TITLE, since);

        client.send_ok(&dsl::find_open()).expect("find open");
        flush_frame(&mut client);
        let open = client.snapshot().expect("open snapshot");
        assert_find_open(&open);

        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("set query");
        wait_find_report(&mut client, 3, Some(1), deadline());

        // Enter steps forward; past the last match it wraps to the first.
        key(&mut client, "enter");
        wait_find_report(&mut client, 3, Some(2), deadline());
        key(&mut client, "enter");
        wait_find_report(&mut client, 3, Some(3), deadline());
        key(&mut client, "enter");
        let wrapped = wait_find_report(&mut client, 3, Some(1), deadline());
        assert!(!wrapped.find.stale, "wrap-around is not staleness");

        // Shift-Enter steps backward and wraps to the last match.
        key(&mut client, "shift-enter");
        wait_find_report(&mut client, 3, Some(3), deadline());

        // A replacement query restarts the search deterministically.
        client
            .send_ok(&dsl::find_set_query("bob"))
            .expect("replacement query");
        let replaced = wait_find_report(&mut client, 1, Some(1), deadline());
        assert_eq!(replaced.find.query, "bob");

        // Zero matches: count 0, no ordinal, highlights cleared.
        client
            .send_ok(&dsl::find_set_query("xyzzy"))
            .expect("zero query");
        let zero = wait_find_report(&mut client, 0, None, deadline());
        assert!(zero.find.open, "the bar stays open on zero matches");

        // Back to a matching query restores a live report.
        client
            .send_ok(&dsl::find_set_query("alice"))
            .expect("restore query");
        wait_find_report(&mut client, 3, Some(1), deadline());

        let final_snapshot = client.snapshot().expect("final snapshot");
        assert_ownership_preserved(&final_snapshot, &settled);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
