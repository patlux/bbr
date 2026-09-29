//! Unit tests for the id-preserving restore import API (`Browser::restore_*`):
//! identity preservation with persisted ids, allocator advancement, collision
//! rejection, the session-less (Suspended) deferred contract, split/active-tab
//! precedence without selection side effects, and recently-closed restoration.
//! These tests run without macOS frameworks or a display server.

use crate::{Browser, DomainError, ProfileHue, ProfileId, SpaceId, Split, TabId, TabLifecycle};

/// Restore the canonical small model: profile 1 with space 10 (tabs 100, 101,
/// 102; pinned prefix [100]; active 101) plus space 11 (tab 110, active 110),
/// one recently-closed entry per space, and a split over 101/102 in space 10.
/// This is the **mid-restore** state: every identity is imported session-less
/// (Suspended), which the invariant checker only accepts once the visible set
/// is hydrated (see [`activate_visible`]).
fn restored() -> Browser {
    let mut b = Browser::new();
    b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "Work")
        .unwrap();
    b.restore_space(SpaceId(10), ProfileId(1), "Main").unwrap();
    b.restore_tab(TabId(100), SpaceId(10), "Pinned", "https://a.example", true)
        .unwrap();
    b.restore_tab(TabId(101), SpaceId(10), "One", "https://b.example", false)
        .unwrap();
    b.restore_tab(TabId(102), SpaceId(10), "Two", "https://c.example", false)
        .unwrap();
    b.restore_active_tab(SpaceId(10), Some(TabId(101))).unwrap();
    b.restore_split(
        SpaceId(10),
        Some(Split {
            primary: TabId(101),
            secondary: TabId(102),
        }),
    )
    .unwrap();
    b.restore_recently_closed(
        SpaceId(10),
        crate::Tab::new(TabId(900), SpaceId(10), "Closed", "https://d.example"),
        1,
    )
    .unwrap();
    b.restore_space(SpaceId(11), ProfileId(1), "Second")
        .unwrap();
    b.restore_tab(TabId(110), SpaceId(11), "Solo", "https://e.example", false)
        .unwrap();
    b.restore_active_tab(SpaceId(11), Some(TabId(110))).unwrap();
    b.restore_recently_closed(
        SpaceId(11),
        crate::Tab::new(TabId(901), SpaceId(11), "Gone", "https://f.example"),
        0,
    )
    .unwrap();
    b
}

/// Complete a restore the way the policy does: activate exactly the visible
/// set (split panes of space 10, the single active tab of space 11) so the
/// domain invariant checker accepts the model.
fn activate_visible(b: &mut Browser) {
    for tab in [TabId(101), TabId(102), TabId(110)] {
        b.restore_activate_tab(tab).unwrap();
    }
}

#[test]
fn restore_preserves_ids_order_pins_active_split_and_closed() {
    let mut b = restored();
    activate_visible(&mut b);
    b.check_invariants().unwrap();

    let profile = b.profile(ProfileId(1)).expect("restored profile");
    assert_eq!(profile.title, "Work");
    assert_eq!(profile.hue, ProfileHue::Cobalt);

    let space = b.space(SpaceId(10)).expect("restored space");
    assert_eq!(space.tabs, vec![TabId(100), TabId(101), TabId(102)]);
    assert_eq!(space.pinned_count, 1, "the pinned prefix is restored");
    assert_eq!(space.active_tab, Some(TabId(101)));
    assert_eq!(
        space.splits,
        vec![
            crate::tiling::SplitGroup::pair(TabId(101), TabId(102), crate::tiling::Axis::Row)
                .unwrap()
        ]
    );
    assert_eq!(space.recently_closed.len(), 1);
    assert_eq!(space.recently_closed[0].tab.id, TabId(900));
    assert_eq!(space.recently_closed[0].index, 1, "former position kept");

    let tab = b.tab(TabId(100)).expect("restored tab");
    assert!(tab.pinned);
    assert_eq!(tab.url, "https://a.example");
}

