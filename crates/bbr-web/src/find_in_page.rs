//! # Find-in-page — portable capability contract
//!
//! Typed vocabulary and pure session rules for find-in-page. No framework,
//! GPUI, `unsafe` code, or dependencies.
//!
//! This module deliberately does not extend [`crate::WebEngine`]. The native
//! adapter, testkit wire, and UI are serial integration surfaces. This module
//! supplies their portable contract: normalized queries, matching options,
//! navigation direction, engine results, request correlation, and lifecycle
//! errors.
//!
//! ## Explicit semantics
//!
//! - **Empty query:** queries are trimmed. Empty or whitespace-only input is
//!   rejected with [`FindError::EmptyQuery`] and leaves the session unchanged.
//! - **Zero matches:** a non-empty query with no matches is active with count
//!   `0` and ordinal `None`; navigation returns [`FindError::NoMatches`].
//! - **Ordinals:** [`FindResult::current_match`] is 1-based. The native
//!   adapter must provide both total count and ordinal. WKWebView's direct
//!   `WKFindResult` only reports a boolean, so the adapter needs an explicit
//!   counting/search strategy for this contract.
//! - **Wrap-around:** wrapping is enabled by default. Without it, a boundary
//!   navigation request is a deterministic no-op once its result is reported.
//! - **Stale results:** document changes call [`FindSession::invalidate`]. Old
//!   callbacks are rejected, including callbacks arriving after a replacement
//!   [`FindSession::start`] or an invalidate-then-restart sequence.
//! - **Asynchronous requests:** `start` and navigation return a unique
//!   [`FindRequestId`]. The callback must report that exact id. While a request
//!   is pending, result access and another navigation return
//!   [`FindError::ResultsPending`].
//!
//! `whole_word` is part of the portable request semantics. It is not a
//! `WKFindConfiguration` flag; the native adapter must implement it in the
//! same search strategy used to count and navigate matches, or reject it with
//! [`FindError::UnsupportedOption`].

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

/// Options controlling a find-in-page run.
///
/// Default: case-insensitive substring matching with wrap-around navigation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindOptions {
    /// Match case-sensitively.
    pub case_sensitive: bool,
    /// Match whole words rather than substrings. The adapter owns the
    /// boundary semantics because native find configuration may not expose it.
    pub whole_word: bool,
    /// Wrap at the first/last match.
    pub wrap_around: bool,
}

impl FindOptions {
    pub const fn new(case_sensitive: bool, whole_word: bool, wrap_around: bool) -> Self {
        Self {
            case_sensitive,
            whole_word,
            wrap_around,
        }
    }
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            wrap_around: true,
        }
    }
}

/// Direction of a find-navigation request.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindDirection {
    Next,
    Previous,
}

/// Correlates a native find request with its asynchronous completion.
///
/// Identities are unique within a [`FindSession`] lifetime and are never
/// reused after replacement or invalidation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindRequestId(u64);

impl FindRequestId {
    fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Engine-reported find results for one request.
///
/// `current_match` is a 1-based ordinal, or `None` exactly when the count is
/// zero.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindResult {
    pub match_count: usize,
    pub current_match: Option<usize>,
}

impl FindResult {
    /// Whether the count/ordinal pair is internally consistent.
    pub fn is_consistent(&self) -> bool {
        match self.current_match {
            None => self.match_count == 0,
            Some(ordinal) => ordinal >= 1 && ordinal <= self.match_count,
        }
    }
}

/// Typed failure of a find-in-page operation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindError {
    /// No find session exists.
    Inactive,
    /// The query is empty or whitespace-only.
    EmptyQuery,
    /// A native result is still pending.
    ResultsPending,
    /// No request is currently awaiting a native result.
    NoRequestPending,
    /// The query has zero matches.
    NoMatches,
    /// The native adapter cannot honor a requested option.
    UnsupportedOption,
    /// A document change made results unusable.
    StaleResults,
    /// The engine reported an inconsistent count/ordinal pair.
    InvalidResult,
}

impl fmt::Display for FindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Inactive => "no active find-in-page session",
            Self::EmptyQuery => "find query is empty (or whitespace only)",
            Self::ResultsPending => "find results not yet reported by the engine",
            Self::NoRequestPending => "no find request is awaiting an engine result",
            Self::NoMatches => "the query has no matches in the page",
            Self::UnsupportedOption => {
                "the native adapter does not support a requested find option"
            }
            Self::StaleResults => {
                "find results are stale; re-run the find after the document changed"
            }
            Self::InvalidResult => "engine reported an inconsistent match count/ordinal",
        };
        f.write_str(message)
    }
}

impl std::error::Error for FindError {}

/// Trim leading and trailing whitespace from a query.
pub fn normalize_query(raw: &str) -> String {
    raw.trim().to_owned()
}

/// Whether a query is empty after trimming.
pub fn is_empty_query(raw: &str) -> bool {
    raw.trim().is_empty()
}

/// Pure state machine for one find-in-page session.
///
/// A request is accepted by [`start`](Self::start), then completed by
/// [`report`](Self::report) with the returned [`FindRequestId`]. Navigation is
/// likewise request/response based; it does not optimistically mutate the
/// current ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindSession {
    state: FindState,
    next_request_id: u64,
}

