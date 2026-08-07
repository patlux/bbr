//! Deterministic tests for the request lifecycle state machine (step 2 of
//! the permissions contract): pending tracking, durable-policy resolution,
//! explicit resolve/cancel/supersede, stale/duplicate/mismatched rejection,
//! and same-scope supersession. The persistence boundary is covered by the
//! sibling `persistence_tests` module.

use super::super::{
    PermissionCapability, PermissionDecision, PermissionOrigin, PermissionPolicyEntry,
    PermissionPolicyKey, PermissionPolicyValue, PermissionRequest, PermissionRequestId,
    PermissionScope, PermissionVerdict,
};
use super::{
    PermissionCompletion, PermissionLifecycle, PermissionRejection, PermissionRequestOutcome,
    PermissionTransition,
};
use crate::ProfileId;

fn origin(raw: &str) -> PermissionOrigin {
    PermissionOrigin::parse(raw).expect("test origin must parse")
}

fn scope(raw: &str, capability: PermissionCapability) -> PermissionScope {
    PermissionScope::new(origin(raw), capability)
}

fn request_id(value: u64) -> PermissionRequestId {
    PermissionRequestId::new(value)
}

fn completion(
    id: PermissionRequestId,
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
) -> PermissionRequest {
    match lifecycle.request(ProfileId(profile), scope(raw, capability), false) {
        PermissionRequestOutcome::Pending { request, .. } => request,
        other => panic!("expected a pending request, got {other:?}"),
    }
}

#[test]
fn request_becomes_pending_with_stable_allocated_id() {
    let mut lifecycle = PermissionLifecycle::new();
    let outcome = lifecycle.request(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
        true,
    );
    let PermissionRequestOutcome::Pending {
        request,
        superseded,
    } = outcome
    else {
        panic!("expected a pending request");
    };
    assert_eq!(superseded, None);
    assert_eq!(request.profile, ProfileId(1));
    assert_eq!(
        request.scope,
        scope("https://example.com", PermissionCapability::Camera)
    );
    assert!(request.user_gesture);
    assert!(lifecycle.is_pending(request.id));
    assert_eq!(lifecycle.pending_request(request.id), Some(&request));

    let second = lifecycle.request(
        ProfileId(1),
        scope("https://example.org", PermissionCapability::Microphone),
        false,
    );
    let PermissionRequestOutcome::Pending {
        request: second_request,
        ..
    } = second
    else {
        panic!("expected a second pending request");
    };
    assert_ne!(second_request.id, request.id);
    assert!(lifecycle.is_pending(second_request.id));
    assert!(!second_request.user_gesture);
}

#[test]
fn durable_policy_hit_resolves_without_pending() {
    let mut lifecycle = PermissionLifecycle::new();
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
        false,
    );
    let PermissionRequestOutcome::ResolvedFromPolicy { request, decision } = outcome else {
        panic!("expected a policy hit");
    };
    assert_eq!(decision, PermissionDecision::Allow);
    assert!(!lifecycle.is_pending(request.id));
    assert!(lifecycle.is_terminal(request.id));

    lifecycle.set_policy(PermissionPolicyEntry::new(
        PermissionPolicyKey::new(
            ProfileId(2),
            scope("https://example.com", PermissionCapability::Microphone),
        ),
        PermissionPolicyValue::new(PermissionDecision::Deny),
    ));
    let outcome = lifecycle.request(
        ProfileId(2),
        scope("https://example.com", PermissionCapability::Microphone),
        true,
    );
    let PermissionRequestOutcome::ResolvedFromPolicy { decision, .. } = outcome else {
        panic!("expected a policy hit");
    };
    assert_eq!(decision, PermissionDecision::Deny);
}

#[test]
fn explicit_one_time_resolution_never_persists() {
    let mut lifecycle = PermissionLifecycle::new();
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
        PermissionVerdict::allow_one_time(),
    ));
    let PermissionTransition::Resolved(resolution) = transition else {
        panic!("expected a resolution");
    };
    assert_eq!(resolution.decision, PermissionDecision::Allow);
    assert_eq!(resolution.command, None);
    assert!(lifecycle.is_terminal(request.id));
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        None
    );

    let duplicate = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert_eq!(
        duplicate,
        PermissionTransition::Rejected(PermissionRejection::Duplicate)
    );
}

#[test]
fn cancel_rejects_later_completions_and_never_allows() {
    let mut lifecycle = PermissionLifecycle::new();
    let request = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    assert_eq!(
        lifecycle.cancel(request.id),
        PermissionTransition::Cancelled
    );
    assert!(!lifecycle.is_pending(request.id));
    assert!(lifecycle.is_terminal(request.id));

    let later = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_persisted(),
    ));
    assert_eq!(
        later,
        PermissionTransition::Rejected(PermissionRejection::Duplicate)
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        None
    );
}

