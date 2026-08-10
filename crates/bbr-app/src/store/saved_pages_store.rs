/// The [`AppStore`] Saved Page composition seam (B2/B3 packets): the public
/// provider operations the command authority, the Library composition, and
/// the testkit dispatch call. Split from `saved_pages.rs` so both files
/// stay under the ~500-line budget; every operation delegates to the
/// provider (`super::saved_pages`) and its pure policies.
use bbr_persistence::{CollectionRecord, SavedPageRecord, SavedPageSaveOutcome};
use gpui::BackgroundExecutor;

use super::AppStore;

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

    /// The number of flat collection records (the typed snapshot evidence
    /// for the B3 Library; asserted by the store's persistence tests).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn saved_collections_count(&self) -> usize {
        self.saved_pages
            .collections
            .lock()
            .expect("saved pages collections lock")
            .len()
    }

    /// The title of one flat collection (the staged rename draft seed the
    /// pointer cycle needs; `None` for an unknown identity).
    pub fn saved_collection_title(&self, id: u64) -> Option<String> {
        self.saved_pages.with(|_, collections| {
            collections
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.title.clone())
        })
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

    /// Removes many records in one poke (the B3 Library multi-select
    /// destructive remove; idempotent per id). Never runs from render.
    pub fn remove_saved_pages(&mut self, ids: &[u64]) {
        self.saved_pages.remove_many(ids);
    }

    /// Sets (or clears) the user label of one record — the B3 Library
    /// rename — and pokes the bounded nonblocking writer. Never runs from
    /// render.
    pub fn rename_saved_page(&mut self, id: u64, label: &str) {
        self.saved_pages.set_label(id, Some(label));
    }

    /// Creates one flat collection in the in-memory provider (the B3
    /// Library new-collection flow) and pokes the bounded nonblocking
    /// writer. Returns the new stable [`CollectionId`] or `None` for an
    /// invalid title. Never runs from render.
    pub fn create_saved_collection(&mut self, title: &str) -> Option<u64> {
        self.saved_pages.create_collection(title)
    }

    /// Renames one flat collection (the B3 Library rename-collection flow)
    /// and pokes the bounded nonblocking writer. An unknown identity or an
    /// invalid title is a no-op. Never runs from render.
    pub fn rename_saved_collection(&mut self, id: u64, title: &str) {
        self.saved_pages.rename_collection(id, title);
    }

    /// Removes one flat collection (the B3 Library remove-collection flow):
    /// member records are moved out first, then the collection row is
    /// removed, so the in-memory state stays internally consistent. Never
    /// runs from render.
    pub fn remove_saved_collection(&mut self, id: u64) {
        self.saved_pages.remove_collection(id);
    }

    /// Moves many records into (or out of) one flat collection in one poke
    /// (the B3 Library multi-select collection move). Unknown record ids
    /// are skipped; `None` clears the membership. Never runs from render.
    pub fn move_saved_pages_to_collection(&mut self, ids: &[u64], collection: Option<u64>) {
        self.saved_pages.move_to_collection(ids, collection);
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
