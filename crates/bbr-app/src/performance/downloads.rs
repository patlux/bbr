//! Downloads-panel presentation identity for the presentation probe (task
//! packet `downloads-sidebar-panel`, DESIGN.md §11.12/§15).
//!
//! The `downloads_panel_open` / `download_progress_commit` /
//! `download_row_retry` benchmarks measure the production downloads paths
//! and commit only from a *render* of the anchored downloads panel that
//! carried the exact armed presentation: the panel open with the keyboard
//! cursor (`DownloadsPanelOpen`), the running row drawn at the armed
//! received/total fraction (`DownloadsProgressCommit`), or the fresh retry
//! attempt presented in the Active group linked via `retry_of`
//! (`DownloadsRowRetry`). The observation is render-gated — the sidebar
//! render records the panel state it actually drew into the probe, so a
//! model mutation alone can never emit a measurement and a frame that
//! omitted the panel can never commit one. A mismatched or stale
//! observation rejects the sample in the app (fast-but-wrong guard).
//!
//! Fixture/network time is excluded by construction: the progress and retry
//! samples apply deterministic L6 events in-process through the same
//! main-thread pump the native adapter's sink uses — never through a
//! network transfer. The real WKDownload lifecycle is proven by the
//! functional downloads E2E and the adapter contract tests; this module
//! gates the *presentation commit* of the panel under the warm budget.
//!
//! This child module keeps the probe's downloads-specific surface out of
//! `performance.rs` (which stays under the ~500-line budget); it reaches
//! the probe's private `State`/`Pending` through parent-module privacy,
//! exactly like `super::find`.

use bbr_testkit::protocol::{DownloadGroup, DownloadMachineState, DownloadRowAction};

use super::{Pending, PerformanceProbe, State};

/// The exact downloads-panel presentation a pending sample must commit from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DownloadsExpectation {
    /// The panel is open with the keyboard cursor (chrome focus). No row
    /// identity is required — opening a fresh or populated panel both
    /// present the open/focused surface.
    Open,
    /// The active row `id` was drawn as a `Running` attempt at exactly
    /// `received`/`total` (a deterministic L6 progress event applied
    /// through the production pump).
    Progress { id: u64, received: u64, total: u64 },
    /// The fresh retry attempt `fresh_id` (linked to the failed `retry_of`
    /// attempt) was drawn in the Active group as `Waiting` with the Cancel
    /// action — the user-visible retry row action.
    Retry { retry_of: u64, fresh_id: u64 },
    /// The fresh restart attempt `fresh_id` (linked to the cancelled
    /// `retry_of` attempt — the settled §17.8 row action) was drawn in the
    /// Active group as `Waiting` with the Cancel action.
    Restart { retry_of: u64, fresh_id: u64 },
    /// The cancelled row `id` was no longer drawn after the §17.8 row
    /// Delete action (the panel render presents the remaining rows).
    Deleted { id: u64 },
    /// The §17.7 Clear action emptied the history: the panel render draws
    /// the empty state (open, focused, zero rows).
    Clear,
}

/// The downloads-panel presentation the sidebar render actually drew for
/// the anchored popover (the render milestone; the panel surface records it
/// after building the rows). No observation is recorded for a frame that
/// omitted the panel, so a pending sample can never commit from a render
/// that did not present it.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct DownloadsObservation {
    pub open: bool,
    pub focused: bool,
    /// The bounded window of rows the render actually built (never the full
    /// history).
    pub rows: Vec<RowObservation>,
}

/// One rendered row's presentation identity (the fields the benchmarks arm).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RowObservation {
    pub id: u64,
    pub machine: DownloadMachineState,
    pub group: DownloadGroup,
    /// The exact fraction the render drew (a zero-byte total reports `1.0`;
    /// `None` while the length is unknown). The arm and the render compute
    /// the identical `received / total` division, so bit-exact equality is
    /// safe and a stale or wrong fraction rejects in the app.
    pub progress_fraction: Option<f64>,
    pub action: DownloadRowAction,
    pub retry_of: Option<u64>,
}

