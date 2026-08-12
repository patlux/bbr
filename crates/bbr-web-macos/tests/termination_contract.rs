//! Public-API-only native adapter lifecycle contract tests (renderer
//! lifecycle acceptance decision, A2).
//!
//! These tests invoke BBR's own retained `WKNavigationDelegate`
//! implementation directly through the documented selectors and map the
//! documented `WKErrorDomain`/`WKErrorWebContentProcessTerminated` error to
//! the same portable termination authority as the
//! `webViewWebContentProcessDidTerminate:` callback.
//!
//! Calling BBR's own delegate implementation is an adapter contract test: it
//! does **not** claim that the test caused WebKit itself to terminate. No
//! private selector, KVC, helper PID, or process operation is used — only
//! public `WKWebView`/`WKNavigationDelegate`/`NSError` API.
//!
//! The delegate and the bare `WKWebView` are main-thread-only public
//! objects, so this binary uses a custom harness (`harness = false` in
//! `Cargo.toml`): `main` runs on the process's main thread, where
//! `MainThreadMarker::new()` succeeds and the AppKit objects can be created
//! and invoked directly. A default Rust test harness would run each test on
//! a spawned thread, where the main dispatch queue is never serviced and any
//! main-queue hop would deadlock.

#![cfg(target_os = "macos")]

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::{msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::NSView;
use objc2_foundation::{NSError, NSPoint, NSRect, NSSize, NSString};
use objc2_web_kit::{WKNavigation, WKWebView, WKWebViewConfiguration};

use bbr_web::web_errors::{CrashReason, IncidentCategory, IncidentSubReason};
use bbr_web::{ProfileId, WebEvent, WebViewId};
use bbr_web_macos::delegate::{EventSink, IncidentSink, WebViewDelegate};
use bbr_web_macos::error_mapping::{
    map_error, map_termination, ClassifiedIncident, WK_ERROR_DOMAIN,
    WK_ERROR_WEB_CONTENT_PROCESS_TERMINATED,
};
use bbr_web_macos::new_window::NewWindowCoordinator;

/// The events and classified incidents one delegate invocation produced.
#[derive(Default)]
struct TerminationProbe {
    events: Rc<RefCell<Vec<WebEvent>>>,
    incidents: Rc<RefCell<Vec<ClassifiedIncident>>>,
}

/// Build a retained `WebViewDelegate` bound to `id`/`profile` whose sinks
/// record every emitted event and classified incident. The loading cover is
/// a plain main-thread `NSView` (the delegate's ivar is `Retained<NSView>`);
/// the tests never navigate, so the cover is never shown.
fn build_delegate(
    mtm: MainThreadMarker,
    id: WebViewId,
    profile: ProfileId,
) -> (Retained<WebViewDelegate>, TerminationProbe) {
    let probe = TerminationProbe::default();
    let events: EventSink = Rc::new(RefCell::new(None));
    let incidents: IncidentSink = Rc::new(RefCell::new(None));
    *events.borrow_mut() = Some(Box::new({
        let captured = Rc::clone(&probe.events);
        move |event| captured.borrow_mut().push(event)
    }));
    *incidents.borrow_mut() = Some(Box::new({
        let captured = Rc::clone(&probe.incidents);
        move |incident| captured.borrow_mut().push(incident)
    }));
    let cover = NSView::new(mtm);
    let delegate = WebViewDelegate::new(
        mtm,
        id,
        profile,
        events,
        incidents,
        cover,
        bbr_web_macos::delegate::DelegateHandles {
            new_window: NewWindowCoordinator::new(),
            favicon_acquisition: bbr_web_macos::favicon_acquisition::FaviconAcquisitionHost::new(
                Rc::new(RefCell::new(None)),
            ),
        },
    );
    (delegate, probe)
}

/// A bare public `WKWebView` (no window, no load) — the receiver the
/// delegate methods are documented to take. Creating it is public API; it
/// never navigates and never spawns a helper for this test.
fn bare_web_view(mtm: MainThreadMarker) -> Retained<WKWebView> {
    let config = unsafe { WKWebViewConfiguration::new(mtm) };
    let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1.0, 1.0));
    unsafe { WKWebView::initWithFrame_configuration(WKWebView::alloc(mtm), frame, &config) }
}

