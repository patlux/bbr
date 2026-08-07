//! Deterministic tests for the portable permission identities, taxonomy, and
//! validation (step 1 of the permissions contract). Serde-derived wire shapes
//! and origin re-validation at the deserialization boundary live in the
//! sibling `tests_serde` module, compiled only with the `serde` feature.

use super::*;

fn origin(raw: &str) -> PermissionOrigin {
    PermissionOrigin::parse(raw).expect("test origin must parse")
}

fn camera_scope(raw: &str) -> PermissionScope {
    PermissionScope::new(origin(raw), PermissionCapability::Camera)
}

fn policy_key(profile: u64, raw: &str, capability: PermissionCapability) -> PermissionPolicyKey {
    PermissionPolicyKey::new(
        ProfileId(profile),
        PermissionScope::new(origin(raw), capability),
    )
}

fn request_id(value: u64) -> PermissionRequestId {
    PermissionRequestId::new(value)
}

/// The taxonomy is closed at exactly the five named capabilities; `ALL` is
/// the authoritative enumeration and contains no duplicates.
#[test]
fn capability_taxonomy_is_exactly_five() {
    use std::collections::HashSet;

    assert_eq!(PermissionCapability::ALL.len(), 5);
    let mut seen = HashSet::new();
    for capability in PermissionCapability::ALL {
        assert!(seen.insert(capability), "duplicate capability in ALL");
    }
    assert_eq!(seen.len(), 5);
    assert!(seen.contains(&PermissionCapability::Camera));
    assert!(seen.contains(&PermissionCapability::Microphone));
    assert!(seen.contains(&PermissionCapability::Location));
    assert!(seen.contains(&PermissionCapability::Notifications));
    assert!(seen.contains(&PermissionCapability::Clipboard));
}

/// Valid origins parse, normalize case and default ports, and strip any
/// path/query/fragment so equivalent spellings collapse to one identity.
#[test]
fn origins_parse_and_normalize() {
    let plain = origin("https://example.com");
    assert_eq!(plain.scheme(), "https");
    assert_eq!(plain.host(), "example.com");
    assert_eq!(plain.port(), None);
    assert_eq!(plain.to_string(), "https://example.com");

    assert_eq!(origin("HTTPS://Example.COM/Path?query=1#frag"), plain);

    let non_default_port = origin("https://example.com:8443");
    assert_eq!(non_default_port.port(), Some(8443));
    assert_eq!(non_default_port.to_string(), "https://example.com:8443");

    let ipv6 = origin("https://[::1]:8443");
    assert_eq!(ipv6.host(), "[::1]");
    assert_eq!(ipv6.port(), Some(8443));

    assert_eq!(
        origin("http://example.com"),
        origin("http://example.com:80")
    );
    assert_eq!(
        origin("https://example.com"),
        origin("https://example.com:443")
    );
}

/// Empty, scheme-less, hostless, userinfo-carrying, whitespace-carrying, and
/// malformed-port origins are rejected with a typed reason instead of being
/// normalized into a policy key.
#[test]
fn invalid_and_empty_origins_are_rejected() {
    assert_eq!(PermissionOrigin::parse(""), Err(OriginError::Empty));
    assert_eq!(PermissionOrigin::parse("   "), Err(OriginError::Empty));
    assert_eq!(
        PermissionOrigin::parse("example.com"),
        Err(OriginError::MissingSchemeSeparator)
    );
    assert_eq!(
        PermissionOrigin::parse("://example.com"),
        Err(OriginError::InvalidScheme)
    );
    assert_eq!(
        PermissionOrigin::parse("1https://example.com"),
        Err(OriginError::InvalidScheme)
    );
    assert_eq!(
        PermissionOrigin::parse("https://"),
        Err(OriginError::EmptyHost)
    );
    assert_eq!(
        PermissionOrigin::parse("https:///path"),
        Err(OriginError::EmptyHost)
    );
    assert_eq!(
        PermissionOrigin::parse("https://user@example.com"),
        Err(OriginError::InvalidHost)
    );
    assert_eq!(
        PermissionOrigin::parse("https://exa mple.com"),
        Err(OriginError::InvalidHost)
    );
    assert_eq!(
        PermissionOrigin::parse("https://example.com:"),
        Err(OriginError::InvalidPort)
    );
    assert_eq!(
        PermissionOrigin::parse("https://example.com:abc"),
        Err(OriginError::InvalidPort)
    );
    assert_eq!(
        PermissionOrigin::parse("https://example.com:0"),
        Err(OriginError::InvalidPort)
    );
    assert_eq!(
        PermissionOrigin::parse("https://example.com:65536"),
        Err(OriginError::InvalidPort)
    );
    assert_eq!(
        PermissionOrigin::parse("https://example.com:80:90"),
        Err(OriginError::InvalidPort)
    );
}

