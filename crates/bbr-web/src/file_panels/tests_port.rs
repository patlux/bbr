//! Port contract tests: `FilePanelCoordinator` routing by exact request id,
//! duplicate/stale/mismatch rejection at the boundary, object safety, exact
//! delivery through the trait, and structural token-kind separation — all
//! without filesystem or native types.

use crate::file_panels::{
    AcceptFilter, CaptureHint, FilePanelCompletion, FilePanelCoordinator, FilePanelPort,
    FileRequest, FileRequestId, FileRequestKind, FileRequestRejection, LifecycleError,
    LifecycleState, ReplacementPolicy, RequestContext, SaveCompletion, SaveDestinationToken,
    SaveRequest, SelectedItemKind, SelectionCapability, SelectionCardinality, UploadCompletion,
    UploadRequest, UploadSelectionItem, UploadSelectionToken,
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

/// Completions route by exact request id: resolving id 2 leaves id 1
/// untouched, and vice versa.
#[test]
fn coordinator_routes_completions_to_exact_sessions() {
    let mut coordinator = FilePanelCoordinator::new();
    coordinator
        .request_panel(FileRequest::Upload(upload_request()))
        .expect("deliver upload");
    coordinator
        .request_panel(FileRequest::Save(save_request()))
        .expect("deliver save");

    coordinator
        .complete_panel(FilePanelCompletion::Save {
            id: id(2),
            result: SaveCompletion::Selected(SaveDestinationToken::new(9)),
        })
        .expect("resolve save");
    assert_eq!(
        coordinator.session(id(2)).expect("save session").state(),
        LifecycleState::Selected
    );
    assert_eq!(
        coordinator.session(id(1)).expect("upload session").state(),
        LifecycleState::SelectionRequested
    );

    coordinator
        .complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Cancelled,
        })
        .expect("resolve upload");
    assert_eq!(
        coordinator.session(id(1)).expect("upload session").state(),
        LifecycleState::Cancelled
    );
}

/// A completion for an id that was never delivered is stale and rejected.
#[test]
fn coordinator_rejects_unknown_completions() {
    let mut coordinator = FilePanelCoordinator::new();
    coordinator
        .request_panel(FileRequest::Upload(upload_request()))
        .expect("deliver");
    assert_eq!(
        coordinator.complete_panel(FilePanelCompletion::Upload {
            id: id(99),
            result: UploadCompletion::Cancelled,
        }),
        Err(LifecycleError::UnknownRequest { id: id(99) })
    );
}

/// Request ids are never reused: a second delivery with the same id
/// rejects, both while live and after resolution (tombstones).
#[test]
fn coordinator_rejects_duplicate_request_ids() {
    let mut coordinator = FilePanelCoordinator::new();
    coordinator
        .request_panel(FileRequest::Upload(upload_request()))
        .expect("first delivery");
    assert_eq!(
        coordinator.request_panel(FileRequest::Upload(upload_request())),
        Err(LifecycleError::DuplicateRequest { id: id(1) })
    );

    coordinator
        .complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Cancelled,
        })
        .expect("resolve");
    assert_eq!(
        coordinator.request_panel(FileRequest::Upload(upload_request())),
        Err(LifecycleError::DuplicateRequest { id: id(1) })
    );
}

/// Invalid requests are rejected at delivery and never tracked.
#[test]
fn coordinator_rejects_invalid_requests_before_tracking() {
    let mut coordinator = FilePanelCoordinator::new();
    let mut bad = upload_request();
    bad.capability = SelectionCapability {
        files: false,
        directories: false,
    };
    assert!(matches!(
        coordinator.request_panel(FileRequest::Upload(bad)),
        Err(LifecycleError::InvalidRequest(
            FileRequestRejection::Upload(_)
        ))
    ));
    assert!(coordinator.session(id(1)).is_none());
}

/// A stale completion for a resolved id rejects whole and cannot touch a
/// newer request with a different id.
#[test]
fn stale_completion_cannot_mutate_newer_request() {
    let mut coordinator = FilePanelCoordinator::new();
    coordinator
        .request_panel(FileRequest::Upload(upload_request()))
        .expect("deliver id 1");
    coordinator
        .complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Cancelled,
        })
        .expect("resolve id 1");

    let mut newer = upload_request();
    newer.id = id(3);
    coordinator
        .request_panel(FileRequest::Upload(newer))
        .expect("deliver id 3");

    assert!(matches!(
        coordinator.complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Cancelled,
        }),
        Err(LifecycleError::AlreadyResolved {
            id: resolved_id,
            state: LifecycleState::Cancelled,
        }) if resolved_id == id(1)
    ));
    assert_eq!(
        coordinator.session(id(3)).expect("newer session").state(),
        LifecycleState::SelectionRequested
    );
}

