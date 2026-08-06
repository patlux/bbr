//! Unit tests for the testkit presentation probe (moved out of
//! `performance.rs` to keep the implementation file under the ~500 line
//! budget). Same privacy as the inline module: this child of
//! `crate::performance` reaches the probe's private helpers through
//! `super::*`.

use super::*;
use bbr_testkit::protocol::BenchmarkOperationId;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn probe_with_sink() -> (PerformanceProbe, Rc<RefCell<Vec<PresentationMeasurement>>>) {
    let probe = PerformanceProbe::new(Instant::now(), None);
    let collected = Rc::new(RefCell::new(Vec::new()));
    let sink_collected = collected.clone();
    probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
    (probe, collected)
}

#[test]
fn page_ready_emits_only_on_expected_view_and_title() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.arm_page_ready(BenchmarkOperationId(0), WebViewId(1), "Fixture");
    // Wrong title must not commit.
    probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Other".into()));
    assert_eq!(collected.borrow().len(), 0);
    // Wrong view must not commit.
    probe.note_web_event(&WebEvent::TitleChanged(WebViewId(2), "Fixture".into()));
    assert_eq!(collected.borrow().len(), 0);
    // Correct title + view commits a single PageReady sample.
    probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Fixture".into()));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].kind, BenchmarkKind::PageReady);
    assert_eq!(samples[0].page_title.as_deref(), Some("Fixture"));
    // A later unrelated event must not emit a duplicate.
    probe.note_web_event(&WebEvent::LoadingChanged(WebViewId(1), false));
    assert_eq!(samples.len(), 1);
    drop(samples);
}

#[test]
fn page_ready_requires_observed_title_not_idle_signal() {
    // L4 regression: a loading-finished (idle) signal with no matching
    // title must NOT commit page-ready, otherwise the milestone records the
    // expectation instead of genuine fixture-content evidence.
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.arm_page_ready(BenchmarkOperationId(0), WebViewId(1), "Fixture");
    probe.note_web_event(&WebEvent::LoadingChanged(WebViewId(1), false));
    assert_eq!(
        collected.borrow().len(),
        0,
        "idle-only must not satisfy the page-ready content guard"
    );
    // The observed title is what commits it.
    probe.note_web_event(&WebEvent::TitleChanged(WebViewId(1), "Fixture".into()));
    assert_eq!(collected.borrow().len(), 1);
}

#[test]
fn startup_is_not_injected() {
    // L5: injection targets interaction latency, not the cold-startup
    // ceiling, so a delayed self-test run still yields an honest startup
    // sample that shares the page-ready (uninjected) clock.
    let probe = PerformanceProbe::new(Instant::now(), Some(Duration::from_millis(40)));
    let collected = Rc::new(RefCell::new(Vec::new()));
    let sink_collected = collected.clone();
    probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        1,
        "initial startup presentation must commit"
    );
    let sample = collected.borrow()[0].clone();
    assert_eq!(sample.kind, BenchmarkKind::Startup);
    assert!(
        sample.duration_ns < 40_000_000,
        "startup must not absorb the injected delay"
    );
}

#[test]
fn injected_delay_inflates_measured_duration() {
    let probe = PerformanceProbe::new(Instant::now(), Some(Duration::from_millis(40)));
    let collected = Rc::new(RefCell::new(Vec::new()));
    let sink_collected = collected.clone();
    probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    // A fresh probe starts with an armed startup pending; clear it so the
    // interaction measurement below is the only one collected.
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(1), BenchmarkKind::WarmTabSwitch)
        .unwrap();
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    let sample = samples.first().expect("one sample");
    assert_eq!(sample.kind, BenchmarkKind::WarmTabSwitch);
    assert!(
        sample.duration_ns >= 40_000_000,
        "injected delay must be reflected in measured work"
    );
}
