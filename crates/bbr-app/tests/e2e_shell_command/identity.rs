//! §11.4/§11.5 identity-preservation real-app E2E (task packet
//! `url-address-control-palette`).
//!
//! Opening and closing the field-anchored overlay — by pointer press on the
//! address pill and by Cmd-L — must never change tab, session, profile or
//! space identity, must not disturb the native web view, and must leave the
//! glass geometry exactly where it was (the overlay is chrome over the same
//! content slot, DESIGN.md §13). Committing through the overlay routes the
//! same normalized dispatch boundary as the blank-page fix; an exact-tab
//! destination is a warm switch that reuses the live sessions. Typed
//! snapshots and §15 semantic assertions only — no sleeps.

use super::command_portal;
use super::command_portal::assert_command_portal_open;
use super::*;
use bbr_testkit::protocol::CommandSurface;

#[test]
#[ignore]
fn opening_and_closing_the_overlay_preserves_tab_session_space_and_profile_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-address-id", &page_one);

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
        // Two tabs across the default space; the second is active.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        // Let the window geometry settle before capturing the identity
        // baseline: the harness window frame still adjusts for a few frames
        // after spawn, and the content-slot rect comparison below must
        // compare like for like (the overlay never resizes the window).
        let settled = wait_for_window_settled(&mut client, deadline());
        assert_eq!(settled.active_web_view, Some(second_view));

        // The identity fingerprint that opening/closing the overlay must
        // never change: tab/session/profile/space identity, the native web
        // view host, the glass geometry, and the active context.
        let baseline = client.snapshot().expect("identity baseline");
        let active_view = baseline.active_web_view.expect("active view");
        assert_eq!(active_view, second_view);
        // The tab identity fingerprint: the stable index, the disposable
        // native session (web_view), and the URL. The `lifecycle` resource
        // state is policy, not identity — it legitimately flips warm/active
        // when the overlay commit switches tabs, so it is not part of the
        // fingerprint.
        let tabs_before: Vec<(usize, Option<u64>, String)> = baseline
            .tabs
            .iter()
            .map(|tab| (tab.index, tab.web_view.map(|view| view.0), tab.url.clone()))
            .collect();
        let all_tabs_before: Vec<String> = baseline
            .all_tabs
            .iter()
            .map(|tab| format!("{}:{}", tab.id, tab.space.0))
            .collect();
        let spaces_before: Vec<(u64, u64)> = baseline
            .spaces
            .iter()
            .map(|space| (space.space.0, space.profile.0))
            .collect();
        let profiles_before: Vec<(u64, String)> = baseline
            .profiles
            .iter()
            .map(|profile| (profile.id.0, profile.title.clone()))
            .collect();
        let sessions_before = baseline.engine_sessions;
        let engine_kind_before = baseline.engine_kind.clone();
        let hosted_before = baseline.web_view_hosted;
        let active_space_before = baseline.active_space;
        let active_profile_before = baseline.active_profile;
        let content_rect_before = baseline.content_rect;

        // Pointer activation (the production pill press): the overlay opens
        // over the control and every identity field stays bit-identical.
        client
            .send_ok(&bbr_testkit::Request::ClickAddress)
            .expect("address press");
        since = wait_chrome_focus(&mut client, true, since);
        let open = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Overlay,
            deadline(),
        );
        assert_command_portal_open(&open, CommandSurface::Overlay);
        assert_eq!(open.command_bar_mode, CommandBarMode::Location);
        assert!(open.chrome_focused);
        assert_identity_unchanged(
            &open,
            &tabs_before,
            &all_tabs_before,
            &spaces_before,
            &profiles_before,
            sessions_before,
            &engine_kind_before,
            hosted_before,
            active_space_before,
            active_profile_before,
            content_rect_before,
            active_view,
            "while the overlay is open",
        );

        // Typing goes to the overlay's field; the pill stays display-only.
        type_char(&mut client, 'x');
        let typed = client.snapshot().expect("typed identity snapshot");
        assert_eq!(typed.command_bar_text, "x");
        assert_eq!(
            find(&typed.semantics, "bbr-address")
                .expect("address pill")
                .name
                .as_deref(),
            Some("127.0.0.1"),
            "the pill never holds typed text"
        );

        // Escape restores the previous state, the focus origin (web), and
        // the unchanged identity.
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let closed = client.snapshot().expect("closed identity snapshot");
        assert_eq!(closed.command_surface, CommandSurface::None);
        assert_eq!(closed.command_bar_mode, CommandBarMode::Inactive);
        assert!(!closed.chrome_focused, "the web origin focus returns");
        assert_identity_unchanged(
            &closed,
            &tabs_before,
            &all_tabs_before,
            &spaces_before,
            &profiles_before,
            sessions_before,
            &engine_kind_before,
            hosted_before,
            active_space_before,
            active_profile_before,
            content_rect_before,
            active_view,
            "after Escape closed the overlay",
        );

        // Cmd-L open/close: same invariant on the keyboard path.
        let overlay = cmd_l(&mut client, &mut since);
        assert_command_portal_open(&overlay, CommandSurface::Overlay);
        assert_identity_unchanged(
            &overlay,
            &tabs_before,
            &all_tabs_before,
            &spaces_before,
            &profiles_before,
            sessions_before,
            &engine_kind_before,
            hosted_before,
            active_space_before,
            active_profile_before,
            content_rect_before,
            active_view,
            "while the Cmd-L overlay is open",
        );
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let closed_again = client.snapshot().expect("closed again identity snapshot");
        assert_eq!(closed_again.command_surface, CommandSurface::None);
        assert_eq!(closed_again.command_bar_mode, CommandBarMode::Inactive);
        assert!(!closed_again.chrome_focused);
        assert_identity_unchanged(
            &closed_again,
            &tabs_before,
            &all_tabs_before,
            &spaces_before,
            &profiles_before,
            sessions_before,
            &engine_kind_before,
            hosted_before,
            active_space_before,
            active_profile_before,
            content_rect_before,
            active_view,
            "after the second Escape",
        );

        // Commit through the overlay: Enter routes through the same
        // normalized dispatch boundary (an exact-tab destination activates
        // the page_one tab). Identity outside the intended tab switch stays
        // bit-identical: no tab or session is created or destroyed, spaces
        // and profiles are untouched, and the web view host stays live.
        cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_one.clone(),
            })
            .expect("set destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush destination");
        key(&mut client, "enter");
        let navigated = client.snapshot().expect("navigated identity snapshot");
        assert_eq!(navigated.command_surface, CommandSurface::None);
        assert!(!navigated.chrome_focused, "Enter moves focus into the page");
        assert_eq!(navigated.active_web_view, Some(first_view));
        // The exact-tab activation is a warm switch: the tab/session/space/
        // profile collection is unchanged (only the active tab moved).
        let tabs_after: Vec<(usize, Option<u64>, String)> = navigated
            .tabs
            .iter()
            .map(|tab| (tab.index, tab.web_view.map(|view| view.0), tab.url.clone()))
            .collect();
        assert_eq!(
            tabs_after, tabs_before,
            "no tab is created or destroyed by an overlay commit"
        );
        assert_eq!(
            navigated.engine_sessions, sessions_before,
            "the warm activation reuses the live sessions"
        );
        assert_eq!(
            navigated
                .all_tabs
                .iter()
                .map(|t| format!("{}:{}", t.id, t.space.0))
                .collect::<Vec<_>>(),
            all_tabs_before,
            "persistent tab identity is untouched"
        );
        assert_eq!(
            navigated
                .spaces
                .iter()
                .map(|space| (space.space.0, space.profile.0))
                .collect::<Vec<_>>(),
            spaces_before
        );
        assert_eq!(
            navigated
                .profiles
                .iter()
                .map(|profile| (profile.id.0, profile.title.clone()))
                .collect::<Vec<_>>(),
            profiles_before
        );
        assert_eq!(navigated.active_space, active_space_before);
        assert_eq!(navigated.active_profile, active_profile_before);
        assert_eq!(navigated.engine_kind, engine_kind_before);
        assert_eq!(navigated.web_view_hosted, hosted_before);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Assert every identity field the overlay must never disturb: the active
