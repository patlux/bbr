//! Real input (reality-gaps packet, item C): a shown key window and real
//! AppKit events aimed from snapshot geometry.
//!
//! AppKit gives a click to a window only when it is key (GPUI does not
//! accept the first mouse of an inactive window) and a key press only to
//! the key window — exactly as for a user. The suites therefore run a shown
//! window, activate the app until the window is key, and check after every
//! event that AppKit routed it to that window.

use std::path::PathBuf;
use std::time::Instant;

use bbr_testkit::protocol::{RealModifier, RealMouseButton, RealMousePhase, Snapshot};
use bbr_testkit::Client;

use super::{deadline, harness_root, Harness};

/// The page's plain link and new-window link (`real-input.html`), as page
/// points: the link rects are fixed in the fixture's CSS.
pub const REAL_INPUT_TITLE: &str = "BBR Real Input";
pub const PLAIN_LINK: (f64, f64) = (160.0, 60.0);
pub const BLANK_LINK: (f64, f64) = (160.0, 140.0);

/// Spawn the harness with a shown window (no `--bbr-hidden`).
pub fn spawn_harness_shown(root_tag: &str, page: &str) -> (Harness, PathBuf, String) {
    let pid = std::process::id();
    let root = harness_root(root_tag, pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = profile.join("bbr-test.sock");
    let token = format!("shell-{root_tag}-{pid}");
    let harness = Harness::spawn(
        env!("CARGO_BIN_EXE_bbr"),
        &[
            "--bbr-initial-url",
            page,
            "--bbr-profile-dir",
            profile.to_str().unwrap(),
            "--bbr-test-token",
            &token,
            "--bbr-ids=sequential",
        ],
        &root,
    );
    (harness, socket, token)
}

/// A drawn snapshot (the probes record while the frame is laid out).
pub fn drawn(client: &mut Client) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
    client.snapshot().expect("snapshot")
}

/// Activate the app until the browser window is key.
pub fn make_key(client: &mut Client) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::protocol::dsl::activate_app())
            .expect("activate app");
        let snapshot = drawn(client);
        if snapshot.window_key {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "the browser window never became key"
        );
        std::thread::yield_now();
    }
}

/// The drawn rect of the probed control `id`.
pub fn rect_of(snapshot: &Snapshot, id: &str) -> Option<[f32; 4]> {
    snapshot
        .element_rects
        .iter()
        .find(|(candidate, _)| candidate == id)
        .map(|(_, rect)| *rect)
}

pub fn center(rect: [f32; 4]) -> (f64, f64) {
    (
        f64::from(rect[0] + rect[2] / 2.0),
        f64::from(rect[1] + rect[3] / 2.0),
    )
}

/// Send one real-input request. AppKit only takes it to a key window, so a
/// request refused because another app took the key window is resent
/// after the window is key again; nothing was delivered before.
fn send_real(client: &mut Client, request: &bbr_testkit::Request) {
    let until = deadline();
    loop {
        match client.send_ok(request) {
            Ok(()) => break,
            Err(error) if error.code == "not_key" => {
                assert!(Instant::now() < until, "the window never stayed key");
                eprintln!("real input: another app took the key window; resending {request:?}");
                make_key(client);
            }
            Err(error) => panic!("real input refused: {error:?}"),
        }
    }
    let snapshot = client.snapshot().expect("snapshot");
    let readback = snapshot.real_input.expect("a real-input readback");
    assert!(
        readback.routed_to_window,
        "AppKit did not take the event to the window: {readback:?}"
    );
}

/// One real mouse event.
pub fn real_mouse(
    client: &mut Client,
    phase: RealMousePhase,
    button: RealMouseButton,
    (x, y): (f64, f64),
    modifiers: &[RealModifier],
) {
    send_real(
        client,
        &bbr_testkit::protocol::dsl::real_mouse(phase, button, x, y, modifiers.to_vec()),
    );
}

/// A real click (down + up) at a window point.
pub fn real_click_at(
    client: &mut Client,
    point: (f64, f64),
    button: RealMouseButton,
    modifiers: &[RealModifier],
) {
    // One gesture: the pointer arrives, presses and releases in one app
    // turn, so another app taking the key window cannot split the click.
    send_real(
        client,
        &bbr_testkit::protocol::dsl::real_click(button, point.0, point.1, modifiers.to_vec()),
    );
}

/// A real left click at the center of the drawn control `id`.
pub fn real_click(client: &mut Client, id: &str) {
    let snapshot = drawn(client);
    let rect = rect_of(&snapshot, id).unwrap_or_else(|| {
        panic!(
            "no drawn rect for {id}; drawn: {:?}",
            snapshot
                .element_rects
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>()
        )
    });
    real_click_at(client, center(rect), RealMouseButton::Left, &[]);
}

/// A real hover (pointer move) at a window point.
pub fn real_hover(client: &mut Client, point: (f64, f64)) {
    real_mouse(
        client,
        RealMousePhase::Moved,
        RealMouseButton::Left,
        point,
        &[],
    );
}

/// A real key press (down + up) in the `DispatchKeystroke` spelling.
pub fn real_key(client: &mut Client, keystroke: &str) {
    send_real(client, &bbr_testkit::protocol::dsl::real_key(keystroke));
}

/// A window point of the web page: the content rect's origin plus a page
/// point.
pub fn page_point(snapshot: &Snapshot, (x, y): (f64, f64)) -> (f64, f64) {
    (
        f64::from(snapshot.content_rect[0]) + x,
        f64::from(snapshot.content_rect[1]) + y,
    )
}

/// Poll drawn snapshots until `ready` holds.
pub fn wait_for(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = drawn(client);
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "timed out waiting for {what}");
        std::thread::yield_now();
    }
}
