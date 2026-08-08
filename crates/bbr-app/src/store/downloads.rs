//! Downloads persistence ↔ controller mapping (DESIGN.md §17.10 settled).
//!
//! The app-owned composition between the persisted terminal records
//! ([`bbr_persistence::DownloadRecord`]) and the portable downloads
//! controller records ([`bbr_browser::DownloadRecord`]): `from_persisted`
//! maps the restored store rows onto the typed [`RestoredTerminal`] inputs
//! the controller's startup restore consumes, and `to_persisted` exports
//! the controller's terminal history in the persisted order for the
//! repository's whole-replace write. Both directions are pure and
//! total, so the mapping is unit-testable without GPUI or SQLite.
//!
//! Only **terminal** outcomes map: `Succeeded`/`Failed`/`Cancelled`. Live
//! attempts are never persisted and never resurrected (the controller
//! rejects a non-terminal restore). A restored `Failed` row carries the
//! generic `Unknown` failure (the terminal outcome is persisted, not the
//! failure reason); its Retry action still starts a fresh linked attempt.
//! The resolved destination is persisted when the attempt reached one
//! (`destination`); v1's silent suggested-destination policy means a
//! restored row re-runs the standard folder on Restart/Retry.

use bbr_browser::RestoredTerminal;
use bbr_persistence::DownloadOutcome;
use bbr_web::downloads::{DownloadFailure, DownloadId, DownloadState};

/// Maps the restored persisted terminal records onto the typed restore
/// inputs the controller's startup restore consumes, in persisted order
/// (ascending terminal order). Every record is terminal by construction:
/// the repository validates outcomes on load and the schema stores only
/// terminal attempts, so the controller restore can never reject a
/// non-terminal record here.
pub fn from_persisted(records: &[bbr_persistence::DownloadRecord]) -> Vec<RestoredTerminal> {
    records
        .iter()
        .map(|record| RestoredTerminal {
            id: DownloadId(record.id),
            url: record.url.clone(),
            filename: record.filename.clone(),
            state: match record.outcome {
                DownloadOutcome::Succeeded => DownloadState::Succeeded,
                DownloadOutcome::Failed => DownloadState::Failed {
                    failure: DownloadFailure::Unknown,
                },
                DownloadOutcome::Cancelled => DownloadState::Cancelled,
            },
            resolved_path: record.destination.clone(),
            final_bytes: Some(record.size),
            terminal_at: record.terminal_at,
            retry_of: record.retry_of.map(DownloadId),
            attempt: record.attempt,
        })
        .collect()
}

