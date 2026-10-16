//! Portal-child stills (formerly withheld, 2026-10-12): Peek's bar, the
//! permission prompt and site settings, the client-certificate sheet, the
//! open-in-app sheet and no-app banner, the credential sheet, and the
//! populated Settings › Privacy lists they leave behind. Requests arrive
//! through the engine's own event transport (as the WebKit delegate emits
//! them) or real fixture navigation; every prompt names the fixture host
//! without its port, so pixels never depend on the ephemeral server.
use super::{key, save, save_portal, Client, Still};
use crate::e2e_common::{
    catch_unwind, connect, deadline, harness_root, harness_socket_path, AssertUnwindSafe,
    FixtureServer, Harness, WebEvent, PAGE_ONE_TITLE,
};
use bbr_testkit::protocol::Snapshot;
use bbr_testkit::protocol::{
    dsl, ClientCertificateInput, OpenAppInput, PermissionInput, Request, SettingsInput,
    ThemeModeOverride,
};
use bbr_web::client_certificates::ClientIdentityInfo;
use bbr_web::new_window::{NewWindowInput, NewWindowIntent};
use bbr_web::permissions::PermissionCapability;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Tuesday 2026-09-29 12:00 UTC: certificate expiry dates read this clock.
const NOW_MS: u64 = 1_790_683_200_000;

fn identities() -> Vec<ClientIdentityInfo> {
    vec![
        ClientIdentityInfo {
            fingerprint: "c".repeat(64),
            name: "Patrick W (old)".into(),
            issuer: "Corp Issuing CA 1".into(),
            // 2 Feb 2026, 12:00 UTC.
            expires_unix: Some(1_770_033_600),
        },
        ClientIdentityInfo {
            fingerprint: "a".repeat(64),
            name: "Patrick Wozniak".into(),
            issuer: "Corp Issuing CA 2".into(),
            // 3 Mar 2037, 12:00 UTC.
            expires_unix: Some(2_119_780_800),
        },
    ]
}

