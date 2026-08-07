//! Default-feature contract tests: identity, validation/normalization, the
//! closed taxonomy, proxy/origin separation, opaque handle separation, and
//! redacted Debug shapes. Serde-only shape tests live in `tests_serde/`.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::auth_challenges::{
    AuthAuthority, AuthChallenge, AuthScheme, ChallengeId, ChallengeKind, ChallengeRejection,
    ClientIdentityHandle, CredentialHandle, Host, HttpScheme, ProtectionSpace,
    ProtectionSpaceError, Realm, UnsupportedChallenge,
};
use crate::{ProfileId, WebViewId};

fn cid(n: u64) -> ChallengeId {
    ChallengeId(n)
}

fn origin_space(host: &str) -> ProtectionSpace {
    ProtectionSpace::new(HttpScheme::Https, host, 0, None, AuthAuthority::Origin)
        .expect("valid origin space")
}

fn basic_challenge(n: u64) -> AuthChallenge {
    AuthChallenge::new(
        cid(n),
        ProfileId(1),
        WebViewId(2),
        origin_space("example.com"),
        ChallengeKind::HttpAuth(AuthScheme::Basic),
        true,
    )
    .expect("valid challenge")
}

/// Identifiers are plain ordered values usable as `BTreeMap`/`BTreeSet` keys
/// and `HashSet` elements, mirroring the `DownloadId` and `ProfileId`/
/// `WebViewId` guards elsewhere in the crate.
#[test]
fn challenge_ids_are_ordered_and_hashable() {
    let mut pending = BTreeMap::new();
    pending.insert(cid(1), "first");
    pending.insert(cid(2), "second");
    assert_eq!(pending.get(&cid(2)), Some(&"second"));

    let mut done = BTreeSet::new();
    done.insert(cid(3));
    assert!(done.contains(&cid(3)));
    assert!(!done.contains(&cid(4)));

    let mut seen = HashSet::new();
    seen.insert(cid(5));
    assert!(seen.contains(&cid(5)));
    assert!(!seen.contains(&cid(6)));
}

/// Hosts are trimmed, lowercased, bounded, and reject any character that
/// would make the protection-space key ambiguous.
#[test]
fn host_validation_normalizes_and_rejects() {
    // Case and surrounding whitespace collapse to one key.
    assert_eq!(
        Host::new("Example.COM").unwrap(),
        Host::new("example.com").unwrap()
    );
    assert_eq!(
        Host::new("  Example.COM  ").unwrap(),
        Host::new("example.com").unwrap()
    );
    assert_eq!(Host::new("example.com").unwrap().as_str(), "example.com");

    // Bracketed IPv6 literals are allowed and lowercased; colons and
    // zone-id percent are legal only inside the brackets.
    assert_eq!(Host::new("[::1]").unwrap().as_str(), "[::1]");

    assert_eq!(
        Host::new("[2001:DB8::1]").unwrap().as_str(),
        "[2001:db8::1]"
    );
    assert_eq!(
        Host::new("[fe80::1%eth0]").unwrap().as_str(),
        "[fe80::1%eth0]"
    );

    // Empty or whitespace-only hosts are rejected.
    assert_eq!(Host::new(""), Err(ProtectionSpaceError::EmptyHost));
    assert_eq!(Host::new("   "), Err(ProtectionSpaceError::EmptyHost));

    // Malformed IPv6 literals and bracket placement are rejected.
    for bad in ["[::1", "::1]", "[]", "[ ]", "[::1]extra", "a[b]", "[a]b"] {
        assert_eq!(
            Host::new(bad),
            Err(ProtectionSpaceError::InvalidHost),
            "host {bad:?} must be rejected"
        );
    }

    // Ambiguous or illegal characters are rejected: whitespace, control
    // bytes, and URL delimiters that could be mistaken for a port, path,
    // query, fragment, userinfo, or encoding. Edge ASCII whitespace (spaces,
    // newlines, tabs) is trimmed like surrounding spaces; interior control
    // bytes and delimiters are rejected.
    for bad in [
        "exa mple.com",
        "example\n.com",
        "example\t.com",
        "example.com:80",
        "a/b",
        "a?b",
        "a#b",
        "a@b",
        "a%b",
        "example.com\u{0}",
    ] {
        assert_eq!(
            Host::new(bad),
            Err(ProtectionSpaceError::InvalidHost),
            "host {bad:?} must be rejected"
        );
    }
    assert_eq!(
        Host::new("example.com\n").unwrap(),
        Host::new("example.com").unwrap()
    );
    assert_eq!(
        Host::new("example.com\t").unwrap(),
        Host::new("example.com").unwrap()
    );

    // Length bound: RFC 1035 full name length.
    assert_eq!(
        Host::new(&"a".repeat(254)),
        Err(ProtectionSpaceError::HostTooLong)
    );
    assert!(Host::new(&"a".repeat(253)).is_ok());
}

