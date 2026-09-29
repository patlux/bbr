//! Blueprint D11 real-app E2E (boards M-8, M-9, L-6, PR-14): permission
//! prompts, remembered decisions, the address's device marks and site
//! settings.
//!
//! Requests and capture state arrive through the engine's own event
//! transport (the events the WebKit delegate emits): WebKit exposes no
//! media devices to an app without camera/microphone usage descriptions —
//! the unbundled test binary — and a real grant would ask macOS for the
//! camera, so the native delegate is verified by hand on the bundled app.
//! A request shows one prompt hanging from the address, naming the site and
//! the profile; Remember + Don't Allow writes a Block and a later request is
//! refused quietly (the address shows the blocked mark, which opens site
//! settings). Site settings change a permission at once and write
//! `site-permissions.txt`; a remembered Allow grants without a prompt; a
//! second request waits behind the first; one-time answers are never
//! remembered; the in-use mark follows the capture state; Settings ›
//! Privacy lists and forgets the site; ⌘T › Site Settings… opens them and
//! Escape closes them. Clear Site Data… removes the site's
//! cookie from the real WebKit store, which the fixture's cookie echo
//! proves.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{PermissionInput, SettingsInput, Snapshot};
use bbr_testkit::Request;
use bbr_web::permissions::PermissionCapability;
use e2e_common::*;

const TAG: &str = "permissions";

fn input(client: &mut Client, input: PermissionInput) -> Snapshot {
    client
        .send_ok(&Request::PermissionInput { input })
        .expect("permission input");
    snapshot(client)
}

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

fn inject(client: &mut Client, event: WebEvent) {
    client
        .send_ok(&Request::InjectWebEvent { event })
        .expect("inject");
}

fn wait_title_prefix(
    client: &mut Client,
    view: WebViewId,
    prefix: &str,
    since: u64,
) -> (u64, String) {
    let (seq, event) = client
        .wait_for_event_since(
            |event| {
                matches!(event, Event::Web(WebEvent::TitleChanged(id, title))
                    if *id == view && title.starts_with(prefix))
            },
            since,
            deadline(),
        )
        .expect("title");
    match event {
        Event::Web(WebEvent::TitleChanged(_, title)) => (seq, title),
        _ => unreachable!(),
    }
}

