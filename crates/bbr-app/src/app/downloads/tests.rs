//! Unit tests for the pure downloads panel model (DESIGN.md §11.12): the
//! deterministic row ordering (Active ascending `DownloadId`, History
//! most-recent-terminal first), the row model mapping from the L6 machine
//! states (machine/group/progress/action/meta), the byte formatting, the
//! bounded-window layout math, and the panel render-state derivation. The
//! controller is the portable authority — these tests only assert the
//! presentation mapping never invents a transition.

use bbr_browser::DownloadController;
use bbr_web::downloads::{
    DestinationDecision, DownloadEvent, DownloadFailure, DownloadId, DownloadProgress,
};

use crate::app::downloads::{format_bytes, row_meta, row_visuals};
use bbr_ui::surfaces::downloads::{
    DownloadActionKind, DownloadGroupKind, DownloadMachine, PanelLayout, DOWNLOADS_MAX_VISIBLE,
    DOWNLOADS_WINDOW_ROWS,
};

fn clock() -> impl Fn() -> u64 {
    let ticks = std::cell::Cell::new(0_u64);
    move || {
        ticks.set(ticks.get() + 1);
        ticks.get() * 1000
    }
}

/// Resolve the attempt's destination (L6 `Waiting{None}` → `Waiting{Some}`).
fn resolve(controller: &mut DownloadController, id: DownloadId) {
    let applied = controller.apply(DownloadEvent::DestinationResolved {
        id,
        destination: format!("downloads/{id:?}.bin"),
    });
    assert!(matches!(applied, bbr_browser::ApplyOutcome::Applied));
}

fn progress(
    controller: &mut DownloadController,
    id: DownloadId,
    received: u64,
    total: Option<u64>,
) {
    let applied = controller.apply(DownloadEvent::Progress {
        id,
        progress: DownloadProgress::new(received, total).expect("valid progress"),
    });
    assert!(matches!(applied, bbr_browser::ApplyOutcome::Applied));
}

fn succeed(controller: &mut DownloadController, id: DownloadId) {
    let applied = controller.apply(DownloadEvent::Succeeded { id });
    assert!(matches!(applied, bbr_browser::ApplyOutcome::Applied));
}

fn fail(controller: &mut DownloadController, id: DownloadId, failure: DownloadFailure) {
    let applied = controller.apply(DownloadEvent::Failed { id, failure });
    assert!(matches!(applied, bbr_browser::ApplyOutcome::Applied));
}

fn cancel(controller: &mut DownloadController, id: DownloadId) {
    let applied = controller.apply(DownloadEvent::Cancelled { id });
    assert!(matches!(applied, bbr_browser::ApplyOutcome::Applied));
}

/// Active rows sort by ascending `DownloadId`; History rows sort
/// most-recent-terminal first — the deterministic panel order so row index
/// `<n>` is stable for E2E.
#[test]
fn row_order_is_active_ascending_then_history_most_recent_terminal_first() {
    let mut controller = DownloadController::with_clock(clock());
    // id 0: fails first (terminal seq 1).
    let first = controller
        .start(
            "http://fixture/a.bin",
            DestinationDecision::Suggested("a.bin".into()),
        )
        .expect("first");
    fail(&mut controller, first, DownloadFailure::Network);
    // id 1: succeeds second (terminal seq 2).
    let second = controller
        .start(
            "http://fixture/b.bin",
            DestinationDecision::Suggested("b.bin".into()),
        )
        .expect("second");
    resolve(&mut controller, second);
    progress(&mut controller, second, 100, Some(100));
    succeed(&mut controller, second);
    // id 2: still active (running).
    let active = controller
        .start(
            "http://fixture/c.bin",
            DestinationDecision::Suggested("c.bin".into()),
        )
        .expect("third");
    resolve(&mut controller, active);
    progress(&mut controller, active, 5, Some(100));

    // Active: [active] ascending by id; History: [second, first]
    // most-recent-terminal first.
    assert_eq!(controller.active_ids(), vec![active]);
    assert_eq!(controller.history_ids(), vec![second, first]);

    // The combined rows mirror that order, with the L6 groups.
    let state = crate::app::downloads::DownloadsState::new();
    // Replace the controller to keep the reveal cache untouched (empty).
    let state = state;
    state.controller.set(controller);
    let rows = row_visuals(&state, None);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].id, active.0);
    assert_eq!(rows[0].group, DownloadGroupKind::Active);
    assert_eq!(rows[1].id, second.0);
    assert_eq!(rows[1].group, DownloadGroupKind::History);
    assert_eq!(rows[2].id, first.0);
    assert_eq!(rows[2].group, DownloadGroupKind::History);
}

