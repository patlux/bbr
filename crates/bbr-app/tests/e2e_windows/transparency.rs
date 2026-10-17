//! Settings › Appearance › Sidebar transparency: the slider's production
//! events re-present every browser window at once, the arrow keys step the
//! active slider, 0 % turns the native material off, Reduce Transparency
//! still wins, and a relaunch restores the stored percent. Typed native
//! observations under deadlines; websites stay opaque throughout.

use super::*;
use bbr_testkit::protocol::{ChromeMaterialSnapshot, SettingsInput};

const ROW: &str = "appearance-transparency";

fn material(snapshot: &Snapshot) -> ChromeMaterialSnapshot {
    snapshot
        .content_card
        .chrome_material
        .expect("native chrome material")
}

fn settings(client: &mut Client, input: SettingsInput) {
    client
        .send_ok(&bbr_testkit::Request::SettingsInput { input })
        .expect("settings input");
}

fn key(client: &mut Client, key: &str) {
    settings(
        client,
        SettingsInput::Key {
            key: key.into(),
            text: None,
        },
    );
}

fn row(snapshot: &Snapshot) -> bbr_testkit::protocol::SettingsRowSnapshot {
    snapshot
        .settings
        .as_ref()
        .expect("Settings open")
        .rows
        .iter()
        .find(|row| row.id == ROW)
        .cloned()
        .expect("transparency row")
}

/// The current window's settled glass at `percent` (`row` shows it).
fn at(client: &mut Client, percent: u8) -> Snapshot {
    let label = format!("{percent}%");
    settle(client, &format!("{percent}% transparency"), |s| {
        s.settings.as_ref().is_some_and(|_| row(s).value == label)
            && s.content_card
                .chrome_material
                .is_some_and(|m| m.attached && m.enabled == (percent > 0))
    })
}

fn glass_on(client: &mut Client) {
    client
        .send_ok(&dsl::set_display_accessibility(
            Some(false),
            Some(true),
            Some(false),
        ))
        .expect("glass independent of the machine");
}

#[test]
#[ignore]
fn the_transparency_slider_updates_every_window_live_and_survives_a_relaunch() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two, mut harness, socket, token) =
        start("windows-transparency");
    let profile = harness.root.join("profile");
    let mut stored_alpha = 0.0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        ready(&mut client, &token);
        glass_on(&mut client);
        let first = settle(&mut client, "first window glass", |s| {
            s.content_card
                .chrome_material
                .is_some_and(|m| m.attached && m.enabled)
        });
        let default_alpha = material(&first).tint_alpha;
        let (second, _) = new_window(&mut client);
        keystroke(&mut client, "cmd-,");
        settle(&mut client, "Settings", |s| s.settings.is_some());
        settings(
            &mut client,
            SettingsInput::Pane {
                pane: "appearance".into(),
            },
        );
        let default = at(&mut client, 50);
        assert_eq!(material(&default).tint_alpha, default_alpha);
        assert_eq!(default.settings.as_ref().unwrap().active_slider, None);

        // A press at 80 %: more desktop in the current (second) window …
        settings(
            &mut client,
            SettingsInput::Slide {
                row: ROW.into(),
                value: 80,
            },
        );
        let more = at(&mut client, 80);
        let more_alpha = material(&more).tint_alpha;
        assert!(more_alpha < default_alpha, "{more_alpha} < {default_alpha}");
        assert_eq!(
            more.settings.as_ref().unwrap().active_slider.as_deref(),
            Some(ROW),
            "the pressed slider takes the arrow keys"
        );
        assert!(row(&more).detail.unwrap().contains("wallpaper"));
        // … and in the first window, without touching it.
        select_window(&mut client, 0);
        let first_now = settle(&mut client, "first window re-presented", |s| {
            s.content_card
                .chrome_material
                .is_some_and(|m| m.enabled && m.tint_alpha == more_alpha)
        });
        assert_eq!(material(&first_now).web_view_opaque, Some(true));
        select_window(&mut client, second);

        // Keys step by 5 %, End/Home jump; 0 % is opaque chrome.
        key(&mut client, "right");
        let stepped = at(&mut client, 85);
        assert!(material(&stepped).tint_alpha < more_alpha);
        key(&mut client, "end");
        at(&mut client, 100);
        key(&mut client, "home");
        let off = at(&mut client, 0);
        let off_material = material(&off);
        assert_eq!(off_material.tint_alpha, 1.0);
        assert!(!off_material.card_mask_valid, "opaque corners return");
        assert_eq!(
            row(&off).detail.as_deref(),
            Some("Opaque sidebar and window frame")
        );
        key(&mut client, "right");
        let thin = at(&mut client, 5);
        assert!(material(&thin).tint_alpha < 1.0 && material(&thin).card_mask_valid);
        settings(
            &mut client,
            SettingsInput::Slide {
                row: ROW.into(),
                value: 30,
            },
        );
        let thirty = at(&mut client, 30);
        stored_alpha = material(&thirty).tint_alpha;
        assert!(stored_alpha > default_alpha && stored_alpha < 1.0);

        // Escape releases the keys; a later arrow changes nothing.
        key(&mut client, "escape");
        key(&mut client, "right");
        let released = at(&mut client, 30);
        assert_eq!(released.settings.as_ref().unwrap().active_slider, None);

        // Reduce Transparency still wins over the slider.
        client
            .send_ok(&dsl::set_display_accessibility(
                Some(false),
                Some(true),
                Some(true),
            ))
            .expect("reduce transparency");
        let reduced = settle(&mut client, "accessibility fallback", |s| {
            s.content_card
                .chrome_material
                .is_some_and(|m| !m.enabled && m.tint_alpha == 1.0)
        });
        assert_eq!(row(&reduced).value, "30%");
        assert_eq!(
            row(&reduced).detail.as_deref(),
            Some("Off while Reduce transparency is on in macOS")
        );
        glass_on(&mut client);
        at(&mut client, 30);

        let file =
            std::fs::read_to_string(profile.join("preferences.txt")).expect("preferences written");
        assert!(file.contains("chrome_transparency=30"), "{file}");
        client
            .send_ok(&bbr_testkit::Request::CloseSettings)
            .expect("close Settings");
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);

    // The same profile restores the stored percent in every window.
    let (mut harness, socket, token) = spawn_harness("windows-transparency", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        glass_on(&mut client);
        let restored = settle(&mut client, "restored transparency", |s| {
            s.content_card
                .chrome_material
                .is_some_and(|m| m.enabled && m.tint_alpha == stored_alpha)
        });
        assert_eq!(material(&restored).web_view_opaque, Some(true));
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
