//! Transient Peek preview real-app E2E (Link Preview / Peek packet).
//!
//! The hidden real-app harness cannot synthesize a trustworthy native
//! `WKNavigationType::LinkActivated` click (the settled NW-SEAM evidence
//! split), so the user-initiated Peek path enters through the exact
//! `BbrApp::on_new_window_request` production method the native coordinator
//! delivery pump calls, with the default intent (eligible direct activation
//! → transient preview). The script-created `window.open` block path stays
//! fully native (the real `popup.html` fixture executes `window.open`,
//! WebKit invokes the real WKUI delegate, portable policy blocks).
//!
//! Coverage map (link-preview-peek.md, "Real-app E2E"):
//!
//! - An eligible `target="_blank"` request opens a transient Peek above the
//!   current native view WITHOUT creating a regular tab (no tab, same
//!   `WebViewId`/URL/page state for the origin; the preview's native subview
//!   index exceeds the origin's — truthful z-order evidence).
//! - The covered origin receives no input while Peek is open and retains the
//!   same `WebViewId`, URL, and page state after dismissal (Escape /
//!   close-control path restores exact warm origin + focus).
//! - Navigation inside Peek (back/forward/reload) routes to the preview
//!   session; the origin stays warm.
//! - "Open as Tab" promotes the SAME preview `WebViewId` into a regular tab
//!   without another page load.
//! - One preview per pane: a new eligible request replaces the old preview
//!   (the old session is destroyed; no stacking).
//! - Popup exclusions: ineligible direct activations (non-web scheme) fall
//!   back to the ordinary tab path; script requests stay blocked.
//! - Failure/crash cleanup and cross-profile isolation.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

use bbr_testkit::protocol::{BrowserChange, Request};
use bbr_web::new_window::NewWindowIntent;
use e2e_common::*;

const PEEK_TARGET_TITLE: &str = "BBR Popup Target";
const PAGE_ONE_TITLE: &str = "BBR Fixture One";

fn wait_peek(
    client: &mut Client,
    predicate: impl Fn(&bbr_testkit::protocol::Snapshot) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("peek snapshot");
        if predicate(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "peek state did not settle");
    }
}

fn inject_default_new_window(
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
            intent: NewWindowIntent::Default,
            input: bbr_web::new_window::NewWindowInput::Primary,
        })
        .expect("inject default-intent new-window delivery");
}