/// The WebKit-parts construction path applies the same validation and
/// normalization as the string path.
#[test]
fn try_new_validates_parts_like_parse() {
    assert_eq!(
        PermissionOrigin::try_new("https", "Example.COM", None).unwrap(),
        origin("https://example.com")
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "example.com", Some(443))
            .unwrap()
            .port(),
        None
    );
    assert_eq!(
        PermissionOrigin::try_new("http", "example.com", Some(80))
            .unwrap()
            .port(),
        None
    );
    assert_eq!(
        PermissionOrigin::try_new("", "example.com", None),
        Err(OriginError::InvalidScheme)
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "", None),
        Err(OriginError::EmptyHost)
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "  ", None),
        Err(OriginError::EmptyHost)
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "a b", None),
        Err(OriginError::InvalidHost)
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "user@example.com", None),
        Err(OriginError::InvalidHost)
    );
    assert_eq!(
        PermissionOrigin::try_new("https", "example.com", Some(0)),
        Err(OriginError::InvalidPort)
    );
}

/// Origin identity collapses equivalent spellings and keeps distinct origins
/// distinct, so a durable key can never conflate two origins.
#[test]
fn origin_identity_prevents_cross_origin_keys() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    assert!(seen.insert(origin("https://example.com")));
    assert!(!seen.insert(origin("HTTPS://Example.COM/")));
    assert!(!seen.insert(origin("https://example.com:443")));
    assert!(seen.insert(origin("http://example.com")));
    assert!(seen.insert(origin("https://example.org")));
    assert!(seen.insert(origin("https://example.com:8443")));
    assert!(seen.insert(origin("https://example.com:8444")));
}

/// Scope identity is exactly origin + capability.
#[test]
fn scope_identity_pairs_origin_with_capability() {
    let camera = camera_scope("https://example.com");
    assert_eq!(camera.capability(), PermissionCapability::Camera);
    assert_eq!(camera.origin(), &origin("https://example.com"));
    assert_eq!(
        camera,
        PermissionScope::new(origin("https://example.com"), PermissionCapability::Camera)
    );
    assert_ne!(
        camera,
        PermissionScope::new(
            origin("https://example.com"),
            PermissionCapability::Microphone
        )
    );
    assert_ne!(
        camera,
        PermissionScope::new(origin("https://example.org"), PermissionCapability::Camera)
    );
}

/// Durable policy keys are equal iff profile, origin, and capability all
/// match, and they work as ordered and hashed collection keys.
#[test]
fn policy_keys_are_profile_origin_capability_isolated() {
    use std::collections::{BTreeMap, HashSet};

    let camera = PermissionCapability::Camera;
    let microphone = PermissionCapability::Microphone;

    let key = policy_key(1, "https://example.com", camera);
    let same_spelling = policy_key(1, "HTTPS://example.com:443", camera);
    assert_eq!(key, same_spelling);

    assert_ne!(key, policy_key(2, "https://example.com", camera));
    assert_ne!(key, policy_key(1, "https://example.org", camera));
    assert_ne!(key, policy_key(1, "https://example.com", microphone));

    let mut by_key = BTreeMap::new();
    by_key.insert(key.clone(), "allow");
    assert_eq!(by_key.get(&same_spelling), Some(&"allow"));
    assert_eq!(
        by_key.get(&policy_key(9, "https://other.example", camera)),
        None
    );

    let mut seen = HashSet::new();
    assert!(seen.insert(key.clone()));
    assert!(!seen.insert(same_spelling));
    assert!(seen.insert(policy_key(2, "https://example.com", camera)));
    assert_eq!(seen.len(), 2);

    assert_eq!(key.profile(), ProfileId(1));
    assert_eq!(key.scope(), &camera_scope("https://example.com"));
}

