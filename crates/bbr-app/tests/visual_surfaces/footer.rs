//! Actual sidebar footer dock (Paper S-7b): every state comes from the real
//! app — production spaces, the footer hover listener path, the plus menu,
//! a chip press, a held native download and a real tab drag — never Gallery.
use super::{wait, Client};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe, Harness,
};
use bbr_testkit::protocol::{dsl, Request, Snapshot, ThemeModeOverride};
use std::{fs, path::PathBuf};

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-footer-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-footer-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                "data:text/html,<title>BBR%20Visual%20Footer</title><p>Footer%20fixture</p>",
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
                "--bbr-window-size=1080x1407",
            ],
            &root,
        );
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut client = connect(&socket);
            client.hello(&token).unwrap();
            client.attach(None).unwrap();
            client
                .send_ok(&dsl::set_appearance_override(Some(mode)))
                .unwrap();
            client
                .send_ok(&dsl::set_display_accessibility(
                    Some(false),
                    Some(true),
                    Some(false),
                ))
                .unwrap();
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            client.send_ok(&Request::FocusWeb).unwrap();
            let initial = wait(&mut client, |s| {
                !s.loading
                    && s.title.as_deref() == Some("BBR Visual Footer")
                    && s.tabs.iter().all(|tab| !tab.unresponsive && !tab.crashed)
            });
            dock(&mut client, &output, appearance, &initial);
            client.quit().unwrap();
            crate::wait_for_process_exit(&mut harness, deadline());
        }));
        match result {
            Ok(()) => harness.success = true,
            Err(panic) => {
                harness.diagnostics(&socket, &token);
                std::panic::resume_unwind(panic);
            }
        }
    }
}

fn dock(client: &mut Client, output: &std::path::Path, appearance: &str, initial: &Snapshot) {
    let profile = initial.active_profile;
    for title in ["Reading", "Music"] {
        client.send_ok(&dsl::create_space(profile, title)).unwrap();
    }
    let created = wait(client, |s| s.space_chips.len() == 3);
    // Distinct identities through the production space configuration
    // draft (open → select → Done), so the strip reads like Paper S-7b.
    for (chip, identity) in created.space_chips[1..].iter().zip(["book", "moon"]) {
        client.send_ok(&dsl::open_space_config(chip.space)).unwrap();
        client
            .send_ok(&dsl::space_config_select_identity(identity))
            .unwrap();
        client.send_ok(&dsl::space_config_done()).unwrap();
    }
    client.send_ok(&Request::FocusWeb).unwrap();
    let rest = wait(client, |s| {
        s.space_chips.len() == 3
            && s.space_chips[1].identity == "book"
            && s.space_chips[2].identity == "moon"
            && at_rest(s)
    });
    assert_eq!(
        rest.active_space, initial.active_space,
        "creating never switches"
    );
    save(client, output, "footer-spaces", appearance, rest.clone());
    let reading = rest.space_chips[1].space;

    hover(
        client,
        output,
        appearance,
        "downloads",
        "footer-hover-downloads",
    );
    let chip = format!("space-{}", reading.0);
    hover(client, output, appearance, &chip, "footer-hover-chip");

    client.send_ok(&Request::ClickFooterPlus).unwrap();
    let plus = wait(client, |s| {
        s.footer.plus_open && s.visible_ids.iter().any(|id| id == "bbr-plus-menu")
    });
    save(client, output, "footer-plus-open", appearance, plus);
    client.send_ok(&Request::ClickFooterPlus).unwrap();
    client.send_ok(&Request::FocusWeb).unwrap();
    wait(client, at_rest);

    client
        .send_ok(&dsl::click_space(rest.active_space))
        .unwrap();
    let focused = wait(client, |s| {
        s.footer.strip_focused && s.active_space == rest.active_space && s.footer.hovered.is_none()
    });
    save(client, output, "footer-strip-focus", appearance, focused);
    client.send_ok(&Request::FocusWeb).unwrap();
    wait(client, at_rest);

    running(client, output, appearance);
    drag_target(client, output, appearance, reading);
    overflow(client, output, appearance, profile);
}

/// No hover, nothing open, no focus, no drag: the dock at rest.
fn at_rest(s: &Snapshot) -> bool {
    s.footer.hovered.is_none()
        && !s.footer.plus_open
        && !s.footer.downloads_open
        && !s.footer.strip_focused
        && s.drag_chip.is_none()
}

fn hover(
    client: &mut Client,
    output: &std::path::Path,
    appearance: &str,
    control: &str,
    case: &str,
) {
    client
        .send_ok(&Request::HoverFooterControl {
            control: control.into(),
        })
        .unwrap();
    let hovered = wait(client, |s| s.footer.hovered.as_deref() == Some(control));
    save(client, output, case, appearance, hovered);
    client.send_ok(&Request::UnhoverFooterControl).unwrap();
    wait(client, at_rest);
}

/// A held native download keeps one attempt in flight: the count badge.
fn running(client: &mut Client, output: &std::path::Path, appearance: &str) {
    let (fixtures, _, _) = crate::e2e_common::fixtures();
    let destination = format!("bbr-footer-{appearance}.bin");
    let hold = fixtures
        .hold_download(&fixtures.url("/__fx/dl/known.bin"))
        .unwrap();
    client
        .send_ok(&Request::DownloadStart {
            url: hold.url().into(),
            destination: destination.clone(),
        })
        .unwrap();
    hold.wait_requested().unwrap();
    let badge = wait(client, |s| {
        s.footer.downloads_count == Some(1)
            && s.visible_ids.iter().any(|id| id == "bbr-downloads-count")
            && !s.footer.downloads_open
    });
    save(
        client,
        output,
        "footer-downloads-running",
        appearance,
        badge,
    );
    hold.release().unwrap();
    wait(client, |s| {
        s.footer.downloads_count.is_none()
            && s.download_rows.iter().any(|r| {
                r.filename == destination
                    && r.group == bbr_testkit::protocol::DownloadGroup::History
            })
    });
}

