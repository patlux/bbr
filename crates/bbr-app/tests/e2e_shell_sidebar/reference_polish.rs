//! Reference-polish rest-state evidence (packet
//! `sidebar-reference-polish`): the typed §11.5/§11.14 rest state (address
//! translucent borderless fill, Reload transparent rest, real
//! hovered/pressed presentation state), the §11.4 organized-regions
//! hairline (`bbr-organized-hairline`) with truthful absence while both
//! organized regions are empty and exact Pinned → Folders → hairline →
//! New Tab → Tabs order, and the §11.3/§7 curated footer chips reporting
//! `identity_kind=glyph`. Both appearance modes are asserted through the
//! existing system-appearance seam, semantically (typed wire values), never
//! via pixels.

use super::*;
use crate::e2e_semantics_common::find;
use bbr_testkit::protocol::{dsl, SemanticRole};

fn tab_list_order(snapshot: &bbr_testkit::protocol::Snapshot) -> Vec<String> {
    let list = find(&snapshot.semantics, "bbr-tab-list")
        .unwrap_or_else(|| panic!("semantic tree missing bbr-tab-list"));
    list.children.iter().map(|node| node.id.clone()).collect()
}

fn assert_hairline(snapshot: &bbr_testkit::protocol::Snapshot, present: bool, label: &str) {
    let in_ids = snapshot
        .visible_ids
        .iter()
        .any(|id| id == "bbr-organized-hairline");
    let in_tree = find(&snapshot.semantics, "bbr-organized-hairline").is_some();
    assert_eq!(in_ids, present, "{label}: hairline visible id presence");
    assert_eq!(in_tree, present, "{label}: hairline semantic node presence");
}

