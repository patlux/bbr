//! B1-owned Saved Pages benchmark corpus (schema v3, packet
//! `saved-pages-portable-persistence`): the ≥1,000-record write/list/search
//! distribution corpus and its constants, extracted from
//! `repository_benchmark` so that target keeps its pre-B1 size. The
//! measurement harness and the artifact contract stay in
//! `repository_benchmark.rs`; this module only builds the deterministic
//! corpus snapshot the `saved_pages_write_list_search` operation writes.

use bbr_persistence::{CollectionRecord, SavedPageRecord, SearchSettings, Snapshot};

/// The Saved Pages distribution corpus size: at least 1,000 records (B1
/// packet evidence).
pub(crate) const SAVED_PAGES_CORPUS: usize = 1_000;
/// The flat-collection count the corpus distributes membership over.
pub(crate) const SAVED_PAGES_COLLECTIONS: usize = 20;

/// The Saved Pages distribution corpus: `record_count` global records over
/// `collection_count` flat collections (ids 1..=collection_count), plus the
/// deterministic settings row. The browser/global rows are empty so the
/// write distribution isolates the Saved Pages path. Titles embed a
/// zero-padded index so a distinctive search term matches exactly one row.
pub(crate) fn saved_pages_snapshot(record_count: usize, collection_count: usize) -> Snapshot {
    let collections = (1..=collection_count)
        .map(|id| CollectionRecord {
            id: id as u64,
            title: format!("Collection {id:02}"),
        })
        .collect::<Vec<_>>();
    let saved_pages = (0..record_count)
        .map(|index| SavedPageRecord {
            id: 10_000 + index as u64,
            canonical_url: format!("https://saved.test/page/{index:04}"),
            captured_title: format!("Saved page {index:04}"),
            user_label: (index % 5 == 0).then(|| format!("label {index:04}")),
            created_at: index as u64 * 10,
            updated_at: index as u64 * 10,
            collection_id: (index % collection_count != 0)
                .then(|| (index % collection_count) as u64 + 1),
        })
        .collect::<Vec<_>>();
    Snapshot {
        profiles: Vec::new(),
        windows: Vec::new(),
        settings: SearchSettings::default(),
        downloads: Vec::new(),
        command_history: Vec::new(),
        saved_pages,
        collections,
    }
}
