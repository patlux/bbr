//! Open / empty / dismissal scenarios (task packet `downloads-sidebar-panel`,
//! DESIGN.md §11.12).
//!
//! The fixed footer `[Downloads]` control opens the anchored popover
//! (`bbr-downloads-panel`) confined to the sidebar column; an empty panel
//! shows only the `bbr-downloads-empty` hint (group labels hidden with
//! empty groups, §11.4 region-label rule). Escape, outside pointer press
//! (one pointer action, §13), and re-activation of the footer control all
//! dismiss it, returning focus to the footer control. Opening and operating
//! the panel never changes the active tab/WebView/session/profile/space.

use bbr_testkit::protocol::{CommandBarMode, Request};

use crate::e2e_common::*;
use crate::support::*;

/// Open (footer control + §11.12 ids + empty hint), dismiss via Escape,
/// re-activation toggle, and outside-press dismissal — every step asserting
/// that tab/WebView/session/profile/space identity never changes and no
/// command surface is left behind.
#[test]
#[ignore]
fn footer_open_empty_ids_and_dismissal_preserves_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("downloads-open-dismiss", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let view = initial.active_web_view.expect("active view");
        let sessions = initial.engine_sessions;
        let tabs = initial.tabs.len();
        let spaces = initial.spaces.len();
        let profile = initial.active_profile;
        let space = initial.active_space;
        assert!(!initial.downloads.open, "panel closed at launch");
        // The §11.4 footer control keeps its §15 id at rest.
        assert_ids(&initial, &["bbr-downloads"]);

        // Open: the anchored popover ids, the empty hint, and — because the
        // groups are empty — no group labels (§11.4 region-label rule).
        let opened = open_downloads(&mut client);
        assert!(opened.downloads.open);
        assert_ids(&opened, &["bbr-downloads-panel", "bbr-downloads-empty"]);
        assert!(
            !opened
                .visible_ids
                .iter()
                .any(|id| id == "bbr-downloads-active" || id == "bbr-downloads-history"),
            "empty groups hide their labels"
        );
        assert!(opened.download_rows.is_empty(), "no attempts yet");
        assert!(opened.chrome_focused, "the open panel owns chrome focus");
        assert_eq!(opened.command_bar_mode, CommandBarMode::Inactive);
        assert_panel_identity(&opened, view, sessions, tabs, spaces, profile, space);

        // Escape dismisses and returns focus to the footer control.
        keystroke(&mut client, "escape");
        let dismissed = wait_downloads_closed(&mut client);
        assert!(
            !dismissed
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-downloads-panel")),
            "the panel surface is gone after Escape"
        );
        assert!(
            dismissed.chrome_focused,
            "focus returns to the footer control"
        );
        assert_eq!(dismissed.command_bar_mode, CommandBarMode::Inactive);
        assert_panel_identity(&dismissed, view, sessions, tabs, spaces, profile, space);

        // Re-activation toggle: open again, then a second activation of the
        // footer control dismisses it (and back).
        open_downloads(&mut client);
        client
            .send_ok(&Request::ClickFooterDownloads)
            .expect("re-activate footer downloads");
        let toggled = wait_downloads_closed(&mut client);
        assert!(!toggled.downloads.open, "re-activation dismisses");
        assert_panel_identity(&toggled, view, sessions, tabs, spaces, profile, space);

        // Outside pointer press (§13 one pointer action) dismisses the open
        // panel and returns focus to the footer control.
        open_downloads(&mut client);
        client
            .send_ok(&Request::ClickDownloadsOutside)
            .expect("outside press");
        let outside = wait_downloads_closed(&mut client);
        assert!(!outside.downloads.open, "outside press dismisses");
        assert!(
            outside.chrome_focused,
            "focus returns to the footer control"
        );
        assert_panel_identity(&outside, view, sessions, tabs, spaces, profile, space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
