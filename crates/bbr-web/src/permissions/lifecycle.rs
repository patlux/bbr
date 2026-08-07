//! Pure request lifecycle and caller-owned persistence boundary (step 2 of
//! the permissions contract).
//!
//! [`PermissionLifecycle`] owns the deterministic state machine that turns a
//! reported request into a typed transition. Pending requests are correlated
//! by stable id; a durable policy hit resolves a request without prompting;
//! explicit resolutions, cancellations, and supersessions move a pending
//! request to a terminal state. Completions are validated whole: a stale id,
//! a duplicate completion, or a completion whose profile or scope does not
//! match the pending request rejects the transition without changing state.
//!
//! Persistence is caller-owned. The lifecycle keeps the in-memory policy view
//! that drives the durable-policy-hit path and hands back
//! [`PermissionPersistenceCommand`]s for profile-persisted verdicts; the
//! caller applies them to a [`PermissionPolicyHook`] backed by real storage.
//! A one-time verdict can never produce a command —
//! [`PermissionPersistenceCommand::store`] returns `None` for it — so no
//! storage implementation can persist a one-time decision through this seam.

use std::collections::{BTreeMap, BTreeSet};

use crate::ProfileId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{
    PermissionDecision, PermissionPolicyEntry, PermissionPolicyKey, PermissionPolicyValue,
    PermissionRequest, PermissionRequestId, PermissionScope, PermissionVerdict,
};

/// The deterministic permission request authority.
///
/// Holds exactly the state needed to correlate completions and answer
/// durable-policy hits: pending requests keyed by stable id, the set of
/// terminal ids for duplicate/stale rejection, a monotonic id allocator, and
/// the in-memory view of the durable policy. The durable store itself stays
/// caller-owned; this type only mirrors it via
/// [`set_policy`](Self::set_policy) / [`remove_policy`](Self::remove_policy)
/// and reports [`PermissionPersistenceCommand`]s to apply.
#[derive(Debug, Default)]
pub struct PermissionLifecycle {
    pending: BTreeMap<PermissionRequestId, PermissionRequest>,
    terminal: BTreeSet<PermissionRequestId>,
    next_id: u64,
    policy: BTreeMap<PermissionPolicyKey, PermissionPolicyValue>,
}

impl PermissionLifecycle {
    /// A new empty authority with no pending requests and no policy view.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a request for `scope` in `profile`.
    ///
    /// A durable policy hit resolves the request immediately and records it
    /// as terminal; otherwise the request becomes pending, superseding any
    /// still-pending request for the same profile and scope.
    pub fn request(
        &mut self,
        profile: ProfileId,
        scope: PermissionScope,
        user_gesture: bool,
    ) -> PermissionRequestOutcome {
        let superseded = self.supersede_same_scope(profile, &scope);
        let decision = self
            .policy
            .get(&PermissionPolicyKey::new(profile, scope.clone()))
            .map(|value| value.decision());
        let request = self.allocate(profile, scope, user_gesture);
        match decision {
            Some(decision) => {
                self.terminal.insert(request.id);
                PermissionRequestOutcome::ResolvedFromPolicy { request, decision }
            }
            None => {
                self.pending.insert(request.id, request.clone());
                PermissionRequestOutcome::Pending {
                    request,
                    superseded,
                }
            }
        }
    }

    /// Complete a pending request with a verdict.
    ///
    /// The whole completion is validated before any state changes: an id
    /// that is not pending rejects as stale or duplicate, and a completion
    /// whose profile or scope does not match the pending request rejects as
    /// mismatched. A profile-persisted verdict updates the policy view and
    /// returns a [`PermissionPersistenceCommand::Store`] for the caller's
    /// hook; one-time verdicts never produce a command.
    pub fn resolve(&mut self, completion: PermissionCompletion) -> PermissionTransition {
        let Some(request) = self.pending.get(&completion.id) else {
            return self.rejection_for(completion.id);
        };
        if request.profile != completion.profile || request.scope != completion.scope {
            return PermissionTransition::Rejected(PermissionRejection::MismatchedContext);
        }
        let request = self
            .pending
            .remove(&completion.id)
            .expect("pending entry verified above");
        self.terminal.insert(request.id);
        let command = completion.verdict.into_policy_value().map(|value| {
            let key = PermissionPolicyKey::new(completion.profile, completion.scope.clone());
            self.policy.insert(key.clone(), value);
            PermissionPersistenceCommand::Store(PermissionPolicyEntry::new(key, value))
        });
        PermissionTransition::Resolved(PermissionResolution {
            decision: completion.verdict.decision,
            command,
        })
    }

