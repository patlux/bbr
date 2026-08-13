//! Tab fall-through regression (integration-review finding, lane
//! `parallel/l0-rest-perf`).
//!
//! To move the credential sheet's field cursor, the lane added two
//! context-less `tab`/`shift-tab` bindings (`AuthSheetFieldTab`) in
//! `window_seam.rs`. Their handler returns early WITHOUT
//! `stop_propagation()` when no sheet is presented, claiming Tab then "falls
//! through to focus traversal". gpui-ce's keystroke dispatch resolves a
//! keystroke to ALL matching bindings (precedence order; a context-less
//! binding is treated as the deepest context but does not shadow the others)
//! and dispatches each one's action in turn, stopping only when a handler
//! calls `stop_propagation` (`keymap.rs::bindings_for_input`,
//! `window.rs::dispatch_key_down_event`'s binding loop). A no-op handler
//! therefore leaves the loop open and gpui-component `Root`'s
//! element-scoped `tab`/`shift-tab` traversal actions
//! (`vendor/gpui-component/src/root.rs:21-22`) still dispatch.
//!
//! This test drives Tab and Shift-Tab through the production key path with
//! NO credential sheet presented — in the chrome focus domain (location bar
//! open, the domain where the `Root` key context is in the focused node's
//! context stack and the fall-through is actually exercised), in the find
//! field (BBR's other chrome text input), and in the web focus domain — and
//! asserts the pre-existing shell contract holds exactly: the field
//! text/selection/cursor are untouched, chrome focus is retained, the input
//! pipeline stays alive, and the auth contract stays closed. It closes with
//! the positive control: the SAME Tab still moves the sheet's field cursor
//! while the sheet IS presented.

use crate::e2e_common::*;
use crate::e2e_semantics_common::find;
use crate::support::*;
use bbr_testkit::protocol::{CommandBarMode, Snapshot};

