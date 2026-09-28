//! Blueprint D22 (DESIGN §2/§11.14): full screen is full-bleed — inset 0,
//! radius 0, no shadow, no chassis band — and the titlebar row keeps its
//! controls without the traffic-light clearance, then everything returns
//! when full screen ends. Synchronizes on typed snapshot state under a
//! deadline.

use super::*;

fn wait(
    client: &mut Client,
    what: &str,
    done: impl Fn(&bbr_testkit::protocol::Snapshot) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        flush_frame(client);
        let snapshot = client.snapshot().expect("snapshot");
        if done(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} never settled");
    }
}

#[test]
#[ignore]
fn full_screen_is_full_bleed_and_the_titlebar_keeps_its_controls() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("titlebar-full-screen", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let windowed = wait_for_window_settled(&mut client, deadline());
        let lights = windowed.titlebar.traffic_lights;
        assert!(lights[2] > 0.0, "windowed: the traffic lights take room");

        client
            .send_ok(&dsl::set_full_screen_override(Some(true)))
            .expect("present full screen");
        let full = wait(&mut client, "full screen", |s| s.content_card.full_screen);
        let card = full.content_card;
        assert_eq!(
            (
                card.inset_top,
                card.inset_trailing,
                card.inset_bottom,
                card.radius,
                card.shadow
            ),
            (0.0, 0.0, 0.0, 0.0, false),
            "full-bleed card"
        );
        assert!(!full.visible_ids.iter().any(|id| id == "bbr-chassis-band"));
        assert_eq!(
            full.titlebar.traffic_lights, [0.0; 4],
            "no traffic-light clearance"
        );
        assert!(
            full.titlebar.safe_area_height >= 40.0,
            "the row keeps its height"
        );
        assert_ids(
            &full,
            &[
                "bbr-titlebar",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
            ],
        );

        client
            .send_ok(&dsl::set_full_screen_override(None))
            .expect("leave full screen");
        let back = wait(&mut client, "windowed again", |s| {
            !s.content_card.full_screen
        });
        assert_eq!(
            back.titlebar.traffic_lights, lights,
            "the clearance returns"
        );
        assert!(back.content_card.shadow && back.content_card.radius > 0.0);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