    /// Cancel a pending request; it can never become an allow.
    pub fn cancel(&mut self, id: PermissionRequestId) -> PermissionTransition {
        self.drop_pending(id, PermissionTransition::Cancelled)
    }

    /// Supersede a pending request; it can never become an allow.
    pub fn supersede(&mut self, id: PermissionRequestId) -> PermissionTransition {
        self.drop_pending(id, PermissionTransition::Superseded)
    }

    /// Whether the authority currently tracks `id` as pending.
    pub fn is_pending(&self, id: PermissionRequestId) -> bool {
        self.pending.contains_key(&id)
    }

    /// The pending request tracked under `id`, if any.
    pub fn pending_request(&self, id: PermissionRequestId) -> Option<&PermissionRequest> {
        self.pending.get(&id)
    }

    /// Whether the request reached a terminal state (resolved, cancelled, or
    /// superseded) and still has history.
    pub fn is_terminal(&self, id: PermissionRequestId) -> bool {
        self.terminal.contains(&id)
    }

    /// Drop terminal history for `id`, so a later completion reports as
    /// stale instead of duplicate. Pending requests cannot be forgotten;
    /// returns whether history was present.
    pub fn forget(&mut self, id: PermissionRequestId) -> bool {
        self.terminal.remove(&id)
    }

    /// The stored decision for `profile` + `scope`, if any.
    pub fn lookup_policy(
        &self,
        profile: ProfileId,
        scope: &PermissionScope,
    ) -> Option<PermissionDecision> {
        self.policy
            .get(&PermissionPolicyKey::new(profile, scope.clone()))
            .map(|value| value.decision())
    }

    /// Adopt an externally known durable decision (startup hydration or an
    /// external store change). Emits no command: this mirrors caller-owned
    /// storage into the authority's view.
    pub fn set_policy(&mut self, entry: PermissionPolicyEntry) {
        let (key, value) = entry.into_parts();
        self.policy.insert(key, value);
    }

    /// Drop the authority's view of a durable decision (external removal).
    /// Returns whether a decision was present.
    pub fn remove_policy(&mut self, key: &PermissionPolicyKey) -> bool {
        self.policy.remove(key).is_some()
    }

    fn allocate(
        &mut self,
        profile: ProfileId,
        scope: PermissionScope,
        user_gesture: bool,
    ) -> PermissionRequest {
        let id = PermissionRequestId::new(self.next_id);
        self.next_id += 1;
        PermissionRequest::new(
            id,
            profile,
            scope.origin().clone(),
            scope.capability(),
            user_gesture,
        )
    }

    fn drop_pending(
        &mut self,
        id: PermissionRequestId,
        transition: PermissionTransition,
    ) -> PermissionTransition {
        match self.pending.remove(&id) {
            Some(_) => {
                self.terminal.insert(id);
                transition
            }
            None => self.rejection_for(id),
        }
    }

    fn supersede_same_scope(
        &mut self,
        profile: ProfileId,
        scope: &PermissionScope,
    ) -> Option<PermissionRequestId> {
        let superseded = self
            .pending
            .iter()
            .find(|(_, request)| request.profile == profile && &request.scope == scope)
            .map(|(id, _)| *id);
        if let Some(id) = superseded {
            self.pending.remove(&id);
            self.terminal.insert(id);
        }
        superseded
    }

    fn rejection_for(&self, id: PermissionRequestId) -> PermissionTransition {
        if self.terminal.contains(&id) {
            PermissionTransition::Rejected(PermissionRejection::Duplicate)
        } else {
            PermissionTransition::Rejected(PermissionRejection::StaleId)
        }
    }
}

