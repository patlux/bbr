//! Controller-level chrome-first restore facade (Phase 7, warm-restore
//! slice): implements the portable [`RestoreTarget`] seam over
//! [`BrowserController`] and owns the id-preserving restore entry point plus
//! the persist-direction model export.
//!
//! ## Restore contract
//!
//! [`BrowserController::restored`] replaces the fresh setup model with the
//! persisted intent **chrome-first**: every identity (profiles → spaces →
//! tabs → active tab → split → recently closed) restores in intent order
//! with its persisted ids (the domain allocator advances past them, so a
//! fresh creation never collides), and **only the active space's visible
//! set** hydrates a session — everything else stays session-less
//! (`Suspended`) until a later activation recreates exactly one session.
//! A dense persisted model (e.g. 500 tab identities) therefore ends with
//! exactly one or two live engine sessions, never 500.
//!
//! ## Non-transactional failure/recovery behavior (the pinned apply contract)
//!
//! [`HydrationPolicy::apply`] is deliberately **non-transactional**: a
//! mid-apply target error leaves every earlier identity op applied and no
//! rollback is attempted by the policy. The facade therefore defines the
//! recovery semantics before startup wiring: `restored` runs against a
//! fresh domain model, and **any** error (validation rejection before the
//! first op, or a mid-apply target failure) rolls the controller back to a
//! fresh default model — every partially created session is destroyed and
//! the default profile/space is recreated — and returns the typed error.
//! The caller (app composition) logs the typed error and starts fresh,
//! mirroring the store's discard-and-recreate disposition: a corrupt or
//! mismatched persisted browser model never presents as a partial restored
//! window, and there is no migration or backward-compatibility path.
//!
//! ## Persist direction
//!
//! [`BrowserController::intent`] exports the live domain as a
//! [`RestoreIntent`] — the same portable shape the policy consumes — so the
//! app maps one model both directions (persist and restore) without
//! `bbr-browser` depending on the persistence crate.

use bbr_domain::{Browser, Tab, TabId};
use bbr_web::ProfileId;

use crate::hydration::{HydrationError, RestoredSplit};
use crate::{BrowserController, HydrationPolicy, RestoreIntent, RestoreTarget};

impl BrowserController {
    /// Restore a persisted browser model chrome-first (see the module docs
    /// for the failure/recovery contract): identities with their persisted
    /// ids first, then exactly the active space's visible-set sessions.
    /// Returns the controller plus the deterministic plan the composition
    /// reports as restore evidence.
    pub fn restored(
        engine: crate::EngineRef,
        allocate_view: crate::ViewIdAllocator,
        intent: &RestoreIntent,
        active_space: bbr_domain::SpaceId,
    ) -> Result<(Self, crate::hydration::RestorePlan), HydrationError> {
        // Fresh setup model, then roll the default profile/space away: the
        // restore import owns the model entirely (fresh creations are the
        // caller's "no persisted model" path).
        let mut controller = Self::new(engine, allocate_view);
        controller.browser = Browser::new();
        controller.active_space = active_space;
        let plan = HydrationPolicy::apply(intent, active_space, &mut controller)
            .inspect_err(|_error| controller.rollback_to_fresh_default())?;
        controller.check_invariants();
        Ok((controller, plan))
    }

