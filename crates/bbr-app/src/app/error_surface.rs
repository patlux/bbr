//! The web-error presentation state (task packet `error-surface-contract`).
//!
//! Composition: the native adapter (`bbr-web-macos::error_mapping`)
//! classifies WebKit failures into the merged contract's closed taxonomy
//! (`bbr-web::web_errors`); the app owns the **incident lifecycle, the
//! contract-derived recovery action policy, the calm copy derivation, and
//! the DTO building** (the anchor keeps DTO building app-owned); the
//! ordinary GPUI chrome surface drawn in the content area is the extracted
//! `bbr_ui::surfaces::error` component (`ErrorSurfaceModel` +
//! `error_surface`). The app wiring (`on_incident`, recovery/dismiss
//! routing, native-view hide/restore) lives in `super::error_surface_behavior`.
//!
//! ## Contract invariants the surface upholds
//!
//! - The surface presents **exactly** the classified error the contract
//!   produces: title, copy, and actions are derived from the closed
//!   category/subreason, never guessed from raw platform codes.
//! - Recovery actions come from the category-legal matrix and every request
//!   routes through [`bbr_web::web_errors::validate_action`] before an
//!   engine action is dispatched. TLS never offers a bypass — the closed
//!   action set has no bypass variant at all.
//! - The surface is plain GPUI chrome: no portal host, no composited web
//!   content. While it shows, the native web view of the affected tab is
//!   hidden (never destroyed — identity is untouched).
//! - Showing, dismissing, or recovering never changes tab, session, profile,
//!   or space identity. The renderer-crash presentation stays on the
//!   DESIGN.md §11.8 crash panel (the controller-driven `bbr-crash-*`
//!   surface whose Reload recreates the session); the incident lifecycle
//!   still records the crash classification, but the crash panel is the
//!   visible surface for the process-crash class. This surface covers the
//!   TLS, navigation, and offline classes.
//! - Copy is conventional and calm, names what failed and what the user can
//!   do, and never shows a raw platform error code as the primary message.

use std::collections::{HashMap, HashSet};

#[cfg(test)]
use bbr_web::web_errors::validate_action;
use bbr_web::web_errors::{
    Incident, IncidentCategory, IncidentContext, IncidentId, IncidentSubReason, Lifecycle,
    LifecycleState, RecoveryAction,
};
use bbr_web::{ProfileId, WebViewId};

use bbr_ui::surfaces::error::ErrorSurfaceModel;
use gpui::SharedString;

/// The readable failing URL the surface presents: the contract's
/// privacy-safe reduced origin (scheme://host[:port]). `None` when the
/// origin is unknown (the surface then omits the URL line rather than
/// guessing).
pub type DisplayOrigin = Option<String>;

/// The presentation spec the render draws: everything is derived from the
/// classified incident and the contract lifecycle — the surface holds no
/// independent copy of the error.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorSurfaceSpec {
    /// The web view the incident is scoped to.
    pub view: WebViewId,
    /// The classified incident (id, category, subreason, privacy-safe
    /// context).
    pub incident: Incident,
    /// The contract lifecycle state of the incident (Reported,
    /// RecoveryRequested, RecoveryFailed).
    pub state: LifecycleState,
    /// Calm, conventional title (never a raw platform code).
    pub title: &'static str,
    /// Calm explanation: what failed and what the user can do.
    pub copy: &'static str,
    /// The readable failing origin, when the contract knows one.
    pub origin: DisplayOrigin,
    /// The primary recovery action (contract-legal for the category).
    pub action: RecoveryAction,
    /// The primary action's button label.
    pub action_label: &'static str,
    /// The secondary action's button label (Dismiss — a lifecycle event,
    /// never a tab mutation).
    pub dismiss_label: &'static str,
}

impl ErrorSurfaceSpec {
    /// The presentation model the extracted `bbr-ui` surface renders (the
    /// anchor keeps DTO building app-owned): the calm title/copy, the
    /// readable failing origin, and the action/dismiss labels. The ids and
    /// anatomy live in `bbr_ui::surfaces::error`; this projection is the
    /// only bridge from the contract-derived spec to the surface.
    pub fn to_model(&self) -> ErrorSurfaceModel {
        ErrorSurfaceModel {
            title: self.title.into(),
            copy: self.copy.into(),
            origin: self.origin.clone().map(SharedString::from),
            action_label: self.action_label.into(),
            dismiss_label: self.dismiss_label.into(),
            // D10: the offline (network) class shows the drawn illustration
            // (the render supplies the playing key).
            illustration: (self.incident.reason.category()
                == bbr_web::web_errors::IncidentCategory::Offline)
                .then_some(None),
        }
    }
}

