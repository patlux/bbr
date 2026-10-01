//! Validation and planning internals.
//!
//! [`ValidatedGraph`] is the single validation pass over a [`RestoreIntent`]:
//! it checks every planning dependency — the active space exists,
//! profile/space/tab ids are globally unique, every space has a pinned
//! prefix, the active tab and split panes are members, recently-closed ids
//! collide with nothing live, and every URL satisfies the bounded URL
//! contract — and normalizes the intent into domain-typed identities plus the
//! resolution maps [`HydrationPolicy::plan`] needs. Only a successful pass
//! produces a graph, so an intent is either fully valid or rejected with a
//! typed [`HydrationError`] before any plan exists.

use std::collections::{HashMap, HashSet};

use bbr_domain::{SpaceId, TabId};
use bbr_web::ProfileId;

use super::{
    HydrationError, RestoreIntent, RestorePlan, RestoredClosedTab, RestoredProfile, RestoredSpace,
    RestoredSplit, RestoredTab, SessionRequest,
};

/// Validated, normalized intent graph plus the resolution maps planning
/// needs. Produced only by a successful validation pass.
pub(super) struct ValidatedGraph {
    identities: Vec<RestoredProfile>,
    /// Live tab id → owning space id.
    tab_spaces: HashMap<u64, u64>,
    /// Space id → owning profile id.
    space_profiles: HashMap<u64, u64>,
    /// Tab id → persisted url (for session requests).
    tab_urls: HashMap<u64, String>,
    /// All live tab ids in intent order.
    ordered_tabs: Vec<u64>,
    /// The active space's visible tab ids, in deterministic order (split
    /// primary, then secondary; otherwise the single active tab).
    visible: Vec<u64>,
}

