//! Downloads panel presentation model (DESIGN.md §11.12, G2 packet
//! `component-gallery-browser-components.md`, Group 5) — the plain `bbr-ui`
//! DTO vocabulary the production surface, the app's DTO building, the
//! testkit snapshot, and the gallery stories all share.
//!
//! The controller stays in `bbr-app` (the portable `DownloadController`
//! and the L6 events); this module owns only the presentation mirror of
//! the machine states, the deterministic row model, the bounded-window
//! layout math, and the contextual action vocabulary. The surface never
//! invents a transition — it renders exactly the L6 states the controller
//! reports (`Waiting`, `Running`, `Succeeded`, `Failed`, `Cancelled`).

use gpui::ScrollHandle;

/// DESIGN.md §11.12: at most 6 rows are visible in the panel viewport at
/// once (beyond that the popover scrolls).
pub const DOWNLOADS_MAX_VISIBLE: usize = 6;
/// The bounded render window: the §11.12/§11.6 visible range (6 rows) plus
/// the 8-row overscan — the full history is never laid out.
pub const DOWNLOADS_WINDOW_ROWS: usize = DOWNLOADS_MAX_VISIBLE + 8;
/// The scroll page for `PgUp`/`PgDn` (one viewport of rows).
pub const DOWNLOADS_PAGE_ROWS: usize = DOWNLOADS_MAX_VISIBLE;

/// The L6 machine state as the panel renders it (a mirror of the portable
/// `DownloadState` for the row model — never a source of truth).
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
/// secondary-line copy, and the reveal eligibility. `bbr-app` derives the
/// model from its controller records + presentation state; the render, the
/// testkit snapshot, the performance probe, and the gallery stories all
/// consume the same DTO, so they can never drift.
///
/// The §10 visual state vocabulary is the `bbr-ui` state
/// ([`crate::state::DownloadVisual`]).
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadRowModel {
    pub index: usize,
    pub id: u64,
    pub filename: String,
    pub machine: DownloadMachine,
    pub group: DownloadGroupKind,
    /// `received / total` when the total is known; `None` while unknown
    /// (indeterminate bar). A zero-byte download reports `1.0`, and a
    /// succeeded attempt always presents the full bar (`1.0`).
    pub progress_fraction: Option<f64>,
    pub visual: crate::state::DownloadVisual,
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

/// The §11.12 anchored-popover presentation model the surface renders.
/// `bbr-app` builds every field from its controller, its presentation
/// state (open/focus/scroll), and its sidebar geometry; the gallery builds
/// plain fixtures. The surface owns the anatomy only.
pub struct DownloadsPanelModel<'a> {
    /// The popover is open (drives the mount/unmount alongside `alpha`).
    pub open: bool,
    /// §9 ring while the panel owns chrome focus.
    pub focused: bool,
    /// The evaluated `dur:fine` opacity fade (the app owns the transition
    /// lifecycle; the surface fades out and unmounts at zero).
    pub alpha: f32,
    /// The panel width (sidebar width − 2 × `density:row-inset`, clamped;
    /// app geometry).
    pub width: f32,
    /// The deterministic full row list (active ascending `DownloadId`,
    /// then history most-recent-terminal first) — `row index <n>` is
    /// stable.
    pub rows: Vec<DownloadRowModel>,
    /// The bounded render window over the combined row list (the visible
    /// range plus the §11.6 8-row overscan — the full history is never
    /// laid out).
    pub window: std::ops::Range<usize>,
    /// The §11.12 layout math (group labels + rows + hairline).
    pub layout: PanelLayout,
    /// The production scroll handle the scroll viewport tracks (the app
    /// derives the window from exactly this handle, so the typed scroll
    /// path and the wheel path never disagree).
    pub scroll: &'a ScrollHandle,
    /// The panel's chrome focus handle (the surface tracks focus on it;
    /// the app owns the handle).
    pub focus: &'a gpui::FocusHandle,
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
