//! Portable chrome-first restore/hydration policy.
//!
//! Phase 7 must restore a persisted browser model at startup **without**
//! eagerly creating every stored web session ("Startup does not instantiate
//! every persisted WKWebView"). This module owns the portable decision core of
//! that restore: given a persisted-state [`RestoreIntent`] — the structural
//! shape the `bbr-persistence` repository restores — it validates the intent
//! and then plans a deterministic two-phase restore:
//!
//! 1. **Identities first** ([`RestorePlan::identities`]): the complete
//!    profile/space/tab graph with pinned state, per-space active tabs,
//!    splits, and recently-closed entries, normalized into domain-typed ids.
//! 2. **Sessions only for the visible set** ([`RestorePlan::sessions`]):
//!    exactly the sessions the first usable frame needs — the *active* space's
//!    selected tab (single pane) or both split panes. Every other tab is
//!    deferred ([`RestorePlan::deferred`]) and owns no session.
//!
//! The two phases are structurally separate, so a restore can never interleave
//! session creation with identity restoration and can never create a session
//! for a tab outside the active space's visible set. A dense persisted model
//! (e.g. 500 tab identities) therefore plans exactly one or two sessions,
//! never 500.
//!
//! ## Why the intent is a local shape
//!
//! `bbr-browser` deliberately depends only on `bbr-domain` and `bbr-web` (no
//! SQLite, no storage). It cannot type-reference `bbr-persistence` without a
//! manifest change, which is a serial integration surface. [`RestoreIntent`]
//! therefore mirrors the structural part of the `bbr-persistence` snapshot
//! contract (ids are raw persisted `u64`s; hues reuse the domain
//! [`ProfileHue`]); the serial app composition maps the restored repository
//! snapshot into it mechanically. Window geometry/sidebar state are optional
//! presentation state recovered by the app and are deliberately not part of
//! the intent.
//!
//! ## Executing the plan
//!
//! [`HydrationPolicy::apply`] drives the plan through a [`RestoreTarget`]:
//! every identity operation (profile → spaces → tabs → active tab → split →
//! recently closed) in deterministic intent order, then one
//! [`RestoreTarget::hydrate_session`] per [`SessionRequest`] (create *and*
//! show — the hydrated set is by definition the visible set). The serial
//! controller integration implements [`RestoreTarget`] over the domain model
//! and engine once the domain gains an id-preserving import path; contract
//! tests prove the policy against a recording target backed by the fake
//! engine.
//!
//! ## Validation scope
//!
//! `plan` validates what *planning* depends on and rejects it with a typed
//! [`HydrationError`] before producing any plan: the active space exists;
//! profile/space/tab ids are globally unique (the persistence and domain
//! identity contract); each space has a pinned prefix; the active tab and
//! split panes are members; recently-closed ids do not collide with live
//! identities or each other; and every URL satisfies the bounded URL
//! contract (non-empty, no ASCII whitespace/control/NUL — the same rule
//! `bbr-persistence` enforces). Storage-range and SQL-level validation stays
//! upstream in the repository; this layer assumes a post-restore intent.
//!
//! ## Layout
//!
//! This file owns the public contract types and [`HydrationError`]. The
//! policy entry points and the [`RestoreTarget`] seam live in the `policy`
//! submodule; the validation/planning internals (`ValidatedGraph`) live in
//! the `validate` submodule.

mod policy;
mod validate;

pub use policy::{HydrationPolicy, RestoreTarget};

use bbr_domain::{PinnedPage, ProfileHue, SpaceId, TabId};
use bbr_web::ProfileId;