/// App-local presentation state: per-web-view incident lifecycles (the
/// contract's identity guards), the monotonic per-view sequence allocator
/// (the contract's "controller allocates increasing values" rule), the
/// app-side retry target per view, and the set of native views currently
/// hidden because their error surface replaced them.
#[derive(Debug, Default)]
pub struct ErrorSurfaceState {
    by_view: HashMap<WebViewId, Lifecycle>,
    next_sequence: HashMap<WebViewId, u64>,
    /// The full failing URL per view (the retry target; display always uses
    /// the reduced origin).
    requested_url: HashMap<WebViewId, String>,
    /// Native views hidden by an active error surface (restored on
    /// dismissal/recovery; identity untouched).
    hidden: HashSet<WebViewId>,
}

impl ErrorSurfaceState {
    /// Present a classified incident for `web_view` in `profile`. The
    /// incident id is allocated here (monotonic per view); duplicate or
    /// stale ids are rejected by the contract lifecycle. Returns the
    /// presentation spec when the incident is now visible.
    pub fn present(
        &mut self,
        profile: ProfileId,
        web_view: WebViewId,
        reason: IncidentSubReason,
        context: IncidentContext,
        requested_url: Option<String>,
    ) -> Option<ErrorSurfaceSpec> {
        let sequence = self
            .next_sequence
            .entry(web_view)
            .and_modify(|s| *s += 1)
            .or_insert(1);
        let incident = Incident::new(
            IncidentId::new(profile, web_view, *sequence),
            reason.category(),
            reason,
            context,
        )
        .expect("mapping outcome is a consistent definition");
        if let Some(url) = requested_url {
            self.requested_url.insert(web_view, url);
        }
        match self.by_view.get(&web_view) {
            Some(tracked) => match tracked.supersede_with(incident.clone()) {
                Ok(next) => {
                    self.by_view.insert(web_view, next);
                }
                Err(error) => {
                    log::warn!("incident supersession rejected: {error}");
                }
            },
            None => {
                self.by_view.insert(web_view, Lifecycle::report(incident));
            }
        }
        self.spec_for(web_view)
    }

    /// The presentation spec for `view`, when its incident is visible
    /// (non-terminal lifecycle). `None` for a terminal or absent incident.
    /// The renderer-crash class is presented by the DESIGN.md §11.8 crash
    /// panel (the controller-driven `bbr-crash-*` surface), so the caller
    /// decides whether a `ProcessCrash` spec is drawn; the lifecycle still
    /// records the classification and its `Restore` recovery here.
    pub fn spec_for(&self, view: WebViewId) -> Option<ErrorSurfaceSpec> {
        let lifecycle = self.by_view.get(&view)?;
        if matches!(
            lifecycle.state(),
            LifecycleState::Recovered | LifecycleState::Dismissed | LifecycleState::Superseded
        ) {
            return None;
        }
        let incident = lifecycle.incident();
        let (title, copy) = copy_for(incident.reason);
        let action = match incident.reason.category() {
            IncidentCategory::Tls | IncidentCategory::Navigation | IncidentCategory::Offline => {
                RecoveryAction::Retry
            }
            IncidentCategory::ProcessCrash => RecoveryAction::Restore,
        };
        Some(ErrorSurfaceSpec {
            view,
            incident: incident.clone(),
            state: lifecycle.state(),
            title,
            copy,
            origin: incident.context.origin().map(str::to_owned),
            action,
            action_label: action_label(action),
            dismiss_label: "Dismiss",
        })
    }

    /// Request a recovery action for `view` (validated against the
    /// category-legal matrix by the contract lifecycle). Returns the action
    /// when the request was accepted, so the caller can dispatch the engine
    /// action.
    pub fn request_recovery(
        &mut self,
        view: WebViewId,
        action: RecoveryAction,
    ) -> Option<RecoveryAction> {
        let lifecycle = self.by_view.get_mut(&view)?;
        lifecycle.request_recovery(action).ok()?;
        Some(action)
    }

    /// The action currently in flight for `view`, if any (drives the
    /// recovery-completion path in `on_web_event`).
    pub fn recovery_in_flight(&self, view: WebViewId) -> Option<RecoveryAction> {
        self.by_view
            .get(&view)
            .and_then(|lifecycle| lifecycle.state().action())
    }

    /// Mark `view`'s incident recovered (a committed navigation after a
    /// recovery request). Returns whether the surface cleared.
    pub fn recovered(&mut self, view: WebViewId, action: RecoveryAction) -> bool {
        let Some(lifecycle) = self.by_view.get_mut(&view) else {
            return false;
        };
        let id = lifecycle.id();
        if lifecycle.recovered(id, action).is_err() {
            return false;
        }
        self.by_view.remove(&view);
        self.requested_url.remove(&view);
        true
    }

    /// Dismiss `view`'s incident (a lifecycle event — identity is never
    /// touched). Returns whether the surface cleared.
    pub fn dismiss(&mut self, view: WebViewId) -> bool {
        let Some(lifecycle) = self.by_view.get_mut(&view) else {
            return false;
        };
        if lifecycle.dismiss().is_err() {
            return false;
        }
        self.by_view.remove(&view);
        self.requested_url.remove(&view);
        true
    }

