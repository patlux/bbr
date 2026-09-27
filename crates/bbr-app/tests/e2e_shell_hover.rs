//! Real-app pointer state and typed semantic-tree E2E (Phase-4 closeout).

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::{SemanticRole, TabVisualState};
use e2e_common::*;
use e2e_semantics_common::*;

#[test]
#[ignore]
fn shell_hover_press_focus_and_drag_semantics() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let long_title = fixtures.url("/long-title.html");
    let (mut harness, socket, token) = spawn_harness("hover", &page_one);

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
        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(
            settled.window_size[0] >= SIDEBAR_DEFAULT + 64.0,
            "settled window must fit the default sidebar",
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush first frame");
        let _blank = open_tab(&mut client, &mut since, "about:blank", None);
        let _long = open_tab(&mut client, &mut since, &long_title, Some(LONG_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush pointer setup");

        // §11.3/§11.4: the focused-snapshot assertions below expect the
        // §11.3 space strip node — always part of the rail, so the strip
        // presence seam verifies it (there is no revealable region).
        strip_presence(&mut client);

        // 3. Hover/press through the production listener methods: the row
        // states derive from app state the real pointer paths drive, never a
        // snapshot mutation. Hovering and pressing never select.
        client.send_ok(&dsl::hover_tab_row(1)).expect("hover row 1");
        let hovered = client.snapshot().expect("hovered snapshot");
        assert_eq!(hovered.tab_states[1], TabVisualState::Hover);
        assert_eq!(hovered.tab_states[2], TabVisualState::Selected);
        assert_eq!(hovered.active_tab_index, Some(2), "hover never selects");
        assert_node(
            &hovered.semantics,
            "bbr-tab-state-1",
            SemanticRole::State,
            Some("hover"),
            None,
            true,
            false,
            false,
        );
        // The more specific composed state wins (§10): hovering the selected
        // row keeps `selected`, never `hover`.
        client.send_ok(&dsl::hover_tab_row(2)).expect("hover row 2");
        let over_selected = client.snapshot().expect("hover-selected snapshot");
        assert_eq!(over_selected.tab_states[2], TabVisualState::Selected);
        assert_node(
            &over_selected.semantics,
            "bbr-tab-state-2",
            SemanticRole::State,
            Some("selected"),
            None,
            true,
            false,
            false,
        );
        client
            .send_ok(&dsl::unhover_tab_row())
            .expect("unhover row 2");
        client.send_ok(&dsl::hover_tab_row(0)).expect("hover row 0");
        let reveal = client.snapshot().expect("reveal snapshot");
        // The close control stays present while hovered (the reveal is an
        // opacity transition; the hit square and semantic id are stable).
        assert_node(
            &reveal.semantics,
            "bbr-tab-close-0",
            SemanticRole::Button,
            Some("Close tab"),
            None,
            true,
            false,
            false,
        );
        client
            .send_ok(&dsl::unhover_tab_row())
            .expect("unhover row 0");
        client.send_ok(&dsl::press_tab_row(0)).expect("press row 0");
        let pressed = client.snapshot().expect("pressed snapshot");
        assert_eq!(pressed.tab_states[0], TabVisualState::Active);
        assert_eq!(pressed.active_tab_index, Some(2), "press never selects");
        assert_node(
            &pressed.semantics,
            "bbr-tab-state-0",
            SemanticRole::State,
            Some("active"),
            None,
            true,
            false,
            false,
        );
        client
            .send_ok(&dsl::release_tab_row())
            .expect("release row 0");
        let released = client.snapshot().expect("released snapshot");
        assert_eq!(released.tab_states[0], TabVisualState::Default);
        assert_eq!(released.active_tab_index, Some(2));

        // 4. Focus clarity, typed: pointer selection moves chrome focus to
        // the tab list; the owning list and its selected row expose
        // `focused`, the space list and command field do not. §9 rings are
        // visual; the typed flags are the model assertion.
        client.send_ok(&dsl::click_tab(0)).expect("click row 0");
        let _ = wait_chrome_focus(&mut client, true, since);
        let focused = client.snapshot().expect("focused list snapshot");
        assert_eq!(focused.active_tab_index, Some(0));
        assert_node(
            &focused.semantics,
            "bbr-tab-list",
            SemanticRole::List,
            Some("Tabs"),
            None,
            true,
            false,
            true,
        );
        assert_node(
            &focused.semantics,
            "bbr-tab-row-0",
            SemanticRole::Row,
            Some(PAGE_ONE_TITLE),
            None,
            true,
            true,
            true,
        );
        assert_node(
            &focused.semantics,
            "bbr-space-strip",
            SemanticRole::List,
            Some("Spaces"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &focused.semantics,
            "bbr-address",
            SemanticRole::Button,
            Some("127.0.0.1"),
            None,
            true,
            false,
            false,
        );
        assert!(
            !find(&focused.semantics, "bbr-address")
                .expect("address pill")
                .focused,
            "the §11.5 address pill never carries a focus-ring state"
        );

        // Web focus: the chrome renders at ease (§9) — no node owns focus.
        client
            .send_ok(&bbr_testkit::Request::FocusWeb)
            .expect("focus web");
        let _ = wait_chrome_focus(&mut client, false, since);
        let web_focus = client.snapshot().expect("web focus snapshot");
        assert!(
            !web_focus.semantics[0].focused,
            "web focus renders the chrome at ease"
        );
        assert_node(
            &web_focus.semantics,
            "bbr-tab-list",
            SemanticRole::List,
            Some("Tabs"),
            None,
            true,
            false,
            false,
        );
        assert!(
            !find(&web_focus.semantics, "bbr-tab-row-0")
                .expect("row node")
                .focused,
            "no row owns focus while web content has it"
        );

        // Command focus: Cmd-L opens the §11.4 overlay **over** the §11.5
        // address control; the overlay's own field owns focus and exposes
        // the §9 block cursor (`bbr-command-overlay-cursor`; visible_ids
        // carries the same structural cue). The pill behind it is never a
        // focus-ring target.
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-l".into(),
            })
            .expect("cmd-l");
        let _ = wait_chrome_focus(&mut client, true, since);
        let command_focus = client.snapshot().expect("command focus snapshot");
        assert_ids(&command_focus, &["bbr-command-overlay-cursor"]);
        assert_node(
            &command_focus.semantics,
            "bbr-command-overlay-field",
            SemanticRole::Field,
            Some(page_one.as_str()),
            None,
            true,
            false,
            true,
        );
        assert_node(
            &command_focus.semantics,
            "bbr-command-overlay-cursor",
            SemanticRole::Cursor,
            None,
            None,
            true,
            false,
            true,
        );
        assert!(
            !find(&command_focus.semantics, "bbr-address")
                .expect("address pill")
                .focused,
            "no focus ring on the address pill behind the overlay"
        );
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape command");
        let _ = wait_chrome_focus(&mut client, false, since);

        // 5. A dragged row disables its own controls (§10.9): the close and
        // pin nodes flip `enabled` false while dragging and back after
        // cancel — the model matches the render's disabled surface.
        client
            .send_ok(&dsl::begin_tab_drag(0, 130.0, 400.0, 16.0))
            .expect("begin drag");
        // A plain press only arms the drag (§10.9): the row stays in its
        // selected presentation until the pointer travels the 4 pt
        // activation distance.
        let armed = client.snapshot().expect("armed snapshot");
        assert_eq!(armed.tab_states[0], TabVisualState::Selected);
        client
            .send_ok(&dsl::move_tab_drag(0.0, 6.0))
            .expect("activation move");
        let dragging = client.snapshot().expect("dragging snapshot");
        assert_eq!(dragging.tab_states[0], TabVisualState::Dragging);
        assert!(
            !find(&dragging.semantics, "bbr-tab-close-0")
                .expect("close node")
                .enabled,
            "a dragged row's close is disabled"
        );
        assert!(
            !find(&dragging.semantics, "bbr-tab-pin-0")
                .expect("pin node")
                .enabled,
            "a dragged row's pin is disabled"
        );
        client
            .send_ok(&dsl::cancel_tab_drag())
            .expect("cancel drag");
        let restored = client.snapshot().expect("restored snapshot");
        assert!(
            find(&restored.semantics, "bbr-tab-close-0")
                .expect("close node")
                .enabled,
            "cancel restores the close control"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// System-appearance dark shell (task packet `system-appearance-theme`):
/// the §10.2/§10.3 hover/press wash states are mode-independent — the same
/// state vocabulary, ids and selection rules under the forced-dark token
/// set. The address pill and footer keep their §11.5/§11.4 surfaces.
#[test]
#[ignore]
fn dark_hover_press_wash_states_are_unchanged() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("hover-dark", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        // Force dark before the first asserted frame.
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Dark,
            )))
            .expect("force dark");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush forced dark");
        let initial = client.snapshot().expect("dark initial");
        assert_eq!(initial.shell_modes.theme, "dark");
        // Row 0 must finish loading first: a still-loading row reports
        // `Loading`, which outranks `Hover` in the §10 state order.
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let _blank = open_tab(&mut client, &mut since, "about:blank", None);
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush pointer setup");

        // Hover wash on the unselected first row: `hover` state, the close
        // control reveals, selection never moves (§10.2 in dark).
        client.send_ok(&dsl::hover_tab_row(0)).expect("hover row 0");
        let hovered = client.snapshot().expect("dark hover snapshot");
        assert_eq!(hovered.shell_modes.theme, "dark");
        assert_eq!(hovered.tab_states[0], TabVisualState::Hover);
        assert_eq!(hovered.tab_states[1], TabVisualState::Selected);
        assert_eq!(hovered.active_tab_index, Some(1), "hover never selects");
        assert_node(
            &hovered.semantics,
            "bbr-tab-state-0",
            SemanticRole::State,
            Some("hover"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &hovered.semantics,
            "bbr-tab-close-0",
            SemanticRole::Button,
            Some("Close tab"),
            None,
            true,
            false,
            false,
        );

        // Press wash deepens (§10.3) while the pointer is down (the row is
        // unhovered first so the release restores `default`, matching the
        // light-suite sequence).
        client
            .send_ok(&dsl::unhover_tab_row())
            .expect("unhover row 0");
        client.send_ok(&dsl::press_tab_row(0)).expect("press row 0");
        let pressed = client.snapshot().expect("dark pressed snapshot");
        assert_eq!(pressed.tab_states[0], TabVisualState::Active);
        assert_eq!(pressed.active_tab_index, Some(1), "press never selects");
        client
            .send_ok(&dsl::release_tab_row())
            .expect("release row 0");
        let released = client.snapshot().expect("dark released snapshot");
        assert_eq!(released.tab_states[0], TabVisualState::Default);

        // The address pill and the footer surfaces present in dark with the
        // same §11.5/§11.4 ids.
        assert_ids(
            &released,
            &[
                "bbr-address",
                "bbr-address-domain",
                "bbr-downloads",
                "bbr-space-strip",
                "bbr-plus",
            ],
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
