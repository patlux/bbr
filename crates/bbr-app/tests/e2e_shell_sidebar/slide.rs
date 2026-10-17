//! Real-app §11.4 sidebar slide E2E (DESIGN §6 principle 1a, §11.4;
//! Patrick 2026-10-17).
//!
//! Over a real WKWebView, Cmd-S commits the toggle on the commit frame
//! (`sidebar_visible`, the committed width, the semantic ids) and then
//! presents it as a slide: the rail edge, the content card and the native
//! slot travel together for `dur:emphasis` and settle exactly on the
//! committed geometry. The suite proves from the typed slide evidence:
//!
//! 1. commit first, then a monotonic slide with intermediate rail edges
//!    *and* intermediate native WKWebView frames (the page travels with the
//!    rail instead of jumping at the end), settling after the full
//!    `dur:emphasis` on the exact collapsed geometry;
//! 2. a leaving rail never floats from a pointer at the window edge;
//! 3. a toggle mid-slide reverses from the presented share without a jump
//!    and settles on the expanded geometry;
//! 4. a settled shell requests no further frames (idle proof);
//! 5. Reduce Motion presents the committed geometry on the first frame.
//!
//! The harness window is hidden and has no display link: every frame is a
//! real production draw forced at the current time (the slide is
//! time-based). No sleeps, no pixels.

use super::*;
use bbr_testkit::protocol::{SidebarMotionFrame, Snapshot};

/// The `dur:emphasis` slide (DESIGN §6 tokens).
const SLIDE_MS: f32 = 260.0;

fn flushed(client: &mut Client) -> Snapshot {
    flush_frame(client);
    client.snapshot().expect("snapshot")
}

fn pointer(client: &mut Client, x: f32, time_ms: f64) {
    client
        .send_ok(&bbr_testkit::Request::RailPointer {
            x,
            y: 400.0,
            time_ms,
            pressed: false,
        })
        .expect("rail pointer");
}

/// The frames strictly between the settled ends.
fn travelling(frames: &[SidebarMotionFrame]) -> Vec<SidebarMotionFrame> {
    frames
        .iter()
        .copied()
        .filter(|frame| frame.presence > 0.0 && frame.presence < 1.0)
        .collect()
}