impl Default for FindSession {
    fn default() -> Self {
        Self {
            state: FindState::Idle,
            next_request_id: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FindState {
    Idle,
    Pending {
        query: String,
        options: FindOptions,
        request_id: FindRequestId,
    },
    Active {
        query: String,
        options: FindOptions,
        result: FindResult,
    },
    NavigationPending {
        query: String,
        options: FindOptions,
        result: FindResult,
        request_id: FindRequestId,
        direction: FindDirection,
    },
    Stale {
        query: String,
        options: FindOptions,
    },
}

impl FindSession {
    /// Start or replace a find. Returns the id required by its completion.
    pub fn start(&mut self, query: &str, options: FindOptions) -> Result<FindRequestId, FindError> {
        let query = normalize_query(query);
        if query.is_empty() {
            return Err(FindError::EmptyQuery);
        }
        let request_id = self.allocate_request_id();
        self.state = FindState::Pending {
            query,
            options,
            request_id,
        };
        Ok(request_id)
    }

    /// Complete the currently pending start or navigation request.
    ///
    /// A mismatched id is rejected as [`FindError::StaleResults`]. Invalid
    /// result data leaves the pending request unchanged.
    pub fn report(
        &mut self,
        request_id: FindRequestId,
        result: FindResult,
    ) -> Result<(), FindError> {
        let (query, options, expected) = match &self.state {
            FindState::Pending {
                query,
                options,
                request_id: expected,
            }
            | FindState::NavigationPending {
                query,
                options,
                request_id: expected,
                ..
            } => (query.clone(), *options, *expected),
            FindState::Idle => return Err(FindError::Inactive),
            FindState::Active { .. } => return Err(FindError::NoRequestPending),
            FindState::Stale { .. } => return Err(FindError::StaleResults),
        };
        if request_id != expected {
            return Err(FindError::StaleResults);
        }
        if !result.is_consistent() {
            return Err(FindError::InvalidResult);
        }
        self.state = FindState::Active {
            query,
            options,
            result,
        };
        Ok(())
    }

    /// Request the next match and return its correlation id.
    pub fn find_next(&mut self) -> Result<FindRequestId, FindError> {
        self.step(FindDirection::Next)
    }

    /// Request the previous match and return its correlation id.
    pub fn find_previous(&mut self) -> Result<FindRequestId, FindError> {
        self.step(FindDirection::Previous)
    }

    /// End the session and clear query/results/highlights at the adapter.
    pub fn finish(&mut self) {
        self.state = FindState::Idle;
    }

    /// Mark all current and in-flight results stale after a document change.
    pub fn invalidate(&mut self) {
        self.state = match &self.state {
            FindState::Pending { query, options, .. }
            | FindState::Active { query, options, .. }
            | FindState::NavigationPending { query, options, .. } => FindState::Stale {
                query: query.clone(),
                options: *options,
            },
            FindState::Idle | FindState::Stale { .. } => return,
        };
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, FindState::Idle)
    }

    pub fn is_pending(&self) -> bool {
        matches!(
            self.state,
            FindState::Pending { .. } | FindState::NavigationPending { .. }
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, FindState::Active { .. })
    }

    pub fn is_stale(&self) -> bool {
        matches!(self.state, FindState::Stale { .. })
    }

    /// The trimmed query, or `None` when idle.
    pub fn query(&self) -> Option<&str> {
        match &self.state {
            FindState::Idle => None,
            FindState::Pending { query, .. }
            | FindState::Active { query, .. }
            | FindState::NavigationPending { query, .. }
            | FindState::Stale { query, .. } => Some(query),
        }
    }

    /// Current options, or defaults when idle.
    pub fn options(&self) -> FindOptions {
        match &self.state {
            FindState::Idle => FindOptions::default(),
            FindState::Pending { options, .. }
            | FindState::Active { options, .. }
            | FindState::NavigationPending { options, .. }
            | FindState::Stale { options, .. } => *options,
        }
    }

    /// Total matches, or an error when results are unavailable.
    pub fn match_count(&self) -> Result<usize, FindError> {
        self.result().map(|result| result.match_count)
    }

    /// Current 1-based ordinal, or `None` for zero matches.
    pub fn current_match(&self) -> Result<Option<usize>, FindError> {
        self.result().map(|result| result.current_match)
    }

    fn allocate_request_id(&mut self) -> FindRequestId {
        let id = FindRequestId::new(self.next_request_id);
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .expect("find request id exhausted");
        id
    }

    fn result(&self) -> Result<FindResult, FindError> {
        match &self.state {
            FindState::Active { result, .. } => Ok(*result),
            FindState::NavigationPending { .. } | FindState::Pending { .. } => {
                Err(FindError::ResultsPending)
            }
            FindState::Idle => Err(FindError::Inactive),
            FindState::Stale { .. } => Err(FindError::StaleResults),
        }
    }

    fn step(&mut self, direction: FindDirection) -> Result<FindRequestId, FindError> {
        let (query, options, result) = match &self.state {
            FindState::Active {
                query,
                options,
                result,
            } => (query.clone(), *options, *result),
            FindState::Idle => return Err(FindError::Inactive),
            FindState::Pending { .. } | FindState::NavigationPending { .. } => {
                return Err(FindError::ResultsPending)
            }
            FindState::Stale { .. } => return Err(FindError::StaleResults),
        };
        if result.match_count == 0 {
            return Err(FindError::NoMatches);
        }
        let request_id = self.allocate_request_id();
        self.state = FindState::NavigationPending {
            query,
            options,
            result,
            request_id,
            direction,
        };
        Ok(request_id)
    }
}

#[cfg(test)]
mod tests;
