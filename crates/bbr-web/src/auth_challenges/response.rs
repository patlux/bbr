//! Challenge-derived legal response choices.
//!
//! An [`AuthResponse`] is the authority's decision for one challenge. Which
//! choices are legal is **derived from the validated challenge kind**
//! ([`validate_response`]): an opaque credential handle can only answer an
//! HTTP authentication challenge, an opaque client identity only a
//! client-certificate challenge, and unsupported challenges are reject-only
//! where TLS trust is involved. Cancellation is legal everywhere; nothing in
//! the response set can express a trust decision or carry secret bytes.
//!
//! The matrix, as enforced by [`validate_response`]:
//!
//! ```text
//! kind \ response        UseCredential  UseClientIdentity  Continue  UseDefault  Cancel
//! HttpAuth(_)            legal          rejected           legal     legal       legal
//! ClientCertificate      rejected       legal              legal     rejected    legal
//! Unsupported(HtmlForm)  rejected       rejected           legal     legal       legal
//! Unsupported(ServerTrust) rejected    rejected           rejected  rejected    legal
//! Unsupported(Other)     rejected       rejected           legal     rejected    legal
//! ```
//!
//! `ServerTrust` is reject-only: continuing, delegating to an engine
//! default, or supplying credentials would each bypass or muddy TLS trust
//! evaluation, which this contract never performs (owned by the web-error
//! contract). `UseDefault` is limited to kinds with a defined, identity-free
//! engine default: HTTP authentication (for example session-based
//! NTLM/Negotiate) and form-based HTML authentication. Client-certificate
//! selection always requires an explicit identity, and an unmapped mechanism
//! has no defined default.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ChallengeKind, ClientIdentityHandle, CredentialHandle, UnsupportedChallenge};

/// The authority's response choice for one authentication challenge.
///
/// The set is closed and deliberately has no trust-bypass or secret-carrying
/// variant: credentials and client identities are opaque storage-key handles
/// ([`CredentialHandle`] / [`ClientIdentityHandle`], redacted in `Debug`),
/// and every other choice injects nothing into the connection.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthResponse {
    /// Use the profile's stored HTTP credential for this challenge. Legal
    /// only for [`ChallengeKind::HttpAuth`]; answering a client-certificate
    /// or unsupported challenge with a credential is rejected whole.
    UseCredential(CredentialHandle),
    /// Use the profile's stored client identity for this challenge. Legal
    /// only for [`ChallengeKind::ClientCertificate`]; answering an HTTP
    /// password or unsupported challenge with an identity is rejected whole.
    UseClientIdentity(ClientIdentityHandle),
    /// Continue the request without providing any credential or identity.
    /// Legal for every kind except
    /// [`UnsupportedChallenge::ServerTrust`], where "continuing" would
    /// bypass TLS trust evaluation.
    Continue,
    /// Let the engine perform its default handling for the challenge. Legal
    /// only where the engine's default is defined and cannot select a client
    /// identity silently: HTTP authentication (session-based NTLM/Negotiate)
    /// and form-based HTML authentication. Not legal for client-certificate
    /// selection (an identity must be chosen explicitly) or an unmapped
    /// mechanism (no defined default).
    UseDefault,
    /// Reject/cancel the challenge. Legal for every kind; the only legal
    /// response to a server-trust challenge.
    Cancel,
}

/// Why an [`AuthResponse`] is not a legal choice for a [`ChallengeKind`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseRejection {
    /// A credential handle answered a challenge that is not HTTP
    /// authentication. Rejected whole: client identities and unsupported
    /// challenges cannot be answered with a stored HTTP credential.
    CrossKindCredential { kind: ChallengeKind },
    /// A client identity answered a challenge that is not client-certificate
    /// selection. Rejected whole: HTTP password and unsupported challenges
    /// cannot be answered with a stored TLS identity.
    CrossKindIdentity { kind: ChallengeKind },
    /// Any response except [`AuthResponse::Cancel`] on a server-trust
    /// challenge. Continuing, delegating to an engine default, or supplying
    /// credentials would each bypass or muddy TLS trust evaluation, which
    /// this contract never performs.
    ServerTrustMustBeRejected,
    /// [`AuthResponse::UseDefault`] on a kind with no defined engine default,
    /// or whose default would silently select a client identity.
    DefaultNotLegal { kind: ChallengeKind },
}

/// Validate that `response` is a legal choice for `kind`.
///
/// This is the response-legality rule the lifecycle enforces on every
/// [`super::AuthEvent::Respond`]: cross-kind credential/identity responses
/// are rejected whole, server-trust challenges accept only cancellation, and
/// default handling is limited to kinds with a defined, identity-free engine
/// default. Cancellation is legal for every kind.
pub fn validate_response(
    kind: &ChallengeKind,
    response: &AuthResponse,
) -> Result<(), ResponseRejection> {
    use AuthResponse::{Cancel, Continue, UseClientIdentity, UseCredential, UseDefault};

    // Server trust is reject-only. This arm runs before the cross-kind
    // checks because a trust challenge is never answerable by any
    // credential/identity; the auth authority makes no TLS decisions.
    if matches!(
        kind,
        ChallengeKind::Unsupported(UnsupportedChallenge::ServerTrust)
    ) {
        return match response {
            Cancel => Ok(()),
            _ => Err(ResponseRejection::ServerTrustMustBeRejected),
        };
    }

    match response {
        UseCredential(_) if matches!(kind, ChallengeKind::HttpAuth(_)) => Ok(()),
        UseCredential(_) => Err(ResponseRejection::CrossKindCredential { kind: kind.clone() }),
        UseClientIdentity(_) if matches!(kind, ChallengeKind::ClientCertificate) => Ok(()),
        UseClientIdentity(_) => Err(ResponseRejection::CrossKindIdentity { kind: kind.clone() }),
        // Continue is legal everywhere outside server trust: the request
        // proceeds without injecting anything.
        Continue => Ok(()),
        UseDefault if matches!(kind, ChallengeKind::HttpAuth(_)) => Ok(()),
        UseDefault
            if matches!(
                kind,
                ChallengeKind::Unsupported(UnsupportedChallenge::HtmlForm)
            ) =>
        {
            Ok(())
        }
        UseDefault => Err(ResponseRejection::DefaultNotLegal { kind: kind.clone() }),
        Cancel => Ok(()),
    }
}
