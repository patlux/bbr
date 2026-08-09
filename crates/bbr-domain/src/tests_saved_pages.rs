//! Unit tests for the portable Saved Page contract: identity/URL/title/
//! label/timestamp/collection validation, the canonical-URL equality seam,
//! privacy-safe `Debug`, deterministic ordering/search, and the
//! credential-free record invariant.

use crate::saved_pages::{
    canonical_url, deterministic_order, validate_collection, BookmarkId, CanonicalUrl, Collection,
    CollectionId, SavedPage, SavedPageError, MAX_CAPTURED_TITLE_LEN, MAX_COLLECTION_TITLE_LEN,
    MAX_USER_LABEL_LEN,
};

fn page(
    id: u64,
    url: &str,
    title: &str,
    label: Option<&str>,
    created: u64,
    updated: u64,
    collection: Option<u64>,
) -> SavedPage {
    SavedPage {
        id: BookmarkId(id),
        url: CanonicalUrl::try_from(url).unwrap(),
        captured_title: title.into(),
        user_label: label.map(str::to_owned),
        created_at: created,
        updated_at: updated,
        collection: collection.map(CollectionId),
    }
}

// ---- Canonical-URL equality seam ------------------------------------------

#[test]
fn canonical_url_folds_only_scheme_and_host() {
    let cases = [
        ("https://EXAMPLE.com/path", "https://example.com/path"),
        ("HTTP://Example.COM/", "http://example.com/"),
        ("example.com", "https://example.com"),
        (
            "EXAMPLE.com:8080/a?q=1#f",
            "https://example.com:8080/a?q=1#f",
        ),
        ("https://example.com", "https://example.com"),
        ("about:blank", "about:blank"),
        ("https://sub.example.com", "https://sub.example.com"),
        (
            "https://example.com/Path?Query#Fragment",
            "https://example.com/Path?Query#Fragment",
        ),
        ("https://example.com/%41%2Fb", "https://example.com/%41%2Fb"),
        ("https://example.com./", "https://example.com./"),
        ("  https://EXAMPLE.com/a  ", "https://example.com/a"),
    ];
    for (input, expected) in cases {
        assert_eq!(canonical_url(input).as_deref(), Ok(expected), "{input:?}");
    }
}

#[test]
fn canonical_url_preserves_trailing_slash_port_www_and_encoding_byte_for_byte() {
    // The settled decision: path, query, fragment, explicit port, trailing
    // slash, `www.`, and percent-encoding are preserved byte-for-byte — no
    // default-port elision and no bookmark-only rewrite.
    assert_ne!(
        canonical_url("https://example.com").unwrap(),
        canonical_url("https://example.com/").unwrap(),
        "trailing slash is a distinct canonical key"
    );
    assert_ne!(
        canonical_url("https://example.com:443/a").unwrap(),
        canonical_url("https://example.com/a").unwrap(),
        "explicit port is never elided"
    );
    assert_ne!(
        canonical_url("https://www.example.com").unwrap(),
        canonical_url("https://example.com").unwrap(),
        "www. is never rewritten"
    );
    assert_ne!(
        canonical_url("https://example.com/a%2fb").unwrap(),
        canonical_url("https://example.com/a/b").unwrap(),
        "percent-encoding is never canonicalized"
    );
    assert_eq!(
        canonical_url("https://example.com/Path?Query#Fragment").unwrap(),
        "https://example.com/Path?Query#Fragment",
        "path/query/fragment case is preserved"
    );
    // Host folding is the ONLY rewrite: two strings equal after folding are
    // the same canonical key.
    assert_eq!(
        canonical_url("https://EXAMPLE.com/a%2fb").unwrap(),
        canonical_url("https://example.com/a%2fb").unwrap()
    );
}

#[test]
fn authority_trim_is_part_of_scheme_defaulting() {
    // The existing navigation authority trims leading/trailing whitespace
    // as part of scheme defaulting, so a padded URL canonicalizes.
    assert_eq!(
        canonical_url("  HTTPS://EXAMPLE.com/a  ").unwrap(),
        "https://example.com/a"
    );
}

