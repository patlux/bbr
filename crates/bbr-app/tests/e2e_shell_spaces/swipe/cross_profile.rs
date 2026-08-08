//! §11.3 swipe **cross-profile** evidence: the grouped display order can
//! place another profile's space adjacent, the commit routes through the
//! same `activate_space` (no session recreation, the one-interaction
//! command-field profile hint), and the reverse swipe is cross-profile too —
//! the hint names the new destination until a user-interaction boundary (a
//! tab mutation) clears it.

use super::*;

/// Cross-profile adjacent swipe keeps the existing authority: the grouped
/// display order can place another profile's space adjacent, the commit
/// routes through the same `activate_space` (no session recreation, the
/// one-interaction command-field profile hint), and the reverse swipe is
/// cross-profile too — the hint names the new destination until a
/// user-interaction boundary (a tab mutation) clears it.
#[test]
#[ignore]
fn cross_profile_adjacent_swipe_keeps_profile_hint_authority() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-swipe-cross", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // A fresh profile: its space (id 6) renders adjacent to the default
        // space (id 2) in grouped display order [2, 6].
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };

        // Swipe next from Default commits into the adjacent cross-profile
        // space through the one activation authority.
        client
            .send_ok(&dsl::swipe_scroll(
                [240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe started");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("swipe ended");
        let seq = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        since = seq;
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after cross-profile commit");
        let cross = client.snapshot().expect("cross-profile snapshot");
        assert_eq!(cross.active_space, private_space);
        assert_eq!(cross.active_profile, private_profile);
        let hint = cross
            .command_profile_hint
            .as_deref()
            .expect("cross-profile activation sets the one-interaction hint");
        assert!(
            hint.contains("Private"),
            "the hint announces the destination profile (got {hint:?})"
        );
        assert_eq!(cross.engine_sessions, 1, "no session recreated");
        assert_eq!(selected_space(&cross), private_space);
        assert!(cross.chrome_focused);
        assert_eq!(cross.swipe.phase, "idle");

        // Swipe prev back into the default profile: the reverse direction is
        // **itself cross-profile** — the one-interaction hint names the new
        // destination (Default), exactly like the canonical hint E2E
        // ("the reverse direction is cross-profile too"). A tab mutation then
        // clears the hint at the deterministic user-interaction boundary.
        client
            .send_ok(&dsl::swipe_scroll(
                [-240.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Started,
            ))
            .expect("swipe back started");
        client
            .send_ok(&dsl::swipe_scroll(
                [0.0, 0.0],
                bbr_testkit::protocol::SwipePhaseWire::Ended,
            ))
            .expect("swipe back ended");
        let seq = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        since = seq;
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after swipe back");
        let back = client.snapshot().expect("swiped back snapshot");
        assert_eq!(back.active_space, default_space);
        assert_eq!(back.active_profile, default_profile);
        assert_eq!(
            back.command_profile_hint.as_deref(),
            Some("in Profile · Default"),
            "the reverse swipe is cross-profile too: the hint names Default"
        );
        assert_eq!(
            back.active_web_view,
            Some(first_view),
            "warm first view restores"
        );
        assert_eq!(back.engine_sessions, 1);
        assert_eq!(selected_space(&back), default_space);
        assert_eq!(back.swipe.phase, "idle");

        // A tab mutation ends the one-interaction hint (the deterministic
        // user-interaction boundary the canonical hint suite proves).
        client
            .send_ok(&dsl::click_tab(0))
            .expect("select the default tab");
        let seq = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let _seq = seq;
        let cleared = client.snapshot().expect("hint cleared snapshot");
        assert_eq!(
            cleared.command_profile_hint, None,
            "a tab mutation clears the one-interaction hint"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
