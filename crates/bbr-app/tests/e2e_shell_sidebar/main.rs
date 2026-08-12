//! Real-app adjustable/collapsible sidebar E2E (Phase-4 sidebar slice).
//!
//! Covers DESIGN.md §5/§11.4 over real WKWebViews: pointer drag on the
//! visible resize divider through the production listener path (the testkit
//! seam drives the exact app methods the element handlers call, never a
//! controller mutation), keyboard resize/reset parity on the focused
//! divider, Cmd-B collapse/expand with last-expanded-width restoration,
//! compact-window clamping that keeps the native content floor (never
//! zero/negative geometry), native content-rect correctness from the
//! engine-received slot geometry, selected WebViewId/session preservation,
//! and the chrome focus contract. No sleeps/pixels: every step synchronizes
//! on snapshot state or a paint-driven content rect under a deadline. The
//! shared harness lives in `e2e_common`.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;
#[path = "../e2e_semantics_common/mod.rs"]
mod e2e_semantics_common;

use bbr_testkit::protocol::dsl;
use e2e_common::*;
use support::*;

/// Pointer drag on the visible divider through the production listener path:
/// relative deltas resize within the DESIGN §5 range, the native slot keeps
/// strictly positive geometry at exactly the rail width, the chrome focus
/// domain is preserved, and the selected WebViewId/session never changes.
#[test]
#[ignore]
fn pointer_drag_resizes_with_stable_native_content_and_focus() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-drag", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        let sessions = initial.engine_sessions;
        // AppKit clamps the spawned window frame shortly after orderFront
        // (GPUI first reports the requested bounds, then the clamped frame
        // lands via windowDidResize), and the clamped width differs per run
        // and per screen on this host — capture geometry only after the
        // frame settles so every downstream expectation is stable.
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        assert!(window_w > 400.0, "spawned window is comfortably wide");
        // Typed geometry contract: visible at the §8 default with typed
        // min/default/max and the stable divider identity (§15).
        assert!(settled.sidebar_visible);
        assert_eq!(settled.sidebar_width, SIDEBAR_DEFAULT);
        assert_eq!(settled.sidebar_min, SIDEBAR_MIN);
        assert_eq!(settled.sidebar_default, SIDEBAR_DEFAULT);
        assert_eq!(settled.sidebar_max, SIDEBAR_MAX);
        assert_eq!(settled.divider_label, "Sidebar divider");
        assert!(!settled.divider_dragging);
        assert_ids(&settled, &["bbr-sidebar", "bbr-sidebar-divider"]);

        // The native slot reports the real WKWebView frame: rail 300, content
        // filling the remainder — never zero or negative. The slot's own
        // height is the drift baseline for every later geometry.
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT,
            deadline(),
        );
        let height = initial_rect.content_rect[3];
        assert!(height > 0.0, "native content height must be positive");
        assert_eq!(initial_rect.content_rect[1], 0.0);

        // Pointer drag +40 pt: the rail grows by the exact delta and the
        // native content follows on the next frame.
        client
            .send_ok(&dsl::drag_sidebar_divider(40.0))
            .expect("drag +40");
        flush_frame(&mut client);
        let grown = client.snapshot().expect("grown snapshot");
        assert_eq!(grown.sidebar_width, SIDEBAR_DEFAULT + 40.0);
        assert!(
            grown.chrome_focused,
            "divider pointer entry keeps chrome focus"
        );
        assert_view_and_sessions_preserved(&grown, first_view, sessions);
        let grown_rect = wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, window_w, height),
            deadline(),
        );
        assert_eq!(
            grown_rect.content_rect[2],
            window_w - (SIDEBAR_DEFAULT + 40.0)
        );
        assert_eq!(grown_rect.content_rect[3], height, "no vertical drift");

        // Dragging far past the range clamps to the DESIGN §5 min…
        client
            .send_ok(&dsl::drag_sidebar_divider(-100_000.0))
            .expect("drag far left");
        flush_frame(&mut client);
        let min_snap = client.snapshot().expect("min snapshot");
        assert_eq!(min_snap.sidebar_width, SIDEBAR_MIN);
        assert_view_and_sessions_preserved(&min_snap, first_view, sessions);
        let min_rect = wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_MIN, window_w, height),
            deadline(),
        );
        assert_eq!(min_rect.content_rect[0], SIDEBAR_MIN);
        assert_eq!(min_rect.content_rect[2], window_w - SIDEBAR_MIN);

        // …and to the max.
        client
            .send_ok(&dsl::drag_sidebar_divider(100_000.0))
            .expect("drag far right");
        flush_frame(&mut client);
        let max_snap = client.snapshot().expect("max snapshot");
        assert_eq!(max_snap.sidebar_width, SIDEBAR_MAX);
        assert_view_and_sessions_preserved(&max_snap, first_view, sessions);
        let max_rect = wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_MAX, window_w, height),
            deadline(),
        );
        assert_eq!(max_rect.content_rect[0], SIDEBAR_MAX);

        // Completed drags leave the divider idle (the §15 drag state is
        // typed in the snapshot for the future Accessibility slice).
        let idle = client.snapshot().expect("idle snapshot");
        assert!(!idle.divider_dragging);
        assert!(idle
            .visible_ids
            .iter()
            .any(|id| id == "bbr-sidebar-divider"));
        assert_view_and_sessions_preserved(&idle, first_view, sessions);

        // The pointer entry into the divider focuses browser chrome; the
        // keyboard path below relies on the same handle.
        client
            .send_ok(&dsl::focus_sidebar_divider())
            .expect("focus divider");
        assert!(client.snapshot().expect("focused").chrome_focused);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard resize/reset of the focused divider through real keystrokes:
