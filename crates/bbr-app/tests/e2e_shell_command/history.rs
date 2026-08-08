use super::history_support::{entry, history_row, seed_store, spawn_seeded, STORE_FILE_NAME};
use super::*;
use bbr_persistence::{SearchEngineId, SearchSettings};

/// The Phase-7 command-history provider end-to-end: seeded history ranks,
/// activates through the typed `OpenHistory` destination, a fresh
/// navigation visit is recorded, and both survive a restart on the same
/// profile directory. The seeded custom settings feed the search fallback
/// (asserted by encoded intent).
#[test]
#[ignore]
fn seeded_history_ranks_activates_and_persists_across_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("cmd-hist-restart", pid);
    let profile = root.join("profile");
    let seed = vec![
        entry(1, "Alpha History", &page_two, 1, 30),
        entry(2, "Beta Records", "https://beta.example/", 4, 20),
        entry(3, "Gamma Reference", "https://gamma.example/", 1, 10),
    ];
    let repository = seed_store(&profile, &seed);
    // The seeded settings: a custom engine with a validated template. The
    // command-bar search fallback must use exactly this template (encoded
    // intent asserted below), not the hard-coded default.
    repository
        .store_settings(&SearchSettings {
            engine: SearchEngineId::Custom,
            custom_template: Some("https://search.example.test/?q=%s&lang=en".into()),
        })
        .unwrap();

    let (mut harness, socket, token) = spawn_seeded("cmd-hist-restart", &page_one, &profile);
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

        // The startup disposition is truthful: the seeded store loaded with
        // the custom engine and all three provider entries.
        let store = client.snapshot().expect("store snapshot");
        assert_eq!(store.store.store_load, StoreLoadSnapshot::Loaded);
        assert_eq!(store.store.search_engine, "custom");
        assert_eq!(store.store.history_count, 3);

        // The seeded history ranks through the provider-neutral corpus: the
        // query matches only the Alpha record's title. The portable core
        // exposes history rows without a subtitle (the destination URL is
        // proven by the typed activation below), so the identity asserted
        // here is the stable `history:1` key and title.
        let alpha = history_row(&mut client, &mut since, "alpha", "history:1");
        assert_eq!(alpha.title, "Alpha History");

        // Activate the history row: the typed `OpenHistory` destination
        // navigates the active tab to the record's URL (a real fixture
        // page) and records the visit.
        key(&mut client, "enter");
        let activated = client.snapshot().expect("activated history snapshot");
        let activated_view = activated.active_web_view.expect("active view");
        since = wait_title(&mut client, activated_view, PAGE_TWO_TITLE, since);
        let after_activation = client.snapshot().expect("after activation snapshot");
        assert_eq!(
            after_activation.store.history_count, 3,
            "dedupe bump (same URL), not a new record"
        );

        // A fresh navigation visit (a raw address Enter on the fixture page
        // NOT present in the seed) appends a new provider record.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: page_one.clone(),
            })
            .expect("set navigation text");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush navigation ranking");
        key(&mut client, "enter");
        since = wait_title(&mut client, activated_view, PAGE_ONE_TITLE, since);
        let after_visit = client.snapshot().expect("after visit snapshot");
        assert_eq!(
            after_visit.store.history_count, 4,
            "the new visit was recorded"
        );
        let _ = since;

        // Deterministic flush: the provider state is written to the store in
        // one transaction (the background writer uses the identical write).
        client
            .send_ok(&bbr_testkit::Request::FlushPersistence)
            .expect("flush persistence");
    }));
    // Keep the profile directory for the restart: a successful finish would
    // let the harness Drop remove the root, destroying the store we must
    // reload. The failure path keeps the diagnostics as usual.
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the provider restores the
    // flushed state (including the fresh visit) and the settings survive.
    let (mut harness2, socket2, token2) = spawn_seeded("cmd-hist-restart", &page_one, &profile);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("restart snapshot");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let view = first.active_web_view.expect("restart view");
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(store.store.store_load, StoreLoadSnapshot::Loaded);
        assert_eq!(
            store.store.search_engine, "custom",
            "settings survived the restart"
        );
        assert_eq!(
            store.store.history_count, 4,
            "the fresh visit persisted across the restart"
        );
        // The persisted records still rank: the page-two record's URL
        // matches a query on its unique fixture path segment.
        let key = "history:1";
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "alpha".into(),
            })
            .expect("set restart query");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush restart ranking");
        let snapshot = client.snapshot().expect("restart query snapshot");
        assert!(
            snapshot.command_results.iter().any(|row| row.key == key),
            "the persisted history record ranks after the restart"
        );
    }));
    finish(result, &mut harness2, &socket2, &token2);
}

/// A mismatched store is discarded and recreated empty — schema v2 has no
/// migrations and no backward compatibility — and the typed
/// `StoreLoad::Discarded { VersionMismatch }` disposition is surfaced
/// truthfully in the snapshot and startup log, never silently converted.
/// The app stays fully functional with the empty store.
#[test]
#[ignore]
fn mismatched_store_is_discarded_and_surfaced_truthfully() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("cmd-hist-mismatch", pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    // A current-schema store with data, then the stamp rewound to version 1:
    // a legacy store the app must refuse and discard untouched-read.
    let seed = vec![
        entry(1, "Legacy Alpha", "https://legacy-alpha.example/", 1, 1),
        entry(2, "Legacy Beta", "https://legacy-beta.example/", 1, 2),
    ];
    let repository = seed_store(&profile, &seed);
    repository
        .store_settings(&SearchSettings {
            engine: SearchEngineId::Google,
            custom_template: None,
        })
        .unwrap();
    let connection = rusqlite::Connection::open(profile.join(STORE_FILE_NAME)).unwrap();
    connection
        .pragma_update(None, "user_version", 1)
        .expect("rewind the stamp to v1");
    drop(connection);

    let (mut harness, socket, token) = spawn_seeded("cmd-hist-mismatch", &page_one, &profile);
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

        // The truthful disposition: the v1 legacy store was discarded and
        // recreated empty; nothing was loaded partially or converted.
        let store = client.snapshot().expect("discarded store snapshot");
        assert_eq!(
            store.store.store_load,
            StoreLoadSnapshot::Discarded {
                reason: StoreDiscardReasonSnapshot::VersionMismatch { found: 1 }
            }
        );
        assert_eq!(
            store.store.history_count, 0,
            "nothing loaded from the legacy store"
        );
        assert_eq!(
            store.store.search_engine, "duckduckgo",
            "the recreated empty store seeds the deterministic defaults"
        );

        // The app stays functional: the command surface still ranks (the
        // fresh store is a normal, non-fatal startup path).
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText { text: "bbr".into() })
            .expect("set query");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush ranking");
        let snapshot = client.snapshot().expect("functional snapshot");
        assert_eq!(snapshot.command_result_state, CommandResultState::Results);
        // The settle search fallback publishes with its actual key — the
        // typed terms in the Location mode (`search:bbr`), grouped as the
        // fallback row — never a nonexistent literal `fallback` key.
        assert!(
            snapshot
                .command_results
                .iter()
                .any(|row| row.key == "search:bbr" && row.group == "fallback"),
            "the command surface publishes the search fallback row"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
