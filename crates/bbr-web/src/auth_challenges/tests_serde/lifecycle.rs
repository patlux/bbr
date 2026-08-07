//! Serde contract tests for the response and lifecycle wire types: exact
//! stable wire shapes for [`AuthResponse`], [`AuthState`], [`AuthEvent`], and
//! [`AuthFailure`], round trips through the recorded token stream, and
//! absence of sensitive material on the wire (handles serialize to their
//! storage key only).

use serde::{Deserialize, Serialize};

use crate::auth_challenges::{
    AuthEvent, AuthFailure, AuthResponse, AuthState, ChallengeId, ClientIdentityHandle,
    CredentialHandle,
};

use super::support::{Tok, D, S};

fn serialize<T: Serialize>(value: &T) -> Vec<Tok> {
    let s = S::default();
    value
        .serialize(s.clone())
        .expect("serialization cannot fail");
    s.tokens()
}

/// Every lifecycle wire type is serializable and deserializable.
#[test]
fn lifecycle_wire_types_are_serde_serializable() {
    fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
    assert_serde::<AuthResponse>();
    assert_serde::<AuthState>();
    assert_serde::<AuthEvent>();
    assert_serde::<AuthFailure>();
}

/// Response wire shapes are exact: credential/identity choices carry the
/// opaque storage-key handle struct, unit choices carry only the variant
/// name. No secret material can reach the wire.
#[test]
fn auth_response_wire_shapes_are_exact() {
    assert_eq!(
        serialize(&AuthResponse::UseCredential(CredentialHandle::new(42))),
        [
            Tok::NewtypeVariant("AuthResponse", "UseCredential"),
            Tok::Struct("CredentialHandle"),
            Tok::Field("id"),
            Tok::U(42),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthResponse::UseClientIdentity(ClientIdentityHandle::new(
            9
        ))),
        [
            Tok::NewtypeVariant("AuthResponse", "UseClientIdentity"),
            Tok::Struct("ClientIdentityHandle"),
            Tok::Field("id"),
            Tok::U(9),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthResponse::Continue),
        [Tok::Variant("AuthResponse", "Continue")]
    );
    assert_eq!(
        serialize(&AuthResponse::UseDefault),
        [Tok::Variant("AuthResponse", "UseDefault")]
    );
    assert_eq!(
        serialize(&AuthResponse::Cancel),
        [Tok::Variant("AuthResponse", "Cancel")]
    );
}

/// State wire shapes are exact: unit states carry only the variant name;
/// `Resolved`, `Failed`, and `Superseded` carry their payloads as declared.
#[test]
fn auth_state_wire_shapes_are_exact() {
    assert_eq!(
        serialize(&AuthState::Pending),
        [Tok::Variant("AuthState", "Pending")]
    );
    assert_eq!(
        serialize(&AuthState::ResponseRequested),
        [Tok::Variant("AuthState", "ResponseRequested")]
    );
    assert_eq!(
        serialize(&AuthState::Cancelled),
        [Tok::Variant("AuthState", "Cancelled")]
    );
    assert_eq!(
        serialize(&AuthState::Resolved {
            response: AuthResponse::UseCredential(CredentialHandle::new(42)),
        }),
        [
            Tok::StructVariant("AuthState", "Resolved"),
            Tok::Field("response"),
            Tok::NewtypeVariant("AuthResponse", "UseCredential"),
            Tok::Struct("CredentialHandle"),
            Tok::Field("id"),
            Tok::U(42),
            Tok::End,
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthState::Failed {
            failure: AuthFailure::EngineRejected,
        }),
        [
            Tok::StructVariant("AuthState", "Failed"),
            Tok::Field("failure"),
            Tok::Variant("AuthFailure", "EngineRejected"),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthState::Superseded { by: ChallengeId(2) }),
        [
            Tok::StructVariant("AuthState", "Superseded"),
            Tok::Field("by"),
            Tok::Newtype("ChallengeId"),
            Tok::U(2),
            Tok::End,
        ]
    );
}

