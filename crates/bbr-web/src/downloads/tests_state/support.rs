//! Downloads state-machine tests: the exhaustive state × event matrix,
//! byte-progress monotonicity and total stability, success byte accounting,
//! destination-decision timing, terminal-state immunity, cancellation
//! legality, and retry legality (the terminal-state/retry rules).

use crate::downloads::{
    DestinationDecision, DownloadEvent, DownloadFailure, DownloadId, DownloadProgress,
    DownloadState, EventRejection, StartRequest,
};

pub(super) const DEST: &str = "/tmp/report.pdf";
pub(super) const FAILURE: DownloadFailure = DownloadFailure::Network;

pub(super) fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

pub(super) fn progress(received: u64, total: Option<u64>) -> DownloadProgress {
    DownloadProgress::new(received, total).expect("valid test progress")
}

/// Canonical progress for the exhaustive matrix: complete (100/100) so both a
/// further equal progress and `Succeeded` are legal for the same `Running`
/// state.
pub(super) fn full_progress() -> DownloadProgress {
    progress(100, Some(100))
}

pub(super) fn states() -> Vec<(&'static str, DownloadState)> {
    vec![
        ("Waiting-None", DownloadState::Waiting { destination: None }),
        (
            "Waiting-Some",
            DownloadState::Waiting {
                destination: Some(DEST.into()),
            },
        ),
        (
            "Running",
            DownloadState::Running {
                destination: DEST.into(),
                progress: full_progress(),
            },
        ),
        ("Succeeded", DownloadState::Succeeded),
        ("Failed", DownloadState::Failed { failure: FAILURE }),
        ("Cancelled", DownloadState::Cancelled),
    ]
}

pub(super) fn events() -> Vec<(&'static str, DownloadEvent)> {
    vec![
        (
            "DestinationResolved",
            DownloadEvent::DestinationResolved {
                id: id(1),
                destination: DEST.into(),
            },
        ),
        (
            "Progress",
            DownloadEvent::Progress {
                id: id(1),
                progress: full_progress(),
            },
        ),
        ("Succeeded", DownloadEvent::Succeeded { id: id(1) }),
        (
            "Failed",
            DownloadEvent::Failed {
                id: id(1),
                failure: FAILURE,
            },
        ),
        ("Cancelled", DownloadEvent::Cancelled { id: id(1) }),
    ]
}

/// The hand-derived expectation for one (state, event) pair of the matrix.
/// Exact payloads (resolved destination, carried progress, failure reason)
/// are asserted by the focused tests below; this function only pins which
/// transitions are legal and with which rejection class the illegal ones die.
pub(super) fn expected_outcome(state: &str, event: &str) -> &'static str {
    match (state, event) {
        ("Waiting-None", "DestinationResolved") => "Ok Waiting-Some",
        ("Waiting-None", "Progress") => "Err UndecidedDestination",
        ("Waiting-None", "Succeeded") => "Err UndecidedDestination",
        ("Waiting-None", "Failed") => "Ok Failed",
        ("Waiting-None", "Cancelled") => "Ok Cancelled",
        ("Waiting-Some", "DestinationResolved") => "Err AlreadyResolved",
        ("Waiting-Some", "Progress") => "Ok Running",
        ("Waiting-Some", "Succeeded") => "Ok Succeeded",
        ("Waiting-Some", "Failed") => "Ok Failed",
        ("Waiting-Some", "Cancelled") => "Ok Cancelled",
        ("Running", "DestinationResolved") => "Err DestinationAfterProgress",
        ("Running", "Progress") => "Ok Running",
        ("Running", "Succeeded") => "Ok Succeeded",
        ("Running", "Failed") => "Ok Failed",
        ("Running", "Cancelled") => "Ok Cancelled",
        ("Succeeded" | "Failed" | "Cancelled", _) => "Err AlreadyTerminal",
        (s, e) => panic!("missing matrix expectation for {s} × {e}"),
    }
}

pub(super) fn classify(result: Result<DownloadState, EventRejection>) -> &'static str {
    match result {
        Ok(DownloadState::Waiting { destination: None }) => "Ok Waiting-None",
        Ok(DownloadState::Waiting {
            destination: Some(_),
        }) => "Ok Waiting-Some",
        Ok(DownloadState::Running { .. }) => "Ok Running",
        Ok(DownloadState::Succeeded) => "Ok Succeeded",
        Ok(DownloadState::Failed { .. }) => "Ok Failed",
        Ok(DownloadState::Cancelled) => "Ok Cancelled",
        Err(EventRejection::AlreadyTerminal { .. }) => "Err AlreadyTerminal",
        Err(EventRejection::UndecidedDestination) => "Err UndecidedDestination",
        Err(EventRejection::AlreadyResolved { .. }) => "Err AlreadyResolved",
        Err(EventRejection::DestinationAfterProgress) => "Err DestinationAfterProgress",
        Err(EventRejection::ProgressRegressed { .. }) => "Err ProgressRegressed",
        Err(EventRejection::TotalChanged { .. }) => "Err TotalChanged",
        Err(EventRejection::SuccessByteMismatch { .. }) => "Err SuccessByteMismatch",
        Err(EventRejection::MismatchedId { .. }) => "Err MismatchedId",
        Err(EventRejection::EmptyDestination) => "Err EmptyDestination",
    }
}

pub(super) fn retry_request() -> StartRequest {
    StartRequest {
        id: id(2),
        url: "https://example.com/report.pdf".into(),
        destination: DestinationDecision::Suggested("report.pdf".into()),
        retry_of: Some(id(1)),
    }
}