/// A request carries the stable id, owning profile, scope, and user-gesture
/// context; equal parts produce an equal request.
#[test]
fn request_carries_stable_profile_origin_gesture_context() {
    let request = PermissionRequest::new(
        request_id(7),
        ProfileId(3),
        origin("https://example.com"),
        PermissionCapability::Notifications,
        true,
    );
    assert_eq!(request.id, request_id(7));
    assert_eq!(request.id.value(), 7);
    assert_eq!(request.profile, ProfileId(3));
    assert_eq!(
        request.scope.capability(),
        PermissionCapability::Notifications
    );
    assert_eq!(request.scope.origin(), &origin("https://example.com"));
    assert!(request.user_gesture);

    let same = PermissionRequest::new(
        request_id(7),
        ProfileId(3),
        origin("https://example.com"),
        PermissionCapability::Notifications,
        true,
    );
    assert_eq!(request, same);

    assert_ne!(
        request,
        PermissionRequest::new(
            request_id(8),
            ProfileId(3),
            origin("https://example.com"),
            PermissionCapability::Notifications,
            true,
        )
    );
    assert_ne!(
        request,
        PermissionRequest::new(
            request_id(7),
            ProfileId(3),
            origin("https://example.com"),
            PermissionCapability::Notifications,
            false,
        )
    );
}

/// One-time verdicts never convert into a durable policy value; only
/// profile-persisted verdicts do.
#[test]
fn one_time_decisions_never_become_policy_records() {
    assert_eq!(
        PermissionVerdict::allow_one_time().into_policy_value(),
        None
    );
    assert_eq!(PermissionVerdict::deny_one_time().into_policy_value(), None);
    assert!(!PermissionVerdict::allow_one_time().is_persistent());
    assert!(!PermissionVerdict::deny_one_time().is_persistent());

    let allowed = PermissionVerdict::allow_persisted();
    assert!(allowed.is_persistent());
    assert_eq!(
        allowed.into_policy_value(),
        Some(PermissionPolicyValue::new(PermissionDecision::Allow))
    );

    let denied = PermissionVerdict::deny_persisted();
    assert!(denied.is_persistent());
    assert_eq!(
        denied.into_policy_value(),
        Some(PermissionPolicyValue::new(PermissionDecision::Deny))
    );

    let verdict = PermissionVerdict::new(PermissionDecision::Deny, PermissionPersistence::OneTime);
    assert_eq!(verdict.decision, PermissionDecision::Deny);
    assert_eq!(verdict.persistence, PermissionPersistence::OneTime);
    assert!(!verdict.is_persistent());
}

/// Identities and decisions are stable plain values: Copy where applicable,
/// ordered, hashable, with stable Debug output.
#[test]
fn identities_and_decisions_are_stable_plain_values() {
    assert_eq!(request_id(1), request_id(1));
    assert_ne!(request_id(1), request_id(2));
    assert!(request_id(1) < request_id(2));
    assert_eq!(request_id(3).value(), 3);

    assert_eq!(PermissionCapability::Camera, PermissionCapability::Camera);
    assert_ne!(
        PermissionCapability::Camera,
        PermissionCapability::Clipboard
    );
    assert_eq!(PermissionDecision::Allow, PermissionDecision::Allow);
    assert_ne!(PermissionDecision::Allow, PermissionDecision::Deny);
    assert_eq!(
        PermissionPersistence::ProfilePersisted,
        PermissionPersistence::ProfilePersisted
    );
    assert_ne!(
        PermissionPersistence::OneTime,
        PermissionPersistence::ProfilePersisted
    );

    assert_eq!(
        format!("{:?}", PermissionCapability::Notifications),
        "Notifications"
    );
    assert_eq!(format!("{:?}", request_id(4)), "PermissionRequestId(4)");

    let entry = PermissionPolicyEntry::new(
        policy_key(1, "https://example.com", PermissionCapability::Camera),
        PermissionPolicyValue::new(PermissionDecision::Allow),
    );
    assert_eq!(entry.clone(), entry);
    assert_eq!(
        entry.key(),
        &policy_key(1, "https://example.com", PermissionCapability::Camera)
    );
    assert_eq!(
        entry.value(),
        &PermissionPolicyValue::new(PermissionDecision::Allow)
    );
    let (key, value) = entry.into_parts();
    assert_eq!(key.profile(), ProfileId(1));
    assert_eq!(value.decision(), PermissionDecision::Allow);
}
