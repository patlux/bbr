//! Command-history provider E2E across the global composition surface
//! (Phase 7): the provider is a single app-owned authority shared by every
//! profile/space context, and the persisted global search settings feed the
//! command-bar fallback.

use super::history_support::{entry, history_row, seed_store, spawn_seeded, STORE_FILE_NAME};
use super::*;
use bbr_persistence::{Repository, SearchEngineId, SearchSettings};

/// The global history provider is a single authority: the same seeded
/// record ranks exactly once in every profile/space context — no profile
/// leak and no second authority, because the provider is app-owned and
/// global (v1 decision), never re-created per profile.
#[test]
#[ignore]
fn global_history_is_a_single_authority_across_profiles() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("cmd-hist-global", pid);
    let profile = root.join("profile");
    let seed = vec![
        entry(1, "Alpha History", &page_two, 1, 30),
        entry(2, "Beta Records", &page_one, 4, 20),
    ];
    seed_store(&profile, &seed);

    let (mut harness, socket, token) = spawn_seeded("cmd-hist-global", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_profile = first.active_profile;

        // In the default profile the seeded record ranks exactly once.
        let _ = history_row(&mut client, &mut since, "alpha", "history:1");

        // A second profile (private, isolated store) creates its own
        // space; the global history provider must rank the SAME record
        // exactly once there too — no duplicate, no per-profile copy.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::activate_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private = client.snapshot().expect("private profile snapshot");
        assert_ne!(
            private.active_profile, default_profile,
            "the private profile is active"
        );
        assert_eq!(private.active_profile_title.as_deref(), Some("Private"));

        let private_row = history_row(&mut client, &mut since, "alpha", "history:1");
        assert_eq!(private_row.title, "Alpha History");
        // The record's identity is stable across contexts: same key, same
        // activation target — the provider is the one authority.
        let store = client.snapshot().expect("global store snapshot");
        assert_eq!(
            store.store.history_count, 2,
            "no per-profile history copies"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

/// The command-bar search fallback uses the persisted global settings: with
/// a custom engine seeded, the fallback row's activation intent renders
/// through the validated custom template (asserted by encoded URL on the
/// wire, never the live provider). Dangerous templates (file:/javascript:/
/// data:) cannot reach the app — the repository rejects them on write and
/// the constructor validates on read.
#[test]
#[ignore]
fn search_fallback_uses_the_persisted_custom_engine() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("cmd-hist-engine", pid);
    let profile = root.join("profile");
    seed_store(&profile, &[]);
    let repository = Repository::new(profile.join(STORE_FILE_NAME));
    repository
        .store_settings(&SearchSettings {
            engine: SearchEngineId::Custom,
            custom_template: Some("https://search.example.test/?q=%s".into()),
        })
        .unwrap();

    let (mut harness, socket, token) = spawn_seeded("cmd-hist-engine", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let store = client.snapshot().expect("engine snapshot");
        assert_eq!(store.store.search_engine, "custom");

        // Type terms that match nothing local: the fallback search row
        // publishes, and Enter opens the URL the custom template renders.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "hello world".into(),
            })
            .expect("set search terms");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush search ranking");
        let snapshot = client.snapshot().expect("search snapshot");
        assert_eq!(snapshot.command_result_state, CommandResultState::Results);
        key(&mut client, "enter");
        // The encoded intent is asserted from the accepted navigation event
        // (never the live provider): the custom template rendered
        // `https://search.example.test/?q=hello%20world`.
        let expected_url = "https://search.example.test/?q=hello%20world";
        since = wait_web_event(
            &mut client,
            |web| matches!(web, WebEvent::UrlChanged(_, url) if url == expected_url),
            since,
        );
        let nav = client.snapshot().expect("navigation snapshot");
        assert_eq!(nav.url.as_deref(), Some(expected_url));
        let _ = since;
    }));
    finish(result, &mut harness, &socket, &token);
}
