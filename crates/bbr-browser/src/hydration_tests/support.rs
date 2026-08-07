//! Shared test fixtures: intent builders, the dense 500-tab fixture, and the
//! recording [`RestoreTarget`] the apply-contract tests execute against.
//!
//! Everything here is test-only and `pub(super)` so the sibling test modules
//! (`planning`, `validation`, `apply`) share one canonical fixture shape
//! instead of duplicating builders.

use bbr_domain::{ProfileHue, SpaceId, TabId};
use bbr_web::{ProfileId, WebViewId};

use crate::harness::seq_ids;
use crate::session::SessionRegistry;
use crate::{
    HydrationError, IntentClosedTab, IntentProfile, IntentSpace, IntentSplit, IntentTab,
    RestoreIntent, RestoreTarget, RestoredClosedTab, RestoredSplit,
};

// ---- Intent builders ------------------------------------------------------

pub(super) fn intent(profiles: Vec<IntentProfile>) -> RestoreIntent {
    RestoreIntent { profiles }
}

pub(super) fn profile(id: u64, spaces: Vec<IntentSpace>) -> IntentProfile {
    IntentProfile {
        id,
        title: format!("profile-{id}"),
        hue: ProfileHue::Cobalt,
        spaces,
    }
}

pub(super) fn space(id: u64, tabs: Vec<IntentTab>) -> IntentSpace {
    IntentSpace {
        id,
        title: format!("space-{id}"),
        tabs,
        active_tab: None,
        recently_closed: Vec::new(),
        split: None,
    }
}

pub(super) fn tab(id: u64, url: &str) -> IntentTab {
    IntentTab {
        id,
        title: format!("tab-{id}"),
        url: url.to_string(),
        pinned: false,
    }
}

pub(super) fn pinned_tab(id: u64, url: &str) -> IntentTab {
    IntentTab {
        pinned: true,
        ..tab(id, url)
    }
}

pub(super) fn closed(id: u64, url: &str, former_position: u64) -> IntentClosedTab {
    IntentClosedTab {
        tab_id: id,
        title: format!("closed-{id}"),
        url: url.to_string(),
        pinned: false,
        former_position,
    }
}

/// A dense intent: 500 tab identities across two profiles (the persistence
/// dense-fixture shape, without the repository). Profile 1 owns spaces 10
/// (tabs 100..350) and 11 (tabs 350..450); profile 2 owns space 20 (tabs
/// 450..600). Every space owns an active tab and a leading pinned prefix;
/// space 11 carries the two-pane split; each space carries one recently-
/// closed entry. The active space is pinned to space 10 — single selected
/// tab 100, **no** split — so the dense proof's bound is unambiguous:
/// exactly one session, 499 deferred, and the split space's panes plus every
/// other space's active tab stay identity-only.
pub(super) fn dense_500() -> RestoreIntent {
    let tabs = |range: std::ops::Range<u64>| -> Vec<IntentTab> {
        range
            .map(|id| tab(id, &format!("https://dense{id}.example")))
            .collect()
    };
    let pin_prefix = |mut tabs: Vec<IntentTab>, pinned: usize| -> Vec<IntentTab> {
        for t in tabs.iter_mut().take(pinned) {
            t.pinned = true;
        }
        tabs
    };
    let mut active = space(10, pin_prefix(tabs(100..350), 3));
    active.active_tab = Some(100);
    active.recently_closed = vec![closed(900, "https://dense900.example", 0)];
    let mut split = space(11, pin_prefix(tabs(350..450), 2));
    split.active_tab = Some(352);
    split.split = Some(IntentSplit {
        primary_tab: 350,
        secondary_tab: 351,
    });
    split.recently_closed = vec![closed(901, "https://dense901.example", 0)];
    let mut second = space(20, pin_prefix(tabs(450..600), 2));
    second.active_tab = Some(450);
    second.recently_closed = vec![closed(902, "https://dense902.example", 0)];
    intent(vec![
        profile(1, vec![active, split]),
        profile(2, vec![second]),
    ])
}

