//! Terminal-history persistence seams (DESIGN.md §17.10 settled).
//!
//! The controller performs no I/O; these are the portable facts a
//! bbr-persistence-backed store needs: reconstruct a terminal record from
//! persisted data ([`DownloadRecord::restored_terminal`]), export the
//! terminal records in persisted order ([`DownloadController::terminal_history`]),
//! restore them at startup ([`DownloadController::restore_terminal_history`]),
//! and the Clear / row-Delete mutations ([`DownloadController::clear_history`],
//! [`DownloadController::delete_history`]).
//!
//! Policy (the settled post-MVP decisions):
//! - **Terminal history persists through restarts**; live attempts are
//!   never exported and never resurrected as running.
//! - **Clear empties the controller history** so deleted history cannot
//!   return from this authority; the returned ids are the persistence
//!   delta the store deletes transactionally.
//! - **Delete removes one terminal record**; a live attempt is never
//!   deleted from under the engine.
//! - A **restored Cancelled row is restartable** through the same machine
//!   path as a failed retry (fresh attempt linked via `retry_of`), and its
//!   destination reconstructs as the settled silent suggested-destination
//!   policy so a restart re-runs the standard folder.

use bbr_web::downloads::{DestinationDecision, DownloadId, DownloadState};

use super::{DownloadController, DownloadError, DownloadRecord};

/// The persisted history facts for one terminal attempt — the typed
/// restore-input structure the app composition builds from the repository
/// DTO (DESIGN.md §17.10 settled: terminal history persists through
/// restarts). Grouping the nine fields keeps the restore seam cohesive and
/// self-documenting: a restore is named, not positional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoredTerminal {
    /// The attempt identity (`DownloadId`).
    pub id: DownloadId,
    /// The source URL (required to Restart/Retry a restored row).
    pub url: String,
    /// The display filename (the panel's name line).
    pub filename: String,
    /// The terminal machine state (`Succeeded`/`Failed`/`Cancelled`).
    pub state: DownloadState,
    /// The resolved destination path, when the attempt reached one.
    pub resolved_path: Option<String>,
    /// The size observed at the terminal transition.
    pub final_bytes: Option<u64>,
    /// The persisted monotonic terminal timestamp.
    pub terminal_at: u64,
    /// The previous attempt this row retries/restarts, when applicable.
    pub retry_of: Option<DownloadId>,
    /// The 1-based attempt count.
    pub attempt: u32,
}

impl DownloadRecord {
    /// The resolved destination while observable (L6
    /// [`DownloadState::resolved_destination`]).
    pub fn resolved_destination(&self) -> Option<&str> {
        self.state.resolved_destination()
    }

    /// Reconstruct a terminal record from persisted history data (the app
    /// composition maps the repository DTO onto a [`RestoredTerminal`];
    /// DESIGN.md §17.10 settled — terminal history persists through
    /// restarts).
    ///
    /// `state` must be terminal (`Succeeded`/`Failed`/`Cancelled`): active
    /// attempts are never resurrected as running. The destination decision
    /// is reconstructed as the v1 settled policy's silent suggested
    /// destination ([`DestinationDecision::Suggested`] with the display
    /// filename), so a later Restart/Retry of a restored row re-runs the
    /// standard suggested-destination folder rather than inventing an
    /// explicit path. `final_bytes` is the size observed at the terminal
    /// transition; `terminal_at` is the persisted monotonic terminal
    /// timestamp; `started_at` is not persisted and reconstructs as `0`.
    pub fn restored_terminal(input: RestoredTerminal) -> Result<Self, DownloadError> {
        if !input.state.is_terminal() {
            return Err(DownloadError::RestoreNonTerminal { id: input.id });
        }
        Ok(Self {
            id: input.id,
            url: input.url,
            filename: input.filename.clone(),
            destination: DestinationDecision::Suggested(input.filename),
            state: input.state,
            resolved_path: input.resolved_path,
            started_at: 0,
            terminal_at: Some(input.terminal_at),
            final_bytes: input.final_bytes,
            terminal_seq: None,
            attempt: input.attempt,
            retry_of: input.retry_of,
            retries: Vec::new(),
        })
    }
}

impl DownloadController {
    /// Remove every terminal record (the settled Clear action, DESIGN.md
    /// §17.7): the controller history is emptied so deleted history cannot
    /// return from this authority. Live (non-terminal) attempts are kept.
    /// Returns the removed ids so the caller can delete the persisted rows
    /// transactionally.
    pub fn clear_history(&mut self) -> Vec<DownloadId> {
        let terminal = self.history_ids();
        for id in &terminal {
            self.attempts.remove(id);
        }
        terminal
    }

    /// Remove one terminal record (the settled row Delete action, DESIGN.md
    /// §17.8). A live (non-terminal) attempt is rejected
    /// ([`DownloadError::DeleteNonTerminal`]) — a running transfer is never
    /// deleted from under the engine. Returns the removed id so the caller
    /// can delete the persisted row transactionally.
    pub fn delete_history(&mut self, id: DownloadId) -> Result<DownloadId, DownloadError> {
        let record = self
            .attempts
            .get(&id)
            .ok_or(DownloadError::UnknownAttempt(id))?;
        if !record.state.is_terminal() {
            return Err(DownloadError::DeleteNonTerminal { id });
        }
        self.attempts.remove(&id);
        Ok(id)
    }

    /// Replace the entire record set with the persisted terminal history
    /// (startup restore / restart; DESIGN.md §17.10 settled).
    ///
    /// The records arrive in **ascending terminal order** (oldest first —
    /// the persisted order, repository position = index); `terminal_seq` is
    /// reassigned `1..=n` in that order so History keeps rendering
    /// most-recent-terminal-first and the ordering survives restarts. Every
    /// current record is dropped — active attempts are **never resurrected**
    /// as running (the settled persistence contract; an in-flight transfer
    /// dies with the process and is not modelled as running again).
    /// `next_id` advances past the largest restored id so fresh attempts can
    /// never collide with restored history identities.
    pub fn restore_terminal_history(
        &mut self,
        records: Vec<DownloadRecord>,
    ) -> Result<(), DownloadError> {
        for record in &records {
            if !record.state.is_terminal() {
                return Err(DownloadError::RestoreNonTerminal { id: record.id });
            }
        }
        self.attempts.clear();
        let mut seq = 0_u64;
        let mut max_id = 0_u64;
        for mut record in records {
            seq += 1;
            record.terminal_seq = Some(seq);
            if record.id.0 > max_id {
                max_id = record.id.0;
            }
            self.attempts.insert(record.id, record);
        }
        self.terminal_seq = seq;
        self.next_id = max_id + 1;
        Ok(())
    }

    /// The complete terminal history in **ascending terminal order**
    /// (oldest first) — the persisted order the repository stores as
    /// `position` (DESIGN.md §17.10). Terminal records only; live attempts
    /// are never persisted.
    pub fn terminal_history(&self) -> Vec<DownloadRecord> {
        let mut records: Vec<DownloadRecord> = self
            .attempts
            .values()
            .filter(|record| record.state.is_terminal())
            .cloned()
            .collect();
        records.sort_unstable_by_key(|record| record.terminal_seq.unwrap_or(0));
        records
    }
}
