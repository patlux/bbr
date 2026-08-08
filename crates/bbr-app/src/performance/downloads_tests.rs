//! Unit tests for the downloads-panel presentation guard of the testkit
//! presentation probe (task packet `downloads-sidebar-panel`).
//!
//! The downloads benchmarks commit only from a render of the anchored panel
//! that carries the armed identity: a frame that omitted the panel, a wrong
//! progress fraction, a rejected event, a stale observation, or a missing
//! retry linkage must all be rejected in the app (fast-but-wrong guard).
//! These tests prove the predicate directly, mirroring the find-guard tests
//! in `super::find_tests`.

use super::*;
use bbr_testkit::protocol::{
    BenchmarkOperationId, DownloadGroup, DownloadMachineState, DownloadRowAction,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::app::downloads::DownloadMachine;
use crate::performance::downloads::{observe_row, RowObservation};

fn probe_with_sink() -> (PerformanceProbe, Rc<RefCell<Vec<PresentationMeasurement>>>) {
    let probe = PerformanceProbe::new(Instant::now(), None);
    let collected = Rc::new(RefCell::new(Vec::new()));
    let sink_collected = collected.clone();
    probe.set_sink(Box::new(move |m| sink_collected.borrow_mut().push(m)));
    (probe, collected)
}

fn arm_downloads_open(probe: &PerformanceProbe) {
    // A fresh probe starts with an armed startup pending; clear it so the
    // downloads measurement below is the only one collected.
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(21), BenchmarkKind::DownloadsPanelOpen)
        .expect("begin downloads panel open");
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    probe.expect_downloads_open();
}

fn running_row(id: u64, received: u64, total: u64) -> RowObservation {
    RowObservation {
        id,
        machine: DownloadMachineState::Running,
        group: DownloadGroup::Active,
        progress_fraction: Some(received as f64 / total as f64),
        action: DownloadRowAction::Cancel,
        retry_of: None,
    }
}

/// The render milestone records the panel state the sidebar render actually
/// drew; the probe commits a panel-open sample only from a matching render
/// observation.
#[test]
fn downloads_open_commits_only_from_matching_render_observation() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_open(&probe);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "no render observation yet → nothing may commit"
    );
    // The render drew the panel open with the keyboard cursor.
    probe.observe_downloads_state(true, true, vec![]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1, "the matching render commits exactly once");
    assert_eq!(samples[0].kind, BenchmarkKind::DownloadsPanelOpen);
    assert!(samples[0].chrome_focused, "the panel owns chrome focus");
    drop(samples);
}

#[test]
fn downloads_open_rejects_closed_or_unfocused_panel_renders() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_open(&probe);
    // A render that drew a closed panel (open = false) must not commit.
    probe.observe_downloads_state(false, false, vec![]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "a closed-panel render never commits the open sample"
    );
    // Open but the keyboard cursor left the panel (outside focus) rejects.
    probe.observe_downloads_state(true, false, vec![]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "unfocused panel rejected");
    // A later render that actually drew the panel commits.
    probe.observe_downloads_state(true, true, vec![]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

fn arm_downloads_progress(probe: &PerformanceProbe, id: u64) {
    probe.cancel();
    probe
        .begin(
            BenchmarkOperationId(22),
            BenchmarkKind::DownloadsProgressCommit,
        )
        .expect("begin downloads progress commit");
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    probe.expect_downloads_progress(id, 2048, 65536);
}

/// A progress sample commits only from a render that drew the running row at
/// exactly the armed received/total fraction.
#[test]
fn downloads_progress_commits_only_at_the_armed_fraction() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_progress(&probe, 7);
    // No render observation yet: nothing commits.
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0);
    // The row at the wrong fraction (4096 instead of 2048) rejects.
    probe.observe_downloads_state(true, true, vec![running_row(7, 4096, 65536)]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "wrong fraction rejected");
    // A terminal row (the first terminal event won) rejects.
    probe.observe_downloads_state(
        true,
        true,
        vec![RowObservation {
            machine: DownloadMachineState::Succeeded,
            group: DownloadGroup::History,
            ..running_row(7, 2048, 65536)
        }],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "terminal row rejected");
    // The matching running row at the armed fraction commits exactly once.
    probe.observe_downloads_state(true, true, vec![running_row(7, 2048, 65536)]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].kind, BenchmarkKind::DownloadsProgressCommit);
    drop(samples);
}

