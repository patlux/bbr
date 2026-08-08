//! Deterministic unit tests for the space-appearance contract: palette
//! stability, emoji/glyph validation, identity variants, store defaults,
//! overrides, reset semantics, ordering, and invariant tamper detection.
//! These tests run without macOS frameworks or a display server.

use std::collections::HashSet;

use crate::space_appearance::{
    AppearanceError, BbrGlyph, IdentityKind, SpaceAppearance, SpaceAppearanceStore, SpaceEmoji,
    SpaceIdentity, SpaceTheme, ALLOWED_EMOJI,
};
use crate::{Browser, ProfileHue, SpaceId};

/// A browser with two profiles, each owning one space.
fn browser_with_two_spaces() -> (Browser, SpaceId, SpaceId) {
    let mut browser = Browser::new();
    let work = browser.create_profile("Work");
    let personal = browser.create_profile("Personal");
    let work_space = browser.create_space(work, "Main").unwrap();
    let personal_space = browser.create_space(personal, "Main").unwrap();
    (browser, work_space, personal_space)
}

// ---- Curated palette -------------------------------------------------------

#[test]
fn theme_catalog_is_curated_and_stable() {
    let names: Vec<&str> = SpaceTheme::ALL.iter().map(|t| t.name()).collect();
    // Identity hues first (ProfileHue::ALL order), then the space-only
    // tokens. Snapshot the exact sequence: any reorder is a contract change.
    assert_eq!(
        names,
        vec![
            "cobalt",
            "violet",
            "fuchsia",
            "amber",
            "terracotta",
            "slate",
            "pine",
            "plum",
            "ink",
        ]
    );
    for token in SpaceTheme::ALL {
        assert!(!token.label().is_empty());
        assert!(!token.name().is_empty());
    }
}

#[test]
fn theme_names_are_unique_and_lowercase() {
    let mut seen = HashSet::new();
    for token in SpaceTheme::ALL {
        let name = token.name();
        assert!(seen.insert(name), "duplicate theme name {name:?}");
        assert_eq!(
            name,
            name.to_ascii_lowercase(),
            "name {name:?} not lowercase"
        );
    }
}

#[test]
fn theme_default_is_first_palette_token() {
    assert_eq!(SpaceTheme::default(), SpaceTheme::ALL[0]);
}

#[test]
fn theme_maps_every_profile_hue_bijectively() {
    let mut mapped = HashSet::new();
    for hue in ProfileHue::ALL {
        let theme = SpaceTheme::from_profile_hue(hue);
        // The existing identity hues "go into" the palette unchanged.
        assert_eq!(theme.name(), hue.name());
        assert!(
            mapped.insert(theme),
            "distinct hues must map to distinct themes"
        );
    }
    // Inverse: the six identity tokens round-trip, the space-only tokens
    // have no profile counterpart.
    for token in SpaceTheme::ALL {
        match token.profile_hue() {
            Some(hue) => assert_eq!(SpaceTheme::from_profile_hue(hue), token),
            None => assert!(!matches!(
                token,
                SpaceTheme::Cobalt
                    | SpaceTheme::Violet
                    | SpaceTheme::Fuchsia
                    | SpaceTheme::Amber
                    | SpaceTheme::Terracotta
                    | SpaceTheme::Slate
            )),
        }
    }
}

// ---- Emoji identity --------------------------------------------------------

#[test]
fn emoji_allowlist_is_nonempty_and_every_entry_is_one_scalar() {
    assert!(
        ALLOWED_EMOJI.len() >= 12,
        "curated allowlist must stay a non-trivial set"
    );
    for entry in ALLOWED_EMOJI {
        assert_eq!(
            entry.chars().count(),
            1,
            "curated emoji {entry:?} is not exactly one Unicode scalar"
        );
    }
}

#[test]
fn emoji_allowlist_has_no_duplicates() {
    let mut seen = HashSet::new();
    for entry in ALLOWED_EMOJI {
        assert!(seen.insert(*entry), "duplicate emoji entry {entry:?}");
    }
}

