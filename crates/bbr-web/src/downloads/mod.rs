//! # Downloads capability contract
//!
//! The portable, platform-neutral contract for the Phase 8 downloads
//! capability (the first item of
//! [Phase 8 — daily-driver browser essentials](../../../docs/roadmap/phases/08-daily-driver.md)):
//! destination decision, lifecycle/progress, cancellation, retry, and
//! failure. It models **one download attempt** end to end without any
//! filesystem, network, or engine implementation. A native engine performs
//! the transfer behind this seam (WebKit's `WKDownload` on macOS); this
//! module fixes the types and state-machine rules both sides must agree on.
//!
//! Like [`crate::WebEngine`], the module is **portable**: no frameworks, no
//! GPUI, no `unsafe`, no file I/O, and no new dependencies (only `std`). The
//! wire types ([`DownloadAction`], [`DownloadEvent`]) derive `serde` behind
//! the crate's optional `serde` feature so a later E2E protocol can
//! serialize downloads events and snapshots without leaking engine types.
//!
//! ## Stateless authority model
//!
//! This module stores **no per-download state**. The owning controller keeps
//! the `DownloadId → DownloadState` map and dispatches [`DownloadEvent`]s by
//! id; every function evaluates against the caller-supplied attempt id and
//! state, returning an ephemeral next state or a typed rejection. The id is
//! checked again at this seam so a stale or misrouted event cannot mutate the
//! wrong attempt. This mirrors the repository's stateless plan pattern (see
//! `crates/bbr-split/src/plan.rs`): the caller owns authoritative storage,
//! the contract owns the rules.
//!
//! ## Lifecycle
//!
//! One attempt starts when the caller records the optimistic initial state
//! [`DownloadState::Waiting`] with `destination: None` and sends
//! [`DownloadAction::Start`] to the engine. From then on only engine events
//! ([`DownloadEvent`]) move the state:
//!
//! ```text
//! Waiting { destination: None }
//!   ├─ DestinationResolved ─────────────► Waiting { destination: Some(..) }
//!   │                                      ├─ Progress ──► Running { destination, progress }
//!   │                                      │                ├─ Progress (monotonic) ──► Running { .. }
//!   │                                      │                ├─ Succeeded ──► Succeeded
//!   │                                      │                ├─ Failed ──► Failed
//!   │                                      │                └─ Cancelled ──► Cancelled
//!   │                                      ├─ Succeeded ──► Succeeded      (zero-byte download)
//!   │                                      ├─ Failed ──► Failed
//!   │                                      └─ Cancelled ──► Cancelled
//!   ├─ Failed ──► Failed
//!   └─ Cancelled ──► Cancelled
//! ```
//!
//! `Succeeded`, `Failed`, and `Cancelled` are **terminal**: no further event
//! may be applied to the same attempt ([`EventRejection::AlreadyTerminal`]).
//! A failed attempt is retryable and a cancelled attempt is restartable;
//! both start a fresh attempt ([`StartRequest`] with `retry_of` set)
//! produced through [`evaluate_retry`]. Cancellation
//! ([`evaluate_cancel`]) is approved only before a terminal
//! state; it does not optimistically change state. The first terminal engine
//! event wins, so a cancellation race with success/failure is deterministic.
//! The caller keeps history (final paths, timestamps, attempt counts) in its
//! own record; this module tracks only what the machine needs.
//!
//! ## Contract invariants (proven by the crate tests)
//!
//! 1. **Byte progress never regresses**: `received` is monotonic
//!    non-decreasing across [`DownloadEvent::Progress`] events
//!    ([`EventRejection::ProgressRegressed`] otherwise).
//! 2. **The declared total is stable once known**: an unknown total may
//!    become known, but a known total never changes and never becomes unknown
//!    ([`EventRejection::TotalChanged`]).
//! 3. **Invalid progress is unrepresentable**: [`DownloadProgress::new`]
//!    rejects `received > total`, so every progress value that reaches the
//!    machine already satisfies the bound; `fraction()` is always in
//!    `[0.0, 1.0]` and never `NaN`.
//! 4. **No bytes before a destination**: [`DownloadEvent::Progress`] and
//!    [`DownloadEvent::Succeeded`] are rejected while the destination is
//!    unresolved ([`EventRejection::UndecidedDestination`]); the destination
//!    resolves at most once and only before bytes flow
//!    ([`EventRejection::AlreadyResolved`] /
//!    [`EventRejection::DestinationAfterProgress`]).
//! 5. **Success accounts for a declared total**: `Succeeded` while the total
//!    is known requires every declared byte to be received
//!    ([`EventRejection::SuccessByteMismatch`]); an unknown-length download
//!    may succeed with any received count.
//! 6. **Terminal states are immutable**: any event on `Succeeded`/`Failed`/
//!    `Cancelled` is rejected ([`EventRejection::AlreadyTerminal`]), and
//!    cancellation is rejected once terminal
//!    ([`CancelRejection::AlreadyTerminal`]).
//! 7. **Only terminal failures/cancellations are retryable/restartable**:
//!    [`evaluate_retry`] accepts a fresh attempt linked via `retry_of` only
//!    when the current state is [`DownloadState::Failed`] (Retry) or
//!    [`DownloadState::Cancelled`] (Restart, the settled post-MVP decision)
//!    for the supplied previous-attempt id, and the new id must differ from
//!    that id ([`RetryRejection::SameAttempt`]).
//! 8. **Event identity is checked**: the event id must equal the id of the
//!    state being updated ([`EventRejection::MismatchedId`]).
//! 9. **Resolved destinations are usable**: an engine cannot resolve an empty
//!    destination ([`EventRejection::EmptyDestination`]).
//! 10. **Requests are well-formed**: a start/retry needs a non-empty `url`
//!     and a non-empty explicit destination; a suggested destination may be
//!     empty, in which case the engine derives it from the URL
//!     ([`StartRejection`] / [`RetryRejection`]).
//!
//! # Example
//!
//! ```
//! use bbr_web::downloads::{
//!     apply_event, evaluate_cancel, evaluate_retry, validate_start,
//!     DestinationDecision, DownloadEvent, DownloadFailure, DownloadId,
//!     DownloadProgress, DownloadState, StartRequest,
//! };
//!
//! let id = DownloadId(1);
//! let start = StartRequest {
//!     id,
//!     url: "https://example.com/report.pdf".into(),
//!     destination: DestinationDecision::Suggested("report.pdf".into()),
//!     retry_of: None,
//! };
//! validate_start(&start).expect("well-formed start");
//!
//! // The caller records the optimistic initial state, then dispatches engine
//! // events by id (the contract also verifies that identity).
//! let mut state = DownloadState::Waiting { destination: None };
//! state = apply_event(
//!     id,
//!     &state,
//!     DownloadEvent::DestinationResolved {
//!         id,
//!         destination: "/tmp/report.pdf".into(),
//!     },
//! )
//! .expect("destination resolves before bytes");
//! state = apply_event(
//!     id,
//!     &state,
//!     DownloadEvent::Progress {
//!         id,
//!         progress: DownloadProgress::new(42, Some(100)).expect("valid progress"),
//!     },
//! )
//! .expect("bytes flow after destination");
//! assert!(matches!(
//!     state,
//!     DownloadState::Running { progress, .. } if progress.received() == 42
//! ));
//!
//! // Cancellation is approved while running; the engine confirms it with an
//! // event. A success/failure event racing with cancellation is the winner.
//! evaluate_cancel(&state).expect("running downloads are cancellable");
//! state = apply_event(
//!     id,
//!     &state,
//!     DownloadEvent::Cancelled { id },
//! )
//! .expect("engine confirms cancellation");
//!
//! // A cancelled attempt is terminal and — per the settled post-MVP
//! // decision — restartable (a fresh attempt linked via `retry_of`, exactly
//! // like a failed retry).
//! assert!(evaluate_retry(id, &state, &start).is_err()); // not linked yet
//! let state = DownloadState::Failed {
//!     failure: DownloadFailure::Network,
//! };
//! let retry = StartRequest {
//!     id: DownloadId(2),
//!     url: start.url.clone(),
//!     destination: start.destination.clone(),
//!     retry_of: Some(id),
//! };
//! evaluate_retry(id, &state, &retry).expect("failed attempts are retryable");
//! ```

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Stable identifier of one download attempt. A retry is a **new** id; an id
/// is never reused once its attempt reached a terminal state.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DownloadId(pub u64);

