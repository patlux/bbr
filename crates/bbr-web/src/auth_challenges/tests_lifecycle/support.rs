//! Shared fixtures for the response-legality and lifecycle contract tests:
//! canonical challenges, the kind/response/state/event enumerations, and the
//! hand-derived expectation tables used by the exhaustive matrices.

use crate::auth_challenges::{
    AuthAuthority, AuthChallenge, AuthEvent, AuthFailure, AuthResponse, AuthScheme, AuthState,
    ChallengeId, ChallengeKind, ClientIdentityHandle, CredentialHandle, EventRejection, HttpScheme,
    ProtectionSpace, UnsupportedChallenge,
};
use crate::{ProfileId, WebViewId};

pub(super) fn id(n: u64) -> ChallengeId {
    ChallengeId(n)
}

pub(super) fn handle(n: u64) -> CredentialHandle {
    CredentialHandle::new(n)
}

pub(super) fn identity(n: u64) -> ClientIdentityHandle {
    ClientIdentityHandle::new(n)
}

/// A canonical challenge with `id == ChallengeId(1)` on an https origin
/// space, carrying `kind`.
pub(super) fn challenge(kind: ChallengeKind) -> AuthChallenge {
    AuthChallenge::new(
        id(1),
        ProfileId(1),
        WebViewId(2),
        ProtectionSpace::new(
            HttpScheme::Https,
            "example.com",
            0,
            None,
            AuthAuthority::Origin,
        )
        .expect("valid space"),
        kind,
        true,
    )
    .expect("valid challenge")
}

/// A Basic origin challenge with `id == ChallengeId(1)`, used for the
/// lifecycle matrix (whose `Respond` event carries a credential response).
pub(super) fn basic_challenge() -> AuthChallenge {
    challenge(ChallengeKind::HttpAuth(AuthScheme::Basic))
}

/// Every closed challenge kind with a stable label for the matrix.
pub(super) fn kinds() -> Vec<(&'static str, ChallengeKind)> {
    vec![
        ("HttpAuth-Basic", ChallengeKind::HttpAuth(AuthScheme::Basic)),
        ("ClientCertificate", ChallengeKind::ClientCertificate),
        (
            "Unsupported-HtmlForm",
            ChallengeKind::Unsupported(UnsupportedChallenge::HtmlForm),
        ),
        (
            "Unsupported-ServerTrust",
            ChallengeKind::Unsupported(UnsupportedChallenge::ServerTrust),
        ),
        (
            "Unsupported-Other",
            ChallengeKind::Unsupported(UnsupportedChallenge::Other {
                mechanism: "KerberosV4".into(),
            }),
        ),
    ]
}

/// Every closed response choice with a stable label for the matrix.
pub(super) fn responses() -> Vec<(&'static str, AuthResponse)> {
    vec![
        ("UseCredential", AuthResponse::UseCredential(handle(7))),
        (
            "UseClientIdentity",
            AuthResponse::UseClientIdentity(identity(9)),
        ),
        ("Continue", AuthResponse::Continue),
        ("UseDefault", AuthResponse::UseDefault),
        ("Cancel", AuthResponse::Cancel),
    ]
}

/// The hand-derived legality of one (kind, response) pair; see
/// `response.rs` for the documented matrix. This function only pins which
/// pairs are legal and which are rejected; the rejection variants are
/// asserted by the focused tests in `response_matrix.rs`.
pub(super) fn expected_legal(kind: &str, response: &str) -> bool {
    match (kind, response) {
        ("HttpAuth-Basic", "UseCredential" | "Continue" | "UseDefault" | "Cancel") => true,
        ("HttpAuth-Basic", "UseClientIdentity") => false,
        ("ClientCertificate", "UseClientIdentity" | "Continue" | "Cancel") => true,
        ("ClientCertificate", "UseCredential" | "UseDefault") => false,
        ("Unsupported-HtmlForm", "Continue" | "UseDefault" | "Cancel") => true,
        ("Unsupported-HtmlForm", "UseCredential" | "UseClientIdentity") => false,
        ("Unsupported-ServerTrust", "Cancel") => true,
        ("Unsupported-ServerTrust", _) => false,
        ("Unsupported-Other", "Continue" | "Cancel") => true,
        ("Unsupported-Other", "UseCredential" | "UseClientIdentity" | "UseDefault") => false,
        (k, r) => panic!("missing matrix expectation for {k} × {r}"),
    }
}

