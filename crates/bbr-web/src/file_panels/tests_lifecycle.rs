//! Lifecycle and port contract tests: the deterministic state matrix,
//! selection legality (single/multiple cardinality, file/directory
//! capability), completion identity and routing, object safety, and
//! duplicate/stale/mismatch rejection — all without filesystem or native
//! types. The session state-machine and selection-legality matrix lives
//! here; coordinator routing, object safety, and exact delivery live in
//! [`tests_port`](self::tests_port).

use crate::file_panels::{
    AcceptFilter, CaptureHint, FilePanelSession, FileRequest, FileRequestId, FileRequestKind,
    FileRequestRejection, LifecycleError, LifecycleState, ReplacementPolicy, RequestContext,
    SaveCompletion, SaveDestinationToken, SaveRequest, SelectedItemKind, SelectionCapability,
    SelectionCardinality, SelectionRejection, UploadCompletion, UploadRequest, UploadSelectionItem,
    UploadSelectionToken,
};
use crate::{ProfileId, WebViewId};

fn id(n: u64) -> FileRequestId {
    FileRequestId(n)
}

fn profile(n: u64) -> ProfileId {
    ProfileId(n)
}

fn view(n: u64) -> WebViewId {
    WebViewId(n)
}

fn context() -> RequestContext {
    RequestContext::new(Some("https://example.com".into()), true)
}

/// Canonical well-formed single-file upload request (id 1).
fn upload_request() -> UploadRequest {
    UploadRequest {
        id: id(1),
        profile: profile(1),
        web_view: view(1),
        context: context(),
        accept: AcceptFilter::any(),
        cardinality: SelectionCardinality::Single,
        capability: SelectionCapability::default(),
        capture: CaptureHint::Any,
    }
}

fn save_request() -> SaveRequest {
    SaveRequest {
        id: id(2),
        profile: profile(1),
        web_view: view(1),
        context: context(),
        suggested_name: Some("report.pdf".into()),
        content_type: Some("application/pdf".into()),
        replacement: ReplacementPolicy::Uniquify,
    }
}

fn file(n: u64) -> UploadSelectionItem {
    UploadSelectionItem::new(SelectedItemKind::File, UploadSelectionToken::new(n))
}

fn directory(n: u64) -> UploadSelectionItem {
    UploadSelectionItem::new(SelectedItemKind::Directory, UploadSelectionToken::new(n))
}

fn upload_session() -> FilePanelSession {
    FilePanelSession::begin(FileRequest::Upload(upload_request())).expect("valid upload")
}

fn save_session() -> FilePanelSession {
    FilePanelSession::begin(FileRequest::Save(save_request())).expect("valid save")
}

fn presented_upload_session() -> FilePanelSession {
    let mut session = upload_session();
    session.request_selection().expect("pending → requested");
    session
}

/// `begin` validates the request and starts in `Pending`; the session keeps
/// the request, its id, and its kind.
#[test]
fn begin_creates_pending_session_for_valid_request() {
    let session = upload_session();
    assert_eq!(session.state(), LifecycleState::Pending);
    assert!(!session.state().is_terminal());
    assert_eq!(session.id(), id(1));
    assert_eq!(session.kind(), FileRequestKind::Upload);
    assert_eq!(session.request(), &FileRequest::Upload(upload_request()));
}

/// An invalid request (nothing selectable) is rejected at `begin` and never
/// tracked.
#[test]
fn begin_rejects_invalid_requests() {
    let mut request = upload_request();
    request.capability = SelectionCapability {
        files: false,
        directories: false,
    };
    assert!(matches!(
        FilePanelSession::begin(FileRequest::Upload(request)),
        Err(FileRequestRejection::Upload(_))
    ));
}

/// Only terminal states are final; pending and requested are live.
#[test]
fn terminal_state_classification() {
    for state in [
        LifecycleState::Selected,
        LifecycleState::Cancelled,
        LifecycleState::Failed,
        LifecycleState::Superseded,
    ] {
        assert!(state.is_terminal(), "{state:?} must be terminal");
    }
    assert!(!LifecycleState::Pending.is_terminal());
    assert!(!LifecycleState::SelectionRequested.is_terminal());
}

/// Pending → selection requested is the delivery transition.
#[test]
fn pending_becomes_selection_requested() {
    let mut session = upload_session();
    assert_eq!(
        session.request_selection().expect("delivery"),
        LifecycleState::SelectionRequested
    );
    assert_eq!(session.state(), LifecycleState::SelectionRequested);
}

