//! Real-app Space-configuration E2E (packet
//! `space-creation-appearance-ui.md`, DESIGN.md §11.3): the
//! create → immediate-activate → optional-config lifecycle both creation
//! paths share, the exact-SpaceId/stale-source contract, Escape/cancel
//! keeping the created Space usable, the name/theme/identity commit through
//! the one controller authorities, the icon-only footer chip identity
//! updates with the accessible Space+Profile name, and the truthful
//! restart/persistence boundary (rename persists; curated appearance is
//! session-valid under the current schema).
//!
//! Every interaction is driven through the production seams the surface
//! listeners and the testkit gateway share: plus-menu New Space
//! (`ClickPlusMenuItem`), the keyboard New-Space staging path
//! (`Cmd-Shift-N` + staged name + Enter), the management-affordance entry
//! (`OpenSpaceConfig`), the draft theme/identity selections, real
//! printable-char keystrokes into the rename draft (`key->char` syntax so
//! GPUI's `dispatch_keystroke` carries a `key_char`, exactly like a real
//! keyboard), Backspace, Escape, Done, Cancel, and the exact outside-press
//! dismissal. No sleeps: every step synchronizes on typed snapshot state or
//! semantic events under an explicit deadline. The suite is
//! `#[ignore]`d and GUI-serialized like every other real-app shell suite.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{CommandBarMode, Request, SpaceConfigSnapshot};
use e2e_common::*;

/// Poll snapshots until `space_config` satisfies `predicate` (a typed
/// state wait, exactly like the other suites' snapshot polls). Fails
/// loudly under `deadline()`; each iteration flushes a frame so the poll
/// observes the next production draw instead of sleeping.
fn wait_for_space_config(
    client: &mut Client,
    predicate: impl Fn(Option<&SpaceConfigSnapshot>) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("space-config snapshot");
        if predicate(snapshot.space_config.as_ref()) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "space-configuration state never matched under deadline"
        );
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush frame while waiting for space-config state");
    }
}

/// The open configuration surface bound to the exact persistent `space`.
fn open_config_for(
    client: &mut Client,
    space: bbr_testkit::protocol::TestSpaceId,
) -> bbr_testkit::protocol::Snapshot {
    wait_for_space_config(client, |config| {
        config.is_some_and(|config| config.space == space)
    })
}

/// The configuration surface closed (not rendered).
fn closed_config(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    wait_for_space_config(client, |config| config.is_none())
}

/// Dispatch one keystroke through GPUI's real input path. Printable chars
/// use the `key->char` syntax so the produced `Keystroke` carries a
/// `key_char` exactly like a real keyboard (the app's chrome key handler
/// edits the rename draft from `key_char`).
fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Replace the rename draft with `name`: Backspace the current draft
/// (read from the typed snapshot), then type the new name char by char
/// through the production keyboard seam.
fn type_rename_draft(client: &mut Client, name: &str) {
    let current = client
        .snapshot()
        .expect("draft snapshot")
        .space_config
        .expect("config surface open")
        .name;
    for _ in current.chars() {
        key(client, "backspace");
    }
    for ch in name.chars() {
        match ch {
            ' ' => key(client, "space-> "),
            other => {
                let k = format!("{other}->{other}");
                key(client, &k);
            }
        }
    }
}

/// Assert the typed chip evidence: exactly one chip renders selected and
/// that chip is exactly the active space.
fn assert_chip_evidence(
    snapshot: &bbr_testkit::protocol::Snapshot,
    active: bbr_testkit::protocol::TestSpaceId,
) {
    assert_eq!(
        snapshot.space_chips.iter().filter(|c| c.selected).count(),
        1,
        "exactly one chip renders selected"
    );
    let chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.selected)
        .expect("selected chip");
    assert_eq!(
        chip.space, active,
        "the selected chip is exactly the active space"
    );
}

/// The footer chip of `space`, and its accessible name from the semantic
/// tree (the icon-only chip carries the Space and Profile names).
fn chip_of(
    snapshot: &bbr_testkit::protocol::Snapshot,
    space: bbr_testkit::protocol::TestSpaceId,
) -> &bbr_testkit::protocol::SpaceChipSnapshot {
    snapshot
        .space_chips
        .iter()
        .find(|c| c.space == space)
        .expect("space chip in the strip")
}