/// The row model maps exactly the L6 machine states; the panel never
/// invents a state.
#[test]
fn row_model_maps_the_l6_machine_states_exactly() {
    let mut controller = DownloadController::with_clock(clock());
    let waiting = controller
        .start(
            "http://fixture/w.bin",
            DestinationDecision::Suggested("w.bin".into()),
        )
        .expect("waiting");
    let running = controller
        .start(
            "http://fixture/r.bin",
            DestinationDecision::Suggested("r.bin".into()),
        )
        .expect("running");
    resolve(&mut controller, running);
    progress(&mut controller, running, 2048, Some(65536));
    let succeeded = controller
        .start(
            "http://fixture/s.bin",
            DestinationDecision::Suggested("s.bin".into()),
        )
        .expect("succeeded");
    resolve(&mut controller, succeeded);
    progress(&mut controller, succeeded, 0, Some(0));
    succeed(&mut controller, succeeded);
    let failed = controller
        .start(
            "http://fixture/f.bin",
            DestinationDecision::Suggested("f.bin".into()),
        )
        .expect("failed");
    resolve(&mut controller, failed);
    fail(&mut controller, failed, DownloadFailure::Network);
    let cancelled = controller
        .start(
            "http://fixture/c.bin",
            DestinationDecision::Suggested("c.bin".into()),
        )
        .expect("cancelled");
    cancel(&mut controller, cancelled);

    let state = crate::app::downloads::DownloadsState::new();
    state.controller.set(controller);
    let rows = row_visuals(&state, None);
    // Active ascending: waiting, running. History most-recent-terminal
    // first: cancelled (seq 3), failed (seq 2), succeeded (seq 1).
    let by_id = |id: u64| rows.iter().find(|row| row.id == id).expect("row");
    assert_eq!(by_id(waiting.0).machine, DownloadMachine::Waiting);
    assert_eq!(by_id(waiting.0).group, DownloadGroupKind::Active);
    assert_eq!(by_id(waiting.0).action, DownloadActionKind::Cancel);
    assert_eq!(by_id(running.0).machine, DownloadMachine::Running);
    assert_eq!(by_id(running.0).group, DownloadGroupKind::Active);
    assert_eq!(by_id(running.0).action, DownloadActionKind::Cancel);
    assert_eq!(
        by_id(running.0).progress_fraction,
        Some(2048.0 / 65536.0),
        "known total renders the exact fraction"
    );
    assert_eq!(by_id(succeeded.0).machine, DownloadMachine::Succeeded);
    assert_eq!(by_id(succeeded.0).group, DownloadGroupKind::History);
    assert_eq!(by_id(succeeded.0).action, DownloadActionKind::Reveal);
    assert_eq!(
        by_id(succeeded.0).progress_fraction,
        Some(1.0),
        "a succeeded attempt presents the full bar (DESIGN §11.12)"
    );
    assert_eq!(by_id(failed.0).machine, DownloadMachine::Failed);
    assert_eq!(by_id(failed.0).group, DownloadGroupKind::History);
    assert_eq!(by_id(failed.0).action, DownloadActionKind::Retry);
    assert_eq!(by_id(cancelled.0).machine, DownloadMachine::Cancelled);
    assert_eq!(by_id(cancelled.0).group, DownloadGroupKind::History);
    assert_eq!(
        by_id(cancelled.0).action,
        DownloadActionKind::Restart,
        "a cancelled row offers Restart (the settled §17.8 decision)"
    );
    assert!(
        by_id(cancelled.0).delete_available,
        "a cancelled row offers the §17.8 Delete control"
    );
    assert!(
        !by_id(failed.0).delete_available,
        "Delete is cancelled-only"
    );
}