/// The delivery transition is legal exactly once.
#[test]
fn request_selection_twice_is_rejected() {
    let mut session = upload_session();
    session.request_selection().expect("first delivery");
    assert_eq!(
        session.request_selection(),
        Err(LifecycleError::WrongState {
            id: id(1),
            expected: LifecycleState::Pending,
            actual: LifecycleState::SelectionRequested,
        })
    );
}

/// A completion before the request was presented is premature and rejected
/// without mutating the session.
#[test]
fn completion_before_presentation_is_rejected() {
    let mut session = upload_session();
    assert_eq!(
        session.complete_upload(UploadCompletion::Cancelled),
        Err(LifecycleError::WrongState {
            id: id(1),
            expected: LifecycleState::SelectionRequested,
            actual: LifecycleState::Pending,
        })
    );
    assert_eq!(session.state(), LifecycleState::Pending);
}

/// A valid single-file selection resolves the lifecycle to `Selected`.
#[test]
fn single_file_upload_resolves_to_selected() {
    let mut session = presented_upload_session();
    assert_eq!(
        session
            .complete_upload(UploadCompletion::Selected(vec![file(1)]))
            .expect("valid selection"),
        LifecycleState::Selected
    );
    assert!(session.state().is_terminal());
}

/// Cancel resolves the session and locks it: a later completion rejects
/// whole and cannot mutate the resolved state.
#[test]
fn cancel_resolves_and_locks_the_session() {
    let mut session = presented_upload_session();
    assert_eq!(
        session
            .complete_upload(UploadCompletion::Cancelled)
            .expect("cancel"),
        LifecycleState::Cancelled
    );
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![file(1)])),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Cancelled,
        })
    );
    assert_eq!(
        session.supersede(),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Cancelled,
        })
    );
    assert_eq!(session.state(), LifecycleState::Cancelled);
}

/// Failure resolves the session and can never become `Selected`.
#[test]
fn failure_never_becomes_selected() {
    let mut session = presented_upload_session();
    assert_eq!(
        session
            .complete_upload(UploadCompletion::Failed)
            .expect("failure"),
        LifecycleState::Failed
    );
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![file(1)])),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Failed,
        })
    );
}

/// A duplicate completion after resolution rejects whole and leaves the
/// resolved state untouched.
#[test]
fn duplicate_selected_completion_is_rejected_whole() {
    let mut session = presented_upload_session();
    session
        .complete_upload(UploadCompletion::Selected(vec![file(1)]))
        .expect("first completion");
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![file(2)])),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Selected,
        })
    );
    assert_eq!(session.state(), LifecycleState::Selected);
}

/// Supersede resolves the session and it can never become `Selected`.
#[test]
fn superseded_session_never_becomes_selected() {
    let mut session = presented_upload_session();
    assert_eq!(
        session.supersede().expect("supersede"),
        LifecycleState::Superseded
    );
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![file(1)])),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Superseded,
        })
    );
    assert_eq!(
        session.complete_upload(UploadCompletion::Cancelled),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Superseded,
        })
    );
}

/// A pending (not yet delivered) request can be superseded too.
#[test]
fn pending_session_can_be_superseded() {
    let mut session = upload_session();
    assert_eq!(
        session.supersede().expect("supersede pending"),
        LifecycleState::Superseded
    );
}

/// Save requests complete with a destination token, or cancel/fail.
#[test]
fn save_lifecycle_matrix() {
    let mut selected = save_session();
    selected.request_selection().expect("delivery");
    assert_eq!(
        selected
            .complete_save(SaveCompletion::Selected(SaveDestinationToken::new(7)))
            .expect("destination"),
        LifecycleState::Selected
    );

    let mut cancelled = save_session();
    cancelled.request_selection().expect("delivery");
    assert_eq!(
        cancelled
            .complete_save(SaveCompletion::Cancelled)
            .expect("cancel"),
        LifecycleState::Cancelled
    );

    let mut failed = save_session();
    failed.request_selection().expect("delivery");
    assert_eq!(
        failed
            .complete_save(SaveCompletion::Failed)
            .expect("failure"),
        LifecycleState::Failed
    );
}

/// An upload completion applied to a save session is a kind mismatch and
/// leaves the session untouched; the correct completion still works.
#[test]
fn upload_completion_on_save_session_is_kind_mismatch() {
    let mut session = save_session();
    session.request_selection().expect("delivery");
    assert_eq!(
        session.complete_upload(UploadCompletion::Cancelled),
        Err(LifecycleError::KindMismatch {
            id: id(2),
            expected: FileRequestKind::Save,
            actual: FileRequestKind::Upload,
        })
    );
    assert_eq!(session.state(), LifecycleState::SelectionRequested);
    session
        .complete_save(SaveCompletion::Cancelled)
        .expect("correct kind still applies");
}

