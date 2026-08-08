//! Unit tests for the find-presentation guard of the testkit presentation
//! probe (task packet `find-in-page-portal-bar`).
//!
//! The find benchmarks commit only from a render of the portal bar that
//! carries the armed open/report identity: a frame that omitted the bar, a
//! pending request, a stale report, a wrong query, or a wrong count/ordinal
//! must all be rejected in the app (fast-but-wrong guard). These tests prove
//! the predicate directly, mirroring the command-benchmark guard tests in
//! `super::tests`.

use super::*;
use bbr_testkit::protocol::BenchmarkOperationId;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

fn probe_with_sink() -> (PerformanceProbe, Rc<RefCell<Vec<PresentationMeasurement>>>) {
    let probe = PerformanceProbe::new(Instant::now(), None);
    let collected = Rc::new(RefCell::new(Vec::new()));
    let sink_collected = collected.clone();
    probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
    (probe, collected)
}

fn arm_find_report(probe: &PerformanceProbe) {
    // A fresh probe starts with an armed startup pending; clear it so the
    // find measurement below is the only one collected (same as the
    // interaction tests in `super::tests`).
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(7), BenchmarkKind::FindQueryResult)
        .expect("begin find query result");
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.expect_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.expect_find_report("alice", 3, Some(1));
}

/// The render milestone records the state the portal bar actually drew; the
/// probe commits a find sample only from a matching render observation.
#[test]
fn find_report_commits_only_from_matching_render_observation() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    arm_find_report(&probe);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "no render observation yet → nothing may commit"
    );
    // The render drew the report: open, focused, query, count 3, ordinal 1,
    // no pending, no staleness.
    probe.observe_find_state(true, true, "alice", Some(3), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1, "the matching render commits exactly once");
    assert_eq!(samples[0].kind, BenchmarkKind::FindQueryResult);
    assert_eq!(samples[0].active_web_view, Some(WebViewId(1)));
    assert!(!samples[0].chrome_focused, "the find field owns focus");
    drop(samples);
}

#[test]
fn find_report_rejects_pending_and_stale_renders() {
    // A render of the bar while the WebKit request is still pending, or of a
    // report marked stale by a document change, must never satisfy the armed
    // report expectation (fast-but-wrong guard).
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    arm_find_report(&probe);
    // Same query/count/ordinal but the request is still pending.
    probe.observe_find_state(true, true, "alice", Some(3), Some(1), true, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "a pending request must not commit the report sample"
    );
    // Same report but stale (document changed since the search started).
    probe.observe_find_state(true, true, "alice", Some(3), Some(1), false, true);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "a stale report must not commit a warm sample"
    );
    // The clean report commits.
    probe.observe_find_state(true, true, "alice", Some(3), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

#[test]
fn find_report_rejects_wrong_query_count_or_ordinal() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    arm_find_report(&probe);
    // Wrong query text.
    probe.observe_find_state(true, true, "bob", Some(3), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "wrong query rejected");
    // Wrong count.
    probe.observe_find_state(true, true, "alice", Some(1), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "wrong count rejected");
    // Wrong ordinal.
    probe.observe_find_state(true, true, "alice", Some(3), Some(2), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "wrong ordinal rejected");
    // A bar render that lost focus (find not focused) is also rejected.
    probe.observe_find_state(true, false, "alice", Some(3), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "unfocused bar rejected");
    // The matching render commits.
    probe.observe_find_state(true, true, "alice", Some(3), Some(1), false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

fn arm_find_open(probe: &PerformanceProbe) {
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(9), BenchmarkKind::FindOpen)
        .expect("begin find open");
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.expect_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.expect_find_open();
}

#[test]
fn find_open_requires_an_open_focused_bar_render() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    arm_find_open(&probe);
    // No render observation yet: nothing commits.
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0);
    // A render that drew the bar open and focused commits.
    probe.observe_find_state(true, true, "", None, None, false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].kind, BenchmarkKind::FindOpen);
    drop(samples);
}

#[test]
fn find_open_rejects_a_render_that_omitted_the_bar() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    arm_find_open(&probe);
    // A render that drew a closed bar (open = false) must not commit.
    probe.observe_find_state(false, false, "", None, None, false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "a closed-bar render never commits the open sample"
    );
    // A later render that actually drew the bar commits.
    probe.observe_find_state(true, true, "", None, None, false, false);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}
