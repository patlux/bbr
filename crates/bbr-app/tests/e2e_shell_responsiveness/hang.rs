use crate::*;
use bbr_testkit::protocol::{CommandBarMode, Request, TabVisualState};

#[test]
#[ignore]
fn real_busy_probe_detects_hang_while_heavy_control_answers() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) = start("renderer-detect");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let initial = client.snapshot().expect("initial");
        let first = initial.active_web_view.expect("first view");
        let heavy = fixtures.url(HEAVY_PATH);
        let heavy_view = open_tab(&mut client, &mut since, &heavy, Some("BBR Hang Heavy"));
        let before = client.snapshot().expect("heavy settled");
        client
            .send_ok(&Request::ProbeRendererResponsiveness { view: heavy_view })
            .expect("probe heavy renderer");
        let answered = client
            .wait_for_event_since(
                |event| matches!(event, Event::Web(WebEvent::UnresponsiveChanged(id, false)) if *id == heavy_view),
                since,
                deadline(),
            )
            .expect("heavy renderer answered")
            .0;
        let heavy_state = client.snapshot().expect("heavy responsive snapshot");
        assert!(!heavy_state.tabs[1].unresponsive);
        assert_ne!(heavy_state.tab_states[1], TabVisualState::Unresponsive);
        assert_eq!(heavy_state.engine_sessions, before.engine_sessions);

        client
            .send_ok(&Request::SelectTab { index: 0 })
            .expect("return to first tab");
        navigate(&mut client, &fixtures.url(BUSY_PATH));
        wait_fixture_hit(&fixtures, BUSY_PATH);
        assert_eq!(fixtures.hits(BUSY_PATH), 1, "busy fixture reached WebKit");
        let hung = wait_real_unresponsive(&mut client, first);
        assert_unresponsive_surface(&hung, first);
        assert_eq!(hung.tabs.len(), 2, "tab identity survives timeout");
        assert_eq!(hung.engine_sessions, 2, "hung session remains live");

        // Chrome remains independently interactive while WebContent is stuck.
        key(&mut client, "cmd-l");
        let location = wait_snapshot(&mut client, |snapshot| {
            snapshot.command_bar_mode == CommandBarMode::Location && snapshot.chrome_focused
        });
        assert_eq!(location.active_web_view, Some(first));
        key(&mut client, "escape");
        let _ = answered;
    }));
    finish_case(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn close_switch_sidebar_space_and_split_paths_ignore_hung_renderer() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, mut harness, socket, token, mut client, mut since) = start("renderer-chrome");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let first = client.snapshot().expect("initial").active_web_view.unwrap();
        let second = open_tab(
            &mut client,
            &mut since,
            &fixtures.url("/page2.html"),
            Some(PAGE_TWO_TITLE),
        );
        client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
        navigate(&mut client, &fixtures.url(BUSY_PATH));
        wait_fixture_hit(&fixtures, BUSY_PATH);
        let hung = wait_real_unresponsive(&mut client, first);
        assert_unresponsive_surface(&hung, first);

        // Warm neighbor activation and split selection stay available.
        client.send_ok(&Request::ClickTab { index: 1 }).unwrap();
        let selected = wait_snapshot(&mut client, |s| s.active_web_view == Some(second));
        assert_eq!(selected.engine_sessions, 2);
        client
            .send_ok(&Request::SetSplit {
                primary: 0,
                secondary: 1,
            })
            .unwrap();
        let split = wait_snapshot(&mut client, |s| {
            s.pane_layout.is_some_and(|p| p.secondary.is_some())
        });
        assert_eq!(split.pane_layout.unwrap().focused, second);
        client.send_ok(&Request::CloseSplit).unwrap();

        // Space creation/switch and sidebar toggle are chrome-only paths.
        client
            .send_ok(&Request::CreateSpace {
                profile: selected.active_profile,
                title: "Other".into(),
            })
            .unwrap();
        let (_, created) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let BrowserChange::SpaceCreated { space, .. } = created else {
            unreachable!()
        };
        client.send_ok(&Request::ActivateSpace { space }).unwrap();
        let other = wait_snapshot(&mut client, |s| s.active_space == space);
        assert_eq!(other.active_web_view, None);
        key(&mut client, "cmd-b");
        let toggled = wait_snapshot(&mut client, |s| s.sidebar_visible != other.sidebar_visible);
        assert_eq!(toggled.active_space, space);

        // Return and close through all three normal product paths.
        let original = selected.active_space;
        client
            .send_ok(&Request::ActivateSpace { space: original })
            .unwrap();
        client.send_ok(&Request::SelectTab { index: 0 }).unwrap();
        key(&mut client, "cmd-w");
        let cmd_closed = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(cmd_closed.active_web_view, Some(second));

        let reopened = open_tab(&mut client, &mut since, &fixtures.url(BUSY_PATH), None);
        let _ = wait_real_unresponsive(&mut client, reopened);
        client
            .send_ok(&Request::ClickTabClose { index: 1 })
            .unwrap();
        let row_closed = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(row_closed.active_web_view, Some(second));

        let third = open_tab(&mut client, &mut since, &fixtures.url(BUSY_PATH), None);
        let _ = wait_real_unresponsive(&mut client, third);
        // The sidebar toggle above deliberately collapsed the rail. Restore it
        // through the real Cmd-B action before using the production row
        // pointer path: an unrendered list cannot own GPUI focus or receive a
        // real Backspace keystroke.
        if !client
            .snapshot()
            .expect("pre-focus snapshot")
            .sidebar_visible
        {
            key(&mut client, "cmd-b");
            let _ = wait_snapshot(&mut client, |s| s.sidebar_visible);
        }
        client.send_ok(&Request::ClickTab { index: 1 }).unwrap();
        client.send_ok(&Request::FlushFrame).unwrap();
        let focused = wait_snapshot(&mut client, |s| {
            s.chrome_focused
                && s.active_web_view == Some(third)
                && semantic_focused(s, "bbr-tab-list")
        });
        assert_eq!(focused.active_web_view, Some(third));
        key(&mut client, "backspace");
        let after_backspace = wait_snapshot(&mut client, |s| s.tabs.len() == 1);
        assert_eq!(
            after_backspace.active_web_view,
            Some(second),
            "focused-list Backspace must close the hung row and present its neighbor"
        );
    }));
    finish_case(result, &mut harness, &socket, &token);
}
