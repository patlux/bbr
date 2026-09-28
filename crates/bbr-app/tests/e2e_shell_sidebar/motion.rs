//! Real-app v2 motion core E2E (task packet `v2-motion-core`, DESIGN §6,
//! §10.5, §18.5 D5). The typed `motion` evidence is resolved through the
//! same `bbr_ui` moment plans and the same row states / address progress
//! the render paints with, so the suite proves over a real slow WKWebView
//! navigation:
//!
//! 1. while the active tab loads, the favicon arc and the address sweep
//!    run (`animated`); the finish plays the address fill-and-fade once and
//!    is gone afterwards, together with the arc;
//! 2. under Reduce Motion both loops are `static` and no finish plays;
//! 3. with nothing transient the shell requests no animation frames: a
//!    forced frame renders but builds no animation element (the only thing
//!    that requests frames), before and after a bounded idle window. This
//!    render-side proof does not depend on the display link ticking, which
//!    it does not for hidden (or screen-locked) test windows.

use std::time::{Duration, Instant};

use super::*;
use bbr_testkit::protocol::{MotionMomentSnapshot, Snapshot};

fn moment(name: &str, target: &str, variant: &str) -> MotionMomentSnapshot {
    MotionMomentSnapshot {
        moment: name.into(),
        target: target.into(),
        variant: variant.into(),
    }
}

/// Force one frame and return the animation elements it built (the
/// render-side proof of whether that frame requests animation frames) and
/// whether the frame really rendered.
fn frame_builds(client: &mut Client) -> (u64, bool) {
    let before = client.snapshot().expect("before frame").motion;
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
    let after = client.snapshot().expect("after frame").motion;
    (
        after.animated_elements - before.animated_elements,
        after.root_renders > before.root_renders,
    )
}

/// Poll the snapshot until `done` holds (bounded by the suite deadline).
fn wait_snapshot(client: &mut Client, done: impl Fn(&Snapshot) -> bool, what: &str) -> Snapshot {
    let end = deadline();
    loop {
        let snapshot = client.snapshot().expect("snapshot");
        if done(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < end, "{what}: {:?}", snapshot.motion);
        std::thread::yield_now();
    }
}

/// Navigate the active tab to the slow fixture and return the snapshot
/// taken while it is still loading, plus the event cursor.
fn start_slow_load(
    client: &mut Client,
    view: WebViewId,
    slow: &str,
    since: u64,
) -> (Snapshot, u64) {
    client.send_ok(&dsl::navigate(slow)).expect("navigate slow");
    let since = wait_web_event(
        client,
        |event| matches!(event, WebEvent::LoadingChanged(v, true) if *v == view),
        since,
    );
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush loading frame");
    (client.snapshot().expect("loading snapshot"), since)
}

#[test]
#[ignore]
fn loading_moments_run_while_loading_and_the_idle_shell_requests_no_frames() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let slow = fixtures.url("/__fx/slow");
    let (mut harness, socket, token) = spawn_harness("sidebar-v2-motion", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");
        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        let index = initial.active_tab_index.expect("active row");
        let favicon = format!("bbr-tab-favicon-{index}");

        // 1. Full motion: both loops run while the tab loads.
        let (loading, next) = start_slow_load(&mut client, view, &slow, since);
        since = next;
        assert_eq!(loading.tab_states[index], TabVisualState::Loading);
        assert!(!loading.shell_modes.reduced_motion);
        assert_eq!(
            loading.motion.moments,
            vec![
                moment("loading-arc", &favicon, "animated"),
                moment("address-progress", "bbr-address-progress", "animated"),
            ],
            "arc and sweep run while the committed loading state holds"
        );
        assert!(
            loading
                .visible_ids
                .iter()
                .any(|id| id == "bbr-address-progress"),
            "the D5 line renders in the address field"
        );
        // The loops really build animation elements: a forced frame while
        // the server still holds the slow response builds the arc and the
        // sweep (each requests the next animation frame).
        let (built, rendered) = frame_builds(&mut client);
        assert!(rendered, "the forced frame rendered");
        assert_eq!(built, 2, "arc + sweep animation elements per frame");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(v, false) if *v == view),
            since,
        );
        let finished = client.snapshot().expect("finished snapshot");
        assert_eq!(finished.tab_states[index], TabVisualState::Selected);
        assert_eq!(
            finished.motion.moments,
            vec![moment(
                "address-progress-finish",
                "bbr-address-progress",
                "animated"
            )],
            "the finish fills and fades once; the arc stopped with the state"
        );
        let settled = wait_snapshot(
            &mut client,
            |snapshot| snapshot.motion.moments.is_empty(),
            "the finish is removed after it played",
        );
        assert!(!settled
            .visible_ids
            .iter()
            .any(|id| id == "bbr-address-progress"));

        // 3. Idle: nothing transient — a rendered frame builds no animation
        // element, so it requests no animation frame; the registry agrees
        // over a bounded idle window.
        let (built, rendered) = frame_builds(&mut client);
        assert!(rendered, "the idle frame really rendered");
        assert_eq!(built, 0, "an idle frame requests no animation frames");
        let idle_end = Instant::now() + Duration::from_millis(600);
        while Instant::now() < idle_end {
            let idle = client.snapshot().expect("idle snapshot");
            assert!(idle.motion.moments.is_empty(), "idle presents no moment");
            std::thread::yield_now();
        }
        let (built, _) = frame_builds(&mut client);
        assert_eq!(built, 0, "still no animation element after the idle window");

        // 2. Reduce Motion: both loops are static; no finish plays.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        let (reduced, next) = start_slow_load(&mut client, view, &slow, since);
        since = next;
        assert!(reduced.shell_modes.reduced_motion);
        assert_eq!(
            reduced.motion.moments,
            vec![
                moment("loading-arc", &favicon, "static"),
                moment("address-progress", "bbr-address-progress", "static"),
            ],
            "Reduce Motion keeps the static arc and line"
        );
        let (built, rendered) = frame_builds(&mut client);
        assert!(rendered);
        assert_eq!(built, 0, "static variants request no animation frames");
        let _ = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::LoadingChanged(v, false) if *v == view),
            since,
        );
        let reduced_done = client.snapshot().expect("reduced finished");
        assert!(
            reduced_done.motion.moments.is_empty(),
            "Reduce Motion removes the line on finish: {:?}",
            reduced_done.motion
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
