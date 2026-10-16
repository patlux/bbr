//! Blueprint D13 real-app E2E (boards M-12, M-13, PR-04, PR-20): where
//! links go.
//!
//! Pop-ups: a real WKWebView page opens a pop-up without a gesture on
//! load; the new-window policy blocks it and a quiet banner over the card
//! says so. Open opens this pop-up as a tab; Always Allow remembers the
//! site in `site-permissions.txt`, after which the page's pop-up opens as a
//! tab through the real WebKit delegate without a banner; site settings
//! show and change the Pop-ups row.
//!
//! Other apps' schemes: a real page goes to `bbr-test-noapp:` — no app on
//! the Mac opens it, so a quiet banner says so; then it goes to `mailto:`
//! — the sheet names the app macOS would use and the site; Cancel drops the
//! link, Open with "Always allow" hands it over (the testkit build records
//! the link instead of launching the app) and remembers the site and
//! scheme, so the next `mailto:` link opens without the sheet.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{OpenAppInput, PermissionInput, Snapshot};
use bbr_testkit::Request;
use e2e_common::*;

const TAG: &str = "links";
const POPUP_SOURCE_TITLE: &str = "BBR Popup Source";

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = snapshot(client);
        if done(&snapshot) {
            return snapshot;
        }
        assert!(Instant::now() < until, "{what} never happened");
    }
}

fn input(client: &mut Client, input: PermissionInput) -> Snapshot {
    client
        .send_ok(&Request::PermissionInput { input })
        .expect("permission input");
    snapshot(client)
}

fn select(client: &mut Client, index: usize) {
    client.send_ok(&dsl::select_tab(index)).expect("select tab");
}

#[test]
#[ignore]
fn blocked_popups_offer_open_and_always_allow() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, _page_two) = fixtures();
    let popup_source = fixtures.url("/popup.html");
    let host = {
        let rest = popup_source.split_once("://").unwrap().1;
        rest.split(':').next().unwrap().to_string()
    };
    let (mut harness, socket, token) = spawn_harness(TAG, &popup_source);
    let permissions_file = harness_root(TAG, std::process::id())
        .join("profile")
        .join("site-permissions.txt");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let mut since = client
            .attach(None)
            .expect("attach")
            .iter()
            .map(|(seq, _)| *seq)
            .max()
            .unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        let source = settled.active_web_view.expect("source view");
        if settled.title.as_deref() != Some(POPUP_SOURCE_TITLE) {
            since = wait_title(&mut client, source, POPUP_SOURCE_TITLE, since);
        }

        // The page's load-time pop-up is blocked with a quiet notice.
        let blocked = wait(&mut client, "the blocked notice", |s| {
            s.permissions.popup_notice.is_some()
        });
        assert_eq!(
            blocked.permissions.popup_notice.as_deref(),
            Some(format!("A pop-up from {host} was blocked.").as_str())
        );
        assert_eq!(blocked.tabs.len(), 1, "nothing opened on its own");
        for id in [
            "bbr-popup-blocked",
            "bbr-popup-blocked-open",
            "bbr-popup-blocked-allow",
        ] {
            assert!(blocked.visible_ids.iter().any(|v| v == id), "{id}");
        }

        // Open: this pop-up opens as a tab.
        let opened = input(&mut client, PermissionInput::PopupOpen);
        assert!(opened.permissions.popup_notice.is_none());
        assert_eq!(opened.tabs.len(), 2);
        assert!(opened.tabs[opened.active_tab_index.unwrap()]
            .url
            .ends_with("/popup-target.html"));

        // Back to the source; a reload asks again. Always Allow remembers
        // the site and opens this pop-up.
        select(&mut client, 0);
        client.send_ok(&dsl::reload()).expect("reload");
        wait(&mut client, "the second notice", |s| {
            s.permissions.popup_notice.is_some()
        });
        let allowed = input(&mut client, PermissionInput::PopupAlwaysAllow);
        assert!(allowed.permissions.popup_notice.is_none());
        assert_eq!(allowed.tabs.len(), 3);
        let origin = {
            let rest = popup_source.split_once("://").unwrap().1;
            format!("http://{}", rest.split('/').next().unwrap())
        };
        assert_eq!(
            std::fs::read_to_string(&permissions_file).unwrap(),
            format!("1\t{origin}\tpopups\tallow\n")
        );

        // The page's next pop-up opens as a tab through the real WebKit
        // delegate — no notice.
        select(&mut client, 0);
        client.send_ok(&dsl::reload()).expect("reload again");
        let direct = wait(&mut client, "the allowed pop-up tab", |s| s.tabs.len() == 4);
        assert!(direct.permissions.popup_notice.is_none());

        // Site settings show the row and switch it back to Block.
        select(&mut client, 0);
        let settings = input(&mut client, PermissionInput::ToggleSiteSettings);
        let rows = settings
            .permissions
            .site_settings
            .expect("site settings")
            .rows;
        assert_eq!(rows[2], ("popups".to_string(), "Allow".to_string()));
        let blocked_again = input(&mut client, PermissionInput::SetSite { row: 2, value: 1 });
        assert_eq!(
            blocked_again.permissions.site_settings.unwrap().rows[2].1,
            "Block"
        );
        assert_eq!(std::fs::read_to_string(&permissions_file).unwrap(), "");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