#[test]
fn canonical_url_rejects_unsupported_dangerous_and_malformed_urls_whole() {
    for (input, expected) in [
        ("", SavedPageError::InvalidUrl("empty URL".into())),
        ("   ", SavedPageError::InvalidUrl("empty URL".into())),
        (
            "javascript:alert(1)",
            SavedPageError::UnsupportedScheme("javascript".into()),
        ),
        (
            "data:text/html,<script>",
            SavedPageError::UnsupportedScheme("data".into()),
        ),
        (
            "file:///etc/passwd",
            SavedPageError::UnsupportedScheme("file".into()),
        ),
        (
            "mailto:user@example.com",
            SavedPageError::UnsupportedScheme("mailto".into()),
        ),
        (
            "custom+scheme:x",
            SavedPageError::UnsupportedScheme("custom+scheme".into()),
        ),
        (
            "//example.com",
            SavedPageError::InvalidUrl("no scheme".into()),
        ),
        (
            "https://exa mple.com",
            SavedPageError::InvalidUrl("NUL, whitespace, or control bytes".into()),
        ),
        (
            "https://example.com/a\tb",
            SavedPageError::InvalidUrl("NUL, whitespace, or control bytes".into()),
        ),
        (
            "https://example.com/\u{0}",
            SavedPageError::InvalidUrl("NUL, whitespace, or control bytes".into()),
        ),
    ] {
        assert_eq!(canonical_url(input), Err(expected), "{input:?}");
    }
}

#[test]
fn canonical_url_rejects_credentials_so_debug_is_privacy_safe() {
    // Userinfo in the authority is rejected whole: a Saved Page record is
    // credential-free by construction, so Debug/Display never leak secrets.
    for input in [
        "https://user:pass@example.com/",
        "https://user@example.com/",
        "https://@example.com/",
        "https://example.com@evil.com/",
    ] {
        assert_eq!(
            canonical_url(input),
            Err(SavedPageError::CredentialsNotAllowed),
            "{input:?}"
        );
    }
    // '@' in the path is not userinfo and stays valid.
    assert_eq!(
        canonical_url("https://example.com/a@b").unwrap(),
        "https://example.com/a@b"
    );
    // The Debug output of a canonical URL contains only the canonical
    // navigable string — no password material can exist inside it.
    let url = CanonicalUrl::try_from("https://EXAMPLE.com/a").unwrap();
    let debug = format!("{url:?}");
    assert_eq!(debug, "CanonicalUrl(\"https://example.com/a\")");
    assert!(!debug.contains("pass"));
}

#[test]
fn canonical_url_host_folding_skips_percent_escapes_and_keeps_ports() {
    assert_eq!(
        canonical_url("https://Example.COM:%41%2Fb").unwrap(),
        "https://example.com:%41%2Fb"
    );
    assert_eq!(
        canonical_url("HTTPS://EXAMPLE.com:8080").unwrap(),
        "https://example.com:8080"
    );
}

// ---- Identity and record validation ----------------------------------------

#[test]
fn bookmark_and_collection_ids_validate_bounds() {
    assert_eq!(
        canonical_url("https://example.com").unwrap(),
        "https://example.com"
    );
    let invalid_ids = [0, i64::MAX as u64 + 1];
    for id in invalid_ids {
        let record = page(id, "https://example.com", "t", None, 1, 1, None);
        assert!(matches!(
            record.validate(),
            Err(SavedPageError::InvalidId(_))
        ));
        let collection = Collection {
            id: CollectionId(id),
            title: "Work".into(),
        };
        assert!(matches!(
            validate_collection(&collection),
            Err(SavedPageError::InvalidId(_))
        ));
    }
    // Boundary ids are valid.
    assert!(page(1, "https://example.com", "t", None, 1, 1, None)
        .validate()
        .is_ok());
    let collection = Collection {
        id: CollectionId(i64::MAX as u64),
        title: "Work".into(),
    };
    assert!(validate_collection(&collection).is_ok());
}