/// Realms are trimmed, length-bounded, and case-preserving (RFC 7617).
#[test]
fn realm_validation_preserves_case_and_rejects_empty_or_long() {
    assert_eq!(Realm::new("  Realm One  ").unwrap().as_str(), "Realm One");
    assert_eq!(
        Realm::new("Realm One").unwrap(),
        Realm::new("  Realm One ").unwrap()
    );
    // Case is preserved, so differently-cased labels are different realms.
    assert_ne!(
        Realm::new("Realm One").unwrap(),
        Realm::new("realm one").unwrap()
    );

    assert_eq!(Realm::new(""), Err(ProtectionSpaceError::EmptyRealm));
    assert_eq!(Realm::new("   "), Err(ProtectionSpaceError::EmptyRealm));
    assert_eq!(
        Realm::new(&"r".repeat(513)),
        Err(ProtectionSpaceError::RealmTooLong)
    );
    assert!(Realm::new(&"r".repeat(512)).is_ok());
}

/// The combined protection-space constructor surfaces host/realm validation
#[test]
fn protection_space_validates_through_combined_constructor() {
    assert_eq!(
        ProtectionSpace::new(HttpScheme::Https, "", 443, None, AuthAuthority::Origin),
        Err(ProtectionSpaceError::EmptyHost)
    );
    assert_eq!(
        ProtectionSpace::new(
            HttpScheme::Https,
            "example.com",
            443,
            Some(""),
            AuthAuthority::Origin
        ),
        Err(ProtectionSpaceError::EmptyRealm)
    );
    assert!(ProtectionSpace::new(
        HttpScheme::Https,
        "example.com",
        443,
        None,
        AuthAuthority::Origin
    )
    .is_ok());
}

/// The scheme's default port is canonicalized to `0` ("scheme default") so a
/// URL spelling the default and one omitting it produce the same key.
#[test]
fn protection_space_canonicalizes_default_ports() {
    let http = |host: &str, port| {
        ProtectionSpace::new(HttpScheme::Http, host, port, None, AuthAuthority::Origin).unwrap()
    };
    let https = |host: &str, port| {
        ProtectionSpace::new(HttpScheme::Https, host, port, None, AuthAuthority::Origin).unwrap()
    };

    assert_eq!(http("example.com", 80).port(), 0); // http default spelled explicitly
    assert_eq!(http("example.com", 0).port(), 0); // canonical default
    assert_eq!(http("example.com", 8080).port(), 8080);
    assert_eq!(https("example.com", 443).port(), 0); // https default spelled explicitly
    assert_eq!(https("example.com", 0).port(), 0);
    assert_eq!(https("example.com", 8443).port(), 8443);
    // A port that is another scheme's default stays distinct.
    assert_eq!(https("example.com", 80).port(), 80);
}

/// Protection spaces are value-keyed and the authority is part of the key:
/// an origin and a proxy never share a space, and normalization collapses
#[test]
fn protection_space_equality_and_proxy_origin_separation() {
    let space = |host: &str, port, realm, authority| {
        ProtectionSpace::new(HttpScheme::Https, host, port, realm, authority).unwrap()
    };
    let origin = space("example.com", 0, None, AuthAuthority::Origin);

    // Host case and an explicitly spelled default port normalize away.
    assert_eq!(
        origin,
        space("Example.COM", 443, None, AuthAuthority::Origin)
    );

    // Authority is part of the key: origin != proxy for identical host/port.
    let proxy = space("example.com", 0, None, AuthAuthority::Proxy);
    assert_ne!(origin, proxy);
    assert!(!origin.is_proxy());
    assert!(proxy.is_proxy());

    // Realm, host, and port all distinguish spaces.
    assert_ne!(
        origin,
        space("example.com", 0, Some("admin"), AuthAuthority::Origin)
    );
    assert_ne!(
        origin,
        space("other.example", 0, None, AuthAuthority::Origin)
    );
    assert_ne!(
        origin,
        space("example.com", 8443, None, AuthAuthority::Origin)
    );

    let mut keys = HashSet::new();
    keys.insert(origin.clone());
    assert!(keys.contains(&origin));
    assert!(!keys.contains(&proxy));
}

