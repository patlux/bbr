//! Downloads row-model derivation (DESIGN.md §11.12): the deterministic
//! mapping from the controller records + app presentation state to the
//! production [`DownloadRowModel`] DTOs the render and the testkit snapshot
//! share — the machine/group derivation, the exactly-one contextual
//! action, the secondary-line copy, the failure copy, the disabled-action
//! reason, and the deterministic byte formatting. The composition state
//! and the render wrapper (the controller, the lifecycle, and the listener
//! bundle) live in the parent [`super`] module; this module is the pure
//! DTO building half of the G2 Group 5 cutover.

use bbr_ui::state::DownloadVisual;
use bbr_ui::surfaces::downloads::{
    DownloadActionKind, DownloadGroupKind, DownloadMachine, DownloadRowModel,
};
use bbr_web::downloads::{DownloadFailure, DownloadId, DownloadState};

use super::DownloadsState;

/// The derive the render/snapshot consume: map every controller record to
/// its deterministic row model. `focus`, `hovered_row`, `pressed_row`, and
/// the reveal cache are app presentation state.
pub fn row_visuals(state: &DownloadsState, focus: Option<usize>) -> Vec<DownloadRowModel> {
    state
        .row_ids()
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            let record = state
                .controller
                .record(id)
                .expect("row ids derive from the controller");
            let (machine, group) = match &record.state {
                DownloadState::Waiting { .. } => {
                    (DownloadMachine::Waiting, DownloadGroupKind::Active)
                }
                DownloadState::Running { .. } => {
                    (DownloadMachine::Running, DownloadGroupKind::Active)
                }
                DownloadState::Succeeded => {
                    (DownloadMachine::Succeeded, DownloadGroupKind::History)
                }
                DownloadState::Failed { .. } => {
                    (DownloadMachine::Failed, DownloadGroupKind::History)
                }
                DownloadState::Cancelled => {
                    (DownloadMachine::Cancelled, DownloadGroupKind::History)
                }
            };
            // DESIGN §11.12 machine fidelity: a succeeded attempt presents a
            // full bar (`fraction()` is never `NaN`; a zero-byte download
            // succeeds with a full bar). The L6 `Succeeded` state carries no
            // progress by design (`DownloadState::progress()` is `Some` only
            // for `Running`), so the row model synthesizes the full bar at
            // success — the controller already guaranteed the declared total
            // was fully received.
            let progress_fraction = match &record.state {
                DownloadState::Succeeded => Some(1.0),
                _ => record.state.progress().and_then(|p| p.fraction()),
            };
            let (action, action_enabled, action_reason) = row_action(state, &record.state, id);
            let meta = row_meta(&record.state, record.final_bytes);
            let visual = if !action_enabled && action == DownloadActionKind::Reveal {
                DownloadVisual::Disabled
            } else if state.pressed_row == Some(index) {
                DownloadVisual::Pressed
            } else if state.hovered_row == Some(index) {
                DownloadVisual::Hover
            } else if focus == Some(index) {
                DownloadVisual::Focused
            } else {
                DownloadVisual::Default
            };
            DownloadRowModel {
                index,
                id: id.0,
                filename: record.filename.clone(),
                machine,
                group,
                progress_fraction,
                visual,
                action,
                action_enabled,
                action_reason,
                meta,
                resolved_path: record.resolved_path.clone(),
                retry_of: record.retry_of.map(|retry| retry.0),
                attempt: record.attempt,
                delete_available: matches!(record.state, DownloadState::Cancelled),
            }
        })
        .collect()
}

