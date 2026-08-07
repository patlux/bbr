//! Response-legality tests: the exhaustive kind × response matrix and the
//! focused rejection-variant assertions (cross-kind handles, reject-only
//! server trust, limited default handling, per-scheme uniformity).

use super::support::*;
use crate::auth_challenges::{
    validate_response, AuthResponse, AuthScheme, ChallengeKind, ResponseRejection,
    UnsupportedChallenge,
};

/// Exhaustive kind × response matrix: every one of the 5 kinds crossed with
/// every one of the 5 responses yields exactly the hand-derived legality, so
/// no legal choice is missing and no illegal one slips through.
#[test]
fn response_legality_matrix_is_exhaustive() {
    for (kind_name, kind) in kinds() {
        for (response_name, response) in responses() {
            let expected = expected_legal(kind_name, response_name);
            let actual = validate_response(&kind, &response).is_ok();
            assert_eq!(
                actual, expected,
                "kind {kind_name} × response {response_name}"
            );
        }
    }
}

/// Cross-kind credential and identity responses are rejected whole, with the
/// offending kind reported.
#[test]
fn cross_kind_credential_and_identity_reject_whole() {
    // A credential can never answer a client-certificate or unsupported
    // challenge.
    let cert = ChallengeKind::ClientCertificate;
    assert_eq!(
        validate_response(&cert, &AuthResponse::UseCredential(handle(7))),
        Err(ResponseRejection::CrossKindCredential { kind: cert.clone() })
    );
    let form = ChallengeKind::Unsupported(UnsupportedChallenge::HtmlForm);
    assert_eq!(
        validate_response(&form, &AuthResponse::UseCredential(handle(7))),
        Err(ResponseRejection::CrossKindCredential { kind: form.clone() })
    );
    let other = ChallengeKind::Unsupported(UnsupportedChallenge::Other {
        mechanism: "X".into(),
    });
    assert_eq!(
        validate_response(&other, &AuthResponse::UseCredential(handle(7))),
        Err(ResponseRejection::CrossKindCredential {
            kind: other.clone()
        })
    );

    // A client identity can never answer an HTTP password or unsupported
    // challenge.
    let http = ChallengeKind::HttpAuth(AuthScheme::Digest);
    assert_eq!(
        validate_response(&http, &AuthResponse::UseClientIdentity(identity(9))),
        Err(ResponseRejection::CrossKindIdentity { kind: http.clone() })
    );
    assert_eq!(
        validate_response(&cert, &AuthResponse::UseClientIdentity(identity(9))),
        Ok(())
    );
    assert_eq!(
        validate_response(&form, &AuthResponse::UseClientIdentity(identity(9))),
        Err(ResponseRejection::CrossKindIdentity { kind: form.clone() })
    );
}

/// Server-trust challenges are reject-only: continuing, delegating to an
/// engine default, or supplying any credential/identity is a trust decision
/// this contract never makes. Only cancellation is legal.
#[test]
fn server_trust_is_reject_only() {
    let trust = ChallengeKind::Unsupported(UnsupportedChallenge::ServerTrust);
    assert_eq!(validate_response(&trust, &AuthResponse::Cancel), Ok(()));
    for response in [
        AuthResponse::UseCredential(handle(7)),
        AuthResponse::UseClientIdentity(identity(9)),
        AuthResponse::Continue,
        AuthResponse::UseDefault,
    ] {
        assert_eq!(
            validate_response(&trust, &response),
            Err(ResponseRejection::ServerTrustMustBeRejected),
            "server trust must reject {response:?}"
        );
    }
}

/// Default handling is limited to kinds with a defined, identity-free engine
/// default: HTTP authentication and form-based HTML authentication.
/// Client-certificate selection must be explicit, and an unmapped mechanism
/// has no defined default.
#[test]
fn default_handling_is_limited_to_defined_kinds() {
    for kind in [
        ChallengeKind::HttpAuth(AuthScheme::Basic),
        ChallengeKind::Unsupported(UnsupportedChallenge::HtmlForm),
    ] {
        assert!(
            validate_response(&kind, &AuthResponse::UseDefault).is_ok(),
            "{kind:?} must accept default handling"
        );
    }
    for kind in [
        ChallengeKind::ClientCertificate,
        ChallengeKind::Unsupported(UnsupportedChallenge::Other {
            mechanism: "KerberosV4".into(),
        }),
    ] {
        assert_eq!(
            validate_response(&kind, &AuthResponse::UseDefault),
            Err(ResponseRejection::DefaultNotLegal { kind: kind.clone() }),
            "{kind:?} must reject default handling"
        );
    }
    // Server trust rejects default handling via its stronger reject-only
    // rule (see `server_trust_is_reject_only`).
}

/// Every supported HTTP scheme shares the same response legality: credentials,
/// continue, and default handling are legal; a client identity is not.
#[test]
fn every_http_scheme_shares_the_same_response_legality() {
    for scheme in AuthScheme::ALL {
        let kind = ChallengeKind::HttpAuth(scheme);
        assert!(validate_response(&kind, &AuthResponse::UseCredential(handle(1))).is_ok());
        assert!(validate_response(&kind, &AuthResponse::Continue).is_ok());
        assert!(validate_response(&kind, &AuthResponse::UseDefault).is_ok());
        assert!(validate_response(&kind, &AuthResponse::Cancel).is_ok());
        assert_eq!(
            validate_response(&kind, &AuthResponse::UseClientIdentity(identity(2))),
            Err(ResponseRejection::CrossKindIdentity { kind: kind.clone() })
        );
    }
}