#[test]
fn emoji_valid_construction_round_trips_every_curated_entry() {
    for entry in ALLOWED_EMOJI {
        let emoji = SpaceEmoji::new(entry).expect("curated entry must validate");
        assert_eq!(emoji.as_str(), *entry);
        assert_eq!(emoji.to_string(), *entry);
    }
}

#[test]
fn emoji_rejects_uncurated_input_whole() {
    let invalid = [
        "",     // empty
        "🔥x",  // emoji plus a second scalar
        "🔥🔥", // two emoji scalars
        "ab",   // plain multi-scalar text
        "home", // plain word
        " ",    // whitespace
        "😀",   // plausible but uncurated emoji
        "🇩🇪",   // flag = two regional-indicator scalars
    ];
    for input in invalid {
        assert_eq!(
            SpaceEmoji::new(input),
            Err(AppearanceError::InvalidEmoji(input.to_owned())),
            "input {input:?} must be rejected"
        );
    }
}

#[test]
fn emoji_default_is_first_curated_entry() {
    assert_eq!(SpaceEmoji::default().as_str(), ALLOWED_EMOJI[0]);
}

// ---- BBR glyph catalog -----------------------------------------------------

#[test]
fn glyph_catalog_is_curated_and_stable() {
    let names: Vec<&str> = BbrGlyph::ALL.iter().map(|g| g.name()).collect();
    assert_eq!(
        names,
        vec![
            "code",
            "terminal",
            "folder",
            "person",
            "briefcase",
            "heart",
            "star",
            "leaf",
            "compass",
            "moon",
            "book",
            "camera",
            "palette",
            "globe",
        ]
    );
    for glyph in BbrGlyph::ALL {
        assert!(!glyph.label().is_empty());
    }
}

#[test]
fn glyph_names_are_unique_lowercase_and_disjoint_from_profile_hues() {
    let mut seen = HashSet::new();
    let hue_names: HashSet<&str> = ProfileHue::ALL.iter().map(|h| h.name()).collect();
    for glyph in BbrGlyph::ALL {
        let name = glyph.name();
        assert!(seen.insert(name), "duplicate glyph name {name:?}");
        assert_eq!(name, name.to_ascii_lowercase());
        assert!(
            !hue_names.contains(name),
            "glyph name {name:?} must not collide with a profile hue name"
        );
    }
}

#[test]
fn glyph_default_is_first_catalog_entry() {
    assert_eq!(BbrGlyph::default(), BbrGlyph::ALL[0]);
}

// ---- Identity model --------------------------------------------------------

#[test]
fn identity_is_exactly_one_variant_with_stable_names() {
    let emoji = SpaceIdentity::Emoji(SpaceEmoji::new("🔥").unwrap());
    let glyph = SpaceIdentity::Glyph(BbrGlyph::Compass);
    assert_eq!(emoji.kind(), IdentityKind::Emoji);
    assert_eq!(glyph.kind(), IdentityKind::Glyph);
    assert_eq!(emoji.name(), "🔥");
    assert_eq!(glyph.name(), "compass");
    assert_eq!(emoji.to_string(), "🔥");
    assert_eq!(glyph.to_string(), "compass");
    assert_ne!(emoji, glyph);
}

#[test]
fn identity_default_is_first_glyph() {
    assert_eq!(
        SpaceIdentity::default(),
        SpaceIdentity::Glyph(BbrGlyph::default())
    );
    assert_eq!(SpaceIdentity::default().kind(), IdentityKind::Glyph);
}

// ---- Store: defaults and effective resolution ------------------------------

#[test]
fn new_store_has_no_explicit_records() {
    let store = SpaceAppearanceStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert_eq!(store.appearance(SpaceId(1)), None);
    assert_eq!(store.iter().count(), 0);
}