// ---- Recording target -----------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(super) enum IdentityOp {
    Profile {
        id: ProfileId,
        title: String,
        hue: ProfileHue,
    },
    Space {
        id: SpaceId,
        profile: ProfileId,
        title: String,
    },
    Tab {
        id: TabId,
        space: SpaceId,
        title: String,
        url: String,
        pinned: bool,
    },
    ActiveTab {
        space: SpaceId,
        tab: Option<TabId>,
    },
    Split {
        space: SpaceId,
        split: Option<RestoredSplit>,
    },
    RecentlyClosed {
        space: SpaceId,
        entries: Vec<RestoredClosedTab>,
    },
}

/// One recorded [`RestoreTarget`] call: an identity op or a session op, in
/// execution order. The combined log is the chrome-first ordering evidence.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum TargetOp {
    Identity(IdentityOp),
    Session {
        tab: TabId,
        profile: ProfileId,
        url: String,
        view: WebViewId,
    },
}

/// A [`RestoreTarget`] that records every call in one combined log and
/// executes session requests against the fake engine through the same
/// create-then-show shape the controller integration will use.
pub(super) struct RecordingTarget {
    engine: crate::EngineRef,
    registry: SessionRegistry,
    allocate_view: crate::ViewIdAllocator,
    /// The combined identity/session op log, in execution order.
    pub(super) ops: Vec<TargetOp>,
}

impl RecordingTarget {
    pub(super) fn new(engine: crate::EngineRef) -> Self {
        Self {
            engine,
            registry: SessionRegistry::new(),
            allocate_view: seq_ids(),
            ops: Vec::new(),
        }
    }

    pub(super) fn identity_ops(&self) -> usize {
        self.ops
            .iter()
            .filter(|op| matches!(op, TargetOp::Identity(_)))
            .count()
    }

    pub(super) fn session_ops(&self) -> Vec<&TargetOp> {
        self.ops
            .iter()
            .filter(|op| matches!(op, TargetOp::Session { .. }))
            .collect()
    }
}

impl RestoreTarget for RecordingTarget {
    fn restore_profile(
        &mut self,
        id: ProfileId,
        title: &str,
        hue: ProfileHue,
    ) -> Result<(), HydrationError> {
        self.ops.push(TargetOp::Identity(IdentityOp::Profile {
            id,
            title: title.to_string(),
            hue,
        }));
        Ok(())
    }

    fn restore_space(
        &mut self,
        id: SpaceId,
        profile: ProfileId,
        title: &str,
    ) -> Result<(), HydrationError> {
        self.ops.push(TargetOp::Identity(IdentityOp::Space {
            id,
            profile,
            title: title.to_string(),
        }));
        Ok(())
    }

    fn restore_tab(
        &mut self,
        id: TabId,
        space: SpaceId,
        title: &str,
        url: &str,
        pinned: bool,
    ) -> Result<(), HydrationError> {
        self.ops.push(TargetOp::Identity(IdentityOp::Tab {
            id,
            space,
            title: title.to_string(),
            url: url.to_string(),
            pinned,
        }));
        Ok(())
    }

    fn set_active_tab(&mut self, space: SpaceId, tab: Option<TabId>) -> Result<(), HydrationError> {
        self.ops
            .push(TargetOp::Identity(IdentityOp::ActiveTab { space, tab }));
        Ok(())
    }

    fn set_split(
        &mut self,
        space: SpaceId,
        split: Option<RestoredSplit>,
    ) -> Result<(), HydrationError> {
        self.ops
            .push(TargetOp::Identity(IdentityOp::Split { space, split }));
        Ok(())
    }

    fn restore_recently_closed(
        &mut self,
        space: SpaceId,
        entries: &[RestoredClosedTab],
    ) -> Result<(), HydrationError> {
        self.ops
            .push(TargetOp::Identity(IdentityOp::RecentlyClosed {
                space,
                entries: entries.to_vec(),
            }));
        Ok(())
    }

    fn hydrate_session(
        &mut self,
        tab: TabId,
        profile: ProfileId,
        url: &str,
    ) -> Result<(), HydrationError> {
        let view = WebViewId((self.allocate_view)());
        self.engine.borrow_mut().create_web_view(view, profile, url);
        self.engine.borrow_mut().show(view, true);
        self.registry
            .insert(tab, view)
            .expect("freshly allocated view id cannot collide");
        self.ops.push(TargetOp::Session {
            tab,
            profile,
            url: url.to_string(),
            view,
        });
        Ok(())
    }
}
