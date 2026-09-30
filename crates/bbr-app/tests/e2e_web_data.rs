//! Persistent website data per profile (reality-gaps packet, item F).
//!
//! Every other suite gives each profile a non-persistent WebKit store, so
//! the identifier-backed stores users get — and their removal when a
//! profile is deleted — never ran under test. Here the app runs with
//! `--bbr-web-data-run <run>`: each profile gets a real persistent store in
//! this run's namespace (`42425e57-<run>-4000-…`, never a production
//! identifier), in WebKit's container of the unbundled test binary.
//!
//! 1. A cookie set in the Default profile is not seen by a second
//!    profile, which sets its own.
//! 2. After a quit and relaunch on the same profile directory, each profile
//!    still reads its own cookie.
//! 3. WebKit lists both profiles' stores (`fetchAllDataStoreIdentifiers:`);
//!    deleting the second profile in Settings removes its store while the
//!    Default profile's stays.
//! 4. A final ordinary launch removes every test run's stores — this run's
//!    and any an earlier, interrupted run left — and WebKit lists none.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{SettingsInput, Snapshot};
use bbr_testkit::Request;
use e2e_common::*;

const TAG: &str = "webdata";
const TEST_STORE_PREFIX: &str = "42425e57-";

fn run_id() -> u16 {
    (std::process::id() & 0xffff) as u16
}

/// The store identifier of `profile` in this run (`bbr_web_macos::store_kind`).
fn store_id(profile: u64) -> String {
    let upper = (profile >> 48) as u16;
    let lower = profile & 0x0000_ffff_ffff_ffff;
    format!(
        "{TEST_STORE_PREFIX}{:04x}-4000-{upper:04x}-{lower:012x}",
        run_id()
    )
}

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let s = snapshot(client);
        if done(&s) {
            return s;
        }
        assert!(Instant::now() < until, "timed out waiting for {what}");
        std::thread::yield_now();
    }
}

/// A fresh listing of WebKit's identifier-backed stores.
fn stores(client: &mut Client) -> Vec<String> {
    client
        .send_ok(&Request::ListWebDataStores)
        .expect("list stores");
    wait(client, "the store listing", |s| {
        s.web_data_stores.identifiers.is_some()
    })
    .web_data_stores
    .identifiers
    .unwrap()
}

/// Load `url` in the active tab and wait for `title`.
fn load(client: &mut Client, view: WebViewId, since: &mut u64, url: &str, title: &str) {
    client.send_ok(&dsl::navigate(url)).expect("navigate");
    *since = wait_title(client, view, title, *since);
}

fn start(client: &mut Client, token: &str) -> u64 {
    client.hello(token).expect("hello");
    client
        .attach(None)
        .expect("attach")
        .iter()
        .map(|(seq, _)| *seq)
        .max()
        .unwrap_or(0)
}