/// Where a download should land, decided by the caller (UI save panel or the
/// controller's policy) before the engine starts the transfer.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationDecision {
    /// The caller chose the exact destination; the engine must use it
    /// verbatim with no folder or uniquification policy.
    Explicit(String),
    /// A suggested destination (usually the filename derived from the URL).
    /// The engine may apply its default-download-folder and uniquification
    /// policy and reports the resolved destination back via
    /// [`DownloadEvent::DestinationResolved`]. An empty suggestion means
    /// "derive the filename from the URL".
    Suggested(String),
}

/// Byte progress of one attempt.
///
/// The value is **invariant-holding**: construction rejects
/// `received > total`, so invalid progress is unrepresentable and every value
/// that reaches the state machine already satisfies the bound. Equality is
/// plain value equality; the machine still re-checks cross-event monotonicity
/// and total stability in [`apply_event`].
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    received: u64,
    total: Option<u64>,
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for DownloadProgress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawProgress {
            received: u64,
            total: Option<u64>,
        }

        let raw = RawProgress::deserialize(deserializer)?;
        DownloadProgress::new(raw.received, raw.total).map_err(|invalid| {
            serde::de::Error::custom(format!(
                "received bytes ({}) exceed declared total ({:?})",
                invalid.received, invalid.total
            ))
        })
    }
}

