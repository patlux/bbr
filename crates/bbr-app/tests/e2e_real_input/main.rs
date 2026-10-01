//! Real input E2E (reality-gaps packet, item C).
//!
//! Every other suite drives the app through testkit seams that call the
//! method a listener would call. These scenarios send real AppKit events
//! (`RealMouse` / `RealKey`) into a shown, key window instead, aimed at the
//! rects the controls drew at (`element_rects`) or at fixed points of a
//! fixture page: AppKit hit-tests the view, GPUI dispatches the listener,
//! menu key equivalents take the menu path, and a page with focus gets the
//! key first — the path a user's hand takes.
//!
//! - [`chrome`]: tab row select and close, Back in the titlebar, the footer
//!   Downloads and `+` controls, a space chip;
//! - [`favorites`]: a Favorite tile dragged back into the tab list;
//! - [`keys`]: ⌘L, ⌘T, ⌘W and ⇧⌘T while the web page has focus;
//! - [`web`]: a click on a link inside the page.
//!
//! No sleeps: every step polls typed snapshots or waits for semantic events
//! under a deadline. The suite takes the serial GUI lock.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod chrome;
mod favorites;
mod keys;
mod web;

use e2e_common::real_input::*;
use e2e_common::*;

/// Connect, settle on page one and make the window key.
fn start(client: &mut Client, token: &str) -> (WebViewId, u64) {
    client.hello(token).expect("hello");
    let mut since = client
        .attach(None)
        .expect("attach")
        .iter()
        .map(|(seq, _)| *seq)
        .max()
        .unwrap_or(0);
    let settled = wait_for_window_settled(client, deadline());
    let view = settled.active_web_view.expect("first native view");
    if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
        since = wait_title(client, view, PAGE_ONE_TITLE, since);
    }
    make_key(client);
    (view, since)
}

fn quit(client: &mut Client, harness: &mut Harness) {
    client.quit().expect("quit");
    let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
    assert!(status.success(), "app exit: {status}");
}