/// The scheme set is closed and exhaustive, and every supported scheme
#[test]
fn auth_scheme_closed_set_is_exhaustive() {
    assert_eq!(AuthScheme::ALL.len(), 4);
    assert_eq!(AuthScheme::ALL[0].as_str(), "Basic");
    assert_eq!(AuthScheme::ALL[1].as_str(), "Digest");
    assert_eq!(AuthScheme::ALL[2].as_str(), "NTLM");
    assert_eq!(AuthScheme::ALL[3].as_str(), "Negotiate");

    for scheme in AuthScheme::ALL {
        let challenge = AuthChallenge::new(
            cid(1),
            ProfileId(1),
            WebViewId(2),
            origin_space("example.com"),
            ChallengeKind::HttpAuth(scheme),
            false,
        )
        .expect("every supported scheme builds a valid challenge");
        assert_eq!(challenge.kind(), &ChallengeKind::HttpAuth(scheme));
    }
}

/// The taxonomy is closed: origin HTTP auth, proxy HTTP auth, client
/// certificates, and the typed unsupported distinctions are all
#[test]
fn challenge_taxonomy_covers_all_closed_distinctions() {
    // Origin HTTP authentication.
    let origin = basic_challenge(1);
    assert_eq!(origin.protection_space().authority(), AuthAuthority::Origin);
    assert!(matches!(
        origin.kind(),
        ChallengeKind::HttpAuth(AuthScheme::Basic)
    ));

    // Proxy HTTP authentication: same kind, proxy authority, http scheme.
    let proxy = AuthChallenge::new(
        cid(2),
        ProfileId(1),
        WebViewId(2),
        ProtectionSpace::new(
            HttpScheme::Http,
            "proxy.corp",
            3128,
            Some("corp"),
            AuthAuthority::Proxy,
        )
        .unwrap(),
        ChallengeKind::HttpAuth(AuthScheme::Digest),
        false,
    )
    .unwrap();
    assert_eq!(proxy.protection_space().authority(), AuthAuthority::Proxy);
    assert!(proxy.protection_space().is_proxy());
    assert!(!origin.protection_space().is_proxy());

    // TLS client-certificate selection.
    let cert = AuthChallenge::new(
        cid(3),
        ProfileId(1),
        WebViewId(2),
        ProtectionSpace::new(
            HttpScheme::Https,
            "bank.example",
            0,
            None,
            AuthAuthority::Origin,
        )
        .unwrap(),
        ChallengeKind::ClientCertificate,
        true,
    )
    .unwrap();
    assert!(matches!(cert.kind(), ChallengeKind::ClientCertificate));

    // Typed unsupported distinctions.
    for unsupported in [
        UnsupportedChallenge::HtmlForm,
        UnsupportedChallenge::ServerTrust,
        UnsupportedChallenge::Other {
            mechanism: "KerberosV4".into(),
        },
    ] {
        let challenge = AuthChallenge::new(
            cid(4),
            ProfileId(1),
            WebViewId(2),
            origin_space("example.com"),
            ChallengeKind::Unsupported(unsupported.clone()),
            false,
        )
        .unwrap();
        assert_eq!(challenge.kind(), &ChallengeKind::Unsupported(unsupported));
    }
}

/// A client-certificate challenge is only legal on an https protection
/// space; fabrication on plain HTTP is rejected at construction.
#[test]
fn client_certificate_requires_an_https_protection_space() {
    let http = ProtectionSpace::new(
        HttpScheme::Http,
        "bank.example",
        0,
        None,
        AuthAuthority::Origin,
    )
    .unwrap();
    assert_eq!(
        AuthChallenge::new(
            cid(1),
            ProfileId(1),
            WebViewId(2),
            http,
            ChallengeKind::ClientCertificate,
            true
        ),
        Err(ChallengeRejection::ClientCertificateWithoutTls)
    );

    let https = ProtectionSpace::new(
        HttpScheme::Https,
        "bank.example",
        0,
        None,
        AuthAuthority::Origin,
    )
    .unwrap();
    assert!(AuthChallenge::new(
        cid(1),
        ProfileId(1),
        WebViewId(2),
        https,
        ChallengeKind::ClientCertificate,
        true
    )
    .is_ok());
}

