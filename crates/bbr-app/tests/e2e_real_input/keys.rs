//! Real key presses while the web page has focus: the menu key equivalents
//! go through AppKit (the window, WebKit, the main menu) to the app.

use bbr_testkit::protocol::{CommandBarMode, RealMouseButton};

use crate::e2e_common::real_input::*;
use crate::e2e_common::*;
use crate::{quit, start};

/// A real click on the page body focuses the page.
fn focus_page(client: &mut Client) {
    let snapshot = drawn(client);
    real_click_at(
        client,
        page_point(&snapshot, (400.0, 400.0)),
        RealMouseButton::Left,
        &[],
    );
    let focused = wait_for(client, "the page focused", |s| !s.chrome_focused);
    let responder = focused
        .real_input
        .and_then(|readback| readback.first_responder)
        .unwrap_or_default();
    assert!(
        responder.contains("WKWebView"),
        "the page is the first responder: {responder}"
    );
}

#[test]
#[ignore]
fn shortcuts_reach_the_app_while_the_page_has_focus() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("real-keys", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let (_view, mut since) = start(&mut client, &token);

        focus_page(&mut client);
        real_key(&mut client, "cmd-l");
        wait_for(&mut client, "⌘L opens the location", |s| {
            s.command_bar_mode == CommandBarMode::Location && s.chrome_focused
        });
        real_key(&mut client, "escape");
        wait_for(&mut client, "Escape returns to the page", |s| {
            s.command_bar_mode == CommandBarMode::Inactive && !s.chrome_focused
        });

        focus_page(&mut client);
        real_key(&mut client, "cmd-t");
        wait_for(&mut client, "⌘T opens New Tab", |s| {
            s.command_bar_mode == CommandBarMode::NewTab && s.chrome_focused
        });
        real_key(&mut client, "escape");
        wait_for(&mut client, "New Tab closed", |s| {
            s.command_bar_mode == CommandBarMode::Inactive
        });

        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        focus_page(&mut client);
        real_key(&mut client, "cmd-w");
        let closed = wait_for(&mut client, "⌘W closes the tab", |s| s.tabs.len() == 1);
        assert!(closed.tabs.iter().all(|tab| tab.url != page_two));
        focus_page(&mut client);
        real_key(&mut client, "shift-cmd-t");
        wait_for(&mut client, "⇧⌘T reopens it", |s| {
            s.tabs.len() == 2 && s.tabs.iter().any(|tab| tab.url == page_two)
        });

        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
