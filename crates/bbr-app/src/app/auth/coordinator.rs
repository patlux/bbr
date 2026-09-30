//! The app-side authentication coordinator (queue item 10).
//!
//! One coordinator per browser window owns the challenge records, the
//! presented challenge, the sheet's edit state, and the in-memory
//! credential store. It holds **no authority** over the challenge
//! lifecycle: every decision is applied by
//! [`bbr_web::auth_challenges::apply_event`] on the record's own
//! `AuthState`, and the resulting state is the lifecycle. The coordinator
//! only chooses *which* challenge is presented, routes submit/cancel
//! through the contract, and keeps credentials out of every observable
//! shape (module invariant).
//!
//! This core is GPUI-free and unit-testable; the `BbrApp` glue (portal
//! presentation, focus, key routing) lives in `crate::app::auth`.

use std::collections::BTreeMap;

use bbr_web::auth_challenges::{
    apply_event, AuthChallenge, AuthEvent, AuthFailure, AuthResponse, AuthState, ChallengeId,
    ChallengeKind, ChallengeRejection, CredentialHandle, ProtectionSpace,
};
use bbr_web::http_auth::{HttpAuthAnswer, HttpCredential};
use bbr_web::{ProfileId, WebViewId};

use super::records::{AuthRecord, StoredCredential};
use super::sheet::AuthSheetState;

pub use super::records::{AuthSheetSurface, AuthSnapshotState};

/// The outcome of a submit/cancel decision, for the glue's presentation
/// choreography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeDecision {
    /// The challenge was resolved with a redacted credential handle.
    Resolved,
    /// The challenge reached the terminal cancelled state; the request
    /// failed and never retried.
    Cancelled,
    /// No sheet was presented (or the lifecycle rejected the decision); the
    /// decision was a deterministic no-op.
    Noop,
}

/// The deterministic per-window auth coordinator.
#[derive(Debug)]
pub struct AuthCoordinator {
    /// All recorded challenges in arrival order (their ids are monotonic).
    records: Vec<AuthRecord>,
    /// The challenge whose sheet is currently presented.
    presented: Option<ChallengeId>,
    /// The sheet's edit state (never serialized; Debug redacted).
    sheet: AuthSheetState,
    /// In-memory credential store: opaque handle id → raw credential.
    credentials: BTreeMap<u64, StoredCredential>,
    /// The focus domain (chrome vs web) captured when the sheet opened;
    /// restored on submit/cancel (plain presentation state).
    origin_chrome_focused: bool,
    /// The redacted decision representation of the last decided challenge
    /// (the contract's redaction, e.g. `credential:#1` / `cancelled`);
    /// `None` while a fresh sheet awaits a decision. Survives the sheet
    /// close so the wire snapshot can prove the sheet's decision; a new
    /// presentation resets it.
    last_outcome: Option<String>,
    next_challenge_id: u64,
    next_handle: u64,
}

