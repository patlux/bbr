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
}

impl NavState {
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
            bbr_web::WebEvent::PageCrashed(_) => {
                self.crashed = true;
                self.loading = false;
            }
            bbr_web::WebEvent::NavigationFailed(_, _) => {
                self.loading = false;
            }
        }
    }
}
