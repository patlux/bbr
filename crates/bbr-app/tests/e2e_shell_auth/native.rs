//! The credential sheet from a real WebKit challenge (reality-gaps packet,
//! item B).
//!
//! Until 2026-09-30 only the testkit opened the sheet: the navigation
//! delegate rejected every HTTP challenge, so a real site that asked for a
//! password showed its 401 page and no sheet, while the seam-driven suites
//! were green. Here the fixture server's Basic route (`/__fx/auth/basic`,
//! `bbr` / `secret`) challenges a real WKWebView through the production
//! delegate — which also keeps the debug-build proxy regression of
//! `be35936c` covered: the delegate messages WebKit's challenge proxy on
//! every request.
//!
//! - the sheet names the host, realm, scheme and authority of the space
//!   that asked, for the view that asked;
//! - a wrong password is answered by the server with a new challenge, and
//!   the sheet asks again;
//! - the right password reaches the server (the protected page loads) and
//!   never appears in any snapshot;
//! - the profile's session keeps the credential: another tab of the same
//!   profile loads the page without asking;
//! - another profile never sees it: its tab is asked, and Cancel shows the
//!   server's 401 page.

use crate::e2e_common::*;
use crate::support::*;
use bbr_testkit::protocol::{dsl, BrowserChange, Snapshot};
use bbr_web::WebViewId;

const SIGNED_IN: &str = "BBR Auth Signed In";
const REQUIRED: &str = "BBR Auth Required";

fn type_text(client: &mut Client, text: &str) {
    for ch in text.chars() {
        type_char(client, ch);
    }
}

fn sign_in(client: &mut Client, user: &str, password: &str) {
    type_text(client, user);
    key(client, "tab");
    type_text(client, password);
    key(client, "enter");
}

/// Wait for a sheet of `view` whose challenge is not `previous`.
fn wait_new_sheet(client: &mut Client, view: WebViewId, previous: Option<u64>) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("sheet snapshot");
        if snapshot.auth.open
            && snapshot.auth.web_view == Some(view)
            && snapshot.auth.challenge != previous
        {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "no new credential sheet for {view:?}: {:?}",
            snapshot.auth
        );
        std::thread::yield_now();
    }
}

fn start(client: &mut Client, token: &str) -> (WebViewId, u64) {
    client.hello(token).expect("hello");
    let mut since = client
        .attach(None)
        .expect("attach")
        .iter()
        .map(|(seq, _)| *seq)
        .max()
        .unwrap_or(0);
    let settled = wait_for_window_settled(client, deadline());
    let view = settled.active_web_view.expect("first native view");
    if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
        since = wait_title(client, view, PAGE_ONE_TITLE, since);
    }
    (view, since)
}

#[test]
#[ignore]
fn a_webkit_challenge_opens_the_sheet_and_the_password_reaches_the_server() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = auth_fixtures();
    let protected = fixtures.url("/__fx/auth/basic");
    let (mut harness, socket, token) = spawn_harness("auth-native", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);

        client
            .send_ok(&dsl::navigate(protected.clone()))
            .expect("navigate");
        let asked = wait_new_sheet(&mut client, view, None);
        assert_sheet_centered(&mut client, "bbr-auth-sheet", "bbr-auth-veil", 300.0);
        assert_eq!(asked.auth.host, "127.0.0.1");
        assert_eq!(asked.auth.realm.as_deref(), Some("BBR Fixture"));
        assert_eq!(asked.auth.scheme, "Basic");
        assert_eq!(asked.auth.authority, "origin");
        assert_eq!(asked.auth.state, "response_requested");

        // A wrong password: the server challenges again.
        sign_in(&mut client, "bbr", "nope-pw-7");
        let again = wait_new_sheet(&mut client, view, asked.auth.challenge);
        assert!(!again.auth.username_entered && !again.auth.password_entered);

        // The right one: the protected page loads.
        sign_in(&mut client, "bbr", "secret");
        since = wait_title(&mut client, view, SIGNED_IN, since);
        let signed_in = client.snapshot().expect("signed-in snapshot");
        assert!(!signed_in.auth.open, "the sheet closed");
        assert_redaction(&signed_in, &["secret", "nope-pw-7"]);
        assert!(
            fixtures.hits("/__fx/auth/basic") >= 3,
            "challenge, wrong answer and right answer all reached the server"
        );

        // The profile's session keeps the credential: no second sheet.
        let other = open_tab(&mut client, &mut since, &protected, Some(SIGNED_IN));
        let kept = client.snapshot().expect("second tab snapshot");
        assert!(!kept.auth.open, "the same profile is not asked again");
        assert_ne!(other, view);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn another_profile_is_asked_again_and_cancel_shows_the_server_page() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = auth_fixtures();
    let protected = fixtures.url("/__fx/auth/basic");
    let (mut harness, socket, token) = spawn_harness("auth-profiles", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);
        client
            .send_ok(&dsl::navigate(protected.clone()))
            .expect("navigate");
        wait_new_sheet(&mut client, view, None);
        sign_in(&mut client, "bbr", "secret");
        since = wait_title(&mut client, view, SIGNED_IN, since);

        client
            .send_ok(&dsl::create_profile("Private", "Private Space"))
            .expect("create profile");
        let (next, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = next;
        let BrowserChange::ProfileCreated { space, .. } = change else {
            unreachable!("matched ProfileCreated");
        };
        client
            .send_ok(&dsl::activate_space(space))
            .expect("activate the other profile's space");
        since = wait_browser_change(&mut client, BrowserChange::SpaceActivated { space }, since);
        let private = open_tab(&mut client, &mut since, &protected, None);
        let asked = wait_new_sheet(&mut client, private, None);
        assert_eq!(asked.auth.realm.as_deref(), Some("BBR Fixture"));

        key(&mut client, "escape");
        wait_title(&mut client, private, REQUIRED, since);
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert!(!cancelled.auth.open);
        assert_eq!(cancelled.auth.outcome.as_deref(), Some("cancelled"));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