/// A persisted structural browser state, shaped like the `bbr-persistence`
/// snapshot contract. Profile/space/tab ids are the raw persisted values;
/// the plan normalizes them into typed ids.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RestoreIntent {
    /// Profiles in restore order.
    pub profiles: Vec<IntentProfile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentProfile {
    pub id: u64,
    pub title: String,
    /// The DESIGN.md §12 identity hue, persisted with the profile.
    pub hue: ProfileHue,
    /// Spaces in restore order.
    pub spaces: Vec<IntentSpace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentSpace {
    pub id: u64,
    pub title: String,
    /// Tabs in order; the leading `pinned` tabs form the pinned prefix.
    pub tabs: Vec<IntentTab>,
    /// The space's own selected tab, restored on activation.
    pub active_tab: Option<u64>,
    /// Recently-closed entries, most recent last.
    pub recently_closed: Vec<IntentClosedTab>,
    /// The two-pane split of this space, if any.
    pub split: Option<IntentSplit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentTab {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    /// The page a pinned tab was pinned at (DESIGN.md §11.2); `None` for a
    /// regular tab, or a pinned tab whose URL is its pinned page.
    pub pinned_page: Option<PinnedPage>,
    /// A pinned tab closed in place (DESIGN.md §10.12): it is never the
    /// selection or a split pane and restores without a session.
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentClosedTab {
    pub tab_id: u64,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    /// Former index in the space's tab list, used to restore position on
    /// reopen. Kept as the persisted `u64`; the domain integration converts
    /// it to the space-local `usize` it needs.
    pub former_position: u64,
    /// The record of a pinned row closed in place: `tab_id` names that
    /// closed row of the same space, and `url`/`title` the page it showed.
    pub in_place: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentSplit {
    pub primary_tab: u64,
    pub secondary_tab: u64,
}

/// The deterministic chrome-first restore plan for an intent.
///
/// The phase separation is the policy: [`identities`](Self::identities) holds
/// the full browser identity graph and [`sessions`](Self::sessions) holds
/// exactly the visible-set session requests, so session creation can never
/// precede or exceed identity restoration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePlan {
    /// Phase 1 — the full restored identity graph, in intent order.
    pub identities: Vec<RestoredProfile>,
    /// Phase 2 — exactly the sessions the first usable frame needs (the
    /// active space's selected tab, or both split panes).
    pub sessions: Vec<SessionRequest>,
    /// Every identity that must be restored **without** a session, in intent
    /// order (all tabs minus the hydrated visible set). The integration
    /// restores these as session-less state (e.g. the domain's `Suspended`
    /// lifecycle) so a later activation recreates exactly one session.
    pub deferred: Vec<TabId>,
}

/// A restored profile identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredProfile {
    pub id: ProfileId,
    pub title: String,
    pub hue: ProfileHue,
    /// Spaces in restore order.
    pub spaces: Vec<RestoredSpace>,
}

/// A restored space identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredSpace {
    pub id: SpaceId,
    /// The owning profile (derived from the intent's nesting).
    pub profile: ProfileId,
    pub title: String,
    /// Tabs in order; leading `pinned` tabs form the pinned prefix.
    pub tabs: Vec<RestoredTab>,
    pub active_tab: Option<TabId>,
    pub recently_closed: Vec<RestoredClosedTab>,
    pub split: Option<RestoredSplit>,
}

/// A restored tab identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredTab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub pinned_page: Option<PinnedPage>,
    pub closed: bool,
}

/// A restored recently-closed entry (identity only — it owns no session).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredClosedTab {
    pub tab_id: TabId,
    pub title: String,
    pub url: String,
    pub pinned: bool,
    pub former_position: u64,
    pub in_place: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestoredSplit {
    pub primary_tab: TabId,
    pub secondary_tab: TabId,
}

/// One Phase-2 session request: create a session for `tab` in `profile`'s
/// data store loading `url`, and show it (the hydrated set is the visible
/// set by definition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRequest {
    pub tab: TabId,
    pub profile: ProfileId,
    pub url: String,
}

/// Errors from planning or applying a chrome-first restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydrationError {
    /// The named active space does not exist in the intent.
    UnknownActiveSpace(SpaceId),
    /// A profile/space/tab identity appears more than once (the persistence
    /// and domain identity contract is global uniqueness across kinds).
    DuplicateIdentity(u64),
    /// A space's tabs do not form a pinned prefix (all pinned first).
    InvalidPinnedRegion { space_id: u64 },
    /// The space's active tab is not one of its tabs.
    InvalidActiveTab { space_id: u64, tab_id: u64 },
    /// The split panes are not distinct members of the space.
    InvalidSplit { space_id: u64 },
    /// A recently-closed tab id collides with a live identity or another
    /// recently-closed id.
    RecentlyClosedCollision { tab_id: u64 },
    /// A pinned page or closed state on a regular tab, or an in-place
    /// record that names no closed pinned row of its space (DESIGN.md
    /// §10.12).
    InvalidKeptPlace { tab_id: u64 },
    /// A tab URL violates the bounded URL contract.
    InvalidUrl { tab_id: u64 },
    /// The restore target rejected an operation.
    Target(String),
}

impl std::fmt::Display for HydrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownActiveSpace(space) => {
                write!(f, "active space {space:?} is not in the restore intent")
            }
            Self::DuplicateIdentity(id) => write!(f, "duplicate identity {id}"),
            Self::InvalidPinnedRegion { space_id } => {
                write!(f, "space {space_id} does not have a pinned prefix")
            }
            Self::InvalidActiveTab { space_id, tab_id } => {
                write!(f, "active tab {tab_id} is not in space {space_id}")
            }
            Self::InvalidSplit { space_id } => {
                write!(f, "invalid split in space {space_id}")
            }
            Self::RecentlyClosedCollision { tab_id } => {
                write!(
                    f,
                    "recently closed identity {tab_id} collides with live state"
                )
            }
            Self::InvalidKeptPlace { tab_id } => {
                write!(f, "tab {tab_id} is not a closed pinned place")
            }
            Self::InvalidUrl { tab_id } => write!(f, "invalid URL for tab {tab_id}"),
            Self::Target(message) => write!(f, "restore target rejected operation: {message}"),
        }
    }
}

impl std::error::Error for HydrationError {}