impl ValidatedGraph {
    pub(super) fn from_intent(
        intent: &RestoreIntent,
        active_space: SpaceId,
    ) -> Result<Self, HydrationError> {
        // The policy's key input is validated first: the space the first
        // usable frame shows must exist in the intent.
        let active_space_exists = intent.profiles.iter().any(|profile| {
            profile
                .spaces
                .iter()
                .any(|space| space.id == active_space.0)
        });
        if !active_space_exists {
            return Err(HydrationError::UnknownActiveSpace(active_space));
        }

        let mut identities_set = HashSet::new();
        let mut tab_spaces = HashMap::new();
        let mut space_profiles = HashMap::new();
        let mut tab_urls = HashMap::new();
        let mut ordered_tabs = Vec::new();
        let mut visible = Vec::new();
        // Recently-closed identities are validated after the full pass, once
        // every live identity is known (the persistence identity contract is
        // global: a closed id must not collide with any live identity in any
        // space, or with any other closed id).
        let mut closed_ids: HashSet<u64> = HashSet::new();
        let mut closed_order: Vec<u64> = Vec::new();

        let mut identities = Vec::with_capacity(intent.profiles.len());
        for profile in &intent.profiles {
            insert_identity(&mut identities_set, profile.id)?;
            let mut restored_spaces = Vec::with_capacity(profile.spaces.len());
            for space in &profile.spaces {
                insert_identity(&mut identities_set, space.id)?;
                space_profiles.insert(space.id, profile.id);

                let mut members = HashSet::new();
                let mut closed_rows = HashSet::new();
                let mut saw_regular = false;
                let mut restored_tabs = Vec::with_capacity(space.tabs.len());
                for tab in &space.tabs {
                    insert_identity(&mut identities_set, tab.id)?;
                    if !valid_url(&tab.url) {
                        return Err(HydrationError::InvalidUrl { tab_id: tab.id });
                    }
                    members.insert(tab.id);
                    ordered_tabs.push(tab.id);
                    tab_spaces.insert(tab.id, space.id);
                    tab_urls.insert(tab.id, tab.url.clone());
                    if !tab.pinned {
                        saw_regular = true;
                    } else if saw_regular {
                        return Err(HydrationError::InvalidPinnedRegion { space_id: space.id });
                    }
                    // DESIGN.md §10.12: only a pinned tab keeps a pinned
                    // page or waits closed in place.
                    if (tab.pinned_page.is_some() || tab.closed) && !tab.pinned {
                        return Err(HydrationError::InvalidKeptPlace { tab_id: tab.id });
                    }
                    if tab
                        .pinned_page
                        .as_ref()
                        .is_some_and(|page| !valid_url(&page.url))
                    {
                        return Err(HydrationError::InvalidUrl { tab_id: tab.id });
                    }
                    if tab.closed {
                        closed_rows.insert(tab.id);
                    }
                    restored_tabs.push(RestoredTab {
                        id: TabId(tab.id),
                        title: tab.title.clone(),
                        url: tab.url.clone(),
                        pinned: tab.pinned,
                        pinned_page: tab.pinned_page.clone(),
                        closed: tab.closed,
                    });
                }

                // A closed row is never shown: it cannot be the selection
                // or a split pane.
                if let Some(active) = space.active_tab {
                    if !members.contains(&active) || closed_rows.contains(&active) {
                        return Err(HydrationError::InvalidActiveTab {
                            space_id: space.id,
                            tab_id: active,
                        });
                    }
                }
                if let Some(split) = space.split {
                    if split.primary_tab == split.secondary_tab
                        || !members.contains(&split.primary_tab)
                        || !members.contains(&split.secondary_tab)
                        || closed_rows.contains(&split.primary_tab)
                        || closed_rows.contains(&split.secondary_tab)
                    {
                        return Err(HydrationError::InvalidSplit { space_id: space.id });
                    }
                }

                let mut restored_closed = Vec::with_capacity(space.recently_closed.len());
                let mut in_place_rows = HashSet::new();
                for closed in &space.recently_closed {
                    if closed.in_place {
                        // An in-place record names a closed row of this
                        // space (its live identity), at most once.
                        if !closed_rows.contains(&closed.tab_id)
                            || !in_place_rows.insert(closed.tab_id)
                        {
                            return Err(HydrationError::InvalidKeptPlace {
                                tab_id: closed.tab_id,
                            });
                        }
                    } else if !closed_ids.insert(closed.tab_id) {
                        // Closed identities must be globally unique among
                        // themselves; live-identity collisions are checked
                        // after the full pass.
                        return Err(HydrationError::RecentlyClosedCollision {
                            tab_id: closed.tab_id,
                        });
                    }
                    if !closed.in_place {
                        closed_order.push(closed.tab_id);
                    }
                    if !valid_url(&closed.url) {
                        return Err(HydrationError::InvalidUrl {
                            tab_id: closed.tab_id,
                        });
                    }
                    restored_closed.push(RestoredClosedTab {
                        tab_id: TabId(closed.tab_id),
                        title: closed.title.clone(),
                        url: closed.url.clone(),
                        pinned: closed.pinned,
                        former_position: closed.former_position,
                        in_place: closed.in_place,
                    });
                }

                if space.id == active_space.0 {
                    visible = match space.split {
                        Some(split) => vec![split.primary_tab, split.secondary_tab],
                        None => space.active_tab.into_iter().collect(),
                    };
                }

                restored_spaces.push(RestoredSpace {
                    id: SpaceId(space.id),
                    profile: ProfileId(profile.id),
                    title: space.title.clone(),
                    tabs: restored_tabs,
                    active_tab: space.active_tab.map(TabId),
                    recently_closed: restored_closed,
                    split: space.split.map(|split| RestoredSplit {
                        primary_tab: TabId(split.primary_tab),
                        secondary_tab: TabId(split.secondary_tab),
                    }),
                });
            }
            identities.push(RestoredProfile {
                id: ProfileId(profile.id),
                title: profile.title.clone(),
                hue: profile.hue,
                spaces: restored_spaces,
            });
        }

        // Global closed-identity contract: no closed id may collide with any
        // live identity (profile, space, or tab) after the full graph is
        // known.
        for closed_id in &closed_order {
            if identities_set.contains(closed_id) {
                return Err(HydrationError::RecentlyClosedCollision { tab_id: *closed_id });
            }
        }

        Ok(Self {
            identities,
            tab_spaces,
            space_profiles,
            tab_urls,
            ordered_tabs,
            visible,
        })
    }

    pub(super) fn plan(self) -> RestorePlan {
        let sessions = self
            .visible
            .iter()
            .map(|&tab_id| SessionRequest {
                tab: TabId(tab_id),
                profile: ProfileId(self.space_profiles[&self.tab_spaces[&tab_id]]),
                url: self.tab_urls[&tab_id].clone(),
            })
            .collect::<Vec<_>>();

        let hydrated: HashSet<u64> = self.visible.iter().copied().collect();
        let deferred = self
            .ordered_tabs
            .iter()
            .copied()
            .filter(|tab| !hydrated.contains(tab))
            .map(TabId)
            .collect();

        RestorePlan {
            identities: self.identities,
            sessions,
            deferred,
        }
    }
}

/// Insert one identity into the global set, rejecting duplicates.
fn insert_identity(ids: &mut HashSet<u64>, id: u64) -> Result<(), HydrationError> {
    if ids.insert(id) {
        Ok(())
    } else {
        Err(HydrationError::DuplicateIdentity(id))
    }
}

/// The bounded URL contract `bbr-persistence` enforces: required URLs are
/// non-empty and contain no ASCII whitespace, control, or NUL bytes.
fn valid_url(url: &str) -> bool {
    !url.is_empty()
        && !url
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_whitespace() || byte.is_ascii_control())
}
