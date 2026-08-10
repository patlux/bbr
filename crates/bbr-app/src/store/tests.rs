//! Unit tests for the app-owned persistence composition
//! (`crate::store`): settings mapping, history policy, bounded persist
//! coalescing, restart identity, the settings-store port, the ephemeral
//! fallback, and the in-memory-only ranking contract. Focused here so the
//! production module stays under the ~500-line budget.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);
static LOGGER: Once = Once::new();

fn logger() {
    LOGGER.call_once(|| {
        let _ = log::set_logger(&Logger);
        log::set_max_level(log::LevelFilter::Off);
    });
    struct Logger;
    impl log::Log for Logger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            false
        }
        fn log(&self, _: &log::Record) {}
        fn flush(&self) {}
    }
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "bbr-app-store-{tag}-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn settings(engine: SearchEngineId, template: Option<&str>) -> SearchSettings {
    SearchSettings {
        engine,
        custom_template: template.map(str::to_string),
    }
}

#[test]
fn settings_dto_mapping_round_trips_all_engines() {
    let cases = [
        (SearchEngineId::DuckDuckGo, None),
        (SearchEngineId::Google, None),
        (SearchEngineId::Bing, None),
        (SearchEngineId::Ecosia, None),
        (SearchEngineId::Kagi, None),
        (
            SearchEngineId::Custom,
            Some("https://search.example.test/?q=%s"),
        ),
    ];
    for (engine, template) in cases {
        let persisted = settings(engine, template);
        let v1 = settings_to_v1(&persisted);
        assert_eq!(settings_to_persisted(&v1), persisted, "engine: {engine:?}");
    }
}

#[test]
fn custom_template_that_cannot_construct_falls_back_to_defaults() {
    // Belt and suspenders: the repository rejects such records on write
    // and recovers them on restore, so this fallback is unreachable in
    // production — but the mapping must never panic.
    let persisted = settings(SearchEngineId::Custom, Some("javascript:alert(%s)"));
    assert_eq!(
        settings_to_v1(&persisted).engine(),
        &SearchEngine::DuckDuckGo
    );
}

#[test]
fn fresh_store_loads_loaded_with_defaults_and_empty_history() {
    logger();
    let dir = temp_dir("fresh");
    let store = AppStore::open(&dir).expect("open store");
    assert_eq!(store.store_load, Some(StoreLoad::Loaded));
    assert_eq!(store.effective_settings(), &SearchSettingsV1::defaults());
    assert_eq!(
        store.effective_settings().engine(),
        &SearchEngine::DuckDuckGo
    );
    store.with_history(|history| assert!(history.is_empty()));
    assert!(store.repository().is_some());
}

#[test]
fn record_visit_dedupes_and_bumps_deterministically() {
    logger();
    let dir = temp_dir("history-policy");
    let mut store = AppStore::open(&dir).expect("open store");
    assert_eq!(store.store_load, Some(StoreLoad::Loaded));

    store.record_visit("https://a.example/", "Alpha");
    store.record_visit("https://b.example/", "Beta");
    store.record_visit("https://a.example/", ""); // dedupe bump
    store.with_history(|history| {
        assert_eq!(history.len(), 2);
        let alpha = history
            .iter()
            .find(|e| e.url == "https://a.example/")
            .unwrap();
        assert_eq!(alpha.visit_count, 2);
        let beta = history
            .iter()
            .find(|e| e.url == "https://b.example/")
            .unwrap();
        assert_eq!(beta.visit_count, 1);
        // Recency markers are monotonic: the deduped bump is more recent.
        assert!(alpha.last_visited > beta.last_visited);
        // The empty-title dedupe keeps the original title.
        assert_eq!(alpha.title, "Alpha");
    });
}

#[test]
fn rapid_visits_coalesce_to_one_pending_persist_signal() {
    logger();
    let dir = temp_dir("coalesce");
    let mut store = AppStore::open(&dir).expect("open store");
    for index in 0..100 {
        store.record_visit(&format!("https://coalesce-{index}.test/"), "Coalesce");
    }
    // The capacity-1 poke queue never grows with visits: a full channel is
    // an accepted coalesced signal (a persist is already pending), so the
    // receiver holds at most one redundant poke.
    let rx = store
        .write_rx
        .as_ref()
        .expect("writer receiver not yet spawned");
    let mut pokes = 0;
    while rx.try_recv().is_ok() {
        pokes += 1;
    }
    assert!(pokes <= 1, "rapid visits never grow the persist queue");
    // The provider holds every accepted record (no invented retention cap),
    // and the deterministic flush persists exactly that final state.
    store.with_history(|history| assert_eq!(history.len(), 100));
    store.flush_persistence();
    let repository = store.repository().expect("persisted repository");
    assert_eq!(repository.load_command_history().unwrap().len(), 100);
}

