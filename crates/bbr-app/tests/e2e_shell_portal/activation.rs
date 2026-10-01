//! Regression: GPUI's PopUp level/all-Spaces defaults must not escape the
//! browser window. These tests use actual AppKit lifecycle transitions.

use super::e2e_common::*;
use super::support::*;
use bbr_testkit::protocol::{CommandSurface, Request, Snapshot};

fn key(client: &mut Client, value: &str) {
    client.send_ok(&dsl::keystroke(value)).expect("key");
}

fn wait(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        flush_frame(client);
        let snapshot = client.snapshot().expect("lifecycle snapshot");
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what}: {:?}", snapshot.portal);
        std::thread::yield_now();
    }
}

fn activate(client: &mut Client) {
    let current = client.snapshot().expect("activation state");
    if current.portal.app_active && current.portal.parent_is_key {
        return;
    }
    client.send_ok(&Request::ActivateApp).expect("activate");
    wait(client, "active owner", |s| {
        s.portal.app_active && s.portal.parent_is_key
    });
}

fn palette(client: &mut Client) -> Snapshot {
    key(client, "cmd-t");
    client
        .send_ok(&Request::SetCommandBarText {
            text: "preserved draft".into(),
        })
        .expect("draft");
    activate(client);
    wait(client, "palette open", |s| {
        s.command_surface == CommandSurface::Palette && s.portal.native_visible
    })
}

fn preserved(snapshot: &Snapshot, open: &Snapshot) {
    assert_eq!(snapshot.command_surface, CommandSurface::Palette);
    assert_eq!(snapshot.command_bar_text, "preserved draft");
    assert_eq!(
        snapshot.portal.portal_window_number,
        open.portal.portal_window_number
    );
    assert!(snapshot.portal.owner_retained);
    if snapshot.portal.native_visible {
        assert!(snapshot.portal.child_attached, "restored native attachment");
    }
    assert!(!snapshot.portal.portal_is_key && !snapshot.portal.portal_is_main);
    assert_ownership_preserved(snapshot, open);
}

