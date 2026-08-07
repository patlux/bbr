use super::{
    validate_start, DownloadEvent, DownloadId, DownloadProgress, DownloadState, StartRejection,
    StartRequest,
};

/// Why [`apply_event`] rejected an event against the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventRejection {
    /// The event was routed to a different attempt than the supplied state.
    MismatchedId {
        expected: DownloadId,
        event: DownloadId,
    },
    /// The state is already terminal (`Succeeded`/`Failed`/`Cancelled`);
    /// no further event may be applied to the same attempt. The engine must
    /// emit each terminal event exactly once.
    AlreadyTerminal { state: DownloadState },
    /// [`DownloadEvent::Progress`] or [`DownloadEvent::Succeeded`] arrived
    /// before the destination was resolved; bytes must not flow without a
    /// decided destination.
    UndecidedDestination,
    /// The engine reported an empty resolved destination.
    EmptyDestination,
    /// The destination was already resolved; a second resolution is a
    /// contradiction. The conflicting re-resolution is reported.
    AlreadyResolved { destination: String },
    /// [`DownloadEvent::DestinationResolved`] arrived after byte progress
    /// began; the destination decision must precede the transfer.
    DestinationAfterProgress,
    /// `received` regressed below the previous value; byte progress is
    /// monotonic non-decreasing.
    ProgressRegressed {
        previous: DownloadProgress,
        event: DownloadProgress,
    },
    /// A known total changed (or reverted to unknown); once declared, the
    /// total is stable.
    TotalChanged {
        previous: Option<u64>,
        event: Option<u64>,
    },
    /// `Succeeded` while a known total is not fully received; a transfer that
    /// declared a total must account for every byte before success. The
    /// adapter must report [`DownloadFailure::Network`] (or reconcile the
    /// declared length) instead of succeeding short.
    SuccessByteMismatch { received: u64, total: Option<u64> },
}

/// Apply one engine event to the caller-supplied current state.
///
/// The caller must pass the state of `expected_id`. The event id is checked
/// before any transition, so a stale or misrouted event leaves the state
/// unchanged. Returns the ephemeral next state or a typed rejection.
pub fn apply_event(
    expected_id: DownloadId,
    state: &DownloadState,
    event: DownloadEvent,
) -> Result<DownloadState, EventRejection> {
    let event_id = match &event {
        DownloadEvent::DestinationResolved { id, .. }
        | DownloadEvent::Progress { id, .. }
        | DownloadEvent::Succeeded { id }
        | DownloadEvent::Failed { id, .. }
        | DownloadEvent::Cancelled { id } => *id,
    };
    if event_id != expected_id {
        return Err(EventRejection::MismatchedId {
            expected: expected_id,
            event: event_id,
        });
    }
    use DownloadEvent::{
        Cancelled, DestinationResolved, Failed, Progress, Succeeded as SucceededEvent,
    };
    // Download events are low-frequency UI/bookkeeping traffic and the states
    // are small, so matching on an owned clone keeps every transition arm
    // free of borrow gymnastics.
    match (state.clone(), event) {
        // Terminal states are immutable.
        (
            terminal @ (DownloadState::Succeeded
            | DownloadState::Failed { .. }
            | DownloadState::Cancelled),
            _,
        ) => Err(EventRejection::AlreadyTerminal { state: terminal }),

        // Waiting with an unresolved destination accepts only resolution,
        // failure, or cancellation.
        (DownloadState::Waiting { destination: None }, DestinationResolved { destination, .. }) => {
            if destination.is_empty() {
                return Err(EventRejection::EmptyDestination);
            }
            Ok(DownloadState::Waiting {
                destination: Some(destination),
            })
        }
        (DownloadState::Waiting { destination: None }, Progress { .. }) => {
            Err(EventRejection::UndecidedDestination)
        }
        (DownloadState::Waiting { destination: None }, SucceededEvent { .. }) => {
            Err(EventRejection::UndecidedDestination)
        }
        (DownloadState::Waiting { destination: None }, Failed { failure, .. }) => {
            Ok(DownloadState::Failed { failure })
        }
        (DownloadState::Waiting { destination: None }, Cancelled { .. }) => {
            Ok(DownloadState::Cancelled)
        }

        // Waiting with a resolved destination: bytes may begin, success may
        // be immediate (zero-byte download), or the attempt ends.
        (
            DownloadState::Waiting {
                destination: Some(_),
            },
            DestinationResolved {
                destination: conflicting,
                ..
            },
        ) => Err(EventRejection::AlreadyResolved {
            destination: conflicting,
        }),
        (
            DownloadState::Waiting {
                destination: Some(destination),
            },
            Progress { progress, .. },
        ) => Ok(DownloadState::Running {
            destination,
            progress,
        }),
        (
            DownloadState::Waiting {
                destination: Some(_),
            },
            SucceededEvent { .. },
        ) => Ok(DownloadState::Succeeded),
        (
            DownloadState::Waiting {
                destination: Some(_),
            },
            Failed { failure, .. },
        ) => Ok(DownloadState::Failed { failure }),
        (
            DownloadState::Waiting {
                destination: Some(_),
            },
            Cancelled { .. },
        ) => Ok(DownloadState::Cancelled),

        // Running: further progress must be monotonic with a stable total;
        // success must account for a declared total; the attempt may end.
        (DownloadState::Running { .. }, DestinationResolved { .. }) => {
            Err(EventRejection::DestinationAfterProgress)
        }
        (
            DownloadState::Running {
                destination,
                progress: previous,
            },
            Progress {
                progress: incoming, ..
            },
        ) => {
            if incoming.received < previous.received {
                return Err(EventRejection::ProgressRegressed {
                    previous,
                    event: incoming,
                });
            }
            // A known total is stable: it must not change to another value and
            // must not revert to unknown. An unknown total may become known.
            let total_conflict = match (previous.total, incoming.total) {
                (Some(previous_total), Some(incoming_total)) => previous_total != incoming_total,
                (Some(_), None) => true,
                // Unknown → unknown, unknown → known, identical known totals.
                _ => false,
            };
            if total_conflict {
                return Err(EventRejection::TotalChanged {
                    previous: previous.total,
                    event: incoming.total,
                });
            }
            Ok(DownloadState::Running {
                destination,
                progress: incoming,
            })
        }
        (DownloadState::Running { progress, .. }, SucceededEvent { .. }) => {
            if progress.total.is_some_and(|t| progress.received != t) {
                return Err(EventRejection::SuccessByteMismatch {
                    received: progress.received,
                    total: progress.total,
                });
            }
            Ok(DownloadState::Succeeded)
        }
        (DownloadState::Running { .. }, Failed { failure, .. }) => {
            Ok(DownloadState::Failed { failure })
        }
        (DownloadState::Running { .. }, Cancelled { .. }) => Ok(DownloadState::Cancelled),
    }
}

