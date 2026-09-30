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

/// Reality gaps (item E): a script that clicks a `target=_blank` link is a
/// pop-up the blocker catches (quiet banner, nothing opens), while the
/// user's own click on the same link opens it (the Peek preview).
#[test]
#[ignore]
fn a_script_click_on_a_new_window_link_is_blocked_and_a_real_one_opens() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-popup", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (view, mut since) = start(&mut client, &token);
        client
            .send_ok(&dsl::navigate(fixtures.url("/real-input.html?script")))
            .expect("navigate");
        since = wait_title(&mut client, view, "BBR Real Input Scripted", since);
        let blocked = wait_for(&mut client, "the blocked pop-up notice", |s| {
            s.permissions.popup_notice.is_some()
        });
        assert_eq!(blocked.tabs.len(), 1, "nothing opened on its own");
        assert!(!blocked.peek_presented, "no preview on its own");

        client
            .send_ok(&dsl::navigate(fixtures.url("/real-input.html")))
            .expect("navigate");
        since = wait_title(&mut client, view, REAL_INPUT_TITLE, since);
        let snapshot = drawn(&mut client);
        real_click_at(
            &mut client,
            page_point(&snapshot, BLANK_LINK),
            RealMouseButton::Left,
            &[],
        );
        wait_for(&mut client, "the user's click opens the link", |s| {
            s.peek_presented
        });
        client
            .send_ok(&bbr_testkit::Request::PeekClose)
            .expect("close the preview");
        wait_for(&mut client, "the preview closed", |s| !s.peek_presented);

        // The keyboard is the user too: Enter on the focused link opens it
        // (the page moves its focus to the link when its body is clicked).
        client
            .send_ok(&dsl::navigate(fixtures.url("/real-input.html?focus")))
            .expect("navigate");
        since = wait_title(&mut client, view, REAL_INPUT_TITLE, since);
        let snapshot = drawn(&mut client);
        real_click_at(
            &mut client,
            page_point(&snapshot, (400.0, 400.0)),
            RealMouseButton::Left,
            &[],
        );
        wait_title(&mut client, view, "BBR Real Input Focused", since);
        real_key(&mut client, "enter");
        // Opened, not blocked (a blocked activation opens nothing; the
        // notice of the scripted load above stays while the page stays on
        // its site, DESIGN §19 D13).
        wait_for(&mut client, "Enter opens the link", |s| s.peek_presented);

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