/// Event wire shapes are exact, including the nested response and the
/// superseding id.
#[test]
fn auth_event_wire_shapes_are_exact() {
    assert_eq!(
        serialize(&AuthEvent::RequestResponse { id: ChallengeId(1) }),
        [
            Tok::StructVariant("AuthEvent", "RequestResponse"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(1),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthEvent::Respond {
            id: ChallengeId(1),
            response: AuthResponse::Continue,
        }),
        [
            Tok::StructVariant("AuthEvent", "Respond"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(1),
            Tok::Field("response"),
            Tok::Variant("AuthResponse", "Continue"),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthEvent::Failed {
            id: ChallengeId(1),
            failure: AuthFailure::Connection,
        }),
        [
            Tok::StructVariant("AuthEvent", "Failed"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(1),
            Tok::Field("failure"),
            Tok::Variant("AuthFailure", "Connection"),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthEvent::Cancel { id: ChallengeId(1) }),
        [
            Tok::StructVariant("AuthEvent", "Cancel"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(1),
            Tok::End,
        ]
    );
    assert_eq!(
        serialize(&AuthEvent::Supersede {
            id: ChallengeId(1),
            by: ChallengeId(2),
        }),
        [
            Tok::StructVariant("AuthEvent", "Supersede"),
            Tok::Field("id"),
            Tok::Newtype("ChallengeId"),
            Tok::U(1),
            Tok::Field("by"),
            Tok::Newtype("ChallengeId"),
            Tok::U(2),
            Tok::End,
        ]
    );
}

/// Serialize then deserialize through the recorded tokens restores the exact
/// value for every state, response, event, and failure distinction.
#[test]
fn lifecycle_wire_round_trip_restores_values() {
    let states = [
        AuthState::Pending,
        AuthState::ResponseRequested,
        AuthState::Resolved {
            response: AuthResponse::UseClientIdentity(ClientIdentityHandle::new(9)),
        },
        AuthState::Failed {
            failure: AuthFailure::Unknown,
        },
        AuthState::Cancelled,
        AuthState::Superseded { by: ChallengeId(2) },
    ];
    for state in states {
        let tokens = serialize(&state);
        let restored = AuthState::deserialize(D::new(&tokens)).expect("state round trip");
        assert_eq!(restored, state);
    }

    let responses = [
        AuthResponse::UseCredential(CredentialHandle::new(42)),
        AuthResponse::UseClientIdentity(ClientIdentityHandle::new(9)),
        AuthResponse::Continue,
        AuthResponse::UseDefault,
        AuthResponse::Cancel,
    ];
    for response in responses {
        let tokens = serialize(&response);
        let restored = AuthResponse::deserialize(D::new(&tokens)).expect("response round trip");
        assert_eq!(restored, response);
    }

    let events = [
        AuthEvent::RequestResponse { id: ChallengeId(1) },
        AuthEvent::Respond {
            id: ChallengeId(1),
            response: AuthResponse::UseDefault,
        },
        AuthEvent::Failed {
            id: ChallengeId(1),
            failure: AuthFailure::EngineRejected,
        },
        AuthEvent::Cancel { id: ChallengeId(1) },
        AuthEvent::Supersede {
            id: ChallengeId(1),
            by: ChallengeId(2),
        },
    ];
    for event in events {
        let tokens = serialize(&event);
        let restored = AuthEvent::deserialize(D::new(&tokens)).expect("event round trip");
        assert_eq!(restored, event);
    }

    for failure in [
        AuthFailure::EngineRejected,
        AuthFailure::Connection,
        AuthFailure::Unknown,
    ] {
        let tokens = serialize(&failure);
        let restored = AuthFailure::deserialize(D::new(&tokens)).expect("failure round trip");
        assert_eq!(restored, failure);
    }
}

/// The lifecycle wire shape carries only identity data (ids, variant names,
/// storage keys) and never secret-shaped material.
#[test]
fn lifecycle_wire_shapes_contain_no_sensitive_material() {
    let tokens = serialize(&AuthState::Resolved {
        response: AuthResponse::UseCredential(CredentialHandle::new(7)),
    });
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
    // The storage key survives on the wire (not secret); the handle type is
    // the only credential-shaped marker and carries no material.
    assert!(tokens.iter().any(|t| matches!(t, Tok::U(7))));
    assert!(tokens.iter().any(|t| matches!(t, Tok::Field("id"))));
}
