//! Authentication challenge sheet (queue item 10, P8c Auth-Adapter).
//!
//! The conventional macOS credential sheet presented inside the one
//! `NativeAboveWeb` portal window: it shows the realm and host of the
//! protection space that asked, collects a username and a password, and
//! routes submit and cancel **through the merged portable contract**
//! (`bbr_web::auth_challenges`). The sheet itself holds no authority over
//! the challenge lifecycle: every decision is applied by
//! [`bbr_web::auth_challenges::apply_event`] on the challenge's own
//! `AuthState`, and the resulting state is the lifecycle.
//!
//! This module owns the coordinator core ([`AuthCoordinator`], GPUI-free
//! and unit-testable), the sheet's edit state ([`AuthSheetState`]), and the
//! `BbrApp` presentation glue (portal presentation, focus, key routing).
//!
//! ## Redaction invariant
//!
//! Credentials never enter logs, snapshots, testkit payloads, benchmark
//! records, or error text. The typed values live **only** in the sheet's
//! edit state ([`AuthSheetState`]) and, after submit, in the coordinator's
//! in-memory credential store ([`StoredCredential`]) under the opaque
//! handle key the contract carries. Both types have `Debug` redacted by
//! construction and are deliberately never `Serialize`/`Deserialize`, so
//! no wire shape can ever contain the typed values. The only
//! representation that leaves the sheet is the contract's redaction: a
//! [`bbr_web::auth_challenges::CredentialHandle`] inside a
//! `UseCredential` response.
//!
//! ## Identity invariant
//!
//! A challenge belongs to its originating web view: the coordinator keys
//! every record by its [`bbr_web::auth_challenges::ChallengeId`] and the
//! record's `AuthChallenge` carries the owning `WebViewId`. Submit and
//! cancel act **only** on the presented challenge id, so a response can
//! never be attributed to a different tab, and a challenge that was never
//! presented cannot be answered (its `AuthState` is still `Pending`, which
//! the contract rejects for any response). Cancellation fails the request
//! rather than silently retrying: `Cancel` moves the challenge to the
//! terminal `Cancelled` state, and every later event — including a
//! credential response — is rejected whole.

mod coordinator;
mod records;
mod sheet;

pub use coordinator::{AuthCoordinator, AuthSheetSurface, AuthSnapshotState, ChallengeDecision};

use bbr_web::auth_challenges::{AuthChallenge, AuthScheme, ChallengeKind, ProtectionSpace};
use bbr_web::http_auth::HttpAuthAnswer;
use bbr_web::WebViewId;
use gpui::{Context, KeyDownEvent, Window};

use crate::portal::geometry::{Edges, Sz};
use crate::portal::state::{DismissalPolicy, PortalPlacement, PortalRequest};

use super::BbrApp;

/// The credential sheet's stable presentation id, allocated in the portal
/// presentation-id space next to the probe (`0xB9`), the command surfaces
/// (`0xC1`/`0xC2`), and the find bar (`0xC3`).
pub const AUTH_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0xC4);

/// The sheet's portal draw priority (same slot as the command surfaces).
pub const AUTH_PORTAL_PRIORITY: usize = 2;

/// The sheet's width in points: the §11.4 command-surface family's compact
/// clamp minimum — settled chrome geometry, not an invented size.
pub const AUTH_SHEET_WIDTH: f32 = 360.0;

/// The sheet's height in points (the conventional macOS credential sheet
/// aspect: header + two fields + two buttons).
pub const AUTH_SHEET_HEIGHT: f32 = 236.0;

impl BbrApp {
    /// The testkit challenge-arrival seam: a challenge for a web view is
    /// recorded and its credential sheet is presented immediately (WebKit's
    /// challenges take [`Self::on_http_auth_request`], which records the
    /// same way and binds the engine request). The sheet shows the realm and
    /// host of the protection space that asked.
    pub fn auth_on_challenge(
        &mut self,
        challenge: AuthChallenge,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let _ = self
            .auth
            .get()
            .record_challenge(
                challenge.profile(),
                challenge.web_view(),
                challenge.protection_space().clone(),
                challenge.kind().clone(),
                challenge.user_initiated(),
            )
            .map_err(|error| log::warn!("auth challenge rejected: {error}"));
        self.auth_present_sheet(window, cx)
    }

