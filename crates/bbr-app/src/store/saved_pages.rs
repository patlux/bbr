//! Saved Pages provider lifecycle (B2 packet `saved-pages-actions-command`):
//! the app-owned in-memory Saved Page snapshot and its bounded nonblocking
//! writer.
//!
//! The provider is the **single runtime authority** ranking and the
//! command consume: it is loaded once at startup from the schema-v3
//! repository (B1), updated only after accepted save/remove mutations, and
//! never queried from SQLite during ranking or render. Mutations persist
//! through the same bounded nonblocking writer discipline as the
//! command-history and browser-state providers — the app replaces a shared
//! slot and pokes a capacity-1 channel; the background writer coalesces
//! and replaces the persisted Saved Page collection in one transaction
//! ([`Repository::replace_saved_pages`]); the testkit flush and the
//! production owning-window-close quit flush write synchronously through
//! the identical repository call. The flat collections are loaded once at
//! startup and mutated only by the B3 Library, so the combined writer
//! persists the records **and** the collections together (the strict
//! collection foreign key stays satisfied by the move-out-then-remove
//! discipline in this module).
//!
//! The pure mutation/ordering/entry policies live in
//! [`super::saved_pages_policy`] (unit-tested there); this module owns the
//! provider lifecycle, the bounded nonblocking writer, and the [`AppStore`]
//! composition seam. Re-exports [`provider_order`] and
//! [`saved_page_to_entry`] for the store composition and the command
//! corpus.

use std::sync::{Arc, Mutex};

use bbr_persistence::{CollectionRecord, Repository, SavedPageRecord, SavedPageSaveOutcome};
use gpui::BackgroundExecutor;

use super::saved_pages_policy::{now_secs, remove_saved_page_into, save_saved_page_into};