/// `←`/`→` move by the 16 pt step and `0` restores the §8 default, all on
/// the same requested width the pointer path mutates (parity by
/// construction), with the native content following on every step.
#[test]
#[ignore]
fn keyboard_resize_and_reset_match_pointer_parity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-keys", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        let sessions = initial.engine_sessions;
        // Geometry captured only after the AppKit frame clamp settles (see
        // the pointer-drag test for the same host quirk).
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT,
            deadline(),
        );
        let height = initial_rect.content_rect[3];

        // Pointer sets 320 (320 = default + 40); the keyboard path must
        // continue from exactly that value.
        client
            .send_ok(&dsl::drag_sidebar_divider(40.0))
            .expect("drag to 320");
        flush_frame(&mut client);
        let at_320 = client.snapshot().expect("320 snapshot");
        assert_eq!(at_320.sidebar_width, SIDEBAR_DEFAULT + 40.0);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, window_w, height),
            deadline(),
        );

        client
            .send_ok(&dsl::focus_sidebar_divider())
            .expect("focus divider");
        let focused = client.snapshot().expect("divider focused");
        assert!(focused.chrome_focused, "divider focus is the chrome domain");

        keystroke(&mut client, "left");
        flush_frame(&mut client);
        let left = client.snapshot().expect("left snapshot");
        assert_eq!(
            left.sidebar_width,
            SIDEBAR_DEFAULT + 40.0 - RESIZE_STEP,
            "pointer→keyboard parity"
        );
        assert_view_and_sessions_preserved(&left, first_view, sessions);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0 - RESIZE_STEP, window_w, height),
            deadline(),
        );

        keystroke(&mut client, "left");
        let left2 = client.snapshot().expect("second left snapshot");
        assert_eq!(
            left2.sidebar_width,
            SIDEBAR_DEFAULT + 40.0 - 2.0 * RESIZE_STEP
        );

        keystroke(&mut client, "right");
        let right = client.snapshot().expect("right snapshot");
        assert_eq!(right.sidebar_width, SIDEBAR_DEFAULT + 40.0 - RESIZE_STEP);

        // Reset restores the §8 default regardless of the current width.
        keystroke(&mut client, "0");
        flush_frame(&mut client);
        let reset = client.snapshot().expect("reset snapshot");
        assert_eq!(reset.sidebar_width, SIDEBAR_DEFAULT);
        assert_view_and_sessions_preserved(&reset, first_view, sessions);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT, window_w, height),
            deadline(),
        );

        // Parity in the other direction: a pointer drag to exactly
        // default + 2 steps, then keyboard steps that land on the same
        // values the drag would produce.
        client
            .send_ok(&dsl::drag_sidebar_divider(2.0 * RESIZE_STEP))
            .expect("drag to 312");
        flush_frame(&mut client);
        let parity = client.snapshot().expect("parity snapshot");
        assert_eq!(parity.sidebar_width, SIDEBAR_DEFAULT + 2.0 * RESIZE_STEP);
        keystroke(&mut client, "right");
        let parity_right = client.snapshot().expect("parity right snapshot");
        assert_eq!(
            parity_right.sidebar_width,
            SIDEBAR_DEFAULT + 3.0 * RESIZE_STEP
        );
        keystroke(&mut client, "left");
        let parity_left = client.snapshot().expect("parity left snapshot");
        assert_eq!(
            parity_left.sidebar_width,
            SIDEBAR_DEFAULT + 2.0 * RESIZE_STEP,
            "keyboard returns to the exact pointer-set width"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Cmd-B collapse/expand through the production path: collapse yields an
/// instant full-bleed native slot (width 0) and expand restores the last
/// expanded width. In a compact window the rail yields below the requested
/// width so the native content never drops below the 64 pt floor, and it
/// recovers the full requested width once the window widens again.
#[test]
#[ignore]
fn collapse_reopen_restores_width_and_compact_window_keeps_content_positive() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-collapse", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        let sessions = initial.engine_sessions;
        // Geometry captured only after the AppKit frame clamp settles (see
        // the pointer-drag test for the same host quirk).
        let settled = wait_for_window_settled(&mut client, deadline());
        let wide_w = settled.window_size[0];
        assert!(wide_w > 400.0);
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            wide_w - SIDEBAR_DEFAULT,
            deadline(),
        );
        let height = initial_rect.content_rect[3];
        // The GPUI window bounds include the macOS titlebar; the native slot
        // fills the content area below it. The delta is stable across
        // resizes, so the compact-window slot height derives from it.
        let titlebar_h = settled.window_size[1] - height;
        assert!(titlebar_h >= 0.0);

        // Remember a non-default width for the collapse test.
        client
            .send_ok(&dsl::drag_sidebar_divider(40.0))
            .expect("drag to 320");
        flush_frame(&mut client);
        let at_320 = client.snapshot().expect("320 snapshot");
        assert_eq!(at_320.sidebar_width, SIDEBAR_DEFAULT + 40.0);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, wide_w, height),
            deadline(),
        );

        // Cmd-B collapses: the rail is gone (width 0) and the native slot
        // re-flows instantly to the full window; the selected session is
        // untouched.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let collapsed = client.snapshot().expect("collapsed snapshot");
        assert!(!collapsed.sidebar_visible);
        assert_eq!(collapsed.sidebar_width, 0.0);
        assert!(
            !collapsed
                .visible_ids
                .iter()
                .any(|id| id == "bbr-sidebar" || id == "bbr-sidebar-divider"),
            "collapsed rail renders no sidebar ids"
        );
        assert_view_and_sessions_preserved(&collapsed, first_view, sessions);
        let collapsed_rect = wait_for_content_rect(
            &mut client,
            content_rect_for(0.0, wide_w, height),
            deadline(),
        );
        assert_eq!(collapsed_rect.content_rect[0], 0.0);
        assert_eq!(collapsed_rect.content_rect[2], wide_w);
        assert_eq!(collapsed_rect.content_rect[3], height);

        // Cmd-B expands: the last expanded width (320) is restored, not the
        // default.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let expanded = client.snapshot().expect("expanded snapshot");
        assert!(expanded.sidebar_visible);
        assert_eq!(
            expanded.sidebar_width,
            SIDEBAR_DEFAULT + 40.0,
            "expand restores the last expanded width"
        );
        assert_view_and_sessions_preserved(&expanded, first_view, sessions);
        wait_for_content_rect(
            &mut client,
            content_rect_for(SIDEBAR_DEFAULT + 40.0, wide_w, height),
            deadline(),
        );

        // Grow to the max, then shrink the window into the clamp region: the
        // rail yields below the requested width so the native content keeps
        // exactly the 64 pt floor — never zero or negative.
        client
            .send_ok(&dsl::drag_sidebar_divider(100_000.0))
            .expect("drag to max");
        flush_frame(&mut client);
        let max_snap = client.snapshot().expect("max snapshot");
        assert_eq!(max_snap.sidebar_width, SIDEBAR_MAX);
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: 340.0,
                height: 700.0,
            })
            .expect("compact window");
        flush_frame(&mut client);
        // The compact state is asserted from ONE frame where the window
        // width AND the native content rect agree (some hosts re-assert the
        // initial frame shortly after a programmatic resize, so the narrow
        // state is only observable transiently — a fast-but-wrong sample
        // must never pass).
        let compact = wait_for_window_and_content(&mut client, 340.0, CONTENT_MIN, deadline());
        let compact_w = compact.window_size[0];
        let expected_compact = SIDEBAR_MAX.min((compact_w - CONTENT_MIN).max(0.0));
        assert_eq!(
            compact.sidebar_width, expected_compact,
            "rail yields to the window"
        );
        assert_view_and_sessions_preserved(&compact, first_view, sessions);
        assert_eq!(
            compact.content_rect[2], CONTENT_MIN,
            "native content keeps exactly the floor"
        );
        assert!(compact.content_rect[2] > 0.0);
        assert!(compact.content_rect[3] > 0.0);

        // Even narrower: the content floor still holds (positive geometry).
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: 300.0,
                height: 700.0,
            })
            .expect("narrower window");
        flush_frame(&mut client);
        let narrow = wait_for_window_and_content(&mut client, 300.0, CONTENT_MIN, deadline());
        let narrow_w = narrow.window_size[0];
        let expected_narrow = SIDEBAR_MAX.min((narrow_w - CONTENT_MIN).max(0.0));
        assert_eq!(narrow.sidebar_width, expected_narrow);
        assert_view_and_sessions_preserved(&narrow, first_view, sessions);
        assert_eq!(narrow.content_rect[2], CONTENT_MIN);
        assert!(narrow.content_rect[2] > 0.0);

        // Widening the window again restores the full requested width.
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: wide_w,
                height: 700.0,
            })
            .expect("restore window");
        flush_frame(&mut client);
        let expected_restored_width = SIDEBAR_MAX.min((wide_w - CONTENT_MIN).max(0.0));
        let restored = wait_for_window_and_content(
            &mut client,
            wide_w,
            wide_w - expected_restored_width,
            deadline(),
        );
        let restored_w = restored.window_size[0];
        let expected_restored = SIDEBAR_MAX.min((restored_w - CONTENT_MIN).max(0.0));
        assert_eq!(restored.sidebar_width, expected_restored);
        assert_view_and_sessions_preserved(&restored, first_view, sessions);

        // Reset returns to the §8 default from the max.
        keystroke(&mut client, "0");
        flush_frame(&mut client);
        let reset = client.snapshot().expect("reset snapshot");
        assert_eq!(reset.sidebar_width, SIDEBAR_DEFAULT);
        assert_view_and_sessions_preserved(&reset, first_view, sessions);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

// U2 §11.4/§11.10/§11.11 scenarios (footer space strip, plus menu,
// favorites grid, folders region) live in focused child modules so this
// suite file stays under the ~500-line budget.
mod composition;
mod composition_geometry;
mod support;
mod u2_favorites;
mod u2_favorites_drag;
mod u2_folders;
mod u2_footer;