    /// A challenge from WebKit (`WebEvent::HttpAuthRequested`): recorded
    /// with the engine request that waits for it and presented at once. A
    /// view that is already gone, or a challenge the contract rejects, is
    /// answered without a credential (WebKit waits for every answer).
    pub(crate) fn on_http_auth_request(
        &mut self,
        view: WebViewId,
        request: u64,
        space: &ProtectionSpace,
        scheme: AuthScheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(profile) = self.controller.read().profile_of_view(view) else {
            self.engine
                .borrow_mut()
                .answer_http_auth(request, HttpAuthAnswer::WithoutCredential);
            return;
        };
        let recorded = self.auth.get().record_challenge(
            profile,
            view,
            space.clone(),
            ChallengeKind::HttpAuth(scheme),
            false,
        );
        match recorded {
            Ok(id) => self.auth.bookkeep().bind_native(id, request),
            Err(error) => {
                log::warn!("auth challenge rejected: {error}");
                self.engine
                    .borrow_mut()
                    .answer_http_auth(request, HttpAuthAnswer::WithoutCredential);
                return;
            }
        }
        self.auth_present_sheet(window, cx);
    }

    /// Hand the engine the answers its decided challenges are owed: the
    /// credential of a submitted sheet, or none for a cancelled,
    /// superseded or failed one.
    fn answer_auth_engine(&mut self) {
        let answers = self.auth.bookkeep().take_native_answers();
        for (request, answer) in answers {
            self.engine.borrow_mut().answer_http_auth(request, answer);
        }
    }

    /// Testkit-only deterministic seeding: record a challenge WITHOUT
    /// presenting the sheet (the awaiting challenge the measured
    /// presentation — and the functional fixtures that present explicitly —
    /// use). Carries only authority metadata, never credentials.
    pub fn auth_record_challenge(
        &mut self,
        profile: bbr_web::ProfileId,
        web_view: bbr_web::WebViewId,
        protection_space: bbr_web::auth_challenges::ProtectionSpace,
        scheme: AuthScheme,
        user_initiated: bool,
    ) -> bool {
        self.auth
            .get()
            .record_challenge(
                profile,
                web_view,
                protection_space,
                ChallengeKind::HttpAuth(scheme),
                user_initiated,
            )
            .map(|_| true)
            .unwrap_or_else(|error| {
                log::warn!("auth challenge record rejected: {error}");
                false
            })
    }

    /// Present the sheet for the oldest awaiting challenge (the production
    /// presentation path; also the benchmark's measured entry). Opening
    /// replaces any open command surface or find presentation (exactly one
    /// portal presentation per window) and captures the origin focus domain
    /// for deterministic restoration on submit/cancel. Returns whether a
    /// sheet was presented.
    pub fn auth_present_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let origin_chrome_focused = self.chrome_focused(window);
        let presented = self.auth.get().present_next();
        // A superseded or unpresentable challenge from WebKit is answered.
        self.answer_auth_engine();
        if presented.is_none() {
            return false;
        }
        self.auth.get().capture_origin_focus(origin_chrome_focused);
        // The sheet owns the keyboard while open (the portal window is
        // never key): any other surface that held it must close.
        self.dismiss_command_portal(cx);
        if self.find.state.open {
            self.find_discard(cx);
        }
        self.present_auth_portal(window, cx);
        window.focus(&self.auth_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
        true
    }

    /// The production submit path: the sheet's credentials are captured into
    /// the coordinator's in-memory store and the decision is applied through
    /// the contract (`Respond { response: UseCredential(handle) }`). The
    /// sheet holds no authority; the lifecycle decides. Closes the sheet and
    /// restores the origin focus domain. Returns whether a decision was made.
    pub fn auth_submit(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.auth_sheet_is_presented() {
            return false;
        }
        let decision = self.auth.get().submit();
        if decision == ChallengeDecision::Noop {
            return false;
        }
        self.answer_auth_engine();
        let origin_chrome_focused = self.auth.read().origin_focus();
        self.auth.get().close_sheet();
        self.dismiss_auth_presentation(cx);
        self.restore_auth_focus(origin_chrome_focused, window, cx);
        cx.notify();
        true
    }

