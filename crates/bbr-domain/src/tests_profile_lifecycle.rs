//! Unit tests for profile rename, colour and deletion (blueprint D06).

use crate::{Browser, DomainError, ProfileHue};

#[test]
fn deleting_a_profile_removes_its_spaces_and_tabs_and_keeps_the_last_one() {
    let mut browser = Browser::new();
    let work = browser.create_profile("Work");
    let home = browser.create_profile("Home");
    let a = browser.create_space(work, "A").unwrap();
    let h = browser.create_space(home, "H").unwrap();
    let b = browser.create_space(work, "B").unwrap();
    let kept = browser.open_tab(h, "https://h.test", "H").unwrap();
    let gone = [
        browser.open_tab(a, "https://a.test", "A").unwrap(),
        browser.open_tab(b, "https://b.test", "B").unwrap(),
    ];
    assert!(browser.can_delete_profile(work));
    let deleted = browser.delete_profile(work).unwrap();
    assert_eq!(deleted.spaces, vec![a, b]);
    assert_eq!(deleted.tabs, gone.to_vec());
    assert_eq!(deleted.fallback, h);
    assert!(browser.profile(work).is_none());
    assert_eq!(
        browser.spaces().map(|space| space.id).collect::<Vec<_>>(),
        vec![h]
    );
    assert!(browser.tab(kept).is_some());
    browser.check_invariants().unwrap();
    // The last profile stays.
    assert!(!browser.can_delete_profile(home));
    assert_eq!(
        browser.delete_profile(home),
        Err(DomainError::LastProfile(home))
    );
    assert_eq!(
        browser.delete_profile(work),
        Err(DomainError::UnknownProfile(work))
    );
}

#[test]
fn a_profile_can_be_renamed_and_recoloured() {
    let mut browser = Browser::new();
    let work = browser.create_profile("Work");
    browser.rename_profile(work, "  Client work ").unwrap();
    browser.rename_profile(work, "   ").unwrap();
    assert_eq!(browser.profile(work).unwrap().title, "Client work");
    browser.set_profile_hue(work, ProfileHue::ALL[4]).unwrap();
    assert_eq!(browser.profile(work).unwrap().hue, ProfileHue::ALL[4]);
}
