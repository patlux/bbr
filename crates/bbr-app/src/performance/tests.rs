//! Unit tests for the testkit presentation probe (moved out of
//! `performance.rs` to keep the implementation file under the ~500 line
//! budget). Same privacy as the inline module: this child of
//! `crate::performance` reaches the probe's private helpers through
//! `super::*`.

use super::*;
use bbr_testkit::protocol::{BenchmarkOperationId, CommandResultState};
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

#[test]
fn command_presentation_guard_rejects_stale_generation_and_wrong_result() {
    // Fast-but-wrong guard for the §11.5 command_type benchmark: the commit
    // predicate must reject every render observation that does not carry the
    // armed command identity — a stale generation, a wrong first row, a
    // wrong selected row, or a wrong typed state — and commit exactly once
    // for the render carrying the matching identity. Observations are
    // render-gated: they only exist when the palette was actually drawn
    // (`rendered = true`).
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.observe_slot_rect(WebRect::new(260.0, 0.0, 820.0, 700.0));
    // A fresh probe starts with an armed startup pending; clear it so the
    // command measurement below is the only one collected.
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(7), BenchmarkKind::CommandType)
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.expect_command_presentation(12, "tab:2", "tab:2", CommandResultState::Results);

    // A stale generation must never commit.
    probe.observe_rendered_command(
        11,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "stale generation rejected");

    // A wrong first result key must never commit.
    probe.observe_rendered_command(
        12,
        true,
        Some("tab:1"),
        Some("tab:1"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "wrong first key rejected");

    // A wrong selected row must never commit.
    probe.observe_rendered_command(
        12,
        true,
        Some("tab:2"),
        Some("action:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "wrong selected row rejected");

    // A wrong typed state must never commit.
    probe.observe_rendered_command(
        12,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Loading,
        &[],
        0,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "wrong state rejected");

    // An armed expectation with no render observation yet must never commit.
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(8), BenchmarkKind::CommandType)
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.expect_command_presentation(13, "tab:2", "tab:2", CommandResultState::Results);
    probe.commit_current_presentation();
    assert_eq!(
        collected.borrow().len(),
        0,
        "unobserved expectation rejected"
    );

    // The render carrying the exact identity commits exactly once; a
    // duplicate is a no-op.
    probe.observe_rendered_command(
        13,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    probe.commit_current_presentation();
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1, "exactly one command-type sample");
    assert_eq!(samples[0].kind, BenchmarkKind::CommandType);
    assert_eq!(samples[0].operation_id, BenchmarkOperationId(8));
    drop(samples);
}

#[test]
fn pre_render_publication_cannot_commit_a_sample() {
    // Remediation regression: the ranking publication must arm the exact
    // command identity but must NOT emit the measurement — only a render of
    // the result surface may commit. A publication that never reaches a
    // render (no observation recorded) must leave the sample pending, and a
    // render that did not present the ranked palette (omitted/not-Results
    // frame) must clear the observation and still not commit.
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.observe_slot_rect(WebRect::new(260.0, 0.0, 820.0, 700.0));
    probe.cancel(); // drop the fresh probe's startup pending
    probe
        .begin(BenchmarkOperationId(21), BenchmarkKind::CommandType)
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.expect_command_presentation(40, "tab:2", "tab:2", CommandResultState::Results);

    // Simulate the old publish-time commit path: no render ever observed the
    // palette, so the sample must stay pending — the publication alone can
    // never emit.
    probe.commit_current_presentation();
    assert_eq!(
        collected.borrow().len(),
        0,
        "pre-render publication must not emit a sample"
    );
    assert_eq!(
        probe.pending_kind(),
        Some(BenchmarkKind::CommandType),
        "the sample stays pending until a render presents the palette"
    );

    // A frame that omitted the palette (typed state Loading, rendered
    // false) clears the observation and must not commit either.
    probe.observe_rendered_command(40, false, None, None, CommandResultState::Loading, &[], 0);
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "omitted palette rejected");

    // Only the render that actually drew the ranked palette commits, exactly
    // once.
    probe.observe_rendered_command(
        40,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1, "the palette render commits exactly once");
    assert_eq!(samples[0].kind, BenchmarkKind::CommandType);
    drop(samples);
}