/// The mirror image: a save completion on an upload session.
#[test]
fn save_completion_on_upload_session_is_kind_mismatch() {
    let mut session = presented_upload_session();
    assert_eq!(
        session.complete_save(SaveCompletion::Selected(SaveDestinationToken::new(1))),
        Err(LifecycleError::KindMismatch {
            id: id(1),
            expected: FileRequestKind::Upload,
            actual: FileRequestKind::Save,
        })
    );
}

/// On a resolved session the stale check runs before the kind check: any
/// later completion rejects as `AlreadyResolved`.
#[test]
fn stale_check_precedes_kind_check() {
    let mut session = save_session();
    session.request_selection().expect("delivery");
    session
        .complete_save(SaveCompletion::Cancelled)
        .expect("cancel");
    assert_eq!(
        session.complete_upload(UploadCompletion::Cancelled),
        Err(LifecycleError::AlreadyResolved {
            id: id(2),
            state: LifecycleState::Cancelled,
        })
    );
}

/// `Selected` with zero items is rejected; a cancel is the legal empty
/// outcome.
#[test]
fn empty_selection_is_rejected() {
    let mut session = upload_session();
    session.request_selection().expect("delivery");
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![])),
        Err(LifecycleError::Selection(
            SelectionRejection::EmptySelection
        ))
    );
    assert_eq!(session.state(), LifecycleState::SelectionRequested);
}

/// A single-cardinality request rejects more than one item.
#[test]
fn single_cardinality_rejects_multiple_items() {
    let mut session = presented_upload_session();
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![file(1), file(2)])),
        Err(LifecycleError::Selection(SelectionRejection::Cardinality {
            expected: SelectionCardinality::Single,
            actual: 2,
        }))
    );
}

/// A multiple-cardinality request accepts several files.
#[test]
fn multiple_cardinality_accepts_several_items() {
    let mut request = upload_request();
    request.cardinality = SelectionCardinality::Multiple;
    let mut session = FilePanelSession::begin(FileRequest::Upload(request)).expect("valid");
    session.request_selection().expect("delivery");
    assert_eq!(
        session
            .complete_upload(UploadCompletion::Selected(vec![file(1), file(2), file(3)]))
            .expect("several files"),
        LifecycleState::Selected
    );
}

/// A files-only request rejects directory items.
#[test]
fn files_only_capability_rejects_directory_item() {
    let mut session = presented_upload_session();
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![directory(1)])),
        Err(LifecycleError::Selection(SelectionRejection::Capability {
            item: SelectedItemKind::Directory,
            allowed: SelectionCapability {
                files: true,
                directories: false,
            },
        }))
    );
}

/// A directories-only request rejects file items but accepts directories.
#[test]
fn directory_capability_rejects_files_and_accepts_directories() {
    let mut request = upload_request();
    request.capability = SelectionCapability {
        files: false,
        directories: true,
    };

    let mut rejected = FilePanelSession::begin(FileRequest::Upload(request.clone())).expect("ok");
    rejected.request_selection().expect("delivery");
    assert_eq!(
        rejected.complete_upload(UploadCompletion::Selected(vec![file(1)])),
        Err(LifecycleError::Selection(SelectionRejection::Capability {
            item: SelectedItemKind::File,
            allowed: SelectionCapability {
                files: false,
                directories: true,
            },
        }))
    );

    let mut accepted = FilePanelSession::begin(FileRequest::Upload(request)).expect("ok");
    accepted.request_selection().expect("delivery");
    assert_eq!(
        accepted
            .complete_upload(UploadCompletion::Selected(vec![directory(1)]))
            .expect("directory accepted"),
        LifecycleState::Selected
    );
}

/// Capability is validated per item before cardinality, so an illegal kind
/// is reported even when the count would also be illegal.
#[test]
fn capability_check_precedes_cardinality() {
    let mut session = presented_upload_session();
    assert_eq!(
        session.complete_upload(UploadCompletion::Selected(vec![directory(1), directory(2)])),
        Err(LifecycleError::Selection(SelectionRejection::Capability {
            item: SelectedItemKind::Directory,
            allowed: SelectionCapability {
                files: true,
                directories: false,
            },
        }))
    );
}
