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
    NewWindowDecision, NewWindowDelivery, NewWindowDisposition, NewWindowFeatures, NewWindowInput,
    NewWindowOutcome, NewWindowRequest,
};
use bbr_web::{ProfileId, WebViewId};
use bbr_web_macos::{NewWindowContext, NewWindowCoordinator};

fn context(url: &str) -> NewWindowContext {
    NewWindowContext {
        source: WebViewId(7),
        source_profile: ProfileId(3),
        url: url.into(),
        user_initiated: true,
        input: NewWindowInput::Primary,
        features: NewWindowFeatures::default(),
    }
}

/// A Command-primary / middle-click context (background link tabs packet).
fn background_context(url: &str) -> NewWindowContext {
    let mut c = context(url);
    c.input = NewWindowInput::Background;
    c
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
fn every_decision_is_delivered_exactly_once_with_its_request() {
    for decision in [
        NewWindowDecision::Preview,
        NewWindowDecision::OpenTab,
        NewWindowDecision::External,
        NewWindowDecision::Block,
    ] {
        let mut coordinator = NewWindowCoordinator::new();
        coordinator.set_handler(Box::new(move |_| decision));
        let deliveries: Rc<RefCell<Vec<NewWindowDelivery>>> = Rc::new(RefCell::new(Vec::new()));
        let delivered = Rc::clone(&deliveries);
        coordinator.set_delivery_reporter(Box::new(move |delivery| {
            delivered.borrow_mut().push(delivery);
        }));

        let expected = context("https://example.com/delivered").to_request();
        let outcome = coordinator.handle(&context("https://example.com/delivered"));
        let deliveries = deliveries.borrow();
        assert_eq!(deliveries.len(), 1, "one delivery for {decision:?}");
        assert_eq!(deliveries[0].request, expected);
        assert_eq!(deliveries[0].outcome, outcome);
    }
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

// ---------------------------------------------------------------------------
// Background input classification (background link tabs packet)
// ---------------------------------------------------------------------------

const COMMAND_FLAG: usize = 1 << 20; // NSEventModifierFlagCommand
                                     // `WKNavigationAction.buttonNumber` is WebKit's button mask (observed on
                                     // real clicks): left 1, right 2, middle 4.
const LEFT: isize = 1;
const RIGHT: isize = 2;
const MIDDLE: isize = 4;

#[test]
fn classify_plain_primary_click_is_primary() {
    assert_eq!(bbr_web_macos::classify_input(0, 0), NewWindowInput::Primary);
    assert_eq!(
        bbr_web_macos::classify_input(0, LEFT),
        NewWindowInput::Primary
    );
}

#[test]
fn classify_right_button_is_not_a_middle_click() {
    assert_eq!(
        bbr_web_macos::classify_input(0, RIGHT),
        NewWindowInput::Primary
    );
}

#[test]
fn classify_command_primary_click_is_background() {
    assert_eq!(
        bbr_web_macos::classify_input(COMMAND_FLAG, LEFT),
        NewWindowInput::Background
    );
}

#[test]
fn classify_middle_click_is_background() {
    assert_eq!(
        bbr_web_macos::classify_input(0, MIDDLE),
        NewWindowInput::Background
    );
}

#[test]
fn classify_command_middle_click_is_background() {
    assert_eq!(
        bbr_web_macos::classify_input(COMMAND_FLAG, MIDDLE),
        NewWindowInput::Background
    );
}

#[test]
fn classify_command_with_other_modifiers_stays_background() {
    // Cmd+Shift and Cmd+Option still carry the Command bit: the background
    // gesture wins. Shift ALONE is deliberately NOT classified (a separate
    // product decision) and stays Primary.
    assert_eq!(
        bbr_web_macos::classify_input(COMMAND_FLAG | (1 << 17), 1),
        NewWindowInput::Background
    );
    assert_eq!(
        bbr_web_macos::classify_input(1 << 17, 1),
        NewWindowInput::Primary
    );
    assert_eq!(
        bbr_web_macos::classify_input(1 << 18, 1),
        NewWindowInput::Primary
    );
}

#[test]
fn context_carries_the_input_disposition_into_the_request() {
    let c = background_context("https://example.com/child");
    let r = c.to_request();
    assert_eq!(r.input, NewWindowInput::Background);
    let p = context("https://example.com/child");
    assert_eq!(p.to_request().input, NewWindowInput::Primary);
}

#[test]
fn background_context_decides_and_delivers_a_background_session() {
    let mut coordinator = NewWindowCoordinator::new();
    coordinator.set_handler(Box::new(|request| {
        bbr_web::new_window::default_decision(&request)
    }));
    let deliveries: Rc<RefCell<Vec<NewWindowDelivery>>> = Rc::new(RefCell::new(Vec::new()));
    let delivered = Rc::clone(&deliveries);
    coordinator.set_delivery_reporter(Box::new(move |delivery| {
        delivered.borrow_mut().push(delivery);
    }));
    let outcome = coordinator.handle(&background_context("https://example.com/child"));
    let NewWindowOutcome::CreateSession { session } = outcome else {
        panic!("expected a session");
    };
    assert_eq!(
        session.disposition(),
        NewWindowDisposition::OpenBackgroundTab
    );
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.url(), "https://example.com/child");
    assert_eq!(deliveries.borrow().len(), 1);
}
