//! App-owned persistence composition (Phase 7, command-history provider
//! slice): the one shared `bbr-app -> bbr-persistence` bootstrap.
//!
//! [`AppStore`] owns the path-based [`Repository`], the truthful startup
//! [`StoreLoad`] disposition, the effective global search settings, the
//! in-memory command-history provider (the single runtime authority ranking
//! consumes), and the bounded nonblocking persist path for recorded
//! activations. It is the modular app-owned seam later lanes attach their
//! repository adapters to (F2b's downloads adapter consumes the same
//! [`Repository`] through [`AppStore::repository`]), so command-only global
//! state is never hard-coded here.
//!
//! History policy (v1, deterministic — inferred from the v2 schema
//! contract, which carries `visit_count` and a monotonic `last_visited`
//! per normalized-URL row):
//! - one provider entry per normalized URL (dedupe by URL; the schema has
//!   no URL-duplication invariant, so the app-owned provider is the dedupe
//!   authority);
//! - a recorded activation either bumps the existing entry (visit count and
//!   recency) or appends a new one with `visit_count = 1` and the next
//!   monotonic recency marker — since blueprint D10 the wall-clock time in
//!   milliseconds, never below the previous marker (`history_ops`);
//! - every accepted record is persisted: this slice introduces **no
//!   retention cap**. The unbounded-retention question is deferred in
//!   Phase 7 (see the phase checklist); no product bound is invented here.
//!   The 10,000-entry scale exists only as the test/performance corpus
//!   size.
//!
//! Persist is a bounded nonblocking write: each accepted activation pokes a
//! capacity-1 channel; the background writer task (spawned on GPUI's
//! background executor by the composition root) coalesces pending pokes and
//! replaces the persisted collection in one transaction outside the render
//! path. A full channel means a persist is already pending — an accepted
//! coalesced signal, never an error, and never a queue that grows with
//! visits. Ranking and render never touch SQLite: they consume the
//! in-memory provider through [`AppStore::with_history`], which is loaded
//! once at startup and updated only after accepted activations.
//!
//! Schema v2 has no migrations and no backward compatibility (product
//! decision 2026-08-08): a mismatched or corrupt store is discarded and
//! recreated empty by the repository — a normal, logged, non-fatal startup
//! path, never a partial load and never a silent conversion. The disposition
//! ([`AppStore::store_load`]) is surfaced truthfully in the startup log and
//! the testkit snapshot. A fatal open failure (the store cannot even be
//! created) logs the typed error and continues with an ephemeral in-memory
//! store ([`AppStore::ephemeral`]) whose writes are refused with a warning —
//! never a silent fallback to a fabricated disposition.

use std::path::Path;
use std::sync::{Arc, Mutex};

use bbr_command::corpus::HistoryEntry;
use bbr_command::id::HistoryId;
use bbr_command::search_settings::engine::{CustomUrlTemplate, SearchEngine};
use bbr_command::search_settings::settings::SearchSettingsV1;
use bbr_command::search_settings::store::{LoadOutcome, ResetOutcome, SettingsStore, StoreOutcome};
use bbr_persistence::{
    BrowserState, CommandHistoryEntry, PersistenceError, Repository, SearchEngineId,
    SearchSettings, StoreLoad,
};
use gpui::BackgroundExecutor;

use self::saved_pages::{provider_order, SavedPagesProvider};

pub(crate) mod appearance;
pub(crate) mod browser;
pub(crate) mod browser_writer;
pub(crate) mod history_ops;
pub(crate) mod saved_pages;
pub(crate) mod saved_pages_policy;
pub(crate) mod saved_pages_store;

/// The persistence store file name inside the profile directory. The E2E
/// harness seeds stores at exactly this path before spawning the app.
pub const STORE_FILE_NAME: &str = "bbr.sqlite";

