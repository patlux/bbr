//! Serde contract tests (compiled only with the `serde` feature): exact
//! stable wire shapes, serialize/deserialize round trips, validation
//! re-checks on deserialization, and absence of sensitive material on the
//! wire.
//!
//! `bbr-web` has no `serde_json` dependency (manifest changes are out of
//! scope for this lane), so the tests drive a compact token serializer/
//! deserializer pair (see `support/`). The token stream is the exact wire
//! shape: struct and field names, variant names, and scalar values in order.
//! Round trips through the same tokens prove that serialize and deserialize
//! agree.

mod support;

mod lifecycle;

use serde::{Deserialize, Serialize};

use crate::auth_challenges::{
    AuthAuthority, AuthChallenge, AuthScheme, ChallengeId, ChallengeKind, ClientIdentityHandle,
    CredentialHandle, Host, HttpScheme, ProtectionSpace, Realm, UnsupportedChallenge,
};
use crate::{ProfileId, WebViewId};

use support::{Tok, D, S};

/// A canonical Basic origin challenge over https with a realm.
fn canonical_challenge() -> AuthChallenge {
    AuthChallenge::new(
        ChallengeId(7),
        ProfileId(1),
        WebViewId(2),
        ProtectionSpace::new(
            HttpScheme::Https,
            "Example.COM",
            443,
            Some("bbr demo"),
            AuthAuthority::Origin,
        )
        .expect("valid space"),
        ChallengeKind::HttpAuth(AuthScheme::Basic),
        true,
    )
    .expect("valid challenge")
}

fn serialize<T: Serialize>(value: &T) -> Vec<Tok> {
    let s = S::default();
    value
        .serialize(s.clone())
        .expect("serialization cannot fail");
    s.tokens()
}

/// Every wire type is serializable and deserializable.
#[test]
fn wire_types_are_serde_serializable() {
    fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_serde::<ChallengeId>();
    assert_serde::<ProfileId>();
    assert_serde::<WebViewId>();
    assert_serde::<AuthChallenge>();
    assert_serde::<ProtectionSpace>();
    assert_serde::<Host>();
    assert_serde::<Realm>();
    assert_serde::<HttpScheme>();
    assert_serde::<AuthAuthority>();
    assert_serde::<AuthScheme>();
    assert_serde::<ChallengeKind>();
    assert_serde::<UnsupportedChallenge>();
    assert_serde::<CredentialHandle>();
    assert_serde::<ClientIdentityHandle>();
}