#[test]
#[ignore]
fn palette_uses_parent_window_level_and_space_policy() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page, _) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-policy", &page);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());
        activate(&mut client);
        let open = palette(&mut client);
        assert_eq!(
            open.portal.portal_level, open.portal.parent_level,
            "the child must not inherit GPUI's global popup level"
        );
        assert!(
            !open.portal.portal_joins_all_spaces,
            "only the parent owns Space membership"
        );
        wait_for_frontmost_evidence(&mut client, deadline());
        client.quit().expect("quit");
        assert!(wait_for_exit(&mut harness.child, deadline()).success());
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn palette_pauses_on_app_deactivation_and_hide_without_losing_draft() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page, _) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-inactive", &page);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());
        activate(&mut client);
        let open = palette(&mut client);
        // A second real process takes the foreground. NSApplication.deactivate
        // alone can resign the key window while leaving isActive true on macOS
        // 27, so it does not prove the user-visible cross-app transition.
        let (mut other_harness, other_socket, other_token) =
            spawn_harness_shown("portal-other-app", &page);
        let mut other = connect(&other_socket);
        other.hello(&other_token).expect("other hello");
        other.attach(None).expect("other attach");
        wait_for_window_settled(&mut other, deadline());
        assert_ne!(harness.child.id(), other_harness.child.id());
        activate(&mut other);
        let inactive = wait(&mut client, "inactive app hides portal", |s| {
            !s.portal.app_active && !s.portal.native_visible
        });
        preserved(&inactive, &open);
        assert!(!inactive.portal.portal_owns_probe_point);
        activate(&mut client);
        let back = wait(&mut client, "reactivation restores portal", |s| {
            s.portal.native_visible
        });
        preserved(&back, &open);
        client
            .send_ok(&Request::SetAppHidden { hidden: true })
            .expect("hide app");
        let hidden = wait(&mut client, "hidden app hides portal", |s| {
            s.portal.app_hidden && !s.portal.native_visible
        });
        preserved(&hidden, &open);
        client
            .send_ok(&Request::SetAppHidden { hidden: false })
            .expect("unhide app");
        activate(&mut client);
        let back = wait(&mut client, "unhide restores portal", |s| {
            s.portal.native_visible
        });
        preserved(&back, &open);
        // A pending re-show or later activation must not revive a dismissal.
        activate(&mut other);
        wait(&mut client, "inactive again", |s| !s.portal.app_active);
        key(&mut client, "escape");
        activate(&mut client);
        wait(&mut client, "dismissal survives reactivation", |s| {
            s.command_surface == CommandSurface::None && !s.portal.native_visible
        });
        other.quit().expect("other quit");
        assert!(wait_for_exit(&mut other_harness.child, deadline()).success());
        client.quit().expect("quit");
        assert!(wait_for_exit(&mut harness.child, deadline()).success());
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn palette_follows_its_owner_between_two_real_browser_windows() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page, _) = fixtures();
    let (mut harness, socket, token) = spawn_harness_shown("portal-windows", &page);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());
        let first = palette(&mut client);
        let first_id = first.windows.iter().find(|w| w.current).unwrap().window;
        key(&mut client, "cmd-n");
        let second = wait(&mut client, "second browser owner", |s| {
            s.windows.len() == 2
                && s.windows.iter().any(|w| w.current && w.window != first_id)
                && s.portal.parent_is_key
                && s.portal.native_visible
        });
        let second_id = second.windows.iter().find(|w| w.current).unwrap().window;
        client
            .send_ok(&Request::SetCommandBarText {
                text: "second draft".into(),
            })
            .expect("second draft");
        assert_ne!(
            first.portal.portal_window_number,
            second.portal.portal_window_number
        );

        for _ in 0..3 {
            // SelectWindow only redirects testkit; it does NOT activate the
            // native owner. Inspect the background window before activation.
            client
                .send_ok(&Request::SelectWindow { window: first_id })
                .expect("inspect first");
            let paused = wait(&mut client, "first owner is inactive", |s| {
                !s.portal.parent_is_key && !s.portal.native_visible
            });
            preserved(&paused, &first);
            assert!(
                paused.portal.parent_visible,
                "both browsers really stay shown"
            );
            assert!(!paused.portal.portal_owns_probe_point);
            activate(&mut client);
            let restored = wait(&mut client, "first restored", |s| s.portal.native_visible);
            preserved(&restored, &first);
            wait_for_frontmost_evidence(&mut client, deadline());

            client
                .send_ok(&Request::SelectWindow { window: second_id })
                .expect("inspect second");
            let paused = wait(&mut client, "second owner is inactive", |s| {
                !s.portal.parent_is_key && !s.portal.native_visible
            });
            assert!(paused.portal.parent_visible);
            assert!(!paused.portal.portal_owns_probe_point);
            assert_eq!(paused.command_bar_text, "second draft");
            assert_eq!(
                paused.portal.portal_window_number,
                second.portal.portal_window_number
            );
            activate(&mut client);
            let restored = wait(&mut client, "second restored", |s| s.portal.native_visible);
            assert!(restored.portal.child_attached);
            assert_eq!(restored.command_bar_text, "second draft");
            assert_eq!(
                restored.portal.portal_window_number,
                second.portal.portal_window_number
            );
        }
        client.quit().expect("quit");
        assert!(wait_for_exit(&mut harness.child, deadline()).success());
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn hidden_browser_never_exposes_a_standalone_palette() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page, _) = fixtures();
    let (mut harness, socket, token) = spawn_harness("portal-hidden", &page);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());
        key(&mut client, "cmd-t");
        let hidden = wait(&mut client, "logical hidden-parent palette", |s| {
            s.command_surface == CommandSurface::Palette && s.portal.visible
        });
        assert!(!hidden.portal.parent_visible);
        assert!(
            !hidden.portal.native_visible,
            "hidden parent cannot expose an orphan child"
        );
        assert!(!hidden.portal.portal_owns_probe_point);
        client.quit().expect("quit");
        assert!(wait_for_exit(&mut harness.child, deadline()).success());
    }));
    finish(result, &mut harness, &socket, &token);
}