impl AuthCoordinator {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            presented: None,
            sheet: AuthSheetState::new(),
            credentials: BTreeMap::new(),
            origin_chrome_focused: false,
            last_outcome: None,
            next_challenge_id: 0,
            next_handle: 0,
        }
    }

    /// Record a challenge bound to its originating web view. The challenge
    /// id is minted here (sequential, deterministic); the challenge is
    /// recorded `Pending` — "recorded but not yet presented to the
    /// authority" — and presented by [`Self::present_next`]. This is the
    /// production challenge-arrival seam's first half; a real WebKit
    /// delegate calls it, then presents immediately.
    pub fn record_challenge(
        &mut self,
        profile: ProfileId,
        web_view: WebViewId,
        protection_space: ProtectionSpace,
        kind: ChallengeKind,
        user_initiated: bool,
    ) -> Result<ChallengeId, ChallengeRejection> {
        self.next_challenge_id += 1;
        let id = ChallengeId(self.next_challenge_id);
        let challenge = AuthChallenge::new(
            id,
            profile,
            web_view,
            protection_space,
            kind,
            user_initiated,
        )?;
        self.records.push(AuthRecord {
            challenge,
            state: AuthState::Pending,
            native: None,
        });
        Ok(id)
    }

    /// Bind a recorded challenge to the engine request that waits for its
    /// answer (a challenge that came from WebKit).
    pub fn bind_native(&mut self, id: ChallengeId, request: u64) {
        if let Some(record) = self
            .records
            .iter_mut()
            .find(|record| record.challenge.id() == id)
        {
            record.native = Some(request);
        }
    }

    /// The answers the engine is owed for decided challenges, each handed
    /// out once. A credential response hands over the stored credential,
    /// which leaves memory here; cancel, supersession and failure answer
    /// without a credential, so the server's own answer loads and the
    /// request is never retried with credentials.
    pub fn take_native_answers(&mut self) -> Vec<(u64, HttpAuthAnswer)> {
        let mut answers = Vec::new();
        for record in &mut self.records {
            if !record.state.is_terminal() {
                continue;
            }
            let Some(request) = record.native.take() else {
                continue;
            };
            let credential = match &record.state {
                AuthState::Resolved {
                    response: AuthResponse::UseCredential(handle),
                } => self.credentials.remove(&handle.id()),
                _ => None,
            };
            answers.push((
                request,
                match credential {
                    Some(stored) => HttpAuthAnswer::Credential(HttpCredential::new(
                        stored.username,
                        stored.password,
                    )),
                    None => HttpAuthAnswer::WithoutCredential,
                },
            ));
        }
        answers
    }

    /// The challenge currently presented, if any.
    pub fn presented(&self) -> Option<ChallengeId> {
        self.presented
    }

    /// The mutable sheet edit state (key routing types into it; the values
    /// never leave the sheet except through the redacted handle).
    pub fn sheet(&mut self) -> &mut AuthSheetState {
        &mut self.sheet
    }

    /// Present the oldest awaiting challenge (FIFO). A currently presented
    /// non-terminal challenge is superseded by the candidate — the contract
    /// records `Superseded { by }` (terminal; can never become credential
    /// use), exactly the "a newer challenge replaced this one" rule. The
    /// sheet starts fresh for the new challenge.
    ///
    /// The username/password sheet only answers HTTP authentication: an
    /// awaiting client-certificate or unsupported challenge is failed
    /// deterministically through the contract (`Failed { EngineRejected }`)
    /// when the authority tries to present it — a typed decision for the
    /// later identity-selection capability, never an opaque escape.
    pub fn present_next(&mut self) -> Option<ChallengeId> {
        // Fail every awaiting challenge the sheet cannot answer, then find
        // the oldest remaining HTTP-authentication candidate.
        let mut candidate_index = None;
        for index in 0..self.records.len() {
            if self.records[index].state != AuthState::Pending {
                continue;
            }
            let challenge = self.records[index].challenge.clone();
            if !matches!(challenge.kind(), ChallengeKind::HttpAuth(_)) {
                let state = self.records[index].state.clone();
                if let Ok(next) = apply_event(
                    &challenge,
                    &state,
                    AuthEvent::Failed {
                        id: challenge.id(),
                        failure: AuthFailure::EngineRejected,
                    },
                ) {
                    self.records[index].state = next;
                }
                continue;
            }
            candidate_index = Some(index);
            break;
        }
        let candidate_index = candidate_index?;
        let candidate_id = self.records[candidate_index].challenge.id();

        // Supersede the currently presented non-terminal challenge.
        if let Some(current) = self.presented {
            if current != candidate_id {
                if let Some(index) = self
                    .records
                    .iter()
                    .position(|record| record.challenge.id() == current)
                {
                    if !self.records[index].state.is_terminal() {
                        let challenge = self.records[index].challenge.clone();
                        let state = self.records[index].state.clone();
                        let next = apply_event(
                            &challenge,
                            &state,
                            AuthEvent::Supersede {
                                id: current,
                                by: candidate_id,
                            },
                        );
                        if let Ok(next) = next {
                            self.records[index].state = next;
                        }
                    }
                }
            }
        }

        // Present the candidate: Pending → ResponseRequested (always legal).
        let challenge = self.records[candidate_index].challenge.clone();
        let state = self.records[candidate_index].state.clone();
        let next = apply_event(
            &challenge,
            &state,
            AuthEvent::RequestResponse { id: candidate_id },
        );
        let Ok(next) = next else {
            return None;
        };
        self.records[candidate_index].state = next;
        self.sheet = AuthSheetState::new();
        self.presented = Some(candidate_id);
        // A fresh presentation resets the decision evidence: the new sheet
        // owns the outcome from here on.
        self.last_outcome = None;
        Some(candidate_id)
    }

    /// Submit the presented sheet's credentials. The sheet holds no
    /// authority: the decision is applied through the contract
    /// ([`apply_event`] with `Respond`), which re-validates the response
    /// against the challenge kind and the current state. The raw values are
    /// captured into the in-memory store under the opaque handle; only the
    /// redacted handle leaves the sheet. A no-op when no sheet is presented
    /// or the lifecycle rejects the decision.
    pub fn submit(&mut self) -> ChallengeDecision {
        let Some(id) = self.presented else {
            return ChallengeDecision::Noop;
        };
        let Some(index) = self
            .records
            .iter()
            .position(|record| record.challenge.id() == id)
        else {
            return ChallengeDecision::Noop;
        };
        let challenge = self.records[index].challenge.clone();
        let state = self.records[index].state.clone();
        // The sheet only answers an HTTP-authentication challenge; the
        // lifecycle must be awaiting a response (the sheet is presented).
        if state != AuthState::ResponseRequested
            || !matches!(challenge.kind(), ChallengeKind::HttpAuth(_))
        {
            return ChallengeDecision::Noop;
        }
        self.next_handle += 1;
        let handle = CredentialHandle::new(self.next_handle);
        let (username, password) = self.sheet.capture();
        self.credentials
            .insert(self.next_handle, StoredCredential::new(username, password));
        let response = AuthResponse::UseCredential(handle);
        match apply_event(&challenge, &state, AuthEvent::Respond { id, response }) {
            Ok(next) => {
                self.last_outcome = outcome_label(&next);
                self.records[index].state = next;
                ChallengeDecision::Resolved
            }
            // Defensive: `UseCredential` on `HttpAuth` in `ResponseRequested`
            // is legal per the contract matrix; a rejection here can only
            // mean the state changed under us, so drop the store entry and
            // report the no-op truthfully.
            Err(_) => {
                self.credentials.remove(&self.next_handle);
                ChallengeDecision::Noop
            }
        }
    }

    /// Cancel the presented challenge. The contract moves the challenge to
    /// the terminal `Cancelled` state — the request **fails**; the app never
    /// retries it and no later response can revive it (terminal states are
    /// immutable). A no-op when no sheet is presented.
    pub fn cancel(&mut self) -> ChallengeDecision {
        let Some(id) = self.presented else {
            return ChallengeDecision::Noop;
        };
        let Some(index) = self
            .records
            .iter()
            .position(|record| record.challenge.id() == id)
        else {
            return ChallengeDecision::Noop;
        };
        let challenge = self.records[index].challenge.clone();
        let state = self.records[index].state.clone();
        if state.is_terminal() {
            return ChallengeDecision::Noop;
        }
        match apply_event(&challenge, &state, AuthEvent::Cancel { id }) {
            Ok(next) => {
                self.last_outcome = outcome_label(&next);
                self.records[index].state = next;
                ChallengeDecision::Cancelled
            }
            Err(_) => ChallengeDecision::Noop,
        }
    }

    /// Capture the focus domain at sheet open (chrome vs web); restored on
    /// submit/cancel like the find bar's origin restoration.
    pub fn capture_origin_focus(&mut self, chrome_focused: bool) {
        self.origin_chrome_focused = chrome_focused;
    }

    /// The focus domain captured at sheet open.
    pub fn origin_focus(&self) -> bool {
        self.origin_chrome_focused
    }

    /// Close the presented sheet (after a submit/cancel decision): the
    /// sheet's typed values are dropped, and the presentation is free for
    /// the next challenge.
    pub fn close_sheet(&mut self) {
        self.presented = None;
        self.sheet = AuthSheetState::new();
        self.origin_chrome_focused = false;
    }

    /// The redacted render data for the presented sheet; `None` when no
    /// sheet is presented or the presented challenge is not awaiting a
    /// response.
    pub fn surface(&self) -> Option<AuthSheetSurface> {
        let id = self.presented?;
        let record = self
            .records
            .iter()
            .find(|record| record.challenge.id() == id)?;
        if record.state != AuthState::ResponseRequested {
            return None;
        }
        let space = record.challenge.protection_space();
        Some(AuthSheetSurface {
            host: space.host().as_str().to_string(),
            realm: space.realm().map(|realm| realm.as_str().to_string()),
            scheme: scheme_name(record.challenge.kind()),
            authority: if space.is_proxy() { "proxy" } else { "origin" }.to_string(),
            focused: self.sheet.focused,
            username: self.sheet.username().to_string(),
            password_entered: self.sheet.password_entered(),
        })
    }

    /// The redacted wire-visible snapshot of the coordinator. By
    /// construction no field can hold a credential.
    pub fn snapshot(&self) -> AuthSnapshotState {
        let awaiting = self
            .records
            .iter()
            .filter(|record| record.state == AuthState::Pending)
            .map(|record| record.challenge.id().0)
            .collect::<Vec<_>>();
        let mut snap = AuthSnapshotState {
            awaiting,
            ..AuthSnapshotState::default()
        };
        let Some(id) = self.presented else {
            // No sheet on screen: the redacted decision evidence of the
            // last decided challenge (submit/cancel) stays visible so the
            // wire snapshot can prove the sheet's decision.
            snap.outcome = self.last_outcome.clone();
            return snap;
        };
        let Some(record) = self
            .records
            .iter()
            .find(|record| record.challenge.id() == id)
        else {
            return snap;
        };
        let space = record.challenge.protection_space();
        snap.open = true;
        snap.challenge = Some(record.challenge.id().0);
        snap.web_view = Some(record.challenge.web_view());
        snap.host = space.host().as_str().to_string();
        snap.realm = space.realm().map(|realm| realm.as_str().to_string());
        snap.scheme = scheme_name(record.challenge.kind());
        snap.authority = if space.is_proxy() { "proxy" } else { "origin" }.to_string();
        snap.state = state_label(&record.state);
        snap.outcome = outcome_label(&record.state);
        snap.focused_field = Some(self.sheet.focused.label().to_string());
        snap.username_entered = self.sheet.username_entered();
        snap.password_entered = self.sheet.password_entered();
        snap
    }

    /// Whether the coordinator holds a challenge that must never be answered
    /// from another tab: the presented challenge's originating web view.
    /// The unit tests prove the identity invariant through this accessor;
    /// the wire snapshot exposes the same fact (`web_view`) to the E2E.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn presented_web_view(&self) -> Option<WebViewId> {
        let id = self.presented?;
        self.records
            .iter()
            .find(|record| record.challenge.id() == id)
            .map(|record| record.challenge.web_view())
    }

    /// Challenge-record diagnostics for unit tests: the lifecycle state of
    /// the challenge with `id`.
    #[cfg(test)]
    fn state_of(&self, id: ChallengeId) -> Option<&AuthState> {
        self.records
            .iter()
            .find(|record| record.challenge.id() == id)
            .map(|record| &record.state)
    }

    /// The raw credential stored under an opaque handle key (unit tests
    /// only): it lives only here, never in any observable shape, until
    /// `take_native_answers` hands it to the engine.
    #[cfg(test)]
    fn stored(&self, handle: u64) -> Option<(&str, &str)> {
        self.credentials
            .get(&handle)
            .map(|credential| (credential.username.as_str(), credential.password.as_str()))
    }
}

