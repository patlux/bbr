//! Unit tests for the v2 space paper (D6/D7): the closed intensity steps,
//! the defaults, the paper mutation, and the persisted-record restore rule.

use crate::space_appearance::{
    AppearanceError, SpaceAppearance, SpaceAppearanceStore, SpaceIdentity, SpacePaper, SpaceTheme,
    TintIntensity,
};
use crate::{Browser, SpaceId};

fn browser_with_space() -> (Browser, SpaceId) {
    let mut browser = Browser::new();
    let work = browser.create_profile("Work");
    let space = browser.create_space(work, "Main").unwrap();
    (browser, space)
}

#[test]
fn intensity_steps_are_closed_ordered_and_round_trip_by_name() {
    let names: Vec<&str> = TintIntensity::ALL.iter().map(|step| step.name()).collect();
    assert_eq!(names, ["subtle", "standard", "rich"]);
    for step in TintIntensity::ALL {
        assert_eq!(TintIntensity::from_name(step.name()), Some(step));
    }
    assert_eq!(TintIntensity::from_name("vivid"), None);
    assert_eq!(
        SpacePaper::default(),
        SpacePaper {
            intensity: TintIntensity::Standard,
            grain: false
        }
    );
}

#[test]
fn set_paper_preserves_theme_and_identity_and_rejects_unknown_spaces() {
    let (browser, space) = browser_with_space();
    let mut store = SpaceAppearanceStore::new();
    store
        .set_theme(&browser, space, SpaceTheme::Pine)
        .expect("theme");
    let paper = SpacePaper {
        intensity: TintIntensity::Rich,
        grain: true,
    };
    store.set_paper(&browser, space, paper).expect("paper");
    let effective = store.effective(&browser, space).expect("space");
    assert_eq!(effective.theme, SpaceTheme::Pine);
    assert_eq!(effective.identity, SpaceIdentity::default());
    assert_eq!(effective.paper, paper);
    let before = store.clone();
    assert_eq!(
        store.set_paper(&browser, SpaceId(999), paper),
        Err(AppearanceError::UnknownSpace(SpaceId(999)))
    );
    assert_eq!(store, before, "rejected whole");
}

#[test]
fn restore_keeps_only_non_default_records() {
    let (browser, space) = browser_with_space();
    let mut store = SpaceAppearanceStore::new();
    // A persisted default creates no record.
    let derived = store.effective(&browser, space).expect("space");
    store.restore(&browser, derived.clone()).expect("restore");
    assert!(store.is_empty());
    // A persisted non-default record round-trips exactly.
    let custom = SpaceAppearance {
        paper: SpacePaper {
            intensity: TintIntensity::Subtle,
            grain: true,
        },
        ..derived.clone()
    };
    store.restore(&browser, custom.clone()).expect("restore");
    assert_eq!(store.effective(&browser, space), Some(custom));
    assert_eq!(store.len(), 1);
    // Restoring the default again removes the record.
    store.restore(&browser, derived).expect("restore");
    assert!(store.is_empty());
    assert_eq!(
        store.restore(&browser, SpaceAppearance::default_for(SpaceId(7))),
        Err(AppearanceError::UnknownSpace(SpaceId(7)))
    );
}