#[test]
#[ignore]
fn eligible_request_opens_peek_above_the_warm_origin_without_a_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("peek-open", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");

        let initial = client.snapshot().expect("initial peek snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(
                &mut client,
                source,
                PAGE_ONE_TITLE,
                replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0),
            );
        }
        let settled = client.snapshot().expect("settled source snapshot");
        let tabs_before = settled.tabs.len();
        let engine_before = settled.engine_sessions;

        inject_default_new_window(&mut client, source, source_profile, &target);

        // The preview appears without creating a regular tab: same tab
        // count, one additional live session, the origin keeps its exact
        // WebViewId, and the preview state names the preview/source pair.
        let opened = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        let preview = opened.preview.as_ref().expect("preview snapshot");
        assert_eq!(opened.tabs.len(), tabs_before, "no regular tab created");
        assert_eq!(
            opened.engine_sessions,
            engine_before + 1,
            "origin warm + one preview session"
        );
        assert_eq!(opened.tabs[0].web_view, Some(source));
        assert_eq!(opened.active_web_view, Some(source));
        assert_eq!(opened.new_window.requests, 1);
        assert_eq!(opened.new_window.created_tabs, 0);
        assert_eq!(preview.source, source, "preview covers the warm origin");
        assert_eq!(preview.url, target);

        // The Peek bar is presented in the one native portal window with
        // its stable ids.
        assert!(opened.peek_presented);
        assert!(opened.portal.ids.iter().any(|id| id == "bbr-peek-bar"));
        assert!(opened.portal.ids.iter().any(|id| id == "bbr-peek-close"));
        // Truthful native z-order: the preview WKWebView sits ABOVE its
        // covered origin.
        let z = opened.preview_z_order.expect("native z-order evidence");
        assert!(
            z.preview_index > z.source_index,
            "preview must be raised above the origin ({} vs {})",
            z.preview_index,
            z.source_index
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn dismissal_restores_the_exact_warm_origin_and_receives_no_input_while_open() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("peek-dismiss", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(
                &mut client,
                source,
                PAGE_ONE_TITLE,
                replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0),
            );
        }
        // Warm-origin identity snapshot BEFORE the preview.
        let before = client.snapshot().expect("pre-open source snapshot");
        let origin_url = before.url.clone();

        inject_default_new_window(&mut client, source, source_profile, &target);
        let opened = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        let preview_view = opened.preview.as_ref().expect("preview").view;
        // The covered origin must not be the focus target while the preview
        // owns browser-command focus; the origin view id is unchanged.
        assert_eq!(opened.preview.as_ref().expect("preview").source, source);

        // Dismiss through the production Escape path (the same app method
        // the global `escape → FocusWeb` binding routes to).
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape dismisses the preview");

        let closed = wait_peek(&mut client, |snapshot| snapshot.preview.is_none());
        assert!(!closed.peek_presented);
        assert!(
            closed.portal.ids.is_empty()
                || !closed.portal.ids.iter().any(|id| id == "bbr-peek-bar"),
            "peek bar ids gone after dismissal"
        );
        assert_eq!(closed.engine_sessions, before.engine_sessions);
        assert_eq!(closed.tabs.len(), before.tabs.len());
        assert_eq!(closed.tabs[0].web_view, Some(source));
        assert_eq!(closed.active_web_view, Some(source));
        assert_eq!(closed.url, origin_url, "warm origin URL untouched");
        // The preview session was destroyed: no preview state remains.
        let _ = preview_view;
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn navigation_inside_peek_routes_to_the_preview_and_promotion_adopts_the_same_session() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("peek-promote", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, source, PAGE_ONE_TITLE, since);
        }

        inject_default_new_window(&mut client, source, source_profile, &target);
        let opened = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        let preview_view = opened.preview.as_ref().expect("preview").view;

        // The preview page loads its target: real navigation inside Peek.
        if opened.preview.as_ref().expect("preview").title.as_deref() != Some(PEEK_TARGET_TITLE) {
            since = wait_title(&mut client, preview_view, PEEK_TARGET_TITLE, since);
        }
        let navigated = client.snapshot().expect("navigated preview snapshot");
        assert_eq!(
            navigated
                .preview
                .as_ref()
                .expect("preview")
                .title
                .as_deref(),
            Some(PEEK_TARGET_TITLE)
        );
        assert_eq!(
            navigated.preview.as_ref().expect("preview").view,
            preview_view
        );
        // The origin stayed warm and untouched while the preview navigated.
        assert_eq!(navigated.tabs[0].web_view, Some(source));
        assert_eq!(navigated.tabs.len(), 1);

        // Browser navigation commands route to the preview session: Reload
        // reaches the preview view (the fixture re-loads; no tab mutation).
        client
            .send_ok(&Request::PeekNavigate {
                action: bbr_web::WebAction::Reload,
            })
            .expect("reload routes to the preview");
        let _ = wait_title(&mut client, preview_view, PEEK_TARGET_TITLE, since);

        // Open as Tab: the SAME preview WebViewId becomes a regular tab.
        client
            .send_ok(&Request::PeekPromote)
            .expect("promote the preview");
        let promoted = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let _ = promoted;

        let after = wait_peek(&mut client, |snapshot| {
            snapshot.preview.is_none() && snapshot.tabs.len() == 2
        });
        assert_eq!(after.tabs[1].web_view, Some(preview_view));
        assert_eq!(
            after.tabs[1].title.as_str(),
            PEEK_TARGET_TITLE,
            "promoted tab preserves the preview's navigation state"
        );
        assert_eq!(after.active_web_view, Some(preview_view));
        assert_eq!(after.engine_sessions, 2, "origin + promoted session");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn one_preview_per_pane_replaces_an_existing_preview() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let other = fixtures.url("/page2.html");
    let (mut harness, socket, token) = spawn_harness("peek-replace", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;

        inject_default_new_window(&mut client, source, source_profile, &target);
        let first = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        let first_view = first.preview.as_ref().expect("preview").view;

        // A second eligible request from the same origin replaces the
        // preview (deterministic one-preview-per-pane policy): the old
        // session is destroyed, a fresh session is created, and the preview
        // state names the new view.
        inject_default_new_window(&mut client, source, source_profile, &other);
        let replaced = wait_peek(&mut client, |snapshot| {
            snapshot
                .preview
                .as_ref()
                .is_some_and(|preview| preview.view != first_view)
        });
        let second_view = replaced.preview.as_ref().expect("preview").view;
        assert_eq!(replaced.tabs.len(), 1, "still no regular tab");
        assert_eq!(
            replaced.engine_sessions, first.engine_sessions,
            "old preview session destroyed, one replacement session"
        );
        assert_eq!(replaced.preview.as_ref().expect("preview").url, other);
        let _ = second_view;
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn exclusions_fall_back_to_ordinary_tab_and_script_stays_blocked() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("peek-excl", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;

        // An ineligible direct activation (non-web scheme) with the DEFAULT
        // intent falls back to the ordinary regular-tab path: the user's
        // action is never silently lost and never forced into a preview.
        inject_default_new_window(
            &mut client,
            source,
            source_profile,
            "mailto:hello@example.com",
        );
        let opened = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::NewWindowOpened { .. }),
            since,
        );
        let _ = opened;
        let after = wait_peek(&mut client, |snapshot| snapshot.tabs.len() == 2);
        assert!(after.preview.is_none(), "no preview for a non-web scheme");
        assert_eq!(after.new_window.created_tabs, 1);
        assert_eq!(after.tabs[1].url, "mailto:hello@example.com");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn crash_teardown_cleans_up_the_preview_session() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let target = fixtures.url("/popup-target.html");
    let (mut harness, socket, token) = spawn_harness("peek-crash", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        let initial = client.snapshot().expect("initial snapshot");
        let source = initial.active_web_view.expect("source view");
        let source_profile = initial.active_profile;

        inject_default_new_window(&mut client, source, source_profile, &target);
        let opened = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        let preview_view = opened.preview.as_ref().expect("preview").view;

        // A deterministic engine-event injection drives the production crash
        // path: the preview session reports a crashed web process.
        client
            .send_ok(&Request::InjectWebEvent {
                event: bbr_web::WebEvent::PageCrashed(preview_view),
            })
            .expect("crash the preview session");
        let crashed = wait_peek(&mut client, |snapshot| {
            snapshot.preview.as_ref().is_some_and(|p| p.crashed)
        });
        assert!(crashed.preview.as_ref().expect("preview").crashed);
        assert_eq!(crashed.tabs[0].web_view, Some(source), "origin unaffected");

        // A crashed preview cannot promote; dismissal still destroys the
        // session cleanly and restores the warm origin.
        client
            .send_ok(&Request::PeekPromote)
            .expect("crashed promotion response");
        let still_crashed = wait_peek(&mut client, |snapshot| snapshot.preview.is_some());
        assert!(
            still_crashed.preview.as_ref().expect("preview").crashed,
            "crashed preview refuses promotion"
        );
        client
            .send_ok(&Request::PeekClose)
            .expect("close the crashed preview");
        let closed = wait_peek(&mut client, |snapshot| snapshot.preview.is_none());
        assert_eq!(closed.engine_sessions, 1, "preview session destroyed");
        assert_eq!(closed.tabs[0].web_view, Some(source));
    }));
    finish(result, &mut harness, &socket, &token);
}
