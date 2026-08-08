//! Pure mapping between the L6 portable download contract and the native
//! `WKDownload` lifecycle.
//!
//! This module contains **no** AppKit/WebKit imports and no `unsafe` code, so
//! the entire portable↔native mapping can be contract-tested without a
//! display server. The native side ([`crate::downloads_web_kit`]) converts
//! Objective-C callbacks into [`NativeSignal`]s and routes them through
//! [`DownloadAdapterCore::signal`]; the core converts them into L6
//! [`DownloadEvent`]s, mirroring the authoritative state machine so rejected
//! or late events never reach the composition.
//!
//! ## WebKit download API verdict
//!
//! The pinned `objc2-web-kit 0.3.2` binding exposes the modern `WKDownload`
//! lifecycle in full:
//!
//! - **Destination:** the **required** `WKDownloadDelegate` method
//!   `download:decideDestinationUsingResponse:suggestedFilename:
//!   completionHandler:` is called once, before any bytes flow, with the
//!   response (whose `expectedContentLength` declares the total, `-1` when
//!   unknown) and a suggested filename. The delegate passes an `NSURL*` (or
//!   `nil` to cancel) through the completion block. This is exactly the L6
//!   destination-before-progress ordering, and the callback fires at most
//!   once per download.
//! - **Progress:** `WKDownload` conforms to `NSProgressReporting`; its
//!   `NSProgress` carries `completedUnitCount` (bytes, monotonic) and
//!   `totalUnitCount` (`0` while unknown, stable once known), observable via
//!   KVO on both key paths.
//! - **Completion:** `downloadDidFinish:` is the success terminal callback.
//! - **Failure:** `download:didFailWithError:resumeData:` is the failure
//!   terminal callback for errors other than user cancellation. A user
//!   cancel (`-cancel`) is confirmed by the `cancel:` completion block
//!   (resume data), which the native strategy routes as
//!   `NativeSignal::Cancelled` — `didFailWithError:` is **not** delivered
//!   for cancellations on this WebKit version. Exactly **one** terminal
//!   callback (`downloadDidFinish`, `didFailWithError`, or the cancel
//!   completion) is delivered per download, so the L6
//!   first-terminal-event-wins rule is deterministic at the adapter seam.
//! - **Cancel:** `WKDownload.cancel:` accepts an optional resume-data
//!   completion block; the terminal confirmation arrives through that
//!   block, and the strategy routes it as the `Cancelled` signal.
//! - **Retry:** a retry is a fresh `startDownloadUsingRequest:
//!   completionHandler:` on the originating web view; L6 mints a new id and
//!   links it via `retry_of`. WebKit exposes no resume continuation API in
//!   this binding's surface beyond the resume-data block, and v1 retry
//!   re-downloads from scratch.
//!
//! The binding therefore expresses the full L6 contract; no private SPI, no
//! fabricated progress.
//!
//! ## Mapping table (native callback → L6 event)
//!
//! | Native signal                                   | Core outcome                            |
//! |-------------------------------------------------|-----------------------------------------|
//! | `decideDestination…` → resolved URL             | `DestinationResolved { path }` (+ total)|
//! | `decideDestination…` → `nil` (no folder)        | `Failed { NoDestination }`              |
//! | `decideDestination…` → HTTP status ≥ 400        | `Failed { Network }`                     |
//! | KVO `completedUnitCount`/`totalUnitCount`       | `Progress { received, total }`          |
//! | `downloadDidFinish:`                            | `Progress(finish)` + `Succeeded`        |
//! | `didFailWithError:` + `NSURLErrorCancelled` + user cancel | `Cancelled`                 |
//! | `WKDownload.cancel:` completion (resume data)    | `Cancelled`                              |
//! | `didFailWithError:` + `NSURLErrorCancelled`, no user cancel | `Failed { Network }`      |
//! | `didFailWithError:` + `WKErrorDomain`           | `Failed { EngineRejected }`             |
//! | `didFailWithError:` + URL network codes         | `Failed { Network }`                    |
//! | `didFailWithError:` + URL file codes            | `Failed { Destination }`                |
//! | `didFailWithError:` + anything else             | `Failed { Unknown }`                    |
//!
//! Every mapped event is applied to a **mirror** of the L6 machine first; a
//! rejected event (monotonicity, total stability, terminal immutability,
//! destination ordering) is dropped and logged, never forwarded. This is the
//! adapter's half of the first-terminal-event-wins rule: the controller's
//! authoritative machine enforces it independently, so the two layers agree.