    /// The production cancel path: the challenge is cancelled through the
    /// contract — the request **fails** and never retries. Closes the sheet
    /// and restores the origin focus domain.
    pub fn auth_cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.auth_sheet_is_presented() {
            return false;
        }
        let decision = self.auth.get().cancel();
        if decision == ChallengeDecision::Noop {
            return false;
        }
        self.answer_auth_engine();
        let origin_chrome_focused = self.auth.read().origin_focus();
        self.auth.get().close_sheet();
        self.dismiss_auth_presentation(cx);
        self.restore_auth_focus(origin_chrome_focused, window, cx);
        cx.notify();
        true
    }

    /// Close the sheet presentation (submit/cancel dismiss the portal; the
    /// portal state enforces exactly one active presentation).
    fn dismiss_auth_presentation(&mut self, cx: &mut Context<Self>) {
        self.portal_dismiss(cx);
    }

    fn restore_auth_focus(
        &mut self,
        origin_chrome_focused: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
    }

    /// Whether the credential sheet is currently presented: the coordinator
    /// has a presented challenge AND the portal shows the sheet's
    /// presentation (a replaced presentation — e.g. another surface opened
    /// over it — reports closed, so a stale submit can never answer a sheet
    /// that is not on screen).
    pub fn auth_sheet_is_presented(&self) -> bool {
        self.auth.read().presented().is_some()
            && self
                .portal
                .active()
                .is_some_and(|presentation| presentation.id == AUTH_PRESENTATION_ID)
    }

    /// The redacted render data for the portal window; `None` when no sheet
    /// is presented.
    pub fn auth_surface(&self) -> Option<AuthSheetSurface> {
        self.auth.read().surface()
    }

    /// The redacted wire-visible snapshot of the credential sheet.
    pub fn auth_snapshot(&self) -> AuthSnapshotState {
        self.auth.read().snapshot()
    }

    /// Handle keystrokes while the credential sheet is open (the main-window
    /// root routes here): printable characters type into the focused field,
    /// Tab moves between fields, Enter submits, Escape cancels. The typed
    /// values stay in the sheet; only the redacted handle can leave it.
    pub fn on_auth_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.auth_sheet_is_presented() {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "enter" => {
                cx.stop_propagation();
                self.auth_submit(window, cx);
            }
            "tab" => {
                cx.stop_propagation();
                self.auth.get().sheet().focus_next();
                cx.notify();
            }
            "backspace" => {
                cx.stop_propagation();
                self.auth.get().sheet().backspace();
                cx.notify();
            }
            "escape" => {
                // The global `escape → FocusWeb` binding normally routes
                // here via `cancel_command_bar`; this arm is the fallback
                // when Escape reaches the root directly.
                cx.stop_propagation();
                self.auth_cancel(window, cx);
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        cx.stop_propagation();
                        self.auth.get().sheet().type_char(ch);
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Move the credential sheet's field cursor to the next field (the
    /// `AuthSheetFieldTab` action handler; the sheet owns the keyboard while
    /// open). A no-op when no sheet is presented — Tab then falls through to
    /// gpui-component's default focus traversal, which had no observable
    /// sheet effect before this action existed.
    pub fn auth_field_tab(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Blueprint D05: the Delete Space sheet shares the Tab binding.
        if self.space_delete_toggle_focus(cx) {
            return;
        }
        if !self.auth_sheet_is_presented() {
            return;
        }
        self.auth.get().sheet().focus_next();
        cx.notify();
    }

    /// Present the sheet through the one native portal host (the §11.4
    /// command-surface family, centered).
    fn present_auth_portal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let request = PortalRequest::native(
            AUTH_PRESENTATION_ID,
            auth_placement(),
            AUTH_PORTAL_PRIORITY,
            // A credential sheet is never dismissed by an outside pointer
            // press: the only exits are Log In and Cancel (sensitive input;
            // the portal backdrop stops propagation).
            DismissalPolicy::none(),
        );
        self.portal_probe
            .borrow_mut()
            .present_auth(request, viewport);
        self.activate_portal_request(request, cx);
    }
}

/// The sheet's centered placement (the §11.4 command-surface family).
fn auth_placement() -> PortalPlacement {
    PortalPlacement::Centered {
        size: Sz::new(AUTH_SHEET_WIDTH, AUTH_SHEET_HEIGHT),
        margin: Edges::all(8.0),
    }
}
