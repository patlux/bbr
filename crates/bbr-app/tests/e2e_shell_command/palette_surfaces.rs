//! §11.4 mutual exclusion and profile-hint clearing real-app E2E (packet U1).
//!
//! The two command-surface presentations — the centered Cmd-T palette and
//! the field-anchored Cmd-L/click overlay — share one engine and one
//! single-state field: at most one is open at a time (opening one replaces
//! the other), Escape restores the exact origin captured at the last entry
//! (including an intermediate overlay origin), and opening either surface
//! clears the §11.3 cross-context profile hint — the hint never coexists
//! with a palette/overlay. Typed-snapshot and §15 semantic assertions, no
//! sleeps.

use super::*;
use bbr_testkit::protocol::CommandSurface;

#[test]
#[ignore]
fn surfaces_are_mutually_exclusive_and_open_clears_the_profile_hint() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-surfaces", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = first.active_space;

        // Mutual exclusion: Cmd-L while the palette is open replaces it with
        // the field-anchored overlay (single-state field, at most one open
        // surface); Cmd-T replaces it back.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let palette = client.snapshot().expect("palette snapshot");
        assert_eq!(palette.command_surface, CommandSurface::Palette);
        key(&mut client, "cmd-l");
        since = wait_chrome_focus(&mut client, true, since);
        let overlay = client.snapshot().expect("overlay snapshot");
        assert_eq!(overlay.command_surface, CommandSurface::Overlay);
        assert_eq!(overlay.command_bar_mode, CommandBarMode::Location);
        assert!(
            !overlay
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-palette")),
            "opening the overlay closes the palette"
        );
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let palette_again = client.snapshot().expect("palette again snapshot");
        assert_eq!(palette_again.command_surface, CommandSurface::Palette);
        assert_eq!(palette_again.command_bar_mode, CommandBarMode::NewTab);
        assert!(
            !palette_again
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-overlay"),
            "opening the palette closes the overlay"
        );
        // Escape restores the origin captured at this Cmd-T — the open
        // overlay's Location surface (chrome focus, page text), never a jump
        // to the page; a second Escape returns to the original web origin.
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, true, since);
        let intermediate = client.snapshot().expect("escape restores overlay origin");
        assert_eq!(intermediate.command_surface, CommandSurface::None);
        assert_eq!(intermediate.command_bar_mode, CommandBarMode::Location);
        assert_eq!(intermediate.command_bar_text, page_one);
        assert!(
            intermediate.chrome_focused,
            "Escape restores the originating chrome focus domain"
        );
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let web = client.snapshot().expect("web restored snapshot");
        assert_eq!(web.command_bar_mode, CommandBarMode::Inactive);
        assert!(!web.chrome_focused, "the original web origin returns");

        // §11.3: opening either surface clears the cross-context profile
        // hint — the hint never coexists with a palette/overlay.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private (cross profile)");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let hinted = client.snapshot().expect("hinted snapshot");
        assert_eq!(
            hinted.command_profile_hint.as_deref(),
            Some("in Profile · Private"),
            "the cross-profile hint is set before the surface opens"
        );

        // Cmd-T: the centered palette opens and clears the hint. The origin
        // at this Cmd-T is chrome (the cross-profile space list focus), so
        // one Escape closes the surface and a second returns to web.
        key(&mut client, "cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let palette_clears = client.snapshot().expect("palette clears hint");
        assert_eq!(palette_clears.command_surface, CommandSurface::Palette);
        assert_eq!(
            palette_clears.command_profile_hint, None,
            "opening the palette clears the §11.3 profile hint"
        );
        assert!(
            !palette_clears
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-profile-hint"),
            "no hint id while the palette is open"
        );
        key(&mut client, "escape");
        let closed = client.snapshot().expect("palette closed snapshot");
        assert_eq!(closed.command_surface, CommandSurface::None);
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);

        // Cmd-L: the field-anchored overlay clears it too.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("back to default (same profile)");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private again");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let hinted_again = client.snapshot().expect("hinted again snapshot");
        assert_eq!(
            hinted_again.command_profile_hint.as_deref(),
            Some("in Profile · Private")
        );
        cmd_l(&mut client, &mut since);
        let overlay_clears = client.snapshot().expect("overlay clears hint");
        assert_eq!(overlay_clears.command_surface, CommandSurface::Overlay);
        assert_eq!(
            overlay_clears.command_profile_hint, None,
            "opening the overlay clears the §11.3 profile hint"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