impl Default for AuthCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

fn scheme_name(kind: &ChallengeKind) -> String {
    match kind {
        ChallengeKind::HttpAuth(scheme) => scheme.as_str().to_string(),
        _ => String::new(),
    }
}

fn state_label(state: &AuthState) -> String {
    match state {
        AuthState::Pending => "pending".to_string(),
        AuthState::ResponseRequested => "response_requested".to_string(),
        AuthState::Resolved { .. } => "resolved".to_string(),
        AuthState::Failed { .. } => "failed".to_string(),
        AuthState::Cancelled => "cancelled".to_string(),
        AuthState::Superseded { .. } => "superseded".to_string(),
    }
}

fn outcome_label(state: &AuthState) -> Option<String> {
    match state {
        AuthState::Pending | AuthState::ResponseRequested => None,
        // The contract's redaction: opaque handle ids only, never
        // credential material.
        AuthState::Resolved { response } => Some(match response {
            AuthResponse::UseCredential(handle) => format!("credential:#{}", handle.id()),
            AuthResponse::UseClientIdentity(handle) => format!("client_identity:#{}", handle.id()),
            AuthResponse::Continue => "continue".to_string(),
            AuthResponse::UseDefault => "default".to_string(),
            AuthResponse::Cancel => "cancelled".to_string(),
        }),
        AuthState::Failed { failure } => Some(match failure {
            AuthFailure::EngineRejected => "failed:engine_rejected".to_string(),
            AuthFailure::Connection => "failed:connection".to_string(),
            AuthFailure::Unknown => "failed:unknown".to_string(),
        }),
        AuthState::Cancelled => Some("cancelled".to_string()),
        AuthState::Superseded { by } => Some(format!("superseded:#{}", by.0)),
    }
}

/// A Basic-origin challenge builder for tests and fixtures: the canonical
/// protection space of the sheet's scope.
#[cfg(test)]
fn basic_space(host: &str, realm: Option<&str>) -> ProtectionSpace {
    ProtectionSpace::new(
        bbr_web::auth_challenges::HttpScheme::Https,
        host,
        0,
        realm,
        bbr_web::auth_challenges::AuthAuthority::Origin,
    )
    .expect("valid space")
}

#[cfg(test)]
#[path = "coordinator_tests.rs"]
mod coordinator_tests;