/// The documented `WKErrorDomain`/`WKErrorWebContentProcessTerminated`
/// `NSError` WebKit delivers when a content process terminates.
fn termination_error() -> Retained<NSError> {
    let domain = NSString::from_str(WK_ERROR_DOMAIN);
    unsafe {
        NSError::errorWithDomain_code_userInfo(
            &domain,
            WK_ERROR_WEB_CONTENT_PROCESS_TERMINATED as isize,
            None,
        )
    }
}

fn termination_callback_emits_page_crashed_and_classified_process_crash() {
    let mtm = MainThreadMarker::new().expect("custom harness runs on the main thread");
    let id = WebViewId(7);
    let profile = ProfileId(3);
    let (delegate, probe) = build_delegate(mtm, id, profile);
    let web_view = bare_web_view(mtm);
    // Direct invocation of BBR's own registered implementation through
    // the documented `WKNavigationDelegate` selector — the exact call
    // WebKit makes when a content process terminates. This is an
    // adapter contract test; it does not claim the test caused WebKit
    // itself to terminate.
    unsafe {
        let _: () = msg_send![&*delegate, webViewWebContentProcessDidTerminate: &*web_view];
    }
    let events = (*probe.events.borrow()).clone();
    let incidents = (*probe.incidents.borrow()).clone();
    assert_eq!(
        events,
        vec![WebEvent::PageCrashed(WebViewId(7))],
        "exactly the bound PageCrashed(WebViewId)"
    );
    assert_eq!(incidents.len(), 1, "exactly one classified incident");
    let incident = &incidents[0];
    assert_eq!(
        incident.profile,
        ProfileId(3),
        "incident scoped to the bound profile"
    );
    assert_eq!(
        incident.web_view,
        WebViewId(7),
        "incident scoped to the bound view"
    );
    assert_eq!(
        incident.reason,
        IncidentSubReason::Crash(CrashReason::WebContentCrashed)
    );
    assert_eq!(incident.reason.category(), IncidentCategory::ProcessCrash);
    assert_eq!(
        incident.requested_url, None,
        "a termination has no failing URL"
    );
}

fn wk_error_domain_termination_maps_to_the_same_authority() {
    // The documented error form of a dead content process maps to the exact
    // same portable termination authority as the delegate callback — never
    // to an invented navigation cause.
    let outcome = map_error(WK_ERROR_DOMAIN, WK_ERROR_WEB_CONTENT_PROCESS_TERMINATED)
        .expect("documented termination error maps");
    assert_eq!(outcome, map_termination());
    assert_eq!(
        outcome.reason,
        IncidentSubReason::Crash(CrashReason::WebContentCrashed)
    );
    assert_eq!(outcome.reason.category(), IncidentCategory::ProcessCrash);
    // Other WKErrorDomain codes never fabricate a navigation or crash cause.
    for code in [
        1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    ] {
        assert_eq!(
            map_error(WK_ERROR_DOMAIN, code),
            None,
            "WKErrorDomain code {code}"
        );
    }
}

