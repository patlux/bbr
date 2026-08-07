//! # Portable authentication challenge contract
//!
//! The platform-neutral contract for Phase 8 HTTP authentication and
//! client-certificate challenges (see
//! [the auth-challenges task packet](../../../docs/roadmap/tasks/auth-challenges-contract.md)).
//! It fixes the types and rules both sides of the seam must agree on before
//! any WebKit delegate or credential UI exists.
//!
//! Like [`crate::WebEngine`], the module is **portable**: no frameworks, no
//! GPUI, no `unsafe`, no secret bytes, and no new dependencies (only `std`
//! plus the crate's optional `serde` feature). Sensitive values do not exist
//! in the portable types: credentials and client identities are **opaque
//! handles** carrying only a storage key, and their `Debug` output is
//! intentionally redacted by construction.
//!
//! ## Challenge identity
//!
//! Every [`AuthChallenge`] carries a stable [`ChallengeId`], the owning
//! [`crate::ProfileId`] and [`crate::WebViewId`], a validated
//! [`ProtectionSpace`], a closed [`ChallengeKind`], and a user-gesture flag.
//! Identity is value-based: two challenges with the same id, profile, web
//! view, protection space, kind, and gesture are equal; any difference makes
//! them distinct.
//!
//! ## Protection space
//!
//! A [`ProtectionSpace`] names the authority (origin or proxy) plus the
//! normalized scheme, host, port, and realm. Construction validates and
//! normalizes, so an empty or ambiguous key is unrepresentable: hosts are
//! trimmed and lowercased, the scheme's default port is canonicalized to `0`,
//! realms are trimmed and length-bounded, and the authority is part of the
//! key so an origin space and a proxy space never collide.
//!
//! ## Closed taxonomy
//!
//! [`ChallengeKind`] is closed: HTTP authentication against the protection
//! space's authority (one of the closed [`AuthScheme`] set), TLS
//! client-certificate selection (legal only on an `https` space), or a typed
//! [`UnsupportedChallenge`]. Unsupported challenges are distinct types —
//! form-based HTML authentication, TLS server-trust (owned by the web-error
//! contract), or an unmapped mechanism — never an opaque native payload or an
//! escape hatch.
//!
//! ## Handles
//!
//! [`CredentialHandle`] and [`ClientIdentityHandle`] are opaque keys into the
//! profile's later credential store / native identity lookup. No username,
//! password, certificate, or private-key bytes cross the portable boundary,
//! and `Debug` renders a redacted one-line shape.
//!
//! ## Responses
//!
//! [`AuthResponse`] is the closed set of legal decisions for a challenge,
//! derived from its validated kind ([`validate_response`]): an opaque
//! credential handle answers HTTP authentication, an opaque client identity
//! answers client-certificate selection, continuing without credentials is
//! legal everywhere except server-trust challenges, and engine-default
//! handling is legal only where it cannot silently pick a client identity.
//! Cancellation is always legal. Nothing in the response set can express a
//! TLS trust decision or carry secret bytes.
//!
//! ## Lifecycle
//!
//! [`AuthState`] runs one immutable decision cycle per [`ChallengeId`]:
//! `Pending → ResponseRequested → Resolved` via a legal response, with
//! `Failed`, `Cancelled`, and `Superseded` as terminal outcomes. Events are
//! applied by [`apply_event`], which rejects mismatched ids, early responses
//! before a request, duplicate requests, illegal responses, and every late
//! callback on a terminal state whole — cancellation and supersession can
//! never become credential use, and the authority never makes TLS trust
//! decisions.
//!
//! ## Non-goals (this packet)
//!
//! Native delegates, credential UI/storage, client-certificate enumeration,
//! trust evaluation, and E2E remain later adapter work. The contract decides
//! and records; it never touches secret bytes.

mod lifecycle;
mod protection_space;
mod response;
mod taxonomy;

pub use lifecycle::{apply_event, AuthEvent, AuthFailure, AuthState, EventRejection};
pub use protection_space::{
    AuthAuthority, Host, HttpScheme, ProtectionSpace, ProtectionSpaceError, Realm, MAX_HOST_LEN,
    MAX_REALM_LEN,
};
pub use response::{validate_response, AuthResponse, ResponseRejection};
pub use taxonomy::{AuthScheme, ChallengeKind, UnsupportedChallenge};

use std::fmt;

use crate::{ProfileId, WebViewId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Stable identifier of one authentication challenge request. A request id
/// is never reused once its challenge left the pending state.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChallengeId(pub u64);

/// Opaque handle to an HTTP credential owned by the profile's credential
/// store.
///
/// The handle is a storage key only: no username, password, or other secret
/// bytes exist in the portable types, and its [`Debug`] shape is redacted by
/// construction. The native adapter maps the key back to the real credential
/// at the boundary; the portable contract never sees secret material.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CredentialHandle {
    id: u64,
}

