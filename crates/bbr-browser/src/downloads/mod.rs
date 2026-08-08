//! # Portable download controller and history store
//!
//! The controller-owned half of the downloads capability: it owns the
//! authoritative `DownloadId → DownloadState` map (the L6 stateless-authority
//! model), the session history records (final paths, timestamps, attempt
//! counts, retry linkage), and the deterministic row ordering the panel and
//! E2E rely on.
//!
//! This module is **fully portable**: no GPUI, no AppKit/WebKit, no
//! filesystem, no `unsafe` — only `std` and the `bbr-web` L6 contract. It is
//! the counterpart of the native download adapter
//! (`bbr-web-macos`); the composition routes [`DownloadAction`]s emitted
//! here to the adapter and feeds the adapter's [`DownloadEvent`]s back
//! through [`DownloadController::apply`].
//!
//! ## Machine fidelity (L6)
//!
//! - The panel renders exactly the L6 states; this controller never invents a
//!   transition. Every mutation goes through [`apply_event`] /
//!   [`evaluate_cancel`] / [`evaluate_retry`].
//! - **First terminal event wins:** [`apply_event`] rejects any event on a
//!   terminal state ([`EventRejection::AlreadyTerminal`]); `apply` leaves the
//!   record unchanged and reports [`ApplyOutcome::Rejected`]. A cancel is a
//!   request — the row stays active until the engine's terminal event lands.
//! - **Retry/Restart** exist on the terminal outcomes the settled post-MVP
//!   decision offers: `Failed` rows offer **Retry**, `Cancelled` rows offer
//!   **Restart** (DESIGN.md §17.8). Both are a fresh attempt id linked via
//!   `retry_of`, with the previous record's `retries` list extended.
//! - Progress is monotonic and a known total is stable — enforced by the L6
//!   machine; this controller only records what the machine accepted.
//!
//! ## Deterministic ordering (E2E contract)
//!
//! - Active: ascending [`DownloadId`] (ids are minted sequentially).
//! - History: most-recent-terminal first (a monotonic per-controller
//!   `terminal_seq`, incremented on each accepted terminal transition — no
//!   clock ties).
//!
//! ## History persistence (DESIGN.md §17.10 settled)
//!
//! Terminal history persists through restarts. This controller owns the
//! portable seams: [`DownloadController::terminal_history`] exports the
//! terminal records in ascending terminal order (the persisted order) and
//! [`DownloadController::restore_terminal_history`] restores them at
//! startup, reassigning deterministic `terminal_seq` values so the panel
//! ordering survives restarts. Live attempts are never exported and never
//! resurrected — an in-flight transfer dies with the process. The
//! controller itself performs no I/O; the app composition (F2c) attaches
//! the bbr-persistence-backed store to the app-owned persistence seam
//! during serial integration.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use bbr_web::downloads::{
    apply_event, evaluate_cancel, evaluate_retry, validate_start, CancelRejection,
    DestinationDecision, DownloadAction, DownloadEvent, DownloadId, DownloadState, EventRejection,
    RetryRejection, StartRejection, StartRequest,
};

/// The composition-facing action sink. The composition wires it to the native
/// download adapter; contract tests record the actions.
pub type DownloadActionSink = Rc<RefCell<Option<Box<dyn FnMut(DownloadAction)>>>>;

/// How a controller operation was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadError {
    /// The operation named an attempt the controller does not know.
    UnknownAttempt(DownloadId),
    /// A start request was malformed (L6 [`StartRejection`]).
    Start(StartRejection),
    /// A cancel was not legal for the attempt's state (L6
    /// [`CancelRejection`]).
    Cancel(CancelRejection),
    /// A retry/restart was not legal (L6 [`RetryRejection`]).
    Retry(RetryRejection),
    /// The row Delete action named a live (non-terminal) attempt; only
    /// terminal records can be deleted.
    DeleteNonTerminal { id: DownloadId },
    /// A restore received a non-terminal record; active attempts are never
    /// resurrected (settled persistence contract).
    RestoreNonTerminal { id: DownloadId },
}

