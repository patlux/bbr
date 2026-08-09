//! Saved Pages provider lifecycle (B2 packet `saved-pages-actions-command`):
//! the app-owned in-memory Saved Page snapshot and its bounded nonblocking
//! writer.
//!
//! The provider is the **single runtime authority** ranking and the command
//! consume: it is loaded once at startup from the schema-v3 repository
//! (B1), updated only after accepted save/remove mutations, and never
//! queried from SQLite during ranking or render. Mutations persist through
//! the same bounded nonblocking writer discipline as the command-history
//! and browser-state providers — the app replaces a shared slot and pokes a
//! capacity-1 channel; the background writer coalesces and replaces the
//! persisted Saved Page collection in one transaction
//! ([`Repository::replace_saved_pages`]); the testkit flush and the
//! production owning-window-close quit flush write synchronously through the
//! identical repository call. The flat collections are loaded once at
//! startup and never mutated by B2, so
//! the writer persists only the page rows (the strict collection foreign
//! key stays satisfied by the untouched collections table).
//!
//! Duplicate semantics mirror the B1 repository contract exactly: saving a
//! canonical URL that already has an active record returns the existing
//! `BookmarkId`, creates no second row, preserves `created_at`, never
//! touches a user label, and updates the captured title only when no user
//! label exists and the title actually changed (a byte-identical re-save
//! performs no write). The stored `updated_at` can never regress below the
//! stored `created_at`. Remove is idempotent.

use std::sync::{Arc, Mutex};

use bbr_persistence::{CollectionRecord, Repository, SavedPageRecord, SavedPageSaveOutcome};
use gpui::BackgroundExecutor;

use super::AppStore;

/// The pure save policy (the in-memory mirror of the B1 repository
/// duplicate semantics): dedupe by canonical URL, return the existing
/// identity, and follow the settled title/label update rule. Deterministic
/// and unit-testable without GPUI or SQLite.
pub(crate) fn save_saved_page_into(
    pages: &mut Vec<SavedPageRecord>,
    next_id: &mut u64,
    canonical_url: &str,
    title: &str,
    now: u64,
) -> SavedPageSaveOutcome {
    if let Some(existing) = pages
        .iter_mut()
        .find(|page| page.canonical_url == canonical_url)
    {
        // Duplicate: the existing identity wins; the user label is never
        // touched by a save; the captured title updates only when no user
        // label exists and the title actually changed. `updated_at` is
        // clamped up to `created_at` so the monotonic invariant can never
        // regress.
        if existing.user_label.is_none() && existing.captured_title != title {
            existing.captured_title = title.to_string();
            existing.updated_at = existing.updated_at.max(now).max(existing.created_at);
        }
        return SavedPageSaveOutcome {
            id: existing.id,
            existed: true,
        };
    }
    let id = *next_id;
    *next_id += 1;
    pages.push(SavedPageRecord {
        id,
        canonical_url: canonical_url.to_string(),
        captured_title: title.to_string(),
        user_label: None,
        created_at: now,
        updated_at: now,
        collection_id: None,
    });
    SavedPageSaveOutcome { id, existed: false }
}

/// The pure remove policy: idempotent — removing an unknown identity is a
/// no-op success. Returns `true` when a record was actually removed.
pub(crate) fn remove_saved_page_into(pages: &mut Vec<SavedPageRecord>, id: u64) -> bool {
    let before = pages.len();
    pages.retain(|page| page.id != id);
    pages.len() != before
}

/// The app-owned in-memory Saved Page provider (B2 packet
/// `saved-pages-actions-command`): the single runtime authority ranking and
/// the command consume, plus its bounded nonblocking writer. Owns the
/// loaded records, the flat collections, the next stable id, the poke
/// channel, and the optional repository handle (shared with the store; the
/// writer only ever reads the in-memory provider). Encapsulated so the
/// store composition stays one field.
pub(crate) struct SavedPagesProvider {
    /// The loaded records; shared with the background writer so a persist
    /// never blocks the main thread. Never queried from SQLite during
    /// ranking or render.
    pages: Arc<Mutex<Vec<SavedPageRecord>>>,
    /// The flat collections loaded once at startup (B2 never mutates them;
    /// the strict collection foreign key stays satisfied by the untouched
    /// collections table).
    collections: Arc<Mutex<Vec<CollectionRecord>>>,
    /// The next stable Saved Page id (above the persisted maximum, so
    /// identities never collide across restarts).
    next_bookmark_id: u64,
    /// Bounded nonblocking persist: one poke per accepted mutation; the
    /// background writer coalesces and replaces the page collection in one
    /// transaction.
    tx: smol::channel::Sender<()>,
    /// The writer's channel receiver, taken by [`SavedPagesProvider::spawn_writer`].
    rx: Option<smol::channel::Receiver<()>>,
    /// The optional repository handle for the background writer and the
    /// synchronous flush. `None` only on the ephemeral store.
    repository: Option<Repository>,
}