#[test]
fn effective_derives_defaults_from_owning_profile_hue() {
    let (browser, work_space, personal_space) = browser_with_two_spaces();
    let store = SpaceAppearanceStore::new();
    let work_effective = store.effective(&browser, work_space).unwrap();
    let personal_effective = store.effective(&browser, personal_space).unwrap();
    // Round-robin: profile 1 -> Cobalt, profile 2 -> Violet.
    assert_eq!(work_effective.theme, SpaceTheme::Cobalt);
    assert_eq!(personal_effective.theme, SpaceTheme::Violet);
    assert_eq!(work_effective.identity, SpaceIdentity::default());
    assert_eq!(work_effective.space, work_space);
    // The identity hues "go into" the theming mechanism unchanged.
    let hue = browser
        .profile(browser.space(work_space).unwrap().profile)
        .unwrap()
        .hue;
    assert_eq!(work_effective.theme.name(), hue.name());
}

#[test]
fn effective_unknown_space_is_none_and_never_creates_records() {
    let (browser, _, _) = browser_with_two_spaces();
    let store = SpaceAppearanceStore::new();
    assert_eq!(store.effective(&browser, SpaceId(999)), None);
    assert!(store.is_empty(), "derived lookups must not create records");
}

// ---- Store: mutations ------------------------------------------------------

