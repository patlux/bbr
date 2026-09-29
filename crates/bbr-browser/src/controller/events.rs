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
            // The renderer-availability monitor owns unresponsive events
            // (sibling of the crash authority; the session stays alive).
            WebEvent::UnresponsiveChanged(_, unresponsive) => {
                return self.on_unresponsive_changed(view, *unresponsive)
            }
            WebEvent::UrlChanged(_, url) => {
                let _ = self.browser.apply_navigation(tab, url, "");
            }
            WebEvent::TitleChanged(_, title) => {
                let _ = self.browser.apply_navigation(tab, "", title);
            }
            // A per-page favicon acquisition observation (task packet
            // `real-favicon-pipeline.md`): the resolved image bytes land in
            // the owning profile's bounded host-keyed store, keyed by the
            // tab's current host, so every presentation of this host
            // (tab rows, Favorites tiles, folder items) derives from the
            // same record. Any other content shape is ignored (the macOS
            // adapter emits only resolved bytes).
            WebEvent::FaviconChanged(_, content) => {
                self.apply_favicon_changed(tab, view, content);
            }
            _ => {}
        }
        // Loading/history/crashed live only in the session's nav state.
        self.nav.entry(view).or_default().apply(&event);
        self.check_invariants();
        ControllerEvent::WebEvent { event }
    }
}

impl BrowserController {
    /// Store one acquired favicon observation for `tab`'s current host in
    /// the owning profile's bounded store (task packet
    /// `real-favicon-pipeline.md`). Only resolved image bytes are stored; a
    /// candidates-shaped payload is ignored (the macOS adapter emits only
    /// `Image`). Unknown/stale sessions are dropped without corruption
    /// (the caller already resolved `tab` from the registry, so the view is
    /// live; a missing profile mapping is still dropped defensively).
    fn apply_favicon_changed(
        &mut self,
        tab: bbr_domain::TabId,
        view: WebViewId,
        content: &bbr_web::FaviconContent,
    ) {
        let bbr_web::FaviconContent::Image(bytes) = content else {
            return;
        };
        let host = crate::favicon::favicon_host(
            &self
                .browser
                .tab(tab)
                .map(|state| state.url.clone())
                .unwrap_or_default(),
        );
        if host.is_empty() {
            return;
        }
        let Some(profile) = self.session_profiles.get(&view).copied() else {
            return;
        };
        self.favicons.insert(profile, &host, bytes.clone());
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
        WebEvent::UnresponsiveChanged(v, _) => *v,
        // A context-menu request names the view it was requested on (the
        // menu is BBR chrome over that view; the request carries no
        // navigation state).
        WebEvent::ContextMenuRequested(v, _) => *v,
        WebEvent::FaviconChanged(v, _) => *v,
        WebEvent::PermissionRequested { view, .. } => *view,
        WebEvent::ClientCertificateRequested { view, .. } => *view,
        WebEvent::ExternalSchemeRequested { view, .. } => *view,
        WebEvent::MediaCaptureChanged { view, .. } => *view,
    }
}