/// Exports the controller's terminal history onto the persisted DTOs in
/// the persisted order (ascending terminal order; `position` = index) —
/// the exact input the repository's `replace_downloads` whole-replace
/// write consumes. Terminal records only; the controller's
/// `terminal_history()` already filters to terminal attempts, so a live
/// attempt can never reach the store.
pub fn to_persisted(
    records: &[bbr_browser::DownloadRecord],
) -> Vec<bbr_persistence::DownloadRecord> {
    records
        .iter()
        .map(|record| bbr_persistence::DownloadRecord {
            id: record.id.0,
            outcome: match &record.state {
                DownloadState::Succeeded => DownloadOutcome::Succeeded,
                DownloadState::Failed { .. } => DownloadOutcome::Failed,
                DownloadState::Cancelled => DownloadOutcome::Cancelled,
                // The controller's terminal-history export never yields a
                // live state; a non-terminal record is a composition bug and
                // must not silently reach the store.
                _ => panic!("non-terminal download record exported for persistence"),
            },
            size: record.final_bytes.unwrap_or(0),
            destination: record.resolved_path.clone(),
            terminal_at: record.terminal_at.unwrap_or(0),
            url: record.url.clone(),
            filename: record.filename.clone(),
            retry_of: record.retry_of.map(|id| id.0),
            attempt: record.attempt,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_browser::DownloadController;
    use bbr_web::downloads::{DestinationDecision, DownloadEvent};

    fn persisted(
        id: u64,
        outcome: DownloadOutcome,
        destination: Option<&str>,
        retry_of: Option<u64>,
    ) -> bbr_persistence::DownloadRecord {
        bbr_persistence::DownloadRecord {
            id,
            outcome,
            size: 4096,
            destination: destination.map(str::to_owned),
            terminal_at: 1000 + id,
            url: format!("https://example.test/file-{id}.bin"),
            filename: format!("file-{id}.bin"),
            retry_of,
            attempt: if retry_of.is_some() { 2 } else { 1 },
        }
    }

    #[test]
    fn from_persisted_maps_every_terminal_outcome_and_keeps_order() {
        let records = vec![
            persisted(1, DownloadOutcome::Succeeded, Some("/tmp/file-1.bin"), None),
            persisted(2, DownloadOutcome::Failed, None, None),
            persisted(3, DownloadOutcome::Cancelled, None, Some(2)),
        ];
        let restored = from_persisted(&records);
        assert_eq!(restored.len(), 3);
        // Persisted order is preserved (ascending terminal order).
        assert_eq!(restored[0].id, DownloadId(1));
        assert_eq!(restored[0].state, DownloadState::Succeeded);
        assert_eq!(
            restored[0].resolved_path.as_deref(),
            Some("/tmp/file-1.bin")
        );
        assert_eq!(restored[0].final_bytes, Some(4096));
        assert_eq!(restored[0].attempt, 1);
        assert!(matches!(
            restored[1].state,
            DownloadState::Failed {
                failure: DownloadFailure::Unknown
            }
        ));
        // A restored Cancelled row carries its retry linkage, so the panel
        // can Restart it as a fresh linked attempt.
        assert_eq!(restored[2].state, DownloadState::Cancelled);
        assert_eq!(restored[2].retry_of, Some(DownloadId(2)));
        assert_eq!(restored[2].attempt, 2);
    }

    #[test]
    fn from_persisted_restores_terminal_records_the_controller_accepts() {
        let records = vec![
            persisted(10, DownloadOutcome::Succeeded, None, None),
            persisted(11, DownloadOutcome::Cancelled, None, Some(10)),
        ];
        let mut controller = DownloadController::new();
        let restored = from_persisted(&records)
            .into_iter()
            .map(bbr_browser::DownloadRecord::restored_terminal)
            .collect::<Result<Vec<_>, _>>()
            .expect("terminal records restore");
        controller
            .restore_terminal_history(restored)
            .expect("restore accepted");
        // History renders most-recent-terminal-first: the restore assigned
        // `terminal_seq` 1→10 and 2→11 (ascending persisted order), so the
        // displayed order is [11, 10].
        assert_eq!(
            controller.history_ids(),
            vec![DownloadId(11), DownloadId(10)]
        );
        // Restored rows keep their name line, linkage, and attempt count.
        let cancelled = controller.record(DownloadId(11)).unwrap();
        assert_eq!(cancelled.filename, "file-11.bin");
        assert_eq!(cancelled.retry_of, Some(DownloadId(10)));
        assert_eq!(cancelled.attempt, 2);
    }

    #[test]
    fn to_persisted_round_trips_the_controller_terminal_history() {
        let mut controller = DownloadController::new();
        let first = controller
            .start(
                "https://example.test/a.bin",
                DestinationDecision::Suggested("a.bin".into()),
            )
            .unwrap();
        let second = controller
            .start(
                "https://example.test/b.bin",
                DestinationDecision::Suggested("b.bin".into()),
            )
            .unwrap();
        controller.apply(DownloadEvent::DestinationResolved {
            id: first,
            destination: "/tmp/a.bin".into(),
        });
        controller.apply(DownloadEvent::Succeeded { id: first });
        controller.apply(DownloadEvent::DestinationResolved {
            id: second,
            destination: "/tmp/b.bin".into(),
        });
        controller.apply(DownloadEvent::Failed {
            id: second,
            failure: DownloadFailure::Network,
        });
        let persisted = to_persisted(&controller.terminal_history());
        assert_eq!(persisted.len(), 2);
        // Ascending terminal order: the succeeded attempt is first.
        assert_eq!(persisted[0].id, first.0);
        assert_eq!(persisted[0].outcome, DownloadOutcome::Succeeded);
        assert_eq!(persisted[0].destination.as_deref(), Some("/tmp/a.bin"));
        assert_eq!(persisted[1].id, second.0);
        assert_eq!(persisted[1].outcome, DownloadOutcome::Failed);
        assert_eq!(persisted[1].filename, "b.bin");
        assert_eq!(persisted[1].attempt, 1);
        // The persistence DTO round-trips back into the same terminal row.
        let restored = from_persisted(&persisted);
        assert_eq!(restored[0].state, DownloadState::Succeeded);
        assert_eq!(
            restored[1].state,
            DownloadState::Failed {
                failure: DownloadFailure::Unknown
            }
        );
    }

    #[test]
    fn to_persisted_exports_only_terminal_records() {
        let mut controller = DownloadController::new();
        controller
            .start(
                "https://example.test/live.bin",
                DestinationDecision::Suggested("live.bin".into()),
            )
            .unwrap();
        // A live (Waiting) attempt is never exported.
        assert!(controller.terminal_history().is_empty());
        assert!(to_persisted(&controller.terminal_history()).is_empty());
    }
}
