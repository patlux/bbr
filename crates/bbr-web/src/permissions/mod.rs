//! # Portable permission authority — identities, taxonomy, validation
//!
//! The portable, platform-neutral contract for Phase 8's permission surface:
//! camera, microphone, location, notifications, and clipboard. This first
//! implementation step fixes the **stable identities and taxonomy** that the
//! later request/decision lifecycle (pending → resolved/cancelled/superseded)
//! and the persistence boundary build on. No WebKit, no AppKit, no GPUI, no
//! storage: only `std` plus the crate's optional `serde` feature.
//!
//! The bounded Phase-8 implementation is owned by
//! `docs/roadmap/tasks/permissions-contract.md`.
//!
//! ## Step-1 contract
//!
//! - **Request identity:** [`PermissionRequestId`] is stable, ordered, and
//!   hashable; the later lifecycle correlates completions by it.
//! - **Closed capability taxonomy:** [`PermissionCapability`] is exactly the
//!   five capabilities named above ([`PermissionCapability::ALL`]) — no
//!   speculative kinds.
//! - **Origin identity:** [`PermissionOrigin`] is a validated, normalized
//!   `scheme://host[:port]` triple (see the `origin` module). Empty or
//!   cross-origin-conflating origins are unrepresentable.
//! - **Scope identity:** [`PermissionScope`] pairs a validated origin with a
//!   capability.
//! - **Request context:** [`PermissionRequest`] carries the stable id, the
//!   owning profile ([`crate::ProfileId`]), the scope, and the user-gesture
//!   flag reported by the engine.
//! - **Decisions:** [`PermissionDecision`] (allow/deny),
//!   [`PermissionPersistence`] (one-time vs profile-persisted), and
//!   [`PermissionVerdict`] joining them.
//! - **Durable policy DTOs:** [`PermissionPolicyKey`] (profile + scope),
//!   [`PermissionPolicyValue`] (the stored allow/deny), and
//!   [`PermissionPolicyEntry`] pairing them.
//!
//! ## Step-1 invariants (unit-proven by `tests.rs`)
//!
//! 1. **Closed taxonomy** — `PermissionCapability::ALL` has exactly five
//!    distinct members.
//! 2. **No empty or cross-origin policy keys** — every construction path
//!    validates: `parse`/`try_new` reject empty input, a missing scheme,
//!    empty hosts, userinfo/whitespace, and malformed ports, and the `serde`
//!    deserializer re-runs the same validation at the wire boundary.
//! 3. **Profile/origin/capability key identity** — two policy keys are equal
//!    iff profile, origin, and capability all match; keys are usable as
//!    ordered and hashed collection keys.
//! 4. **One-time decisions never persist** — only `ProfilePersisted`
//!    verdicts convert into a [`PermissionPolicyValue`] via
//!    [`PermissionVerdict::into_policy_value`].
//!
//! ## Step-2 contract
//!
//! - **Typed lifecycle:** [`PermissionLifecycle`] tracks pending requests by
//!   stable id, resolves them from the durable policy view or an explicit
//!   completion, and supports cancellation and supersession — all with typed
//!   [`PermissionTransition`]s that reject stale, duplicate, or mismatched
//!   completions whole.
//! - **Caller-owned persistence:** durable verdicts become
//!   [`PermissionPersistenceCommand`]s the caller applies to a
//!   [`PermissionPolicyHook`]; one-time verdicts can never produce a command.

use crate::ProfileId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod lifecycle;
mod origin;

pub use lifecycle::{
    PermissionCompletion, PermissionLifecycle, PermissionPersistenceCommand, PermissionPolicyHook,
    PermissionRejection, PermissionRequestOutcome, PermissionResolution, PermissionTransition,
};
pub use origin::{OriginError, PermissionOrigin};

/// Stable identity of one permission request.
///
/// Identities are allocated by the permission authority and are never reused;
/// a completion reports the id back and a mismatched or stale id rejects the
/// whole completion. The tuple field is private so ids cannot be fabricated
/// outside the authority.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionRequestId(u64);

impl PermissionRequestId {
    /// Allocated by the request lifecycle via a monotonic counter; ids are
    /// never reused and cannot be fabricated outside this module.
    fn new(value: u64) -> Self {
        Self(value)
    }

    /// The underlying value, for logging and wire correlation.
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Closed taxonomy of the capabilities BBR exposes to web content.
///
/// This is exactly the five named capabilities; no speculative kinds are
/// added. `ALL` documents and proves the closed set.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionCapability {
    Camera,
    Microphone,
    Location,
    Notifications,
    Clipboard,
}

impl PermissionCapability {
    /// The closed taxonomy: exactly the five capabilities.
    pub const ALL: [Self; 5] = [
        Self::Camera,
        Self::Microphone,
        Self::Location,
        Self::Notifications,
        Self::Clipboard,
    ];
}

/// A validated origin + capability pair.
///
/// The scope is the smallest unit a permission decision applies to. Because
/// the origin is validated at construction, an invalid scope is
/// unrepresentable.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionScope {
    origin: PermissionOrigin,
    capability: PermissionCapability,
}

impl PermissionScope {
    /// Pair a validated origin with a capability.
    pub fn new(origin: PermissionOrigin, capability: PermissionCapability) -> Self {
        Self { origin, capability }
    }

    /// The validated origin this scope applies to.
    pub fn origin(&self) -> &PermissionOrigin {
        &self.origin
    }

    /// The capability this scope applies to.
    pub fn capability(&self) -> PermissionCapability {
        self.capability
    }
}

