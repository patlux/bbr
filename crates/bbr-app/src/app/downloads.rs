//! Downloads panel composition state (DESIGN.md §11.12).
//!
//! The app-side half of the Phase-8 downloads surface: the controller-owned
//! attempt/history store ([`bbr_browser::DownloadController`], built on the
//! L6 portable contract), the anchored-popover presentation state (open,
//! keyboard-cursor focus, scroll anchor), the deterministic row model the
//! render and the testkit snapshot share, and the pure panel layout math.
//! The native `WKDownload` adapter lives in `bbr-web-macos`; the
//! composition (`app::downloads_behavior`) wires actions and events between
//! the controller and the adapter.
//!
//! Machine fidelity (L6): the panel renders exactly the L6 states and never
//! invents a transition; the first terminal event wins; cancel is a request
//! that stays active until the engine's terminal event lands; retry exists
//! on `Failed` and restart on `Cancelled` (the settled §17.8 decision) —
//! both link a fresh attempt via `retry_of`; progress is monotonic and a
//! zero-byte download succeeds with a full bar. All of that is the
//! controller's contract — this module only maps records to rows.
//!
//! Deterministic ordering: Active by ascending `DownloadId`, History
//! most-recent-terminal first, so the panel row index `<n>` is stable for
//! E2E. Bounded rendering: at most [`DOWNLOADS_MAX_VISIBLE`] rows in the
//! viewport, a [`DOWNLOADS_WINDOW_ROWS`] render window (visible range plus
//! the §11.6 8-row overscan), and the full history is never laid out.

use std::collections::HashMap;

use bbr_browser::DownloadController;
use bbr_ui::state::DownloadVisual;
use bbr_web::downloads::{DownloadFailure, DownloadId, DownloadState};
use gpui::{ScrollHandle, Transition};

/// DESIGN.md §11.12: at most 6 rows are visible in the panel viewport at
/// once (beyond that the popover scrolls).
pub const DOWNLOADS_MAX_VISIBLE: usize = 6;
/// The bounded render window: the §11.12/§11.6 visible range (6 rows) plus
/// the 8-row overscan — the full history is never laid out.
pub const DOWNLOADS_WINDOW_ROWS: usize = DOWNLOADS_MAX_VISIBLE + 8;
/// The scroll page for `PgUp`/`PgDn` (one viewport of rows).
pub const DOWNLOADS_PAGE_ROWS: usize = DOWNLOADS_MAX_VISIBLE;

/// What the last render actually built (recorded during render so the
/// testkit snapshot reports exactly what the shell built).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DownloadsRenderState {
    /// First rendered combined row index.
    pub first_rendered: usize,
    /// How many rows the render built (bounded by [`DOWNLOADS_WINDOW_ROWS`]).
    pub rendered_rows: usize,
    /// The full deterministic row list length.
    pub item_count: usize,
    /// First row at the viewport top, derived from the production scroll
    /// offset.
    pub scroll_anchor: usize,
    /// The scroll viewport's row count (`min(6, total)`).
    pub viewport_rows: usize,
}

/// The Reveal-in-Finder eligibility of one succeeded record (DESIGN.md
/// §11.12: enabled only when the resolved path exists on disk; otherwise
/// §10.8 `disabled` with a reason). Computed through the native adapter —
/// never in a portable crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealEligibility {
    pub eligible: bool,
    pub reason: Option<String>,
}

/// The L6 machine state as the panel renders it (a mirror of
/// [`DownloadState`] for the row model — never a source of truth).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadMachine {
    Waiting,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// The panel group a row renders in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadGroupKind {
    Active,
    History,
}

/// The exactly-one contextual primary action of a row (DESIGN.md §11.12):
/// active → Cancel, failed → Retry, succeeded → Reveal-in-Finder, cancelled
/// → Restart (a fresh linked attempt; DESIGN.md §17.8 settled).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadActionKind {
    Cancel,
    Retry,
    /// The settled cancelled-row action (DESIGN.md §17.8): a fresh attempt
    /// linked via `retry_of` — the same machine path as a failed retry,
    /// with the panel label "Restart".
    Restart,
    Reveal,
}