fn open_app(client: &mut Client, input: OpenAppInput) -> Snapshot {
    client
        .send_ok(&Request::OpenAppInput { input })
        .expect("open-app input");
    snapshot(client)
}

#[test]
#[ignore]
fn links_to_other_apps_ask_first() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, _page_two) = fixtures();
    let page = |to: &str| fixtures.url(&format!("/external-scheme.html?to={to}"));
    let no_app = page("bbr-test-noapp:ping");
    let mail = page("mailto:someone@example.com");
    let host = {
        let rest = no_app.split_once("://").unwrap().1;
        rest.split(':').next().unwrap().to_string()
    };
    let origin = {
        let rest = no_app.split_once("://").unwrap().1;
        format!("http://{}", rest.split('/').next().unwrap())
    };
    let (mut harness, socket, token) = spawn_harness("links-apps", &no_app);
    let permissions_file = harness_root("links-apps", std::process::id())
        .join("profile")
        .join("site-permissions.txt");

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());

        // No app opens the scheme: a quiet banner says so.
        let banner = wait(&mut client, "the no-app banner", |s| {
            s.open_app.no_app_banner.is_some()
        });
        assert_eq!(
            banner.open_app.no_app_banner.as_deref(),
            Some("No app on this Mac opens “bbr-test-noapp:” links.")
        );
        assert!(banner.open_app.sheet.is_none());
        assert!(banner
            .visible_ids
            .iter()
            .any(|id| id == "bbr-external-no-app"));

        // mailto: asks, naming the app and the site.
        client
            .send_ok(&dsl::navigate(mail.clone()))
            .expect("mail page");
        let asked = wait(&mut client, "the open-in-app sheet", |s| {
            s.open_app.sheet.is_some()
        });
        assert_sheet_centered(
            &mut client,
            "bbr-external-open",
            "bbr-external-open-veil",
            260.0,
        );
        let sheet = asked.open_app.sheet.clone().unwrap();
        let app = sheet
            .title
            .strip_prefix("Open in ")
            .and_then(|rest| rest.strip_suffix('?'))
            .expect("Open in <App>?")
            .to_string();
        assert!(!app.is_empty());
        assert_eq!(
            sheet.message,
            format!("{host} wants to open the {app} app.")
        );
        assert_eq!(
            sheet.remember_label.as_deref(),
            Some(format!("Always allow {host} to open {app} links").as_str())
        );
        assert_eq!(sheet.confirm, format!("Open {app}"));
        assert!(asked.visible_ids.iter().any(|id| id == "bbr-external-open"));

        // Cancel drops the link.
        let cancelled = open_app(&mut client, OpenAppInput::Cancel);
        assert!(cancelled.open_app.sheet.is_none());
        assert!(cancelled.open_app.opened.is_empty());

        // Again: Always allow + Open hands it over and remembers.
        client.send_ok(&dsl::reload()).expect("reload");
        wait(&mut client, "the sheet again", |s| {
            s.open_app.sheet.is_some()
        });
        open_app(&mut client, OpenAppInput::Remember);
        let opened = open_app(&mut client, OpenAppInput::Open);
        assert!(opened.open_app.sheet.is_none());
        assert_eq!(opened.open_app.opened, ["mailto:someone@example.com"]);
        assert_eq!(
            std::fs::read_to_string(&permissions_file).unwrap(),
            format!("1\t{origin}\tapp:mailto\tallow\n")
        );

        // The next mailto: link opens without asking.
        client.send_ok(&dsl::reload()).expect("reload again");
        let direct = wait(&mut client, "the direct hand-over", |s| {
            s.open_app.opened.len() == 2
        });
        assert!(direct.open_app.sheet.is_none());

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn links_from_other_apps_open_in_the_last_space_or_ask() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("links-spaces", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _ = client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let default_space = settled.active_space;
        let profile = settled.active_profile;

        // A second space; the first stays the last active one.
        client
            .send_ok(&dsl::create_space(profile, "Work"))
            .expect("create space");
        let created = wait(&mut client, "the second space", |s| s.spaces.len() == 2);
        let work = created
            .spaces
            .iter()
            .find(|space| space.title == "Work")
            .expect("work space")
            .space;
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("back to default");
        wait(&mut client, "default active", |s| {
            s.active_space == default_space
        });

        // Default: a link from another app opens in the last active space.
        client
            .send_ok(&Request::DeliverExternalUrl {
                url: page_two.clone(),
            })
            .expect("deliver");
        let routed = wait(&mut client, "the routed tab", |s| s.tabs.len() == 2);
        assert_eq!(routed.active_space, default_space);
        assert!(routed.open_app.space_chooser.is_none());

        // Settings › General › Ask each time: the chooser asks.
        client.send_ok(&Request::OpenSettings).expect("settings");
        client
            .send_ok(&Request::SettingsInput {
                input: bbr_testkit::protocol::SettingsInput::Choose {
                    row: "general-external-links".into(),
                    index: 1,
                },
            })
            .expect("ask");
        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");
        let asked_url = format!("{page_two}?asked");
        client
            .send_ok(&Request::DeliverExternalUrl {
                url: asked_url.clone(),
            })
            .expect("deliver again");
        let chooser = wait(&mut client, "the space chooser", |s| {
            s.open_app.space_chooser.is_some()
        });
        let (title, rows, selected) = chooser.open_app.space_chooser.clone().unwrap();
        assert!(title.starts_with("Open 127.0.0.1 in"), "{title}");
        assert_eq!(rows, ["Open in Default", "Open in Work"]);
        assert_eq!(selected, 0, "the last active space is preselected");
        assert!(chooser
            .visible_ids
            .iter()
            .any(|id| id == "bbr-space-chooser"));
        assert_eq!(chooser.tabs.len(), 2, "nothing opened yet");

        // ↓ Return opens it in Work.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "down".into(),
            })
            .expect("down");
        assert_eq!(snapshot(&mut client).open_app.space_chooser.unwrap().2, 1);
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("enter");
        let opened = wait(&mut client, "the tab in Work", |s| {
            s.active_space == work && s.tabs.len() == 1
        });
        assert!(opened.open_app.space_chooser.is_none());
        assert_eq!(opened.tabs[0].url, asked_url);

        // Escape drops a link.
        client
            .send_ok(&Request::DeliverExternalUrl {
                url: format!("{page_two}?dropped"),
            })
            .expect("deliver a third");
        wait(&mut client, "the chooser again", |s| {
            s.open_app.space_chooser.is_some()
        });
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape");
        let dropped = wait(&mut client, "the chooser closed", |s| {
            s.open_app.space_chooser.is_none()
        });
        assert_eq!(dropped.tabs.len(), 1, "nothing opened");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