impl PerformanceProbe {
    /// Arm the exact panel-open presentation the pending sample must commit
    /// from (the panel open with the keyboard cursor). Arming clears any
    /// prior observation: a render that drew the closed panel (or an older
    /// open state) can never satisfy the newer expectation.
    pub fn expect_downloads_open(&self) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Open);
            state.latest_downloads = None;
        }
    }

    /// Arm the exact progress presentation the pending sample must commit
    /// from: the active row `id` drawn as `Running` at exactly
    /// `received`/`total` (fast-but-wrong guard — a row at any other
    /// fraction, a rejected event, or a terminal row rejects in the app).
    pub fn expect_downloads_progress(&self, id: u64, received: u64, total: u64) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Progress {
                id,
                received,
                total,
            });
            state.latest_downloads = None;
        }
    }

    /// Arm the exact retry presentation the pending sample must commit from:
    /// the fresh attempt `fresh_id` drawn in the Active group as `Waiting`,
    /// linked to the failed `retry_of` attempt, with the Cancel action.
    pub fn expect_downloads_retry(&self, retry_of: u64, fresh_id: u64) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Retry { retry_of, fresh_id });
            state.latest_downloads = None;
        }
    }

    /// Arm the exact restart presentation (the settled §17.8 cancelled-row
    /// action): the fresh attempt `fresh_id` drawn in the Active group as
    /// `Waiting`, linked to the cancelled `retry_of` attempt, with the
    /// Cancel action — the same machine path as a failed retry.
    pub fn expect_downloads_restart(&self, retry_of: u64, fresh_id: u64) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Restart { retry_of, fresh_id });
            state.latest_downloads = None;
        }
    }

    /// Arm the exact row-Delete presentation (the settled §17.8 cancelled-
    /// row action): the row `id` is gone from the rendered panel — the
    /// durable one-row removal must present immediately.
    pub fn expect_downloads_deleted(&self, id: u64) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Deleted { id });
            state.latest_downloads = None;
        }
    }

    /// Arm the exact Clear presentation (the settled §17.7 panel action):
    /// the panel render draws the empty state — open, focused, zero rows.
    pub fn expect_downloads_clear(&self) {
        let mut state = self.inner.borrow_mut();
        if let Some(pending) = state.pending.as_mut() {
            pending.downloads = Some(DownloadsExpectation::Clear);
            state.latest_downloads = None;
        }
    }

    /// Record the panel presentation the sidebar render actually drew
    /// (`open`/`focused` plus the bounded row window). Called by the panel
    /// surface after building its rows; a frame that omitted the panel
    /// records nothing, so a pending sample can never commit from a render
    /// that did not present it.
    pub fn observe_downloads_state(&self, open: bool, focused: bool, rows: Vec<RowObservation>) {
        let mut state = self.inner.borrow_mut();
        state.latest_downloads = Some(DownloadsObservation {
            open,
            focused,
            rows,
        });
    }
}

/// Build a `RowObservation` for one rendered row from the app row model
/// (mirrors the snapshot mapping; the panel surface calls this for the
/// bounded window it actually built, so the probe never sees the full
/// history).
pub(crate) fn observe_row(visual: &crate::app::downloads::DownloadRowVisual) -> RowObservation {
    RowObservation {
        id: visual.id,
        machine: match visual.machine {
            crate::app::downloads::DownloadMachine::Waiting => DownloadMachineState::Waiting,
            crate::app::downloads::DownloadMachine::Running => DownloadMachineState::Running,
            crate::app::downloads::DownloadMachine::Succeeded => DownloadMachineState::Succeeded,
            crate::app::downloads::DownloadMachine::Failed => DownloadMachineState::Failed,
            crate::app::downloads::DownloadMachine::Cancelled => DownloadMachineState::Cancelled,
        },
        group: match visual.group {
            crate::app::downloads::DownloadGroupKind::Active => DownloadGroup::Active,
            crate::app::downloads::DownloadGroupKind::History => DownloadGroup::History,
        },
        // The exact fraction the render drew (`received / total`); a
        // zero-byte total reports `1.0`, `None` while unknown.
        progress_fraction: visual.progress_fraction,
        action: match visual.action {
            crate::app::downloads::DownloadActionKind::Cancel => DownloadRowAction::Cancel,
            crate::app::downloads::DownloadActionKind::Retry => DownloadRowAction::Retry,
            crate::app::downloads::DownloadActionKind::Restart => DownloadRowAction::Restart,
            crate::app::downloads::DownloadActionKind::Reveal => DownloadRowAction::Reveal,
        },
        retry_of: visual.retry_of,
    }
}

/// The downloads-presentation part of the presentation predicate: true when
/// no downloads expectation is armed, or when the observed rendered panel
/// carries exactly the armed identity. An armed expectation with no render
/// observation yet can never commit.
pub(super) fn downloads_matches(pending: &Pending, state: &State) -> bool {
    let Some(expected) = pending.downloads.as_ref() else {
        return true;
    };
    let observed = state.latest_downloads.as_ref();
    match expected {
        DownloadsExpectation::Open => {
            observed.is_some_and(|observed| observed.open && observed.focused)
        }
        DownloadsExpectation::Progress {
            id,
            received,
            total,
        } => {
            let Some(observed) = observed else {
                return false;
            };
            observed.open
                && observed.focused
                && observed.rows.iter().any(|row| {
                    row.id == *id
                        && row.machine == DownloadMachineState::Running
                        && row.group == DownloadGroup::Active
                        && row.progress_fraction == Some(*received as f64 / *total as f64)
                })
        }
        DownloadsExpectation::Retry { retry_of, fresh_id } => {
            let Some(observed) = observed else {
                return false;
            };
            observed.open
                && observed.focused
                && observed.rows.iter().any(|row| {
                    row.id == *fresh_id
                        && row.machine == DownloadMachineState::Waiting
                        && row.group == DownloadGroup::Active
                        && row.retry_of == Some(*retry_of)
                        && row.action == DownloadRowAction::Cancel
                })
        }
        DownloadsExpectation::Restart { retry_of, fresh_id } => {
            let Some(observed) = observed else {
                return false;
            };
            observed.open
                && observed.focused
                && observed.rows.iter().any(|row| {
                    row.id == *fresh_id
                        && row.machine == DownloadMachineState::Waiting
                        && row.group == DownloadGroup::Active
                        && row.retry_of == Some(*retry_of)
                        && row.action == DownloadRowAction::Cancel
                })
        }
        DownloadsExpectation::Deleted { id } => {
            let Some(observed) = observed else {
                return false;
            };
            observed.open && observed.focused && !observed.rows.iter().any(|row| row.id == *id)
        }
        DownloadsExpectation::Clear => observed
            .is_some_and(|observed| observed.open && observed.focused && observed.rows.is_empty()),
    }
}
