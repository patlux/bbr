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
    /// §18.5: `(download id, moment key)` of the row whose success was just
    /// committed — its check draws once with one `status.ok` ring.
    pub check: Option<(u64, u64)>,
}

/// The §11.12 row and panel metrics (Paper M-4, N density): every height
/// the render paints and the scroll math uses derives from these, so the
/// two can never drift. Text lines take the `type:meta` leading (12 pt
/// text on a 16 pt line at N); the badge label sits on its own 13 pt line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownloadsMetrics {
    /// One name/secondary text line (N 16).
    pub line: f32,
    /// The badge label line (N 13).
    pub label_line: f32,
    /// The vertical gap between the name, bar and secondary lines (4).
    pub gap: f32,
    /// The progress bar height (4).
    pub bar: f32,
    /// The progress bar width (150).
    pub bar_w: f32,
    /// The row's vertical padding (6).
    pub pad_y: f32,
    /// The panel's inner padding (6).
    pub panel_pad: f32,
    /// The visible action pill height (24); its hit area stays
    /// `density:hit-min`.
    pub action_h: f32,
}

impl DownloadsMetrics {
    pub fn for_theme(theme: &crate::theme::Theme) -> Self {
        DownloadsMetrics {
            line: f32::from(theme.type_.meta.size) + 4.0,
            label_line: f32::from(theme.type_.badge.size) + 2.0,
            gap: 4.0,
            bar: 4.0,
            bar_w: 150.0,
            pad_y: 6.0,
            panel_pad: 6.0,
            action_h: 24.0,
        }
    }

    /// The content a terminal (History) row must hold: name + secondary.
    pub fn history_content(&self) -> f32 {
        2.0 * self.pad_y + 2.0 * self.line + self.gap
    }

    /// The content an Active row must hold: name + bar + secondary.
    pub fn active_content(&self) -> f32 {
        self.history_content() + self.bar + self.gap
    }
}

/// One flow item of the panel's scroll content, in paint order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelItem {
    /// Unrendered content above or below the bounded window.
    Spacer(f32),
    ActiveLabel,
    Row(usize),
    /// The group separator between the Active and History groups.
    Hairline,
    HistoryLabel,
}

/// The pure panel layout math (DESIGN.md §11.12): the scroll content is the
/// group labels (fixed token-derived heights) plus the deterministic rows,
/// with a hairline between the groups. The render windows rows over the
/// combined list; the spacers keep the container's content height equal to
/// the full content so the native scrollbar and wheel behave as if every row
/// were laid out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelLayout {
    /// The fixed height of one `type:badge` group label (its line plus the
    /// label padding — derived from tokens, never invented).
    pub label_h: f32,
    /// One Active row (name + progress bar + secondary line).
    pub active_row_h: f32,
    /// One History row (name + secondary line).
    pub history_row_h: f32,
    pub active_count: usize,
    pub history_count: usize,
    /// The group separator item (a 1 px hairline centred in its band).
    pub hairline: f32,
}

impl PanelLayout {
    /// The token-derived layout the production app, the snapshot and the
    /// gallery all use.
    pub fn for_theme(
        theme: &crate::theme::Theme,
        active_count: usize,
        history_count: usize,
    ) -> Self {
        let metrics = DownloadsMetrics::for_theme(theme);
        PanelLayout {
            // Paper M-4: 4 pt above the badge line, 2 pt below.
            label_h: 4.0 + metrics.label_line + 2.0,
            active_row_h: metrics.active_content(),
            history_row_h: metrics.history_content(),
            active_count,
            history_count,
            // Paper M-4: a 9 pt band with the 1 px hairline centred.
            hairline: 9.0,
        }
    }

    /// The height of row `i` (Active rows first, then History rows).
    pub fn row_h(&self, i: usize) -> f32 {
        if i < self.active_count {
            self.active_row_h
        } else {
            self.history_row_h
        }
    }

    /// The summed height of the rows before row `i` (labels excluded) — the
    /// scroll offset that anchors the window at row `i`.
    pub fn row_offset(&self, i: usize) -> f32 {
        let active = i.min(self.active_count);
        let history = i.saturating_sub(self.active_count);
        active as f32 * self.active_row_h + history as f32 * self.history_row_h
    }

    /// The inverse of [`Self::row_offset`]: the row a scroll offset (in
    /// positive points scrolled) anchors on.
    pub fn anchor_at(&self, scrolled: f32) -> usize {
        let scrolled = scrolled.max(0.0);
        let active_span = self.active_count as f32 * self.active_row_h;
        if scrolled < active_span {
            (scrolled / self.active_row_h.max(1.0)).floor() as usize
        } else {
            self.active_count
                + ((scrolled - active_span) / self.history_row_h.max(1.0)).floor() as usize
        }
    }

    /// The scroll viewport height: the full content up to
    /// [`DOWNLOADS_MAX_VISIBLE`] rows, otherwise exactly six History rows
    /// tall (labels scroll with the content).
    pub fn viewport_height(&self) -> f32 {
        if self.total() > DOWNLOADS_MAX_VISIBLE {
            DOWNLOADS_MAX_VISIBLE as f32 * self.history_row_h
        } else {
            self.content_height()
        }
    }

    /// The flow items of the bounded window in paint order.
    pub fn items(&self, window: &std::ops::Range<usize>) -> Vec<PanelItem> {
        let mut items = Vec::new();
        let top = self.top_spacer(window);
        if top > 0.0 {
            items.push(PanelItem::Spacer(top));
        }
        let shown = self.items_in_window(window);
        if shown.active_label {
            items.push(PanelItem::ActiveLabel);
        }
        for index in window.clone() {
            // The History group opens with its separator and label, right
            // before its first row.
            if index == self.active_count {
                if shown.hairline {
                    items.push(PanelItem::Hairline);
                }
                if shown.history_label {
                    items.push(PanelItem::HistoryLabel);
                }
            }
            items.push(PanelItem::Row(index));
        }
        let bottom = self.bottom_spacer(window);
        if bottom > 0.0 {
            items.push(PanelItem::Spacer(bottom));
        }
        items
    }

    /// The painted height of one flow item.
    pub fn item_height(&self, item: PanelItem) -> f32 {
        match item {
            PanelItem::Spacer(height) => height,
            PanelItem::ActiveLabel | PanelItem::HistoryLabel => self.label_h,
            PanelItem::Row(index) => self.row_h(index),
            PanelItem::Hairline => self.hairline,
        }
    }
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
        self.above(i) + self.row_offset(i)
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

    /// The top spacer height: everything above the first rendered item
    /// (rows before the window plus the labels/hairline above them). When
    /// the window opens on the first History row, its separator and label
    /// render inside the window, so the spacer stops above them.
    pub fn top_spacer(&self, window: &std::ops::Range<usize>) -> f32 {
        if window.start == 0 || window.is_empty() {
            return 0.0;
        }
        let shown = self.items_in_window(window);
        let mut top = self.row_y(window.start);
        if window.start == self.active_count && shown.history_label {
            top -= self.label_h;
            if shown.hairline {
                top -= self.hairline;
            }
        }
        top
    }

    /// The bottom spacer height: the remaining content below the last
    /// rendered row (unrendered rows plus any labels/hairline below the
    /// window, including a History label that would head the next row).
    pub fn bottom_spacer(&self, window: &std::ops::Range<usize>) -> f32 {
        let Some(last) = window.end.checked_sub(1).filter(|_| !window.is_empty()) else {
            return self.content_height();
        };
        (self.content_height() - (self.row_y(last) + self.row_h(last))).max(0.0)
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
