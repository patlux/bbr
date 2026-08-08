//! Object-safe settings store port with typed load/store/reset outcomes.

use crate::search_settings::settings::SearchSettingsV1;

/// The typed outcome of a settings load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    /// A previously stored settings value exists.
    Stored(SearchSettingsV1),
    /// The store is empty; callers apply [`SearchSettingsV1::defaults`].
    Empty,
    /// The store could not produce a value; callers apply defaults and may
    /// retry later.
    Unavailable,
}

/// The typed outcome of a settings store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOutcome {
    /// The value was accepted and persisted by the store.
    Stored,
    /// The store refused to persist the value.
    Rejected,
}

/// The typed outcome of a settings reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetOutcome {
    /// A previously stored value was removed; the store is now empty.
    Reset,
    /// No value was stored, so there was nothing to reset.
    NothingStored,
    /// The reset could not be completed.
    Failed,
}

/// A portable, object-safe boundary for persisting global search settings.
///
/// Implementations may back this with any storage — a later SQLite
/// repository, a file, or process memory — without leaking it to callers:
/// every method takes and returns only portable [`SearchSettingsV1`] values
/// and typed outcome enums, and no SQLite, filesystem, or native object
/// appears in the API. Implementations must be `Send + Sync` so the store
/// can live behind an app/service seam.
pub trait SettingsStore: Send + Sync {
    /// Loads the stored settings, if any.
    fn load_settings(&self) -> LoadOutcome;

    /// Stores `settings`, replacing any previously stored value.
    fn store_settings(&self, settings: SearchSettingsV1) -> StoreOutcome;

    /// Removes any stored settings, returning the store to its empty state.
    fn reset_settings(&self) -> ResetOutcome;