#[test]
fn title_label_and_collection_title_validate_bounds() {
    let over_long = "x".repeat(MAX_CAPTURED_TITLE_LEN + 1);
    let record = page(1, "https://example.com", &over_long, None, 1, 1, None);
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTitle(_))
    ));
    let over_long = "x".repeat(MAX_USER_LABEL_LEN + 1);
    let record = page(1, "https://example.com", "t", Some(&over_long), 1, 1, None);
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTitle(_))
    ));
    // A label, when present, must be non-empty.
    let record = page(1, "https://example.com", "t", Some(""), 1, 1, None);
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTitle(_))
    ));
    // Captured titles may be empty; control bytes never.
    assert!(page(1, "https://example.com", "", None, 1, 1, None)
        .validate()
        .is_ok());
    let record = page(1, "https://example.com", "ti\ntle", None, 1, 1, None);
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTitle(_))
    ));
    // Collection titles are non-empty and bounded.
    let collection = Collection {
        id: CollectionId(1),
        title: String::new(),
    };
    assert!(matches!(
        validate_collection(&collection),
        Err(SavedPageError::InvalidTitle(_))
    ));
    let collection = Collection {
        id: CollectionId(1),
        title: "x".repeat(MAX_COLLECTION_TITLE_LEN + 1),
    };
    assert!(matches!(
        validate_collection(&collection),
        Err(SavedPageError::InvalidTitle(_))
    ));
}

#[test]
fn timestamps_validate_range_and_monotonicity() {
    let record = page(
        1,
        "https://example.com",
        "t",
        None,
        i64::MAX as u64 + 1,
        1,
        None,
    );
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTimestamp(_))
    ));
    // updated_at must never precede created_at.
    let record = page(1, "https://example.com", "t", None, 10, 9, None);
    assert!(matches!(
        record.validate(),
        Err(SavedPageError::InvalidTimestamp(_))
    ));
    assert!(page(1, "https://example.com", "t", None, 10, 10, None)
        .validate()
        .is_ok());
}

#[test]
fn record_validation_covers_every_field() {
    // A fully populated record validates.
    let record = page(
        1,
        "https://example.com/a",
        "Title",
        Some("label"),
        100,
        200,
        Some(7),
    );
    assert!(record.validate().is_ok());
    // An unsupported scheme rejects at the CanonicalUrl boundary.
    assert_eq!(
        CanonicalUrl::try_from("javascript:alert(1)"),
        Err(SavedPageError::UnsupportedScheme("javascript".into()))
    );
}

// ---- Deterministic ordering and search -------------------------------------

#[test]
fn deterministic_order_sorts_by_updated_then_id_with_ties() {
    // Equal titles/URLs/timestamps/membership still order deterministically
    // through the id tiebreak.
    let mut pages = vec![
        page(3, "https://example.com/a", "Same", None, 10, 10, None),
        page(1, "https://example.com/a", "Same", None, 10, 10, None),
        page(2, "https://example.com/b", "Same", None, 10, 20, None),
        page(4, "https://example.com/c", "Other", None, 10, 5, None),
    ];
    deterministic_order(&mut pages);
    let ids = pages.iter().map(|p| p.id.0).collect::<Vec<_>>();
    assert_eq!(ids, vec![4, 1, 3, 2], "ascending (updated_at, id)");
    // Idempotent: sorting again yields the same order.
    deterministic_order(&mut pages);
    let again = pages.iter().map(|p| p.id.0).collect::<Vec<_>>();
    assert_eq!(again, ids);
}

#[test]
fn matches_is_case_insensitive_over_title_label_and_url() {
    let record = page(
        1,
        "https://example.com/Path",
        "My Saved Page",
        Some("work"),
        1,
        1,
        Some(2),
    );
    assert!(record.matches("saved"));
    assert!(record.matches("SAVED"));
    assert!(record.matches("work"));
    assert!(record.matches("example.com"));
    assert!(record.matches(""));
    assert!(!record.matches("zzz"));
    assert!(!record.matches("label"));
}

#[test]
fn global_ownership_has_no_profile_or_tab_dimension() {
    // The record type carries no profile/space/tab/session field by
    // construction: the field set is exactly the B1 contract.
    let record = page(1, "https://example.com", "t", None, 1, 1, None);
    let fields = format!("{record:?}");
    assert!(!fields.contains("profile"));
    assert!(!fields.contains("tab"));
    assert!(!fields.contains("session"));
    assert!(!fields.contains("webview"));
}