/// The outcome of registering a request with the authority.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionRequestOutcome {
    /// The request needs a decision and is now tracked as pending.
    Pending {
        /// The tracked request; the caller correlates its completion by id.
        request: PermissionRequest,
        /// The id of the previous pending request for the same profile and
        /// scope that this request superseded, if any.
        superseded: Option<PermissionRequestId>,
    },
    /// The durable policy already decided the request; nothing stays pending.
    ResolvedFromPolicy {
        /// The issued request (its id is valid for correlation and logging
        /// only — the request is terminal from the start).
        request: PermissionRequest,
        /// The stored decision.
        decision: PermissionDecision,
    },
}

/// The result of a lifecycle transition attempt.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionTransition {
    /// The request moved from pending to resolved with a decision.
    Resolved(PermissionResolution),
    /// The pending request was cancelled; it can never become an allow.
    Cancelled,
    /// The pending request was superseded by a newer request for the same
    /// scope; it can never become an allow.
    Superseded,
    /// The transition was refused and the request state is unchanged.
    Rejected(PermissionRejection),
}

/// A resolved transition: the decision plus the persistence command to
/// apply, when the verdict was durable.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionResolution {
    /// The granted or refused capability outcome.
    pub decision: PermissionDecision,
    /// The command to apply to the caller-owned durable store, or `None`
    /// when the verdict was one-time.
    pub command: Option<PermissionPersistenceCommand>,
}

/// Why a lifecycle transition was refused.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionRejection {
    /// The id was never issued, or its history was already forgotten.
    StaleId,
    /// The request already reached a terminal state; the completion is a
    /// duplicate.
    Duplicate,
    /// The completion's profile or scope does not match the pending request.
    MismatchedContext,
}

/// A completion reported by the engine back to the authority.
///
/// Carries the full request context so a mismatched or replayed completion
/// rejects whole instead of resolving the wrong request.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionCompletion {
    /// The id of the request being completed.
    pub id: PermissionRequestId,
    /// The profile the completion claims to complete for.
    pub profile: ProfileId,
    /// The scope the completion claims to complete for.
    pub scope: PermissionScope,
    /// The decision and its persistence scope.
    pub verdict: PermissionVerdict,
}

impl PermissionCompletion {
    /// Build a completion from the parts a native delegate reports.
    pub fn new(
        id: PermissionRequestId,
        profile: ProfileId,
        scope: PermissionScope,
        verdict: PermissionVerdict,
    ) -> Self {
        Self {
            id,
            profile,
            scope,
            verdict,
        }
    }
}

/// A durable policy mutation the caller applies to its storage.
///
/// Constructing a `Store` requires a profile-persisted verdict:
/// [`PermissionPersistenceCommand::store`] returns `None` for one-time
/// verdicts, so a one-time outcome can never cross this boundary.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionPersistenceCommand {
    /// Store (or overwrite) a durable decision for its key.
    Store(PermissionPolicyEntry),
    /// Remove any durable decision for the key.
    Remove(PermissionPolicyKey),
}

impl PermissionPersistenceCommand {
    /// The command to persist `verdict` under `key`, or `None` when the
    /// verdict is one-time and must never be stored.
    pub fn store(key: PermissionPolicyKey, verdict: PermissionVerdict) -> Option<Self> {
        verdict
            .into_policy_value()
            .map(|value| Self::Store(PermissionPolicyEntry::new(key, value)))
    }
}

/// The portable seam for the caller-owned durable policy store.
///
/// The authority never imports storage; it hands the caller
/// [`PermissionPersistenceCommand`]s and the caller applies them here.
/// Implementors are adapters over real repositories or in-memory doubles
/// that record what was stored and removed.
pub trait PermissionPolicyHook {
    /// Persist (or overwrite) a durable decision.
    fn store(&mut self, entry: PermissionPolicyEntry);

    /// Remove any durable decision for the key.
    fn remove(&mut self, key: &PermissionPolicyKey);

    /// Apply a batch of persistence commands in order.
    fn apply(&mut self, commands: &[PermissionPersistenceCommand]) {
        for command in commands {
            match command {
                PermissionPersistenceCommand::Store(entry) => self.store(entry.clone()),
                PermissionPersistenceCommand::Remove(key) => self.remove(key),
            }
        }
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod persistence_tests;
