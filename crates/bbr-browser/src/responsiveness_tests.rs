//! Unit tests for the portable renderer-availability monitor
//! (renderer-isolation lane), split from `responsiveness.rs` so both files
//! stay under the ~500-line budget. The monitor's policy is proven with an
//! injectable clock — no sleeping, no display server.

use std::time::{Duration, Instant};

use bbr_web::WebViewId;

use crate::responsiveness::{RendererAvailability, ResponsivenessConfig, ResponsivenessMonitor};

/// A monitor whose clock is a shared mutable `now`, so tests can advance
/// time deterministically without sleeping.
fn clocked() -> (
    ResponsivenessMonitor,
    std::rc::Rc<std::cell::RefCell<Instant>>,
) {
    let now = std::rc::Rc::new(std::cell::RefCell::new(Instant::now()));
    let clock_now = now.clone();
    let monitor = ResponsivenessMonitor::with_config(
        ResponsivenessConfig::default(),
        Box::new(move || *clock_now.borrow()),
    );
    (monitor, now)
}

fn advance(now: &std::rc::Rc<std::cell::RefCell<Instant>>, by: Duration) {
    *now.borrow_mut() += by;
}

fn armed(monitor: &mut ResponsivenessMonitor, view: WebViewId, arms: &mut Vec<WebViewId>) {
    monitor.pump(view, true, &mut |v| arms.push(v));
}

#[test]
fn unknown_is_the_fresh_default() {
    let monitor = ResponsivenessMonitor::new();
    assert_eq!(monitor.state(WebViewId(1)), RendererAvailability::Unknown);
    assert!(!monitor.is_unresponsive(WebViewId(1)));
}

#[test]
fn unknown_to_responsive_to_unresponsive_to_responsive() {
    let (mut monitor, now) = clocked();
    let view = WebViewId(1);
    monitor.register(view);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert_eq!(arms, vec![view], "a visible fresh session is probed");
    // A completing probe answers: Responsive.
    let generation = monitor.generation(view).unwrap();
    assert_eq!(
        monitor.probe_completed(view, generation),
        Some(RendererAvailability::Responsive)
    );
    assert_eq!(monitor.state(view), RendererAvailability::Responsive);
    // The next due probe is armed after the cadence, then times out
    // inside its deadline: Unresponsive (fail-closed).
    let interval = monitor.config().probe_interval;
    advance(&now, interval);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert_eq!(arms, vec![view], "responsive session re-probed on cadence");
    let generation = monitor.generation(view).unwrap();
    assert_eq!(
        monitor.probe_timed_out(view, generation),
        Some(RendererAvailability::Unresponsive)
    );
    assert_eq!(monitor.state(view), RendererAvailability::Unresponsive);
    assert!(monitor.is_unresponsive(view));
    // A later completing probe answers again: Responsive.
    let generation = monitor.generation(view).unwrap();
    assert_eq!(
        monitor.probe_completed(view, generation),
        Some(RendererAvailability::Responsive)
    );
    assert_eq!(monitor.state(view), RendererAvailability::Responsive);
}

#[test]
fn deadline_policy_is_bounded_and_fail_closed() {
    let (mut monitor, now) = clocked();
    monitor.set_config(ResponsivenessConfig {
        probe_deadline: Duration::from_millis(500),
        probe_interval: Duration::from_millis(100),
        timeout_backoff: Duration::from_millis(900),
    });
    let view = WebViewId(2);
    monitor.register(view);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert!(arms.len() == 1);
    // Inside the deadline: still in flight, no transition.
    advance(&now, Duration::from_millis(300));
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert!(arms.is_empty(), "in-flight probe is not re-armed");
    assert_eq!(monitor.state(view), RendererAvailability::Unknown);
    // Past the deadline: the portable pump declares it unresponsive.
    advance(&now, Duration::from_millis(300));
    let mut timed_out = Vec::new();
    let transition = monitor.pump(view, true, &mut |v| timed_out.push(v));
    assert_eq!(transition, Some(RendererAvailability::Unresponsive));
    assert!(timed_out.is_empty());
    assert_eq!(monitor.state(view), RendererAvailability::Unresponsive);
    // Backoff: no probe is armed before it elapses.
    advance(&now, Duration::from_millis(400));
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert!(arms.is_empty(), "backoff suppresses probe traffic");
    // After the backoff the session is re-probed.
    advance(&now, Duration::from_millis(600));
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert_eq!(arms, vec![view]);
}

#[test]
fn hidden_sessions_are_never_probed() {
    let mut monitor = ResponsivenessMonitor::new();
    let view = WebViewId(3);
    monitor.register(view);
    let mut arms = Vec::new();
    assert_eq!(
        monitor.pump(view, false, &mut |v| arms.push(v)),
        None,
        "hidden session is not probed"
    );
    assert!(arms.is_empty());
    assert_eq!(monitor.state(view), RendererAvailability::Unknown);
}