#[test]
fn set_theme_on_unknown_space_rejects_whole() {
    let (browser, _, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    assert_eq!(
        store.set_theme(&browser, SpaceId(999), SpaceTheme::Pine),
        Err(AppearanceError::UnknownSpace(SpaceId(999)))
    );
    assert!(
        store.is_empty(),
        "rejected mutation must leave the store empty"
    );
}

#[test]
fn set_identity_on_unknown_space_rejects_whole() {
    let (browser, _, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    assert_eq!(
        store.set_identity(
            &browser,
            SpaceId(999),
            SpaceIdentity::Emoji(SpaceEmoji::new("🔥").unwrap())
        ),
        Err(AppearanceError::UnknownSpace(SpaceId(999)))
    );
    assert!(store.is_empty());
}

#[test]
fn set_theme_preserves_derived_identity_and_vice_versa() {
    let (browser, work_space, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_theme(&browser, work_space, SpaceTheme::Pine)
        .unwrap();
    let record = store.appearance(work_space).unwrap();
    assert_eq!(record.theme, SpaceTheme::Pine);
    // The derived default identity is preserved when only the theme is set.
    assert_eq!(record.identity, SpaceIdentity::default());
    assert_eq!(
        store.effective(&browser, work_space).unwrap().theme,
        SpaceTheme::Pine
    );

    let heart = SpaceIdentity::Emoji(SpaceEmoji::new("💼").unwrap());
    store
        .set_identity(&browser, work_space, heart.clone())
        .unwrap();
    let record = store.appearance(work_space).unwrap();
    assert_eq!(
        record.theme,
        SpaceTheme::Pine,
        "theme survives identity set"
    );
    assert_eq!(record.identity, heart);
}

#[test]
fn set_appearance_is_atomic() {
    let (browser, work_space, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    let identity = SpaceIdentity::Glyph(BbrGlyph::Globe);
    store
        .set_appearance(&browser, work_space, SpaceTheme::Plum, identity.clone())
        .unwrap();
    let record = store.appearance(work_space).unwrap();
    assert_eq!(record.theme, SpaceTheme::Plum);
    assert_eq!(record.identity, identity);
    assert_eq!(store.len(), 1);

    // Unknown space: rejected whole, no record, store unchanged.
    assert_eq!(
        store.set_appearance(&browser, SpaceId(999), SpaceTheme::Ink, identity.clone()),
        Err(AppearanceError::UnknownSpace(SpaceId(999)))
    );
    assert_eq!(store.len(), 1);
    assert_eq!(
        store.appearance(work_space).unwrap().theme,
        SpaceTheme::Plum
    );
}

#[test]
fn records_keep_deterministic_assignment_order() {
    let (browser, work_space, personal_space) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_theme(&browser, personal_space, SpaceTheme::Ink)
        .unwrap();
    store
        .set_theme(&browser, work_space, SpaceTheme::Amber)
        .unwrap();
    let order: Vec<SpaceId> = store.iter().map(|record| record.space).collect();
    assert_eq!(order, vec![personal_space, work_space]);
}

#[test]
fn reset_removes_the_override_and_falls_back_to_derived_defaults() {
    let (browser, work_space, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_appearance(
            &browser,
            work_space,
            SpaceTheme::Ink,
            SpaceIdentity::Emoji(SpaceEmoji::new("🚀").unwrap()),
        )
        .unwrap();
    store.reset(&browser, work_space).unwrap();
    assert_eq!(store.appearance(work_space), None);
    assert!(
        store.is_empty(),
        "reset must remove the record, not rewrite it"
    );
    let effective = store.effective(&browser, work_space).unwrap();
    assert_eq!(effective.theme, SpaceTheme::Cobalt);
    assert_eq!(effective.identity, SpaceIdentity::default());
}

#[test]
fn reset_without_record_is_noop_and_unknown_space_rejects() {
    let (browser, work_space, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store.reset(&browser, work_space).unwrap();
    assert!(store.is_empty());
    assert_eq!(
        store.reset(&browser, SpaceId(999)),
        Err(AppearanceError::UnknownSpace(SpaceId(999)))
    );
}

#[test]
fn identities_are_isolated_between_spaces() {
    let (browser, work_space, personal_space) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    let fire = SpaceIdentity::Emoji(SpaceEmoji::new("🔥").unwrap());
    store.set_identity(&browser, work_space, fire).unwrap();
    // The untouched space keeps its derived default identity.
    let effective = store.effective(&browser, personal_space).unwrap();
    assert_eq!(effective.identity, SpaceIdentity::default());
    assert_eq!(
        effective.theme,
        SpaceTheme::from_profile_hue(ProfileHue::Violet)
    );
}

#[test]
fn identical_operation_sequences_produce_identical_stores() {
    let build = || {
        let (browser, work_space, personal_space) = browser_with_two_spaces();
        let mut store = SpaceAppearanceStore::new();
        store
            .set_appearance(
                &browser,
                personal_space,
                SpaceTheme::Pine,
                SpaceIdentity::Glyph(BbrGlyph::Leaf),
            )
            .unwrap();
        store
            .set_theme(&browser, work_space, SpaceTheme::Amber)
            .unwrap();
        store.reset(&browser, personal_space).unwrap();
        store
            .set_identity(&browser, personal_space, SpaceIdentity::default())
            .unwrap();
        store
    };
    let store_a = build();
    let store_b = build();
    assert_eq!(store_a, store_b);
    for record in store_a.iter() {
        assert_eq!(
            store_b.appearance(record.space),
            Some(record),
            "deterministic store divergence for space {:?}",
            record.space
        );
    }
}

// ---- Store: invariants -----------------------------------------------------

#[test]
fn check_invariants_passes_on_clean_store() {
    let (browser, work_space, personal_space) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_theme(&browser, work_space, SpaceTheme::Pine)
        .unwrap();
    store
        .set_identity(
            &browser,
            personal_space,
            SpaceIdentity::Emoji(SpaceEmoji::new("🔥").unwrap()),
        )
        .unwrap();
    store.check_invariants(&browser).unwrap();
}

#[test]
fn check_invariants_detects_unknown_space_tamper() {
    let (browser, _, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    // Forge a record for a space that never existed (mutations forbid it).
    store
        .appearances
        .push(SpaceAppearance::default_for(SpaceId(999)));
    let violation = store.check_invariants(&browser).unwrap_err();
    assert!(violation.0.contains("unknown space"));
    assert!(violation.0.contains("999"));
}

#[test]
fn check_invariants_detects_duplicate_space_tamper() {
    let (browser, work_space, _) = browser_with_two_spaces();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_theme(&browser, work_space, SpaceTheme::Ink)
        .unwrap();
    // Forge a second record for the same space (mutations forbid it).
    store
        .appearances
        .push(SpaceAppearance::default_for(work_space));
    let violation = store.check_invariants(&browser).unwrap_err();
    assert!(violation.0.contains("duplicate appearance"));
}