/// The app-owned persistence composition (see the module docs).
pub struct AppStore {
    /// The path-based repository. `None` only for the ephemeral fallback
    /// after a fatal open failure; every normal store owns it, and later
    /// lanes (F2b's downloads adapter) attach their repository methods
    /// through it.
    repository: Option<Repository>,
    /// The startup store disposition: `Some(Loaded)` or
    /// `Some(Discarded { reason })` on every normal path, `None` only for
    /// the ephemeral fallback. Surfaced truthfully in the startup log and
    /// the testkit snapshot — never silently converted.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub store_load: Option<StoreLoad>,
    /// Blueprint D17: why the last session did not come back, if it did
    /// not (the card explains it once instead of starting silently empty).
    pub recovery: Option<StoreRecovery>,
    /// Effective global search settings (persisted row; the v2 exactly-one-
    /// row invariant means the startup restore always yields a value, with
    /// defaults on a fresh or incoherent record).
    settings: SearchSettingsV1,
    /// The in-memory command-history provider: the single runtime authority
    /// the ranking corpus maps. Shared with the background writer so a
    /// persist never blocks the main thread.
    history: Arc<Mutex<Vec<HistoryEntry>>>,
    /// The next stable history-record id (above the persisted maximum, so
    /// identities never collide across restarts).
    next_id: u64,
    /// The next monotonic recency marker (above the persisted maximum, so
    /// new visits always rank as more recent than restored ones).
    next_recency: u64,
    /// The restored terminal download history (DESIGN.md §17.10 settled):
    /// the terminal records loaded from the startup snapshot, in persisted
    /// order. The app composition restores them into the downloads
    /// controller before the first frame; live attempts are never
    /// persisted, so this collection holds terminal records only.
    downloads: Vec<bbr_persistence::DownloadRecord>,
    /// Bounded nonblocking persist: one poke per accepted activation; the
    /// background writer coalesces and writes the latest provider state.
    write_tx: smol::channel::Sender<()>,
    /// The writer's channel receiver, taken by [`AppStore::spawn_writer`]
    /// (the composition root spawns the writer after `open`; unit tests
    /// drive persistence synchronously through `flush_persistence`).
    write_rx: Option<smol::channel::Receiver<()>>,
    /// The restored browser structural state (profiles/spaces/tabs plus the
    /// window/sidebar presentation rows) the app composes at startup into
    /// the chrome-first restore. Empty on a fresh or discarded store.
    restored_browser: BrowserState,
    /// The latest browser-state poke (shared with the background writer):
    /// the app replaces it on every structural mutation, so the writer and
    /// the quit flush always persist the current model.
    browser_slot: std::sync::Arc<std::sync::Mutex<Option<BrowserState>>>,
    /// Bounded nonblocking browser-state persist: one poke per structural
    /// mutation; the background writer coalesces and replaces the browser
    /// tables in one transaction.
    browser_tx: smol::channel::Sender<()>,
    /// The browser-state channel receiver, taken by
    /// [`AppStore::spawn_browser_writer`].
    browser_rx: Option<smol::channel::Receiver<()>>,
    /// The in-memory Saved Page provider (B2 packet
    /// `saved-pages-actions-command`): the single runtime authority ranking
    /// and the command consume, plus its bounded nonblocking writer. Loaded
    /// once at startup from the schema-v3 repository; updated only after
    /// accepted save/remove mutations; never queried from SQLite during
    /// ranking or render.
    saved_pages: SavedPagesProvider,
}

/// Blueprint D17: a startup that could not restore the last session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreRecovery {
    /// The store was unreadable; it was kept aside at `kept` (if the copy
    /// could be written) and BBR started empty.
    Unreadable { kept: Option<std::path::PathBuf> },
    /// The store was written by a newer BBR and left unchanged; this
    /// session runs in memory.
    Newer { found: u32 },
}