    /// The effective settings for callers: the stored value when available,
    /// otherwise [`SearchSettingsV1::defaults`].
    fn load_effective_settings(&self) -> SearchSettingsV1 {
        match self.load_settings() {
            LoadOutcome::Stored(settings) => settings,
            LoadOutcome::Empty | LoadOutcome::Unavailable => SearchSettingsV1::defaults(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_settings::engine::SearchEngine;
    use std::sync::Mutex;

    /// Deterministic in-memory store exercising the port contract. Failures
    /// are opt-in per method so every outcome path is reachable without a
    /// real persistence implementation.
    struct FakeSettingsStore {
        stored: Mutex<Option<SearchSettingsV1>>,
        fail_load: bool,
        reject_store: bool,
        fail_reset: bool,
    }

    impl FakeSettingsStore {
        fn empty() -> Self {
            Self {
                stored: Mutex::new(None),
                fail_load: false,
                reject_store: false,
                fail_reset: false,
            }
        }

        fn failing_load() -> Self {
            Self {
                stored: Mutex::new(None),
                fail_load: true,
                reject_store: false,
                fail_reset: false,
            }
        }

        fn rejecting_store() -> Self {
            Self {
                stored: Mutex::new(None),
                fail_load: false,
                reject_store: true,
                fail_reset: false,
            }
        }

        fn failing_reset() -> Self {
            Self {
                stored: Mutex::new(Some(SearchSettingsV1::defaults())),
                fail_load: false,
                reject_store: false,
                fail_reset: true,
            }
        }
    }

    impl SettingsStore for FakeSettingsStore {
        fn load_settings(&self) -> LoadOutcome {
            if self.fail_load {
                return LoadOutcome::Unavailable;
            }
            match &*self.stored.lock().expect("fake store lock") {
                Some(settings) => LoadOutcome::Stored(settings.clone()),
                None => LoadOutcome::Empty,
            }
        }

        fn store_settings(&self, settings: SearchSettingsV1) -> StoreOutcome {
            if self.reject_store {
                return StoreOutcome::Rejected;
            }
            *self.stored.lock().expect("fake store lock") = Some(settings);
            StoreOutcome::Stored
        }

        fn reset_settings(&self) -> ResetOutcome {
            if self.fail_reset {
                return ResetOutcome::Failed;
            }
            let mut slot = self.stored.lock().expect("fake store lock");
            if slot.take().is_some() {
                ResetOutcome::Reset
            } else {
                ResetOutcome::NothingStored
            }
        }
    }

    fn custom_engine(template: &str) -> SearchEngine {
        SearchEngine::Custom(
            crate::search_settings::engine::CustomUrlTemplate::new(template).unwrap(),
        )
    }

    #[test]
    fn empty_store_loads_effective_defaults() {
        let store = FakeSettingsStore::empty();
        assert_eq!(store.load_settings(), LoadOutcome::Empty);
        assert_eq!(
            store.load_effective_settings(),
            SearchSettingsV1::defaults()
        );
    }

    #[test]
    fn store_then_load_roundtrips() {
        let store = FakeSettingsStore::empty();
        let settings = SearchSettingsV1::with_engine(custom_engine("https://example.com/?q=%s"));
        assert_eq!(store.store_settings(settings.clone()), StoreOutcome::Stored);
        assert_eq!(store.load_settings(), LoadOutcome::Stored(settings.clone()));
        assert_eq!(store.load_effective_settings(), settings);
        // Repeated reads stay deterministic.
        assert_eq!(store.load_settings(), LoadOutcome::Stored(settings));
    }

    #[test]
    fn store_supersedes_previous_value() {
        let store = FakeSettingsStore::empty();
        let first = SearchSettingsV1::with_engine(SearchEngine::Google);
        let second = SearchSettingsV1::with_engine(SearchEngine::Kagi);
        assert_eq!(store.store_settings(first), StoreOutcome::Stored);
        assert_eq!(store.store_settings(second.clone()), StoreOutcome::Stored);
        assert_eq!(store.load_settings(), LoadOutcome::Stored(second));
    }

    #[test]
    fn reset_removes_stored_value() {
        let store = FakeSettingsStore::empty();
        store.store_settings(SearchSettingsV1::defaults());
        assert_eq!(store.reset_settings(), ResetOutcome::Reset);
        assert_eq!(store.load_settings(), LoadOutcome::Empty);
        assert_eq!(
            store.load_effective_settings(),
            SearchSettingsV1::defaults()
        );
    }

    #[test]
    fn reset_without_stored_value_is_nothing_stored() {
        let store = FakeSettingsStore::empty();
        assert_eq!(store.reset_settings(), ResetOutcome::NothingStored);
    }

    #[test]
    fn unavailable_load_falls_back_to_defaults() {
        let store = FakeSettingsStore::failing_load();
        assert_eq!(store.load_settings(), LoadOutcome::Unavailable);
        assert_eq!(
            store.load_effective_settings(),
            SearchSettingsV1::defaults()
        );
    }

    #[test]
    fn rejected_store_reports_rejection() {
        let store = FakeSettingsStore::rejecting_store();
        assert_eq!(
            store.store_settings(SearchSettingsV1::defaults()),
            StoreOutcome::Rejected
        );
        assert_eq!(store.load_settings(), LoadOutcome::Empty);
    }

    #[test]
    fn failed_reset_reports_failure() {
        let store = FakeSettingsStore::failing_reset();
        assert_eq!(store.reset_settings(), ResetOutcome::Failed);
    }

    #[test]
    fn store_port_is_object_safe() {
        fn exercise(store: &dyn SettingsStore) {
            let _ = store.load_settings();
            let _ = store.store_settings(SearchSettingsV1::defaults());
            let _ = store.reset_settings();
            let _ = store.load_effective_settings();
        }

        let concrete = FakeSettingsStore::empty();
        exercise(&concrete);
        let boxed: Box<dyn SettingsStore> = Box::new(FakeSettingsStore::empty());
        exercise(boxed.as_ref());
        let _shared: std::sync::Arc<dyn SettingsStore> =
            std::sync::Arc::new(FakeSettingsStore::empty());
    }

    #[test]
    fn outcomes_are_typed_and_exhaustive() {
        // The enums are closed: every variant is reachable through the fake,
        // so a later store implementation cannot invent new states.
        let mut seen_load = Vec::new();
        let mut seen_store = Vec::new();
        let mut seen_reset = Vec::new();
        let empty = FakeSettingsStore::empty();
        match empty.load_settings() {
            LoadOutcome::Stored(_) => {}
            LoadOutcome::Empty => seen_load.push("empty"),
            LoadOutcome::Unavailable => {}
        }
        match empty.store_settings(SearchSettingsV1::defaults()) {
            StoreOutcome::Stored => seen_store.push("stored"),
            StoreOutcome::Rejected => {}
        }
        match FakeSettingsStore::empty().reset_settings() {
            ResetOutcome::Reset => {}
            ResetOutcome::NothingStored => seen_reset.push("nothing"),
            ResetOutcome::Failed => {}
        }
        match FakeSettingsStore::failing_load().load_settings() {
            LoadOutcome::Stored(_) => {}
            LoadOutcome::Empty => {}
            LoadOutcome::Unavailable => seen_load.push("unavailable"),
        }
        match FakeSettingsStore::rejecting_store().store_settings(SearchSettingsV1::defaults()) {
            StoreOutcome::Stored => {}
            StoreOutcome::Rejected => seen_store.push("rejected"),
        }
        match FakeSettingsStore::failing_reset().reset_settings() {
            ResetOutcome::Reset => {}
            ResetOutcome::NothingStored => {}
            ResetOutcome::Failed => seen_reset.push("failed"),
        }
        assert_eq!(seen_load, vec!["empty", "unavailable"]);
        assert_eq!(seen_store, vec!["stored", "rejected"]);
        assert_eq!(seen_reset, vec!["nothing", "failed"]);
    }
}
