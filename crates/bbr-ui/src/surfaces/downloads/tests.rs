//! Downloads surface unit/contract tests (G2 packet
//! `component-gallery-browser-components.md`, Group 5; anchor Evidence
//! contract): id constants and derivations, the action-label vocabulary,
//! the DTO round-trips, and the bounded-window layout math. The gallery
//! E2E proves the surface renders with recorder callbacks through the
//! production component; this module pins the moved vocabulary so ids,
//! labels, and layout math can never drift.

use super::ids::{
    download_row_id, download_slot_id, DOWNLOADS_ACTIVE_ID, DOWNLOADS_CLEAR_ID, DOWNLOADS_EMPTY_ID,
    DOWNLOADS_HISTORY_ID, DOWNLOADS_PANEL_ID,
};
use super::model::{
    action_label, DownloadActionKind, DownloadGroupKind, DownloadMachine, DownloadRowModel,
    PanelLayout, DOWNLOADS_MAX_VISIBLE, DOWNLOADS_PAGE_ROWS, DOWNLOADS_WINDOW_ROWS,
};
use crate::state::DownloadVisual;

/// The §11.12/§15 id constants and the deterministic row/slot derivations.
#[test]
fn ids_are_stable_and_deterministic() {
    assert_eq!(DOWNLOADS_PANEL_ID, "bbr-downloads-panel");
    assert_eq!(DOWNLOADS_ACTIVE_ID, "bbr-downloads-active");
    assert_eq!(DOWNLOADS_HISTORY_ID, "bbr-downloads-history");
    assert_eq!(DOWNLOADS_EMPTY_ID, "bbr-downloads-empty");
    assert_eq!(DOWNLOADS_CLEAR_ID, "bbr-downloads-clear");
    assert_eq!(download_row_id(0), "bbr-download-row-0");
    assert_eq!(download_row_id(12), "bbr-download-row-12");
    assert_eq!(download_slot_id("status", 3), "bbr-download-status-3");
    assert_eq!(download_slot_id("name", 3), "bbr-download-name-3");
    assert_eq!(download_slot_id("meta", 3), "bbr-download-meta-3");
    assert_eq!(download_slot_id("progress", 3), "bbr-download-progress-3");
    assert_eq!(download_slot_id("action", 3), "bbr-download-action-3");
    assert_eq!(download_slot_id("delete", 3), "bbr-download-delete-3");
}

/// The bounded-window constants: 6 visible, 6 + 8 overscan, one page.
#[test]
fn bounded_window_constants_are_pinned() {
    assert_eq!(DOWNLOADS_MAX_VISIBLE, 6);
    assert_eq!(DOWNLOADS_WINDOW_ROWS, DOWNLOADS_MAX_VISIBLE + 8);
    assert_eq!(DOWNLOADS_PAGE_ROWS, DOWNLOADS_MAX_VISIBLE);
}

/// The §11.12 contextual action labels (the panel vocabulary).
#[test]
fn action_labels_follow_the_settled_decisions() {
    assert_eq!(action_label(DownloadActionKind::Cancel), "Cancel");
    assert_eq!(action_label(DownloadActionKind::Retry), "Retry");
    // §17.8: the cancelled row action is labelled "Restart".
    assert_eq!(action_label(DownloadActionKind::Restart), "Restart");
    // §11.12: the succeeded row carries the conventional macOS copy.
    assert_eq!(action_label(DownloadActionKind::Reveal), "Show in Finder");
}

/// One row model round-trips every presentation field (the DTO the
/// render, the snapshot, the performance probe, and the gallery share).
#[test]
fn row_model_round_trips_its_fields() {
    let row = DownloadRowModel {
        index: 4,
        id: 42,
        filename: "report.pdf".into(),
        machine: DownloadMachine::Running,
        group: DownloadGroupKind::Active,
        progress_fraction: Some(0.5),
        visual: DownloadVisual::Hover,
        action: DownloadActionKind::Cancel,
        action_enabled: true,
        action_reason: None,
        meta: "32.0 KB / 64.0 KB".into(),
        resolved_path: None,
        retry_of: None,
        attempt: 1,
        delete_available: false,
    };
    assert_eq!(row.index, 4);
    assert_eq!(row.id, 42);
    assert_eq!(row.filename, "report.pdf");
    assert_eq!(row.machine, DownloadMachine::Running);
    assert_eq!(row.group, DownloadGroupKind::Active);
    assert_eq!(row.progress_fraction, Some(0.5));
    assert_eq!(row.visual, DownloadVisual::Hover);
    assert_eq!(row.action, DownloadActionKind::Cancel);
    assert!(row.action_enabled);
    assert_eq!(row.meta, "32.0 KB / 64.0 KB");
    assert!(!row.delete_available);
    // A cancelled row carries the §17.8 delete availability.
    let cancelled = DownloadRowModel {
        delete_available: true,
        ..row.clone()
    };
    assert!(cancelled.delete_available);
}

