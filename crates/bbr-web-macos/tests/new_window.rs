//! Deterministic tests for the macOS new-window adapter seam.
//!
//! These run headless (no display server, no WKWebView): the coordinator's
//! request/decision/outcome flow is pure logic over the portable contract.
//! WebKit object extraction ([`extract_context`] and friends) is
//! compile-checked glue only — WebKit delivers its delegate objects, which
//! cannot be constructed deterministically in a unit test.

#![cfg(target_os = "macos")]

use std::cell::RefCell;
use std::rc::Rc;

use bbr_web::new_window::{
    NewWindowDecision, NewWindowDisposition, NewWindowFeatures, NewWindowOutcome, NewWindowRequest,
};
use bbr_web::{ProfileId, WebViewId};
use bbr_web_macos::{NewWindowContext, NewWindowCoordinator};

fn context(url: &str) -> NewWindowContext {
    NewWindowContext {
        source: WebViewId(7),
        source_profile: ProfileId(3),
        url: url.into(),
        user_initiated: true,
        features: NewWindowFeatures::default(),
    }
}

#[test]
fn context_builds_the_portable_request() {
    let c = context("https://example.com");
    let r = c.to_request();
    assert_eq!(r.source, WebViewId(7));
    assert_eq!(r.source_profile, ProfileId(3));
    assert_eq!(r.url, "https://example.com");
    assert!(r.user_initiated);
    assert_eq!(r.target, bbr_web::new_window::NewWindowTarget::NewWindow);
    assert_eq!(r.features, NewWindowFeatures::default());
}

#[test]
fn no_handler_deterministically_blocks() {
    let coordinator = NewWindowCoordinator::new();
    assert_eq!(
        coordinator.handle(&context("https://example.com")),
        NewWindowOutcome::Blocked
    );
}

#[test]
fn no_handler_block_is_reported_not_silent() {
    let mut coordinator = NewWindowCoordinator::new();
    let reported: Rc<RefCell<Option<NewWindowRequest>>> = Rc::new(RefCell::new(None));
    let report = Rc::clone(&reported);
    coordinator.set_blocked_reporter(Box::new(move |request| {
        *report.borrow_mut() = Some(request);
    }));
    let outcome = coordinator.handle(&context("https://example.com"));
    assert_eq!(outcome, NewWindowOutcome::Blocked);
    let reported = reported.borrow();
    assert_eq!(
        reported.as_ref().map(|r| r.url.as_str()),
        Some("https://example.com")
    );
    assert_eq!(
        reported.as_ref().map(|r| r.source_profile),
        Some(ProfileId(3))
    );
}

#[test]
fn handler_receives_the_exact_request() {
    let mut coordinator = NewWindowCoordinator::new();
    let seen: Rc<RefCell<Option<NewWindowRequest>>> = Rc::new(RefCell::new(None));
    let seen_for_handler = Rc::clone(&seen);
    coordinator.set_handler(Box::new(move |request| {
        *seen_for_handler.borrow_mut() = Some(request);
        NewWindowDecision::OpenTab
    }));
    let c = context("https://example.com/path");
    let outcome = coordinator.handle(&c);
    let NewWindowOutcome::CreateSession { session } = outcome else {
        panic!("expected session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.disposition(), NewWindowDisposition::OpenTab);
    assert_eq!(session.url(), "https://example.com/path");
    let seen = seen.borrow();
    let seen = seen.as_ref().expect("handler was called");
    assert_eq!(seen.source, c.source);
    assert_eq!(seen.source_profile, c.source_profile);
    assert_eq!(seen.url, c.url);
    assert_eq!(seen.user_initiated, c.user_initiated);
    assert_eq!(seen.features, c.features);
}

#[test]
fn handler_decisions_map_to_outcomes() {
    for decision in [
        NewWindowDecision::Preview,
        NewWindowDecision::OpenTab,
        NewWindowDecision::External,
        NewWindowDecision::Block,
    ] {
        let mut coordinator = NewWindowCoordinator::new();
        coordinator.set_handler(Box::new(move |_| decision));
        let outcome = coordinator.handle(&context("https://example.com"));
        match (decision, outcome) {
            (NewWindowDecision::Preview, NewWindowOutcome::CreateSession { session }) => {
                assert_eq!(session.source_profile(), ProfileId(3));
                assert_eq!(session.disposition(), NewWindowDisposition::Preview);
                assert_eq!(session.url(), "https://example.com");
            }
            (NewWindowDecision::OpenTab, NewWindowOutcome::CreateSession { session }) => {
                assert_eq!(session.source_profile(), ProfileId(3));
                assert_eq!(session.disposition(), NewWindowDisposition::OpenTab);
                assert_eq!(session.url(), "https://example.com");
            }
            (NewWindowDecision::External, NewWindowOutcome::External { url }) => {
                assert_eq!(url, "https://example.com");
            }
            (NewWindowDecision::Block, NewWindowOutcome::Blocked) => {}
            (decision, outcome) => panic!("{decision:?} produced {outcome:?}"),
        }
    }
}

#[test]
fn blocked_decision_invokes_the_reporter() {
    let mut coordinator = NewWindowCoordinator::new();
    coordinator.set_handler(Box::new(|_| NewWindowDecision::Block));
    let reported = Rc::new(RefCell::new(0));
    let report = Rc::clone(&reported);
    coordinator.set_blocked_reporter(Box::new(move |_| {
        *report.borrow_mut() += 1;
    }));
    coordinator.handle(&context("https://example.com"));
    assert_eq!(*reported.borrow(), 1);
}

#[test]
fn non_blocked_decisions_never_report() {
    for decision in [
        NewWindowDecision::Preview,
        NewWindowDecision::OpenTab,
        NewWindowDecision::External,
    ] {
        let mut coordinator = NewWindowCoordinator::new();
        coordinator.set_handler(Box::new(move |_| decision));
        let reported = Rc::new(RefCell::new(0));
        let report = Rc::clone(&reported);
        coordinator.set_blocked_reporter(Box::new(move |_| {
            *report.borrow_mut() += 1;
        }));
        coordinator.handle(&context("https://example.com"));
        assert_eq!(*reported.borrow(), 0, "for {decision:?}");
    }
}

#[test]
fn set_handler_replaces_the_previous_handler() {
    let mut coordinator = NewWindowCoordinator::new();
    coordinator.set_handler(Box::new(|_| NewWindowDecision::Block));
    coordinator.set_handler(Box::new(|_| NewWindowDecision::OpenTab));
    let NewWindowOutcome::CreateSession { session } =
        coordinator.handle(&context("https://example.com"))
    else {
        panic!("expected session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.disposition(), NewWindowDisposition::OpenTab);
}