/// view, the tab/session collection, the persistent tab identities, the
/// space/profile collections, the active context, the native web-view host,
/// and the glass geometry (the overlay is chrome over the same content slot).
#[allow(clippy::too_many_arguments)]
fn assert_identity_unchanged(
    snapshot: &bbr_testkit::protocol::Snapshot,
    tabs_before: &[(usize, Option<u64>, String)],
    all_tabs_before: &[String],
    spaces_before: &[(u64, u64)],
    profiles_before: &[(u64, String)],
    sessions_before: usize,
    engine_kind_before: &str,
    hosted_before: bool,
    active_space_before: bbr_testkit::protocol::TestSpaceId,
    active_profile_before: bbr_web::ProfileId,
    content_rect_before: [f32; 4],
    active_view: bbr_web::WebViewId,
    context: &str,
) {
    let tabs: Vec<(usize, Option<u64>, String)> = snapshot
        .tabs
        .iter()
        .map(|tab| (tab.index, tab.web_view.map(|view| view.0), tab.url.clone()))
        .collect();
    assert_eq!(tabs, tabs_before, "no tab identity change {context}");
    assert_eq!(
        snapshot
            .all_tabs
            .iter()
            .map(|t| format!("{}:{}", t.id, t.space.0))
            .collect::<Vec<_>>(),
        all_tabs_before,
        "no persistent tab identity change {context}"
    );
    assert_eq!(
        snapshot
            .spaces
            .iter()
            .map(|space| (space.space.0, space.profile.0))
            .collect::<Vec<_>>(),
        spaces_before,
        "no space identity change {context}"
    );
    assert_eq!(
        snapshot
            .profiles
            .iter()
            .map(|profile| (profile.id.0, profile.title.clone()))
            .collect::<Vec<_>>(),
        profiles_before,
        "no profile identity change {context}"
    );
    assert_eq!(
        snapshot.engine_sessions, sessions_before,
        "no session created or destroyed {context}"
    );
    assert_eq!(snapshot.engine_kind, engine_kind_before, "{context}");
    assert_eq!(snapshot.web_view_hosted, hosted_before, "{context}");
    assert_eq!(snapshot.active_space, active_space_before, "{context}");
    assert_eq!(snapshot.active_profile, active_profile_before, "{context}");
    assert_eq!(
        snapshot.content_rect, content_rect_before,
        "the native content slot geometry is undisturbed {context}"
    );
    assert_eq!(
        snapshot.active_web_view,
        Some(active_view),
        "the active view is undisturbed {context}"
    );
}