#[test]
fn flush_persistence_round_trips_the_provider() {
    logger();
    let dir = temp_dir("flush");
    let mut store = AppStore::open(&dir).expect("open store");
    store.record_visit("https://flush.example/", "Flushed");
    store.flush_persistence();
    let repository = store.repository().expect("persisted repository");
    let loaded = repository.load_command_history().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].url, "https://flush.example/");
    assert_eq!(loaded[0].visit_count, 1);
}

#[test]
fn reopened_store_continues_ids_and_recency_above_restored_maxima() {
    logger();
    let dir = temp_dir("restart");
    {
        let mut store = AppStore::open(&dir).expect("open store");
        store.record_visit("https://restart.example/", "Restart");
        store.record_visit("https://older.example/", "Older");
        store.flush_persistence();
    }
    let store = AppStore::open(&dir).expect("reopen store");
    assert_eq!(store.store_load, Some(StoreLoad::Loaded));
    store.with_history(|history| assert_eq!(history.len(), 2));
    // The persisted maxima seed the counters: a new visit ranks above
    // every restored one and gets a fresh, non-colliding identity.
    let mut store = store;
    store.record_visit("https://new.example/", "New");
    store.with_history(|history| {
        let new = history
            .iter()
            .find(|e| e.url == "https://new.example/")
            .unwrap();
        assert_eq!(new.id.0, 3);
        assert!(
            new.last_visited
                > history
                    .iter()
                    .map(|e| e.last_visited)
                    .max()
                    .unwrap()
                    .saturating_sub(1)
        );
        assert!(
            history
                .iter()
                .filter(|e| e.url == "https://restart.example/")
                .count()
                == 1
        );
    });
}

#[test]
fn settings_store_port_round_trips_over_the_repository() {
    logger();
    let dir = temp_dir("settings-port");
    let store = AppStore::open(&dir).expect("open store");
    // The v2 exactly-one-row invariant: a fresh store already carries the
    // seeded defaults, so load never reports Empty.
    assert!(matches!(
        store.load_settings(),
        LoadOutcome::Stored(settings) if settings == SearchSettingsV1::defaults()
    ));
    let custom = SearchSettingsV1::with_engine(SearchEngine::Custom(
        CustomUrlTemplate::new("https://search.example.test/?q=%s").unwrap(),
    ));
    assert_eq!(store.store_settings(custom.clone()), StoreOutcome::Stored);
    assert_eq!(store.load_settings(), LoadOutcome::Stored(custom));
    assert_eq!(store.reset_settings(), ResetOutcome::Reset);
    assert_eq!(
        store.load_settings(),
        LoadOutcome::Stored(SearchSettingsV1::defaults())
    );
}

#[test]
fn ephemeral_store_refuses_persistence_truthfully() {
    logger();
    let store = AppStore::ephemeral();
    assert_eq!(store.store_load, None);
    assert!(store.repository().is_none());
    assert_eq!(
        store.load_settings(),
        LoadOutcome::Unavailable,
        "the ephemeral store reports Unavailable, never a fabricated value"
    );
    assert_eq!(
        store.store_settings(SearchSettingsV1::defaults()),
        StoreOutcome::Rejected
    );
    assert_eq!(store.reset_settings(), ResetOutcome::Failed);
    store.flush_persistence();
}

#[test]
fn ranking_consumes_only_in_memory_provider_data() {
    logger();
    let dir = temp_dir("rank-in-memory");
    let store = AppStore::open(&dir).expect("open store");
    // The provider data is the only input ranking sees: corpus build and
    // rank consume a plain slice, so no SQLite can be reached from the
    // ranking path (the repository methods are only reachable through
    // `repository()`, `flush_persistence`, and the background writer).
    let history = store.with_history(|history| history.to_vec());
    let corpus = bbr_command::Corpus {
        history,
        ..bbr_command::Corpus::new()
    };
    let registry = bbr_command::CommandRegistry::builtin();
    let query = bbr_command::Query::new(bbr_command::Mode::Location, "example");
    let context = bbr_command::QueryContext::with_current(
        bbr_command::id::SpaceId(1),
        bbr_command::id::ProfileId(1),
    );
    let ranked = bbr_command::rank(&corpus, &registry, &query, &context, 16);
    assert!(!ranked.results.is_empty());
}