/// The machine/group/action enums cover exactly the L6 states the panel
/// renders (never invented transitions).
#[test]
fn machine_vocabulary_is_exactly_the_l6_states() {
    use DownloadMachine::*;
    assert_eq!(format!("{:?}", Waiting), "Waiting");
    assert_eq!(format!("{:?}", Running), "Running");
    assert_eq!(format!("{:?}", Succeeded), "Succeeded");
    assert_eq!(format!("{:?}", Failed), "Failed");
    assert_eq!(format!("{:?}", Cancelled), "Cancelled");
    assert_ne!(DownloadGroupKind::Active, DownloadGroupKind::History);
}

/// The layout math keeps the scroll content height equal to the full
/// content while the spacers fill the unrendered rows (§11.6 bounded
/// window).
#[test]
fn panel_layout_spacers_sum_to_the_full_content_height() {
    let layout = PanelLayout {
        label_h: 20.0,
        row_h: 32.0,
        active_count: 2,
        history_count: 20,
        hairline: 1.0,
    };
    let window = 0..DOWNLOADS_WINDOW_ROWS;
    let top = layout.top_spacer(&window);
    let bottom = layout.bottom_spacer(&window);
    let window_rows_height = window.len() as f32 * layout.row_h;
    let items = layout.items_in_window(&window);
    let items_height = (if items.active_label {
        layout.label_h
    } else {
        0.0
    }) + (if items.history_label {
        layout.label_h
    } else {
        0.0
    }) + (if items.hairline { layout.hairline } else { 0.0 });
    assert!(
        (top + items_height + window_rows_height + bottom - layout.content_height()).abs() < 0.01,
        "spacers + window items + rows must equal the full content height"
    );
    assert!(layout.row_y(layout.active_count) > 0.0);
    assert!(layout.row_y(layout.total()) == layout.content_height());
}

/// The window never exceeds the deterministic list and the viewport is at
/// most `DOWNLOADS_MAX_VISIBLE` rows.
#[test]
fn window_clamps_to_the_deterministic_list() {
    let layout = PanelLayout {
        label_h: 20.0,
        row_h: 32.0,
        active_count: 0,
        history_count: 40,
        hairline: 1.0,
    };
    let total = layout.total();
    let anchor = 30;
    let window = {
        let anchor = total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor);
        let end = (anchor + DOWNLOADS_WINDOW_ROWS).min(total);
        anchor..end
    };
    assert_eq!(window.len(), DOWNLOADS_WINDOW_ROWS);
    assert!(window.end <= total);
    assert_eq!(DOWNLOADS_MAX_VISIBLE.min(total), DOWNLOADS_MAX_VISIBLE);
    assert_eq!(DOWNLOADS_MAX_VISIBLE.min(3), 3);
}

/// Group-label gating follows §11.4 region rules: labels and the hairline
/// render only when their group is non-empty and inside the window.
#[test]
fn items_in_window_gates_labels_and_hairline() {
    let both = PanelLayout {
        label_h: 20.0,
        row_h: 32.0,
        active_count: 2,
        history_count: 2,
        hairline: 1.0,
    };
    let items = both.items_in_window(&(0..4));
    assert!(items.active_label);
    assert!(items.history_label);
    assert!(items.hairline);
    let history_only = PanelLayout {
        label_h: 20.0,
        row_h: 32.0,
        active_count: 0,
        history_count: 2,
        hairline: 1.0,
    };
    let items = history_only.items_in_window(&(0..2));
    assert!(!items.active_label);
    assert!(items.history_label);
    assert!(!items.hairline, "no hairline without both groups");
}