/// Challenge identity is stable and value-compared: every identity component
#[test]
fn challenge_identity_is_stable_and_value_compared() {
    let a = basic_challenge(7);
    assert_eq!(a.id(), cid(7));
    assert_eq!(a.profile(), ProfileId(1));
    assert_eq!(a.web_view(), WebViewId(2));
    assert_eq!(a.protection_space().host().as_str(), "example.com");
    assert_eq!(a.protection_space().scheme(), HttpScheme::Https);
    assert!(a.user_initiated());

    assert_eq!(a, basic_challenge(7));
    assert_ne!(a, basic_challenge(8)); // id part of identity

    let with = |profile: ProfileId, web_view: WebViewId, user_initiated: bool| {
        AuthChallenge::new(
            cid(7),
            profile,
            web_view,
            origin_space("example.com"),
            ChallengeKind::HttpAuth(AuthScheme::Basic),
            user_initiated,
        )
        .unwrap()
    };
    assert_ne!(a, with(ProfileId(9), WebViewId(2), true));
    assert_ne!(a, with(ProfileId(1), WebViewId(9), true));
    assert_ne!(a, with(ProfileId(1), WebViewId(2), false));
}

#[test]
fn credential_handles_are_opaque_storage_keys() {
    let handle = CredentialHandle::new(42);
    assert_eq!(handle.id(), 42);
    assert_eq!(handle, CredentialHandle::new(42));
    assert_ne!(handle, CredentialHandle::new(43));

    let mut keys = HashSet::new();
    keys.insert(handle);
    assert!(keys.contains(&CredentialHandle::new(42)));
    assert!(!keys.contains(&CredentialHandle::new(43)));
}

#[test]
fn client_identity_handles_are_opaque_and_separate_from_credentials() {
    let identity = ClientIdentityHandle::new(9);
    assert_eq!(identity.id(), 9);
    assert_eq!(identity, ClientIdentityHandle::new(9));
    assert_ne!(identity, ClientIdentityHandle::new(10));

    fn assert_separate<T, U>() {}
    assert_separate::<CredentialHandle, ClientIdentityHandle>();
}

#[test]
fn handle_debug_shapes_are_exact_and_redacted() {
    assert_eq!(
        format!("{:?}", CredentialHandle::new(7)),
        "CredentialHandle(#7)"
    );
    assert_eq!(
        format!("{:?}", ClientIdentityHandle::new(9)),
        "ClientIdentityHandle(#9)"
    );

    for rendered in [
        format!("{:?}", CredentialHandle::new(7)),
        format!("{:?}", ClientIdentityHandle::new(9)),
    ] {
        for needle in [
            "BEGIN",
            "password",
            "secret",
            "private",
            "certificate",
            "-----",
        ] {
            assert!(!rendered.contains(needle), "{rendered} leaks {needle:?}");
        }
    }
}

/// Challenge Debug output carries the non-sensitive identity (host, realm,
/// ids, kind) and never renders secret-shaped material.
#[test]
fn challenge_debug_is_identity_only_and_leaks_nothing_sensitive() {
    let challenge = AuthChallenge::new(
        cid(7),
        ProfileId(1),
        WebViewId(2),
        ProtectionSpace::new(
            HttpScheme::Https,
            "bank.example",
            0,
            Some("bbr demo"),
            AuthAuthority::Origin,
        )
        .unwrap(),
        ChallengeKind::ClientCertificate,
        true,
    )
    .unwrap();

    let rendered = format!("{challenge:?}");
    assert!(rendered.contains("bank.example"));
    assert!(rendered.contains("bbr demo"));
    assert!(rendered.contains("ChallengeId(7)"));
    for needle in ["BEGIN", "password", "secret", "private", "-----"] {
        assert!(!rendered.contains(needle), "{rendered} leaks {needle:?}");
    }
}