/// A real tab drag hovering another space's chip: the drop-target chip and
/// its destination-profile tag, beside the lifted drag proxy.
fn drag_target(
    client: &mut Client,
    output: &std::path::Path,
    appearance: &str,
    target: bbr_testkit::protocol::TestSpaceId,
) {
    let before = client.snapshot().unwrap();
    let row = before
        .element_rects
        .iter()
        .find(|(id, _)| id == "bbr-tab-row-0")
        .map(|(_, rect)| *rect)
        .expect("the tab row renders");
    let (x, y) = (row[0] + 40.0, row[1] + row[3] / 2.0);
    let index = before
        .space_chips
        .iter()
        .position(|chip| chip.space == target)
        .unwrap();
    let scroll = &before.space_strip_scroll;
    let chip = scroll.chip_bounds[index];
    let centre = (
        chip[0] + scroll.offset[0] + chip[2] / 2.0,
        chip[1] + scroll.offset[1] + chip[3] / 2.0,
    );
    client
        .send_ok(&dsl::begin_tab_drag(0, x, y, row[3] / 2.0))
        .unwrap();
    client.send_ok(&dsl::move_tab_drag(0.0, 6.0)).unwrap();
    client
        .send_ok(&dsl::move_tab_drag(centre.0 - x, centre.1 - y))
        .unwrap();
    let dragging = wait(client, |s| {
        s.drag_chip == Some(target)
            && s.space_chips
                .iter()
                .any(|chip| chip.space == target && chip.drag_target)
    });
    // Paper S-9: the lifted row floats above the strip, so the live chip and
    // its destination tag stay visible.
    let proxy = dragging.drag_offers.proxy_rect.expect("proxy rect");
    assert!(proxy[1] + proxy[3] <= dragging.space_strip_scroll.viewport[1] - 3.5);
    save(client, output, "footer-drag-target", appearance, dragging);
    client.send_ok(&dsl::cancel_tab_drag()).unwrap();
    client.send_ok(&Request::FocusWeb).unwrap();
    wait(client, at_rest);
}

/// Eight spaces overflow the strip; activating the last scrolls its chip
/// fully into view through the production scroll-into-view authority.
fn overflow(
    client: &mut Client,
    output: &std::path::Path,
    appearance: &str,
    profile: bbr_web::ProfileId,
) {
    for title in ["Work", "Notes", "Travel", "Inbox", "Archive"] {
        client.send_ok(&dsl::create_space(profile, title)).unwrap();
    }
    let created = wait(client, |s| s.space_chips.len() == 8);
    for (chip, identity) in
        created.space_chips[3..]
            .iter()
            .zip(["briefcase", "leaf", "compass", "star", "globe"])
    {
        client.send_ok(&dsl::open_space_config(chip.space)).unwrap();
        client
            .send_ok(&dsl::space_config_select_identity(identity))
            .unwrap();
        client.send_ok(&dsl::space_config_done()).unwrap();
    }
    let created = wait(client, |s| {
        s.space_chips.len() == 8 && s.space_chips[7].identity == "globe"
    });
    let last = created.space_chips[7].space;
    client.send_ok(&dsl::activate_space(last)).unwrap();
    client.send_ok(&Request::FocusWeb).unwrap();
    let scrolled = wait(client, |s| {
        let scroll = &s.space_strip_scroll;
        let Some(chip) = scroll.chip_bounds.get(7) else {
            return false;
        };
        let left = chip[0] + scroll.offset[0];
        s.active_space == last
            && at_rest(s)
            && scroll.max_offset[0] > 0.0
            && scroll.offset[0] < 0.0
            && left >= scroll.viewport[0] - 0.5
            && left + chip[2] <= scroll.viewport[0] + scroll.viewport[2] + 0.5
    });
    save(client, output, "footer-overflow", appearance, scrolled);
}

fn save(
    client: &mut Client,
    output: &std::path::Path,
    case: &str,
    appearance: &str,
    before: Snapshot,
) {
    assert!(before
        .tabs
        .iter()
        .all(|tab| !tab.unresponsive && !tab.crashed));
    assert!(before.sidebar_visible);
    assert_eq!(before.window_size, [1080.0, 1407.0]);
    assert!(before.shell_modes.reduced_motion && before.motion.moments.is_empty());
    // Only the dock, plus the overlay a state opens over it.
    let dock = super::Still::Elements(match case {
        "footer-plus-open" => &["bbr-footer", "bbr-plus-menu"],
        "footer-drag-target" => &["bbr-footer", "bbr-tab-drag-proxy"],
        _ => &["bbr-footer"],
    });
    super::save(client, output, case, appearance, dock, before.clone());
    let after = client.snapshot().unwrap();
    assert_eq!(
        after.footer, before.footer,
        "readback preserves the dock state"
    );
    assert_eq!(after.space_chips, before.space_chips);
    assert_eq!(after.drag_chip, before.drag_chip);
}
