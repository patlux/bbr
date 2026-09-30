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

/// Reality gaps (item D): a ⌘-click and a middle click on an ordinary link
/// (no `target`) open background children after the opener, in click
/// order, and the opener stays the active page.
#[test]
#[ignore]
fn command_and_middle_clicks_on_a_link_open_background_tabs() {
    use bbr_testkit::protocol::RealModifier;

    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-bg", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);
        let opener_url = fixtures.url("/real-input.html");
        client
            .send_ok(&dsl::navigate(opener_url.clone()))
            .expect("navigate");
        since = wait_title(&mut client, view, REAL_INPUT_TITLE, since);
        let _ = since;

        let snapshot = drawn(&mut client);
        let link = page_point(&snapshot, PLAIN_LINK);
        real_click_at(
            &mut client,
            link,
            RealMouseButton::Left,
            &[RealModifier::Command],
        );
        let one = wait_for(&mut client, "a background tab from ⌘-click", |s| {
            s.tabs.len() == 2
        });
        assert_eq!(one.tabs[0].url, opener_url, "the opener keeps its page");
        assert_eq!(one.active_tab_index, Some(0), "the opener stays active");
        assert_eq!(one.active_web_view, Some(view));
        wait_for(&mut client, "the child loads the link", |s| {
            s.tabs.get(1).is_some_and(|tab| tab.url == page_two)
        });

        real_click_at(&mut client, link, RealMouseButton::Middle, &[]);
        let two = wait_for(&mut client, "a background tab from a middle click", |s| {
            s.tabs.len() == 3
        });
        assert_eq!(two.tabs[0].url, opener_url);
        assert_eq!(two.active_tab_index, Some(0));
        wait_for(
            &mut client,
            "both children load the link, in click order",
            |s| s.tabs[1].url == page_two && s.tabs[2].url == page_two,
        );

        // A link that asks for a new window: still exactly one background
        // tab (the policy takes it before WebKit asks for a new view).
        let blank = page_point(&drawn(&mut client), BLANK_LINK);
        real_click_at(
            &mut client,
            blank,
            RealMouseButton::Left,
            &[RealModifier::Command],
        );
        let three = wait_for(
            &mut client,
            "one background tab from ⌘-click on _blank",
            |s| s.tabs.len() == 4,
        );
        assert_eq!(three.active_tab_index, Some(0));
        let settled = wait_for(&mut client, "no further tab", |s| s.tabs[3].url == page_two);
        assert_eq!(settled.tabs.len(), 4, "exactly one tab per click");
        assert!(!settled.peek_presented, "no preview for a background open");

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
