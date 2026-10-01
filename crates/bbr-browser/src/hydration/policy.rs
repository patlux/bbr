//! Policy entry points and the execution seam.
//!
//! [`HydrationPolicy`] is the stateless, deterministic policy: validate an
//! intent and plan the two-phase restore ([`HydrationPolicy::plan`]), or
//! drive the plan through a [`RestoreTarget`] ([`HydrationPolicy::apply`]).
//! The target-application concern lives here; the public contract types live
//! in the module root and the validation/planning internals in `validate`.

use bbr_domain::{PinnedPage, ProfileHue, SpaceId, TabId};
use bbr_web::ProfileId;

use super::validate::ValidatedGraph;
use super::{HydrationError, RestoreIntent, RestorePlan, RestoredClosedTab, RestoredSplit};

/// The portable chrome-first restore policy: validate an intent, plan the
/// two-phase restore, and optionally drive it through a [`RestoreTarget`].
///
/// The policy is stateless and deterministic — the same intent and active
/// space always produce the same [`RestorePlan`].
#[derive(Debug, Clone, Copy, Default)]
pub struct HydrationPolicy;

impl HydrationPolicy {
    /// The deterministic default active space for a restore: the first space
    /// (in profile order, then space order) that owns an active tab, else the
    /// first space, else `None`. The composition may choose any other space —
    /// this is a policy default, not persisted truth — but a space that owns
    /// an active tab is the sane "restore the window's selected tab" choice.
    pub fn default_active_space(intent: &RestoreIntent) -> Option<SpaceId> {
        let mut first_space = None;
        for profile in &intent.profiles {
            for space in &profile.spaces {
                first_space.get_or_insert(SpaceId(space.id));
                if space.active_tab.is_some() {
                    return Some(SpaceId(space.id));
                }
            }
        }
        first_space
    }

    /// Validate `intent` and produce the deterministic chrome-first restore
    /// plan for `active_space` — the space whose visible tabs the first
    /// usable frame must present.
    pub fn plan(
        intent: &RestoreIntent,
        active_space: SpaceId,
    ) -> Result<RestorePlan, HydrationError> {
        ValidatedGraph::from_intent(intent, active_space).map(ValidatedGraph::plan)
    }

    /// [`HydrationPolicy::plan`] plus execution: restore every identity in
    /// phase-1 order (profiles, then spaces, then tabs, then per-space active
    /// tab, split, and recently-closed entries), then hydrate exactly the
    /// planned sessions. Returns the same plan `plan` would.
    pub fn apply(
        intent: &RestoreIntent,
        active_space: SpaceId,
        target: &mut dyn RestoreTarget,
    ) -> Result<RestorePlan, HydrationError> {
        let plan = Self::plan(intent, active_space)?;
        for profile in &plan.identities {
            target.restore_profile(profile.id, &profile.title, profile.hue)?;
            for space in &profile.spaces {
                target.restore_space(space.id, space.profile, &space.title)?;
                for tab in &space.tabs {
                    target.restore_tab(tab.id, space.id, &tab.title, &tab.url, tab.pinned)?;
                    if tab.pinned_page.is_some() || tab.closed {
                        let page = tab.pinned_page.clone().unwrap_or_else(|| PinnedPage {
                            url: tab.url.clone(),
                            title: tab.title.clone(),
                        });
                        target.restore_kept_place(tab.id, page, tab.closed)?;
                    }
                }
                target.set_active_tab(space.id, space.active_tab)?;
                target.set_split(space.id, space.split)?;
                target.restore_recently_closed(space.id, &space.recently_closed)?;
            }
        }
        for request in &plan.sessions {
            target.hydrate_session(request.tab, request.profile, &request.url)?;
        }
        Ok(plan)
    }
}

/// The seam a chrome-first restore executes against. The serial controller
/// integration implements this over the domain model and the engine once the
/// domain gains an id-preserving import path; contract tests drive a
/// recording target through the real [`HydrationPolicy::apply`] path.
///
/// Implementations must restore identities **before** any
/// [`RestoreTarget::hydrate_session`] call is made — the policy already
/// guarantees that order, and a target must not create sessions on its own.
pub trait RestoreTarget {
    /// Restore one profile identity.
    fn restore_profile(
        &mut self,
        id: ProfileId,
        title: &str,
        hue: ProfileHue,
    ) -> Result<(), HydrationError>;
    /// Restore one space identity, attached to its owning profile.
    fn restore_space(
        &mut self,
        id: SpaceId,
        profile: ProfileId,
        title: &str,
    ) -> Result<(), HydrationError>;
    /// Restore one tab identity into `space` at its intended position
    /// (`pinned` participates in the space's pinned prefix).
    fn restore_tab(
        &mut self,
        id: TabId,
        space: SpaceId,
        title: &str,
        url: &str,
        pinned: bool,
    ) -> Result<(), HydrationError>;
    /// Restore a pinned tab's kept place right after its identity: the page
    /// it was pinned at, and whether it waits closed in place (DESIGN.md
    /// §10.12; never the selection, a split pane or a hydrated session).
    fn restore_kept_place(
        &mut self,
        tab: TabId,
        page: PinnedPage,
        closed: bool,
    ) -> Result<(), HydrationError>;
    /// Set the space's own selected tab (restored on activation).
    fn set_active_tab(&mut self, space: SpaceId, tab: Option<TabId>) -> Result<(), HydrationError>;
    /// Set (or clear) the space's two-pane split.
    fn set_split(
        &mut self,
        space: SpaceId,
        split: Option<RestoredSplit>,
    ) -> Result<(), HydrationError>;
    /// Restore the space's recently-closed entries (identity only).
    fn restore_recently_closed(
        &mut self,
        space: SpaceId,
        entries: &[RestoredClosedTab],
    ) -> Result<(), HydrationError>;
    /// Create and show one session for `tab` in `profile`'s store loading
    /// `url` (the hydrated set is the visible set by definition).
    fn hydrate_session(
        &mut self,
        tab: TabId,
        profile: ProfileId,
        url: &str,
    ) -> Result<(), HydrationError>;
}