/// A permission request reported by the engine to the authority.
///
/// The request is valid by construction: id and profile come as stable
/// identity types and the scope carries an already-validated origin, so no
/// invalid or empty request can exist at this seam.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    /// Stable identity the later lifecycle correlates completions by.
    pub id: PermissionRequestId,
    /// The profile that owns the requesting web session. Mandatory on every
    /// request; a decision can never move across profiles.
    pub profile: ProfileId,
    /// The origin + capability being requested.
    pub scope: PermissionScope,
    /// Whether the request was preceded by a user activation (click or
    /// keyboard gesture), as reported by the engine.
    pub user_gesture: bool,
}

impl PermissionRequest {
    /// Build a request from validated parts.
    pub fn new(
        id: PermissionRequestId,
        profile: ProfileId,
        origin: PermissionOrigin,
        capability: PermissionCapability,
        user_gesture: bool,
    ) -> Self {
        Self {
            id,
            profile,
            scope: PermissionScope::new(origin, capability),
            user_gesture,
        }
    }
}

/// The binary outcome of a permission decision.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionDecision {
    /// Grant the capability to the requesting origin.
    Allow,
    /// Refuse the capability to the requesting origin.
    Deny,
}

/// How long a decision applies.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionPersistence {
    /// Applies to the current request only; never stored.
    OneTime,
    /// Applies until changed or cleared for the profile; eligible for the
    /// durable policy store.
    ProfilePersisted,
}

/// A decision plus its persistence scope.
///
/// This is the unit produced by the future decision authority. Only
/// `ProfilePersisted` verdicts become durable policy records:
/// [`PermissionVerdict::into_policy_value`] returns `None` for every
/// one-time verdict, so one-time decisions can never reach a persistence
/// hook.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionVerdict {
    /// Whether the capability is granted.
    pub decision: PermissionDecision,
    /// Whether the decision is one-time or profile-persisted.
    pub persistence: PermissionPersistence,
}

impl PermissionVerdict {
    /// Build a verdict from explicit parts.
    pub const fn new(decision: PermissionDecision, persistence: PermissionPersistence) -> Self {
        Self {
            decision,
            persistence,
        }
    }

    /// Allow for the current request only.
    pub const fn allow_one_time() -> Self {
        Self::new(PermissionDecision::Allow, PermissionPersistence::OneTime)
    }

    /// Deny for the current request only.
    pub const fn deny_one_time() -> Self {
        Self::new(PermissionDecision::Deny, PermissionPersistence::OneTime)
    }

    /// Allow and remember for the profile.
    pub const fn allow_persisted() -> Self {
        Self::new(
            PermissionDecision::Allow,
            PermissionPersistence::ProfilePersisted,
        )
    }

    /// Deny and remember for the profile.
    pub const fn deny_persisted() -> Self {
        Self::new(
            PermissionDecision::Deny,
            PermissionPersistence::ProfilePersisted,
        )
    }

    /// Whether this verdict is eligible for the durable policy store.
    pub fn is_persistent(&self) -> bool {
        self.persistence == PermissionPersistence::ProfilePersisted
    }

    /// The durable value this verdict contributes to the policy store, or
    /// `None` when the decision must stay one-time.
    pub fn into_policy_value(self) -> Option<PermissionPolicyValue> {
        if self.is_persistent() {
            Some(PermissionPolicyValue::new(self.decision))
        } else {
            None
        }
    }
}

/// Durable policy key: `ProfileId + origin + capability`.
///
/// Equality is exactly profile + scope (origin + capability), so a persisted
/// decision can never leak across profiles, origins, or capabilities.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionPolicyKey {
    profile: ProfileId,
    scope: PermissionScope,
}

impl PermissionPolicyKey {
    /// Build a durable policy key.
    pub fn new(profile: ProfileId, scope: PermissionScope) -> Self {
        Self { profile, scope }
    }

    /// The profile this policy record belongs to.
    pub fn profile(&self) -> ProfileId {
        self.profile
    }

    /// The origin + capability this policy record applies to.
    pub fn scope(&self) -> &PermissionScope {
        &self.scope
    }
}

/// The stored outcome of a durable decision.
///
/// Only allow/deny is stored; the one-time/persisted distinction is decided
/// by [`PermissionVerdict::into_policy_value`] before a value is ever
/// created, so a stored value never represents a one-time decision.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionPolicyValue {
    decision: PermissionDecision,
}

impl PermissionPolicyValue {
    /// Wrap a stored allow/deny decision.
    pub fn new(decision: PermissionDecision) -> Self {
        Self { decision }
    }

    /// The stored decision.
    pub fn decision(&self) -> PermissionDecision {
        self.decision
    }
}

/// A durable policy record: key + value.
///
/// This is the unit a persistence hook stores and removes; it never carries a
/// one-time decision.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPolicyEntry {
    key: PermissionPolicyKey,
    value: PermissionPolicyValue,
}

impl PermissionPolicyEntry {
    /// Pair a durable key with its stored value.
    pub fn new(key: PermissionPolicyKey, value: PermissionPolicyValue) -> Self {
        Self { key, value }
    }

    /// The durable key.
    pub fn key(&self) -> &PermissionPolicyKey {
        &self.key
    }

    /// The stored value.
    pub fn value(&self) -> &PermissionPolicyValue {
        &self.value
    }

    /// Split the record back into key and value.
    pub fn into_parts(self) -> (PermissionPolicyKey, PermissionPolicyValue) {
        (self.key, self.value)
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "serde"))]
mod tests_serde;
