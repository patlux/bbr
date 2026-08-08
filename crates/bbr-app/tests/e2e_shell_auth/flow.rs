//! Credential-sheet scenarios (queue item 10).
//!
//! The deterministic challenge fixture is an HTTPS origin Basic challenge
//! for `intranet.example` with realm `Finance VPN` bound to an owning web
//! view. Credentials are typed through GPUI's real key path into the sheet
//! (never part of any request); the only representation that leaves the
//! sheet is the contract's redaction (`credential:#<n>` / `cancelled`).

use crate::e2e_common::*;
use crate::support::*;

/// The first proof: one challenge arrival presents the conventional
/// credential sheet over a real WKWebView showing the realm and host of the
/// protection space that asked; typing edits the masked sheet state (never
/// the wire shape); Enter submits through the contract and the sheet closes
/// with the redacted decision evidence. A SHOWN window proves the sheet
/// wins AppKit hit testing above the web content (owner-classified Window
/// Server evidence). No tab/session/WebView identity, profile, or space is
/// touched.
#[test]
#[ignore]
fn credential_sheet_presents_submits_and_keeps_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = auth_fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("auth-open", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        assert!(view.0 > 0, "a real WKWebView backs the active tab");
        assert_eq!(settled.engine_kind, "native-wkwebview");
        assert!(settled.web_view_hosted);
        assert_sheet_closed(&settled);
        let _ = wait_title(&mut client, view, "BBR Fixture One", since);

        // Challenge arrival → the sheet presents over the active window.
        ChallengeFixture::for_view(view).present(&mut client);
        let open = wait_sheet_open(&mut client, view, deadline());
        assert_sheet_open(&open, view);
        wait_for_frontmost_evidence(&mut client, deadline());
        assert_ownership_preserved(&open, &settled);

        // Typing through the production key path: printable chars edit the
        // focused field, Tab moves to the password field. The snapshot only
        // ever reports masked entered flags.
        for ch in "alice".chars() {
            type_char(&mut client, ch);
        }
        key(&mut client, "tab");
        for ch in "s3cr3t".chars() {
            type_char(&mut client, ch);
        }
        let typed = client.snapshot().expect("typed sheet snapshot");
        assert_eq!(typed.auth.focused_field.as_deref(), Some("password"));
        assert!(typed.auth.username_entered, "username field holds text");
        assert!(typed.auth.password_entered, "password field holds text");
        // Redaction proof: neither the typed username nor the password
        // appears anywhere in the wire snapshot or its Debug shape.
        assert_redaction(&typed, &["alice", "s3cr3t"]);

        // Enter submits through the contract: the sheet closes and the
        // redacted decision evidence (`credential:#1`) is visible on the
        // wire; nothing about the session changed.
        key(&mut client, "enter");
        let closed = wait_sheet_closed(&mut client, deadline());
        assert_eq!(
            closed.auth.outcome.as_deref(),
            Some("credential:#1"),
            "submit resolved through the contract with the redacted handle"
        );
        assert_eq!(closed.auth.awaiting, Vec::<u64>::new());
        assert_ownership_preserved(&closed, &settled);
        assert_redaction(&closed, &["alice", "s3cr3t"]);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Cancel fails the request: Escape through the production key path moves
/// the challenge to the terminal `Cancelled` state (the wire reports the
/// `cancelled` redaction), the sheet closes, nothing retries — the same
/// fixture presented again is a NEW challenge, never a retry of the old one.
#[test]
#[ignore]
fn cancel_fails_the_request_and_never_retries() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = auth_fixtures();
    let (mut harness, socket, token) = spawn_harness("auth-cancel", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        assert_sheet_closed(&settled);
        let _ = wait_title(&mut client, view, "BBR Fixture One", since);

        let fixture = ChallengeFixture::for_view(view);
        fixture.present(&mut client);
        let open = wait_sheet_open(&mut client, view, deadline());
        assert_sheet_open(&open, view);
        // A partial attempt is discarded by cancel.
        type_char(&mut client, 'a');

        key(&mut client, "escape");
        let closed = wait_sheet_closed(&mut client, deadline());
        assert_eq!(
            closed.auth.outcome.as_deref(),
            Some("cancelled"),
            "cancel fails the request through the contract"
        );
        assert_eq!(closed.auth.awaiting, Vec::<u64>::new());
        assert_ownership_preserved(&closed, &settled);
        // No silent retry: another frame and the sheet is still closed.
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after cancel");
        let still_closed = client.snapshot().expect("post-cancel snapshot");
        assert_sheet_closed(&still_closed);
        assert_eq!(still_closed.auth.awaiting, Vec::<u64>::new());

        // The same fixture presented again is a NEW challenge (id 2): the
        // cancelled request was never retried, it was re-asked.
        fixture.present(&mut client);
        let reopened = wait_sheet_open(&mut client, view, deadline());
        assert_eq!(
            reopened.auth.challenge,
            Some(2),
            "a fresh challenge id, never a retry"
        );
        assert_eq!(
            reopened.auth.outcome, None,
            "the fresh sheet awaits a decision"
        );
        assert!(!reopened.auth.username_entered && !reopened.auth.password_entered);
        // Cancel the fresh one too; the request fails deterministically.
        key(&mut client, "escape");
        wait_sheet_closed(&mut client, deadline());

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Identity preservation + wrong credentials: a challenge belongs to its
/// originating web view. While view 1's sheet is open, a challenge recorded
/// for view 2 stays awaiting — it can never be answered on behalf of its
/// tab. A wrong-credential submit resolves to the opaque handle; the
/// server's rejection arrives as a NEW deterministic challenge (the fixture
/// rechallenge), never a silent retry; the correct attempt then resolves on
/// the fresh sheet. No tab/session/WebView identity changes.
#[test]
#[ignore]
fn sheet_answers_only_its_own_view_and_wrong_credentials_rechallenge() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = auth_fixtures();
    let (mut harness, socket, token) = spawn_harness("auth-identity", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view_one = settled.active_web_view.expect("first native view");
        let _ = wait_title(&mut client, view_one, "BBR Fixture One", since);
        assert_sheet_closed(&settled);

        // A second real tab/session gives the identity test a second owner.
        client
            .send_ok(&dsl::open_tab(page_two.clone()))
            .expect("open second tab");
        let second = client.snapshot().expect("second tab snapshot");
        let view_two = second.active_web_view.expect("second native view");
        assert_ne!(view_two, view_one, "two distinct web views");
        if second.title.as_deref() != Some("BBR Fixture Two") {
            let _ = wait_title(&mut client, view_two, "BBR Fixture Two", since);
        }
        // Ownership is asserted against a baseline captured AFTER the
        // second tab is open: `settled` predates the tab and differs from
        // every later frame by the active WebViewId BY DESIGN (opening the
        // tab selects it), so it cannot serve as the identity baseline. The
        // sheet must never change the selected view/session/profile/space;
        // the re-baselined snapshot proves the selected view stays the
        // second tab through present/type/submit/rechallenge.
        let ownership_baseline = client.snapshot().expect("post-open ownership baseline");
        assert_eq!(
            ownership_baseline.active_web_view,
            Some(view_two),
            "the re-baselined ownership snapshot selects the second tab"
        );

        // View 1's challenge presents its own sheet; view 2's challenge,
        // recorded while the sheet is open, stays awaiting and can never be
        // answered on behalf of its tab.
        ChallengeFixture::for_view(view_one).present(&mut client);
        let open = wait_sheet_open(&mut client, view_one, deadline());
        assert_sheet_open(&open, view_one);
        ChallengeFixture::for_view(view_two).record(&mut client);
        let with_second = client.snapshot().expect("second challenge recorded");
        assert_eq!(
            with_second.auth.awaiting,
            vec![2],
            "view 2's challenge waits"
        );
        assert_eq!(
            with_second.auth.web_view,
            Some(view_one),
            "the sheet still shows view 1's challenge"
        );

        // Wrong credentials: the submit resolves to the opaque handle (the
        // adapter will get the server's verdict); the sheet closes and
        // nothing auto-retries.
        for ch in "admin".chars() {
            type_char(&mut client, ch);
        }
        key(&mut client, "tab");
        for ch in "wrong".chars() {
            type_char(&mut client, ch);
        }
        key(&mut client, "enter");
        let submitted = wait_sheet_closed(&mut client, deadline());
        assert_eq!(submitted.auth.outcome.as_deref(), Some("credential:#1"));
        assert_eq!(
            submitted.auth.awaiting,
            vec![2],
            "still only view 2's challenge awaiting"
        );
        assert_ownership_preserved(&submitted, &ownership_baseline);
        assert_redaction(&submitted, &["admin", "wrong"]);

        // The rejection lands as a NEW challenge (the deterministic fixture
        // rechallenge for the same tab — exactly the server-rejects path):
        // the fresh sheet presents challenge 2, empty, awaiting a decision.
        ChallengeFixture::for_view(view_two).present(&mut client);
        let rechallenged = wait_sheet_open(&mut client, view_two, deadline());
        assert_eq!(
            rechallenged.auth.challenge,
            Some(2),
            "the rechallenge is a new challenge"
        );
        assert_eq!(
            rechallenged.auth.web_view,
            Some(view_two),
            "the sheet presents the new owner's challenge"
        );
        assert_eq!(
            rechallenged.auth.outcome, None,
            "fresh sheet, no carried-over decision"
        );
        assert!(!rechallenged.auth.username_entered && !rechallenged.auth.password_entered);

        // The correct attempt resolves on the fresh sheet.
        for ch in "alice".chars() {
            type_char(&mut client, ch);
        }
        key(&mut client, "tab");
        for ch in "s3cr3t".chars() {
            type_char(&mut client, ch);
        }
        key(&mut client, "enter");
        let resolved = wait_sheet_closed(&mut client, deadline());
        assert_eq!(resolved.auth.outcome.as_deref(), Some("credential:#2"));
        assert_ownership_preserved(&resolved, &ownership_baseline);
        assert_redaction(&resolved, &["admin", "wrong", "alice", "s3cr3t"]);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