/// The canonical challenge's wire shape is exact and stable: struct/field
/// names, variant names, and normalized values in order.
#[test]
fn canonical_challenge_wire_shape_is_exact() {
    assert_eq!(
        serialize(&canonical_challenge()),
        [
            Tok::Struct("AuthChallenge"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(7),
            Tok::Field("profile"),
            Tok::Newtype("ProfileId"),
            Tok::U(1),
            Tok::Field("web_view"),
            Tok::Newtype("WebViewId"),
            Tok::U(2),
            Tok::Field("protection_space"),
            Tok::Struct("ProtectionSpace"),
            Tok::Field("scheme"),
            Tok::Variant("HttpScheme", "Https"),
            Tok::Field("host"),
            Tok::Newtype("Host"),
            Tok::Str("example.com".into()),
            Tok::Field("port"),
            Tok::U(0),
            Tok::Field("realm"),
            Tok::Newtype("Realm"),
            Tok::Str("bbr demo".into()),
            Tok::Field("authority"),
            Tok::Variant("AuthAuthority", "Origin"),
            Tok::End,
            Tok::Field("kind"),
            Tok::NewtypeVariant("ChallengeKind", "HttpAuth"),
            Tok::Variant("AuthScheme", "Basic"),
            Tok::Field("user_initiated"),
            Tok::Bool(true),
            Tok::End,
        ]
    );
}

/// A proxy protection space is a distinct, exact wire shape: proxy authority,
/// http scheme, non-default port, no realm.
#[test]
fn proxy_protection_space_wire_shape_is_exact() {
    let space = ProtectionSpace::new(
        HttpScheme::Http,
        "proxy.corp",
        3128,
        None,
        AuthAuthority::Proxy,
    )
    .unwrap();
    assert_eq!(
        serialize(&space),
        [
            Tok::Struct("ProtectionSpace"),
            Tok::Field("scheme"),
            Tok::Variant("HttpScheme", "Http"),
            Tok::Field("host"),
            Tok::Newtype("Host"),
            Tok::Str("proxy.corp".into()),
            Tok::Field("port"),
            Tok::U(3128),
            Tok::Field("realm"),
            Tok::None,
            Tok::Field("authority"),
            Tok::Variant("AuthAuthority", "Proxy"),
            Tok::End,
        ]
    );
}

/// Handles serialize to their storage key only — no credential or identity
/// material can reach the wire.
#[test]
fn handle_wire_shapes_are_storage_keys_only() {
    assert_eq!(
        serialize(&CredentialHandle::new(42)),
        [
            Tok::Struct("CredentialHandle"),
            Tok::Field("id"),
            Tok::U(42),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&ClientIdentityHandle::new(9)),
        [
            Tok::Struct("ClientIdentityHandle"),
            Tok::Field("id"),
            Tok::U(9),
            Tok::End,
        ]
    );
}

/// Serialize then deserialize through the recorded tokens restores the exact
/// value, for every closed kind and distinction.
#[test]
fn wire_round_trip_restores_every_challenge_kind() {
    let base = |kind: ChallengeKind| {
        AuthChallenge::new(
            ChallengeId(3),
            ProfileId(4),
            WebViewId(5),
            ProtectionSpace::new(
                HttpScheme::Https,
                "roundtrip.example",
                0,
                None,
                AuthAuthority::Origin,
            )
            .unwrap(),
            kind,
            false,
        )
        .unwrap()
    };
    let challenges = [
        base(ChallengeKind::HttpAuth(AuthScheme::Basic)),
        base(ChallengeKind::HttpAuth(AuthScheme::Digest)),
        base(ChallengeKind::HttpAuth(AuthScheme::Ntlm)),
        base(ChallengeKind::HttpAuth(AuthScheme::Negotiate)),
        base(ChallengeKind::ClientCertificate),
        base(ChallengeKind::Unsupported(UnsupportedChallenge::HtmlForm)),
        base(ChallengeKind::Unsupported(
            UnsupportedChallenge::ServerTrust,
        )),
        base(ChallengeKind::Unsupported(UnsupportedChallenge::Other {
            mechanism: "KerberosV4".into(),
        })),
    ];
    for challenge in challenges {
        let tokens = serialize(&challenge);
        let restored = AuthChallenge::deserialize(D::new(&tokens)).expect("round trip");
        assert_eq!(restored, challenge);
    }

    // Handles round-trip too.
    let handle = CredentialHandle::new(42);
    assert_eq!(
        CredentialHandle::deserialize(D::new(&serialize(&handle))).unwrap(),
        handle
    );
    let identity = ClientIdentityHandle::new(9);
    assert_eq!(
        ClientIdentityHandle::deserialize(D::new(&serialize(&identity))).unwrap(),
        identity
    );
}

/// Deserialization re-runs validation: an invalid host or realm on the wire
/// is rejected, never silently accepted.
#[test]
fn deserialize_revalidates_host_and_realm() {
    for raw_host in ["", "   ", "bad host", "a/b"] {
        let tokens = [Tok::Newtype("Host"), Tok::Str(raw_host.into())];
        assert!(
            Host::deserialize(D::new(&tokens)).is_err(),
            "host {raw_host:?} must be rejected on deserialize"
        );
    }
    for raw_realm in ["", "   "] {
        let tokens = [Tok::Newtype("Realm"), Tok::Str(raw_realm.into())];
        assert!(
            Realm::deserialize(D::new(&tokens)).is_err(),
            "realm {raw_realm:?} must be rejected on deserialize"
        );
    }

    // An invalid host inside a full protection space is rejected as well.
    let tokens = [
        Tok::Struct("ProtectionSpace"),
        Tok::Field("scheme"),
        Tok::Variant("HttpScheme", "Https"),
        Tok::Field("host"),
        Tok::Newtype("Host"),
        Tok::Str("exa mple.com".into()),
        Tok::Field("port"),
        Tok::U(443),
        Tok::Field("realm"),
        Tok::None,
        Tok::Field("authority"),
        Tok::Variant("AuthAuthority", "Origin"),
        Tok::End,
    ];
    assert!(ProtectionSpace::deserialize(D::new(&tokens)).is_err());
}

/// Deserialization re-runs normalization: a wire value spelling the
/// scheme's default port (443 on https, 80 on http) or a non-canonical
/// host (`API.Example`) is normalized to the constructor's canonical key,
/// so a spelled default port and an omitted one never diverge on the wire.
#[test]
fn deserialize_canonicalizes_default_port_and_host() {
    let wire = |scheme: HttpScheme, host: &str, port: u16| {
        let scheme_variant = match scheme {
            HttpScheme::Http => "Http",
            HttpScheme::Https => "Https",
        };
        [
            Tok::Struct("ProtectionSpace"),
            Tok::Field("scheme"),
            Tok::Variant("HttpScheme", scheme_variant),
            Tok::Field("host"),
            Tok::Newtype("Host"),
            Tok::Str(host.into()),
            Tok::Field("port"),
            Tok::U(u64::from(port)),
            Tok::Field("realm"),
            Tok::None,
            Tok::Field("authority"),
            Tok::Variant("AuthAuthority", "Origin"),
            Tok::End,
        ]
    };

    // https with an explicitly spelled default port canonicalizes to the
    // constructor's `0` ("scheme default") and equals the canonical space.
    let https_443 =
        ProtectionSpace::deserialize(D::new(&wire(HttpScheme::Https, "API.Example", 443)))
            .expect("https default port canonicalizes on the wire");
    assert_eq!(https_443.port(), 0);
    assert_eq!(
        https_443,
        ProtectionSpace::new(
            HttpScheme::Https,
            "api.example",
            0,
            None,
            AuthAuthority::Origin,
        )
        .unwrap()
    );

    // The http default port canonicalizes the same way.
    let http_80 = ProtectionSpace::deserialize(D::new(&wire(HttpScheme::Http, "api.example", 80)))
        .expect("http default port canonicalizes on the wire");
    assert_eq!(http_80.port(), 0);
    assert_eq!(
        http_80,
        ProtectionSpace::new(
            HttpScheme::Http,
            "api.example",
            0,
            None,
            AuthAuthority::Origin,
        )
        .unwrap()
    );

    // A non-default port survives, so distinct authorities stay distinct.
    let https_8443 =
        ProtectionSpace::deserialize(D::new(&wire(HttpScheme::Https, "api.example", 8443)))
            .expect("non-default port preserved on the wire");
    assert_eq!(https_8443.port(), 8443);
    assert_ne!(https_8443, https_443);
}

/// Deserialization re-runs challenge-level validation: a client-certificate
/// challenge on a plain-http protection space is rejected on the wire.
#[test]
fn deserialize_rejects_client_certificate_without_tls() {
    let tokens = [
        Tok::Struct("AuthChallenge"),
        Tok::Field("id"),
        Tok::Newtype("ChallengeId"),
        Tok::U(1),
        Tok::Field("profile"),
        Tok::Newtype("ProfileId"),
        Tok::U(1),
        Tok::Field("web_view"),
        Tok::Newtype("WebViewId"),
        Tok::U(2),
        Tok::Field("protection_space"),
        Tok::Struct("ProtectionSpace"),
        Tok::Field("scheme"),
        Tok::Variant("HttpScheme", "Http"),
        Tok::Field("host"),
        Tok::Newtype("Host"),
        Tok::Str("bank.example".into()),
        Tok::Field("port"),
        Tok::U(0),
        Tok::Field("realm"),
        Tok::None,
        Tok::Field("authority"),
        Tok::Variant("AuthAuthority", "Origin"),
        Tok::End,
        Tok::Field("kind"),
        Tok::Variant("ChallengeKind", "ClientCertificate"),
        Tok::Field("user_initiated"),
        Tok::Bool(true),
        Tok::End,
    ];
    assert!(AuthChallenge::deserialize(D::new(&tokens)).is_err());
}

/// The wire shape carries only identity data (host, realm, ids, kinds) and
/// never secret-shaped material.
#[test]
fn wire_shape_contains_no_sensitive_material() {
    let tokens = serialize(&canonical_challenge());
    for tok in &tokens {
        if let Tok::Str(value) = tok {
            let lower = value.to_ascii_lowercase();
            for needle in [
                "begin",
                "password",
                "secret",
                "private",
                "-----",
                "certificate",
            ] {
                assert!(
                    !lower.contains(needle),
                    "wire token {value:?} leaks {needle:?}"
                );
            }
        }
    }
    // The non-sensitive identity survives on the wire.
    assert!(tokens
        .iter()
        .any(|t| matches!(t, Tok::Str(v) if v == "example.com")));
    assert!(tokens
        .iter()
        .any(|t| matches!(t, Tok::Str(v) if v == "bbr demo")));
}