/// The outcome of applying one engine event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The event moved the attempt forward.
    Applied,
    /// The L6 machine rejected the event (late terminal, monotonicity, total
    /// stability, destination ordering); the record is unchanged.
    Rejected(EventRejection),
    /// The event named an attempt the controller does not know (stale or
    /// misrouted event).
    Unknown(DownloadId),
}

/// One download attempt's controller record: the authoritative machine state
/// plus the history the panel renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRecord {
    /// The attempt id (fresh per retry; never reused).
    pub id: DownloadId,
    /// The source URL.
    pub url: String,
    /// The display filename (from the destination decision / URL).
    pub filename: String,
    /// The caller's destination decision (L6).
    pub destination: DestinationDecision,
    /// The authoritative L6 machine state.
    pub state: DownloadState,
    /// The resolved destination path once the engine decided it.
    pub resolved_path: Option<String>,
    /// Wall-clock start (injected clock for deterministic tests).
    pub started_at: u64,
    /// Wall-clock terminal transition, when reached.
    pub terminal_at: Option<u64>,
    /// The byte count observed at the terminal transition (the last accepted
    /// `Running` progress; for a known total `Succeeded` this equals the
    /// total). Terminal rows render it as the size in the secondary line.
    pub final_bytes: Option<u64>,
    /// Monotonic terminal ordering (history = descending).
    pub terminal_seq: Option<u64>,
    /// 1-based attempt count (increments per retry).
    pub attempt: u32,
    /// The failed attempt this attempt retries, when applicable.
    pub retry_of: Option<DownloadId>,
    /// Attempt ids spawned as retries of this record.
    pub retries: Vec<DownloadId>,
}

/// The controller-owned download/history store around the L6 machine.
///
/// Ownership shape: the controller owns the authoritative
/// `DownloadId → DownloadRecord` map, mints ids sequentially, emits
/// [`DownloadAction`]s into an installable sink, and applies engine
/// [`DownloadEvent`]s through the L6 contract. The persistence-focused
/// seams live in [`super::history`]: terminal-history export/restore and
/// the Clear/Delete mutations.
pub struct DownloadController {
    attempts: HashMap<DownloadId, DownloadRecord>,
    /// Fresh id source (ascending; drives the Active ordering).
    next_id: u64,
    /// Monotonic terminal counter (drives the History ordering).
    terminal_seq: u64,
    /// Clock for record timestamps (injectable for deterministic tests).
    now: Box<dyn Fn() -> u64>,
    /// Sink for emitted [`DownloadAction`]s (wired to the adapter).
    actions: DownloadActionSink,
}

impl std::fmt::Debug for DownloadController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadController")
            .field("attempts", &self.attempts)
            .field("next_id", &self.next_id)
            .field("terminal_seq", &self.terminal_seq)
            .finish_non_exhaustive()
    }
}

impl Default for DownloadController {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadController {
    /// Create a controller with the wall clock.
    pub fn new() -> Self {
        Self::with_clock(unix_millis)
    }

    /// Create a controller with an injected clock (deterministic tests).
    pub fn with_clock(now: impl Fn() -> u64 + 'static) -> Self {
        Self {
            attempts: HashMap::new(),
            next_id: 0,
            terminal_seq: 0,
            now: Box::new(now),
            actions: Rc::new(RefCell::new(None)),
        }
    }

    /// Install the action sink (the composition wires it to the adapter).
    pub fn set_action_sink(&mut self, sink: Box<dyn FnMut(DownloadAction)>) {
        *self.actions.borrow_mut() = Some(sink);
    }

    /// Begin a fresh download: validate the request, record the optimistic
    /// `Waiting { destination: None }` state, and emit
    /// [`DownloadAction::Start`]. Returns the minted attempt id.
    pub fn start(
        &mut self,
        url: &str,
        destination: DestinationDecision,
    ) -> Result<DownloadId, DownloadError> {
        let id = self.mint();
        let request = StartRequest {
            id,
            url: url.to_owned(),
            destination: destination.clone(),
            retry_of: None,
        };
        validate_start(&request).map_err(DownloadError::Start)?;
        let started_at = (self.now)();
        let filename = display_filename(&request.url, &request.destination);
        self.attempts.insert(
            id,
            DownloadRecord {
                id,
                url: request.url.clone(),
                filename,
                destination,
                state: DownloadState::Waiting { destination: None },
                resolved_path: None,
                started_at,
                terminal_at: None,
                final_bytes: None,
                terminal_seq: None,
                attempt: 1,
                retry_of: None,
                retries: Vec::new(),
            },
        );
        self.emit(DownloadAction::Start(request));
        Ok(id)
    }