    /// Drop `view`'s resolved presentation and restore its native view.
    pub fn clear_view(&mut self, view: WebViewId) -> bool {
        self.by_view.remove(&view);
        self.requested_url.remove(&view);
        self.hidden.remove(&view)
    }

    /// The retry target (full failing URL) for `view`.
    pub fn retry_url(&self, view: WebViewId) -> Option<String> {
        self.requested_url.get(&view).cloned()
    }

    /// Whether `view`'s native session is currently hidden for the surface.
    pub fn is_hidden(&self, view: WebViewId) -> bool {
        self.hidden.contains(&view)
    }

    /// Record that the surface replaced `view`'s native view (the engine
    /// `show(false)` was issued). Idempotent.
    pub fn mark_hidden(&mut self, view: WebViewId) {
        self.hidden.insert(view);
    }

    /// Restore `view`'s native view after its surface cleared. Returns
    /// whether a `show(true)` is needed (the view was hidden by us).
    pub fn take_hidden(&mut self, view: WebViewId) -> bool {
        self.hidden.remove(&view)
    }

    /// Drop `view`'s presentation entirely (a tab closed): the session is
    /// gone, so no native restore is needed.
    pub fn forget_view(&mut self, view: WebViewId) {
        self.by_view.remove(&view);
        self.next_sequence.remove(&view);
        self.requested_url.remove(&view);
        self.hidden.remove(&view);
    }
}

/// Calm, conventional copy per classified subreason: names what failed and
/// what the user can do. Never a raw platform error code; TLS copy never
/// suggests a bypass.
fn copy_for(reason: IncidentSubReason) -> (&'static str, &'static str) {
    use bbr_web::web_errors::{NavigationReason, OfflineReason, TlsReason};
    use IncidentSubReason::{Navigation, Offline, Tls};
    match reason {
        Tls(TlsReason::CertificateExpired) => (
            "This page can't be loaded securely",
            "The site's security certificate has expired, so the connection wasn't made. Contact the site owner or try again later.",
        ),
        Tls(TlsReason::CertificateNotYetValid) => (
            "This page can't be loaded securely",
            "The site's security certificate isn't valid yet, so the connection wasn't made. Contact the site owner or try again later.",
        ),
        Tls(TlsReason::CertificateUntrusted) => (
            "This page can't be loaded securely",
            "The site's security certificate isn't trusted, so the connection wasn't made. Contact the site owner.",
        ),
        Tls(TlsReason::CertificateRevoked) => (
            "This page can't be loaded securely",
            "The site's security certificate has been revoked, so the connection wasn't made. Contact the site owner.",
        ),
        Tls(TlsReason::HostnameMismatch) => (
            "This page can't be loaded securely",
            "The site's security certificate doesn't match this address, so the connection wasn't made. Check the address and try again.",
        ),
        Tls(TlsReason::HandshakeFailed) => (
            "This page can't be loaded securely",
            "The secure connection to this site failed. Try again in a moment.",
        ),
        Navigation(NavigationReason::ServerNotFound) => (
            "This page can't be loaded",
            "The site's server couldn't be found. Check the address and try again.",
        ),
        Navigation(NavigationReason::ConnectionRefused) => (
            "This page can't be loaded",
            "The site refused the connection. It may be temporarily unavailable — try again.",
        ),
        Navigation(NavigationReason::RequestTimedOut) => (
            "This page can't be loaded",
            "The site took too long to respond. Try again.",
        ),
        Navigation(NavigationReason::RedirectLoop) => (
            "This page can't be loaded",
            "The page redirected too many times — a problem with the site. Try again or check the address.",
        ),
        Navigation(NavigationReason::UnsupportedScheme) => (
            "This page can't be loaded",
            "This address uses a scheme this browser can't open.",
        ),
        Offline(OfflineReason::NoNetworkConnection) | Offline(OfflineReason::ProxyUnreachable) => (
            "You're offline",
            "There's no network connection right now. Check your connection and try again.",
        ),
        Offline(OfflineReason::NetworkUnreachable) => (
            "You're offline",
            "The network can't reach this site right now. Check your connection and try again.",
        ),
        IncidentSubReason::Crash(_) => (
            "This page crashed",
            "The tab stays open; reload to restore it.",
        ),
    }
}

/// The button label for a recovery action (DESIGN §11.8 tone: verbs, calm).
fn action_label(action: RecoveryAction) -> &'static str {
    match action {
        RecoveryAction::Retry => "Try again",
        RecoveryAction::Reload => "Reload",
        RecoveryAction::GoBack => "Back",
        RecoveryAction::OpenSettings => "Network settings",
        RecoveryAction::InspectCertificate => "Show certificate",
        RecoveryAction::Restore => "Reload",
        RecoveryAction::Dismiss => "Dismiss",
    }
}

/// Test-only helper: the recovery action must always pass the contract's
/// own validation before the engine dispatches anything.
#[cfg(test)]
pub fn is_legal_action(category: IncidentCategory, action: RecoveryAction) -> bool {
    validate_action(category, action).is_ok()
}

#[cfg(test)]
mod error_surface_tests;