impl SavedPagesProvider {
    /// Builds the provider from the restored startup snapshot (or an empty
    /// provider on a fresh/discarded store) and creates the capacity-1 poke
    /// channel. The caller owns presentation ordering via
    /// [`provider_order`].
    pub(crate) fn new(
        repository: Option<Repository>,
        pages: Vec<SavedPageRecord>,
        collections: Vec<CollectionRecord>,
        next_bookmark_id: u64,
    ) -> Self {
        let (tx, rx) = smol::channel::bounded::<()>(1);
        Self {
            pages: Arc::new(Mutex::new(pages)),
            collections: Arc::new(Mutex::new(collections)),
            next_bookmark_id,
            tx,
            rx: Some(rx),
            repository,
        }
    }

    /// The restored Saved Page snapshot at startup (empty on a fresh or
    /// discarded store): the in-memory provider ranking and the command
    /// consume. Loaded once; never re-read from SQLite during ranking or
    /// render.
    fn with<R>(&self, f: impl FnOnce(&[SavedPageRecord], &[CollectionRecord]) -> R) -> R {
        let pages = self.pages.lock().expect("saved pages provider lock");
        let collections = self
            .collections
            .lock()
            .expect("saved pages collections lock");
        f(&pages, &collections)
    }

    /// The provider record for a stable `BookmarkId`, when it still exists
    /// (activation revalidation; a stale/deleted id returns `None`).
    fn by_id(&self, id: u64) -> Option<SavedPageRecord> {
        let pages = self.pages.lock().expect("saved pages provider lock");
        pages.iter().find(|page| page.id == id).cloned()
    }

    /// The provider record for a canonical URL key, when it exists. The key
    /// must be produced by the `bbr-domain` canonical seam (the one
    /// canonical-URL equality authority); the provider compares it
    /// byte-for-byte.
    fn by_url(&self, canonical_url: &str) -> Option<SavedPageRecord> {
        let pages = self.pages.lock().expect("saved pages provider lock");
        pages
            .iter()
            .find(|page| page.canonical_url == canonical_url)
            .cloned()
    }

    /// The number of provider records (the typed snapshot evidence).
    fn count(&self) -> usize {
        self.pages.lock().expect("saved pages provider lock").len()
    }

    /// Saves one canonical URL into the in-memory provider (idempotent,
    /// B1 duplicate semantics) and pokes the bounded nonblocking writer.
    /// The canonical key must already be validated by the caller through
    /// the `bbr-domain` seam. Never runs from render.
    fn save(&mut self, canonical_url: &str, title: &str) -> SavedPageSaveOutcome {
        let now = now_secs();
        let outcome = {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            let outcome = save_saved_page_into(
                &mut pages,
                &mut self.next_bookmark_id,
                canonical_url,
                title,
                now,
            );
            // The provider's deterministic presentation order: a save may
            // bump `updated_at` (title update), so the collection is
            // re-presented most-recently-updated first.
            provider_order(&mut pages);
            outcome
        };
        let _ = self.tx.try_send(());
        outcome
    }

    /// Removes one record from the in-memory provider (idempotent) and
    /// pokes the bounded nonblocking writer. Never runs from render.
    fn remove(&mut self, id: u64) {
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            remove_saved_page_into(&mut pages, id);
        }
        let _ = self.tx.try_send(());
    }

    /// Spawns the bounded nonblocking Saved Pages writer on the background
    /// executor: each poke coalesces (only the latest provider state is ever
    /// written) and replaces the persisted Saved Page collection in one
    /// transaction. No-op on the ephemeral store. Called exactly once by the
    /// composition root after `open`, alongside the history and
    /// browser-state writers.
    pub(crate) fn spawn_writer(&mut self, background: &BackgroundExecutor) {
        let Some(repository) = self.repository.clone() else {
            log::warn!("ephemeral store: background saved-pages writer not spawned");
            return;
        };
        let Some(rx) = self.rx.take() else {
            return;
        };
        let pages = self.pages.clone();
        background
            .spawn(async move {
                while rx.recv().await.is_ok() {
                    // Coalesce: drain any redundant pokes already queued
                    // (capacity 1, so at most one); the collection is
                    // replaced whole, so only the latest state matters.
                    while rx.try_recv().is_ok() {}
                    let records = pages.lock().expect("saved pages provider lock").clone();
                    if let Err(error) = repository.replace_saved_pages(&records) {
                        log::warn!("saved-pages persist failed: {error:?}");
                    }
                }
            })
            .detach();
    }

    /// Synchronously persists the current provider state in one transaction
    /// (the deterministic flush seam and the quit flush). Off the render
    /// path; the background writer uses the identical repository write.
    fn flush(&self) {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: saved-pages flush refused");
            return;
        };
        let records = self
            .pages
            .lock()
            .expect("saved pages provider lock")
            .clone();
        if let Err(error) = repository.replace_saved_pages(&records) {
            log::warn!("saved-pages flush failed: {error:?}");
        }
    }
}

