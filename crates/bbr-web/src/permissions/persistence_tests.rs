//! Deterministic tests for the caller-owned persistence boundary (step 2 of
//! the permissions contract): durable verdicts become store commands applied
//! through a recording hook, one-time verdicts never reach the hook,
//! profile/origin isolation holds across keys, and policy hydration and
//! external removal round-trip through the authority's view.

use super::super::{
    PermissionCapability, PermissionDecision, PermissionOrigin, PermissionPolicyEntry,
    PermissionPolicyKey, PermissionPolicyValue, PermissionScope, PermissionVerdict,
};
use super::{
    PermissionCompletion, PermissionLifecycle, PermissionPersistenceCommand, PermissionPolicyHook,
    PermissionRequestOutcome, PermissionTransition,
};
use crate::ProfileId;

/// A test double that records every durable decision the authority reports.
#[derive(Debug, Default)]
struct RecordingHook {
    stored: Vec<PermissionPolicyEntry>,
    removed: Vec<PermissionPolicyKey>,
}

impl PermissionPolicyHook for RecordingHook {
    fn store(&mut self, entry: PermissionPolicyEntry) {
        self.stored.push(entry);
    }

    fn remove(&mut self, key: &PermissionPolicyKey) {
        self.removed.push(key.clone());
    }
}

fn origin(raw: &str) -> PermissionOrigin {
    PermissionOrigin::parse(raw).expect("test origin must parse")
}

fn scope(raw: &str, capability: PermissionCapability) -> PermissionScope {
    PermissionScope::new(origin(raw), capability)
}

fn completion(
    id: super::super::PermissionRequestId,
    profile: u64,
    raw: &str,
    capability: PermissionCapability,
    verdict: PermissionVerdict,
) -> PermissionCompletion {
    PermissionCompletion::new(id, ProfileId(profile), scope(raw, capability), verdict)
}

fn pending_request(
    lifecycle: &mut PermissionLifecycle,
    profile: u64,
    raw: &str,
    capability: PermissionCapability,
) -> super::super::PermissionRequest {
    match lifecycle.request(ProfileId(profile), scope(raw, capability), false) {
        PermissionRequestOutcome::Pending { request, .. } => request,
        other => panic!("expected a pending request, got {other:?}"),
    }
}

#[test]
fn persistent_resolution_updates_policy_and_emits_store() {
    let mut lifecycle = PermissionLifecycle::new();
    let mut hook = RecordingHook::default();
    let request = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    let transition = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_persisted(),
    ));
    let PermissionTransition::Resolved(resolution) = transition else {
        panic!("expected a resolution");
    };
    assert_eq!(resolution.decision, PermissionDecision::Allow);
    let Some(command) = resolution.command else {
        panic!("persistent verdict must emit a store command");
    };
    hook.apply(&[command]);
    assert_eq!(hook.stored.len(), 1);
    assert_eq!(
        hook.stored[0].key(),
        &PermissionPolicyKey::new(
            ProfileId(1),
            scope("https://example.com", PermissionCapability::Camera)
        )
    );
    assert_eq!(hook.stored[0].value().decision(), PermissionDecision::Allow);

    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        Some(PermissionDecision::Allow)
    );

    let replay = lifecycle.request(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
        false,
    );
    assert!(matches!(
        replay,
        PermissionRequestOutcome::ResolvedFromPolicy {
            decision: PermissionDecision::Allow,
            ..
        }
    ));
}

#[test]
fn one_time_verdicts_never_reach_the_hook() {
    let key = PermissionPolicyKey::new(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
    );
    assert_eq!(
        PermissionPersistenceCommand::store(key.clone(), PermissionVerdict::allow_one_time()),
        None
    );
    assert_eq!(
        PermissionPersistenceCommand::store(key.clone(), PermissionVerdict::deny_one_time()),
        None
    );
    assert!(matches!(
        PermissionPersistenceCommand::store(key, PermissionVerdict::deny_persisted()),
        Some(PermissionPersistenceCommand::Store(_))
    ));

    let mut lifecycle = PermissionLifecycle::new();
    let mut hook = RecordingHook::default();
    let camera = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );
    let microphone = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Microphone,
    );

    let transitions = [
        lifecycle.resolve(completion(
            camera.id,
            1,
            "https://example.com",
            PermissionCapability::Camera,
            PermissionVerdict::allow_one_time(),
        )),
        lifecycle.resolve(completion(
            microphone.id,
            1,
            "https://example.com",
            PermissionCapability::Microphone,
            PermissionVerdict::deny_one_time(),
        )),
    ];
    let commands: Vec<_> = transitions
        .iter()
        .filter_map(|transition| match transition {
            PermissionTransition::Resolved(resolution) => resolution.command.clone(),
            _ => None,
        })
        .collect();
    assert!(commands.is_empty());
    hook.apply(&commands);
    assert!(hook.stored.is_empty());
    assert!(hook.removed.is_empty());
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        None
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Microphone)
        ),
        None
    );
}