impl AppStore {
    /// Opens (or creates) the store at `{profile_dir}/bbr.sqlite`, restores
    /// the v2 snapshot, records the truthful [`StoreLoad`] disposition in
    /// the startup log, and returns the composed store. The bounded
    /// nonblocking writer is spawned separately by the composition root
    /// ([`AppStore::spawn_writer`]) so the store stays constructible and
    /// testable without a GPUI executor.
    ///
    /// Schema v2 has no migrations and no backward compatibility: a store
    /// whose stamp or structure does not match is discarded and recreated
    /// empty by the repository — a normal, logged, non-fatal startup path,
    /// never a partial load and never a silent conversion. A fatal storage
    /// error (the store cannot be opened or created at all) returns the
    /// typed error; the composition root logs it and falls back to
    /// [`AppStore::ephemeral`].
    pub fn open(profile_dir: &Path) -> Result<AppStore, PersistenceError> {
        let repository = Repository::new(profile_dir.join(STORE_FILE_NAME));
        let outcome = repository.initialize_and_restore()?;
        let store_load = outcome.store;
        // The disposition is the log surface: a discarded legacy/mismatched
        // store is a normal, truthful, non-fatal startup path.
        log::info!("persistence store: {store_load:?}");
        let settings = settings_to_v1(&outcome.snapshot.settings);
        let loaded = outcome
            .snapshot
            .command_history
            .iter()
            .map(history_to_provider)
            .collect::<Vec<_>>();
        let next_id = loaded.iter().map(|entry| entry.id.0).max().unwrap_or(0) + 1;
        let next_recency = loaded
            .iter()
            .map(|entry| entry.last_visited)
            .max()
            .unwrap_or(0)
            + 1;
        // Capacity-1 poke queue: a full channel means a persist is already
        // pending (an accepted coalesced signal), so the queue never grows
        // with rapid visits and the writer never misses the latest state.
        let (write_tx, write_rx) = smol::channel::bounded::<()>(1);
        let (browser_tx, browser_rx) = smol::channel::bounded::<()>(1);
        let restored_browser = BrowserState {
            profiles: outcome.snapshot.profiles,
            windows: outcome.snapshot.windows,
            favorites: outcome.snapshot.favorites,
        };
        let next_bookmark_id = outcome
            .snapshot
            .saved_pages
            .iter()
            .map(|page| page.id)
            .max()
            .unwrap_or(0)
            + 1;
        // The provider's deterministic presentation order: the persisted
        // dense order is the B1 write order, so the loaded snapshot is
        // re-presented most-recently-updated first (the one allowed
        // presentation rule, never a second sort).
        let mut saved_pages = outcome.snapshot.saved_pages;
        provider_order(&mut saved_pages);
        Ok(AppStore {
            saved_pages: SavedPagesProvider::new(
                Some(repository.clone()),
                saved_pages,
                outcome.snapshot.collections,
                next_bookmark_id,
            ),
            repository: Some(repository),
            recovery: match (&store_load, outcome.kept_aside) {
                (_, Some(kept)) => Some(StoreRecovery::Unreadable { kept: Some(kept) }),
                (
                    StoreLoad::Discarded {
                        reason: bbr_persistence::StoreDiscardReason::Corrupt { .. },
                    },
                    None,
                ) => Some(StoreRecovery::Unreadable { kept: None }),
                _ => None,
            },
            store_load: Some(store_load),
            settings,
            history: Arc::new(Mutex::new(loaded)),
            next_id,
            next_recency,
            downloads: outcome.snapshot.downloads,
            write_tx,
            write_rx: Some(write_rx),
            restored_browser,
            browser_slot: Arc::new(Mutex::new(None)),
            browser_tx,
            browser_rx: Some(browser_rx),
        })
    }

    /// The ephemeral fallback after a fatal open failure: history and
    /// settings still work in memory for the session (defaults), but no
    /// store exists — every persist is refused with a logged warning and
    /// [`AppStore::store_load`] is `None`, the truthful "no store" surface.
    pub fn ephemeral() -> AppStore {
        // Same capacity-1 poke shape as the real store (see `open`).
        let (write_tx, write_rx) = smol::channel::bounded::<()>(1);
        let (browser_tx, browser_rx) = smol::channel::bounded::<()>(1);
        AppStore {
            repository: None,
            recovery: None,
            store_load: None,
            settings: SearchSettingsV1::defaults(),
            history: Arc::new(Mutex::new(Vec::new())),
            next_id: 1,
            next_recency: 1,
            downloads: Vec::new(),
            write_tx,
            write_rx: Some(write_rx),
            restored_browser: BrowserState::default(),
            browser_slot: Arc::new(Mutex::new(None)),
            browser_tx,
            browser_rx: Some(browser_rx),
            saved_pages: SavedPagesProvider::new(None, Vec::new(), Vec::new(), 1),
        }
    }

