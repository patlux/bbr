//! Blueprint D13 (board M-13): the blocked pop-up notice.
//!
//! A page's pop-up without a user gesture is blocked by the new-window
//! policy. Instead of vanishing silently, a quiet info banner over the card
//! says so — "A pop-up from example-shop.com was blocked." — with Open
//! (this one, as a tab) and Always Allow (the site's pop-ups from now on,
//! per profile, in `site-permissions.txt`). The banner belongs to the page
//! that asked: it shows while that page is active and leaves when the page
//! goes to another site, when a newer pop-up replaces it, or after 12 s.

use std::time::Duration;

use bbr_web::new_window::{NewWindowDecision, NewWindowDelivery, NewWindowRequest};
use bbr_web::ProfileId;
use gpui::{Context, Window};

use crate::app::BbrApp;
use crate::site_permissions;

/// How long the notice stays.
pub(crate) const POPUP_NOTICE_DURATION: Duration = Duration::from_secs(12);

/// The blocked pop-up the notice offers.
#[derive(Debug, Clone)]
pub(crate) struct PopupNotice {
    pub request: NewWindowRequest,
    pub origin: String,
    pub profile: ProfileId,
    pub generation: u64,
}

impl BbrApp {
    /// Republish which live pages may open pop-ups (the synchronous
    /// new-window handler reads it).
    pub(crate) fn publish_popup_views(&self) {
        let policies = self.permissions.policies.read();
        let views = self
            .controller
            .read()
            .live_view_urls()
            .into_iter()
            .filter(|(_, profile, url)| {
                site_permissions::origin_of(url)
                    .is_some_and(|origin| policies.popups_allowed(*profile, &origin))
            })
            .map(|(view, _, _)| view.0)
            .collect();
        site_permissions::publish_popup_views(views);
    }

    /// The new-window policy blocked a pop-up: show the notice.
    pub(crate) fn popup_blocked(&mut self, request: NewWindowRequest, cx: &mut Context<Self>) {
        let Some((_, profile, url)) = self
            .controller
            .read()
            .live_view_urls()
            .into_iter()
            .find(|(view, _, _)| *view == request.source)
        else {
            return;
        };
        let Some(origin) = site_permissions::origin_of(&url) else {
            return;
        };
        self.permissions.popup_generation += 1;
        let generation = self.permissions.popup_generation;
        self.permissions.popup = Some(PopupNotice {
            request,
            origin,
            profile,
            generation,
        });
        self.permissions.popup_bounds.set(None);
        self.permissions.popup_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(POPUP_NOTICE_DURATION).await;
            let _ = this.update(cx, |app, cx| {
                if app
                    .permissions
                    .popup
                    .as_ref()
                    .is_some_and(|notice| notice.generation == generation)
                {
                    app.dismiss_popup_notice(cx);
                }
            });
        }));
        self.sync_portal_overlays(cx);
    }

    pub(crate) fn dismiss_popup_notice(&mut self, cx: &mut Context<Self>) {
        if self.permissions.popup.take().is_some() {
            self.permissions.popup_bounds.set(None);
            self.permissions.popup_task = None;
            self.sync_portal_overlays(cx);
        }
    }

    /// The notice shows while its page is the active one.
    pub(crate) fn popup_notice_visible(&self) -> Option<&PopupNotice> {
        let notice = self.permissions.popup.as_ref()?;
        (self.controller.read().snapshot().active_view == Some(notice.request.source))
            .then_some(notice)
    }

    /// The notice's message.
    pub(crate) fn popup_notice_message(&self) -> Option<String> {
        self.popup_notice_visible().map(|notice| {
            format!(
                "A pop-up from {} was blocked.",
                site_permissions::origin_host(&notice.origin)
            )
        })
    }

    /// Open: this pop-up opens as a tab of the page's profile.
    pub(crate) fn popup_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(notice) = self.permissions.popup.take() else {
            return;
        };
        self.permissions.popup_task = None;
        self.permissions.popup_bounds.set(None);
        let delivery = NewWindowDelivery {
            outcome: bbr_web::decide(&notice.request, NewWindowDecision::OpenTab),
            request: notice.request,
        };
        let _ = self.on_new_window_request(delivery, window, cx);
        self.sync_portal_overlays(cx);
    }

    /// Always Allow: remember the site, then open this pop-up.
    pub(crate) fn popup_always_allow(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(notice) = self.permissions.popup.as_ref() else {
            return;
        };
        let (profile, origin) = (notice.profile, notice.origin.clone());
        self.permissions
            .policies
            .get()
            .set_popups(profile, &origin, true);
        self.save_site_permissions();
        self.publish_popup_views();
        self.popup_open(window, cx);
    }
}