/// One download row's render model: the deterministic panel index `<n>`,
/// the machine/group/action identity, the progress fraction, the
/// secondary-line copy, and the reveal eligibility. Derived from the
/// controller records + app presentation state; the render and the testkit
/// snapshot consume the same model, so the snapshot cannot drift from the
/// shell.
///
/// The §10 visual state vocabulary ([`bbr_ui::state::DownloadVisual`])
/// moved to `bbr-ui` in the G1a cutover; the render model references it
/// from there.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadRowVisual {
    pub index: usize,
    pub id: u64,
    pub filename: String,
    pub machine: DownloadMachine,
    pub group: DownloadGroupKind,
    /// `received / total` when the total is known; `None` while unknown
    /// (indeterminate bar). A zero-byte download reports `1.0`, and a
    /// succeeded attempt always presents the full bar (`1.0`).
    pub progress_fraction: Option<f64>,
    pub visual: DownloadVisual,
    pub action: DownloadActionKind,
    pub action_enabled: bool,
    pub action_reason: Option<String>,
    /// The secondary-line copy (`type:meta`, `tnum`).
    pub meta: String,
    pub resolved_path: Option<String>,
    pub retry_of: Option<u64>,
    pub attempt: u32,
    /// The row offers the Delete control (DESIGN.md §17.8 settled: a
    /// cancelled row offers Restart **and** Delete — the one-row durable
    /// removal). False for every other state; the panel renders the
    /// `bbr-download-delete-<n>` control only when this is true.
    pub delete_available: bool,
}

/// App-side downloads composition state. The controller is the portable
/// authority; this struct owns only presentation state (open, focus, scroll,
/// hover/press, reveal cache, last render state).
pub struct DownloadsState {
    pub controller: DownloadController,
    /// The anchored popover is open.
    pub open: bool,
    /// The keyboard-cursor row (no persistent selection — focus is the
    /// cursor; DESIGN.md §11.12).
    pub focus: usize,
    pub hovered_row: Option<usize>,
    pub pressed_row: Option<usize>,
    /// Production scroll state shared by the render, the keyboard traversal,
    /// and the native wheel listener.
    pub scroll: ScrollHandle,
    /// What the last render built (testkit snapshot evidence).
    pub render_state: DownloadsRenderState,
    /// Reveal eligibility cache keyed by attempt id (computed through the
    /// native adapter at success time and on panel open).
    pub reveal: HashMap<DownloadId, RevealEligibility>,
    /// The §11.12 open/close opacity fade (`dur:fine` only — never translate
    /// or scale). Created lazily on the first render that needs it; the goal
    /// is `1.0` while open and `0.0` on dismissal, and the render keeps the
    /// panel mounted (fading out) until the value reaches zero.
    pub opacity: Option<Transition<f32>>,
}

impl Default for DownloadsState {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadsState {
    pub fn new() -> Self {
        Self {
            controller: DownloadController::new(),
            open: false,
            focus: 0,
            hovered_row: None,
            pressed_row: None,
            scroll: ScrollHandle::new(),
            render_state: DownloadsRenderState::default(),
            reveal: HashMap::new(),
            opacity: None,
        }
    }

    /// The full deterministic row list (active ascending `DownloadId`, then
    /// history most-recent-terminal first) — `row index <n>` is stable.
    pub fn row_ids(&self) -> Vec<DownloadId> {
        self.controller.rows()
    }

    /// The scroll anchor derived from the production scroll offset (negative
    /// as the user scrolls down), clamped to the deterministic row list.
    pub fn anchor_from_offset(&self, row_h: f32) -> usize {
        let total = self.controller.len();
        if total == 0 {
            return 0;
        }
        let offset = self.scroll.offset().y;
        let row_h = row_h.max(1.0);
        let scrolled = (-f32::from(offset)).max(0.0);
        let anchor = (scrolled / row_h).floor() as usize;
        total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor)
    }

    /// The bounded render window over the combined row list.
    pub fn window_for(&self, anchor: usize) -> std::ops::Range<usize> {
        let total = self.controller.len();
        if total == 0 {
            return 0..0;
        }
        let anchor = total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor);
        let end = (anchor + DOWNLOADS_WINDOW_ROWS).min(total);
        anchor..end
    }

    /// The scroll viewport's row count: at most [`DOWNLOADS_MAX_VISIBLE`].
    pub fn viewport_rows(&self, total: usize) -> usize {
        DOWNLOADS_MAX_VISIBLE.min(total)
    }
}