use std::collections::HashMap;

use bbr_web::downloads::{
    apply_event, evaluate_cancel, CancelRejection, DownloadEvent, DownloadFailure, DownloadId,
    DownloadProgress, DownloadState, EventRejection, StartRequest,
};

/// `NSError.domain` value of the Foundation URL loading system
/// (`NSURLErrorDomain`).
pub const NSURL_ERROR_DOMAIN: &str = "NSURLErrorDomain";
/// `NSError.domain` value of the WebKit error system (`WKErrorDomain`).
pub const WEBKIT_ERROR_DOMAIN: &str = "WKErrorDomain";
/// `NSURLErrorCancelled` (-999): the code a cancelled transfer fails with.
pub const NSURL_ERROR_CANCELLED: i64 = -999;

/// Network-layer `NSURLErrorDomain` codes mapped to
/// [`DownloadFailure::Network`].
const NETWORK_ERROR_CODES: &[i64] = &[
    -1000, // NSURLErrorBadURL
    -1001, // NSURLErrorTimedOut
    -1002, // NSURLErrorUnsupportedURL
    -1003, // NSURLErrorCannotFindHost
    -1004, // NSURLErrorCannotConnectToHost
    -1005, // NSURLErrorNetworkConnectionLost
    -1006, // NSURLErrorDNSLookupFailed
    -1007, // NSURLErrorHTTPTooManyRedirects
    -1008, // NSURLErrorResourceUnavailable
    -1009, // NSURLErrorNotConnectedToInternet
    -1011, // NSURLErrorBadServerResponse
    -1017, // NSURLErrorCannotParseResponse
];

/// Destination-writing `NSURLErrorDomain` codes mapped to
/// [`DownloadFailure::Destination`].
const DESTINATION_ERROR_CODES: &[i64] = &[
    -1100, // NSURLErrorFileDoesNotExist
    -1101, // NSURLErrorFileIsDirectory
    -1102, // NSURLErrorNoPermissionsToReadFile
    -1103, // NSURLErrorDataLengthExceedsMaximum
    -1104, // NSURLErrorFileOutsideSafeArea
];

/// A typed signal the native `WKDownload` layer reports for one attempt.
///
/// The native side is responsible only for converting Objective-C values;
/// every classification and transition rule lives here (pure, testable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeSignal {
    /// `decideDestination…` was answered with `path`; `expected_total` is the
    /// response's declared content length (`None` = unknown). The L6 total
    /// is seeded from this value and is authoritative over any later
    /// `NSProgress` total.
    DestinationResolved {
        id: DownloadId,
        path: String,
        expected_total: Option<u64>,
    },
    /// `decideDestination…` was answered with `nil` because no destination
    /// could be established (download folder missing/uncreatable).
    DestinationUnavailable { id: DownloadId },
    /// KVO progress observation: `received` bytes with the `NSProgress` total
    /// (`0` = unknown; the core prefers the expected total).
    Progress {
        id: DownloadId,
        received: u64,
        total: u64,
    },
    /// `downloadDidFinish:` with the final byte snapshot.
    Finished {
        id: DownloadId,
        received: u64,
        total: u64,
    },
    /// `download:didFailWithError:resumeData:` — `domain`/`code` from the
    /// `NSError`.
    Failed {
        id: DownloadId,
        domain: String,
        code: i64,
    },
    /// The engine confirmed a user cancel request: `WKDownload.cancel:`'s
    /// completion handler (resume data) is the terminal confirmation
    /// `WKDownload` delivers on cancellation — `didFailWithError:` is not
    /// called for cancellations.
    Cancelled { id: DownloadId },
    /// `decideDestination…` saw an HTTP error status (≥ 400): the transfer
    /// is refused (nil destination) and classified [`DownloadFailure::Network`]
    /// — the L6 contract names HTTP status explicitly. `WKDownload` otherwise
    /// downloads the error body and reports success.
    HttpStatus { id: DownloadId, status: u16 },
    /// The engine refused to start the attempt before any transfer began
    /// (unparseable URL, no web view).
    Rejected {
        id: DownloadId,
        failure: DownloadFailure,
    },
}