/// The typed rest-state and organized-regions hairline evidence over the
/// real app, in both appearance modes: the address pill reports the
/// translucent borderless fill token for the effective chrome polarity,
/// Reload reports the transparent rest surface, hovering/pressing the
/// Reload control flips the real titlebar-model hovered/pressed state and
/// leaving returns it quiet, the hairline stays truthfully absent while
/// neither pinned rows nor folders exist and appears exactly when an
/// organized region gains content (in the settled order), and the footer
/// Space chip reports the curated BbrGlyph identity kind.
#[test]
#[ignore]
fn rest_states_hairline_and_glyph_chips_in_both_appearance_modes() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-reference-polish", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        flush_frame(&mut client);

        // Force the light mode through the existing system-appearance seam
        // so the rest-state assertions are host-independent (the host
        // appearance is not part of the contract).
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Light,
            )))
            .expect("force light");
        flush_frame(&mut client);
        let settled = client.snapshot().expect("settled snapshot");

        // §11.5/§11.14 typed rest state in the effective light mode: the
        // address pill's translucent borderless fill follows the light
        // chrome polarity, Reload rests transparent, and no titlebar
        // control carries a hover/press presentation at rest.
        assert_eq!(
            settled.shell_modes.theme, "light",
            "the forced appearance resolves to light"
        );
        assert_eq!(
            settled.titlebar.address_rest_fill,
            bbr_testkit::protocol::AddressRestFill::BorderedFieldOnLight,
            "§11.5 translucent borderless fill on light chrome"
        );
        assert_eq!(
            settled.titlebar.reload_rest,
            bbr_testkit::protocol::ReloadRestSurface::Transparent,
            "§11.14 Reload rests with no surface"
        );
        assert!(!settled.titlebar.reload_hovered, "quiet at rest");
        assert!(!settled.titlebar.reload_pressed, "quiet at rest");
        assert!(!settled.address_hovered, "address pill quiet at rest");

        // §11.4 hairline: truthfully absent while both organized regions
        // are empty (no pinned rows, no folders) — nothing is fabricated
        // between the active-Space heading and New Tab.
        assert_hairline(&settled, false, "empty organized regions");
        assert!(settled.pinned_count == 0 && settled.folders.is_empty());
        assert!(
            settled.visible_ids.iter().any(|id| id == "bbr-new-tab"),
            "New Tab remains visible without the hairline"
        );

        // §11.3/§7 curated footer chip: the single Space chip renders the
        // drawn BbrGlyph identity form (never the two-letter badge) — the
        // classifier the footer renders with reports `glyph`.
        assert_eq!(settled.space_chips.len(), 1);
        assert_eq!(
            settled.space_chips[0].identity_kind, "glyph",
            "the curated identity renders as a drawn glyph path"
        );

        // An organized region appears: open a second tab so a regular row
        // remains after New Tab, then pin the first tab through the Tab
        // menu's pointer path. The hairline renders exactly then —
        // visible id and semantic Separator node — between the organized
        // Pinned area and the New Tab row, keeping the settled
        // Pinned → Folders → hairline → New Tab → Tabs order.
        let _second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        toggle_pin_via_tab_menu(&mut client, 0);
        since = wait_browser_change(&mut client, BrowserChange::TabPinned { index: 0 }, since);
        flush_frame(&mut client);
        let pinned = client.snapshot().expect("pinned snapshot");
        assert_eq!(pinned.pinned_count, 1);
        assert_hairline(&pinned, true, "pinned rows exist");
        let order = tab_list_order(&pinned);
        let hairline_at = order
            .iter()
            .position(|id| id == "bbr-organized-hairline")
            .expect("hairline in the tab list order");
        let new_tab_at = order
            .iter()
            .position(|id| id == "bbr-new-tab")
            .expect("New Tab in the tab list order");
        assert!(
            hairline_at < new_tab_at,
            "hairline sits before New Tab (§11.4); order: {order:?}"
        );
        assert_eq!(
            order[hairline_at - 1],
            "bbr-tab-row-0",
            "the pinned row precedes the hairline; order: {order:?}"
        );
        assert_eq!(
            order[new_tab_at + 1],
            "bbr-tab-row-1",
            "the first regular row follows New Tab; order: {order:?}"
        );
        let hairline = find(&pinned.semantics, "bbr-organized-hairline").expect("hairline node");
        assert_eq!(
            hairline.role,
            SemanticRole::Separator,
            "typed organized-regions separator role"
        );

        // Dark mode through the existing system-appearance seam: the same
        // typed rest state flips to the dark chrome polarity, the hairline
        // and its separator role stay, and the Reload rest surface stays
        // transparent (the rest-state calibration is mode-independent).
        client
            .send_ok(&dsl::set_appearance_override(Some(
                bbr_testkit::protocol::ThemeModeOverride::Dark,
            )))
            .expect("force dark");
        flush_frame(&mut client);
        let dark = client.snapshot().expect("dark rest-state snapshot");
        assert_eq!(dark.shell_modes.theme, "dark", "dark mode took effect");
        assert_eq!(
            dark.titlebar.address_rest_fill,
            bbr_testkit::protocol::AddressRestFill::BorderedFieldOnDark,
            "§11.5 translucent borderless fill on dark chrome"
        );
        assert_eq!(
            dark.titlebar.reload_rest,
            bbr_testkit::protocol::ReloadRestSurface::Transparent,
            "Reload stays transparent at rest in dark"
        );
        assert!(!dark.titlebar.reload_hovered && !dark.titlebar.reload_pressed);
        assert_hairline(&dark, true, "dark mode keeps the hairline");

        // Reload hover/press through the production titlebar seams: the
        // real titlebar-model presentation state flips and returns quiet.
        client
            .send_ok(&dsl::hover_titlebar_control("reload"))
            .expect("hover reload");
        flush_frame(&mut client);
        let hovered = client.snapshot().expect("reload hovered snapshot");
        assert!(hovered.titlebar.reload_hovered, "§10.2 hover state typed");
        assert!(!hovered.titlebar.reload_pressed);
        assert_eq!(
            hovered.titlebar.address_rest_fill,
            bbr_testkit::protocol::AddressRestFill::BorderedFieldOnDark,
            "address rest stays translucent while Reload hovers"
        );
        client
            .send_ok(&dsl::press_titlebar_control("reload"))
            .expect("press reload");
        flush_frame(&mut client);
        let pressed = client.snapshot().expect("reload pressed snapshot");
        assert!(pressed.titlebar.reload_pressed, "§10.3 pressed state typed");
        client
            .send_ok(&dsl::release_titlebar_control())
            .expect("release reload");
        client
            .send_ok(&dsl::unhover_titlebar_control())
            .expect("unhover reload");
        flush_frame(&mut client);
        let quiet = client.snapshot().expect("reload quiet snapshot");
        assert!(!quiet.titlebar.reload_hovered, "leave returns quiet");
        assert!(!quiet.titlebar.reload_pressed, "release returns quiet");

        // Folder hover/focus-reveal over the real app: create a folder,
        // move the pinned tab into it, hover the folder row — the
        // `type:meta` count reveals through the production hover path and
        // returns quiet on leave (the keyboard half is covered by the
        // folders-region focus unit semantics).
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after folder");
        let with_folder = client.snapshot().expect("folder created snapshot");
        assert_eq!(with_folder.folders.len(), 1);
        let folder_id = with_folder.folders[0].id;
        client
            .send_ok(&dsl::move_tab_into_folder(0, folder_id))
            .expect("move tab into folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after move into folder");
        let with_member = client.snapshot().expect("folder member snapshot");
        assert_eq!(with_member.folders[0].tab_count, 1);
        assert!(
            !with_member.folders[0].count_revealed,
            "quiet at rest (no hover, no focus)"
        );
        client
            .send_ok(&dsl::hover_folder_row(folder_id))
            .expect("hover folder row");
        flush_frame(&mut client);
        let revealed = client.snapshot().expect("folder revealed snapshot");
        assert!(
            revealed.folders[0].count_revealed,
            "§11.11 hover reveals the count"
        );
        client
            .send_ok(&dsl::unhover_folder_row())
            .expect("unhover folder row");
        flush_frame(&mut client);
        let quiet_folder = client.snapshot().expect("folder quiet snapshot");
        assert!(
            !quiet_folder.folders[0].count_revealed,
            "leave returns the count quiet"
        );

        // Back to the host appearance: the typed rest evidence follows the
        // seam — the address fill polarity always matches the effective
        // chrome polarity the snapshot reports.
        client
            .send_ok(&dsl::set_appearance_override(None))
            .expect("clear appearance override");
        flush_frame(&mut client);
        let restored = client.snapshot().expect("restored snapshot");
        let expected_fill = if restored.shell_modes.theme == "dark" {
            bbr_testkit::protocol::AddressRestFill::BorderedFieldOnDark
        } else {
            bbr_testkit::protocol::AddressRestFill::BorderedFieldOnLight
        };
        assert_eq!(
            restored.titlebar.address_rest_fill, expected_fill,
            "the rest fill polarity matches the effective chrome polarity"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