#[test]
fn persistence_is_profile_and_origin_isolated() {
    let mut lifecycle = PermissionLifecycle::new();
    let mut hook = RecordingHook::default();

    let p1_camera = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );
    let p2_camera = pending_request(
        &mut lifecycle,
        2,
        "https://example.com",
        PermissionCapability::Camera,
    );
    let p1_other = pending_request(
        &mut lifecycle,
        1,
        "https://example.org",
        PermissionCapability::Camera,
    );

    let transitions = [
        lifecycle.resolve(completion(
            p1_camera.id,
            1,
            "https://example.com",
            PermissionCapability::Camera,
            PermissionVerdict::allow_persisted(),
        )),
        lifecycle.resolve(completion(
            p2_camera.id,
            2,
            "https://example.com",
            PermissionCapability::Camera,
            PermissionVerdict::deny_persisted(),
        )),
        lifecycle.resolve(completion(
            p1_other.id,
            1,
            "https://example.org",
            PermissionCapability::Camera,
            PermissionVerdict::allow_persisted(),
        )),
    ];
    let commands: Vec<_> = transitions
        .into_iter()
        .filter_map(|transition| match transition {
            PermissionTransition::Resolved(resolution) => resolution.command,
            _ => None,
        })
        .collect();
    assert_eq!(commands.len(), 3);
    hook.apply(&commands);

    assert_eq!(hook.stored.len(), 3);
    let p1_key = PermissionPolicyKey::new(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
    );
    let p2_key = PermissionPolicyKey::new(
        ProfileId(2),
        scope("https://example.com", PermissionCapability::Camera),
    );
    let p1_other_key = PermissionPolicyKey::new(
        ProfileId(1),
        scope("https://example.org", PermissionCapability::Camera),
    );
    let keys: Vec<_> = hook
        .stored
        .iter()
        .map(|entry| entry.key().clone())
        .collect();
    assert!(keys.contains(&p1_key));
    assert!(keys.contains(&p2_key));
    assert!(keys.contains(&p1_other_key));
    assert_ne!(p1_key, p2_key);
    assert_ne!(p1_key, p1_other_key);

    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        Some(PermissionDecision::Allow)
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(2),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        Some(PermissionDecision::Deny)
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.org", PermissionCapability::Camera)
        ),
        Some(PermissionDecision::Allow)
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Microphone)
        ),
        None
    );
}

#[test]
fn policy_hydration_and_external_removal_round_trip() {
    let mut lifecycle = PermissionLifecycle::new();
    let key = PermissionPolicyKey::new(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
    );
    lifecycle.set_policy(PermissionPolicyEntry::new(
        key.clone(),
        PermissionPolicyValue::new(PermissionDecision::Allow),
    ));
    assert!(matches!(
        lifecycle.request(
            ProfileId(1),
            scope("https://example.com", PermissionCapability::Camera),
            false
        ),
        PermissionRequestOutcome::ResolvedFromPolicy {
            decision: PermissionDecision::Allow,
            ..
        }
    ));

    assert!(lifecycle.remove_policy(&key));
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        None
    );
    assert!(matches!(
        lifecycle.request(
            ProfileId(1),
            scope("https://example.com", PermissionCapability::Camera),
            false
        ),
        PermissionRequestOutcome::Pending { .. }
    ));

    assert!(!lifecycle.remove_policy(&key));
}

#[test]
fn policy_hit_supersedes_pending_same_scope_request() {
    let mut lifecycle = PermissionLifecycle::new();
    let first = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    lifecycle.set_policy(PermissionPolicyEntry::new(
        PermissionPolicyKey::new(
            ProfileId(1),
            scope("https://example.com", PermissionCapability::Camera),
        ),
        PermissionPolicyValue::new(PermissionDecision::Allow),
    ));

    let outcome = lifecycle.request(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
        true,
    );
    assert!(matches!(
        outcome,
        PermissionRequestOutcome::ResolvedFromPolicy {
            decision: PermissionDecision::Allow,
            ..
        }
    ));
    assert!(lifecycle.is_terminal(first.id));
    assert!(!lifecycle.is_pending(first.id));
}

#[test]
fn hook_applies_store_then_remove_in_order() {
    let mut hook = RecordingHook::default();
    let key = PermissionPolicyKey::new(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
    );
    let store =
        PermissionPersistenceCommand::store(key.clone(), PermissionVerdict::allow_persisted())
            .expect("persisted verdict must produce a command");
    let remove = PermissionPersistenceCommand::Remove(key.clone());
    hook.apply(&[store, remove]);
    assert_eq!(hook.stored.len(), 1);
    assert_eq!(hook.removed.len(), 1);
    assert_eq!(hook.removed[0], key);
}
