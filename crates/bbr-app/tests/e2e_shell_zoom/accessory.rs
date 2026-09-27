//! §17.14 address zoom accessory (task packet
//! `address-field-space-identity`): the address field's trailing slot shows
//! the page zoom chip only while the site zoom is not 100 %, and pressing
//! the chip resets the site to 100 % through the production zoom authority
//! without opening the address overlay. Identity never changes.

use crate::e2e_common::*;
use crate::support::*;

const ZOOM_CHIP_ID: &str = "bbr-address-zoom";

fn chip_presented(snapshot: &bbr_testkit::protocol::Snapshot) -> bool {
    snapshot.visible_ids.iter().any(|id| id == ZOOM_CHIP_ID)
}

#[test]
#[ignore]
fn address_zoom_chip_appears_off_default_and_resets_on_press() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("zoom-chip", &fixtures.a1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let (baseline, _view) = start(&mut client, PAGE_ONE_TITLE, since);
        let site = origin_site(&fixtures.server_a.base_url());
        let rest = client.snapshot().expect("rest snapshot");
        assert_eq!(rest.address_zoom, None, "at 100 % the slot stays empty");
        assert!(!chip_presented(&rest));
        // A press on an absent chip is rejected, never a silent pass.
        assert!(client.send_ok(&dsl::click_address_zoom()).is_err());

        // Off 100 %: the chip shows the exact level.
        let zoomed = zoom_in_key(&mut client, ZOOM_DEFAULT + ZOOM_STEP, &site, &baseline);
        assert_eq!(zoomed.address_zoom, Some(ZOOM_DEFAULT + ZOOM_STEP));
        assert!(chip_presented(&zoomed), "the chip is in the accessory slot");

        // Pressing the chip resets to 100 % and never opens the overlay.
        client
            .send_ok(&dsl::click_address_zoom())
            .expect("press the zoom chip");
        let reset = wait_zoom(&mut client, ZOOM_DEFAULT, deadline());
        assert_zoom_at(&reset, ZOOM_DEFAULT, &site, &baseline);
        assert_eq!(reset.address_zoom, None, "back at 100 % the chip is gone");
        assert!(!chip_presented(&reset));
        assert_eq!(
            reset.command_surface,
            bbr_testkit::protocol::CommandSurface::None,
            "the chip press never opens the address overlay"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