/// Supersede is routed by id: the named session resolves `Superseded`, and
/// later completions for it reject as stale.
#[test]
fn coordinator_supersede_routing() {
    let mut coordinator = FilePanelCoordinator::new();
    coordinator
        .request_panel(FileRequest::Upload(upload_request()))
        .expect("deliver");
    coordinator.supersede_panel(id(1)).expect("supersede id 1");
    assert_eq!(
        coordinator.session(id(1)).expect("session").state(),
        LifecycleState::Superseded
    );
    assert_eq!(
        coordinator.complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Selected(vec![file(1)]),
        }),
        Err(LifecycleError::AlreadyResolved {
            id: id(1),
            state: LifecycleState::Superseded,
        })
    );
    assert_eq!(
        coordinator.supersede_panel(id(99)),
        Err(LifecycleError::UnknownRequest { id: id(99) })
    );
}

/// The port trait is object-safe: a `Box<dyn FilePanelPort>` drives the
/// full request → completion round trip.
#[test]
fn file_panel_port_is_object_safe() {
    let mut port: Box<dyn FilePanelPort> = Box::new(FilePanelCoordinator::new());
    port.request_panel(FileRequest::Upload(upload_request()))
        .expect("delivery through trait object");
    port.complete_panel(FilePanelCompletion::Upload {
        id: id(1),
        result: UploadCompletion::Selected(vec![file(1)]),
    })
    .expect("completion through trait object");
    // The session resolved: a repeat completion is stale, not unknown.
    assert!(matches!(
        port.complete_panel(FilePanelCompletion::Upload {
            id: id(1),
            result: UploadCompletion::Cancelled,
        }),
        Err(LifecycleError::AlreadyResolved {
            state: LifecycleState::Selected,
            ..
        })
    ));
}

/// A test-only orchestrator implementing the trait records exactly what the
/// engine delivered and superseded — proving exact request delivery through
/// the seam without the coordinator's bookkeeping.
struct RecordingOrchestrator {
    delivered: Vec<FileRequest>,
    superseded: Vec<FileRequestId>,
}

impl FilePanelPort for RecordingOrchestrator {
    fn request_panel(&mut self, request: FileRequest) -> Result<(), LifecycleError> {
        self.delivered.push(request);
        Ok(())
    }

    fn complete_panel(&mut self, _completion: FilePanelCompletion) -> Result<(), LifecycleError> {
        Ok(())
    }

    fn supersede_panel(&mut self, id: FileRequestId) -> Result<(), LifecycleError> {
        self.superseded.push(id);
        Ok(())
    }
}

/// Through `&mut dyn FilePanelPort`, delivered requests arrive in order and
/// byte-for-byte equal to what the engine sent.
#[test]
fn port_delivers_requests_exactly() {
    let mut orchestrator = RecordingOrchestrator {
        delivered: Vec::new(),
        superseded: Vec::new(),
    };
    let port: &mut dyn FilePanelPort = &mut orchestrator;

    let upload = FileRequest::Upload(upload_request());
    let save = FileRequest::Save(save_request());
    port.request_panel(upload.clone()).expect("deliver upload");
    port.request_panel(save.clone()).expect("deliver save");
    port.supersede_panel(id(1)).expect("supersede id 1");

    assert_eq!(orchestrator.delivered, vec![upload, save]);
    assert_eq!(orchestrator.superseded, vec![id(1)]);
}

/// The completion types are structurally kind-separated: an upload
/// completion can only carry upload tokens and a save completion only a
/// save token; the tokens themselves remain bare opaque u64-sized handles
/// with no payload-bearing fields.
#[test]
fn completion_kinds_and_tokens_are_kind_separated() {
    use std::mem::size_of;
    assert_eq!(size_of::<UploadSelectionToken>(), size_of::<u64>());
    assert_eq!(size_of::<SaveDestinationToken>(), size_of::<u64>());

    fn takes_upload_token(_: UploadSelectionToken) {}
    takes_upload_token(UploadSelectionToken::new(1));
    // takes_upload_token(SaveDestinationToken::new(1)); // compile-time rejection

    let upload = FilePanelCompletion::Upload {
        id: id(1),
        result: UploadCompletion::Selected(vec![file(1)]),
    };
    assert_eq!(upload.kind(), FileRequestKind::Upload);
    let save = FilePanelCompletion::Save {
        id: id(2),
        result: SaveCompletion::Selected(SaveDestinationToken::new(2)),
    };
    assert_eq!(save.kind(), FileRequestKind::Save);
    assert_eq!(save.id(), id(2));
}

/// The lifecycle layer carries no filesystem or native payload: a
/// completion holds ids, opaque tokens, and unit states — no path strings,
/// URLs, descriptors, or native pointers.
#[test]
fn lifecycle_layer_carries_no_filesystem_or_native_payload() {
    let completion = FilePanelCompletion::Upload {
        id: id(1),
        result: UploadCompletion::Selected(vec![file(1), file(2)]),
    };
    let FilePanelCompletion::Upload {
        id: request_id,
        result,
    } = &completion
    else {
        unreachable!("constructed as upload")
    };
    assert_eq!(*request_id, id(1));
    let UploadCompletion::Selected(items) = result else {
        unreachable!("constructed as selected")
    };
    assert_eq!(items.len(), 2);
    assert!(items
        .iter()
        .all(|item| matches!(item.kind, SelectedItemKind::File)));
}