    /// Spawns the bounded nonblocking writer on the background executor:
    /// each poke coalesces (only the latest provider state is ever written)
    /// and replaces the persisted collection in one transaction. No-op on
    /// the ephemeral store (nothing to persist to). Called exactly once by
    /// the composition root after `open`.
    pub fn spawn_writer(&mut self, background: &BackgroundExecutor) {
        let Some(repository) = self.repository.clone() else {
            log::warn!("ephemeral store: background history writer not spawned");
            return;
        };
        let Some(write_rx) = self.write_rx.take() else {
            return;
        };
        let history = self.history.clone();
        background
            .spawn(async move {
                while write_rx.recv().await.is_ok() {
                    // Coalesce: drain any redundant pokes already queued
                    // (capacity 1, so at most one); the collection is
                    // replaced whole, so only the latest state matters.
                    while write_rx.try_recv().is_ok() {}
                    let entries = {
                        let guard = history.lock().expect("history provider lock");
                        guard.iter().map(provider_to_persisted).collect::<Vec<_>>()
                    };
                    if let Err(error) = repository.store_command_history(&entries) {
                        log::warn!("command-history persist failed: {error:?}");
                    }
                }
            })
            .detach();
    }

    /// The path-based repository, exposed for modular composition: later
    /// lanes (F2b's downloads adapter) attach their own repository methods
    /// to the same shared bootstrap instead of hard-coding global state.
    /// `None` only on the ephemeral fallback.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn repository(&self) -> Option<&Repository> {
        self.repository.as_ref()
    }

    /// The restored terminal download history (the persisted records the
    /// startup snapshot loaded; empty on the ephemeral fallback and on a
    /// fresh store). The app composition maps these onto the downloads
    /// controller before the first frame (DESIGN.md §17.10 settled: terminal
    /// history persists through restarts).
    pub fn restored_downloads(&self) -> &[bbr_persistence::DownloadRecord] {
        &self.downloads
    }

    /// Durably replaces the persisted terminal download history with
    /// `records` in one transaction — the download-specific write path the
    /// app calls outside render/ranking (on an accepted terminal transition,
    /// on Clear, and on a row Delete), so a cleared or deleted row can never
    /// return from the store (DESIGN.md §17.7/§17.8/§17.10 settled). The
    /// records arrive in persisted history order (ascending terminal order;
    /// `position` = index). The ephemeral fallback refuses the write with a
    /// logged warning — never a silent fabricated disposition.
    pub fn replace_downloads(&self, records: &[bbr_persistence::DownloadRecord]) {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: downloads persist refused");
            return;
        };
        if let Err(error) = repository.replace_downloads(records) {
            log::warn!("downloads persist failed: {error:?}");
        }
    }

    /// Blueprint D16 (Settings › Search): choose the search engine; the
    /// effective settings change at once and the row is written when a
    /// repository exists (an ephemeral store keeps the choice in memory).
    pub fn set_search_engine(&mut self, engine: SearchEngine) {
        self.settings = SearchSettingsV1::with_engine(engine);
        let _ = self.store_settings(self.settings.clone());
    }

    /// The effective global search settings (defaults on a fresh store or
    /// on the ephemeral fallback).
    pub fn effective_settings(&self) -> &SearchSettingsV1 {
        &self.settings
    }

    /// The in-memory history provider for one ranking pass. Ranking and
    /// render never touch SQLite: they consume exactly this provider data,
    /// which is loaded once at startup and updated only after accepted
    /// activations.
    pub fn with_history<R>(&self, f: impl FnOnce(&[HistoryEntry]) -> R) -> R {
        let guard = self.history.lock().expect("history provider lock");
        f(&guard)
    }

    /// The provider entry for a history-record id (used by `OpenHistory`
    /// activation and the testkit snapshot).
    pub fn history_entry(&self, id: HistoryId) -> Option<(String, String)> {
        let guard = self.history.lock().expect("history provider lock");
        guard
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| (entry.url.clone(), entry.title.clone()))
    }

    /// Records one accepted user navigation activation into the provider:
    /// dedupe by normalized URL (bump `visit_count` + recency), otherwise
    /// append a new entry, then poke the bounded nonblocking writer. Never
    /// runs from render; the SQLite write happens on the background task.
    /// The poke is nonblocking: a full capacity-1 queue means a persist is
    /// already pending, so the signal is coalesced (the pending write reads
    /// the latest provider state) — never an error and never a growing
    /// queue.
    pub fn record_visit(&mut self, url: &str, title: &str) {
        let normalized = bbr_web::normalize_url(url);
        if normalized.is_empty() {
            return;
        }
        // Blueprint D10: the recency marker is the visit's wall-clock time
        // (never below the next monotonic marker).
        self.stamp_now();
        let mut history = self.history.lock().expect("history provider lock");
        record_visit_into(
            &mut history,
            &mut self.next_id,
            &mut self.next_recency,
            &normalized,
            title,
        );
        drop(history);
        let _ = self.write_tx.try_send(());
    }

    /// Synchronously persists the current provider state in one transaction
    /// (the deterministic flush seam and the quit flush). Off the render
    /// path; the background writer uses the identical repository write.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn flush_persistence(&self) {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: persistence flush refused");
            return;
        };
        let entries = self.persisted_entries();
        if let Err(error) = repository.store_command_history(&entries) {
            log::warn!("command-history flush failed: {error:?}");
        }
    }

    /// The provider state as persisted DTOs (whole-collection replacement).
    fn persisted_entries(&self) -> Vec<CommandHistoryEntry> {
        let guard = self.history.lock().expect("history provider lock");
        guard.iter().map(provider_to_persisted).collect()
    }
}