#[test]
fn restore_advances_the_allocator_past_imported_ids() {
    let mut b = restored();
    activate_visible(&mut b);
    // A fresh creation must never reuse a restored id (all kinds share one
    // identity space).
    let profile = b.create_profile("Fresh");
    let space = b.create_space(profile, "Fresh Space").unwrap();
    let tab = b.open_tab(space, "", "https://fresh.example").unwrap();
    assert!(profile.0 > 901, "fresh profile id above every restored id");
    assert!(space.0 > 901);
    assert!(tab.0 > 901);
    b.check_invariants().unwrap();
}

#[test]
fn restore_rejects_duplicate_and_zero_ids() {
    let mut b = Browser::new();
    b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "A")
        .unwrap();
    // The same id as a space collides across kinds.
    assert!(matches!(
        b.restore_space(SpaceId(1), ProfileId(1), "B"),
        Err(DomainError::RestoreCollision(1))
    ));
    assert!(matches!(
        b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "again"),
        Err(DomainError::RestoreCollision(1))
    ));
    // Recently-closed ids join the identity space too.
    b.restore_space(SpaceId(10), ProfileId(1), "Main").unwrap();
    assert!(matches!(
        b.restore_recently_closed(
            SpaceId(10),
            crate::Tab::new(TabId(1), SpaceId(10), "", "u"),
            0,
        ),
        Err(DomainError::RestoreCollision(1))
    ));
    // Zero is never a valid persisted identity.
    assert!(matches!(
        b.restore_profile(ProfileId(0), ProfileHue::Cobalt, "zero"),
        Err(DomainError::RestoreCollision(0))
    ));
}

#[test]
fn restore_rejects_broken_ownership_and_membership() {
    let mut b = Browser::new();
    // Space before its profile.
    assert!(matches!(
        b.restore_space(SpaceId(10), ProfileId(1), "orphan"),
        Err(DomainError::UnknownProfile(ProfileId(1)))
    ));
    b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "Work")
        .unwrap();
    b.restore_space(SpaceId(10), ProfileId(1), "Main").unwrap();
    // Active tab that is not a member.
    assert!(matches!(
        b.restore_active_tab(SpaceId(10), Some(TabId(999))),
        Err(DomainError::TabNotInSpace {
            tab: TabId(999),
            space: SpaceId(10)
        })
    ));
    // Split panes that are not members.
    assert!(matches!(
        b.restore_split(
            SpaceId(10),
            Some(Split {
                primary: TabId(999),
                secondary: TabId(1000),
            }),
        ),
        Err(DomainError::TabNotInSpace { .. })
    ));
    // A tab into a missing space.
    assert!(matches!(
        b.restore_tab(TabId(100), SpaceId(77), "", "u", false),
        Err(DomainError::UnknownSpace(SpaceId(77)))
    ));
}

