//! # File-panel completion lifecycle and engine/orchestrator port
//!
//! Deterministic state machine for one file-panel request plus the
//! module-local object-safe port through which the engine delivers requests
//! to an orchestrator and receives typed completions back. The shared root
//! [`crate::WebEngine`] trait is untouched; a later serial adapter composes
//! this seam. Pure `std`: no frameworks, GPUI, `unsafe`, file I/O, network,
//! or new dependencies.
//!
//! ## States
//!
//! `Pending` → `SelectionRequested` → `Selected` | `Cancelled` | `Failed`;
//! `Pending` or `SelectionRequested` → `Superseded`. The four terminal
//! states are final: no later command may mutate the session. Cancel,
//! failure, and supersession can never become `Selected`.
//!
//! ## Completion rules
//!
//! A completion carries the request id and a kind-typed result.
//! [`FilePanelCoordinator`] routes by id: an id that was never delivered is
//! [`LifecycleError::UnknownRequest`], a kind mismatch rejects whole
//! ([`LifecycleError::KindMismatch`]), duplicate or stale completions
//! against a resolved session reject whole ([`LifecycleError::AlreadyResolved`]),
//! and an upload selection must satisfy the request's cardinality and
//! capability ([`LifecycleError::Selection`]). Tokens stay opaque; returning
//! one through a completion transfers no sandbox or lifetime right — the
//! **adapter owns sandbox lifetime** for as long as a token is in flight.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use crate::file_panels::{
    FileRequest, FileRequestId, FileRequestKind, FileRequestRejection, SaveDestinationToken,
    SelectionCapability, SelectionCardinality, UploadRequest, UploadSelectionToken,
};

/// One step of a request's deterministic lifecycle.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Validated and tracked, but not yet handed to the orchestrator.
    Pending,
    /// Delivered; the orchestrator is (or will be) presenting a panel.
    SelectionRequested,
    /// A valid selection or destination resolved the request. Terminal.
    Selected,
    /// The user dismissed the panel. Terminal.
    Cancelled,
    /// The panel failed before producing a selection. Terminal.
    Failed,
    /// A newer request replaced this one. Terminal.
    Superseded,
}

impl LifecycleState {
    /// Terminal states can never be left, re-entered, or completed again.
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            LifecycleState::Selected
                | LifecycleState::Cancelled
                | LifecycleState::Failed
                | LifecycleState::Superseded
        )
    }
}

/// What a selected upload item is: a file or a directory. The adapter
/// classifies each item; the token itself stays opaque.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedItemKind {
    File,
    Directory,
}

/// One selected upload item: an opaque token plus its file/directory kind,
/// so the machine can enforce the request's capability without ever seeing
/// a path.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadSelectionItem {
    pub kind: SelectedItemKind,
    pub token: UploadSelectionToken,
}

impl UploadSelectionItem {
    pub const fn new(kind: SelectedItemKind, token: UploadSelectionToken) -> Self {
        Self { kind, token }
    }
}

/// Kind-typed result of an upload panel.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadCompletion {
    /// Items were selected. Validated against the request's cardinality and
    /// capability before the lifecycle may resolve; an empty selection is
    /// rejected — a cancel is the legal way to end without items.
    Selected(Vec<UploadSelectionItem>),
    /// The user dismissed the panel.
    Cancelled,
    /// The panel failed before producing a selection.
    Failed,
}

/// Kind-typed result of a save panel. Save panels are single-selection, so
/// a selected destination is one token by construction.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveCompletion {
    Selected(SaveDestinationToken),
    Cancelled,
    Failed,
}

/// A completion returned to the engine, carrying the request id it resolves.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilePanelCompletion {
    Upload {
        id: FileRequestId,
        result: UploadCompletion,
    },
    Save {
        id: FileRequestId,
        result: SaveCompletion,
    },
}

impl FilePanelCompletion {
    /// The request this completion resolves; the coordinator routes by it.
    pub fn id(&self) -> FileRequestId {
        match self {
            Self::Upload { id, .. } => *id,
            Self::Save { id, .. } => *id,
        }
    }

    /// The panel kind this completion came from.
    pub fn kind(&self) -> FileRequestKind {
        match self {
            Self::Upload { .. } => FileRequestKind::Upload,
            Self::Save { .. } => FileRequestKind::Save,
        }
    }
}

/// Why a selected upload completion was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionRejection {
    /// `Selected` with zero items; a cancel is the legal empty outcome.
    EmptySelection,
    /// Item count violates the request's single/multiple cardinality.
    Cardinality {
        expected: SelectionCardinality,
        actual: usize,
    },
    /// An item kind the request does not allow (files versus directories).
    Capability {
        item: SelectedItemKind,
        allowed: SelectionCapability,
    },
}

