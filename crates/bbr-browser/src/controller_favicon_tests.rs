//! Controller favicon-acquisition contract tests (task packet
//! `real-favicon-pipeline.md`).
//!
//! Proves the portable half of the pipeline through the deterministic fake
//! engine: a typed per-page favicon event lands in the owning profile's
//! bounded host-keyed store, the tab presentation model reports the §7.2
//! monogram until the record exists and `Image` afterwards, two profiles
//! never share a cache entry, a repeated host acquires once and every
//! presentation derives from the same record, and unknown/stale views are
//! dropped without corruption. The macOS adapter and the real-app E2E cover
//! the native half (candidate collection, profile-bound download, fixtures).

use bbr_web::{FaviconContent, ProfileId, WebEvent, WebViewId};

use crate::harness::{open, wired};
use crate::snapshot::TabFavicon;

fn favicon(view: WebViewId, bytes: impl AsRef<[u8]>) -> WebEvent {
    WebEvent::FaviconChanged(view, FaviconContent::Image(bytes.as_ref().to_vec()))
}

#[test]
fn favicon_event_lands_in_the_owning_profiles_store_and_presents_image() {
    let (c, h) = wired();
    open(&mut c.borrow_mut(), "https://example.com/start");
    let snap = c.borrow().snapshot();
    let view = snap.active_view.expect("live session");
    let tab_index = snap.active_index.expect("active tab");
    // Before the acquisition event: the deterministic monogram presents.
    assert_eq!(snap.tabs[tab_index].favicon, TabFavicon::Monogram);

    h.emit(favicon(view, [0x89, b'P', b'N', b'G']));

    let snap = c.borrow().snapshot();
    assert_eq!(snap.tabs[tab_index].favicon, TabFavicon::Image);
    assert_eq!(
        c.borrow().favicon_bytes(ProfileId(1), "example.com"),
        Some(&[0x89, b'P', b'N', b'G'][..]),
        "the record is keyed by the owning profile and the tab's host"
    );
}

#[test]
fn two_profiles_on_the_same_host_never_share_cache_entries() {
    let (c, h) = wired();
    open(&mut c.borrow_mut(), "https://shared.example/a");
    let first_view = c.borrow().snapshot().active_view.unwrap();

    // Second profile + space, same host.
    let event = c
        .borrow_mut()
        .create_profile_space("Private", "Private")
        .unwrap();
    let (space, second_profile) = match event {
        crate::ControllerEvent::ProfileCreated { space, profile } => (space, profile),
        other => panic!("unexpected event: {other:?}"),
    };
    assert_ne!(
        second_profile,
        ProfileId(1),
        "the domain allocates a fresh profile id for the created profile"
    );
    c.borrow_mut().activate_space(space).unwrap();
    open(&mut c.borrow_mut(), "https://shared.example/b");
    let second_view = c.borrow().snapshot().active_view.unwrap();

    // Each session acquires its own image through its own profile store.
    h.emit(favicon(first_view, vec![1, 2, 3]));
    h.emit(favicon(second_view, vec![4, 5, 6]));

    assert_eq!(
        c.borrow().favicon_bytes(ProfileId(1), "shared.example"),
        Some(&[1, 2, 3][..])
    );
    assert_eq!(
        c.borrow().favicon_bytes(second_profile, "shared.example"),
        Some(&[4, 5, 6][..]),
        "the second profile's record is separate (the domain allocated \
         {second_profile:?} for the created profile)"
    );
    assert_eq!(
        c.borrow().favicon_entries(ProfileId(1)),
        vec![("shared.example".to_string(), vec![1, 2, 3])]
    );
    assert_eq!(
        c.borrow().favicon_entries(second_profile),
        vec![("shared.example".to_string(), vec![4, 5, 6])],
        "two profiles never share a cache entry"
    );
    // Activating profile 1's space presents its own image.
    let first_space = {
        let snap = c.borrow().snapshot();
        snap.spaces
            .iter()
            .find(|s| s.profile == ProfileId(1))
            .unwrap()
            .id
    };
    c.borrow_mut().activate_space(first_space).unwrap();
    let snap = c.borrow().snapshot();
    assert_eq!(snap.tabs[0].favicon, TabFavicon::Image);
}

#[test]
fn repeated_host_acquires_once_and_every_presentation_derives_from_it() {
    let (c, h) = wired();
    open(&mut c.borrow_mut(), "https://repeat.example/one");
    let first_view = c.borrow().snapshot().active_view.unwrap();
    // A second tab on the same host (same profile).
    open(&mut c.borrow_mut(), "https://repeat.example/two");
    let second_view = c.borrow().snapshot().active_view.unwrap();

    h.emit(favicon(first_view, vec![7, 8, 9]));
    // The second tab's session reports no separate acquisition event, yet
    // its presentation derives from the same host record.
    let snap = c.borrow().snapshot();
    assert_eq!(snap.tabs[1].favicon, TabFavicon::Image);
    assert_eq!(c.borrow().favicon_entry_count(ProfileId(1)), 1);
    assert_eq!(c.borrow().favicon_entries(ProfileId(1)).len(), 1);

    // A late event for the second session is a no-op insert (idempotent).
    h.emit(favicon(second_view, vec![7, 8, 9]));
    assert_eq!(c.borrow().favicon_entry_count(ProfileId(1)), 1);
}

#[test]
fn unknown_and_stale_views_are_dropped() {
    let (c, h) = wired();
    open(&mut c.borrow_mut(), "https://a.example/start");
    let view = c.borrow().snapshot().active_view.unwrap();

    // A favicon event for an unknown view is dropped.
    h.emit(favicon(WebViewId(999), vec![1]));
    assert_eq!(c.borrow().favicon_entry_count(ProfileId(1)), 0);

    // A favicon event after the session was destroyed (stale view) is
    // dropped — the registry no longer maps the view to a tab.
    c.borrow_mut().close_tab(0).unwrap();
    h.emit(favicon(view, vec![1]));
    assert_eq!(c.borrow().favicon_entry_count(ProfileId(1)), 0);
}

#[test]
fn candidate_shape_never_lands_in_the_store() {
    let (c, h) = wired();
    open(&mut c.borrow_mut(), "https://cand.example/start");
    let view = c.borrow().snapshot().active_view.unwrap();
    // The contract allows a candidates-shaped event; the controller only
    // stores resolved image bytes (the macOS adapter emits only Image).
    h.emit(WebEvent::FaviconChanged(
        view,
        FaviconContent::Candidates(vec![bbr_web::FaviconCandidate {
            url: "https://cand.example/favicon.png".into(),
            size: Some(32),
        }]),
    ));
    assert_eq!(c.borrow().favicon_entry_count(ProfileId(1)), 0);
    let snap = c.borrow().snapshot();
    assert_eq!(snap.tabs[0].favicon, TabFavicon::Monogram);
}