#[test]
#[ignore]
fn permissions_prompt_remember_and_site_settings() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let set_alpha = fixtures.url("/__fx/cookie/set-alpha");
    let read_cookie = fixtures.url("/__fx/cookie/read");
    let origin = {
        let rest = page_one.split_once("://").unwrap().1;
        format!("http://{}", rest.split('/').next().unwrap())
    };
    let host = origin
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap()
        .to_string();
    let (mut harness, socket, token) = spawn_harness(TAG, &page_one);
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
        let view = settled.active_web_view.expect("view");
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        let camera = |request| WebEvent::PermissionRequested {
            view,
            request,
            origin: origin.clone(),
            capabilities: vec![PermissionCapability::Camera],
        };

        // The page asks for the camera: one prompt, anchored to the
        // address, names the site and the profile.
        inject(&mut client, camera(7001));
        let asked = wait(&mut client, "the camera prompt", |s| {
            s.permissions.prompt.is_some()
        });
        let prompt = asked.permissions.prompt.clone().unwrap();
        assert_eq!(prompt.message, format!("{host} wants to use your camera"));
        assert_eq!(prompt.profile, "Profile · Default");
        assert_eq!(prompt.capabilities, ["camera"]);
        assert!(!prompt.remember, "nothing is remembered by default");
        let prompt = wait(&mut client, "the drawn prompt", |s| {
            s.permissions
                .prompt
                .as_ref()
                .is_some_and(|p| p.rect.is_some())
        })
        .permissions
        .prompt
        .unwrap();
        let [px, py, _, _] = prompt.rect.unwrap();
        let address = asked.address_rect.expect("address rect");
        assert!((px - address[0]).abs() < 1.0, "hangs from the address");
        assert!(py > address[1] + address[3], "below the address");

        // Remember + Don't Allow: the engine is answered, the Block is
        // written.
        input(&mut client, PermissionInput::Remember);
        let denied = input(&mut client, PermissionInput::Decide { allow: false });
        assert!(denied.permissions.prompt.is_none());
        assert_eq!(denied.permissions.decisions, [(7001, false)]);
        let file = std::fs::read_to_string(&permissions_file).expect("permissions file");
        assert_eq!(file, format!("1\t{origin}\tcamera\tblock\n"));

        // The next request is refused quietly and the address shows the
        // blocked mark.
        inject(&mut client, camera(7002));
        let quiet = wait(&mut client, "the blocked mark", |s| {
            s.permissions.address_mark.as_deref() == Some("blocked-camera")
        });
        assert!(quiet.permissions.prompt.is_none(), "never a prompt storm");
        assert_eq!(quiet.permissions.decisions, [(7001, false), (7002, false)]);
        assert!(quiet
            .visible_ids
            .iter()
            .any(|id| id == "bbr-address-blocked"));

        // The mark opens site settings: Camera Block, Microphone Ask.
        let open = input(&mut client, PermissionInput::ToggleSiteSettings);
        let settings = open
            .permissions
            .site_settings
            .clone()
            .expect("site settings");
        assert_eq!(settings.host, host);
        assert_eq!(
            settings.connection,
            "Connection is secure · Profile Default"
        );
        assert_eq!(
            settings.rows,
            [
                ("camera".to_string(), "Block".to_string()),
                ("microphone".to_string(), "Ask".to_string())
            ]
        );
        // Camera → Ask applies at once and ends the blocked mark.
        let changed = input(&mut client, PermissionInput::SetSite { row: 0, value: 1 });
        assert_eq!(changed.permissions.site_settings.unwrap().rows[0].1, "Ask");
        assert!(changed.permissions.address_mark.is_none());
        assert_eq!(std::fs::read_to_string(&permissions_file).unwrap(), "");
        // Microphone → Allow is remembered.
        input(&mut client, PermissionInput::SetSite { row: 1, value: 0 });
        assert_eq!(
            std::fs::read_to_string(&permissions_file).unwrap(),
            format!("1\t{origin}\tmicrophone\tallow\n")
        );
        // A press outside closes the popover.
        let drawn = wait(&mut client, "the drawn popover", |s| {
            s.permissions
                .site_settings
                .as_ref()
                .is_some_and(|p| p.rect.is_some())
        });
        let [sx, sy, sw, sh] = drawn.permissions.site_settings.unwrap().rect.unwrap();
        let inside = input(
            &mut client,
            PermissionInput::PointerDown {
                x: sx + sw / 2.0,
                y: sy + sh / 2.0,
            },
        );
        assert!(
            inside.permissions.site_settings.is_some(),
            "a press inside keeps it"
        );
        let outside = input(
            &mut client,
            PermissionInput::PointerDown {
                x: sx + sw + 200.0,
                y: sy + sh + 200.0,
            },
        );
        assert!(outside.permissions.site_settings.is_none());

        // Allow path through the engine's own transport: a remembered Allow
        // (microphone) grants at once; the camera now asks again, and a
        // second request waits behind it.
        inject(
            &mut client,
            WebEvent::PermissionRequested {
                view,
                request: 9001,
                origin: origin.clone(),
                capabilities: vec![PermissionCapability::Microphone],
            },
        );
        let granted = snapshot(&mut client);
        assert_eq!(granted.permissions.decisions.last(), Some(&(9001, true)));
        assert!(granted.permissions.prompt.is_none());
        for request in [9002, 9003] {
            inject(
                &mut client,
                WebEvent::PermissionRequested {
                    view,
                    request,
                    origin: origin.clone(),
                    capabilities: vec![
                        PermissionCapability::Camera,
                        PermissionCapability::Microphone,
                    ],
                },
            );
        }
        let queued = snapshot(&mut client);
        let prompt = queued.permissions.prompt.clone().expect("prompt");
        assert_eq!(prompt.request, 9002);
        assert_eq!(
            prompt.message,
            format!("{host} wants to use your camera and microphone")
        );
        assert_eq!(prompt.queued, 1);
        // Allow once (not remembered): the next request shows.
        let next = input(&mut client, PermissionInput::Decide { allow: true });
        assert_eq!(next.permissions.decisions.last(), Some(&(9002, true)));
        assert_eq!(
            next.permissions.prompt.as_ref().map(|p| p.request),
            Some(9003)
        );
        let done = input(&mut client, PermissionInput::Decide { allow: false });
        assert!(done.permissions.prompt.is_none());
        assert_eq!(
            std::fs::read_to_string(&permissions_file).unwrap(),
            format!("1\t{origin}\tmicrophone\tallow\n"),
            "one-time answers are never remembered"
        );

        // The in-use mark follows WebKit's capture state.
        inject(
            &mut client,
            WebEvent::MediaCaptureChanged {
                view,
                camera: true,
                microphone: true,
            },
        );
        let live = snapshot(&mut client);
        assert_eq!(
            live.permissions.address_mark.as_deref(),
            Some("live-camera")
        );
        assert!(live
            .visible_ids
            .iter()
            .any(|id| id == "bbr-address-capture"));
        inject(
            &mut client,
            WebEvent::MediaCaptureChanged {
                view,
                camera: false,
                microphone: false,
            },
        );
        assert!(snapshot(&mut client).permissions.address_mark.is_none());

        // Settings › Privacy lists the remembered site; Forget clears it.
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
            .find(|row| row.id == "privacy-site-0")
            .expect("the remembered site")
            .clone();
        assert_eq!(row.title, host);
        assert_eq!(row.detail.as_deref(), Some("Microphone: Allow"));
        client
            .send_ok(&Request::SettingsInput {
                input: SettingsInput::Press {
                    row: "privacy-site-0".into(),
                },
            })
            .expect("forget");
        assert_eq!(std::fs::read_to_string(&permissions_file).unwrap(), "");
        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");

        // ⌘T › Site Settings… opens them too; Escape closes them.
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "cmd-t".into(),
            })
            .expect("cmd-t");
        client
            .send_ok(&Request::SetCommandBarText {
                text: "> site settings".into(),
            })
            .expect("query");
        client
            .send_ok(&Request::FlushCommandRanking)
            .expect("flush ranking");
        let ranked = snapshot(&mut client);
        assert_eq!(
            ranked.command_results.first().map(|row| row.key.as_str()),
            Some("action:19"),
            "{:?}",
            ranked.command_results
        );
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "enter".into(),
            })
            .expect("enter");
        wait(&mut client, "site settings from ⌘T", |s| {
            s.permissions.site_settings.is_some()
        });
        client
            .send_ok(&Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape");
        wait(&mut client, "site settings closed", |s| {
            s.permissions.site_settings.is_none()
        });

        // Clear Site Data… removes the site's cookie.
        client
            .send_ok(&dsl::navigate(set_alpha.clone()))
            .expect("set cookie");
        let (seq, _) = wait_title_prefix(&mut client, view, "Cookie Set alpha", since);
        since = seq;
        client
            .send_ok(&dsl::navigate(read_cookie.clone()))
            .expect("read cookie");
        since = wait_title(&mut client, view, "Cookie alpha", since);
        input(&mut client, PermissionInput::ToggleSiteSettings);
        let confirm = input(&mut client, PermissionInput::ClearData);
        assert!(confirm.permissions.site_settings.unwrap().clear_confirm);
        let cleared = input(&mut client, PermissionInput::ClearConfirm);
        assert!(cleared.permissions.site_settings.is_none());
        assert_eq!(
            cleared.toast.as_ref().map(|t| t.message.clone()),
            Some(format!("Cleared the data of {host}"))
        );
        let until = deadline();
        loop {
            client
                .send_ok(&dsl::navigate(read_cookie.clone()))
                .expect("read again");
            // The echo page's title: "Cookie alpha" or "Cookie none" (never
            // the set page's "Cookie Set …").
            let (seq, title) = wait_title_prefix(&mut client, view, "Cookie ", since);
            since = seq;
            if title.starts_with("Cookie Set") {
                continue;
            }
            if title == "Cookie none" {
                break;
            }
            assert!(
                Instant::now() < until,
                "the cookie survived Clear Site Data"
            );
        }

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