    /// Request cancellation of `id` (L6 [`evaluate_cancel`]): legal only
    /// while the attempt is non-terminal. No optimistic state change; the
    /// engine confirms with a terminal event.
    pub fn cancel(&mut self, id: DownloadId) -> Result<(), DownloadError> {
        let state = self
            .attempts
            .get(&id)
            .ok_or(DownloadError::UnknownAttempt(id))?
            .state
            .clone();
        evaluate_cancel(&state).map_err(DownloadError::Cancel)?;
        self.emit(DownloadAction::Cancel { id });
        Ok(())
    }

    /// Retry the failed attempt `id` (or restart the cancelled attempt
    /// `id` — the settled post-MVP decision, DESIGN.md §17.8): mint a fresh
    /// attempt linked via `retry_of`, record it, and emit
    /// [`DownloadAction::Start`]. Returns the new attempt id. Both terminal
    /// outcomes are deliberate user decisions the panel offers as **Retry**
    /// (Failed) / **Restart** (Cancelled); the machine path is identical.
    pub fn retry(&mut self, id: DownloadId) -> Result<DownloadId, DownloadError> {
        let (url, destination, filename, attempt, state) = {
            let previous = self
                .attempts
                .get(&id)
                .ok_or(DownloadError::UnknownAttempt(id))?;
            (
                previous.url.clone(),
                previous.destination.clone(),
                previous.filename.clone(),
                previous.attempt,
                previous.state.clone(),
            )
        };
        let new_id = self.mint();
        let request = StartRequest {
            id: new_id,
            url,
            destination: destination.clone(),
            retry_of: Some(id),
        };
        evaluate_retry(id, &state, &request).map_err(DownloadError::Retry)?;
        let started_at = (self.now)();
        self.attempts.insert(
            new_id,
            DownloadRecord {
                id: new_id,
                url: request.url.clone(),
                filename,
                destination,
                state: DownloadState::Waiting { destination: None },
                resolved_path: None,
                started_at,
                terminal_at: None,
                final_bytes: None,
                terminal_seq: None,
                attempt: attempt + 1,
                retry_of: Some(id),
                retries: Vec::new(),
            },
        );
        if let Some(previous) = self.attempts.get_mut(&id) {
            previous.retries.push(new_id);
        }
        let request_id = request.id;
        self.emit(DownloadAction::Start(request));
        Ok(request_id)
    }

    /// Apply one engine event by id through the L6 machine. The first
    /// terminal event wins: a later event on a terminal record is reported
    /// as [`ApplyOutcome::Rejected`] and leaves the record unchanged.
    pub fn apply(&mut self, event: DownloadEvent) -> ApplyOutcome {
        let id = event_id(&event);
        let Some(record) = self.attempts.get_mut(&id) else {
            return ApplyOutcome::Unknown(id);
        };
        // The byte count at the terminal transition: the last accepted
        // `Running` progress (for a known-total `Succeeded` that equals the
        // total). Captured before the event so terminal rows can render
        // their size even though the L6 state drops progress on success.
        let pre_event_bytes = record.state.progress().map(|p| p.received());
        match apply_event(id, &record.state, event) {
            Ok(next) => {
                let was_terminal = record.state.is_terminal();
                if !was_terminal && next.is_terminal() {
                    self.terminal_seq += 1;
                    record.terminal_seq = Some(self.terminal_seq);
                    record.terminal_at = Some((self.now)());
                    record.final_bytes = record.final_bytes.or(pre_event_bytes);
                }
                // Record the resolved path as soon as the machine accepts it
                // (`Waiting { Some }` or `Running`).
                if record.resolved_path.is_none() {
                    if let Some(path) = next.resolved_destination() {
                        record.resolved_path = Some(path.to_owned());
                    }
                }
                record.state = next;
                ApplyOutcome::Applied
            }
            Err(rejection) => ApplyOutcome::Rejected(rejection),
        }
    }

