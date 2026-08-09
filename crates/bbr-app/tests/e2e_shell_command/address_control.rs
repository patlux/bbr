//! §11.5 address-control real-app E2E (task packet
//! `url-address-control-palette`).
//!
//! Over real WKWebViews: the sidebar address is a **display control** — the
//! `bbr-address` pill renders the derived domain only, is never a Field,
//! never carries a focus-ring state in any chrome-focus situation, and
//! never accepts typed text (the overlay's own field is the only text
//! surface). Pointer click on the pill and Cmd-L both open the §11.4
//! overlay **over** the control (top/leading edges flush, the overlay
//! covers the pill), the full URL is preselected so one keystroke replaces
//! it, Enter navigates, and Escape restores the previous state and the
//! origin focus domain. The §10.2/§11.5 hover wash follows the pointer
//! through the production `on_hover` path. Typed-snapshot and §15
//! semantic assertions only — no sleeps, no pixel comparisons.

use super::command_portal;
use super::command_portal::assert_command_portal_open;
use super::*;
use bbr_testkit::protocol::CommandSurface;

#[test]
#[ignore]
fn address_pill_is_display_only_never_focused_and_never_receives_text() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-address", &page_one);

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
        // The fixture URL is `http://127.0.0.1:<port>/page1.html`: the pill
        // shows the domain only — scheme, path, query and port dropped.
        assert_eq!(first.address_domain, "127.0.0.1");

        // At rest the command slot renders the §11.5 pill family and no
        // field: `bbr-address` + domain + reserved accessory, with no
        // `bbr-command-bar`/`-text`/`-cursor` anywhere.
        assert_ids(
            &first,
            &["bbr-address", "bbr-address-domain", "bbr-address-accessory"],
        );
        assert!(
            !first.visible_ids.iter().any(|id| id == "bbr-command-bar"
                || id == "bbr-command-text"
                || id == "bbr-command-cursor"),
            "no sidebar field ids exist at rest"
        );
        let pill = assert_node(
            &first.semantics,
            "bbr-address",
            SemanticRole::Button,
            Some("127.0.0.1"),
            None,
            true,
            false,
            false,
        );
        assert!(
            pill.children
                .iter()
                .any(|node| node.id == "bbr-address-domain"),
            "the domain title is a child of the pill"
        );
        let accessory = find(&first.semantics, "bbr-address-accessory").expect("accessory slot");
        assert_eq!(accessory.role, SemanticRole::Button);
        assert!(
            !accessory.enabled,
            "the v1 accessory slot is empty (DESIGN.md §17.14 open)"
        );

        // Chrome focus (pointer-selected tab row) must never put a focus
        // ring on the pill: the Button reports `focused: false` and no
        // field cursor appears.
        client.send_ok(&dsl::click_tab(0)).expect("chrome focus");
        let _ = wait_chrome_focus(&mut client, true, since);
        let chrome = client.snapshot().expect("chrome focus snapshot");
        assert!(chrome.chrome_focused);
        assert!(
            !find(&chrome.semantics, "bbr-address")
                .expect("address pill")
                .focused,
            "no focus ring on the address pill in a chrome-focused state"
        );
        assert!(
            !chrome
                .visible_ids
                .iter()
                .any(|id| id == "bbr-command-cursor"),
            "no field cursor while chrome owns focus without a surface"
        );

        // §11.5 hover wash through the production `on_hover` path: the
        // pointer over the pill flips the typed hover state (the token pair
        // steps the fill), leaving it restores the resting fill.
        client
            .send_ok(&bbr_testkit::Request::HoverAddress)
            .expect("hover address");
        let hovered = client.snapshot().expect("hovered address snapshot");
        assert!(
            hovered.address_hovered,
            "the pointer over the pill drives the §11.5 hover wash"
        );
        client
            .send_ok(&bbr_testkit::Request::UnhoverAddress)
            .expect("unhover address");
        assert!(
            !client
                .snapshot()
                .expect("unhovered address snapshot")
                .address_hovered,
            "leaving the pill clears the hover wash"
        );

        // Cmd-L: the overlay opens over the control; the pill stays behind
        // it as the same display-only Button (never the editing surface).
        let overlay = cmd_l(&mut client, &mut since);
        assert_command_portal_open(&overlay, CommandSurface::Overlay);
        let behind = find(&overlay.semantics, "bbr-address").expect("pill behind overlay");
        assert_eq!(behind.role, SemanticRole::Button);
        assert_eq!(behind.name.as_deref(), Some("127.0.0.1"));
        assert!(
            !behind.focused,
            "the pill is never a focus-ring target behind the overlay"
        );
        let overlay_field =
            find(&overlay.semantics, "bbr-command-overlay-field").expect("overlay field");
        assert_eq!(overlay_field.role, SemanticRole::Field);
        assert!(
            overlay_field.focused,
            "the overlay's own field owns chrome focus"
        );

        // Typing goes to the overlay's field only: after a real keystroke
        // the pill's name stays the derived domain — the address control
        // never receives typed text.
        type_char(&mut client, 'q');
        let typed = client.snapshot().expect("typed snapshot");
        assert_eq!(
            typed.command_bar_text, "q",
            "the overlay field received the keystroke"
        );
        assert_eq!(
            find(&typed.semantics, "bbr-address")
                .expect("address pill")
                .name
                .as_deref(),
            Some("127.0.0.1"),
            "the pill never holds typed text"
        );

        // Escape restores the previous state and the focus origin: the
        // overlay closes and the chrome tab list (the Cmd-L origin — this
        // flow opened the overlay from a pointer-selected row) owns focus
        // again; a second Escape returns to web.
        key(&mut client, "escape");
        since = wait_chrome_focus(&mut client, true, since);
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(restored.command_surface, CommandSurface::None);
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(
            restored.chrome_focused,
            "the chrome origin focus returns across the overlay"
        );
        assert_eq!(restored.active_web_view, Some(first_view));
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        assert!(
            !client
                .snapshot()
                .expect("web restored snapshot")
                .chrome_focused,
            "a second Escape returns to web"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn pointer_activation_covers_the_control_and_one_keystroke_replaces_then_enter_navigates() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-address-nav", &page_one);

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
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let page_two_key = tab_key(&client.snapshot().expect("address keys"), &page_two, None);
        // The freshly opened page_two tab is active; pointer-select page_one
        // so the address control shows the seeded URL (chrome origin).
        client.send_ok(&dsl::click_tab(0)).expect("select page_one");
        let _ = wait_chrome_focus(&mut client, true, since);
        assert_eq!(first.address_domain, "127.0.0.1");

        // Pointer activation: a press on the address pill opens the overlay
        // **over** the control — top and leading edges flush, the overlay
        // covers the pill — with the full current URL preselected.
        client
            .send_ok(&bbr_testkit::Request::ClickAddress)
            .expect("address press");
        since = wait_chrome_focus(&mut client, true, since);
        let overlay = command_portal::wait_command_portal_open(
            &mut client,
            CommandSurface::Overlay,
            deadline(),
        );
        assert_command_portal_open(&overlay, CommandSurface::Overlay);
        let address_rect = overlay.address_rect.expect("measured pill rect");
        let portal_rect = overlay.portal.rect.expect("resolved overlay rect");
        let flush = |a: f32, b: f32| (a - b).abs() <= 0.5;
        // §11.4: the overlay **covers** the control — top and leading edges
        // flush, and the overlay extends at least to the pill's trailing and
        // bottom edges, so the address control is not visible behind it (the
        // overlay takes its place; it is never a dropdown below it).
        assert!(
            flush(portal_rect[0], address_rect[0])
                && flush(portal_rect[1], address_rect[1])
                && portal_rect[0] + portal_rect[2] >= address_rect[0] + address_rect[2] - 0.5
                && portal_rect[1] + portal_rect[3] >= address_rect[1] + address_rect[3] - 0.5,
            "the overlay covers the control: pill {address_rect:?}, overlay {portal_rect:?}"
        );
        assert_eq!(
            overlay.command_bar_text, page_one,
            "the full URL is present"
        );
        assert_eq!(
            overlay.command_selection,
            Some([0, page_one.len()]),
            "the full URL is preselected"
        );

        // One keystroke replaces the selection (real keystroke through the
        // production key-down handler): the whole URL is gone.
        type_char(&mut client, 'h');
        let replaced = client.snapshot().expect("replaced snapshot");
        assert_eq!(
            replaced.command_bar_text, "h",
            "one keystroke replaced the URL"
        );
        assert_eq!(replaced.command_selection, None);

        // A committed destination navigates: rank the page_two tab and
        // Enter activates it in web focus, closing the surface.
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("set destination");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush destination");
        let exact = client.snapshot().expect("exact tab snapshot");
        assert_eq!(exact.command_results[0].key, page_two_key);
        key(&mut client, "enter");
        let navigated = client.snapshot().expect("navigated snapshot");
        assert_eq!(navigated.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(navigated.command_surface, CommandSurface::None);
        assert!(!navigated.chrome_focused, "Enter moves focus into the page");
        assert_eq!(navigated.active_web_view, Some(second_view));
        assert_eq!(navigated.command_bar_text, page_two);

        // Cmd-L again, then Escape: the previous state is restored exactly
        // and focus returns to its origin (web).
        let overlay_again = cmd_l(&mut client, &mut since);
        assert_command_portal_open(&overlay_again, CommandSurface::Overlay);
        key(&mut client, "escape");
        let _ = wait_chrome_focus(&mut client, false, since);
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(restored.command_surface, CommandSurface::None);
        assert_eq!(restored.command_bar_mode, CommandBarMode::Inactive);
        assert!(!restored.chrome_focused, "the web origin focus returns");
        assert_eq!(restored.active_web_view, Some(second_view));
        assert_eq!(restored.command_bar_text, page_two);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
