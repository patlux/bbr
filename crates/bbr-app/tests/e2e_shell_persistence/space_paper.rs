//! v2 S10 curated space paper across restart (task packet
//! `v2-theme-picker`): the §11.3 configuration surface edits the tint
//! intensity and the static grain as a draft, Cancel keeps the committed
//! values, Done applies them (the resolved rail tint is the chosen step),
//! and a graceful restart restores the whole curated appearance — theme,
//! intensity and grain — from the schema-v4 store.

use std::path::Path;

use bbr_testkit::protocol::{SpaceAppearanceSnapshot, ThemeModeOverride};

use crate::*;

/// The resolved dark/standard-contrast rail tint for a token and step
/// through the one public presentation authority.
fn dark_chrome(token: bbr_domain::SpaceTheme, step: bbr_domain::TintIntensity) -> String {
    let theme = bbr_ui::theme::Theme::new(
        bbr_ui::theme::ThemeMode::Dark,
        bbr_ui::theme::ContrastMode::Standard,
        bbr_ui::theme::Density::Normal,
    );
    bbr_ui::theme::hex(bbr_ui::theme::space_tint(token, step, &theme).chrome)
}

fn appearance(client: &mut Client) -> SpaceAppearanceSnapshot {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
    client
        .snapshot()
        .expect("appearance snapshot")
        .active_appearance
        .expect("active appearance")
}

fn force_dark(client: &mut Client) {
    client
        .send_ok(&dsl::set_appearance_override(Some(ThemeModeOverride::Dark)))
        .expect("force dark");
}

/// Keep the profile directory for the next launch and rethrow a failure.
fn keep_root(result: std::thread::Result<()>, harness: &mut Harness, socket: &Path, token: &str) {
    match result {
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

#[test]
#[ignore]
fn space_paper_edits_as_a_draft_commits_and_survives_restart() {
    use bbr_domain::{SpaceTheme, TintIntensity};
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let profile = harness_root("persist-paper", pid).join("profile");
    seed_browser_state(&profile, &small_browser_state(&page_one));

    let (mut harness, socket, token) = spawn_harness("persist-paper", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        force_dark(&mut client);
        let initial = appearance(&mut client);
        let space = initial.space;
        assert_eq!(
            (
                initial.theme.as_str(),
                initial.intensity.as_str(),
                initial.grain
            ),
            ("cobalt", "standard", false),
            "deterministic defaults"
        );
        assert_eq!(
            initial.chrome_tint,
            dark_chrome(SpaceTheme::Cobalt, TintIntensity::Standard)
        );

        // Draft only: the surface shows the edits, the rail does not.
        client
            .send_ok(&dsl::open_space_config(space))
            .expect("open config");
        client
            .send_ok(&dsl::space_config_select_intensity("rich"))
            .expect("draft rich");
        client
            .send_ok(&dsl::space_config_toggle_grain())
            .expect("draft grain");
        let draft = client.snapshot().expect("draft snapshot");
        let surface = draft.space_config.as_ref().expect("surface open");
        assert_eq!((surface.intensity.as_str(), surface.grain), ("rich", true));
        assert!(draft
            .visible_ids
            .iter()
            .any(|id| id == "bbr-space-config-intensity-rich"));
        assert!(draft
            .visible_ids
            .iter()
            .any(|id| id == "bbr-space-config-grain"));
        assert_eq!(appearance(&mut client), initial, "a draft never applies");

        // Cancel keeps the committed values; a fresh open shows them.
        client.send_ok(&dsl::space_config_cancel()).expect("cancel");
        assert_eq!(appearance(&mut client), initial, "cancel keeps the paper");
        client
            .send_ok(&dsl::open_space_config(space))
            .expect("reopen config");
        let reopened = client.snapshot().expect("reopened snapshot");
        let surface = reopened.space_config.as_ref().expect("surface open");
        assert_eq!(
            (surface.intensity.as_str(), surface.grain),
            ("standard", false),
            "the draft restarts from the committed paper"
        );

        // Done applies theme + paper; the rail resolves the chosen step.
        client
            .send_ok(&dsl::space_config_select_theme("pine"))
            .expect("draft pine");
        client
            .send_ok(&dsl::space_config_select_intensity("rich"))
            .expect("draft rich");
        client
            .send_ok(&dsl::space_config_toggle_grain())
            .expect("draft grain");
        client.send_ok(&dsl::space_config_done()).expect("done");
        let committed = appearance(&mut client);
        assert_eq!(
            (
                committed.theme.as_str(),
                committed.intensity.as_str(),
                committed.grain
            ),
            ("pine", "rich", true)
        );
        assert_eq!(
            committed.chrome_tint,
            dark_chrome(SpaceTheme::Pine, TintIntensity::Rich)
        );
        assert_ne!(
            committed.chrome_tint,
            dark_chrome(SpaceTheme::Pine, TintIntensity::Standard),
            "rich is a distinct rail"
        );

        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    // Restart: the schema-v4 store restores theme, intensity and grain.
    let (mut harness2, socket2, token2) = spawn_harness("persist-paper", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        force_dark(&mut client);
        let restored = appearance(&mut client);
        assert_eq!(
            (
                restored.theme.as_str(),
                restored.intensity.as_str(),
                restored.grain
            ),
            ("pine", "rich", true),
            "the curated paper restored"
        );
        assert_eq!(
            restored.chrome_tint,
            dark_chrome(SpaceTheme::Pine, TintIntensity::Rich)
        );
        let snapshot = client.snapshot().expect("restored snapshot");
        assert!(
            matches!(snapshot.restore, RestoreDisposition::Restored { .. }),
            "restored from the store: {:?}",
            snapshot.restore
        );
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}
