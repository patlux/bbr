//! The observable browser-shell state (Phase 1).
//!
//! This is the subset of the future `bbr-domain` model that the spike needs:
//! it is what the GPUI chrome renders and what the E2E gateway snapshots.
//! It is deliberately plain-data and portable (no GPUI, no WebKit).

use std::fmt;

use bbr_web::{ProfileId, WebEvent, WebViewId};

/// A typed snapshot of the observable app state, matched 1:1 to
/// `bbr_testkit::protocol::Snapshot` (conversion lives in the testkit glue).
#[cfg(feature = "testkit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TestSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub web_view_hosted: bool,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    pub window_size: [f32; 2],
}

/// Navigation state of the active web view, mirrored from `WebEvent`s.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NavigationState {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub crashed: bool,
}

impl NavigationState {
    /// Apply one engine event to the mirrored state.
    pub fn apply(&mut self, event: &WebEvent) {
        match event {
            WebEvent::UrlChanged(_, url) => {
                self.url = Some(url.clone());
                self.crashed = false;
            }
            WebEvent::TitleChanged(_, title) => {
                self.title = Some(title.clone());
                self.crashed = false;
            }
            WebEvent::LoadingChanged(_, loading) => self.loading = *loading,
            WebEvent::CanGoBackChanged(_, can) => self.can_go_back = *can,
            WebEvent::CanGoForwardChanged(_, can) => self.can_go_forward = *can,
            WebEvent::PageCrashed(_) => {
                self.crashed = true;
                self.loading = false;
            }
            WebEvent::NavigationFailed(_, message) => {
                self.loading = false;
                // Keep the last good URL; the failure is observable via the
                // gateway event and the status line.
                log::warn!("navigation failed: {message}");
            }
        }
    }

    /// A human-readable one-liner for the status line.
    pub fn status_line(&self) -> String {
        if self.crashed {
            return "⚠ Web content process terminated".into();
        }
        if self.loading {
            return "Loading…".into();
        }
        match (&self.url, &self.title) {
            (Some(_), Some(title)) if !title.is_empty() => title.to_string(),
            (Some(url), _) => url.clone(),
            _ => "No page".into(),
        }
    }
}

/// One profile + one active web view: the whole Phase-1 identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellState {
    pub profile: ProfileId,
    pub active_view: WebViewId,
    pub nav: NavigationState,
    /// Whether the native web view is currently hosted by the engine.
    pub web_view_hosted: bool,
    /// Engine implementation tag for diagnostics/snapshots.
    pub engine_kind: &'static str,
}

impl ShellState {
    pub fn new(profile: ProfileId, active_view: WebViewId, engine_kind: &'static str) -> Self {
        Self {
            profile,
            active_view,
            nav: NavigationState::default(),
            web_view_hosted: false,
            engine_kind,
        }
    }
}

impl fmt::Display for ShellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ShellState {{ profile: {:?}, view: {:?}, nav: {} }}",
            self.profile,
            self.active_view,
            self.nav.status_line()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_mirror_into_navigation_state() {
        let mut nav = NavigationState::default();
        nav.apply(&WebEvent::LoadingChanged(WebViewId(1), true));
        nav.apply(&WebEvent::UrlChanged(WebViewId(1), "https://a".into()));
        nav.apply(&WebEvent::TitleChanged(WebViewId(1), "A".into()));
        nav.apply(&WebEvent::CanGoBackChanged(WebViewId(1), true));
        nav.apply(&WebEvent::LoadingChanged(WebViewId(1), false));
        assert_eq!(
            nav,
            NavigationState {
                url: Some("https://a".into()),
                title: Some("A".into()),
                loading: false,
                can_go_back: true,
                can_go_forward: false,
                crashed: false,
            }
        );
        assert_eq!(nav.status_line(), "A");
    }

    #[test]
    fn crash_is_observable() {
        let mut nav = NavigationState::default();
        nav.apply(&WebEvent::PageCrashed(WebViewId(1)));
        assert!(nav.crashed);
        assert!(!nav.loading);
        assert!(nav.status_line().contains("terminated"));
    }
}