pub fn capture() {
    let _serial = crate::e2e_common::serial_gui();
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    for (appearance, mode) in [
        ("light", ThemeModeOverride::Light),
        ("dark", ThemeModeOverride::Dark),
    ] {
        let tag = format!("visual-portal-{appearance}");
        let root = harness_root(&tag, std::process::id());
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        let socket = harness_socket_path(&tag, std::process::id());
        let token = format!("visual-portal-{}", std::process::id());
        let output = PathBuf::from(std::env::var_os("BBR_VISUAL_CAPTURE_DIR").unwrap());
        fs::create_dir_all(&output).unwrap();
        let bin = std::env::var("BBR_VISUAL_CAPTURE_BIN").unwrap();
        let mut harness = Harness::spawn(
            &bin,
            &[
                "--bbr-initial-url",
                &page_one,
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
                .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
                .unwrap();
            client
                .send_ok(&Request::SetWindowSize {
                    width: 1080.0,
                    height: 1407.0,
                })
                .unwrap();
            crate::e2e_common::wait_for_window_settled(&mut client, deadline());
            client.send_ok(&Request::FocusWeb).unwrap();
            wait(&mut client, "the loaded page", |s| {
                !s.loading
                    && s.title.as_deref() == Some(PAGE_ONE_TITLE)
                    && s.tabs.iter().all(|tab| !tab.unresponsive && !tab.crashed)
            });
            client
                .send_ok(&Request::SetDisplayClock {
                    now_ms: Some(NOW_MS),
                    offset_s: 0,
                })
                .expect("pin the display clock");
            peek(&mut client, &output, appearance, &fixtures);
            permissions(&mut client, &output, appearance, &page_one);
            certificates(&mut client, &output, appearance);
            privacy(&mut client, &output, appearance);
            open_app(&mut client, &output, appearance, &fixtures);
            credentials(&mut client, &output, appearance, &fixtures);
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

/// The surface wait with its name and the portal-relevant state on timeout
/// (a guest failure leaves no other trace once the clone is gone).
fn wait(client: &mut Client, what: &str, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&Request::FlushFrame)
            .expect("flush surface frame");
        let s = client.snapshot().expect("surface snapshot");
        // Reduced motion draws a held load (a credential challenge keeps the
        // page loading) as static marks in the parent window: settled, never
        // animating, and outside the portal scene.
        if ready(&s) && s.motion.moments.iter().all(|m| m.variant == "static") {
            return s;
        }
        assert!(
            crate::e2e_common::Instant::now() < until,
            "{what} never became ready: portal visible={} synced={} presentation={:?} \
             ids={:?} peek={} preview={:?} prompt={:?} site={:?} mark={:?} cert={:?} \
             open_app={:?} auth_open={} settings_rows={:?} moments={:?} title={:?}",
            s.portal.visible,
            s.portal.frame_synced,
            s.portal.presentation,
            s.portal.ids,
            s.peek_presented,
            s.preview.as_ref().map(|p| (&p.url, &p.title, p.loading)),
            s.permissions.prompt.as_ref().map(|p| (&p.message, p.rect)),
            s.permissions.site_settings.as_ref().map(|v| &v.rows),
            s.permissions.address_mark,
            s.client_certificates.sheet.is_some(),
            (&s.open_app.no_app_banner, s.open_app.sheet.is_some()),
            s.auth.open,
            s.settings
                .as_ref()
                .map(|v| v.rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>()),
            s.motion.moments,
            s.title,
        );
        std::thread::yield_now();
    }
}

fn peek(client: &mut Client, output: &Path, appearance: &str, fixtures: &FixtureServer) {
    let initial = client.snapshot().unwrap();
    let source = initial.active_web_view.expect("source view");
    client
        .send_ok(&Request::NewWindow {
            source,
            source_profile: initial.active_profile,
            url: fixtures.url("/popup-target.html"),
            user_initiated: true,
            intent: NewWindowIntent::Default,
            input: NewWindowInput::Primary,
        })
        .expect("eligible new-window delivery");
    // The bar names the loaded page by its title, never the port.
    let opened = wait(client, "Peek's bar", |s| {
        s.peek_presented
            && s.portal.visible
            && s.portal.frame_synced
            && s.preview
                .as_ref()
                .is_some_and(|p| !p.loading && p.title.as_deref() == Some("BBR Popup Target"))
    });
    assert!(opened.portal.ids.iter().any(|id| id == "bbr-peek-bar"));
    assert_eq!(opened.tabs.len(), initial.tabs.len(), "no regular tab");
    save_portal(client, output, "peek-bar", appearance, opened);
    client.send_ok(&Request::PeekClose).unwrap();
    wait(client, "Peek closed", |s| {
        s.preview.is_none() && !s.portal.visible
    });
}

fn permission(client: &mut Client, input: PermissionInput) {
    client
        .send_ok(&Request::PermissionInput { input })
        .expect("permission input");
}

fn inject(client: &mut Client, event: WebEvent) {
    client
        .send_ok(&Request::InjectWebEvent { event })
        .expect("engine event");
}

fn permissions(client: &mut Client, output: &Path, appearance: &str, page: &str) {
    let view = client.snapshot().unwrap().active_web_view.expect("view");
    let origin = {
        let rest = page.split_once("://").unwrap().1;
        format!("http://{}", rest.split('/').next().unwrap())
    };
    let request = |request, capabilities| WebEvent::PermissionRequested {
        view,
        request,
        origin: origin.clone(),
        capabilities,
    };
    inject(
        client,
        request(
            7001,
            vec![
                PermissionCapability::Camera,
                PermissionCapability::Microphone,
            ],
        ),
    );
    let asked = wait(client, "the prompt", |s| {
        s.portal.frame_synced
            && s.permissions
                .prompt
                .as_ref()
                .is_some_and(|p| p.rect.is_some())
    });
    assert_eq!(
        asked.permissions.prompt.as_ref().unwrap().profile,
        "Profile · Default"
    );
    save_portal(client, output, "permission-prompt", appearance, asked);

    // Remember + Don't Allow: the site is blocked and remembered.
    permission(client, PermissionInput::Remember);
    permission(client, PermissionInput::Decide { allow: false });
    wait(client, "the prompt answered", |s| {
        s.permissions.prompt.is_none()
    });
    inject(client, request(7002, vec![PermissionCapability::Camera]));
    wait(client, "the blocked mark", |s| {
        s.permissions.address_mark.as_deref() == Some("blocked-camera")
    });
    permission(client, PermissionInput::ToggleSiteSettings);
    let settings = wait(client, "site settings", |s| {
        s.portal.frame_synced && s.permissions.site_settings.is_some()
    });
    save_portal(
        client,
        output,
        "permission-site-settings",
        appearance,
        settings,
    );
    permission(client, PermissionInput::ToggleSiteSettings);
    wait(client, "site settings closed", |s| {
        s.permissions.site_settings.is_none()
    });
}

fn certificate(client: &mut Client, input: ClientCertificateInput) {
    client
        .send_ok(&Request::ClientCertificateInput { input })
        .expect("certificate input");
}

fn certificates(client: &mut Client, output: &Path, appearance: &str) {
    let view = client.snapshot().unwrap().active_web_view.expect("view");
    inject(
        client,
        WebEvent::ClientCertificateRequested {
            view,
            request: 8001,
            host: "intranet.example.test".into(),
            identities: identities(),
        },
    );
    let sheet = wait(client, "the certificate sheet", |s| {
        s.portal.frame_synced && s.client_certificates.sheet.is_some()
    });
    save_portal(
        client,
        output,
        "client-certificate-sheet",
        appearance,
        sheet,
    );
    // Choose the valid identity and remember it for the host.
    certificate(client, ClientCertificateInput::Select { index: 1 });
    certificate(client, ClientCertificateInput::Remember);
    certificate(client, ClientCertificateInput::Continue);
    wait(client, "the remembered certificate", |s| {
        s.client_certificates.sheet.is_none() && !s.client_certificates.answers.is_empty()
    });
}

fn privacy(client: &mut Client, output: &Path, appearance: &str) {
    key(client, "cmd-,");
    wait(client, "Settings", |s| s.settings.is_some());
    client
        .send_ok(&Request::SettingsInput {
            input: SettingsInput::Pane {
                pane: "privacy".into(),
            },
        })
        .unwrap();
    let populated = wait(client, "the populated Privacy pane", |s| {
        s.settings.as_ref().is_some_and(|v| {
            v.pane == "privacy"
                && v.rows.iter().any(|row| row.id == "privacy-site-0")
                && v.rows.iter().any(|row| row.id == "privacy-certificate-0")
        })
    });
    save(
        client,
        output,
        "settings-privacy-populated",
        appearance,
        Still::Settings,
        populated,
    );
    client
        .send_ok(&Request::CloseSettings)
        .expect("close actual Settings window");
    wait(client, "Settings closed", |s| s.settings.is_none());
}

fn open_app(client: &mut Client, output: &Path, appearance: &str, fixtures: &FixtureServer) {
    let page = |to: &str| fixtures.url(&format!("/external-scheme.html?to={to}"));
    client
        .send_ok(&dsl::navigate(page("bbr-test-noapp:ping")))
        .unwrap();
    let banner = wait(client, "the no-app banner", |s| {
        s.portal.frame_synced && s.open_app.no_app_banner.is_some()
    });
    save_portal(client, output, "open-app-no-app", appearance, banner);
    // The banner leaves on its own 8 s timer; later stills never race it.
    wait(client, "the banner gone", |s| {
        s.open_app.no_app_banner.is_none()
    });
    client
        .send_ok(&dsl::navigate(page("mailto:someone@example.com")))
        .unwrap();
    let sheet = wait(client, "the open-in-app sheet", |s| {
        s.portal.frame_synced && s.open_app.sheet.is_some()
    });
    save_portal(client, output, "open-app-sheet", appearance, sheet);
    client
        .send_ok(&Request::OpenAppInput {
            input: OpenAppInput::Cancel,
        })
        .unwrap();
    wait(client, "the sheet cancelled", |s| {
        s.open_app.sheet.is_none()
    });
}

fn credentials(client: &mut Client, output: &Path, appearance: &str, fixtures: &FixtureServer) {
    client
        .send_ok(&dsl::navigate(fixtures.url("/__fx/auth/basic")))
        .unwrap();
    let asked = wait(client, "the credential sheet", |s| {
        s.portal.frame_synced && s.auth.open
    });
    assert_eq!(asked.auth.host, "127.0.0.1");
    save_portal(client, output, "auth-sheet", appearance, asked);
}
