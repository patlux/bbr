//! Deterministic per-challenge lifecycle.
//!
//! One [`AuthChallenge`] id runs exactly one decision cycle. The lifecycle
//! is pure and immutable-transitioning: [`apply_event`] takes the current
//! [`AuthState`] and an [`AuthEvent`] and returns the next state or a typed
//! rejection, never mutating in place.
//!
//! ```text
//! Pending ──RequestResponse──► ResponseRequested ──Respond(legal)──► Resolved{response}
//!    │                              │                                   (terminal)
//!    ├─Failed{f}───► Failed{f}     ├─Failed{f}───► Failed{f}
//!    ├─Cancel───► Cancelled         ├─Cancel───► Cancelled
//!    └─Supersede{by}─► Superseded   └─Supersede{by}─► Superseded
//! ```
//!
//! Invariants:
//!
//! 1. **Response legality is challenge-derived.** A `Respond` is
//!    re-validated against the challenge's validated kind
//!    ([`super::validate_response`]) at apply time; a legal response is the
//!    only path into `Resolved`.
//! 2. **Cancellation/supersession never become credential use.** `Cancel`
//!    and `Supersede` move the challenge to a terminal state; every later
//!    event — including a legal-looking credential response — is rejected
//!    whole and can never produce `Resolved`.
//! 3. **Terminal states are immutable.** `Resolved`, `Failed`, `Cancelled`,
//!    and `Superseded` reject every further event (stale/duplicate/late
//!    callbacks).
//! 4. **Mismatched ids reject whole.** An event whose id differs from the
//!    challenge's id leaves the state unchanged.
//! 5. **Early responses reject.** A `Respond` before `RequestResponse` is a
//!    mismatched callback, never an implicit decision.
//! 6. **No TLS trust bypass.** A server-trust challenge accepts only
//!    cancellation (see `response.rs`); the auth authority makes no trust
//!    decisions.
//! 7. **Supersession names a different challenge.** `Supersede { by }` with
//!    `by == id` is rejected whole: a challenge cannot supersede itself, and
//!    the successor id is always a distinct, newer challenge.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{validate_response, AuthChallenge, AuthResponse, ChallengeId, ResponseRejection};

/// The state of one challenge's decision cycle. `Resolved`, `Failed`,
/// `Cancelled`, and `Superseded` are terminal: no event may be applied once
/// the challenge left the non-terminal states.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// The challenge was recorded but not yet presented to the authority.
    Pending,
    /// The challenge was presented; the authority is awaiting a legal
    /// response.
    ResponseRequested,
    /// A legal response was accepted and handed to the adapter. Terminal.
    /// The response carries only opaque storage-key handles, never secret
    /// bytes.
    Resolved { response: AuthResponse },
    /// The challenge failed natively (engine rejection, connection loss, or
    /// an unclassifiable adapter failure). Terminal.
    Failed { failure: AuthFailure },
    /// The challenge was cancelled. Terminal; can never become credential
    /// use.
    Cancelled,
    /// A newer challenge (`by`) replaced this one. Terminal; can never
    /// become credential use. `by` is always a distinct challenge id:
    /// self-supersession (`by == id`) is rejected at apply time.
    Superseded { by: ChallengeId },
}

impl AuthState {
    /// True for `Resolved`, `Failed`, `Cancelled`, and `Superseded`. No
    /// event may be applied to a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AuthState::Resolved { .. }
                | AuthState::Failed { .. }
                | AuthState::Cancelled
                | AuthState::Superseded { .. }
        )
    }

    /// The accepted response while the state is `Resolved`; `None`
    /// otherwise. The adapter reads the decision here and applies it to the
    /// native challenge.
    pub fn resolved_response(&self) -> Option<&AuthResponse> {
        match self {
            AuthState::Resolved { response } => Some(response),
            _ => None,
        }
    }
}

/// A closed reason a challenge failed natively. The native adapter maps its
/// concrete error into one of these; the contract does not interpret engine
/// internals.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFailure {
    /// The engine/adapter refused the challenge itself (no policy, invalid
    /// request).
    EngineRejected,
    /// The connection failed before the challenge could be satisfied.
    Connection,
    /// Anything the adapter cannot classify.
    Unknown,
}

/// A lifecycle event for one challenge, keyed by [`ChallengeId`]. The caller
/// dispatches by `id`; [`apply_event`] also checks that id against the
/// supplied challenge's id.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthEvent {
    /// Present the pending challenge to the authority. Legal only once, from
    /// [`AuthState::Pending`].
    RequestResponse { id: ChallengeId },
    /// The authority's response choice. Legal only from
    /// [`AuthState::ResponseRequested`] and only when
    /// [`super::validate_response`] accepts it for the challenge kind.
    Respond {
        id: ChallengeId,
        response: AuthResponse,
    },
    /// The challenge failed natively (engine rejection, connection loss, or
    /// an unclassifiable adapter failure). Legal from `Pending` and
    /// `ResponseRequested`.
    Failed {
        id: ChallengeId,
        failure: AuthFailure,
    },
    /// The challenge was cancelled (authority decision or engine stop).
    /// Legal from `Pending` and `ResponseRequested`.
    Cancel { id: ChallengeId },
    /// A newer challenge (`by`) replaced this one before it was decided.
    /// Legal from `Pending` and `ResponseRequested`; `by` must differ from
    /// this challenge's own id.
    Supersede { id: ChallengeId, by: ChallengeId },
}

