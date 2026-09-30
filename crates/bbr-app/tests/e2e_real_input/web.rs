//! A real click inside the web page: WebKit gets the event from AppKit and
//! the page's own link activation navigates.

use bbr_testkit::protocol::RealMouseButton;

use crate::e2e_common::real_input::*;
use crate::e2e_common::*;
use crate::{quit, start};

#[test]
#[ignore]
fn a_real_click_on_a_page_link_follows_it() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-web", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);
        client
            .send_ok(&dsl::navigate(fixtures.url("/real-input.html")))
            .expect("navigate");
        since = wait_title(&mut client, view, REAL_INPUT_TITLE, since);

        let snapshot = drawn(&mut client);
        real_click_at(
            &mut client,
            page_point(&snapshot, PLAIN_LINK),
            RealMouseButton::Left,
            &[],
        );
        wait_title(&mut client, view, PAGE_TWO_TITLE, since);
        let after = drawn(&mut client);
        assert_eq!(after.tabs.len(), 1, "a plain click stays in the tab");

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
