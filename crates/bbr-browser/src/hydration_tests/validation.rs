//! Typed validation-rejection contract tests.
//!
//! Every planning dependency is rejected with the exact [`HydrationError`]
//! variant before any plan exists: unknown active space, duplicate
//! identities, non-pinned prefixes, non-member active tabs/split panes,
//! recently-closed collisions, and invalid URLs.

use bbr_domain::SpaceId;

use super::support::{closed, intent, pinned_tab, profile, space, tab};
use crate::{HydrationError, HydrationPolicy, IntentSplit};

#[test]
fn unknown_active_space_is_rejected() {
    let intent = intent(vec![profile(
        1,
        vec![space(10, vec![tab(100, "https://a.example")])],
    )]);
    assert_eq!(
        HydrationPolicy::plan(&intent, SpaceId(99)),
        Err(HydrationError::UnknownActiveSpace(SpaceId(99)))
    );
}

#[test]
fn duplicate_identities_are_rejected_across_all_kinds() {
    // Duplicate profile id.
    let dup_profiles = intent(vec![
        profile(1, vec![space(10, vec![tab(100, "https://a.example")])]),
        profile(1, vec![]),
    ]);
    assert_eq!(
        HydrationPolicy::plan(&dup_profiles, SpaceId(10)),
        Err(HydrationError::DuplicateIdentity(1))
    );

    // Duplicate space id within one profile.
    let dup_spaces = intent(vec![profile(
        1,
        vec![
            space(10, vec![tab(100, "https://a.example")]),
            space(10, vec![]),
        ],
    )]);
    assert_eq!(
        HydrationPolicy::plan(&dup_spaces, SpaceId(10)),
        Err(HydrationError::DuplicateIdentity(10))
    );

    // Duplicate tab id within one space.
    let dup_tabs = intent(vec![profile(
        1,
        vec![space(
            10,
            vec![tab(100, "https://a.example"), tab(100, "https://b.example")],
        )],
    )]);
    assert_eq!(
        HydrationPolicy::plan(&dup_tabs, SpaceId(10)),
        Err(HydrationError::DuplicateIdentity(100))
    );
}

#[test]
fn non_pinned_prefix_is_rejected() {
    let sp = space(
        10,
        vec![
            tab(100, "https://a.example"),
            pinned_tab(101, "https://b.example"),
        ],
    );
    let intent = intent(vec![profile(1, vec![sp])]);
    assert_eq!(
        HydrationPolicy::plan(&intent, SpaceId(10)),
        Err(HydrationError::InvalidPinnedRegion { space_id: 10 })
    );
}

#[test]
fn active_tab_must_be_a_member_of_its_space() {
    let sp = space(10, vec![tab(100, "https://a.example")]);
    let mut intent = intent(vec![profile(1, vec![sp])]);
    intent.profiles[0].spaces[0].active_tab = Some(999);
    assert_eq!(
        HydrationPolicy::plan(&intent, SpaceId(10)),
        Err(HydrationError::InvalidActiveTab {
            space_id: 10,
            tab_id: 999
        })
    );
}

#[test]
fn split_panes_must_be_distinct_members() {
    // Identical panes.
    let mut identical = space(10, vec![tab(100, "https://a.example")]);
    identical.split = Some(IntentSplit {
        primary_tab: 100,
        secondary_tab: 100,
    });
    let identical_intent = intent(vec![profile(1, vec![identical])]);
    assert_eq!(
        HydrationPolicy::plan(&identical_intent, SpaceId(10)),
        Err(HydrationError::InvalidSplit { space_id: 10 })
    );

    // Non-member pane.
    let mut non_member = space(10, vec![tab(100, "https://a.example")]);
    non_member.split = Some(IntentSplit {
        primary_tab: 100,
        secondary_tab: 999,
    });
    let non_member_intent = intent(vec![profile(1, vec![non_member])]);
    assert_eq!(
        HydrationPolicy::plan(&non_member_intent, SpaceId(10)),
        Err(HydrationError::InvalidSplit { space_id: 10 })
    );
}

#[test]
fn recently_closed_collisions_are_rejected() {
    // Closed id collides with a live tab id.
    let mut collides_live = space(10, vec![tab(100, "https://a.example")]);
    collides_live.recently_closed = vec![closed(100, "https://closed.example", 0)];
    let colliding = intent(vec![profile(1, vec![collides_live])]);
    assert_eq!(
        HydrationPolicy::plan(&colliding, SpaceId(10)),
        Err(HydrationError::RecentlyClosedCollision { tab_id: 100 })
    );

    // Two closed entries share one id.
    let mut duplicate_closed = space(10, vec![tab(100, "https://a.example")]);
    duplicate_closed.recently_closed = vec![
        closed(200, "https://closed.example", 0),
        closed(200, "https://closed2.example", 1),
    ];
    let duplicated = intent(vec![profile(1, vec![duplicate_closed])]);
    assert_eq!(
        HydrationPolicy::plan(&duplicated, SpaceId(10)),
        Err(HydrationError::RecentlyClosedCollision { tab_id: 200 })
    );

    // A closed id in one space collides with a live tab id in another space
    // (the identity contract is global, not per-space).
    let mut with_closed = space(10, vec![tab(100, "https://a.example")]);
    with_closed.recently_closed = vec![closed(200, "https://closed.example", 0)];
    let cross_space = intent(vec![profile(
        1,
        vec![with_closed, space(11, vec![tab(200, "https://b.example")])],
    )]);
    assert_eq!(
        HydrationPolicy::plan(&cross_space, SpaceId(10)),
        Err(HydrationError::RecentlyClosedCollision { tab_id: 200 })
    );
}

#[test]
fn invalid_urls_are_rejected() {
    // Empty live-tab url.
    let mut empty = space(10, vec![tab(100, "")]);
    empty.active_tab = Some(100);
    let empty_url = intent(vec![profile(1, vec![empty])]);
    assert_eq!(
        HydrationPolicy::plan(&empty_url, SpaceId(10)),
        Err(HydrationError::InvalidUrl { tab_id: 100 })
    );

    // Whitespace in a live-tab url.
    let whitespace = space(10, vec![tab(100, "https://a b.example")]);
    let whitespace_url = intent(vec![profile(1, vec![whitespace])]);
    assert_eq!(
        HydrationPolicy::plan(&whitespace_url, SpaceId(10)),
        Err(HydrationError::InvalidUrl { tab_id: 100 })
    );

    // Empty closed-tab url.
    let mut bad_closed = space(10, vec![tab(100, "https://a.example")]);
    bad_closed.recently_closed = vec![closed(200, "", 0)];
    let closed_url = intent(vec![profile(1, vec![bad_closed])]);
    assert_eq!(
        HydrationPolicy::plan(&closed_url, SpaceId(10)),
        Err(HydrationError::InvalidUrl { tab_id: 200 })
    );
}