/// Why [`apply_event`] rejected an event against the supplied challenge and
/// state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventRejection {
    /// The event was routed to a different challenge than the supplied
    /// state. The conflicting event id is reported.
    MismatchedId {
        expected: ChallengeId,
        event: ChallengeId,
    },
    /// A `Respond` arrived before `RequestResponse`: an early/mismatched
    /// callback can never become an implicit decision.
    NotRequested { response: AuthResponse },
    /// A second `RequestResponse` for the same challenge; the request was
    /// already made.
    AlreadyRequested,
    /// The response is not legal for the challenge kind (cross-kind
    /// credential/identity, server-trust continuation, or undefined default
    /// handling). Rejected whole; the state is unchanged.
    IllegalResponse { rejection: ResponseRejection },
    /// The state is already terminal (`Resolved`/`Failed`/`Cancelled`/
    /// `Superseded`); no further event may be applied. Stale, duplicate, and
    /// late callbacks all land here.
    AlreadyTerminal { state: AuthState },
    /// A `Supersede` event named the challenge's own id as its successor; a
    /// challenge cannot supersede itself. Rejected whole; the state is
    /// unchanged.
    SelfSupersede { id: ChallengeId },
}

/// Apply one lifecycle event to the caller-supplied current state.
///
/// The caller must pass the state of `challenge.id()`. The event id is
/// checked before any transition, and a `Respond` is re-validated against
/// the challenge's kind, so a stale, misrouted, cross-kind, or illegal event
/// leaves the state unchanged. Returns the ephemeral next state or a typed
/// rejection.
pub fn apply_event(
    challenge: &AuthChallenge,
    state: &AuthState,
    event: AuthEvent,
) -> Result<AuthState, EventRejection> {
    let event_id = match &event {
        AuthEvent::RequestResponse { id }
        | AuthEvent::Respond { id, .. }
        | AuthEvent::Failed { id, .. }
        | AuthEvent::Cancel { id }
        | AuthEvent::Supersede { id, .. } => *id,
    };
    if event_id != challenge.id() {
        return Err(EventRejection::MismatchedId {
            expected: challenge.id(),
            event: event_id,
        });
    }

    use AuthEvent::{Cancel, Failed, RequestResponse, Respond, Supersede};
    use AuthState::{
        Cancelled as CancelledState, Failed as FailedState, Pending, Resolved as ResolvedState,
        ResponseRequested, Superseded as SupersededState,
    };

    // Terminal states are immutable: any stale/duplicate/late callback is
    // rejected whole and can never revive or redirect the challenge.
    match (state, event) {
        (
            terminal @ (ResolvedState { .. }
            | FailedState { .. }
            | CancelledState
            | SupersededState { .. }),
            _,
        ) => Err(EventRejection::AlreadyTerminal {
            state: terminal.clone(),
        }),

        // Pending: presentation, native failure, cancellation, and
        // supersession are legal; a response before the request is not.
        (Pending, RequestResponse { .. }) => Ok(ResponseRequested),
        (Pending, Failed { failure, .. }) => Ok(FailedState { failure }),
        (Pending, Cancel { .. }) => Ok(CancelledState),
        (Pending, Respond { response, .. }) => Err(EventRejection::NotRequested { response }),

        // Pending and response-requested: supersession records a distinct
        // successor id — a challenge cannot supersede itself.
        (Pending | ResponseRequested, Supersede { by, .. }) => {
            if by == challenge.id() {
                return Err(EventRejection::SelfSupersede { id: by });
            }
            Ok(SupersededState { by })
        }

        // Response requested: exactly one request, then a legal response,
        // native failure, cancellation, or supersession.
        (ResponseRequested, RequestResponse { .. }) => Err(EventRejection::AlreadyRequested),
        (ResponseRequested, Respond { response, .. }) => {
            validate_response(challenge.kind(), &response)
                .map_err(|rejection| EventRejection::IllegalResponse { rejection })?;
            match response {
                AuthResponse::Cancel => Ok(CancelledState),
                legal => Ok(ResolvedState { response: legal }),
            }
        }
        (ResponseRequested, Failed { failure, .. }) => Ok(FailedState { failure }),
        (ResponseRequested, Cancel { .. }) => Ok(CancelledState),
    }
}