/// Why [`evaluate_cancel`] rejected the cancellation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelRejection {
    /// The attempt is already terminal; cancelling it would be a no-op and is
    /// rejected explicitly.
    AlreadyTerminal { state: DownloadState },
}

/// Validate that the caller may send [`DownloadAction::Cancel`] for the
/// attempt in `state`.
///
/// Legal only while `Waiting` or `Running`; a terminal attempt is rejected
/// ([`CancelRejection::AlreadyTerminal`]). This function does not transition
/// state optimistically. The engine confirms with [`DownloadEvent::Cancelled`],
/// and the first terminal event wins if cancellation races with success or
/// failure.
pub fn evaluate_cancel(state: &DownloadState) -> Result<(), CancelRejection> {
    if state.is_terminal() {
        return Err(CancelRejection::AlreadyTerminal {
            state: state.clone(),
        });
    }
    Ok(())
}

/// Why [`evaluate_retry`] rejected the retry request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryRejection {
    /// The current attempt is not in [`DownloadState::Failed`] — the only
    /// retryable state. Success needs no retry, cancellation is a deliberate
    /// user decision, and a live attempt must not be retried.
    NotRetryable { state: DownloadState },
    /// The request has no `retry_of` link; a retry must name its previous
    /// attempt.
    MissingRetryLink,
    /// The retry link names an attempt other than the failed attempt being
    /// retried.
    MismatchedRetryLink {
        failed_attempt: DownloadId,
        request_retry_of: DownloadId,
    },
    /// The retry reuses the failed attempt's id; a retry is a fresh attempt
    /// with a distinct id.
    SameAttempt {
        previous_attempt: DownloadId,
        new_id: DownloadId,
    },
    /// The retry's `url` is empty.
    EmptyUrl,
    /// The retry's explicit destination is empty.
    EmptyExplicitDestination,
}

/// Validate that the caller may retry the attempt in `state` with a fresh
/// `request` (a [`StartRequest`] whose `retry_of` names the previous attempt
/// and whose `id` is a new id).
///
/// Legal only when `state` is [`DownloadState::Failed`]; the request must
/// link the previous attempt with a distinct new id and pass
/// [`validate_start`]'s well-formedness checks. On success the caller sends
/// `DownloadAction::Start(request.clone())` and records the new attempt's
/// optimistic initial state.
pub fn evaluate_retry(
    failed_attempt_id: DownloadId,
    state: &DownloadState,
    request: &StartRequest,
) -> Result<(), RetryRejection> {
    if !matches!(state, DownloadState::Failed { .. }) {
        return Err(RetryRejection::NotRetryable {
            state: state.clone(),
        });
    }
    let Some(previous_attempt) = request.retry_of else {
        return Err(RetryRejection::MissingRetryLink);
    };
    if previous_attempt != failed_attempt_id {
        return Err(RetryRejection::MismatchedRetryLink {
            failed_attempt: failed_attempt_id,
            request_retry_of: previous_attempt,
        });
    }
    if failed_attempt_id == request.id {
        return Err(RetryRejection::SameAttempt {
            previous_attempt: failed_attempt_id,
            new_id: request.id,
        });
    }
    match validate_start(request) {
        Ok(()) => Ok(()),
        Err(StartRejection::EmptyUrl) => Err(RetryRejection::EmptyUrl),
        Err(StartRejection::EmptyExplicitDestination) => {
            Err(RetryRejection::EmptyExplicitDestination)
        }
    }
}
