//! Ladder, clamp, and reset scenarios (queue item 11, part A).
//!
//! Proves the merged contract's integer-percent ladder through the REAL key
//! bindings (`cmd-+` / `cmd--` / `cmd-0` dispatched through GPUI's input
//! path): 100 → 125 → 150 → …, deterministic clamps at 25 and 400, and
//! `Cmd-0` returning to the documented default with the override record
//! removed. Every step asserts the unchanged tab/session/profile/space
//! identity (never a session creation, move, or replacement).

use crate::e2e_common::*;
use crate::support::*;

/// The iteration-policy first proof: one `Cmd-Plus` from the default, one
/// native application, exact snapshot state, identity untouched.
#[test]
#[ignore]
fn one_cmd_plus_steps_the_contract_ladder() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("zoom-one", &fixtures.a1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let (baseline, _view) = start(&mut client, PAGE_ONE_TITLE, since);
        let site = origin_site(&fixtures.server_a.base_url());

        // One real Cmd-Plus: the ladder moves exactly one contract step
        // (25 percentage points) and the level is applied natively.
        zoom_in_key(&mut client, ZOOM_DEFAULT + ZOOM_STEP, &site, &baseline);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The full ladder, both clamps, and the `Cmd-0` reset through the real key
/// path: 100 → 400 in 12 exact steps, clamping at 400; back down to 25 in
/// 15 exact steps, clamping at 25; `Cmd-0` returns to 100 with no pending
/// write and no identity change.
#[test]
#[ignore]
fn ladder_clamps_at_both_ends_and_reset_restores_default() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("zoom-ladder", &fixtures.a1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let (baseline, _view) = start(&mut client, PAGE_ONE_TITLE, since);
        let site = origin_site(&fixtures.server_a.base_url());

        // 100 → 400: exactly 12 steps of 25 (the merged contract's ladder).
        let mut level = ZOOM_DEFAULT;
        for _ in 0..12 {
            level += ZOOM_STEP;
            zoom_in_key(&mut client, level, &site, &baseline);
        }
        assert_eq!(level, ZOOM_MAX);

        // Clamp at the top: further Cmd-Plus is a deterministic no-op.
        zoom_in_key(&mut client, ZOOM_MAX, &site, &baseline);
        zoom_in_key(&mut client, ZOOM_MAX, &site, &baseline);

        // 400 → 25: exactly 15 steps of 25, then the bottom clamp holds.
        let mut level = ZOOM_MAX;
        for _ in 0..15 {
            level -= ZOOM_STEP;
            zoom_out_key(&mut client, level, &site, &baseline);
        }
        assert_eq!(level, ZOOM_MIN);
        zoom_out_key(&mut client, ZOOM_MIN, &site, &baseline);
        zoom_out_key(&mut client, ZOOM_MIN, &site, &baseline);

        // Cmd-0 resets to the documented default; no pending write remains
        // (the override record was removed, never set to the default).
        key(&mut client, "cmd-0");
        let snapshot = wait_zoom(&mut client, ZOOM_DEFAULT, deadline());
        assert_zoom_at(&snapshot, ZOOM_DEFAULT, &site, &baseline);
        assert_eq!(snapshot.zoom.applied, Some(ZOOM_DEFAULT));
        assert!(!snapshot.zoom.pending, "reset removed the override record");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