#[test]
fn old_render_identity_cannot_commit_a_new_arming() {
    // Remediation regression: re-arming a new expectation clears the
    // observation from the previous sample's render, so an old render
    // identity (stale generation/keys drawn before the new arming) can never
    // satisfy the new expectation.
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.observe_slot_rect(WebRect::new(260.0, 0.0, 820.0, 700.0));
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(31), BenchmarkKind::CommandType)
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.expect_command_presentation(50, "tab:2", "tab:2", CommandResultState::Results);
    // The first sample's render drew the matching identity and committed.
    probe.observe_rendered_command(
        50,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 1);

    // A second sample re-arms with a fresh generation; the arming clears the
    // old render's observation.
    probe
        .begin(BenchmarkOperationId(32), BenchmarkKind::CommandType)
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.expect_command_presentation(51, "tab:2", "tab:2", CommandResultState::Results);
    // The old render identity (generation 50) must not commit the new
    // expectation, even if a stale commit attempt happens before any new
    // render.
    probe.commit_current_presentation();
    assert_eq!(
        collected.borrow().len(),
        1,
        "an old render identity cannot commit a new arming"
    );
    // The new arming only commits from a render that drew generation 51.
    probe.observe_rendered_command(
        51,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    let samples = collected.borrow();
    assert_eq!(samples.len(), 2, "exactly two command-type samples");
    assert_eq!(samples[1].operation_id, BenchmarkOperationId(32));
    drop(samples);
}

#[test]
fn non_command_benchmarks_ignore_the_command_identity() {
    // The command-presentation identity is an additive guard: benchmarks
    // that arm no command expectation commit on the generic state match.
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, false, CommandBarMode::Inactive);
    probe.observe_slot_rect(WebRect::new(260.0, 0.0, 820.0, 700.0));
    probe.cancel(); // drop the fresh probe's startup pending
    probe
        .begin(BenchmarkOperationId(9), BenchmarkKind::WarmTabSwitch)
        .unwrap();
    probe.expect_browser_state(Some(WebViewId(1)), true);
    // An unrelated observation must not block the generic commit.
    probe.observe_rendered_command(
        1,
        true,
        Some("tab:2"),
        Some("tab:2"),
        CommandResultState::Results,
        &[],
        0,
    );
    probe.commit_current_presentation();
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].kind, BenchmarkKind::WarmTabSwitch);
}

#[test]
fn full_search_guard_requires_exact_window_and_total() {
    // The `command_bar_full_search` guard (queue item 16): the complete
    // result-set presentation must carry the exact limit-16 row-key window
    // plus the truthful total match count — a partial/truncated window, a
    // reordered window, a wrong total, or a stale generation must reject the
    // sample, and only the render drawing the exact armed presentation
    // commits. The first/selected-key guard alone cannot see a degraded
    // complete presentation.
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    probe.observe_slot_rect(WebRect::new(260.0, 0.0, 820.0, 700.0));
    probe.cancel(); // drop the fresh probe's startup pending
    probe
        .begin(
            BenchmarkOperationId(60),
            BenchmarkKind::CommandBarFullSearch,
        )
        .unwrap();
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Location);
    let keys: Vec<String> = (0..16).map(|i| format!("history:{}", 10000 - i)).collect();
    probe.expect_command_full_set(90, &keys, 10_001, CommandResultState::Results);

    // A stale generation with the exact window must never commit.
    probe.observe_rendered_command(
        89,
        true,
        Some("history:10000"),
        Some("history:10000"),
        CommandResultState::Results,
        &keys,
        10_001,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "stale generation rejected");

    // A truncated window (row 15 missing) must never commit.
    let truncated: Vec<String> = keys[..15].to_vec();
    probe.observe_rendered_command(
        90,
        true,
        Some("history:10000"),
        Some("history:10000"),
        CommandResultState::Results,
        &truncated,
        10_001,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "truncated window rejected");

    // A reordered window must never commit.
    let mut reordered = keys.clone();
    reordered.swap(0, 1);
    probe.observe_rendered_command(
        90,
        true,
        Some("history:9999"),
        Some("history:9999"),
        CommandResultState::Results,
        &reordered,
        10_001,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "reordered window rejected");

    // A wrong total must never commit.
    probe.observe_rendered_command(
        90,
        true,
        Some("history:10000"),
        Some("history:10000"),
        CommandResultState::Results,
        &keys,
        1_000,
    );
    probe.commit_current_presentation();
    assert_eq!(collected.borrow().len(), 0, "wrong total rejected");

    // The render carrying the exact window and total commits exactly once.
    probe.observe_rendered_command(
        90,
        true,
        Some("history:10000"),
        Some("history:10000"),
        CommandResultState::Results,
        &keys,
        10_001,
    );
    probe.commit_current_presentation();
    probe.commit_current_presentation();
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1, "exactly one full-search sample");
    assert_eq!(samples[0].kind, BenchmarkKind::CommandBarFullSearch);
    assert_eq!(samples[0].operation_id, BenchmarkOperationId(60));
}