fn keep_root(
    result: std::thread::Result<()>,
    harness: &mut Harness,
    socket: &std::path::Path,
    token: &str,
) {
    match result {
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

fn quit(client: &mut Client, harness: &mut Harness) {
    client.quit().expect("quit");
    let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
    assert!(status.success(), "app exit: {status}");
}

#[test]
#[ignore]
fn each_profile_keeps_its_own_persistent_website_data() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, _page_two) = fixtures();
    let run = format!("{:04x}", run_id());
    let flags = ["--bbr-web-data-run", run.as_str()];
    let read = fixtures.url("/__fx/cookie/read");
    let root = harness_root(TAG, std::process::id());
    let _ = std::fs::remove_dir_all(&root);

    // 1. Two profiles, two cookies, no crossing.
    let (mut harness, socket, token) = spawn_harness_with(TAG, &page_one, &flags);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = start(&mut client, &token);
        let settled = wait_for_window_settled(&mut client, deadline());
        let view = settled.active_web_view.expect("view");
        load(
            &mut client,
            view,
            &mut since,
            &fixtures.url("/__fx/cookie/keep-alpha"),
            "Cookie Kept alpha",
        );
        load(&mut client, view, &mut since, &read, "Cookie alpha");

        client
            .send_ok(&dsl::create_profile("Second", "Second Space"))
            .expect("create profile");
        let (next, change) = wait_browser_change_matching(
            &mut client,
            |change| matches!(change, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = next;
        let BrowserChange::ProfileCreated { space, .. } = change else {
            unreachable!("matched ProfileCreated");
        };
        client
            .send_ok(&dsl::activate_space(space))
            .expect("activate the second profile's space");
        since = wait_browser_change(&mut client, BrowserChange::SpaceActivated { space }, since);
        let second = open_tab(&mut client, &mut since, &read, Some("Cookie none"));
        load(
            &mut client,
            second,
            &mut since,
            &fixtures.url("/__fx/cookie/keep-beta"),
            "Cookie Kept beta",
        );
        load(&mut client, second, &mut since, &read, "Cookie beta");
        client
            .send_ok(&Request::FlushBrowserState)
            .expect("flush browser state");
        quit(&mut client, &mut harness);
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    // 2 + 3. After a relaunch each profile reads its own cookie; deleting
    // the second profile removes its store.
    let (mut harness, socket, token) = spawn_harness_with(TAG, &page_one, &flags);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = start(&mut client, &token);
        let restored = wait_for_window_settled(&mut client, deadline());
        assert_eq!(restored.profiles.len(), 2, "both profiles came back");
        let default_profile = restored.profiles[0].id;
        let second_profile = restored.profiles[1].id;
        for profile in [default_profile, second_profile] {
            let space = restored
                .spaces
                .iter()
                .find(|space| space.profile == profile)
                .expect("a space of the profile")
                .space;
            if snapshot(&mut client).active_space != space {
                client
                    .send_ok(&dsl::activate_space(space))
                    .expect("activate space");
                since = wait_browser_change(
                    &mut client,
                    BrowserChange::SpaceActivated { space },
                    since,
                );
            }
            let cookie = if profile == default_profile {
                "Cookie alpha"
            } else {
                "Cookie beta"
            };
            open_tab(&mut client, &mut since, &read, Some(cookie));
        }

        let listed = stores(&mut client);
        for profile in [default_profile, second_profile] {
            assert!(
                listed.contains(&store_id(profile.0)),
                "WebKit keeps {}: {listed:?}",
                store_id(profile.0)
            );
        }

        // Delete the second profile in Settings › Profiles.
        client.send_ok(&Request::OpenSettings).expect("settings");
        for input in [
            SettingsInput::Pane {
                pane: "profiles".into(),
            },
            SettingsInput::Select {
                row: "profile-1".into(),
            },
            SettingsInput::Press {
                row: "profile-delete".into(),
            },
            SettingsInput::SheetConfirm,
        ] {
            client
                .send_ok(&Request::SettingsInput { input })
                .expect("settings input");
        }
        wait(&mut client, "one profile left", |s| s.profiles.len() == 1);
        client
            .send_ok(&Request::CloseSettings)
            .expect("close settings");
        let until = deadline();
        let remaining = loop {
            let listed = stores(&mut client);
            if !listed.contains(&store_id(second_profile.0)) {
                break listed;
            }
            assert!(
                Instant::now() < until,
                "the deleted profile's store stayed: {listed:?}"
            );
            std::thread::yield_now();
        };
        assert!(
            remaining.contains(&store_id(default_profile.0)),
            "the Default profile's store stays: {remaining:?}"
        );
        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);

    // 4. An ordinary launch removes every test run's stores.
    let (mut harness, socket, token) = spawn_harness("webdata-clean", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        start(&mut client, &token);
        wait_for_window_settled(&mut client, deadline());
        client
            .send_ok(&Request::RemoveTestWebDataStores)
            .expect("remove test stores");
        let removed = wait(&mut client, "the test stores removed", |s| {
            s.web_data_stores.removed.is_some() || s.web_data_stores.removal_error.is_some()
        });
        assert_eq!(
            removed.web_data_stores.removal_error, None,
            "every test store was removed"
        );
        assert!(
            removed.web_data_stores.removed.unwrap() >= 1,
            "this run's store"
        );
        let listed = stores(&mut client);
        assert!(
            listed.iter().all(|id| !id.starts_with(TEST_STORE_PREFIX)),
            "no test store is left: {listed:?}"
        );
        quit(&mut client, &mut harness);
    }));
    finish(result, &mut harness, &socket, &token);
}
