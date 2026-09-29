//! Per-session navigation state mirrored from engine events.
//!
//! Loading and history availability are deliberately *not* part of the
//! persistent domain model: they describe a live session, reset whenever the
//! session is recreated (suspend/discard/reopen), and are owned by the
//! controller.

/// The observable navigation state of one session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavState {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub crashed: bool,
    /// The session's initial document committed at least once
    /// (`DidCommitNavigation`). A child tab whose initial document never
    /// committed (an ambiguous provisional load) is not a settled root and
    /// must never trigger the Back-at-root opener return.
    pub initial_committed: bool,
    /// A navigation failure was reported for this session (settled
    /// failure/crash state; also covered by `crashed`).
    pub failed: bool,
}

impl NavState {
    /// True when the session is at a **settled root**: its initial document
    /// committed, or it is showing a settled navigation-failure/crash state.
    /// An ambiguous provisional load (no commit yet, no failure) is never
    /// settled, so a Back gesture over a still-loading child is a no-op.
    pub fn settled_root(&self) -> bool {
        self.initial_committed || self.crashed || self.failed
    }

    /// Apply a navigation-state event (url/title/loading/history/crash).
    pub fn apply(&mut self, event: &bbr_web::WebEvent) {
        match event {
            bbr_web::WebEvent::UrlChanged(_, url) => {
                self.url = Some(url.clone());
                self.crashed = false;
            }
            bbr_web::WebEvent::TitleChanged(_, title) => {
                self.title = Some(title.clone());
                self.crashed = false;
            }
            bbr_web::WebEvent::LoadingChanged(_, loading) => self.loading = *loading,
            bbr_web::WebEvent::CanGoBackChanged(_, can) => self.can_go_back = *can,
            bbr_web::WebEvent::CanGoForwardChanged(_, can) => self.can_go_forward = *can,
            bbr_web::WebEvent::DidCommitNavigation(_) => {
                self.initial_committed = true;
                self.loading = false;
            }
            bbr_web::WebEvent::PageCrashed(_) => {
                self.crashed = true;
                self.loading = false;
            }
            bbr_web::WebEvent::NavigationFailed(_, _) => {
                self.failed = true;
                self.loading = false;
            }
            // Renderer availability is not navigation state: the liveness
            // monitor owns it (Unresponsive is a sibling of Crashed, never
            // a url/title/loading/history fact).
            bbr_web::WebEvent::UnresponsiveChanged(_, _) => {}
            // A context-menu request carries no navigation state (the menu
            // is BBR chrome over the page; opening/closing it never changes
            // url/title/loading/history).
            bbr_web::WebEvent::ContextMenuRequested(_, _) => {}
            // A favicon acquisition carries no navigation state (the image
            // is presentation data keyed by host in the controller's store).
            bbr_web::WebEvent::FaviconChanged(_, _) => {}
            // Blueprint D11: permission requests and capture state are the
            // app's site-permission state, not navigation state.
            bbr_web::WebEvent::PermissionRequested { .. }
            | bbr_web::WebEvent::ClientCertificateRequested { .. }
            | bbr_web::WebEvent::ExternalSchemeRequested { .. }
            | bbr_web::WebEvent::MediaCaptureChanged { .. } => {}
        }
    }
}