impl CredentialHandle {
    /// Wrap a store-issued key. Keys are assigned by the credential store;
    /// all other callers must treat the handle as opaque.
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// The store key this handle refers to. Not secret; exposed so the
    /// adapter can look up the underlying credential.
    pub fn id(self) -> u64 {
        self.id
    }
}

impl fmt::Debug for CredentialHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Intentionally redacted one-line shape: the handle contains only a
        // storage key and never renders credential material.
        write!(f, "CredentialHandle(#{})", self.id)
    }
}

/// Opaque handle to a client identity (certificate) owned by the profile's
/// identity store.
///
/// Like [`CredentialHandle`], the handle is a storage key only: no
/// certificate, private-key, or other secret bytes exist in the portable
/// types, and [`Debug`] is redacted by construction. The native adapter maps
/// the key back to the real `SecIdentity` at the boundary.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientIdentityHandle {
    id: u64,
}

impl ClientIdentityHandle {
    /// Wrap a store-issued key. Keys are assigned by the identity store; all
    /// other callers must treat the handle as opaque.
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// The store key this handle refers to. Not secret; exposed so the
    /// adapter can look up the underlying identity.
    pub fn id(self) -> u64 {
        self.id
    }
}

impl fmt::Debug for ClientIdentityHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Intentionally redacted one-line shape: the handle contains only a
        // storage key and never renders certificate/private-key material.
        write!(f, "ClientIdentityHandle(#{})", self.id)
    }
}

/// Why an [`AuthChallenge`] could not be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeRejection {
    /// A client-certificate challenge requires an `https` protection space;
    /// adapters must not fabricate one on plain HTTP.
    ClientCertificateWithoutTls,
}

impl fmt::Display for ChallengeRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChallengeRejection::ClientCertificateWithoutTls => {
                write!(
                    f,
                    "client-certificate challenges require an https protection space"
                )
            }
        }
    }
}

impl std::error::Error for ChallengeRejection {}

/// One authentication challenge request with its full identity.
///
/// Constructed via [`AuthChallenge::new`], which validates kind/space
/// consistency (a client-certificate challenge must sit on an `https` space).
/// Deserialization re-runs the same checks, so an inconsistent challenge can
/// never enter the contract through the wire.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthChallenge {
    id: ChallengeId,
    profile: ProfileId,
    web_view: WebViewId,
    protection_space: ProtectionSpace,
    kind: ChallengeKind,
    user_initiated: bool,
}

impl AuthChallenge {
    /// Build a challenge after validating kind/space consistency.
    ///
    /// The protection space is already validated by its own constructor; this
    /// step enforces the cross-field rule that client-certificate challenges
    /// require an `https` space.
    pub fn new(
        id: ChallengeId,
        profile: ProfileId,
        web_view: WebViewId,
        protection_space: ProtectionSpace,
        kind: ChallengeKind,
        user_initiated: bool,
    ) -> Result<Self, ChallengeRejection> {
        if matches!(kind, ChallengeKind::ClientCertificate)
            && protection_space.scheme() != HttpScheme::Https
        {
            return Err(ChallengeRejection::ClientCertificateWithoutTls);
        }
        Ok(Self {
            id,
            profile,
            web_view,
            protection_space,
            kind,
            user_initiated,
        })
    }

    /// The stable request id.
    pub fn id(&self) -> ChallengeId {
        self.id
    }

    /// The owning profile.
    pub fn profile(&self) -> ProfileId {
        self.profile
    }

    /// The owning web view / tab content slot.
    pub fn web_view(&self) -> WebViewId {
        self.web_view
    }

    /// The validated protection space the challenge belongs to.
    pub fn protection_space(&self) -> &ProtectionSpace {
        &self.protection_space
    }

    /// The closed challenge kind.
    pub fn kind(&self) -> &ChallengeKind {
        &self.kind
    }

    /// Whether the challenge stems from a user gesture (as opposed to a
    /// background load).
    pub fn user_initiated(&self) -> bool {
        self.user_initiated
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AuthChallenge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawChallenge {
            id: ChallengeId,
            profile: ProfileId,
            web_view: WebViewId,
            protection_space: ProtectionSpace,
            kind: ChallengeKind,
            user_initiated: bool,
        }

        let raw = RawChallenge::deserialize(deserializer)?;
        AuthChallenge::new(
            raw.id,
            raw.profile,
            raw.web_view,
            raw.protection_space,
            raw.kind,
            raw.user_initiated,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests_contract;
#[cfg(test)]
mod tests_lifecycle;

#[cfg(all(test, feature = "serde"))]
mod tests_serde;
