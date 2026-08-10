//! Engine-event application on the [`super::BrowserController`]: applies one
//! [`WebEvent`] to its owning tab's domain/navigation state and reports
//! unknown/stale views without corrupting any state. Split from
//! `controller.rs` so the gateway module stays under the ~500-line budget.

use bbr_web::{WebEvent, WebViewId};

use crate::outcome::ControllerEvent;

use super::BrowserController;

impl BrowserController {
    /// Apply one engine event to its owning tab. Events for unknown/stale
    /// `WebViewId`s are reported without corrupting any state.
    pub fn apply_event(&mut self, event: WebEvent) -> ControllerEvent {
        let view = event_view(&event);
        if self.apply_preview_event(&event) {
            return ControllerEvent::WebEvent { event };
        }
        let Some(tab) = self.registry.tab_for_view(view) else {
            return ControllerEvent::UnknownWebView { view };
        };
        match &event {
            WebEvent::PageCrashed(_) => return self.on_page_crash(view),
            WebEvent::UrlChanged(_, url) => {
                let _ = self.browser.apply_navigation(tab, url, "");
            }
            WebEvent::TitleChanged(_, title) => {
                let _ = self.browser.apply_navigation(tab, "", title);
            }
            _ => {}
        }
        // Loading/history/crashed live only in the session's nav state.
        self.nav.entry(view).or_default().apply(&event);
        self.check_invariants();
        ControllerEvent::WebEvent { event }
    }
}

/// The [`WebViewId`] an event names.
fn event_view(event: &WebEvent) -> WebViewId {
    match event {
        WebEvent::UrlChanged(v, _) => *v,
        WebEvent::TitleChanged(v, _) => *v,
        WebEvent::LoadingChanged(v, _) => *v,
        WebEvent::CanGoBackChanged(v, _) => *v,
        WebEvent::CanGoForwardChanged(v, _) => *v,
        WebEvent::DidCommitNavigation(v) => *v,
        WebEvent::PageCrashed(v) => *v,
        WebEvent::NavigationFailed(v, _) => *v,
        // A context-menu request names the view it was requested on (the
        // menu is BBR chrome over that view; the request carries no
        // navigation state).
        WebEvent::ContextMenuRequested(v, _) => *v,
    }
}