impl NativeSignal {
    /// The attempt id the signal belongs to.
    pub fn id(&self) -> DownloadId {
        match self {
            NativeSignal::DestinationResolved { id, .. }
            | NativeSignal::DestinationUnavailable { id }
            | NativeSignal::Progress { id, .. }
            | NativeSignal::Finished { id, .. }
            | NativeSignal::Failed { id, .. }
            | NativeSignal::Cancelled { id }
            | NativeSignal::HttpStatus { id, .. }
            | NativeSignal::Rejected { id, .. } => *id,
        }
    }

    /// True for signals that conclude the native transfer (`downloadDidFinish`
    /// and `didFailWithError` family, the cancel completion, and the
    /// destination-decision refusals). Used by the facade to release native
    /// resources after routing.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            NativeSignal::DestinationUnavailable { .. }
                | NativeSignal::Finished { .. }
                | NativeSignal::Failed { .. }
                | NativeSignal::Cancelled { .. }
                | NativeSignal::HttpStatus { .. }
                | NativeSignal::Rejected { .. }
        )
    }
}

/// Why [`DownloadAdapterCore::adopt`] rejected an attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdoptRejection {
    /// The id is already registered; ids are minted once by the controller.
    Duplicate { id: DownloadId },
}

/// The classified outcome of a native failure signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// The transfer was cancelled **in response to a user cancel request**.
    Cancelled,
    /// A closed [`DownloadFailure`] classification.
    Failed(DownloadFailure),
}

/// Classify a native `NSError` into the L6 failure vocabulary.
///
/// A cancellation error (`NSURLErrorCancelled`) is [`FailureKind::Cancelled`]
/// only when the user requested cancellation; an engine-internal cancel (no
/// user request) is a network failure, because the L6 `Cancelled` state is
/// reserved for deliberate user cancellation.
pub fn classify_failure(domain: &str, code: i64, cancel_requested: bool) -> FailureKind {
    let cancellation = domain == NSURL_ERROR_DOMAIN && code == NSURL_ERROR_CANCELLED;
    if cancellation {
        return if cancel_requested {
            FailureKind::Cancelled
        } else {
            FailureKind::Failed(DownloadFailure::Network)
        };
    }
    if domain == WEBKIT_ERROR_DOMAIN {
        return FailureKind::Failed(DownloadFailure::EngineRejected);
    }
    if domain == NSURL_ERROR_DOMAIN {
        if NETWORK_ERROR_CODES.contains(&code) {
            return FailureKind::Failed(DownloadFailure::Network);
        }
        if DESTINATION_ERROR_CODES.contains(&code) {
            return FailureKind::Failed(DownloadFailure::Destination);
        }
    }
    FailureKind::Failed(DownloadFailure::Unknown)
}

/// Per-attempt adapter state: a mirror of the L6 machine plus the native
/// bookkeeping the mapping needs.
#[derive(Debug, Clone)]
struct AttemptCore {
    /// Mirror of the authoritative controller state; `apply_event` is the
    /// only way it changes, so rejected events never move it.
    machine: DownloadState,
    /// A user cancel request was issued; consumed by the failure classifier.
    cancel_requested: bool,
    /// The response's declared content length, authoritative for the L6 total.
    expected_total: Option<u64>,
    /// The destination decision was answered (resolved or unavailable).
    destination_decided: bool,
}

