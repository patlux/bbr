//! Blueprint D12 real-app E2E (board M-11, DESIGN §18.4): the
//! client-certificate sheet.
//!
//! Requests arrive through the engine's own event transport (the event the
//! WebKit navigation delegate emits with the matching Keychain identities'
//! metadata): a server that requires client certificates and a test
//! identity in the Keychain would change the user's Keychain, so the native
//! enumeration is covered by its contract test against a real certificate
//! and by hand. The sheet blocks the card, names the server and the
//! profile, lists the identities with issuer and expiry — the expired one
//! visible, dimmed and not selectable — and takes the keyboard: ↓ skips the
//! expired row, Return continues with the chosen identity. Remember keeps
//! the choice in `client-certificates.txt`, so the next request of the
//! server is answered without the sheet; Don't Send and Escape (cancel)
//! answer the engine accordingly; Settings › Privacy lists and forgets the
//! remembered server.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{ClientCertificateInput, SettingsInput, Snapshot};
use bbr_testkit::Request;
use bbr_web::client_certificates::ClientIdentityInfo;
use e2e_common::*;

const TAG: &str = "client-certs";
const FP_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const FP_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const FP_OLD: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn key(client: &mut Client, keystroke: &str) -> Snapshot {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("keystroke");
    snapshot(client)
}

fn input(client: &mut Client, input: ClientCertificateInput) -> Snapshot {
    client
        .send_ok(&Request::ClientCertificateInput { input })
        .expect("certificate input");
    snapshot(client)
}

fn identities() -> Vec<ClientIdentityInfo> {
    vec![
        ClientIdentityInfo {
            fingerprint: FP_OLD.into(),
            name: "Patrick W (old)".into(),
            issuer: "Corp Issuing CA 1".into(),
            // 2 Feb 2026, 12:00 UTC.
            expires_unix: Some(1_770_033_600),
        },
        ClientIdentityInfo {
            fingerprint: FP_A.into(),
            name: "Patrick Wozniak".into(),
            issuer: "Corp Issuing CA 2".into(),
            // 3 Mar 2037, 12:00 UTC.
            expires_unix: Some(2_119_780_800),
        },
        ClientIdentityInfo {
            fingerprint: FP_B.into(),
            name: "p.wozniak@corp.example".into(),
            issuer: "Corp Device CA".into(),
            // 12 Jan 2037, 12:00 UTC.
            expires_unix: Some(2_115_374_400),
        },
    ]
}

fn request(client: &mut Client, view: WebViewId, request: u64, host: &str) {
    client
        .send_ok(&Request::InjectWebEvent {
            event: WebEvent::ClientCertificateRequested {
                view,
                request,
                host: host.into(),
                identities: identities(),
            },
        })
        .expect("inject");
}