/// The derive the render/snapshot consume: map every controller record to
/// its deterministic row model. `focus`, `hovered_row`, `pressed_row`, and
/// the reveal cache are app presentation state.
pub fn row_visuals(state: &DownloadsState, focus: Option<usize>) -> Vec<DownloadRowVisual> {
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
            DownloadRowVisual {
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

/// The contextual action label (DESIGN.md §11.12: succeeded rows carry the
/// conventional macOS "Show in Finder" copy; cancelled rows carry the
/// settled "Restart" label, DESIGN.md §17.8).
pub fn action_label(action: DownloadActionKind) -> &'static str {
    match action {
        DownloadActionKind::Cancel => "Cancel",
        DownloadActionKind::Retry => "Retry",
        DownloadActionKind::Restart => "Restart",
        DownloadActionKind::Reveal => "Show in Finder",
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

/// The pure panel layout math (DESIGN.md §11.12): the scroll content is the
/// group labels (fixed token-derived heights) plus the deterministic rows,
/// with a hairline between the groups. The render windows rows over the
/// combined list; the spacers keep the container's content height equal to
/// the full content so the native scrollbar and wheel behave as if every row
/// were laid out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelLayout {
    /// The fixed height of one `type:badge` group label (`badge.size` +
    /// 2 × `sp:1` padding — derived from tokens, never invented).
    pub label_h: f32,
    /// One `density:row` row height.
    pub row_h: f32,
    pub active_count: usize,
    pub history_count: usize,
    /// The 1 px hairline between the groups.
    pub hairline: f32,
}

impl PanelLayout {
    pub fn total(&self) -> usize {
        self.active_count + self.history_count
    }

    pub fn has_active(&self) -> bool {
        self.active_count > 0
    }

    pub fn has_history(&self) -> bool {
        self.history_count > 0
    }

    /// Height of the labels and the hairline above row `i` in the content.
    fn above(&self, i: usize) -> f32 {
        let mut height = 0.0;
        if self.has_active() {
            height += self.label_h;
        }
        if self.has_history() && i >= self.active_count {
            height += self.label_h;
            if self.has_active() {
                height += self.hairline;
            }
        }
        height
    }

    /// The content y offset of the top of row `i`.
    pub fn row_y(&self, i: usize) -> f32 {
        self.above(i) + i as f32 * self.row_h
    }

    /// The full content height: all labels, the hairline, and every row.
    pub fn content_height(&self) -> f32 {
        self.row_y(self.total())
    }

    /// The group label + hairline items that fall inside the rendered row
    /// window and must be laid out as flow children (not in the spacers).
    pub fn items_in_window(&self, window: &std::ops::Range<usize>) -> PanelWindowItems {
        PanelWindowItems {
            active_label: self.has_active() && window.start == 0,
            history_label: self.has_history()
                && window.start <= self.active_count
                && window.end > self.active_count,
            hairline: self.has_active()
                && self.has_history()
                && window.start <= self.active_count
                && window.end > self.active_count,
        }
    }

    /// The top spacer height: everything above the first rendered row
    /// (rows before the window plus any labels/hairline above it).
    pub fn top_spacer(&self, window: &std::ops::Range<usize>) -> f32 {
        if window.start == 0 {
            return 0.0;
        }
        self.row_y(window.start)
    }

    /// The bottom spacer height: the remaining content below the last
    /// rendered row (unrendered rows plus any labels/hairline below the
    /// window). By construction `row_y(end)` covers everything above the
    /// window's end (rows and interleaved items), so the remainder is exact.
    pub fn bottom_spacer(&self, window: &std::ops::Range<usize>) -> f32 {
        (self.content_height() - self.row_y(window.end)).max(0.0)
    }
}

/// Which label/hairline items the render lays out as flow children inside
/// the rendered row window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PanelWindowItems {
    pub active_label: bool,
    pub history_label: bool,
    pub hairline: bool,
}

#[cfg(test)]
mod tests;
