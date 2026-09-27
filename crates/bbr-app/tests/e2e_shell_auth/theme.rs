//! The credential sheet follows the shell appearance (task packet
//! `ui-contract-conformance`).
//!
//! Regression: the portal rendered the sheet with a fixed `Theme::dark()`,
//! so a light shell presented a dark sheet. The evidence is the theme the
//! portal render actually drew (`auth.presented_theme`), compared with the
//! shell's §15 `theme` mode under both testkit appearance overrides.

use crate::e2e_common::*;
use crate::support::*;
use bbr_testkit::protocol::Snapshot;

/// Poll until the portal render reports the sheet drawn in `mode`.
fn wait_presented_theme(client: &mut Client, mode: &str, deadline: Instant) -> Snapshot {
    loop {
        // The hidden harness draws the portal window on request only.
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush main and portal frames");
        let snapshot = client.snapshot().expect("auth theme snapshot");
        if snapshot.auth.open && snapshot.auth.presented_theme.as_deref() == Some(mode) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "credential sheet was not drawn in {mode}: {:?}",
            snapshot.auth.presented_theme
        );
        std::thread::yield_now();
    }
}

#[test]
#[ignore]
fn credential_sheet_follows_the_shell_appearance() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = auth_fixtures();
    let (mut harness, socket, token) = spawn_harness("auth-theme", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("first native view");
        assert_sheet_closed(&settled);
        let _ = wait_title(&mut client, view, "BBR Fixture One", since);

        for (mode, over) in [
            ("light", bbr_testkit::protocol::ThemeModeOverride::Light),
            ("dark", bbr_testkit::protocol::ThemeModeOverride::Dark),
        ] {
            client
                .send_ok(&dsl::set_appearance_override(Some(over)))
                .expect("force appearance");
            ChallengeFixture::for_view(view).present(&mut client);
            // `wait_sheet_open` proves the sheet is open for this view and the
            // portal is visible; each round presents a new challenge id.
            let _ = wait_sheet_open(&mut client, view, deadline());
            let drawn = wait_presented_theme(&mut client, mode, deadline());
            assert_eq!(
                drawn.shell_modes.theme, mode,
                "the shell and the sheet share one appearance"
            );
            key(&mut client, "escape");
            let _ = wait_sheet_closed(&mut client, deadline());
        }

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