/// Reveal-in-Finder eligibility comes from the app's reveal cache (the
/// native existence query); an absent cache entry disables the action with
/// a reason, and an eligible entry enables it.
#[test]
fn reveal_action_uses_the_eligibility_cache() {
    let mut controller = DownloadController::with_clock(clock());
    let id = controller
        .start(
            "http://fixture/s.bin",
            DestinationDecision::Suggested("s.bin".into()),
        )
        .expect("succeeded");
    resolve(&mut controller, id);
    succeed(&mut controller, id);

    // No cache entry: disabled with a neutral reason.
    let state = crate::app::downloads::DownloadsState::new();
    state.controller.set(controller);
    let rows = row_visuals(&state, None);
    let row = rows.iter().find(|row| row.id == id.0).expect("row");
    assert_eq!(row.action, DownloadActionKind::Reveal);
    assert!(!row.action_enabled);
    assert_eq!(row.action_reason.as_deref(), Some("not checked"));

    // Eligible cache entry: enabled.
    state.reveal.get().insert(
        id,
        crate::app::downloads::RevealEligibility {
            eligible: true,
            reason: None,
        },
    );
    let rows = row_visuals(&state, None);
    let row = rows.iter().find(|row| row.id == id.0).expect("row");
    assert!(row.action_enabled);
    assert_eq!(row.action_reason, None);
}

/// The secondary-line copy names the machine outcome per §11.12/§11.8.
#[test]
fn secondary_line_copy_names_the_machine_outcome() {
    use bbr_web::downloads::DownloadState;
    assert_eq!(
        row_meta(&DownloadState::Waiting { destination: None }, None),
        "Waiting…"
    );
    assert_eq!(
        row_meta(
            &DownloadState::Running {
                destination: "d".into(),
                progress: DownloadProgress::new(2048, Some(65536)).unwrap(),
            },
            None
        ),
        "2.0 KB / 64.0 KB"
    );
    // Unknown length: transferred bytes only.
    assert_eq!(
        row_meta(
            &DownloadState::Running {
                destination: "d".into(),
                progress: DownloadProgress::new(2048, None).unwrap(),
            },
            None
        ),
        "2.0 KB"
    );
    assert_eq!(
        row_meta(&DownloadState::Succeeded, Some(65536)),
        "64.0 KB · Completed"
    );
    assert_eq!(
        row_meta(
            &DownloadState::Failed {
                failure: DownloadFailure::Network
            },
            Some(10)
        ),
        "10 B · Network error"
    );
    assert_eq!(
        row_meta(&DownloadState::Cancelled, Some(10)),
        "10 B · Cancelled"
    );
}

/// Deterministic byte formatting (1024 units, macOS convention).
#[test]
fn byte_formatting_is_deterministic() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(2048), "2.0 KB");
    assert_eq!(format_bytes(100 * 1024), "100 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
}

/// The bounded window: at most `DOWNLOADS_MAX_VISIBLE` visible rows, the
/// render window is the visible range plus the §11.6 8-row overscan, and
/// the full history is never laid out.
#[test]
fn bounded_window_never_lays_out_the_full_history() {
    let layout = PanelLayout {
        label_h: 20.0,
        row_h: 32.0,
        active_count: 0,
        history_count: 40,
        hairline: 1.0,
    };
    let state = crate::app::downloads::DownloadsState::new();
    // The anchor clamps so the window always fits the deterministic list.
    let anchor = 30;
    let window = layout_total_window(&layout, anchor);
    assert_eq!(
        window.len(),
        DOWNLOADS_WINDOW_ROWS,
        "window = 6 + 8 overscan"
    );
    assert!(window.end <= layout.total());
    assert_eq!(
        state.viewport_rows(layout.total()),
        DOWNLOADS_MAX_VISIBLE,
        "the viewport shows at most 6 rows"
    );
    assert_eq!(state.viewport_rows(3), 3, "fewer rows → the full content");
}

fn layout_total_window(layout: &PanelLayout, anchor: usize) -> std::ops::Range<usize> {
    let total = layout.total();
    let anchor = total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor);
    let end = (anchor + DOWNLOADS_WINDOW_ROWS).min(total);
    anchor..end
}

/// The layout math keeps the scroll content height equal to the full
/// content while the spacers fill the unrendered rows.
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
    // Everything above the window plus the window rows plus the remainder.
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
    // The group labels anchor above the rows: the first history row sits
    // below the active label, the hairline, and the history label.
    assert!(layout.row_y(layout.active_count) > 0.0);
    assert!(layout.row_y(layout.total()) == layout.content_height());
}
