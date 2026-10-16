//! Bounded-window rendering and deterministic ordering (task packet
//! `downloads-sidebar-panel`, DESIGN.md §11.12/§11.6).
//!
//! At most 6 rows are visible at once; beyond that the popover's render
//! window is the visible range plus the 8-row §11.6 overscan and the full
//! history is never laid out. Ordering is deterministic: Active by
//! ascending `DownloadId`, History most-recent-terminal first, so the panel
//! row index `<n>` is stable for E2E. The native wheel listener's units and
//! sign move the shared scroll state; the scrollbar and the keyboard
//! traversal consume the same state.

use bbr_testkit::protocol::{DownloadGroup, DownloadMachineState, Request};

use crate::e2e_common::*;
use crate::support::*;

/// The bounded panel window with 16 terminal history rows: exactly 6
/// visible rows, a render window of at most 6 + 8 (the §11.6 overscan), the
/// deterministic most-recent-terminal-first ordering, and wheel-scroll that
/// moves the anchor without ever laying out the full history.
#[test]
#[ignore]
fn bounded_window_scrolls_and_orders_history_deterministically() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-bounded", &page_one);
    let known_url = fixtures.url("/__fx/dl/known.bin");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("active view");
        let sessions = initial.engine_sessions;
        let tabs = initial.tabs.len();
        let spaces = initial.spaces.len();
        let profile = initial.active_profile;
        let space = initial.active_space;

        // Seed 16 deterministic terminal history rows (fast known-length
        // successes) so the panel genuinely overflows its 6-row viewport.
        let mut ids = Vec::with_capacity(16);
        for index in 0..16 {
            let destination = format!("bulk-{index:02}.bin");
            let id = start_download(&mut client, &fixtures, &known_url, &destination);
            let _ = wait_row_terminal(&mut client, id, DownloadMachineState::Succeeded);
            ids.push(id);
        }

        // Deterministic ordering: History most-recent-terminal first — row 0
        // is the newest terminal attempt, the last row the oldest.
        let seeded = client.snapshot().expect("seeded history snapshot");
        assert_eq!(seeded.download_rows.len(), 16);
        assert!(seeded
            .download_rows
            .iter()
            .all(|row| row.group == DownloadGroup::History));
        assert_eq!(
            seeded.download_rows[0].id, ids[15],
            "most-recent-terminal first"
        );
        assert_eq!(seeded.download_rows[15].id, ids[0], "oldest terminal last");

        // Open the panel: the bounded window contract — 6 visible rows, a
        // render window capped at the 6 + 8 overscan, the full history never
        // laid out.
        let opened = open_downloads(&mut client);
        let panel = opened.downloads;
        assert!(panel.open);
        assert_eq!(panel.total_rows, 16);
        assert_eq!(panel.viewport_rows, 6, "at most 6 rows visible");
        assert_eq!(panel.max_visible, 6);
        assert_eq!(panel.anchor, 0, "at the top before scrolling");
        assert!(
            panel.rendered_rows <= 14,
            "visible range + 8-row overscan; rendered {}",
            panel.rendered_rows
        );
        // The exposed row ids are exactly the bounded window — rows beyond
        // the rendered range are never laid out.
        for index in 0..panel.rendered_rows {
            assert_ids(&opened, &[&row_id(index)]);
        }
        for index in panel.rendered_rows..16 {
            assert!(
                !opened.visible_ids.iter().any(|id| id == &row_id(index)),
                "row {index} is outside the bounded window and must never be laid out"
            );
        }
        assert_ids(&opened, &["bbr-downloads-panel", "bbr-downloads-history"]);
        assert_panel_identity(&opened, view, sessions, tabs, spaces, profile, space);

        // Wheel scroll (native units and sign): a large negative delta
        // scrolls to the end; the anchor clamps to `total − window`.
        client
            .send_ok(&Request::ScrollDownloads { delta_px: -4000.0 })
            .expect("scroll down");
        flush_frame(&mut client);
        let scrolled = client.snapshot().expect("scrolled snapshot");
        assert_eq!(
            scrolled.downloads.anchor, 2,
            "anchor clamps to total − window (16 − 14)"
        );
        assert_eq!(scrolled.downloads.total_rows, 16);
        assert_eq!(scrolled.downloads.viewport_rows, 6);
        assert!(
            scrolled.downloads.rendered_rows <= 14,
            "the scrolled window stays bounded"
        );
        assert_panel_identity(&scrolled, view, sessions, tabs, spaces, profile, space);

        // Wheel scroll back up returns the anchor to the top.
        client
            .send_ok(&Request::ScrollDownloads { delta_px: 4000.0 })
            .expect("scroll up");
        flush_frame(&mut client);
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(restored.downloads.anchor, 0, "scrolled back to the top");

        // Keyboard traversal moves the cursor and scrolls it into view
        // (focused_row is the keyboard cursor; no persistent selection).
        keystroke(&mut client, "down");
        flush_frame(&mut client);
        let focus_down = client.snapshot().expect("focus down snapshot");
        assert_eq!(focus_down.downloads.focused_row, Some(1));
        keystroke(&mut client, "pagedown");
        flush_frame(&mut client);
        let focus_page = client.snapshot().expect("focus page snapshot");
        assert_eq!(
            focus_page.downloads.focused_row,
            Some(7),
            "PgDn pages one viewport"
        );
        assert_panel_identity(&focus_page, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The `bbr-download-row-<n>` id of the deterministic panel index.
fn row_id(index: usize) -> String {
    format!("bbr-download-row-{index}")
}