#[test]
fn stale_and_late_completions_are_rejected() {
    let (mut monitor, now) = clocked();
    let view = WebViewId(4);
    monitor.register(view);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    // A completion for a generation that was never armed is dropped.
    assert_eq!(monitor.probe_completed(view, 99), None);
    // The first probe times out (its completion may still be in flight
    // in the adapter); the session is marked Unresponsive.
    let first = monitor.generation(view).unwrap();
    assert_eq!(
        monitor.probe_timed_out(view, first),
        Some(RendererAvailability::Unresponsive)
    );
    // A LATER completing probe answers; the record re-arms (generation
    // advances). The OLD probe's completion is stale: it can never
    // clear the newer timeout.
    let backoff = monitor.config().timeout_backoff;
    advance(&now, backoff);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    let second = monitor.generation(view).unwrap();
    assert!(second > first, "re-arm advances the generation");
    assert_eq!(
        monitor.probe_completed(view, first),
        None,
        "a late answer from a previous probe mutates nothing"
    );
    assert_eq!(
        monitor.state(view),
        RendererAvailability::Unresponsive,
        "the stale completion cannot clear the newer timeout"
    );
    // The current generation's completion does clear it.
    assert_eq!(
        monitor.probe_completed(view, second),
        Some(RendererAvailability::Responsive)
    );
    // A completion for a forgotten (destroyed) view is dropped.
    monitor.forget(view);
    assert_eq!(monitor.probe_completed(view, second), None);
    assert_eq!(monitor.state(view), RendererAvailability::Unknown);
    // A stale timeout for a crashed session is dropped (crash authority).
    monitor.register(view);
    monitor.on_crash(view);
    assert_eq!(monitor.probe_timed_out(view, 1), None);
    assert_eq!(monitor.state(view), RendererAvailability::Crashed);
}

#[test]
fn close_and_recreate_under_an_in_flight_probe_are_clean() {
    let (mut monitor, _now) = clocked();
    let old = WebViewId(5);
    monitor.register(old);
    let mut arms = Vec::new();
    armed(&mut monitor, old, &mut arms);
    let old_generation = monitor.generation(old).unwrap();
    // The tab is closed while a probe is in flight: the record is
    // dropped. A late completion cannot resurrect the session.
    monitor.forget(old);
    assert_eq!(monitor.probe_completed(old, old_generation), None);
    assert_eq!(monitor.state(old), RendererAvailability::Unknown);
    // The controller allocates a FRESH id for the recreated session (the
    // monitor's stale-id discipline mirrors the UnknownWebView path):
    // the old id stays forgotten and the fresh session registers its own
    // record.
    let fresh = WebViewId(6);
    monitor.register(fresh);
    let mut arms = Vec::new();
    armed(&mut monitor, fresh, &mut arms);
    assert_eq!(monitor.probe_completed(old, old_generation), None);
    let fresh_generation = monitor.generation(fresh).unwrap();
    assert_eq!(
        monitor.probe_completed(fresh, fresh_generation),
        Some(RendererAvailability::Responsive)
    );
}

#[test]
fn crash_marks_the_view_and_reconciles_siblings() {
    let (mut monitor, _now) = clocked();
    let a = WebViewId(10);
    let b = WebViewId(11);
    let c = WebViewId(12);
    for view in [a, b, c] {
        monitor.register(view);
    }
    // Two siblings are mid-backoff (their next probe is far in the
    // future) when `a` crashes.
    monitor.wait(b);
    monitor.wait(c);
    monitor.on_crash(a);
    assert_eq!(monitor.state(a), RendererAvailability::Crashed);
    // The reconciliation sweep re-arms the other live sessions so each
    // is probed independently (shared WebContent tolerance).
    let mut arms = Vec::new();
    armed(&mut monitor, b, &mut arms);
    armed(&mut monitor, c, &mut arms);
    assert_eq!(arms, vec![b, c], "siblings re-probed after the crash");
    assert_eq!(monitor.state(b), RendererAvailability::Unknown);
    assert_eq!(monitor.state(c), RendererAvailability::Unknown);
}

#[test]
fn unresponsive_timeout_reconciles_siblings_too() {
    let (mut monitor, _now) = clocked();
    let a = WebViewId(20);
    let b = WebViewId(21);
    monitor.register(a);
    monitor.register(b);
    monitor.wait(b);
    let mut arms = Vec::new();
    armed(&mut monitor, a, &mut arms);
    let generation = monitor.generation(a).unwrap();
    assert_eq!(
        monitor.probe_timed_out(a, generation),
        Some(RendererAvailability::Unresponsive)
    );
    // The sibling is due immediately after the timeout sweep.
    let mut arms = Vec::new();
    armed(&mut monitor, b, &mut arms);
    assert_eq!(arms, vec![b]);
}

#[test]
fn wait_keeps_monitoring_and_a_later_answer_clears() {
    let (mut monitor, now) = clocked();
    let view = WebViewId(30);
    monitor.register(view);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    let generation = monitor.generation(view).unwrap();
    monitor.probe_timed_out(view, generation);
    assert!(monitor.is_unresponsive(view));
    // Wait keeps monitoring: the next probe is due after one interval.
    monitor.wait(view);
    let interval = monitor.config().probe_interval;
    advance(&now, interval);
    let mut arms = Vec::new();
    armed(&mut monitor, view, &mut arms);
    assert_eq!(arms, vec![view]);
    // The page answers again (busy→alive): the state clears.
    let generation = monitor.generation(view).unwrap();
    assert_eq!(
        monitor.probe_completed(view, generation),
        Some(RendererAvailability::Responsive)
    );
    assert!(!monitor.is_unresponsive(view));
}
