//! Search and deterministic-list contracts of the global Saved Pages store
//! (schema v3, B1 packet `saved-pages-portable-persistence`): persisted
//! order is deterministic, search is case-insensitive literal-substring over
//! title/label/canonical URL with no profile dimension, and the search
//! predicate matches the documented `bbr-domain` Unicode-lowercase
//! authority for non-ASCII (umlaut) queries (review F1 regression).

use bbr_persistence::SavedPageRecord;

use super::support::{sample_snapshot, saved_page_record, TempDb};

#[test]
fn list_and_search_are_deterministic_and_global_without_a_profile_dimension() {
    let db = TempDb::new("list-search");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    // Equal timestamps and equal titles: order is still deterministic
    // (position order, dense and caller-stable).
    let records = vec![
        saved_page_record(1, "https://a.test/", "Same title", None, 10, 10, None),
        saved_page_record(2, "https://b.test/", "Same title", None, 10, 10, None),
        saved_page_record(3, "https://c.test/", "Other", Some("Work"), 20, 20, None),
    ];
    for record in &records {
        repository.store_saved_page(record).unwrap();
    }
    // The store also carries a profile/space/tab graph; the global Saved
    // Pages read is independent of it (no profile filter, no tab link).
    let mut snapshot = sample_snapshot(3);
    repository.replace_snapshot(&snapshot).unwrap();
    for record in &records {
        repository.store_saved_page(record).unwrap();
    }
    snapshot.saved_pages = records.clone();
    let restored = repository.restore_read_only().unwrap();
    assert_eq!(
        restored.snapshot.saved_pages, records,
        "no profile dimension"
    );
    assert_eq!(
        restored.snapshot.profiles.len(),
        1,
        "browser graph untouched"
    );

    // Deterministic reads: repeated loads are identical.
    let first = repository.load_saved_pages().unwrap();
    let second = repository.load_saved_pages().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 3);

    // Deterministic case-insensitive search over title, label, and URL.
    assert_eq!(
        repository.search_saved_pages("same title").unwrap(),
        records[0..2]
    );
    assert_eq!(repository.search_saved_pages("SAME").unwrap().len(), 2);
    assert_eq!(
        repository.search_saved_pages("work").unwrap(),
        vec![records[2].clone()]
    );
    assert_eq!(
        repository.search_saved_pages("b.test").unwrap(),
        vec![records[1].clone()]
    );
    assert_eq!(
        repository.search_saved_pages("zzz").unwrap(),
        Vec::<SavedPageRecord>::new()
    );
    // Empty query = full list; LIKE wildcards in the query are literal.
    assert_eq!(repository.search_saved_pages("").unwrap(), records);
    assert_eq!(
        repository.search_saved_pages("100%").unwrap(),
        Vec::<SavedPageRecord>::new()
    );
}

#[test]
fn search_matches_unicode_queries_like_the_domain_literal_substring_predicate() {
    // Regression (review F1): the repository search must behave exactly like
    // the documented domain literal-substring predicate (`bbr-domain`
    // `SavedPage::matches`): Unicode-lowercase case-insensitive containment
    // over title, user label, and canonical URL. SQLite `LIKE` folds ASCII
    // case only and a byte-wise escape loop corrupts multi-byte UTF-8
    // (mojibake), so non-ASCII queries such as `müller` and `über` must be
    // proven against this seam, not against SQL behavior.
    let db = TempDb::new("unicode-search");
    let repository = db.repository();
    repository.initialize_and_restore().unwrap();
    let mueller = saved_page_record(
        1000,
        "https://muller.test/a",
        "Müller",
        None,
        100,
        100,
        None,
    );
    let uebersicht = saved_page_record(
        1001,
        "https://example.test/b",
        "Overview",
        Some("Übersicht"),
        200,
        200,
        None,
    );
    let plain = saved_page_record(
        1002,
        "https://uber.test/c",
        "uber service",
        None,
        300,
        300,
        None,
    );
    for record in [&mueller, &uebersicht, &plain] {
        repository.store_saved_page(record).unwrap();
    }
    // `müller` matches the umlaut title (no mojibake, Unicode folding).
    assert_eq!(
        repository.search_saved_pages("müller").unwrap(),
        vec![mueller.clone()]
    );
    // Uppercase umlaut query folds too, exactly like `to_lowercase`.
    assert_eq!(
        repository.search_saved_pages("MÜLLER").unwrap(),
        vec![mueller.clone()]
    );
    // `über` matches `Übersicht` (Unicode case fold) and never the plain
    // `uber` record (the letters u/ü are distinct).
    assert_eq!(
        repository.search_saved_pages("über").unwrap(),
        vec![uebersicht.clone()]
    );
    assert_eq!(
        repository.search_saved_pages("ÜBER").unwrap(),
        vec![uebersicht.clone()]
    );
    // Wildcard bytes in a query stay literal under the same predicate.
    assert_eq!(
        repository.search_saved_pages("100%").unwrap(),
        Vec::<SavedPageRecord>::new()
    );
    assert_eq!(
        repository.search_saved_pages("_er").unwrap(),
        Vec::<SavedPageRecord>::new()
    );
    assert_eq!(
        repository.search_saved_pages("über%").unwrap(),
        Vec::<SavedPageRecord>::new()
    );
}