#[test]
#[ignore]
fn cmd_s_commits_first_then_slides_the_rail_card_and_native_view_together() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-slide", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");

        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("first native view");
        let sessions = initial.engine_sessions;
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        let expanded_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT - CONTENT_INSET,
            deadline(),
        );
        let height = expanded_rect.content_rect[3];
        let band = expanded_rect.titlebar.safe_area_height;
        assert_eq!(expanded_rect.sidebar_motion.toggles, 0);
        assert_eq!(expanded_rect.sidebar_motion.variant, None);
        assert_eq!(expanded_rect.sidebar_motion.presence, 1.0);
        let frames_before = expanded_rect.motion.transition_frames;

        // 1. Commit first: the very next snapshot already reports the
        // committed state while the presentation still shows the rail.
        keystroke(&mut client, "cmd-s");
        let committed = client.snapshot().expect("committed snapshot");
        assert!(!committed.sidebar_visible, "the toggle commits at once");
        assert_eq!(committed.sidebar_width, 0.0);
        assert!(committed.titlebar.sidebar_collapsed);
        assert!(
            !committed.visible_ids.iter().any(|id| id == "bbr-sidebar"),
            "the committed (semantic) rail is hidden at once"
        );
        let motion = &committed.sidebar_motion;
        assert_eq!(motion.variant.as_deref(), Some("animated"));
        assert_eq!(motion.toggles, 1);
        assert_eq!(
            motion.start_presence,
            Some(1.0),
            "a full slide starts at the rail"
        );
        assert_eq!(motion.duration_ms, Some(SLIDE_MS), "dur:emphasis");
        assert!(
            motion.animating && motion.presence > 0.0,
            "the presentation still travels right after the commit: {motion:?}"
        );
        assert_view_and_sessions_preserved(&committed, view, sessions);

        // 2. A leaving rail never floats: a fast arrival at the edge while
        // it slides out keeps the reveal idle.
        pointer(&mut client, 500.0, 0.0);
        pointer(&mut client, 4.0, 40.0);
        let edge = client.snapshot().expect("edge while sliding");
        if edge.sidebar_motion.animating {
            assert_eq!(edge.rail_reveal, "idle", "a leaving rail never floats");
            assert!(edge.floating_rail.is_none());
        }

        let collapsed = wait_sidebar_settled(&mut client);
        let motion = &collapsed.sidebar_motion;
        let settle = motion.last_settle_ms.expect("settled");
        assert!(
            settle >= SLIDE_MS - 1.0,
            "a full slide travels dur:emphasis, settled after {settle} ms"
        );
        assert_eq!(motion.presence, 0.0);
        assert_eq!(motion.presented_width, 0.0);
        // The trace: monotonic, through intermediate rail edges, ending on
        // the committed share.
        let frames = &motion.frames;
        eprintln!("slide-out settled after {settle:.1} ms; frames {frames:?}");
        assert!(
            frames.len() >= 3,
            "a slide renders several frames: {frames:?}"
        );
        for pair in frames.windows(2) {
            assert!(pair[1].at_ms >= pair[0].at_ms);
            assert!(
                pair[1].presence <= pair[0].presence,
                "the rail only travels out: {frames:?}"
            );
            assert!(pair[1].rail_width <= pair[0].rail_width);
        }
        let last = frames.last().expect("last frame");
        assert_eq!((last.presence, last.rail_width), (0.0, 0.0));
        let mid = travelling(frames);
        assert!(
            !mid.is_empty(),
            "intermediate frames presented the rail mid-way: {frames:?}"
        );
        // Whole points: a share within half a point of an end rounds onto
        // it, every other travelling frame sits strictly between.
        assert!(mid.iter().all(|frame| {
            (0.0..=SIDEBAR_DEFAULT).contains(&frame.rail_width)
                && frame.rail_width == frame.rail_width.round()
        }));
        assert!(
            mid.iter()
                .any(|frame| frame.rail_width > 0.0 && frame.rail_width < SIDEBAR_DEFAULT),
            "the rail edge stood mid-way: {frames:?}"
        );
        // The WKWebView travelled with the rail edge: the engine held
        // native frames strictly between the expanded and collapsed slots.
        assert!(
            frames
                .iter()
                .any(|frame| frame.native_x > CONTENT_INSET && frame.native_x < SIDEBAR_DEFAULT),
            "the native slot moved through intermediate frames: {frames:?}"
        );
        // Settled exactly on the committed collapsed geometry.
        assert_eq!(
            collapsed.content_rect,
            collapsed_rect_for(window_w, height, band)
        );
        assert_eq!(collapsed.content_card.inset_leading, CONTENT_INSET);
        assert_eq!(collapsed.content_card.inset_top, band.max(CONTENT_INSET));
        assert_view_and_sessions_preserved(&collapsed, view, sessions);
        let slide_frames = collapsed.motion.transition_frames;
        assert!(
            slide_frames > frames_before,
            "the slide requested animation frames while it travelled"
        );

        // 4. Idle: a settled shell requests no further frames.
        let idle = flushed(&mut client);
        assert!(idle.motion.root_renders > collapsed.motion.root_renders);
        assert_eq!(idle.motion.transition_frames, slide_frames);
        assert!(!idle.sidebar_motion.animating);

        // 3. Reversal: expand, then collapse again while the rail is still
        // sliding in. The second slide starts at the share on screen at the
        // toggle (no jump back to the expanded rail) and covers only that
        // distance in the matching share of `dur:emphasis`.
        keystroke(&mut client, "cmd-s");
        let entering = flushed(&mut client);
        assert!(entering.sidebar_visible);
        let presented = entering.sidebar_motion.presence;
        assert!(
            presented > 0.0 && presented < 1.0,
            "the rail is mid-way in: {:?}",
            entering.sidebar_motion
        );
        keystroke(&mut client, "cmd-s");
        let turned = client.snapshot().expect("reversed snapshot");
        let start = turned
            .sidebar_motion
            .start_presence
            .expect("the reversal slides");
        assert!(
            start >= presented - 1e-4 && start < 1.0,
            "the reversal starts on screen ({presented} ≤ {start} < 1): {:?}",
            turned.sidebar_motion
        );
        let duration = turned.sidebar_motion.duration_ms.expect("duration");
        assert!(
            (duration - SLIDE_MS * start).abs() < 1.0,
            "only the remaining distance travels: {duration} ms for {start}"
        );
        let reversed = wait_sidebar_settled(&mut client);
        assert!(!reversed.sidebar_visible);
        assert_eq!(reversed.sidebar_motion.toggles, 3);
        let frames = &reversed.sidebar_motion.frames;
        let first = frames.first().expect("reversal frames");
        assert!(
            first.presence <= start + 1e-4,
            "the reversal never jumps back towards the rail: {frames:?}"
        );
        for pair in frames.windows(2) {
            assert!(pair[1].presence <= pair[0].presence, "{frames:?}");
        }
        assert_eq!(
            reversed.content_rect,
            collapsed_rect_for(window_w, height, band)
        );

        // Back to expanded: the slide in ends on the default rail.
        keystroke(&mut client, "cmd-s");
        let expanded = wait_sidebar_settled(&mut client);
        assert!(expanded.sidebar_visible);
        assert_eq!(expanded.sidebar_motion.presented_width, SIDEBAR_DEFAULT);
        let frames = &expanded.sidebar_motion.frames;
        for pair in frames.windows(2) {
            assert!(pair[1].presence >= pair[0].presence, "{frames:?}");
        }
        assert!(!travelling(frames).is_empty(), "{frames:?}");
        assert_eq!(
            expanded.content_rect,
            content_rect_for(SIDEBAR_DEFAULT, window_w, height)
        );
        assert_view_and_sessions_preserved(&expanded, view, sessions);

        // 5. Reduce Motion: the committed geometry on the first frame.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        let before = flushed(&mut client).motion.transition_frames;
        keystroke(&mut client, "cmd-s");
        let snapped = flushed(&mut client);
        let motion = &snapped.sidebar_motion;
        assert_eq!(motion.variant.as_deref(), Some("static"));
        assert!(!motion.animating);
        assert_eq!((motion.start_presence, motion.duration_ms), (None, None));
        assert!(
            motion.last_settle_ms.is_some(),
            "settled on the first frame"
        );
        assert_eq!(motion.frames.len(), 1, "{:?}", motion.frames);
        assert_eq!(
            snapped.content_rect,
            collapsed_rect_for(window_w, height, band)
        );
        assert_eq!(snapped.motion.transition_frames, before);
        keystroke(&mut client, "cmd-s");
        let back = flushed(&mut client);
        assert_eq!(back.sidebar_motion.variant.as_deref(), Some("static"));
        assert_eq!(
            back.content_rect,
            content_rect_for(SIDEBAR_DEFAULT, window_w, height)
        );
        assert_view_and_sessions_preserved(&back, view, sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
