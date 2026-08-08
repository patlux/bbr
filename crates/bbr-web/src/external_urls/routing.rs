//! Routing decision: where an accepted external URL opens.
//!
//! ## The rule (conventional macOS browser behavior)
//!
//! An external link opens in the **frontmost eligible window**: a fresh tab
//! in that window's space, in the window's own profile. Only when no
//! eligible window exists does the browser create a **new window**, in the
//! caller-declared default profile and its default space.
//!
//! This is the established macOS convention: Safari ("Open links from
//! applications in a new tab instead of a new window" is the default),
//! Chrome, and Firefox all target the window the user is currently looking
//! at for OS-dispatched links — reusing it with a new tab avoids a window
//! explosion — and fall back to a new window only when none can be reused.
//! The contract fixes that behavior explicitly so every platform adapter
//! behaves identically.
//!
//! ## Profile and space selection
//!
//! - **Reuse** takes the reused window's own profile and space. The decision
//!   can never express a cross-profile move: an external URL never silently
//!   crosses a profile boundary.
//! - **New window** takes the caller-declared `default_profile` /
//!   `default_space`. The contract refuses to guess a profile or space.
//!
//! ## Window eligibility
//!
//! A window is eligible for reuse only when `accepts_external_urls` is true.
//! Private/incognito windows, utility panels, and devtools windows are not
//! eligible: an external link must not land in a session the user did not
//! intend for ordinary browsing. The adapter decides the flag per window.
//!
//! ## Determinism and tab semantics
//!
//! The caller orders `windows` front-to-back; the rule picks the **first**
//! eligible window, so the outcome is deterministic without any wall clock.
//! A reuse always creates a **new tab** — it never navigates an existing tab
//! and never clobbers the user's current page; a new window starts with one
//! tab.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::external_urls::{SpaceId, WindowId};
use crate::ProfileId;

/// One open window the routing rule may reuse.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub window: WindowId,
    /// The profile this window belongs to; a reused tab stays in it.
    pub profile: ProfileId,
    /// The space this window currently presents; a reused tab joins exactly
    /// this space.
    pub space: SpaceId,
    /// Whether the window may receive an external URL open. See the module
    /// docs for eligibility.
    pub accepts_external_urls: bool,
}

/// Call-time routing state. Deliberately non-serde: it borrows the caller's
/// window list and exists only for the duration of one decision.
#[derive(Debug)]
pub struct RoutingInput<'a> {
    /// Open windows in front-to-back order (first = frontmost).
    pub windows: &'a [WindowSnapshot],
    /// Profile for a new window when no eligible window exists.
    pub default_profile: ProfileId,
    /// Default space of `default_profile` for a new window.
    pub default_space: SpaceId,
}

/// Where an accepted external URL opens.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    /// A fresh tab in the frontmost eligible window: the tab belongs to
    /// `space` in `profile` — the window's own, never any other.
    ReuseWindow {
        window: WindowId,
        profile: ProfileId,
        space: SpaceId,
    },
    /// No eligible window exists: create a new window in the caller-declared
    /// default profile; its first tab lives in the caller-declared default
    /// space.
    NewWindow { profile: ProfileId, space: SpaceId },
}

/// The deterministic routing rule; see the module docs.
///
/// Picks the first window in the caller's front-to-back order whose
/// `accepts_external_urls` flag is set (the frontmost eligible window), else
/// a new window in the caller-declared default profile and space.
pub fn decide_routing(input: &RoutingInput<'_>) -> RoutingDecision {
    match input
        .windows
        .iter()
        .find(|window| window.accepts_external_urls)
    {
        Some(window) => RoutingDecision::ReuseWindow {
            window: window.window,
            profile: window.profile,
            space: window.space,
        },
        None => RoutingDecision::NewWindow {
            profile: input.default_profile,
            space: input.default_space,
        },
    }
}
