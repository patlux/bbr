//! Deterministic process-shutdown E2E (task packet
//! `process-lifecycle-shutdown`).
//!
//! Launches the real `bbr` binary and proves both user process-lifecycle
//! paths through real process exit, not hidden windows or internal shutdown
//! state:
//!
//! 1. **Owning-window close** (`CloseWindow`): the real GPUI browser window
//!    is removed through the production window path; BBR's browser-window
//!    observer runs the graceful quit and the complete process exits
//!    successfully within 10 seconds. The gateway response for the close is
//!    flushed to the client before teardown (no EOF race), matching the
//!    graceful-`Quit` guarantee.
//! 2. **Owning-window close with an open portal child**: the implementation-
//!    owned native portal host window is presented and attached; closing the
//!    owning browser window still terminates the complete process — a portal
//!    child never owns process lifetime.
//! 3. **Real `SIGINT`** (terminal Ctrl-C): the signal reaches the async
//!    listener, which schedules graceful `cx.quit()` on GPUI's
//!    foreground/main thread; the complete process exits successfully within
//!    10 seconds (a default-disposition signal death would fail the success
//!    assertion).
//!
//! The suite waits on process state under explicit deadlines and uses the
//! same failure artifacts as the other real-app E2E harnesses (event ring +
//! app stderr preserved in the kept profile dir). Run through
//! `just e2e-shutdown`; the canonical sequential `just e2e` gate includes it
//! at position 2.

#![cfg(target_os = "macos")]

mod e2e_common;

use std::process::Command;
use std::time::{Duration, Instant};

use bbr_testkit::protocol::{dsl, Request};
use e2e_common::*;

/// The probe presentation's stable id (`PortalPresentationId(0xB9)` in
/// `crate::portal::probe`), exposed through the snapshot's typed
/// `portal.presentation` field.
const PROBE_PRESENTATION_ID: u64 = 0xB9;

/// Assert the real app is ready to be shut down: the gateway answered the
/// handshake and the snapshot shows a live native session (the deterministic
/// fixture page is loaded into a real WKWebView).
fn wait_ready(client: &mut Client, socket: &std::path::Path) -> u64 {
    let replay = client.attach(None).expect("attach replay");
    let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
    // The engine session may take a moment to attach its first web view;
    // poll the typed snapshot under the standard deadline.
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("readiness snapshot");
        if snapshot.web_view_hosted {
            return since;
        }
        assert!(
            Instant::now() < until,
            "app did not reach readiness (web_view_hosted); artifacts at {}",
            socket.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Terminal Ctrl-C (`SIGINT`) reaches the async listener and the complete
/// process exits gracefully within 10 seconds.
#[test]
#[ignore]
fn sigint_terminates_the_complete_process() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("shutdown-sigint", &page_one);
    let pid = harness.child.id() as i32;

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _since = wait_ready(&mut client, &socket);
        drop(client); // the gateway socket is about to die with the process

        // Deliver a real SIGINT to the spawned BBR process — the exact
        // signal a terminal Ctrl-C sends.
        let status = Command::new("kill")
            .arg("-INT")
            .arg(pid.to_string())
            .status()
            .expect("kill -INT the bbr process");
        assert!(status.success(), "kill -INT failed for pid {pid}: {status}");

        let exit = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(
            exit.success(),
            "SIGINT must quit BBR gracefully, got {exit} (a default-disposition \
             signal death would mean the async listener is broken)"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Closing the real owning browser window through the production GPUI path
/// terminates the complete process within 10 seconds, with the close
/// response flushed to the gateway client before teardown.
#[test]
#[ignore]
fn close_window_terminates_the_complete_process() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("shutdown-close", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _since = wait_ready(&mut client, &socket);

        // Close the browser window through the production GPUI window path
        // (`window.remove_window()` → window-closed observer → `cx.quit()`).
        // `send_ok` returns only after the gateway flushed the successful
        // response, so the response can never race process/socket teardown.
        client
            .send_ok(&Request::CloseWindow)
            .expect("CloseWindow response must be flushed before teardown");

        let exit = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(
            exit.success(),
            "owning-window close must quit BBR gracefully, got {exit}"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Closing the owning browser window while an implementation-owned portal
/// child window is presented and attached still terminates the complete
/// process: the portal child never owns process lifetime.
#[test]
#[ignore]
fn close_window_with_open_portal_child_terminates_the_process() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("shutdown-portal", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _since = wait_ready(&mut client, &socket);

        // Present the portal probe: the one implementation-owned native
        // portal host window becomes active and stays attached to the
        // browser window. The parent stays hidden, so only the structural
        // portal evidence is asserted (no Window Server hit-test claims).
        client
            .send_ok(&dsl::portal_open_probe())
            .expect("open portal probe");
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush portal frame");
        let with_portal = client.snapshot().expect("portal snapshot");
        assert_eq!(
            with_portal.portal.presentation,
            Some(PROBE_PRESENTATION_ID),
            "portal probe presentation must be active"
        );
        assert!(
            with_portal.portal.child_attached,
            "the portal child window must be attached to the browser window"
        );

        // Close the owning browser window through the production GPUI path.
        client
            .send_ok(&Request::CloseWindow)
            .expect("CloseWindow response must be flushed before teardown");

        let exit = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(
            exit.success(),
            "owning-window close must quit BBR even with a portal child \
             presented, got {exit}"
        );
    }));

    finish(result, &mut harness, &socket, &token);
}