/// Maps the persisted settings DTO onto the portable global-v1 value. The
/// custom template is re-validated at construction (belt and suspenders —
/// the repository already validates on write and recovers on restore); a
/// template that cannot construct falls back to the DuckDuckGo default.
pub fn settings_to_v1(persisted: &SearchSettings) -> SearchSettingsV1 {
    let engine = match persisted.engine {
        SearchEngineId::DuckDuckGo => SearchEngine::DuckDuckGo,
        SearchEngineId::Google => SearchEngine::Google,
        SearchEngineId::Bing => SearchEngine::Bing,
        SearchEngineId::Ecosia => SearchEngine::Ecosia,
        SearchEngineId::Kagi => SearchEngine::Kagi,
        SearchEngineId::Custom => match persisted.custom_template.as_deref() {
            Some(template) => CustomUrlTemplate::new(template)
                .map(SearchEngine::Custom)
                .unwrap_or(SearchEngine::DuckDuckGo),
            None => SearchEngine::DuckDuckGo,
        },
    };
    SearchSettingsV1::with_engine(engine)
}

/// Maps the portable global-v1 value onto the persisted settings DTO.
pub fn settings_to_persisted(settings: &SearchSettingsV1) -> SearchSettings {
    match settings.engine() {
        SearchEngine::DuckDuckGo => SearchSettings {
            engine: SearchEngineId::DuckDuckGo,
            custom_template: None,
        },
        SearchEngine::Google => SearchSettings {
            engine: SearchEngineId::Google,
            custom_template: None,
        },
        SearchEngine::Bing => SearchSettings {
            engine: SearchEngineId::Bing,
            custom_template: None,
        },
        SearchEngine::Ecosia => SearchSettings {
            engine: SearchEngineId::Ecosia,
            custom_template: None,
        },
        SearchEngine::Kagi => SearchSettings {
            engine: SearchEngineId::Kagi,
            custom_template: None,
        },
        SearchEngine::Custom(template) => SearchSettings {
            engine: SearchEngineId::Custom,
            custom_template: Some(template.as_str().to_string()),
        },
    }
}