fn did_fail_navigation_with_termination_error_routes_to_termination_authority() {
    let mtm = MainThreadMarker::new().expect("custom harness runs on the main thread");
    let id = WebViewId(9);
    let profile = ProfileId(4);
    let (delegate, probe) = build_delegate(mtm, id, profile);
    let web_view = bare_web_view(mtm);
    let error = termination_error();
    unsafe {
        let _: () = msg_send![
            &*delegate,
            webView: &*web_view,
            didFailNavigation: None::<&WKNavigation>,
            withError: &*error
        ];
    }
    let events = (*probe.events.borrow()).clone();
    let incidents = (*probe.incidents.borrow()).clone();
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, WebEvent::NavigationFailed(..))),
        "a dead renderer must not emit a misleading NavigationFailed: {events:?}"
    );
    // The failure path clears the loading flag before routing to the
    // termination authority; the essential contract is exactly one
    // `PageCrashed` for the bound view and no invented navigation cause.
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, WebEvent::PageCrashed(..)))
            .count(),
        1,
        "exactly one PageCrashed: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WebEvent::PageCrashed(v) if *v == WebViewId(9))),
        "the termination error routes to the same PageCrashed authority: {events:?}"
    );
    assert_eq!(incidents.len(), 1, "exactly one classified incident");
    assert_eq!(
        incidents[0].reason.category(),
        IncidentCategory::ProcessCrash
    );
    assert_eq!(incidents[0].web_view, WebViewId(9));
}

fn did_fail_provisional_navigation_with_termination_error_routes_to_termination_authority() {
    let mtm = MainThreadMarker::new().expect("custom harness runs on the main thread");
    let id = WebViewId(11);
    let profile = ProfileId(5);
    let (delegate, probe) = build_delegate(mtm, id, profile);
    let web_view = bare_web_view(mtm);
    let error = termination_error();
    unsafe {
        let _: () = msg_send![
            &*delegate,
            webView: &*web_view,
            didFailProvisionalNavigation: None::<&WKNavigation>,
            withError: &*error
        ];
    }
    let events = (*probe.events.borrow()).clone();
    let incidents = (*probe.incidents.borrow()).clone();
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, WebEvent::NavigationFailed(..))),
        "a dead renderer must not emit a misleading NavigationFailed: {events:?}"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, WebEvent::PageCrashed(..)))
            .count(),
        1,
        "exactly one PageCrashed: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WebEvent::PageCrashed(v) if *v == WebViewId(11))),
        "the termination error routes to the same PageCrashed authority: {events:?}"
    );
    assert_eq!(incidents.len(), 1);
    assert_eq!(
        incidents[0].reason.category(),
        IncidentCategory::ProcessCrash
    );
}

fn ordinary_navigation_failure_still_emits_navigation_failed() {
    // The termination routing must not swallow ordinary navigation failures:
    // a non-termination `WKErrorDomain` error keeps the legacy
    // `NavigationFailed` event and the classified incident path.
    let mtm = MainThreadMarker::new().expect("custom harness runs on the main thread");
    let id = WebViewId(13);
    let profile = ProfileId(6);
    let (delegate, probe) = build_delegate(mtm, id, profile);
    let web_view = bare_web_view(mtm);
    let domain = NSString::from_str(WK_ERROR_DOMAIN);
    let error = unsafe {
        NSError::errorWithDomain_code_userInfo(&domain, 4, None) // JS exception
    };
    unsafe {
        let _: () = msg_send![
            &*delegate,
            webView: &*web_view,
            didFailNavigation: None::<&WKNavigation>,
            withError: &*error
        ];
    }
    let events = (*probe.events.borrow()).clone();
    let incidents = (*probe.incidents.borrow()).clone();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WebEvent::NavigationFailed(v, _) if *v == WebViewId(13))),
        "ordinary failures keep the NavigationFailed event: {events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, WebEvent::PageCrashed(..))),
        "an ordinary failure is not a crash: {events:?}"
    );
    // A JS exception is not a classifiable incident (no invented cause).
    assert_eq!(incidents.len(), 0, "unclassified codes emit no incident");
}

fn main() {
    // Every scenario runs directly on the process's main thread; a panic in
    // any scenario fails the binary (non-zero exit), exactly like a test.
    termination_callback_emits_page_crashed_and_classified_process_crash();
    wk_error_domain_termination_maps_to_the_same_authority();
    did_fail_navigation_with_termination_error_routes_to_termination_authority();
    did_fail_provisional_navigation_with_termination_error_routes_to_termination_authority();
    ordinary_navigation_failure_still_emits_navigation_failed();
    println!("termination_contract: 5 passed; 0 failed");
}