#[test]
fn downloads_progress_requires_the_panel_open_and_focused() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_progress(&probe, 7);
    // The row at the armed fraction but the panel render omitted/closed it.
    probe.observe_downloads_state(false, false, vec![running_row(7, 2048, 65536)]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "a closed panel cannot commit a progress sample"
    );
    probe.observe_downloads_state(true, true, vec![running_row(7, 2048, 65536)]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

fn arm_downloads_retry(probe: &PerformanceProbe) {
    probe.cancel();
    probe
        .begin(BenchmarkOperationId(23), BenchmarkKind::DownloadsRowRetry)
        .expect("begin downloads row retry");
    probe.expect_browser_state(Some(WebViewId(1)), true);
    probe.expect_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    probe.expect_downloads_retry(3, 8);
}

/// A retry sample commits only from a render that drew the fresh attempt
/// (Waiting, linked via `retry_of`, Cancel action) in the Active group.
#[test]
fn downloads_retry_commits_only_from_the_fresh_active_row_render() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_retry(&probe);
    // The failed source row alone (History) must not satisfy the retry.
    probe.observe_downloads_state(
        true,
        true,
        vec![RowObservation {
            id: 3,
            machine: DownloadMachineState::Failed,
            group: DownloadGroup::History,
            progress_fraction: None,
            action: DownloadRowAction::Retry,
            retry_of: None,
        }],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "no fresh row → no retry commit"
    );
    // The fresh Waiting row linked via `retry_of` with Cancel commits.
    probe.observe_downloads_state(
        true,
        true,
        vec![
            RowObservation {
                id: 8,
                machine: DownloadMachineState::Waiting,
                group: DownloadGroup::Active,
                progress_fraction: None,
                action: DownloadRowAction::Cancel,
                retry_of: Some(3),
            },
            RowObservation {
                id: 3,
                machine: DownloadMachineState::Failed,
                group: DownloadGroup::History,
                progress_fraction: None,
                action: DownloadRowAction::Retry,
                retry_of: None,
            },
        ],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    let samples = collected.borrow();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].kind, BenchmarkKind::DownloadsRowRetry);
    drop(samples);
}

#[test]
fn downloads_retry_rejects_broken_linkage_and_wrong_action() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    arm_downloads_retry(&probe);
    // The fresh row but with the wrong `retry_of` linkage rejects.
    probe.observe_downloads_state(
        true,
        true,
        vec![RowObservation {
            id: 8,
            machine: DownloadMachineState::Waiting,
            group: DownloadGroup::Active,
            progress_fraction: None,
            action: DownloadRowAction::Cancel,
            retry_of: Some(99),
        }],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "broken linkage rejected");
    // The fresh row but Running (the transfer already started) rejects the
    // Waiting identity.
    probe.observe_downloads_state(
        true,
        true,
        vec![RowObservation {
            id: 8,
            machine: DownloadMachineState::Running,
            group: DownloadGroup::Active,
            progress_fraction: Some(0.0),
            action: DownloadRowAction::Cancel,
            retry_of: Some(3),
        }],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 0, "non-Waiting row rejected");
    // The matching render commits.
    probe.observe_downloads_state(
        true,
        true,
        vec![RowObservation {
            id: 8,
            machine: DownloadMachineState::Waiting,
            group: DownloadGroup::Active,
            progress_fraction: None,
            action: DownloadRowAction::Cancel,
            retry_of: Some(3),
        }],
    );
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

/// A stale observation must never satisfy a newer arm: arming clears the
/// prior render's observation.
#[test]
fn downloads_arming_clears_the_prior_observation() {
    let (probe, collected) = probe_with_sink();
    probe.observe_render_state(Some(WebViewId(1)), true, true, CommandBarMode::Inactive);
    // A prior open render recorded an observation.
    probe.observe_downloads_state(true, true, vec![]);
    // The new sample arms open; the old observation is cleared.
    arm_downloads_open(&probe);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(
        collected.borrow().len(),
        0,
        "the pre-arm observation must not satisfy the fresh arm"
    );
    probe.observe_downloads_state(true, true, vec![]);
    probe.commit_presentation(WebRect::new(260.0, 0.0, 820.0, 700.0));
    assert_eq!(collected.borrow().len(), 1);
}

/// The row mapping mirrors the L6 states exactly (the render's vocabulary).
#[test]
fn observe_row_maps_the_l6_machine_states() {
    fn visual(machine: DownloadMachine) -> crate::app::downloads::DownloadRowVisual {
        crate::app::downloads::DownloadRowVisual {
            index: 0,
            id: 1,
            filename: "dl.bin".into(),
            machine,
            group: crate::app::downloads::DownloadGroupKind::Active,
            progress_fraction: None,
            visual: crate::app::downloads::DownloadVisual::Default,
            action: crate::app::downloads::DownloadActionKind::Cancel,
            action_enabled: true,
            action_reason: None,
            meta: "0 B".into(),
            resolved_path: None,
            retry_of: None,
            attempt: 1,
        }
    }
    assert_eq!(
        observe_row(&visual(DownloadMachine::Waiting)).machine,
        DownloadMachineState::Waiting
    );
    assert_eq!(
        observe_row(&visual(DownloadMachine::Running)).machine,
        DownloadMachineState::Running
    );
    assert_eq!(
        observe_row(&visual(DownloadMachine::Succeeded)).machine,
        DownloadMachineState::Succeeded
    );
    assert_eq!(
        observe_row(&visual(DownloadMachine::Failed)).machine,
        DownloadMachineState::Failed
    );
    assert_eq!(
        observe_row(&visual(DownloadMachine::Cancelled)).machine,
        DownloadMachineState::Cancelled
    );
}
