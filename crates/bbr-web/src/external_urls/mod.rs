//! # External URL opens — portable request/classification/routing contract
//!
//! Portable vocabulary for OS-dispatched external URL opens (BBR as the
//! default handler for `http`/`https`), owned by
//! `docs/roadmap/tasks/external-url-open-contract.md`. Pure `std`: no
//! frameworks, GPUI, `unsafe`, file I/O, network, or new dependencies.
//! AppKit registration (`CFBundleURLTypes`,
//! `LSSetDefaultHandlerForURLScheme`), `Info.plist`, the `NSApplication`
//! open-URLs delegate, and the adapter that feeds OS events into this
//! contract are explicit later native work.
//!
//! ## Pipeline
//!
//! One incoming open goes through three deterministic steps, in order:
//!
//! 1. [`classify`] turns the raw URL string into a typed [`UrlClass`]:
//!    supported web URL, dangerous scheme, unsupported scheme, or malformed.
//!    The exact scheme rules are documented on [`classify`].
//! 2. [`BurstTracker`] suppresses repeat routes of the same URL within a
//!    caller-supplied monotonic-tick window (the OS can deliver duplicates
//!    in a burst).
//! 3. [`decide_routing`] applies the conventional macOS behavior: reuse the
//!    frontmost eligible window when one exists, otherwise open a new window
//!    in the caller-declared default profile/space.
//!
//! [`ExternalUrlCoordinator`] composes all three behind one
//! [`handle`](ExternalUrlCoordinator::handle) call with explicit reject
//! reasons and no panics.
//!
//! ## Invariants
//!
//! - **Profile isolation.** An external open routes into the profile of an
//!   existing eligible window (the window the user is currently using) or
//!   into the caller-declared default profile for a new window — never any
//!   other profile. The decision type cannot express a cross-profile move.
//! - **No rewriting.** The URL is never normalized, scheme-defaulted, or
//!   otherwise "cleaned"; anything that cannot be routed as given is
//!   rejected rather than guessed. The navigated URL is the request's own
//!   trimmed string, never reconstructed from parts.
//! - **Dangerous schemes are never opened.** `javascript:`, `data:`,
//!   `file:`, `about:`, and `vbscript:` are classified explicitly and
//!   rejected.
//! - **Privacy.** Diagnostics and retention never contain full URLs with
//!   credentials or query strings. Authority-bearing URLs are reduced to
//!   `scheme://host[:port]` (lower-cased, userinfo stripped) — the same
//!   reduction [`crate::web_errors::identity`] establishes — and the burst
//!   tracker retains only that reduced origin plus a non-cryptographic hash
//!   of the untransformed remainder.
//! - **Determinism.** No wall clock anywhere. Burst expiry is expressed in
//!   caller-supplied monotonic ticks; window order is caller-supplied
//!   front-to-back.
//!
//! ## Out of scope (native slice)
//!
//! URL-scheme registration, default-handler claims, `Info.plist`, the
//! `NSApplication` open-URLs delegate, and the adapter that maps OS events
//! into [`ExternalUrlRequest`]s are deferred to the follow-up native slice
//! named in the owning packet.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod classify;
pub mod coordinator;
pub mod dedup;
pub mod routing;

pub use classify::{
    classify, MalformedUrl, ReducedUrl, RejectionReason, UrlClass, WebScheme, WebUrl,
};
pub use coordinator::{ExternalUrlCoordinator, ExternalUrlError, ExternalUrlOutcome};
pub use dedup::{BurstConfig, BurstConfigError, BurstTracker, DedupVerdict};
pub use routing::{decide_routing, RoutingDecision, RoutingInput, WindowSnapshot};

/// Stable identity of one incoming external-URL open, assigned by the
/// adapter. Never reused for the lifetime of an [`ExternalUrlCoordinator`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalUrlRequestId(pub u64);

/// Activation intent an OS open may carry, when the platform supplies one.
///
/// Informational for the adapter's focus handling only: it never changes the
/// routing rule (where the URL opens). A `Background` open must not steal
/// focus; a `Foreground` open may bring the target window forward.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationHint {
    Background,
    Foreground,
}

/// One OS-dispatched external URL open.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalUrlRequest {
    pub id: ExternalUrlRequestId,
    /// The URL exactly as the operating system delivered it. The contract
    /// never normalizes or rewrites it; the navigated URL is this same
    /// (trimmed) string.
    pub raw_url: String,
    /// Activation intent, when the platform supplies one; `None` otherwise.
    pub activation: Option<ActivationHint>,
}

/// Opaque identity of one app window in the routing vocabulary.
///
/// The adapter maps this to a real native window (and to the space the
/// window presents); bbr-web keeps it opaque so no platform pointer crosses
/// the seam — the same pattern as [`crate::WebViewId`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

/// Opaque identity of one space (workspace) in the routing vocabulary.
///
/// Deliberately distinct from the persistent `bbr-domain::SpaceId`: this
/// crate is the seam `bbr-domain` depends on, so it cannot name the domain's
/// type; the adapter converts between the two at the boundary. A tab always
/// belongs to exactly one space, and the contract never invents or moves
/// one.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpaceId(pub u64);

#[cfg(test)]
mod tests_classify;
#[cfg(test)]
mod tests_coordinator;
#[cfg(test)]
mod tests_dedup;
#[cfg(test)]
mod tests_routing;
#[cfg(all(test, feature = "serde"))]
mod tests_serde;
