//! Plan-shape and visibility contract tests.
//!
//! These prove the phase separation (`identities` first, then exactly the
//! visible-set `sessions`), the split/active-tab visibility precedence, the
//! dense no-eager-session-explosion bound, determinism, cross-profile store
//! selection, and the default-active-space policy — all through the real
//! [`HydrationPolicy::plan`] path.

use bbr_domain::{ProfileHue, SpaceId, TabId};
use bbr_web::ProfileId;

use super::support::{closed, dense_500, intent, pinned_tab, profile, space, tab};
use crate::{
    HydrationPolicy, IntentSplit, RestoreIntent, RestoredClosedTab, RestoredSplit, SessionRequest,
};

#[test]
fn plan_restores_identities_first_and_hydrates_only_the_active_spaces_visible_tab() {
    let mut active = space(
        10,
        vec![
            tab(100, "https://a.example"),
            tab(101, "https://b.example"),
            tab(102, "https://c.example"),
        ],
    );
    active.active_tab = Some(102);
    let intent = intent(vec![profile(
        1,
        vec![active, space(11, vec![tab(103, "https://d.example")])],
    )]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    // Phase 1: the complete identity graph, in intent order.
    assert_eq!(plan.identities.len(), 1);
    let restored = &plan.identities[0];
    assert_eq!(restored.id, ProfileId(1));
    assert_eq!(restored.title, "profile-1");
    assert_eq!(restored.hue, ProfileHue::Cobalt);
    assert_eq!(restored.spaces.len(), 2);
    assert_eq!(restored.spaces[0].id, SpaceId(10));
    assert_eq!(restored.spaces[0].tabs.len(), 3);
    assert_eq!(restored.spaces[0].active_tab, Some(TabId(102)));
    assert_eq!(restored.spaces[0].tabs[2].url, "https://c.example");
    assert_eq!(restored.spaces[1].tabs[0].id, TabId(103));

    // Phase 2: exactly the active space's visible tab.
    assert_eq!(
        plan.sessions,
        vec![SessionRequest {
            tab: TabId(102),
            profile: ProfileId(1),
            url: "https://c.example".into(),
        }]
    );
    // Everything else is deferred, in intent order.
    assert_eq!(plan.deferred, vec![TabId(100), TabId(101), TabId(103)]);
}

#[test]
fn plan_hydrates_both_split_panes_of_the_active_space_in_order() {
    let mut split_space = space(
        10,
        vec![
            tab(100, "https://a.example"),
            tab(101, "https://b.example"),
            tab(102, "https://c.example"),
        ],
    );
    split_space.active_tab = Some(100);
    split_space.split = Some(IntentSplit {
        primary_tab: 100,
        secondary_tab: 102,
    });
    let intent = intent(vec![profile(1, vec![split_space])]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    assert_eq!(
        plan.sessions,
        vec![
            SessionRequest {
                tab: TabId(100),
                profile: ProfileId(1),
                url: "https://a.example".into(),
            },
            SessionRequest {
                tab: TabId(102),
                profile: ProfileId(1),
                url: "https://c.example".into(),
            },
        ],
        "the split panes hydrate in primary/secondary order"
    );
    assert_eq!(plan.deferred, vec![TabId(101)]);
    assert_eq!(
        plan.identities[0].spaces[0].split,
        Some(RestoredSplit {
            primary_tab: TabId(100),
            secondary_tab: TabId(102),
        })
    );
}

#[test]
fn splits_and_active_tabs_of_non_active_spaces_hydrate_nothing() {
    let mut active = space(10, vec![tab(100, "https://a.example")]);
    active.active_tab = Some(100);
    let mut other = space(
        11,
        vec![tab(101, "https://b.example"), tab(102, "https://c.example")],
    );
    other.active_tab = Some(102);
    other.split = Some(IntentSplit {
        primary_tab: 101,
        secondary_tab: 102,
    });
    let intent = intent(vec![profile(1, vec![active, other])]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    assert_eq!(
        plan.sessions,
        vec![SessionRequest {
            tab: TabId(100),
            profile: ProfileId(1),
            url: "https://a.example".into(),
        }],
        "only the active space's visible set hydrates; the other space's split stays identity-only"
    );
    assert_eq!(plan.deferred, vec![TabId(101), TabId(102)]);
}

#[test]
fn an_empty_active_space_hydrates_nothing() {
    let mut empty = space(10, Vec::new());
    empty.active_tab = None;
    let intent = intent(vec![profile(
        1,
        vec![empty, space(11, vec![tab(100, "https://a.example")])],
    )]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    assert!(plan.sessions.is_empty());
    assert_eq!(plan.deferred, vec![TabId(100)]);
}

#[test]
fn sessions_use_the_owning_profiles_store() {
    let mut one = space(10, vec![tab(100, "https://one.example")]);
    one.active_tab = Some(100);
    let mut two = space(20, vec![tab(200, "https://two.example")]);
    two.active_tab = Some(200);
    let intent = intent(vec![profile(1, vec![one]), profile(2, vec![two])]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(20)).expect("valid intent plans");
    assert_eq!(
        plan.sessions,
        vec![SessionRequest {
            tab: TabId(200),
            profile: ProfileId(2),
            url: "https://two.example".into(),
        }],
        "the visible tab hydrates in its own profile's store"
    );
}

#[test]
fn pinned_flags_hues_and_recently_closed_survive_into_the_plan() {
    let mut sp = space(
        10,
        vec![
            pinned_tab(100, "https://pinned.example"),
            pinned_tab(101, "https://pinned2.example"),
            tab(102, "https://regular.example"),
        ],
    );
    sp.active_tab = Some(100);
    sp.recently_closed = vec![closed(200, "https://closed.example", 1)];
    let mut prof = profile(1, vec![sp]);
    prof.hue = ProfileHue::Fuchsia;
    let intent = intent(vec![prof]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    let restored = &plan.identities[0];
    assert_eq!(restored.hue, ProfileHue::Fuchsia);
    assert_eq!(
        restored.spaces[0]
            .tabs
            .iter()
            .map(|t| (t.id, t.pinned))
            .collect::<Vec<_>>(),
        vec![(TabId(100), true), (TabId(101), true), (TabId(102), false)]
    );
    assert_eq!(
        restored.spaces[0].recently_closed,
        vec![RestoredClosedTab {
            tab_id: TabId(200),
            title: "closed-200".into(),
            url: "https://closed.example".into(),
            pinned: false,
            former_position: 1,
            in_place: false,
        }]
    );
    // Closed entries are identities — never sessions.
    assert_eq!(plan.sessions.len(), 1);
    assert_eq!(plan.sessions[0].tab, TabId(100));
}

#[test]
fn dense_500_tab_intent_plans_exactly_one_session() {
    let intent = dense_500();
    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    let total_tabs: usize = plan
        .identities
        .iter()
        .flat_map(|p| p.spaces.iter())
        .map(|s| s.tabs.len())
        .sum();
    assert_eq!(total_tabs, 500, "every identity is restored");
    assert_eq!(
        plan.sessions,
        vec![SessionRequest {
            tab: TabId(100),
            profile: ProfileId(1),
            url: "https://dense100.example".into(),
        }],
        "exactly the selected tab hydrates"
    );
    assert_eq!(plan.deferred.len(), 499);

    // The dense fixture's full shape survives into the plan as identities:
    // per-space active tabs, pinned prefixes, the non-active split space, and
    // recently-closed entries — none of which hydrate anything.
    let spaces = plan
        .identities
        .iter()
        .flat_map(|p| p.spaces.iter())
        .collect::<Vec<_>>();
    assert_eq!(
        spaces
            .iter()
            .map(|s| (s.id, s.active_tab))
            .collect::<Vec<_>>(),
        vec![
            (SpaceId(10), Some(TabId(100))),
            (SpaceId(11), Some(TabId(352))),
            (SpaceId(20), Some(TabId(450))),
        ]
    );
    let pinned_prefixes = spaces
        .iter()
        .map(|s| s.tabs.iter().take_while(|t| t.pinned).count())
        .collect::<Vec<_>>();
    assert_eq!(
        pinned_prefixes,
        vec![3, 2, 2],
        "every space keeps its pinned prefix"
    );
    assert_eq!(
        spaces[1].split,
        Some(RestoredSplit {
            primary_tab: TabId(350),
            secondary_tab: TabId(351),
        }),
        "the non-active split space keeps its split as identity"
    );
    assert_eq!(
        spaces
            .iter()
            .map(|s| s.recently_closed.len())
            .sum::<usize>(),
        3,
        "recently-closed entries restore as identities"
    );
    assert!(
        plan.deferred.contains(&TabId(350)) && plan.deferred.contains(&TabId(351)),
        "the split panes stay identity-only, deferred"
    );
}

#[test]
fn plan_is_deterministic_for_the_same_intent() {
    let intent = dense_500();
    let a = HydrationPolicy::plan(&intent, SpaceId(10)).expect("plan a");
    let b = HydrationPolicy::plan(&intent, SpaceId(10)).expect("plan b");
    assert_eq!(a, b);
    // Identity restoration order is intent order, never map-iteration order.
    let restored_spaces: Vec<SpaceId> = a
        .identities
        .iter()
        .flat_map(|p| p.spaces.iter())
        .map(|s| s.id)
        .collect();
    assert_eq!(restored_spaces, vec![SpaceId(10), SpaceId(11), SpaceId(20)]);
}

#[test]
fn default_active_space_prefers_the_first_space_with_an_active_tab() {
    let intent = intent(vec![profile(
        1,
        vec![space(10, Vec::new()), {
            let mut sp = space(11, vec![tab(100, "https://a.example")]);
            sp.active_tab = Some(100);
            sp
        }],
    )]);
    assert_eq!(
        HydrationPolicy::default_active_space(&intent),
        Some(SpaceId(11)),
        "the first space that owns an active tab wins"
    );
    assert_eq!(
        HydrationPolicy::default_active_space(&RestoreIntent::default()),
        None
    );
}

#[test]
fn split_precedence_matches_domain_visibility_even_with_an_outside_active_tab() {
    // A snapshot may carry a split plus an active tab outside its panes.
    // Domain `Browser::is_visible` gives the split precedence (the domain
    // dissolves a split only when a non-member tab is selected later), so
    // the policy hydrates the panes and restores both the split and the
    // outside active tab as identities for the composition to reconcile.
    let mut sp = space(
        10,
        vec![
            tab(100, "https://a.example"),
            tab(101, "https://b.example"),
            tab(102, "https://c.example"),
        ],
    );
    sp.active_tab = Some(101);
    sp.split = Some(IntentSplit {
        primary_tab: 100,
        secondary_tab: 102,
    });
    let intent = intent(vec![profile(1, vec![sp])]);

    let plan = HydrationPolicy::plan(&intent, SpaceId(10)).expect("valid intent plans");

    assert_eq!(
        plan.sessions,
        vec![
            SessionRequest {
                tab: TabId(100),
                profile: ProfileId(1),
                url: "https://a.example".into(),
            },
            SessionRequest {
                tab: TabId(102),
                profile: ProfileId(1),
                url: "https://c.example".into(),
            },
        ],
        "the split wins over the outside active tab, matching domain is_visible"
    );
    assert_eq!(plan.deferred, vec![TabId(101)]);
    assert_eq!(plan.identities[0].spaces[0].active_tab, Some(TabId(101)));
    assert_eq!(
        plan.identities[0].spaces[0].split,
        Some(RestoredSplit {
            primary_tab: TabId(100),
            secondary_tab: TabId(102),
        })
    );
}