    /// The record for `id`.
    pub fn record(&self, id: DownloadId) -> Option<&DownloadRecord> {
        self.attempts.get(&id)
    }

    /// Active attempts (non-terminal) in ascending [`DownloadId`] order —
    /// the Active group's stable row order.
    pub fn active_ids(&self) -> Vec<DownloadId> {
        let mut ids: Vec<DownloadId> = self
            .attempts
            .values()
            .filter(|record| !record.state.is_terminal())
            .map(|record| record.id)
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Terminal attempts in most-recent-terminal-first order (descending
    /// `terminal_seq`) — the History group's stable row order.
    pub fn history_ids(&self) -> Vec<DownloadId> {
        let mut ids: Vec<DownloadId> = self
            .attempts
            .values()
            .filter(|record| record.state.is_terminal())
            .map(|record| record.id)
            .collect();
        // Stable: descending terminal_seq (ties impossible — seq is unique).
        ids.sort_unstable_by_key(|id| std::cmp::Reverse(self.attempts[id].terminal_seq));
        ids
    }

    /// Active then History — the full deterministic panel row order, so E2E
    /// row index `<n>` is stable.
    pub fn rows(&self) -> Vec<DownloadId> {
        let mut rows = self.active_ids();
        rows.extend(self.history_ids());
        rows
    }

    pub fn reveal_candidate(&self, id: DownloadId) -> Option<&str> {
        let record = self.attempts.get(&id)?;
        match record.state {
            DownloadState::Succeeded => record.resolved_path.as_deref(),
            _ => None,
        }
    }

    /// Number of recorded attempts.
    pub fn len(&self) -> usize {
        self.attempts.len()
    }

    /// Whether any attempt is recorded.
    pub fn is_empty(&self) -> bool {
        self.attempts.is_empty()
    }

    /// The next attempt id (test/diagnostic access).
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    fn mint(&mut self) -> DownloadId {
        let id = DownloadId(self.next_id);
        self.next_id += 1;
        id
    }

    fn emit(&self, action: DownloadAction) {
        if let Some(sink) = self.actions.borrow_mut().as_mut() {
            sink(action);
        }
    }
}

fn event_id(event: &DownloadEvent) -> DownloadId {
    match event {
        DownloadEvent::DestinationResolved { id, .. }
        | DownloadEvent::Progress { id, .. }
        | DownloadEvent::Succeeded { id }
        | DownloadEvent::Failed { id, .. }
        | DownloadEvent::Cancelled { id } => *id,
    }
}

/// The unix epoch in milliseconds (the default controller clock).
fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The display filename for a start request (pure; the panel's name line).
pub fn display_filename(url: &str, destination: &DestinationDecision) -> String {
    match destination {
        DestinationDecision::Explicit(path) => path
            .rsplit('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or(path)
            .to_owned(),
        DestinationDecision::Suggested(name) if !name.is_empty() => name.clone(),
        DestinationDecision::Suggested(_) => url_filename(url),
    }
}

/// Derive a suggested filename from a URL: the last non-empty path segment
/// (after the scheme+authority, with any query/fragment stripped); falls back
/// to `"download"` for URLs with no usable segment.
pub fn url_filename(url: &str) -> String {
    let trimmed = url.trim();
    let path = trimmed.split(['?', '#']).next().unwrap_or(trimmed);
    // Skip the scheme and authority; only the path portion may name a file.
    let after_authority = match path.find("://") {
        Some(idx) => {
            let rest = &path[idx + 3..];
            match rest.find('/') {
                Some(slash) => &rest[slash + 1..],
                None => "",
            }
        }
        None => path,
    };
    after_authority
        .rsplit('/')
        .find(|segment| !segment.is_empty())
        .unwrap_or("download")
        .to_owned()
}

#[cfg(test)]
mod tests;

mod history;

pub use history::RestoredTerminal;