/// Why a command or completion could not be applied.
///
/// Guard order is fixed and documented: terminal sessions reject every
/// later command ([`AlreadyResolved`](Self::AlreadyResolved)), then the
/// kind is checked ([`KindMismatch`](Self::KindMismatch)), then the state
/// must be `SelectionRequested` ([`WrongState`](Self::WrongState)), then
/// the selection is validated ([`Selection`](Self::Selection)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// The request failed pure validation and was never tracked.
    InvalidRequest(FileRequestRejection),
    /// No session is tracked for this id: never delivered or dropped.
    UnknownRequest { id: FileRequestId },
    /// A request with this id is already tracked; ids are never reused.
    DuplicateRequest { id: FileRequestId },
    /// A transition is not legal from the session's current non-terminal
    /// state (for example a completion before the request was presented).
    WrongState {
        id: FileRequestId,
        expected: LifecycleState,
        actual: LifecycleState,
    },
    /// The session already resolved; no later command may mutate it
    /// (duplicate/stale completion, or supersede after resolution).
    AlreadyResolved {
        id: FileRequestId,
        state: LifecycleState,
    },
    /// Completion kind does not match the request kind.
    KindMismatch {
        id: FileRequestId,
        expected: FileRequestKind,
        actual: FileRequestKind,
    },
    /// The selection violates the request's cardinality or capability.
    Selection(SelectionRejection),
}

/// The deterministic lifecycle machine for one file-panel request.
///
/// Created [`Pending`](LifecycleState::Pending) from a validated request;
/// the orchestrator marks it [`SelectionRequested`](LifecycleState::SelectionRequested),
/// and one completion or supersession resolves it to a terminal state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePanelSession {
    request: FileRequest,
    state: LifecycleState,
}

impl FilePanelSession {
    /// Validate a request and start its lifecycle in
    /// [`LifecycleState::Pending`].
    pub fn begin(request: FileRequest) -> Result<Self, FileRequestRejection> {
        request.validate()?;
        Ok(Self {
            request,
            state: LifecycleState::Pending,
        })
    }

    pub fn id(&self) -> FileRequestId {
        self.request.id()
    }

    pub fn kind(&self) -> FileRequestKind {
        self.request.kind()
    }

    /// The current lifecycle state.
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// The validated request this session tracks.
    pub fn request(&self) -> &FileRequest {
        &self.request
    }

    /// Hand the request to the orchestrator:
    /// [`Pending`](LifecycleState::Pending) →
    /// [`SelectionRequested`](LifecycleState::SelectionRequested).
    pub fn request_selection(&mut self) -> Result<LifecycleState, LifecycleError> {
        match self.state {
            LifecycleState::Pending => {
                self.state = LifecycleState::SelectionRequested;
                Ok(self.state)
            }
            LifecycleState::SelectionRequested => Err(LifecycleError::WrongState {
                id: self.id(),
                expected: LifecycleState::Pending,
                actual: LifecycleState::SelectionRequested,
            }),
            state => Err(LifecycleError::AlreadyResolved {
                id: self.id(),
                state,
            }),
        }
    }

    /// Apply an upload panel's result. The selection is validated against
    /// the request's cardinality and capability before the lifecycle may
    /// resolve.
    pub fn complete_upload(
        &mut self,
        result: UploadCompletion,
    ) -> Result<LifecycleState, LifecycleError> {
        self.reject_if_resolved()?;
        self.require_kind(FileRequestKind::Upload)?;
        self.require_selection_requested()?;
        match result {
            UploadCompletion::Selected(items) => {
                let FileRequest::Upload(upload) = &self.request else {
                    unreachable!("kind is checked above")
                };
                validate_selection(upload, &items).map_err(LifecycleError::Selection)?;
                self.state = LifecycleState::Selected;
            }
            UploadCompletion::Cancelled => self.state = LifecycleState::Cancelled,
            UploadCompletion::Failed => self.state = LifecycleState::Failed,
        }
        Ok(self.state)
    }

    /// Apply a save panel's result; a save is single-selection by
    /// construction, so no cardinality or capability check applies.
    pub fn complete_save(
        &mut self,
        result: SaveCompletion,
    ) -> Result<LifecycleState, LifecycleError> {
        self.reject_if_resolved()?;
        self.require_kind(FileRequestKind::Save)?;
        self.require_selection_requested()?;
        match result {
            SaveCompletion::Selected(_) => self.state = LifecycleState::Selected,
            SaveCompletion::Cancelled => self.state = LifecycleState::Cancelled,
            SaveCompletion::Failed => self.state = LifecycleState::Failed,
        }
        Ok(self.state)
    }

    /// Replace the outstanding request with a newer one. Legal while
    /// pending or presented; a resolved session can never be superseded,
    /// and a superseded session can never complete.
    pub fn supersede(&mut self) -> Result<LifecycleState, LifecycleError> {
        match self.state {
            LifecycleState::Pending | LifecycleState::SelectionRequested => {
                self.state = LifecycleState::Superseded;
                Ok(self.state)
            }
            state => Err(LifecycleError::AlreadyResolved {
                id: self.id(),
                state,
            }),
        }
    }