impl AttemptCore {
    /// Merge the native byte snapshot with the expected total. The response's
    /// declared length wins; the `NSProgress` total only fills an unknown
    /// (`0` means unknown). Invalid progress (`received > total`) is dropped.
    fn merge_progress(&self, received: u64, ns_total: u64) -> Option<DownloadProgress> {
        let total = self
            .expected_total
            .or_else(|| (ns_total > 0).then_some(ns_total));
        DownloadProgress::new(received, total).ok()
    }

    /// The candidate L6 events for a signal, before machine application.
    fn candidates(&self, _id: DownloadId, signal: NativeSignal) -> Vec<DownloadEvent> {
        match signal {
            NativeSignal::DestinationResolved { id, path, .. } => {
                // The response-declared total is captured by the caller
                // (`DownloadAdapterCore::signal`) before `candidates` runs
                // and is authoritative over later NSProgress totals.
                vec![DownloadEvent::DestinationResolved {
                    id,
                    destination: path,
                }]
            }
            NativeSignal::DestinationUnavailable { id } => {
                vec![DownloadEvent::Failed {
                    id,
                    failure: DownloadFailure::NoDestination,
                }]
            }
            NativeSignal::Progress {
                id,
                received,
                total,
            } => {
                // Bytes require a decided destination (L6 invariant #4); a
                // stray pre-destination progress is dropped defensively —
                // WebKit orders the required destination callback first.
                if !self.destination_decided {
                    log::warn!("download {id:?}: progress before destination; dropped");
                    return Vec::new();
                }
                let Some(progress) = self.merge_progress(received, total) else {
                    log::warn!("download {id:?}: invalid progress ({received}/{total}); dropped");
                    return Vec::new();
                };
                vec![DownloadEvent::Progress { id, progress }]
            }
            NativeSignal::Finished {
                id,
                received,
                total,
            } => {
                if !self.destination_decided {
                    log::warn!("download {id:?}: finished before destination; dropped");
                    return Vec::new();
                }
                let merged = self.merge_progress(received, total);
                match (&self.machine, merged) {
                    // A known total was already fully received (or the total
                    // is unknown): a direct zero-byte/immediate success.
                    (DownloadState::Running { .. }, Some(progress)) => vec![
                        DownloadEvent::Progress { id, progress },
                        DownloadEvent::Succeeded { id },
                    ],
                    // Zero-byte downloads (declared total `Some(0)`) and
                    // unknown-length downloads both land here; the final
                    // progress snapshot precedes success so the controller's
                    // machine accounts for every declared byte.
                    (
                        DownloadState::Waiting {
                            destination: Some(_),
                        },
                        Some(progress),
                    ) => vec![
                        DownloadEvent::Progress { id, progress },
                        DownloadEvent::Succeeded { id },
                    ],
                    // `None` means the native snapshot is invalid
                    // (`received > total`); never invent success from it.
                    _ => Vec::new(),
                }
            }
            NativeSignal::Failed { id, domain, code } => {
                let kind = classify_failure(&domain, code, self.cancel_requested);
                match kind {
                    FailureKind::Cancelled => vec![DownloadEvent::Cancelled { id }],
                    FailureKind::Failed(failure) => vec![DownloadEvent::Failed { id, failure }],
                }
            }
            NativeSignal::Cancelled { id } => {
                // The engine's cancel confirmation (`WKDownload.cancel:`'s
                // completion handler) — the request was confirmed, so the
                // row becomes `Cancelled`.
                vec![DownloadEvent::Cancelled { id }]
            }
            NativeSignal::HttpStatus { id, .. } => {
                // L6 `DownloadFailure::Network` names HTTP status explicitly;
                // a refused error response is a network-layer failure.
                vec![DownloadEvent::Failed {
                    id,
                    failure: DownloadFailure::Network,
                }]
            }
            NativeSignal::Rejected { id, failure } => {
                vec![DownloadEvent::Failed { id, failure }]
            }
        }
    }
}