#[test]
fn restored_tabs_are_session_less_until_hydration_activates_them() {
    let mut b = restored();
    for id in [TabId(100), TabId(101), TabId(102), TabId(110)] {
        assert_eq!(
            b.tab(id).unwrap().lifecycle,
            TabLifecycle::Suspended,
            "restored tabs are session-less by design"
        );
    }
    // Hydration activates exactly the visible set; everything else stays
    // session-less.
    b.restore_activate_tab(TabId(101)).unwrap();
    assert_eq!(b.tab(TabId(101)).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(
        b.tab(TabId(102)).unwrap().lifecycle,
        TabLifecycle::Suspended
    );
    // The active-tab and split fields are untouched by activation.
    assert_eq!(b.space(SpaceId(10)).unwrap().active_tab, Some(TabId(101)));
    assert!(!b.space(SpaceId(10)).unwrap().splits.is_empty());
    // Unknown activation is rejected.
    assert!(matches!(
        b.restore_activate_tab(TabId(777)),
        Err(DomainError::UnknownTab(TabId(777)))
    ));
}

#[test]
fn split_with_outside_active_tab_reconciles_active_into_panes() {
    let mut b = Browser::new();
    b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "Work")
        .unwrap();
    b.restore_space(SpaceId(10), ProfileId(1), "Main").unwrap();
    b.restore_tab(TabId(100), SpaceId(10), "A", "u1", false)
        .unwrap();
    b.restore_tab(TabId(101), SpaceId(10), "B", "u2", false)
        .unwrap();
    b.restore_tab(TabId(102), SpaceId(10), "C", "u3", false)
        .unwrap();
    // Active tab outside the split panes (the policy's split-precedence
    // edge): the split is restored and the outside active tab is reconciled
    // into the panes like `set_split` does (the domain invariant requires a
    // split space's active tab to be a member) — the outside tab stays a
    // member identity of the space, never dropped.
    b.restore_active_tab(SpaceId(10), Some(TabId(102))).unwrap();
    b.restore_split(
        SpaceId(10),
        Some(Split {
            primary: TabId(100),
            secondary: TabId(101),
        }),
    )
    .unwrap();
    assert_eq!(b.space(SpaceId(10)).unwrap().active_tab, Some(TabId(100)));
    assert!(!b.space(SpaceId(10)).unwrap().splits.is_empty());
    // Tab 102 is still a member identity of the space.
    assert!(b.space(SpaceId(10)).unwrap().tabs.contains(&TabId(102)));
    // Visibility follows `is_visible`: split precedence means the panes are
    // visible and the outside tab is not.
    assert!(b.is_visible(TabId(100)));
    assert!(b.is_visible(TabId(101)));
    assert!(!b.is_visible(TabId(102)));
    // Complete the restore the policy way: activate the visible panes.
    for tab in [TabId(100), TabId(101)] {
        b.restore_activate_tab(tab).unwrap();
    }
    b.check_invariants().unwrap();
}

#[test]
fn restore_split_rejects_duplicates_and_existing_split() {
    let mut b = Browser::new();
    b.restore_profile(ProfileId(1), ProfileHue::Cobalt, "Work")
        .unwrap();
    b.restore_space(SpaceId(10), ProfileId(1), "Main").unwrap();
    b.restore_tab(TabId(100), SpaceId(10), "A", "u1", false)
        .unwrap();
    b.restore_tab(TabId(101), SpaceId(10), "B", "u2", false)
        .unwrap();
    assert!(matches!(
        b.restore_split(
            SpaceId(10),
            Some(Split {
                primary: TabId(100),
                secondary: TabId(100),
            }),
        ),
        Err(DomainError::SplitRequiresDistinctTabs)
    ));
    b.restore_split(
        SpaceId(10),
        Some(Split {
            primary: TabId(100),
            secondary: TabId(101),
        }),
    )
    .unwrap();
    assert!(matches!(
        b.restore_split(
            SpaceId(10),
            Some(Split {
                primary: TabId(100),
                secondary: TabId(101),
            }),
        ),
        Err(DomainError::SplitAlreadyExists(SpaceId(10)))
    ));
    // Clearing an existing split is legal and idempotent.
    b.restore_split(SpaceId(10), None).unwrap();
    assert!(b.space(SpaceId(10)).unwrap().splits.is_empty());
}

#[test]
fn recently_closed_restores_and_reopens_at_former_position() {
    let mut b = restored();
    activate_visible(&mut b);
    // The closed tab id is reserved: a fresh open cannot reuse it.
    let profile = b.create_profile("Fresh");
    let space = b.create_space(profile, "Fresh Space").unwrap();
    let fresh = b.open_tab(space, "", "https://fresh.example").unwrap();
    assert!(fresh.0 > 901);
    // Reopen restores the closed identity at its former position.
    let reopened = b.reopen_tab(SpaceId(11)).unwrap();
    assert_eq!(reopened, TabId(901));
    assert_eq!(b.tab(reopened).unwrap().title, "Gone");
    assert_eq!(b.space(SpaceId(11)).unwrap().active_tab, Some(TabId(901)));
    b.check_invariants().unwrap();
}