#[test]
#[ignore]
fn client_certificate_sheet_chooses_remembers_and_forgets() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness(TAG, &page_one);
    let choices_file = harness_root(TAG, std::process::id())
        .join("profile")
        .join("client-certificates.txt");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("view");

        // The sheet names the server and the profile and lists every
        // identity; the expired one is dimmed and the first valid one is
        // chosen.
        request(&mut client, view, 41, "vpn.corp.example");
        let open = snapshot(&mut client);
        let sheet = open.client_certificates.sheet.clone().expect("the sheet");
        assert_eq!(
            sheet.message,
            "vpn.corp.example asks for a certificate to identify you. Profile · Default."
        );
        assert_eq!(sheet.rows.len(), 3);
        assert_eq!(sheet.rows[0].0, "Patrick W (old)");
        assert!(sheet.rows[0].1.starts_with("Corp Issuing CA 1 · expired "));
        assert!(sheet.rows[0].2, "expired");
        assert!(sheet.rows[1].1.starts_with("Corp Issuing CA 2 · expires "));
        assert!(!sheet.rows[1].2);
        assert_eq!(sheet.selected, Some(1), "the first valid identity");
        assert!(!sheet.remember);
        for id in [
            "bbr-client-certificate",
            "bbr-client-certificate-row-0",
            "bbr-client-certificate-continue",
            "bbr-client-certificate-none",
        ] {
            assert!(open.visible_ids.iter().any(|v| v == id), "{id}");
        }

        // The expired identity cannot be chosen; ↓/↑ move over the valid
        // rows only.
        let refused = input(&mut client, ClientCertificateInput::Select { index: 0 });
        assert_eq!(refused.client_certificates.sheet.unwrap().selected, Some(1));
        let down = key(&mut client, "down");
        assert_eq!(down.client_certificates.sheet.unwrap().selected, Some(2));
        let up = key(&mut client, "up");
        assert_eq!(up.client_certificates.sheet.unwrap().selected, Some(1));
        let up = key(&mut client, "up");
        assert_eq!(up.client_certificates.sheet.unwrap().selected, Some(1));

        // Remember + Return: the engine gets the identity; the choice is
        // written.
        input(&mut client, ClientCertificateInput::Remember);
        let chosen = key(&mut client, "enter");
        assert!(chosen.client_certificates.sheet.is_none());
        assert_eq!(
            chosen.client_certificates.answers,
            [(41, format!("identity:{FP_A}"))]
        );
        assert_eq!(
            std::fs::read_to_string(&choices_file).unwrap(),
            format!("1\tvpn.corp.example\t{FP_A}\tPatrick Wozniak\n")
        );

        // The server asks again: answered from memory, no sheet.
        request(&mut client, view, 42, "vpn.corp.example");
        let remembered = snapshot(&mut client);
        assert!(remembered.client_certificates.sheet.is_none());
        assert_eq!(
            remembered.client_certificates.answers.last(),
            Some(&(42, format!("identity:{FP_A}")))
        );

        // Two other servers: the second waits behind the first. Don't Send
        // answers the first, Escape cancels the second.
        request(&mut client, view, 43, "intranet.example:8443");
        request(&mut client, view, 44, "mail.example");
        let queued = snapshot(&mut client);
        let first = queued.client_certificates.sheet.clone().unwrap();
        assert_eq!(first.request, 43);
        assert_eq!(first.queued, 1);
        let next = input(&mut client, ClientCertificateInput::DontSend);
        assert_eq!(
            next.client_certificates.sheet.as_ref().map(|s| s.request),
            Some(44)
        );
        let cancelled = key(&mut client, "escape");
        assert!(cancelled.client_certificates.sheet.is_none());
        assert_eq!(
            cancelled.client_certificates.answers[2..],
            [(43, "none".to_string()), (44, "cancel".to_string())]
        );
        assert_eq!(
            std::fs::read_to_string(&choices_file).unwrap(),
            format!("1\tvpn.corp.example\t{FP_A}\tPatrick Wozniak\n"),
            "unremembered answers stay one-time"
        );

        // A navigation to another server cancels its waiting request.
        request(&mut client, view, 45, "elsewhere.example");
        assert!(snapshot(&mut client).client_certificates.sheet.is_some());
        client
            .send_ok(&dsl::navigate(page_one.clone()))
            .expect("navigate");
        let until = deadline();
        let navigated = loop {
            let s = snapshot(&mut client);
            if s.client_certificates.sheet.is_none() {
                break s;
            }
            assert!(Instant::now() < until, "the navigation never cancelled");
        };
        assert_eq!(
            navigated.client_certificates.answers.last(),
            Some(&(45, "cancel".to_string()))
        );

        // Settings › Privacy lists the remembered server; Forget clears it.
        client.send_ok(&Request::OpenSettings).expect("settings");
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Pane {
                    pane: "privacy".into(),
                },
            })
            .expect("privacy");
        let privacy = snapshot(&mut client);
        let row = privacy
            .settings
            .as_ref()
            .unwrap()
            .rows
            .iter()
            .find(|row| row.id == "privacy-certificate-0")
            .expect("the remembered server")
            .clone();
        assert_eq!(row.title, "vpn.corp.example");
        assert_eq!(row.detail.as_deref(), Some("Patrick Wozniak"));
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Press {
                    row: "privacy-certificate-0".into(),
                },
            })
            .expect("forget");
        assert_eq!(std::fs::read_to_string(&choices_file).unwrap(), "");
        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");

        // Forgotten: the server's next request shows the sheet again.
        request(&mut client, view, 46, "vpn.corp.example");
        assert!(snapshot(&mut client).client_certificates.sheet.is_some());
        input(&mut client, ClientCertificateInput::Cancel);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