    /// The live model as a [`RestoreIntent`] (the persist-direction export):
    /// the app maps this shape onto the persistence browser-state DTO plus
    /// the window/sidebar presentation state it owns. Session state is
    /// deliberately absent — the model is identity truth, and only the
    /// controller/session layer knows which sessions are live.
    pub fn intent(&self) -> RestoreIntent {
        RestoreIntent {
            profiles: self
                .browser
                .profiles()
                .map(|profile| crate::hydration::IntentProfile {
                    id: profile.id.0,
                    title: profile.title.clone(),
                    hue: profile.hue,
                    spaces: self
                        .browser
                        .spaces()
                        .filter(|space| space.profile == profile.id)
                        .map(|space| crate::hydration::IntentSpace {
                            id: space.id.0,
                            title: space.title.clone(),
                            tabs: space
                                .tabs
                                .iter()
                                .filter_map(|&tab| {
                                    let tab = self.browser.tab(tab)?;
                                    Some(crate::hydration::IntentTab {
                                        id: tab.id.0,
                                        title: tab.title.clone(),
                                        url: tab.url.clone(),
                                        pinned: tab.pinned,
                                    })
                                })
                                .collect(),
                            active_tab: space.active_tab.map(|tab| tab.0),
                            recently_closed: space
                                .recently_closed
                                .iter()
                                .map(|closed| crate::hydration::IntentClosedTab {
                                    tab_id: closed.tab.id.0,
                                    title: closed.tab.title.clone(),
                                    url: closed.tab.url.clone(),
                                    pinned: closed.tab.pinned,
                                    former_position: closed.index as u64,
                                })
                                .collect(),
                            // Blueprint D23: split groups persist in the
                            // app's `splits.txt` (a tree the two-pane
                            // table cannot hold).
                            split: None,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    /// The mid-restore rollback (see the module docs): destroy every
    /// partially created session and reset to the fresh default model, so a
    /// failed restore never leaks partial sessions or partial identities.
    fn rollback_to_fresh_default(&mut self) {
        for view in self.registry.views().collect::<Vec<_>>() {
            self.engine.borrow_mut().destroy_web_view(view);
        }
        self.browser = Browser::new();
        self.registry = crate::session::SessionRegistry::new();
        self.session_profiles.clear();
        self.visibility.clear();
        self.nav.clear();
        self.crashed_tabs.clear();
        let profile = self.browser.create_profile("Default");
        let space = self
            .browser
            .create_space(profile, "Default")
            .expect("fresh profile exists");
        self.active_space = space;
    }
}

/// The portable [`RestoreTarget`] seam over the controller. Identity ops
/// restore through the id-preserving domain import; `hydrate_session`
/// activates the tab (the session policy requires Active/Warm tabs to own a
/// session) and creates + shows exactly one session in the tab's profile.
impl RestoreTarget for BrowserController {
    fn restore_profile(
        &mut self,
        id: ProfileId,
        title: &str,
        hue: bbr_domain::ProfileHue,
    ) -> Result<(), HydrationError> {
        self.browser
            .restore_profile(id, hue, title)
            .map_err(target_error)
    }

    fn restore_space(
        &mut self,
        id: bbr_domain::SpaceId,
        profile: ProfileId,
        title: &str,
    ) -> Result<(), HydrationError> {
        self.browser
            .restore_space(id, profile, title)
            .map_err(target_error)
    }

    fn restore_tab(
        &mut self,
        id: TabId,
        space: bbr_domain::SpaceId,
        title: &str,
        url: &str,
        pinned: bool,
    ) -> Result<(), HydrationError> {
        self.browser
            .restore_tab(id, space, title, url, pinned)
            .map_err(target_error)
    }

    fn set_active_tab(
        &mut self,
        space: bbr_domain::SpaceId,
        tab: Option<TabId>,
    ) -> Result<(), HydrationError> {
        self.browser
            .restore_active_tab(space, tab)
            .map_err(target_error)
    }

    fn set_split(
        &mut self,
        space: bbr_domain::SpaceId,
        split: Option<RestoredSplit>,
    ) -> Result<(), HydrationError> {
        let split = split.map(|split| bbr_domain::Split {
            primary: split.primary_tab,
            secondary: split.secondary_tab,
        });
        self.browser
            .restore_split(space, split)
            .map_err(target_error)
    }

    fn restore_recently_closed(
        &mut self,
        space: bbr_domain::SpaceId,
        entries: &[crate::hydration::RestoredClosedTab],
    ) -> Result<(), HydrationError> {
        for entry in entries {
            let mut tab = Tab::new(entry.tab_id, space, &entry.title, &entry.url);
            tab.pinned = entry.pinned;
            self.browser
                .restore_recently_closed(space, tab, entry.former_position as usize)
                .map_err(target_error)?;
        }
        Ok(())
    }

    fn hydrate_session(
        &mut self,
        tab: TabId,
        profile: ProfileId,
        url: &str,
    ) -> Result<(), HydrationError> {
        self.browser
            .restore_activate_tab(tab)
            .map_err(target_error)?;
        let view = self.create_session(tab, profile, url);
        // The hydrated set is the visible set by definition: show it now
        // (the domain's own visibility derives from the same split/active
        // precedence the policy planned from).
        self.engine.borrow_mut().show(view, true);
        self.visibility.insert(view, true);
        Ok(())
    }
}

/// Flatten a domain error into the typed [`HydrationError::Target`] the
/// policy's apply contract pins (the facade surfaces the string; the
/// composition logs it).
fn target_error(error: bbr_domain::DomainError) -> HydrationError {
    HydrationError::Target(error.to_string())
}