/// Records one accepted activation into the provider collection (the pure
/// history policy): dedupe by normalized URL, bump or append. No retention
/// cap in this slice (retention deferred in Phase 7); every accepted record
/// is kept. Pure and deterministic, so the policy is unit-testable without
/// GPUI or SQLite.
pub(crate) fn record_visit_into(
    history: &mut Vec<HistoryEntry>,
    next_id: &mut u64,
    next_recency: &mut u64,
    normalized_url: &str,
    title: &str,
) {
    let recency = *next_recency;
    *next_recency += 1;
    if let Some(entry) = history.iter_mut().find(|entry| entry.url == normalized_url) {
        entry.visit_count = entry.visit_count.saturating_add(1);
        entry.last_visited = recency;
        if !title.trim().is_empty() && entry.title.is_empty() {
            entry.title = title.trim().to_string();
        }
    } else {
        let id = *next_id;
        *next_id += 1;
        history.push(HistoryEntry::new(
            HistoryId(id),
            title.trim().to_string(),
            normalized_url.to_string(),
            1,
            recency,
        ));
    }
}

/// Maps a persisted history DTO onto the provider-neutral corpus entry.
fn history_to_provider(entry: &CommandHistoryEntry) -> HistoryEntry {
    HistoryEntry::new(
        HistoryId(entry.id),
        entry.title.clone(),
        entry.url.clone(),
        entry.visit_count,
        entry.last_visited,
    )
}

/// Maps a provider entry back onto the persisted DTO.
fn provider_to_persisted(entry: &HistoryEntry) -> CommandHistoryEntry {
    CommandHistoryEntry {
        id: entry.id.0,
        title: entry.title.clone(),
        url: entry.url.clone(),
        visit_count: entry.visit_count,
        last_visited: entry.last_visited,
    }
}

/// The bbr-command `SettingsStore` port, implemented over the persistence
/// repository (delivery-ledger decision: `bbr-persistence` owns the
/// settings-store capability through its repository methods; the app wires
/// the portable trait at startup). The v2 exactly-one-row invariant makes a
/// missing row impossible — the seeded defaults are the stored value until
/// customized — so `load` never reports `Empty`; a storage failure (or the
/// ephemeral fallback) reports `Unavailable` truthfully.
impl SettingsStore for AppStore {
    fn load_settings(&self) -> LoadOutcome {
        let Some(repository) = self.repository.as_ref() else {
            return LoadOutcome::Unavailable;
        };
        match repository.load_settings() {
            Ok(persisted) => LoadOutcome::Stored(settings_to_v1(&persisted)),
            Err(error) => {
                log::warn!("settings load failed: {error:?}");
                LoadOutcome::Unavailable
            }
        }
    }

    fn store_settings(&self, settings: SearchSettingsV1) -> StoreOutcome {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: settings store refused");
            return StoreOutcome::Rejected;
        };
        let persisted = settings_to_persisted(&settings);
        match repository.store_settings(&persisted) {
            Ok(()) => StoreOutcome::Stored,
            Err(error) => {
                log::warn!("settings store failed: {error:?}");
                StoreOutcome::Rejected
            }
        }
    }

    fn reset_settings(&self) -> ResetOutcome {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: settings reset refused");
            return ResetOutcome::Failed;
        };
        match repository.store_settings(&SearchSettings::default()) {
            Ok(()) => ResetOutcome::Reset,
            Err(error) => {
                log::warn!("settings reset failed: {error:?}");
                ResetOutcome::Failed
            }
        }
    }
}

/// The persistence DTO ↔ downloads-controller mapping (DESIGN.md §17.10
/// settled): the app-owned composition between the restored/persisted
/// terminal records and the portable `bbr-browser` controller records.
pub(crate) mod downloads;

#[cfg(test)]
mod tests;
