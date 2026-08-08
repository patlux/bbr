//! The store that composes classification, burst deduplication, and routing
//! behind one synchronous handle.
//!
//! [`ExternalUrlCoordinator::handle`] is the single entry point for an
//! OS-dispatched external URL open. It applies two contract-discipline
//! guards (fixed order, both leave the store unchanged when they fire), then
//! classifies, deduplicates, and routes:
//!
//! 1. **Request-id reuse** — ids are never reused for the lifetime of the
//!    store ([`ExternalUrlError::DuplicateRequestId`]).
//! 2. **Tick monotonicity** — ticks must not go backwards
//!    ([`ExternalUrlError::NonMonotonicTick`]); equal ticks are allowed.
//! 3. **Classification** — a non-web URL is rejected
//!    ([`ExternalUrlOutcome::Rejected`]); deduplication never applies to
//!    rejections, which are deterministic on their own.
//! 4. **Burst dedup** — a repeat of an accepted web URL within the window
//!    yields [`ExternalUrlOutcome::Deduplicated`] instead of a second route.
//! 5. **Routing** — a fresh web URL yields [`ExternalUrlOutcome::Routed`]
//!    with the [`RoutingDecision`] the adapter must execute.
//!
//! The store is panic-free by construction: every fallible path returns a
//! typed result, and no internal operation can panic.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::external_urls::{
    classify, decide_routing, BurstConfig, BurstTracker, DedupVerdict, ExternalUrlRequest,
    ExternalUrlRequestId, RejectionReason, RoutingDecision, RoutingInput, UrlClass,
};

/// What one handled request produced.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalUrlOutcome {
    /// An accepted web URL routed to a window decision the adapter must
    /// execute.
    Routed {
        request: ExternalUrlRequestId,
        decision: RoutingDecision,
    },
    /// The same URL was already routed within the burst window; no second
    /// navigation happens.
    Deduplicated {
        request: ExternalUrlRequestId,
        /// The request id of the first (routed) observation.
        original: ExternalUrlRequestId,
        /// Total observations of this URL so far, including this one.
        observations: u64,
    },
    /// The URL must not be opened.
    Rejected {
        request: ExternalUrlRequestId,
        reason: RejectionReason,
    },
}

/// A contract violation by the caller. The store is unchanged when one is
/// returned. Non-serde by design, like the sibling contracts' control-path
/// errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalUrlError {
    /// A request id was reused; ids must increase for the store's lifetime.
    DuplicateRequestId {
        id: ExternalUrlRequestId,
        last: ExternalUrlRequestId,
    },
    /// A tick went backwards; ticks must be non-decreasing.
    NonMonotonicTick { tick: u64, last: u64 },
}

/// The deterministic external-URL store.
///
/// One coordinator serves the whole app for its lifetime: it owns the
/// monotonic request-id watermark and tick watermark, so the adapter
/// allocates increasing ids and supplies non-decreasing ticks per instance.
#[derive(Debug, Clone)]
pub struct ExternalUrlCoordinator {
    dedup: BurstTracker,
    last_request_id: Option<ExternalUrlRequestId>,
    last_tick: u64,
}

impl ExternalUrlCoordinator {
    /// A store with the given bounded-burst configuration. The configuration
    /// is invariant-holding ([`BurstConfig::new`] already validated it).
    pub fn new(config: BurstConfig) -> Self {
        Self {
            dedup: BurstTracker::new(config),
            last_request_id: None,
            last_tick: 0,
        }
    }

    /// Handle one external URL open at `tick` against the given window
    /// state. See the module docs for the guard order.
    pub fn handle(
        &mut self,
        request: ExternalUrlRequest,
        tick: u64,
        input: &RoutingInput<'_>,
    ) -> Result<ExternalUrlOutcome, ExternalUrlError> {
        if let Some(last) = self.last_request_id {
            if request.id <= last {
                return Err(ExternalUrlError::DuplicateRequestId {
                    id: request.id,
                    last,
                });
            }
        }
        if tick < self.last_tick {
            return Err(ExternalUrlError::NonMonotonicTick {
                tick,
                last: self.last_tick,
            });
        }
        self.last_request_id = Some(request.id);
        self.last_tick = tick;

        match classify(&request.raw_url) {
            UrlClass::Web(url) => match self.dedup.observe(tick, &url, request.id) {
                DedupVerdict::Fresh => Ok(ExternalUrlOutcome::Routed {
                    request: request.id,
                    decision: decide_routing(input),
                }),
                DedupVerdict::Duplicate {
                    original,
                    observations,
                } => Ok(ExternalUrlOutcome::Deduplicated {
                    request: request.id,
                    original,
                    observations,
                }),
            },
            UrlClass::DangerousScheme { scheme } => Ok(ExternalUrlOutcome::Rejected {
                request: request.id,
                reason: RejectionReason::DangerousScheme { scheme },
            }),
            UrlClass::UnsupportedScheme { scheme } => Ok(ExternalUrlOutcome::Rejected {
                request: request.id,
                reason: RejectionReason::UnsupportedScheme { scheme },
            }),
            UrlClass::Malformed(malformed) => Ok(ExternalUrlOutcome::Rejected {
                request: request.id,
                reason: RejectionReason::Malformed(malformed),
            }),
        }
    }
}