/// Every lifecycle state with a stable label for the matrix.
pub(super) fn states() -> Vec<(&'static str, AuthState)> {
    vec![
        ("Pending", AuthState::Pending),
        ("ResponseRequested", AuthState::ResponseRequested),
        (
            "Resolved",
            AuthState::Resolved {
                response: AuthResponse::UseCredential(handle(42)),
            },
        ),
        (
            "Failed",
            AuthState::Failed {
                failure: AuthFailure::EngineRejected,
            },
        ),
        ("Cancelled", AuthState::Cancelled),
        ("Superseded", AuthState::Superseded { by: id(2) }),
    ]
}

/// Every lifecycle event (all routed to `id == ChallengeId(1)`, matching the
/// canonical challenge) with a stable label for the matrix.
pub(super) fn events() -> Vec<(&'static str, AuthEvent)> {
    vec![
        ("RequestResponse", AuthEvent::RequestResponse { id: id(1) }),
        (
            "Respond",
            AuthEvent::Respond {
                id: id(1),
                response: AuthResponse::UseCredential(handle(42)),
            },
        ),
        (
            "Failed",
            AuthEvent::Failed {
                id: id(1),
                failure: AuthFailure::EngineRejected,
            },
        ),
        ("Cancel", AuthEvent::Cancel { id: id(1) }),
        (
            "Supersede",
            AuthEvent::Supersede {
                id: id(1),
                by: id(2),
            },
        ),
    ]
}

/// The hand-derived expectation for one (state, event) pair of the matrix
/// against a Basic origin challenge whose legal `Respond` carries a
/// credential. Exact payloads (resolved response, superseding id, failure
/// reason) are asserted by the focused tests in `lifecycle.rs`; this
/// function only pins which transitions are legal and with which rejection
/// class the illegal ones die.
pub(super) fn expected_outcome(state: &str, event: &str) -> &'static str {
    match (state, event) {
        ("Pending", "RequestResponse") => "Ok ResponseRequested",
        ("Pending", "Respond") => "Err NotRequested",
        ("Pending", "Failed") => "Ok Failed",
        ("Pending", "Cancel") => "Ok Cancelled",
        ("Pending", "Supersede") => "Ok Superseded",
        ("ResponseRequested", "RequestResponse") => "Err AlreadyRequested",
        ("ResponseRequested", "Respond") => "Ok Resolved",
        ("ResponseRequested", "Failed") => "Ok Failed",
        ("ResponseRequested", "Cancel") => "Ok Cancelled",
        ("ResponseRequested", "Supersede") => "Ok Superseded",
        ("Resolved" | "Failed" | "Cancelled" | "Superseded", _) => "Err AlreadyTerminal",
        (s, e) => panic!("missing matrix expectation for {s} × {e}"),
    }
}

pub(super) fn classify(result: Result<AuthState, EventRejection>) -> &'static str {
    match result {
        Ok(AuthState::Pending) => "Ok Pending",
        Ok(AuthState::ResponseRequested) => "Ok ResponseRequested",
        Ok(AuthState::Resolved { .. }) => "Ok Resolved",
        Ok(AuthState::Failed { .. }) => "Ok Failed",
        Ok(AuthState::Cancelled) => "Ok Cancelled",
        Ok(AuthState::Superseded { .. }) => "Ok Superseded",
        Err(EventRejection::MismatchedId { .. }) => "Err MismatchedId",
        Err(EventRejection::NotRequested { .. }) => "Err NotRequested",
        Err(EventRejection::AlreadyRequested) => "Err AlreadyRequested",
        Err(EventRejection::IllegalResponse { .. }) => "Err IllegalResponse",
        Err(EventRejection::SelfSupersede { .. }) => "Err SelfSupersede",
        Err(EventRejection::AlreadyTerminal { .. }) => "Err AlreadyTerminal",
    }
}