/// The exactly-one contextual primary action of a row (DESIGN.md §11.12):
/// active → Cancel, failed → Retry, succeeded → Reveal-in-Finder (enabled
/// only when the resolved path exists, §10.8 `disabled` with a reason
/// otherwise), cancelled → Restart (the settled §17.8 decision: a fresh
/// attempt linked via `retry_of`, the same machine path as a failed retry).
fn row_action(
    state: &DownloadsState,
    machine: &DownloadState,
    id: DownloadId,
) -> (DownloadActionKind, bool, Option<String>) {
    match machine {
        DownloadState::Waiting { .. } | DownloadState::Running { .. } => {
            (DownloadActionKind::Cancel, true, None)
        }
        DownloadState::Failed { .. } => (DownloadActionKind::Retry, true, None),
        DownloadState::Cancelled => (DownloadActionKind::Restart, true, None),
        DownloadState::Succeeded => match state.reveal.get(&id) {
            Some(eligibility) => (
                DownloadActionKind::Reveal,
                eligibility.eligible,
                eligibility.reason.clone(),
            ),
            None => (
                DownloadActionKind::Reveal,
                // No cache entry (e.g. a non-macOS build or a row that never
                // ran the native check): the action is disabled with a
                // neutral reason until a check lands.
                false,
                Some("not checked".into()),
            ),
        },
    }
}

/// The secondary-line copy (`type:meta`, `tnum`): active — transferred/total
/// bytes (transferred only while the length is unknown); terminal — size +
/// status copy naming the machine outcome ("Completed" / the
/// [`DownloadFailure`] reason / "Cancelled"; §11.8 tone).
pub fn row_meta(machine: &DownloadState, final_bytes: Option<u64>) -> String {
    match machine {
        DownloadState::Waiting { .. } => "Waiting…".into(),
        DownloadState::Running { progress, .. } => match progress.total() {
            Some(total) => {
                format!(
                    "{} / {}",
                    format_bytes(progress.received()),
                    format_bytes(total)
                )
            }
            None => format_bytes(progress.received()),
        },
        DownloadState::Succeeded => {
            let size = final_bytes
                .map(format_bytes)
                .unwrap_or_else(|| format_bytes(0));
            format!("{size} · Completed")
        }
        DownloadState::Failed { failure } => {
            let size = final_bytes.map(format_bytes).unwrap_or_default();
            let reason = failure_copy(*failure);
            if size.is_empty() {
                reason
            } else {
                format!("{size} · {reason}")
            }
        }
        DownloadState::Cancelled => {
            let size = final_bytes.map(format_bytes).unwrap_or_default();
            if size.is_empty() {
                "Cancelled".into()
            } else {
                format!("{size} · Cancelled")
            }
        }
    }
}

/// The status copy naming a [`DownloadFailure`] (DESIGN.md §11.12/§11.8
/// tone: plain, never decorated).
pub fn failure_copy(failure: DownloadFailure) -> String {
    match failure {
        DownloadFailure::Network => "Network error".into(),
        DownloadFailure::Destination => "File error".into(),
        DownloadFailure::EngineRejected => "Rejected".into(),
        DownloadFailure::NoDestination => "No destination".into(),
        DownloadFailure::Unknown => "Failed".into(),
    }
}

/// The disabled-action reason copy (§10.8; the tooltip presentation is
/// deferred with DESIGN.md §17.2 — the reason rides the semantic tree).
#[cfg(feature = "testkit")]
pub fn disabled_reason(reason: Option<&str>) -> &'static str {
    match reason {
        Some("file missing") => "File no longer exists",
        Some("no resolved path") => "No saved file",
        _ => "Unavailable",
    }
}

/// Deterministic byte formatting (1024 units): `64 KB`, `1.2 MB`, `512 B`.
/// Values at or above 100 of a unit drop the decimal (macOS convention);
/// smaller values keep one decimal.
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    let (unit, scale) = if value >= GB {
        ("GB", GB)
    } else if value >= MB {
        ("MB", MB)
    } else if value >= KB {
        ("KB", KB)
    } else {
        return format!("{bytes} B");
    };
    let scaled = value / scale;
    if scaled >= 100.0 {
        format!("{:.0} {unit}", scaled)
    } else {
        format!("{scaled:.1} {unit}")
    }
}
