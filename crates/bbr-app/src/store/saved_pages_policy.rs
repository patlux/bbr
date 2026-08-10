//! Pure Saved Pages provider policy (B2/B3 packets): the deterministic
//! in-memory mirror of the B1 repository duplicate semantics and the
//! presentation mapping the provider and the command corpus consume.
//!
//! Everything here is pure and unit-testable without GPUI or SQLite: the
//! save/remove mutation policies, the deterministic presentation order, the
//! provider-neutral corpus entry mapping, and the monotonic timestamp.

use bbr_persistence::{CollectionRecord, SavedPageRecord, SavedPageSaveOutcome};

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

/// The deterministic monotonic timestamp for a save (seconds since the
/// Unix epoch; the B1 contract only requires monotonicity and the sort
/// key, never wall-clock precision).
pub(crate) fn now_secs() -> u64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: u64, url: &str, title: &str) -> SavedPageRecord {
        SavedPageRecord {
            id,
            canonical_url: url.into(),
            captured_title: title.into(),
            user_label: None,
            created_at: 1,
            updated_at: 1,
            collection_id: None,
        }
    }

    #[test]
    fn save_is_canonical_idempotent_and_never_touches_a_label() {
        let mut pages = Vec::new();
        let mut next = 1;
        let first = save_saved_page_into(&mut pages, &mut next, "https://a.test/", "Alpha", 10);
        assert_eq!(first.id, 1);
        assert!(!first.existed);
        // A duplicate returns the same identity and creates no second row.
        let second = save_saved_page_into(&mut pages, &mut next, "https://a.test/", "Beta", 20);
        assert_eq!(second.id, first.id);
        assert!(second.existed);
        assert_eq!(pages.len(), 1);
        // No user label exists, so the captured title updates; `updated_at`
        // never regresses below `created_at`.
        assert_eq!(pages[0].captured_title, "Beta");
        assert!(pages[0].updated_at >= pages[0].created_at);
        // A user label freezes the captured title on later saves.
        pages[0].user_label = Some("Work".into());
        save_saved_page_into(&mut pages, &mut next, "https://a.test/", "Gamma", 30);
        assert_eq!(pages[0].captured_title, "Beta");
    }

    #[test]
    fn remove_is_idempotent() {
        let mut pages = vec![
            record(1, "https://a.test/", "A"),
            record(2, "https://b.test/", "B"),
        ];
        assert!(remove_saved_page_into(&mut pages, 1));
        assert_eq!(pages.len(), 1);
        assert!(!remove_saved_page_into(&mut pages, 1), "unknown is a no-op");
        assert!(!remove_saved_page_into(&mut pages, 99));
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn provider_order_presents_most_recently_updated_first() {
        let mut pages = vec![
            record(1, "https://a.test/", "A"),
            record(2, "https://b.test/", "B"),
            record(3, "https://c.test/", "C"),
        ];
        pages[0].updated_at = 10;
        pages[1].updated_at = 30;
        pages[2].updated_at = 20;
        provider_order(&mut pages);
        assert_eq!(
            pages.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![2, 3, 1],
            "most recently updated first, then id descending"
        );
    }

    #[test]
    fn entry_mapping_uses_the_display_title_ladder_and_collection_label() {
        let collections = vec![CollectionRecord {
            id: 7,
            title: "Work".into(),
        }];
        let page = SavedPageRecord {
            id: 3,
            canonical_url: "https://example.test/".into(),
            captured_title: "Captured".into(),
            user_label: Some("Label".into()),
            created_at: 1,
            updated_at: 1,
            collection_id: Some(7),
        };
        let entry = saved_page_to_entry(&page, &collections);
        assert_eq!(entry.title, "Label", "the label heads the ladder");
        assert_eq!(entry.collection.as_deref(), Some("Work"));
        // Without a label the captured title is used; the URL is the final
        // fallback for an empty captured title.
        let mut page = page;
        page.user_label = None;
        let entry = saved_page_to_entry(&page, &collections);
        assert_eq!(entry.title, "Captured");
        page.captured_title.clear();
        let entry = saved_page_to_entry(&page, &collections);
        assert_eq!(entry.title, "https://example.test/");
    }
}