/// Why [`DownloadProgress::new`] rejected the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProgress {
    pub received: u64,
    pub total: Option<u64>,
}

impl DownloadProgress {
    /// Build a progress value; `total` is `None` when the length is unknown.
    ///
    /// Returns [`InvalidProgress`] when a known `total` is smaller than
    /// `received`. Zero-byte progress (`received == 0`, `total == Some(0)`)
    /// is valid and complete.
    pub fn new(received: u64, total: Option<u64>) -> Result<Self, InvalidProgress> {
        if total.is_some_and(|t| received > t) {
            return Err(InvalidProgress { received, total });
        }
        Ok(Self { received, total })
    }

    /// Bytes received so far, monotonic non-decreasing across events.
    pub fn received(self) -> u64 {
        self.received
    }

    /// The declared total length, `None` while unknown. Once `Some`, the
    /// value never changes and never reverts to `None`.
    pub fn total(self) -> Option<u64> {
        self.total
    }

    /// The fraction of the declared total received, in `[0.0, 1.0]`.
    ///
    /// `None` while the total is unknown. A zero-byte total yields `1.0`
    /// (everything of the declared zero bytes is received), so the result is
    /// never `NaN`.
    pub fn fraction(self) -> Option<f64> {
        match self.total {
            None => None,
            Some(0) => Some(1.0),
            Some(total) => Some(self.received as f64 / total as f64),
        }
    }

    /// True when a known total has been fully received. Unknown-length
    /// downloads are never "complete" through progress alone; completion is
    /// signalled by [`DownloadEvent::Succeeded`].
    pub fn is_complete(self) -> bool {
        self.total.is_some_and(|t| self.received == t)
    }
}

/// Closed reason a transfer failed. The native adapter maps its concrete
/// error into one of these; the contract does not interpret engine internals.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadFailure {
    /// The engine refused to start the transfer (no policy, invalid request).
    EngineRejected,
    /// The destination decision could not be established.
    NoDestination,
    /// The transfer failed at the network layer (connection, HTTP status,
    /// truncation).
    Network,
    /// The transfer failed while writing to or replacing the destination.
    Destination,
    /// Anything the adapter cannot classify.
    Unknown,
}

/// The state of one download attempt. Terminal states are `Succeeded`,
/// `Failed`, and `Cancelled`; each is immutable once reached.
///
/// The state carries the resolved destination only where the machine needs
/// it (`Waiting`/`Running`); final history (paths, timestamps) is the
/// caller's record, not part of the state machine.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// The attempt was accepted and no bytes have flowed yet.
    ///
    /// `destination: None` is the optimistic initial state the caller records
    /// when issuing [`DownloadAction::Start`]; the engine resolves the
    /// destination ([`DownloadEvent::DestinationResolved`]) before any
    /// [`DownloadEvent::Progress`] is legal.
    Waiting { destination: Option<String> },
    /// Bytes are transferring to the resolved `destination`.
    Running {
        destination: String,
        progress: DownloadProgress,
    },
    /// The transfer finished; success implies the destination was decided and
    /// a declared total was fully received ([`EventRejection::SuccessByteMismatch`]
    /// guards the latter).
    Succeeded,
    /// The transfer ended in [`DownloadFailure`]. The only retryable state.
    Failed { failure: DownloadFailure },
    /// The attempt was cancelled on request ([`DownloadAction::Cancel`]);
    /// terminal and immutable once reached. A cancelled row may be
    /// **restarted** as a fresh attempt linked via `retry_of` (the settled
    /// post-MVP decision, DESIGN.md §17.8) — the same machine path as a
    /// failed retry, with a different panel label.
    Cancelled,
}