/// Drive Tab and Shift-Tab with no credential sheet and assert the
/// pre-existing shell behavior, then prove the sheet still owns Tab.
#[test]
#[ignore]
fn tab_without_sheet_falls_through_to_the_shell_contract() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = auth_fixtures();
    // Short tag: the socket path must stay below the 104-byte `SUN_LEN`
    // macOS limit (TMPDIR + `bbr-st-<tag>-<pid>/profile/bbr-test.sock`).
    let (mut harness, socket, token) = spawn_harness_shown("auth-fall", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        assert_auth_closed(&settled);
        let _ = wait_title(&mut client, view, "BBR Fixture One", since);

        // --- Chrome focus, location bar open, NO sheet. ---
        // Cmd-L through the production key path: the command field owns
        // chrome focus and the full URL is selected (the Cmd-L overlay is a
        // DIFFERENT portal presentation, so the sheet-closed contract is
        // asserted on the auth fields, not on the whole portal).
        key(&mut client, "cmd-l");
        let _ = wait_chrome_focus(&mut client, true, since);
        let open = client.snapshot().expect("location bar open");
        assert_eq!(open.command_bar_mode, CommandBarMode::Location);
        assert!(open.chrome_focused);
        assert_eq!(open.command_bar_text, page_one);
        assert_eq!(open.command_selection, Some([0, page_one.len()]));
        assert_eq!(open.command_cursor, page_one.len());
        assert_command_field_focused(&open);
        assert_auth_closed(&open);

        // Tab with NO sheet: the pre-existing shell behavior must hold
        // exactly — the field text, selection and cursor are untouched,
        // chrome focus is retained, and the auth contract stays closed.
        key(&mut client, "tab");
        let after_tab = client.snapshot().expect("post-tab snapshot");
        assert_auth_closed(&after_tab);
        assert_eq!(after_tab.command_bar_mode, CommandBarMode::Location);
        assert_eq!(
            after_tab.command_bar_text, page_one,
            "Tab must not edit the field text"
        );
        assert_eq!(
            after_tab.command_selection,
            Some([0, page_one.len()]),
            "Tab must not collapse the selection"
        );
        assert_eq!(
            after_tab.command_cursor,
            page_one.len(),
            "Tab must not move the field cursor"
        );
        assert!(
            after_tab.chrome_focused,
            "Tab must not steal chrome focus (the no-sheet handler must not consume the keystroke)"
        );
        assert_command_field_focused(&after_tab);
        assert_eq!(after_tab.auth.awaiting, Vec::<u64>::new());
        assert_eq!(after_tab.auth.outcome, None);

        // Shift-Tab: the identical no-op contract.
        key(&mut client, "shift-tab");
        let after_shift = client.snapshot().expect("post-shift-tab snapshot");
        assert_auth_closed(&after_shift);
        assert_eq!(after_shift.command_bar_text, page_one);
        assert_eq!(after_shift.command_selection, Some([0, page_one.len()]));
        assert_eq!(after_shift.command_cursor, page_one.len());
        assert!(after_shift.chrome_focused);
        assert_command_field_focused(&after_shift);
        assert_eq!(after_shift.auth.awaiting, Vec::<u64>::new());

        // The input pipeline is alive after both keystrokes: typing still
        // replaces the selection and cursor movement still works.
        type_char(&mut client, 'q');
        let typed = client.snapshot().expect("typed after tab");
        assert_eq!(typed.command_bar_text, "q");
        assert_eq!(typed.command_cursor, 1);
        key(&mut client, "left");
        assert_eq!(
            client
                .snapshot()
                .expect("cursor-left after tab")
                .command_cursor,
            0
        );

        // Escape restores the exact origin text and the web focus domain.
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let restored = client.snapshot().expect("restored after tab");
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(!restored.chrome_focused);
        assert_eq!(restored.command_bar_text, page_one);
        assert_auth_closed(&restored);

        // --- Web focus, NO sheet. ---
        // In the web focus domain gpui-component `Root`'s context is not in
        // the focused node's context stack, so Tab resolves only the
        // context-less binding and must be inert: no chrome entry, no sheet.
        key(&mut client, "tab");
        key(&mut client, "shift-tab");
        let web = client.snapshot().expect("web-domain tab snapshot");
        assert!(
            !web.chrome_focused,
            "Tab in web focus must not enter chrome"
        );
        assert_eq!(web.command_bar_mode, CommandBarMode::Inactive);
        assert_auth_closed(&web);
        assert_eq!(web.active_web_view, Some(view));

        // --- Find input, NO sheet. ---
        // The find field is BBR's other chrome text input: Tab (no sheet)
        // must leave the presentation, the field focus, and the query
        // untouched.
        key(&mut client, "cmd-f");
        let find = wait_find_open(&mut client, deadline());
        assert!(find.find.focused, "the find field owns focus");
        assert_eq!(find.find.query, "");
        key(&mut client, "tab");
        key(&mut client, "shift-tab");
        let find_after = client.snapshot().expect("post-tab find snapshot");
        assert!(find_after.find.open, "Tab must not close the find bar");
        assert!(
            find_after.find.focused,
            "Tab must not steal find field focus"
        );
        assert_eq!(find_after.find.query, "");
        assert!(!find_after.find.pending);
        assert_auth_closed(&find_after);
        key(&mut client, "escape");
        let _ = wait_find_closed(&mut client, deadline());

        // --- Positive control: the same Tab still drives the sheet. ---
        ChallengeFixture::for_view(view).present(&mut client);
        let presented = wait_sheet_open(&mut client, view, deadline());
        assert_sheet_open(&presented, view);
        assert_eq!(presented.auth.focused_field.as_deref(), Some("username"));
        key(&mut client, "tab");
        let moved = client.snapshot().expect("sheet tab moved cursor");
        assert_eq!(
            moved.auth.focused_field.as_deref(),
            Some("password"),
            "the sheet still owns Tab while presented"
        );
        key(&mut client, "escape");
        wait_sheet_closed(&mut client, deadline());

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The Address-Control revision removed the in-rail command field. Location
/// editing now lives only in the covering overlay, whose own §15 field node
/// reports the same chrome focus domain. Keep this assertion explicit so the
/// Tab fall-through proof cannot pass by merely leaving `chrome_focused` stale.
fn assert_command_field_focused(snapshot: &Snapshot) {
    assert!(
        find(&snapshot.semantics, "bbr-command-bar").is_none(),
        "the removed in-rail command field must not return"
    );
    let overlay = find(&snapshot.semantics, "bbr-command-overlay")
        .expect("address-editing overlay semantic node");
    assert!(overlay.focused, "the address overlay owns chrome focus");
    let field = overlay
        .children
        .iter()
        .find(|node| node.id == "bbr-command-overlay-field")
        .expect("address overlay field semantic node");
    assert!(field.focused, "the overlay field owns focus");
    assert!(
        field
            .children
            .iter()
            .any(|node| node.id == "bbr-command-overlay-cursor" && node.focused),
        "the focused overlay field exposes its structural cursor"
    );
}

/// The auth-specific "no credential sheet" contract (the command overlay
/// and find bar are separate portal presentations, so this asserts only the
/// auth fields — the sheet must never be open, never be awaiting, and never
/// carry a decision).
fn assert_auth_closed(snapshot: &Snapshot) {
    assert!(!snapshot.auth.open, "credential sheet closed");
    assert_eq!(snapshot.auth.awaiting, Vec::<u64>::new());
    assert_eq!(snapshot.auth.outcome, None);
}

/// Poll until the find presentation is open (the `Cmd-F` action defers into
/// the next update).
fn wait_find_open(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find open snapshot");
        if snapshot.find.open
            && snapshot.portal.visible
            && snapshot.portal.native_visible
            && snapshot.portal.frame_synced
        {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "find did not open");
        std::thread::yield_now();
    }
}

/// Poll until the find presentation is fully dismissed.
fn wait_find_closed(client: &mut Client, deadline: Instant) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("find closed snapshot");
        if !snapshot.find.open
            && snapshot.portal.presentation.is_none()
            && !snapshot.portal.visible
            && !snapshot.portal.native_visible
        {
            assert_auth_closed(&snapshot);
            return snapshot;
        }
        assert!(Instant::now() < deadline, "find did not close");
        std::thread::yield_now();
    }
}