/// The pure adapter core: per-attempt bookkeeping plus the signal→event
/// mapping. No AppKit, no filesystem, no `unsafe` — fully contract-testable.
#[derive(Debug, Default)]
pub struct DownloadAdapterCore {
    attempts: HashMap<DownloadId, AttemptCore>,
}

impl DownloadAdapterCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an attempt with its optimistic initial state. Called before
    /// the native transfer starts, so a signal can never precede the record.
    pub fn adopt(&mut self, request: &StartRequest) -> Result<(), AdoptRejection> {
        if self.attempts.contains_key(&request.id) {
            return Err(AdoptRejection::Duplicate { id: request.id });
        }
        self.attempts.insert(
            request.id,
            AttemptCore {
                machine: DownloadState::Waiting { destination: None },
                cancel_requested: false,
                expected_total: None,
                destination_decided: false,
            },
        );
        Ok(())
    }

    /// Record a user cancel request for `id`. Rejects a terminal attempt
    /// (L6 [`CancelRejection`]) and does not optimistically change state —
    /// the engine confirms via its terminal callback.
    pub fn cancel_requested(&mut self, id: DownloadId) -> Result<(), CancelRejection> {
        let Some(attempt) = self.attempts.get_mut(&id) else {
            // Unknown ids are cancelled by dropping the request: the native
            // side has nothing to cancel, and the controller validates
            // existence before ever sending the action.
            return Ok(());
        };
        evaluate_cancel(&attempt.machine)?;
        attempt.cancel_requested = true;
        Ok(())
    }

    /// Whether the core has a record for `id`.
    pub fn is_known(&self, id: DownloadId) -> bool {
        self.attempts.contains_key(&id)
    }

    /// Map one native signal to the L6 events the composition must apply.
    ///
    /// Every candidate event is applied to the mirror machine first; only
    /// accepted transitions are returned, so a late terminal signal, a
    /// regressing progress, or a conflicting total never leaks out. A
    /// `Succeeded` that would land short of a declared total is converted to
    /// [`DownloadFailure::Network`] per the L6 contract.
    pub fn signal(&mut self, signal: NativeSignal) -> Vec<DownloadEvent> {
        let id = signal.id();
        let Some(attempt) = self.attempts.get_mut(&id) else {
            log::warn!("download {id:?}: signal for unknown attempt; dropped");
            return Vec::new();
        };
        if attempt.machine.is_terminal() {
            // First terminal event already won; everything later is dropped.
            return Vec::new();
        }

        // The expected total travels with the destination signal; capture it
        // before candidates() reads it.
        if let NativeSignal::DestinationResolved { expected_total, .. } = &signal {
            attempt.expected_total = *expected_total;
        }

        let candidates = attempt.candidates(id, signal);
        let mut forwarded = Vec::new();
        for event in candidates {
            match apply_event(id, &attempt.machine, event.clone()) {
                Ok(next) => {
                    if let DownloadEvent::DestinationResolved { .. } = &event {
                        attempt.destination_decided = true;
                    }
                    attempt.machine = next;
                    forwarded.push(event);
                }
                Err(EventRejection::SuccessByteMismatch { .. })
                    if matches!(event, DownloadEvent::Succeeded { .. }) =>
                {
                    // L6: never succeed short of a declared total; report the
                    // network failure instead of inventing success.
                    let failed = DownloadEvent::Failed {
                        id,
                        failure: DownloadFailure::Network,
                    };
                    if let Ok(next) = apply_event(id, &attempt.machine, failed.clone()) {
                        attempt.machine = next;
                        forwarded.push(failed);
                    }
                }
                Err(rejection) => {
                    log::warn!("download {id:?}: dropped {event:?}: {rejection:?}");
                }
            }
        }
        forwarded
    }
}

#[cfg(test)]
mod tests;
