//! D4 (§18.1, §18.5) space-switch tint fade over the real app (task packet
//! `v2-motion-moments`). Commit first: the destination space and its exact
//! chrome tint are active in the same snapshot whose motion evidence
//! records the fade starting — full motion cross-fades only the chrome
//! tint (`animated`), Reduce Motion switches instantly (`static`), and a
//! swipe commit (already at the destination tint) starts no fade at all.

use super::*;
use bbr_testkit::protocol::{MotionMomentSnapshot, Snapshot};

/// All `space-tint-fade` starts, oldest first.
fn fades(snapshot: &Snapshot) -> Vec<MotionMomentSnapshot> {
    snapshot
        .motion
        .started
        .iter()
        .filter(|start| start.moment == "space-tint-fade")
        .cloned()
        .collect()
}

#[test]
#[ignore]
fn a_committed_switch_fades_only_the_chrome_tint() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-v2-tint-fade", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light");
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");
        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let reading = create_space_id(&mut client, &mut since, initial.active_profile, "Reading");
        client
            .send_ok(&dsl::set_space_theme(reading, "pine"))
            .expect("set pine theme");
        activate_space_id(&mut client, &mut since, default_space);
        let (cobalt_chrome, _) = light_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        let (pine_chrome, _) = light_standard_tints(bbr_domain::SpaceTheme::Pine);
        let before = fades(&client.snapshot().expect("baseline")).len();

        // Full motion: the destination tint is committed at once; only the
        // chrome tint fades.
        activate_space_id(&mut client, &mut since, reading);
        let switched = client.snapshot().expect("switched snapshot");
        assert_eq!(switched.active_space, reading, "the switch commits first");
        assert_eq!(
            switched.active_appearance.as_ref().unwrap().chrome_tint,
            pine_chrome,
            "the committed tint is the destination"
        );
        let started = fades(&switched);
        assert_eq!(started.len(), before + 1, "one fade per committed switch");
        let fade = started.last().expect("fade");
        assert_eq!(fade.target, "bbr-sidebar");
        assert_eq!(fade.variant, "animated");

        // Reduce Motion: instant.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        activate_space_id(&mut client, &mut since, default_space);
        let instant = client.snapshot().expect("reduced switch snapshot");
        assert_eq!(instant.active_space, default_space);
        assert_eq!(
            instant.active_appearance.as_ref().unwrap().chrome_tint,
            cobalt_chrome
        );
        assert_eq!(fades(&instant).last().expect("start").variant, "static");
        assert!(
            !instant
                .motion
                .moments
                .iter()
                .any(|moment| moment.moment == "space-tint-fade"),
            "no fade runs under Reduce Motion"
        );

        // A swipe commit already reached the destination tint: no fade.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion again");
        let starts = fades(&client.snapshot().expect("pre-swipe")).len();
        for (delta, phase) in [
            ([120.0, 0.0], bbr_testkit::protocol::SwipePhaseWire::Started),
            ([240.0, 0.0], bbr_testkit::protocol::SwipePhaseWire::Moved),
            ([0.0, 0.0], bbr_testkit::protocol::SwipePhaseWire::Ended),
        ] {
            client
                .send_ok(&dsl::swipe_scroll(delta, phase))
                .expect("swipe step");
        }
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let swiped = client.snapshot().expect("swiped snapshot");
        assert_eq!(swiped.active_space, reading);
        assert_eq!(
            swiped.active_appearance.as_ref().unwrap().chrome_tint,
            pine_chrome
        );
        assert_eq!(fades(&swiped).len(), starts, "a swipe commit never fades");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