impl DownloadState {
    /// True for `Succeeded`, `Failed`, and `Cancelled`. No event may be
    /// applied to a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            DownloadState::Succeeded | DownloadState::Failed { .. } | DownloadState::Cancelled
        )
    }

    /// True for `Failed` (Retry) and `Cancelled` (Restart): both are
    /// deliberate terminal outcomes the user may re-run as a fresh linked
    /// attempt. Success needs no retry and a live attempt must not be
    /// retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DownloadState::Failed { .. } | DownloadState::Cancelled
        )
    }

    /// The resolved destination while one is observable: after
    /// [`DownloadEvent::DestinationResolved`] and while bytes flow.
    pub fn resolved_destination(&self) -> Option<&str> {
        match self {
            DownloadState::Waiting {
                destination: Some(d),
            } => Some(d),
            DownloadState::Running { destination, .. } => Some(destination),
            _ => None,
        }
    }

    /// The current byte progress while the attempt is running.
    pub fn progress(&self) -> Option<&DownloadProgress> {
        match self {
            DownloadState::Running { progress, .. } => Some(progress),
            _ => None,
        }
    }
}

/// A well-formed request to begin one download attempt.
///
/// `retry_of: None` is a fresh user-initiated download; `retry_of:
/// Some(previous)` marks the attempt as a retry of a failed one and is only
/// legal through [`evaluate_retry`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartRequest {
    pub id: DownloadId,
    pub url: String,
    pub destination: DestinationDecision,
    pub retry_of: Option<DownloadId>,
}

/// Why [`validate_start`] rejected the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartRejection {
    /// The `url` is empty; a download needs a source.
    EmptyUrl,
    /// An [`DestinationDecision::Explicit`] destination is empty; an explicit
    /// decision must name a concrete destination. A `Suggested` destination
    /// may be empty (the engine derives it from the URL).
    EmptyExplicitDestination,
}

/// Validate the well-formedness of a start request (empty `url`, empty
/// explicit destination). Retry-linkage legality is [`evaluate_retry`]'s
/// job; this function accepts any `retry_of` value.
pub fn validate_start(request: &StartRequest) -> Result<(), StartRejection> {
    if request.url.is_empty() {
        return Err(StartRejection::EmptyUrl);
    }
    if let DestinationDecision::Explicit(destination) = &request.destination {
        if destination.is_empty() {
            return Err(StartRejection::EmptyExplicitDestination);
        }
    }
    Ok(())
}

/// Commands the portable side sends to the engine for one download attempt.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadAction {
    /// Begin a new attempt. The engine either resolves the destination
    /// ([`DownloadEvent::DestinationResolved`]) or fails the attempt
    /// ([`DownloadEvent::Failed`] with [`DownloadFailure::EngineRejected`]).
    Start(StartRequest),
    /// Request cancellation of an in-flight attempt; legal only while the
    /// attempt is non-terminal ([`evaluate_cancel`]). The engine confirms
    /// with [`DownloadEvent::Cancelled`].
    Cancel { id: DownloadId },
}

/// Events the engine emits for one download attempt, keyed by [`DownloadId`].
/// The caller dispatches by `id`; [`apply_event`] also checks that id against
/// the explicitly supplied state identity.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    /// The destination was decided (explicit verbatim, or suggested resolved
    /// through engine policy). May arrive only before any byte progress.
    DestinationResolved { id: DownloadId, destination: String },
    /// Byte progress. `received` is monotonic non-decreasing; a known total
    /// is stable.
    Progress {
        id: DownloadId,
        progress: DownloadProgress,
    },
    /// The transfer finished successfully.
    Succeeded { id: DownloadId },
    /// The transfer failed with a closed reason.
    Failed {
        id: DownloadId,
        failure: DownloadFailure,
    },
    /// The attempt was cancelled (always in response to
    /// [`DownloadAction::Cancel`]).
    Cancelled { id: DownloadId },
}

mod machine;

pub use machine::{
    apply_event, evaluate_cancel, evaluate_retry, CancelRejection, EventRejection, RetryRejection,
};

#[cfg(test)]
mod tests_contract;
#[cfg(test)]
mod tests_state;