#[test]
fn supersede_rejects_later_completions_and_never_allows() {
    let mut lifecycle = PermissionLifecycle::new();
    let request = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    assert_eq!(
        lifecycle.supersede(request.id),
        PermissionTransition::Superseded
    );
    assert!(!lifecycle.is_pending(request.id));
    assert!(lifecycle.is_terminal(request.id));

    let later = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert_eq!(
        later,
        PermissionTransition::Rejected(PermissionRejection::Duplicate)
    );
    assert_eq!(
        lifecycle.lookup_policy(
            ProfileId(1),
            &scope("https://example.com", PermissionCapability::Camera)
        ),
        None
    );
}

#[test]
fn same_scope_request_supersedes_pending_previous() {
    let mut lifecycle = PermissionLifecycle::new();
    let first = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    let outcome = lifecycle.request(
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Camera),
        true,
    );
    let PermissionRequestOutcome::Pending {
        request: second,
        superseded,
    } = outcome
    else {
        panic!("expected a pending request");
    };
    assert_eq!(superseded, Some(first.id));
    assert_ne!(second.id, first.id);
    assert!(lifecycle.is_terminal(first.id));
    assert!(!lifecycle.is_pending(first.id));
    assert!(lifecycle.is_pending(second.id));

    let stale = lifecycle.resolve(completion(
        first.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert_eq!(
        stale,
        PermissionTransition::Rejected(PermissionRejection::Duplicate)
    );

    let resolution = lifecycle.resolve(completion(
        second.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert!(matches!(resolution, PermissionTransition::Resolved(_)));
}

#[test]
fn different_scope_requests_stay_pending_independently() {
    let mut lifecycle = PermissionLifecycle::new();
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
    let other_origin = pending_request(
        &mut lifecycle,
        1,
        "https://example.org",
        PermissionCapability::Camera,
    );
    let other_profile = pending_request(
        &mut lifecycle,
        2,
        "https://example.com",
        PermissionCapability::Camera,
    );
    for id in [camera.id, microphone.id, other_origin.id, other_profile.id] {
        assert!(lifecycle.is_pending(id));
        assert!(!lifecycle.is_terminal(id));
    }
}

#[test]
fn stale_unknown_ids_reject_whole() {
    let mut lifecycle = PermissionLifecycle::new();
    let missing = request_id(999);
    assert_eq!(
        lifecycle.resolve(PermissionCompletion::new(
            missing,
            ProfileId(1),
            scope("https://example.com", PermissionCapability::Camera),
            PermissionVerdict::allow_one_time(),
        )),
        PermissionTransition::Rejected(PermissionRejection::StaleId)
    );
    assert_eq!(
        lifecycle.cancel(missing),
        PermissionTransition::Rejected(PermissionRejection::StaleId)
    );
    assert_eq!(
        lifecycle.supersede(missing),
        PermissionTransition::Rejected(PermissionRejection::StaleId)
    );
    assert!(!lifecycle.is_pending(missing));
    assert!(!lifecycle.is_terminal(missing));
}

#[test]
fn mismatched_completion_context_rejects_whole() {
    let mut lifecycle = PermissionLifecycle::new();
    let request = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    let wrong_profile = PermissionCompletion::new(
        request.id,
        ProfileId(2),
        scope("https://example.com", PermissionCapability::Camera),
        PermissionVerdict::allow_one_time(),
    );
    assert_eq!(
        lifecycle.resolve(wrong_profile),
        PermissionTransition::Rejected(PermissionRejection::MismatchedContext)
    );

    let wrong_origin = PermissionCompletion::new(
        request.id,
        ProfileId(1),
        scope("https://example.org", PermissionCapability::Camera),
        PermissionVerdict::allow_one_time(),
    );
    assert_eq!(
        lifecycle.resolve(wrong_origin),
        PermissionTransition::Rejected(PermissionRejection::MismatchedContext)
    );

    let wrong_capability = PermissionCompletion::new(
        request.id,
        ProfileId(1),
        scope("https://example.com", PermissionCapability::Microphone),
        PermissionVerdict::allow_one_time(),
    );
    assert_eq!(
        lifecycle.resolve(wrong_capability),
        PermissionTransition::Rejected(PermissionRejection::MismatchedContext)
    );

    assert!(lifecycle.is_pending(request.id));

    let ok = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::deny_one_time(),
    ));
    assert!(matches!(ok, PermissionTransition::Resolved(_)));
}

#[test]
fn forget_drops_terminal_history_only() {
    let mut lifecycle = PermissionLifecycle::new();
    let request = pending_request(
        &mut lifecycle,
        1,
        "https://example.com",
        PermissionCapability::Camera,
    );

    assert!(!lifecycle.forget(request.id));

    let transition = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert!(matches!(transition, PermissionTransition::Resolved(_)));
    assert!(lifecycle.is_terminal(request.id));

    assert!(lifecycle.forget(request.id));
    assert!(!lifecycle.is_terminal(request.id));

    let later = lifecycle.resolve(completion(
        request.id,
        1,
        "https://example.com",
        PermissionCapability::Camera,
        PermissionVerdict::allow_one_time(),
    ));
    assert_eq!(
        later,
        PermissionTransition::Rejected(PermissionRejection::StaleId)
    );
}