impl AppStore {
    /// The restored Saved Page snapshot at startup (empty on a fresh or
    /// discarded store): the in-memory provider ranking and the command
    /// consume. Loaded once; never re-read from SQLite during ranking or
    /// render.
    pub fn with_saved_pages<R>(
        &self,
        f: impl FnOnce(&[SavedPageRecord], &[CollectionRecord]) -> R,
    ) -> R {
        self.saved_pages.with(f)
    }

    /// The provider record for a stable `BookmarkId`, when it still exists
    /// (activation revalidation; a stale/deleted id returns `None`).
    pub fn saved_page_by_id(&self, id: u64) -> Option<SavedPageRecord> {
        self.saved_pages.by_id(id)
    }

    /// The provider record for a canonical URL key, when it exists. The key
    /// must be produced by the `bbr-domain` canonical seam (the one
    /// canonical-URL equality authority); the provider compares it
    /// byte-for-byte.
    pub fn saved_page_by_url(&self, canonical_url: &str) -> Option<SavedPageRecord> {
        self.saved_pages.by_url(canonical_url)
    }

    /// The number of provider records (the typed snapshot evidence).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    pub fn saved_pages_count(&self) -> usize {
        self.saved_pages.count()
    }

    /// Saves one canonical URL into the in-memory provider (idempotent,
    /// B1 duplicate semantics) and pokes the bounded nonblocking writer.
    /// The canonical key must already be validated by the caller through
    /// the `bbr-domain` seam. Never runs from render.
    pub fn save_saved_page(&mut self, canonical_url: &str, title: &str) -> SavedPageSaveOutcome {
        self.saved_pages.save(canonical_url, title)
    }

    /// Removes one record from the in-memory provider (idempotent) and
    /// pokes the bounded nonblocking writer. Never runs from render.
    pub fn remove_saved_page(&mut self, id: u64) {
        self.saved_pages.remove(id);
    }

    /// Spawns the bounded nonblocking Saved Pages writer on the background
    /// executor: each poke coalesces (only the latest provider state is ever
    /// written) and replaces the persisted Saved Page collection in one
    /// transaction. No-op on the ephemeral store. Called exactly once by the
    /// composition root after `open`, alongside the history and
    /// browser-state writers.
    pub fn spawn_saved_pages_writer(&mut self, background: &BackgroundExecutor) {
        self.saved_pages.spawn_writer(background);
    }

    /// Synchronously persists the current provider state in one transaction
    /// (the deterministic flush seam and the production quit flush). Off the
    /// render path; the background writer uses the identical repository
    /// write.
    pub fn flush_saved_pages(&self) {
        self.saved_pages.flush();
    }
}

/// The deterministic monotonic timestamp for a save (seconds since the
/// Unix epoch; the B1 contract only requires monotonicity and the sort
/// key, never wall-clock precision).
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Maps the persisted DTO onto the provider-neutral corpus entry (the
/// settled §11.16 display-title ladder: user label, then captured title,
/// then the URL) plus the optional flat collection label.
pub(crate) fn saved_page_to_entry(
    page: &SavedPageRecord,
    collections: &[CollectionRecord],
) -> bbr_command::corpus::SavedPageEntry {
    let collection = page.collection_id.and_then(|id| {
        collections
            .iter()
            .find(|collection| collection.id == id)
            .map(|collection| collection.title.clone())
    });
    let title = page.user_label.clone().unwrap_or_else(|| {
        if page.captured_title.is_empty() {
            page.canonical_url.clone()
        } else {
            page.captured_title.clone()
        }
    });
    bbr_command::corpus::SavedPageEntry::new(
        bbr_command::id::BookmarkId(page.id),
        title,
        page.canonical_url.clone(),
        collection,
        page.captured_title.clone(),
    )
}

/// The provider's deterministic presentation order: most recently updated
/// first, then the stable id descending (the B1 sort key `(updated_at, id)`
/// ascending presented in the opposite direction — the one allowed
/// presentation rule, never a second sort).
pub(crate) fn provider_order(pages: &mut [SavedPageRecord]) {
    pages.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(b.id.cmp(&a.id)));
}