/// Plus → New Space creates and activates a real Space immediately, opens
/// its optional name/icon configuration surface bound to the exact new
/// SpaceId, and Cancel keeps the created Space usable (the §11.3
/// creation-commits-before-configuration contract).
#[test]
#[ignore]
fn plus_menu_new_space_creates_activates_and_configures_then_cancel_keeps_it() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-config-create", &page_one);

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
        let default_space = initial.active_space;
        let sessions_before = initial.engine_sessions;

        // §11.3/§11.4: the permanent space strip is always part of the rail.
        strip_presence(&mut client);

        // Plus menu → New Space: a real space is created in the active
        // profile and activated immediately (the standing daily-driver
        // decision), then the configuration surface opens on top of the
        // usable Space.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(0))
            .expect("plus New Space");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let created_id = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };

        // The new Space is active immediately and the config surface is
        // open bound to its exact persistent id (create → activate →
        // configure; creation committed before configuration).
        let after_create = open_config_for(&mut client, created_id);
        assert_eq!(
            after_create.active_space, created_id,
            "the created space activates immediately"
        );
        let created_space = after_create
            .spaces
            .iter()
            .find(|s| s.space == created_id)
            .expect("created space in the model");
        assert_eq!(
            created_space.title, "New Space",
            "deterministic default title"
        );
        let config = after_create
            .space_config
            .as_ref()
            .expect("config surface open");
        assert_eq!(config.space, created_id, "surface bound to the exact id");
        assert_eq!(config.name, "New Space", "draft prefill is the title");
        assert!(config.focused, "the rename field owns chrome focus");
        assert!(
            after_create
                .visible_ids
                .iter()
                .any(|id| id == "bbr-space-config"),
            "the production surface renders while open"
        );
        assert!(
            after_create
                .visible_ids
                .iter()
                .any(|id| id == "bbr-space-config-done")
                && after_create
                    .visible_ids
                    .iter()
                    .any(|id| id == "bbr-space-config-cancel"),
            "the Done/Cancel controls render"
        );
        assert_eq!(
            after_create.engine_sessions, sessions_before,
            "creating a Space never recreates sessions"
        );
        assert_chip_evidence(&after_create, created_id);

        // Cancel keeps the created Space and its valid defaults usable.
        client
            .send_ok(&dsl::space_config_cancel())
            .expect("cancel config");
        let cancelled = closed_config(&mut client);
        assert_eq!(
            cancelled.active_space, created_id,
            "cancel keeps the created space active"
        );
        assert!(
            cancelled.spaces.iter().any(|s| s.space == created_id),
            "cancel never rolls back creation"
        );
        assert!(
            cancelled
                .visible_ids
                .iter()
                .any(|id| id == &format!("bbr-space-chip-{}", created_id.0)),
            "the created space's footer chip renders after cancel"
        );
        assert_eq!(
            cancelled.spaces.len(),
            2,
            "configuration never creates a second space"
        );
        assert_chip_evidence(&cancelled, created_id);
        assert_eq!(
            cancelled.spaces.iter().filter(|s| s.active).count(),
            1,
            "exactly one space is active"
        );
        // The default space is untouched.
        assert_eq!(
            cancelled
                .spaces
                .iter()
                .find(|s| s.space == default_space)
                .expect("default space")
                .title,
            "Default"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Done commits the rename draft, the curated theme, and the curated
/// identity for the exact bound SpaceId only: the footer chip and the
/// semantic name update without session recreation, and no sibling Space
/// mutates.
#[test]
#[ignore]
fn done_commits_rename_theme_and_identity_to_the_exact_space_only() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-config-commit", &page_one);

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
        let default_space = initial.active_space;
        let sessions_before = initial.engine_sessions;

        // Keyboard create: Cmd-Shift-N stages a name, Enter commits and
        // activates the fresh space and opens its config surface.
        key(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set space name");
        key(&mut client, "enter");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        let open = open_config_for(&mut client, reading);
        assert_eq!(open.space_config.as_ref().expect("open").name, "Reading");

        // Real keystrokes edit the rename draft through the production
        // keyboard seam: Backspace clears the prefill, printable chars type
        // the new name, then the curated theme/identity selections.
        type_rename_draft(&mut client, "Work notes");
        client
            .send_ok(&dsl::space_config_select_theme("pine"))
            .expect("select pine theme");
        client
            .send_ok(&dsl::space_config_select_identity("🌊"))
            .expect("select wave emoji");
        let draft = client
            .snapshot()
            .expect("draft snapshot")
            .space_config
            .expect("config open")
            .clone();
        assert_eq!(draft.name, "Work notes", "the keyboard draft is exact");
        assert_eq!(draft.theme, "pine", "the draft theme is exact");
        assert_eq!(draft.identity, "🌊", "the draft identity is exact");

        // Done commits the draft for the exact bound SpaceId.
        client.send_ok(&dsl::space_config_done()).expect("done");
        let committed = closed_config(&mut client);
        assert_eq!(
            committed.active_space, reading,
            "Done never switches the active space"
        );
        assert_eq!(
            committed.engine_sessions, sessions_before,
            "identity/rename commits never recreate sessions"
        );
        let renamed = committed
            .spaces
            .iter()
            .find(|s| s.space == reading)
            .expect("reading space");
        assert_eq!(renamed.title, "Work notes", "the rename is committed");
        assert_eq!(
            committed
                .spaces
                .iter()
                .find(|s| s.space == default_space)
                .expect("default space")
                .title,
            "Default",
            "renaming never mutates a sibling space"
        );
        // The footer chip renders the committed identity/theme/title.
        let chip = chip_of(&committed, reading);
        assert_eq!(chip.title, "Work notes");
        assert_eq!(chip.identity, "🌊");
        assert_eq!(chip.theme, "pine");
        let default_chip = chip_of(&committed, default_space);
        assert_eq!(default_chip.identity, "code", "sibling identity untouched");
        assert_eq!(default_chip.theme, "cobalt", "sibling theme untouched");
        assert_chip_evidence(&committed, reading);
        // The visible ids still carry the persistent-space chip ids.
        assert!(
            committed
                .visible_ids
                .iter()
                .any(|id| id == &format!("bbr-space-chip-{}", reading.0)),
            "committed chip id present"
        );

        // The icon-only footer still switches spaces with warm sessions
        // (the rename/identity never changed the switch authority): a
        // pointer activation of the default space presents its warm tab
        // without session recreation.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("switch back to default");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let switched = client.snapshot().expect("switched snapshot");
        assert_eq!(switched.active_space, default_space);
        assert_eq!(
            switched.engine_sessions, sessions_before,
            "switching after configuration never recreates sessions"
        );
        assert_chip_evidence(&switched, default_space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Escape and the §13 outside press both cancel without applying the
/// draft; the created Space and its valid defaults stay usable.
#[test]
#[ignore]
fn escape_and_outside_press_cancel_without_applying() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-config-cancel", &page_one);

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

        // Create a space and open its config surface.
        key(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set space name");
        key(&mut client, "enter");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        open_config_for(&mut client, reading);

        // A draft edit, then the real Escape key: the global
        // `escape → FocusWeb` binding routes to the app's cancel path,
        // which closes the surface without applying.
        type_rename_draft(&mut client, "Work notes X");
        key(&mut client, "escape");
        let escaped = closed_config(&mut client);
        assert_eq!(
            escaped
                .spaces
                .iter()
                .find(|s| s.space == reading)
                .expect("reading space")
                .title,
            "Reading",
            "Escape never applies the draft"
        );
        assert_eq!(
            escaped.active_space, reading,
            "Escape keeps the created space active"
        );
        assert!(
            escaped.command_bar_mode == CommandBarMode::Inactive,
            "Escape restores the inactive command surface"
        );

        // Reopen the config (management-affordance entry) and dismiss with
        // an outside press: the exact §13 one-pointer-action dismissal.
        client
            .send_ok(&dsl::open_space_config(reading))
            .expect("reopen config");
        open_config_for(&mut client, reading);
        type_rename_draft(&mut client, "Never applied");
        client
            .send_ok(&dsl::space_config_press_outside())
            .expect("outside press");
        let outside = closed_config(&mut client);
        assert_eq!(
            outside
                .spaces
                .iter()
                .find(|s| s.space == reading)
                .expect("reading space")
                .title,
            "Reading",
            "outside press never applies the draft"
        );
        assert_eq!(
            outside.active_space, reading,
            "outside press keeps the created space usable"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The stale-source contract: the surface captures its bound SpaceId at
/// open and never re-derives it, so a Done after switching spaces commits
/// to the bound Space — never to the newly active one.
#[test]
#[ignore]
fn stale_surface_never_mutates_another_space() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-config-stale", &page_one);

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
        let default_space = initial.active_space;

        // Create "Reading" — the config surface opens bound to it.
        key(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set space name");
        key(&mut client, "enter");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        open_config_for(&mut client, reading);

        // Switch to the default space while the surface stays open: the
        // bound id is captured at open and never re-derived, so the surface
        // is still bound to Reading.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("activate default while config is open");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let after_switch = client.snapshot().expect("after switch snapshot");
        assert_eq!(after_switch.active_space, default_space);
        assert_eq!(
            after_switch
                .space_config
                .as_ref()
                .expect("still open")
                .space,
            reading,
            "the bound SpaceId is captured at open and never re-derived"
        );

        // Done commits to the bound Reading space; the newly active default
        // space is untouched.
        client
            .send_ok(&dsl::space_config_select_theme("violet"))
            .expect("select violet theme");
        client
            .send_ok(&dsl::space_config_done())
            .expect("done on the stale-bound surface");
        let committed = closed_config(&mut client);
        assert_eq!(
            committed.active_space, default_space,
            "the switch stays the active space"
        );
        assert_eq!(
            chip_of(&committed, reading).theme,
            "violet",
            "the bound space received the committed theme"
        );
        assert_eq!(
            chip_of(&committed, default_space).theme,
            "cobalt",
            "the newly active space never mutated"
        );
        assert_eq!(
            committed
                .spaces
                .iter()
                .find(|s| s.space == default_space)
                .expect("default space")
                .title,
            "Default",
            "the newly active space's title never mutated"
        );
        assert_chip_evidence(&committed, default_space);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Truthful restart/persistence boundary under the current schema: the
/// committed rename persists across a graceful restart (the structural
/// browser model carries the title through the persistence DTO), while the
/// curated theme/identity is session-valid only — the persisted rows store
/// the canonical derived defaults and restore drops tint/glyph, so the
/// derived appearance returns. The E2E asserts exactly that boundary and
/// never a false durable claim.
#[test]
#[ignore]
fn restart_truth_rename_persists_curated_identity_is_session_valid() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("spaces-config-restart", pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();

    let (mut harness, socket, token) = spawn_harness("spaces-config-restart", &page_one);
    // The created SpaceId is captured in leg 1 and reused by the restart
    // leg: restore preserves the persisted identity graph.
    let mut reading = bbr_testkit::protocol::TestSpaceId(0);
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

        // Create + configure: rename to "Work notes", curated pine + wave.
        key(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set space name");
        key(&mut client, "enter");
        let (_seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        open_config_for(&mut client, reading);
        type_rename_draft(&mut client, "Work notes");
        client
            .send_ok(&dsl::space_config_select_theme("pine"))
            .expect("select pine theme");
        client
            .send_ok(&dsl::space_config_select_identity("🌊"))
            .expect("select wave emoji");
        client.send_ok(&dsl::space_config_done()).expect("done");
        let committed = closed_config(&mut client);
        assert_eq!(
            chip_of(&committed, reading).identity,
            "🌊",
            "the curated identity is presented in this session"
        );
        assert_eq!(chip_of(&committed, reading).theme, "pine");

        // Deterministic flush of the committed structural model, then a
        // graceful exit (the profile directory is kept for the restart).
        client
            .send_ok(&Request::FlushBrowserState)
            .expect("flush browser state");
    }));
    // Keep the profile directory for the restart leg (a successful finish
    // would let the harness Drop remove the root).
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the rename survived (the
    // structural model carries titles through the persistence DTO), while
    // the curated appearance did not — the restored effective appearance is
    // the canonical derived default (profile hue theme + default glyph),
    // exactly the truthful boundary the current schema provides.
    let (mut harness2, socket2, token2) = spawn_harness("spaces-config-restart", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("restart snapshot");
        let renamed = restored
            .spaces
            .iter()
            .find(|s| s.title == "Work notes")
            .expect("the renamed space restored with its persisted title");
        assert_eq!(renamed.space, reading, "the persisted SpaceId is restored");
        assert_eq!(
            renamed.title, "Work notes",
            "the committed rename persists across restart"
        );
        // The curated identity/theme are session-valid: the derived default
        // glyph and the profile-derived hue theme are presented instead.
        let chip = chip_of(&restored, reading);
        assert_eq!(
            chip.identity, "code",
            "the curated identity is session-valid (derived default after restart)"
        );
        assert_eq!(
            chip.theme, "cobalt",
            "the curated theme is session-valid (profile-derived after restart)"
        );
        // No config surface opens on restore, and the default space is
        // intact.
        assert_eq!(restored.space_config, None, "no surface on restore");
        assert!(
            restored
                .visible_ids
                .iter()
                .any(|id| id == &format!("bbr-space-chip-{}", reading.0)),
            "the restored space's chip renders in the footer"
        );
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}