#[test]
fn b3_library_mutations_round_trip_through_the_combined_writer() {
    logger();
    let dir = temp_dir("b3-library-mutations");
    let mut store = AppStore::open(&dir).expect("open store");

    // Seed two records through the B2 save seam.
    store.save_saved_page("https://example.test/alpha", "Alpha");
    store.save_saved_page("https://example.test/beta", "Beta");
    assert_eq!(store.saved_pages_count(), 2);

    // Rename one record's user label (the display ladder head).
    store.rename_saved_page(1, "Alpha Renamed");
    store.with_saved_pages(|pages, _| {
        assert_eq!(
            pages
                .iter()
                .find(|p| p.id == 1)
                .unwrap()
                .user_label
                .as_deref(),
            Some("Alpha Renamed")
        );
    });
    // An invalid label is a no-op (bounds, control bytes, empty).
    store.rename_saved_page(1, "");
    store.rename_saved_page(1, "a\x00b");
    store.with_saved_pages(|pages, _| {
        assert_eq!(
            pages
                .iter()
                .find(|p| p.id == 1)
                .unwrap()
                .user_label
                .as_deref(),
            Some("Alpha Renamed")
        );
    });

    // Create a flat collection and move both records into it.
    let collection = store
        .create_saved_collection("Work")
        .expect("created collection");
    assert!(collection > 0);
    store.move_saved_pages_to_collection(&[1, 2], Some(collection));
    store.with_saved_pages(|pages, collections| {
        assert_eq!(collections.len(), 1);
        assert!(pages.iter().all(|p| p.collection_id == Some(collection)));
    });
    // Moving to an unknown collection is a no-op.
    store.move_saved_pages_to_collection(&[1], Some(999));
    store.with_saved_pages(|pages, _| {
        assert!(pages.iter().all(|p| p.collection_id == Some(collection)));
    });

    // Rename the collection; move one record out.
    store.rename_saved_collection(collection, "Renamed Work");
    store.move_saved_pages_to_collection(&[2], None);
    store.with_saved_pages(|pages, collections| {
        assert_eq!(collections[0].title, "Renamed Work");
        assert_eq!(
            pages.iter().find(|p| p.id == 1).unwrap().collection_id,
            Some(collection)
        );
        assert_eq!(
            pages.iter().find(|p| p.id == 2).unwrap().collection_id,
            None
        );
    });

    // Multi-select batch remove (one poke), then remove the collection.
    store.remove_saved_pages(&[1, 2]);
    assert_eq!(store.saved_pages_count(), 0);
    store.remove_saved_collection(collection);
    assert_eq!(store.saved_collections_count(), 0);

    // The combined writer persists the whole state in one transaction: a
    // reopen restores exactly the in-memory result (records + collections).
    store.flush_saved_pages();
    drop(store);
    let reopened = AppStore::open(&dir).expect("reopen store");
    assert_eq!(reopened.saved_pages_count(), 0);
    assert_eq!(reopened.saved_collections_count(), 0);
}

#[test]
fn b3_library_collection_remove_moves_members_out_first() {
    logger();
    let dir = temp_dir("b3-collection-remove");
    let mut store = AppStore::open(&dir).expect("open store");
    store.save_saved_page("https://example.test/alpha", "Alpha");
    store.save_saved_page("https://example.test/beta", "Beta");
    let collection = store.create_saved_collection("Work").expect("created");
    store.move_saved_pages_to_collection(&[1, 2], Some(collection));

    // Removing the collection clears membership in the same poke, so the
    // combined persisted write can never dangle (the strict collection
    // foreign key stays satisfied).
    store.remove_saved_collection(collection);
    store.with_saved_pages(|pages, collections| {
        assert!(collections.is_empty());
        assert!(pages.iter().all(|p| p.collection_id.is_none()));
    });
    store.flush_saved_pages();
    drop(store);
    let reopened = AppStore::open(&dir).expect("reopen store");
    assert_eq!(reopened.saved_collections_count(), 0);
    assert_eq!(reopened.saved_pages_count(), 2);
    reopened.with_saved_pages(|pages, _| {
        assert!(pages.iter().all(|p| p.collection_id.is_none()));
    });
}