pub(crate) use super::saved_pages_policy::{provider_order, saved_page_to_entry};

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
    /// The flat collections loaded once at startup and mutated only by the
    /// B3 Library; shared with the background writer so a persist never
    /// blocks the main thread.
    pub(crate) collections: Arc<Mutex<Vec<CollectionRecord>>>,
    /// The next stable Saved Page id (above the persisted maximum, so
    /// identities never collide across restarts).
    next_bookmark_id: u64,
    /// The next stable flat-collection id (above the persisted maximum, so
    /// collection identities never collide across restarts).
    next_collection_id: u64,
    /// Bounded nonblocking persist: one poke per accepted mutation; the
    /// background writer coalesces and replaces the Saved Page collection
    /// in one transaction.
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
        let next_collection_id = collections
            .iter()
            .map(|collection| collection.id)
            .max()
            .unwrap_or(0)
            + 1;
        let (tx, rx) = smol::channel::bounded::<()>(1);
        Self {
            pages: Arc::new(Mutex::new(pages)),
            collections: Arc::new(Mutex::new(collections)),
            next_bookmark_id,
            next_collection_id,
            tx,
            rx: Some(rx),
            repository,
        }
    }

    /// The restored Saved Page snapshot at startup (empty on a fresh or
    /// discarded store): the in-memory provider ranking and the command
    /// consume. Loaded once; never re-read from SQLite during ranking or
    /// render.
    pub(crate) fn with<R>(
        &self,
        f: impl FnOnce(&[SavedPageRecord], &[CollectionRecord]) -> R,
    ) -> R {
        let pages = self.pages.lock().expect("saved pages provider lock");
        let collections = self
            .collections
            .lock()
            .expect("saved pages collections lock");
        f(&pages, &collections)
    }

    /// The provider record for a stable `BookmarkId`, when it still exists
    /// (activation revalidation; a stale/deleted id returns `None`).
    pub(crate) fn by_id(&self, id: u64) -> Option<SavedPageRecord> {
        let pages = self.pages.lock().expect("saved pages provider lock");
        pages.iter().find(|page| page.id == id).cloned()
    }

    /// The provider record for a canonical URL key, when it exists. The key
    /// must be produced by the `bbr-domain` canonical seam (the one
    /// canonical-URL equality authority); the provider compares it
    /// byte-for-byte.
    pub(crate) fn by_url(&self, canonical_url: &str) -> Option<SavedPageRecord> {
        let pages = self.pages.lock().expect("saved pages provider lock");
        pages
            .iter()
            .find(|page| page.canonical_url == canonical_url)
            .cloned()
    }

    /// The number of provider records (the typed snapshot evidence).
    pub(crate) fn count(&self) -> usize {
        self.pages.lock().expect("saved pages provider lock").len()
    }

    /// Saves one canonical URL into the in-memory provider (idempotent,
    /// B1 duplicate semantics) and pokes the bounded nonblocking writer.
    /// The canonical key must already be validated by the caller through
    /// the `bbr-domain` seam. Never runs from render.
    pub(crate) fn save(&mut self, canonical_url: &str, title: &str) -> SavedPageSaveOutcome {
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
    pub(crate) fn remove(&mut self, id: u64) {
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            remove_saved_page_into(&mut pages, id);
        }
        let _ = self.tx.try_send(());
    }

    /// Removes many records from the in-memory provider in one poke (the
    /// B3 Library multi-select destructive remove; idempotent per id — an
    /// unknown identity is a no-op). Never runs from render.
    pub(crate) fn remove_many(&mut self, ids: &[u64]) {
        let mut changed = false;
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            for id in ids {
                changed |= remove_saved_page_into(&mut pages, *id);
            }
        }
        if changed {
            let _ = self.tx.try_send(());
        }
    }

    /// Sets (or clears) the user label of one record in the in-memory
    /// provider (the B3 Library rename) and pokes the bounded nonblocking
    /// writer. The label follows the B1 bounds (non-empty, ≤256 bytes, no
    /// control bytes); an unknown identity is a no-op. `updated_at` is
    /// clamped up to the stored `created_at` so the monotonic invariant can
    /// never regress in memory, exactly like the repository write. Never
    /// runs from render.
    pub(crate) fn set_label(&mut self, id: u64, label: Option<&str>) {
        let Some(label) = label.map(str::to_owned) else {
            return;
        };
        if label.is_empty()
            || label.len() > bbr_domain::MAX_USER_LABEL_LEN
            || label
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return;
        }
        let mut changed = false;
        let now = now_secs();
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            if let Some(page) = pages.iter_mut().find(|page| page.id == id) {
                if page.user_label.as_deref() != Some(label.as_str()) {
                    page.user_label = Some(label);
                    page.updated_at = page.updated_at.max(now).max(page.created_at);
                    changed = true;
                }
            }
            if changed {
                // The deterministic presentation order: a rename may bump
                // `updated_at`, so the collection is re-presented
                // most-recently-updated first.
                provider_order(&mut pages);
            }
        }
        if changed {
            let _ = self.tx.try_send(());
        }
    }

    /// Creates one flat collection in the in-memory provider (the B3
    /// Library new-collection flow) and pokes the bounded nonblocking
    /// writer. Returns the new stable [`CollectionId`]. A bounded, non-empty
    /// title is required; an invalid title creates nothing and returns
    /// `None`. Never runs from render.
    pub(crate) fn create_collection(&mut self, title: &str) -> Option<u64> {
        if title.is_empty()
            || title.len() > bbr_domain::MAX_COLLECTION_TITLE_LEN
            || title
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return None;
        }
        let id = self.next_collection_id;
        self.next_collection_id += 1;
        self.collections
            .lock()
            .expect("saved pages collections lock")
            .push(CollectionRecord {
                id,
                title: title.to_string(),
            });
        let _ = self.tx.try_send(());
        Some(id)
    }

    /// Renames one flat collection in the in-memory provider (the B3
    /// Library rename-collection flow) and pokes the bounded nonblocking
    /// writer. An unknown identity or an invalid title is a no-op. Never
    /// runs from render.
    pub(crate) fn rename_collection(&mut self, id: u64, title: &str) {
        if title.is_empty()
            || title.len() > bbr_domain::MAX_COLLECTION_TITLE_LEN
            || title
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return;
        }
        let mut changed = false;
        {
            let mut collections = self
                .collections
                .lock()
                .expect("saved pages collections lock");
            if let Some(collection) = collections
                .iter_mut()
                .find(|collection| collection.id == id)
            {
                if collection.title != title {
                    collection.title = title.to_string();
                    changed = true;
                }
            }
        }
        if changed {
            let _ = self.tx.try_send(());
        }
    }

    /// Removes one flat collection from the in-memory provider (the B3
    /// Library remove-collection flow). Strict membership: every member
    /// record is moved out **first** (its `collection_id` is cleared) and
    /// then the collection row is removed, so the in-memory state stays
    /// internally consistent and the combined persisted write can never
    /// dangle. An unknown identity is a no-op. Never runs from render.
    pub(crate) fn remove_collection(&mut self, id: u64) {
        {
            let mut collections = self
                .collections
                .lock()
                .expect("saved pages collections lock");
            if !collections.iter().any(|collection| collection.id == id) {
                return;
            }
            collections.retain(|collection| collection.id != id);
        }
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            let mut any_member = false;
            for page in pages.iter_mut() {
                if page.collection_id == Some(id) {
                    page.collection_id = None;
                    any_member = true;
                }
            }
            if any_member {
                provider_order(&mut pages);
            }
            drop(pages);
            let _ = self.tx.try_send(());
        }
    }

    /// Moves many records into (or out of) one flat collection in one poke
    /// (the B3 Library multi-select collection move). Unknown record ids
    /// are skipped; `None` clears the membership. Never runs from render.
    pub(crate) fn move_to_collection(&mut self, ids: &[u64], collection: Option<u64>) {
        if let Some(collection) = collection {
            let known = self
                .collections
                .lock()
                .expect("saved pages collections lock")
                .iter()
                .any(|existing| existing.id == collection);
            if !known {
                return;
            }
        }
        let mut changed = false;
        {
            let mut pages = self.pages.lock().expect("saved pages provider lock");
            for page in pages.iter_mut() {
                if ids.contains(&page.id) && page.collection_id != collection {
                    page.collection_id = collection;
                    changed = true;
                }
            }
        }
        if changed {
            let _ = self.tx.try_send(());
        }
    }

    /// Spawns the bounded nonblocking Saved Pages writer on the background
    /// executor: each poke coalesces (only the latest provider state is ever
    /// written) and replaces the persisted Saved Page state — the records
    /// **and** the flat collections — in one transaction. No-op on the
    /// ephemeral store. Called exactly once by the composition root after
    /// `open`, alongside the history and browser-state writers.
    pub(crate) fn spawn_writer(&mut self, background: &BackgroundExecutor) {
        let Some(repository) = self.repository.clone() else {
            log::warn!("ephemeral store: background saved-pages writer not spawned");
            return;
        };
        let Some(rx) = self.rx.take() else {
            return;
        };
        let pages = self.pages.clone();
        let collections = self.collections.clone();
        background
            .spawn(async move {
                while rx.recv().await.is_ok() {
                    // Coalesce: drain any redundant pokes already queued
                    // (capacity 1, so at most one); the combined state is
                    // replaced whole, so only the latest state matters.
                    while rx.try_recv().is_ok() {}
                    let records = pages.lock().expect("saved pages provider lock").clone();
                    let set = collections
                        .lock()
                        .expect("saved pages collections lock")
                        .clone();
                    if let Err(error) = repository.replace_saved_snapshot(&records, &set) {
                        log::warn!("saved-pages persist failed: {error:?}");
                    }
                }
            })
            .detach();
    }

    /// Synchronously persists the current provider state in one transaction
    /// (the deterministic flush seam and the quit flush). Off the render
    /// path; the background writer uses the identical repository write.
    pub(crate) fn flush(&self) {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: saved-pages flush refused");
            return;
        };
        let records = self
            .pages
            .lock()
            .expect("saved pages provider lock")
            .clone();
        let set = self
            .collections
            .lock()
            .expect("saved pages collections lock")
            .clone();
        if let Err(error) = repository.replace_saved_snapshot(&records, &set) {
            log::warn!("saved-pages flush failed: {error:?}");
        }
    }
}