    fn reject_if_resolved(&self) -> Result<(), LifecycleError> {
        if self.state.is_terminal() {
            return Err(LifecycleError::AlreadyResolved {
                id: self.id(),
                state: self.state,
            });
        }
        Ok(())
    }

    fn require_kind(&self, actual: FileRequestKind) -> Result<(), LifecycleError> {
        let expected = self.kind();
        if expected != actual {
            return Err(LifecycleError::KindMismatch {
                id: self.id(),
                expected,
                actual,
            });
        }
        Ok(())
    }

    fn require_selection_requested(&self) -> Result<(), LifecycleError> {
        match self.state {
            LifecycleState::SelectionRequested => Ok(()),
            state => Err(LifecycleError::WrongState {
                id: self.id(),
                expected: LifecycleState::SelectionRequested,
                actual: state,
            }),
        }
    }
}

/// Enforce the request's single/multiple cardinality and file/directory
/// capability on a selected upload. Empty selections are rejected: a cancel
/// is the legal way to end without items.
fn validate_selection(
    request: &UploadRequest,
    items: &[UploadSelectionItem],
) -> Result<(), SelectionRejection> {
    if items.is_empty() {
        return Err(SelectionRejection::EmptySelection);
    }
    for item in items {
        let allowed = match item.kind {
            SelectedItemKind::File => request.capability.files,
            SelectedItemKind::Directory => request.capability.directories,
        };
        if !allowed {
            return Err(SelectionRejection::Capability {
                item: item.kind,
                allowed: request.capability,
            });
        }
    }
    if request.cardinality == SelectionCardinality::Single && items.len() > 1 {
        return Err(SelectionRejection::Cardinality {
            expected: SelectionCardinality::Single,
            actual: items.len(),
        });
    }
    Ok(())
}

/// Engine → orchestrator command/result seam for file panels.
///
/// Object-safe (no generics), so the engine may hold `Box<dyn FilePanelPort>`;
/// the shared root [`crate::WebEngine`] trait is deliberately untouched — a
/// later serial adapter composes this seam. Methods take `&mut self`; an
/// adapter behind a multi-threaded delegate boundary serializes access
/// itself.
pub trait FilePanelPort {
    /// Deliver a validated request. The orchestrator presents the panel and
    /// later returns a completion carrying the same request id.
    fn request_panel(&mut self, request: FileRequest) -> Result<(), LifecycleError>;

    /// Return the user's result for a previously delivered request, routed
    /// by request id.
    fn complete_panel(&mut self, completion: FilePanelCompletion) -> Result<(), LifecycleError>;

    /// Replace an outstanding request: its session resolves
    /// [`Superseded`](LifecycleState::Superseded) and can never complete.
    fn supersede_panel(&mut self, id: FileRequestId) -> Result<(), LifecycleError>;
}

/// Default engine-side [`FilePanelPort`] implementation: one tracked
/// session per request id.
///
/// Resolved sessions are retained, so a request id can never be delivered
/// twice or re-armed after resolution ([`LifecycleError::DuplicateRequest`]).
/// A completion whose id was never delivered is
/// [`LifecycleError::UnknownRequest`]. Returning tokens through a completion
/// does not transfer sandbox or lifetime rights — the **adapter owns
/// sandbox lifetime** for as long as a token is in flight.
#[derive(Debug, Clone, Default)]
pub struct FilePanelCoordinator {
    sessions: BTreeMap<FileRequestId, FilePanelSession>,
}

impl FilePanelCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// The tracked session for a request id, if any (resolved sessions are
    /// retained as tombstones).
    pub fn session(&self, id: FileRequestId) -> Option<&FilePanelSession> {
        self.sessions.get(&id)
    }
}

impl FilePanelPort for FilePanelCoordinator {
    fn request_panel(&mut self, request: FileRequest) -> Result<(), LifecycleError> {
        let id = request.id();
        if self.sessions.contains_key(&id) {
            return Err(LifecycleError::DuplicateRequest { id });
        }
        let mut session =
            FilePanelSession::begin(request).map_err(LifecycleError::InvalidRequest)?;
        session.request_selection()?;
        self.sessions.insert(id, session);
        Ok(())
    }

    fn complete_panel(&mut self, completion: FilePanelCompletion) -> Result<(), LifecycleError> {
        let id = completion.id();
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(LifecycleError::UnknownRequest { id })?;
        match completion {
            FilePanelCompletion::Upload { result, .. } => {
                session.complete_upload(result)?;
            }
            FilePanelCompletion::Save { result, .. } => {
                session.complete_save(result)?;
            }
        }
        Ok(())
    }

    fn supersede_panel(&mut self, id: FileRequestId) -> Result<(), LifecycleError> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(LifecycleError::UnknownRequest { id })?;
        session.supersede()?;
        Ok(())
    }
}
