//! Blueprint D11 (boards M-8, M-9, L-6, PR-14, DESIGN §11.5): permission
//! prompts, the in-use and blocked device marks, and site settings.
//!
//! A page's camera/microphone request arrives as
//! [`WebEvent::PermissionRequested`]. A remembered Allow for every asked
//! capability grants it at once; a remembered Block refuses it quietly
//! (the address shows the blocked mark); anything else queues one prompt,
//! anchored to the address, for the active page — a second request waits
//! behind it. Nothing is granted by default. The prompt, and the site
//! settings the address's leading slot opens, float over the card in the
//! native Portal's overlay layer.

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use bbr_web::permissions::PermissionCapability;
use bbr_web::{ProfileId, WebEvent, WebViewId};
use gpui::Context;

use super::BbrApp;
use crate::site_permissions::{self, SitePermissions, SitePolicy};

pub(crate) mod popups;
pub(crate) mod site_settings;
#[cfg(feature = "testkit")]
pub(crate) mod testkit;

/// One request waiting for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingRequest {
    pub request: u64,
    pub view: WebViewId,
    pub profile: ProfileId,
    pub origin: String,
    pub capabilities: Vec<PermissionCapability>,
}

/// The open site-settings popover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteSettingsState {
    pub origin: String,
    pub profile: ProfileId,
    pub clear_confirm: bool,
}

/// The app's permission state.
#[derive(Default)]
pub(crate) struct PermissionsState {
    // Blueprint D19: the policies, requests and per-view marks are shared
    // by every window; the prompt, popover and notice are this window's.
    pub policies: crate::shared::Shared<SitePermissions>,
    pub path: Option<std::path::PathBuf>,
    pub pending: crate::shared::Shared<VecDeque<PendingRequest>>,
    /// "Remember for this site" on the prompt shown now.
    pub remember: bool,
    /// Live capture per view: (camera, microphone).
    pub capture: crate::shared::Shared<HashMap<WebViewId, (bool, bool)>>,
    /// A remembered Block refused this view's request for the origin
    /// (the camera was among the refused capabilities).
    pub blocked: crate::shared::Shared<HashMap<WebViewId, (String, bool)>>,
    pub site_settings: Option<SiteSettingsState>,
    /// The prompt's and the popover's rendered bounds (their hit regions),
    /// recorded by the Portal render.
    pub prompt_bounds: Rc<Cell<Option<[f32; 4]>>>,
    pub popover_bounds: Rc<Cell<Option<[f32; 4]>>>,
    /// Blueprint D13: the blocked pop-up notice, its bounds and timer.
    pub popup: Option<popups::PopupNotice>,
    pub popup_generation: u64,
    pub popup_bounds: Rc<Cell<Option<[f32; 4]>>>,
    pub popup_task: Option<gpui::Task<()>>,
    /// Every decision handed to the engine (testkit evidence).
    #[cfg(feature = "testkit")]
    pub decisions: crate::shared::Shared<Vec<(u64, bool)>>,
}

/// "your camera", "your camera and microphone".
pub(crate) fn wants_line(host: &str, capabilities: &[PermissionCapability]) -> String {
    let camera = capabilities.contains(&PermissionCapability::Camera);
    let microphone = capabilities.contains(&PermissionCapability::Microphone);
    let what = match (camera, microphone) {
        (true, true) => "your camera and microphone",
        (true, false) => "your camera",
        _ => "your microphone",
    };
    format!("{host} wants to use {what}")
}

/// What a remembered policy says about a request: `Some(true)` allows,
/// `Some(false)` refuses, `None` asks.
pub(crate) fn remembered_decision(
    policies: &SitePermissions,
    profile: ProfileId,
    origin: &str,
    capabilities: &[PermissionCapability],
) -> Option<bool> {
    let policy = |capability| policies.policy(profile, origin, capability);
    if capabilities
        .iter()
        .any(|capability| policy(*capability) == SitePolicy::Block)
    {
        return Some(false);
    }
    if !capabilities.is_empty()
        && capabilities
            .iter()
            .all(|capability| policy(*capability) == SitePolicy::Allow)
    {
        return Some(true);
    }
    None
}

impl BbrApp {
    /// Load the remembered decisions (none when the file is missing).
    pub(crate) fn load_site_permissions(&mut self, path: std::path::PathBuf) {
        self.permissions.policies.set(site_permissions::load(&path));
        self.permissions.path = Some(path);
    }

    pub(crate) fn save_site_permissions(&self) {
        if let Some(path) = self.permissions.path.as_ref() {
            let saved = site_permissions::save(path, &self.permissions.policies.read());
            if let Err(error) = saved {
                log::warn!("site permissions not saved: {error}");
            }
        }
    }

    /// Route the D11 engine events (before the controller applies them).
    pub(crate) fn permission_web_event(&mut self, event: &WebEvent, cx: &mut Context<Self>) {
        match event {
            WebEvent::PermissionRequested {
                view,
                request,
                origin,
                capabilities,
            } => self.on_permission_request(*view, *request, origin, capabilities, cx),
            WebEvent::MediaCaptureChanged {
                view,
                camera,
                microphone,
            } => {
                if *camera || *microphone {
                    self.permissions
                        .capture
                        .get()
                        .insert(*view, (*camera, *microphone));
                } else {
                    self.permissions.capture.get().remove(view);
                }
                cx.notify();
            }
            // A navigation to another site ends that view's pending
            // requests, its blocked mark and its pop-up notice.
            WebEvent::UrlChanged(view, url) => {
                let origin = site_permissions::origin_of(url);
                self.publish_popup_views();
                if self.permissions.popup.as_ref().is_some_and(|n| {
                    n.request.source == *view && Some(&n.origin) != origin.as_ref()
                }) {
                    self.dismiss_popup_notice(cx);
                }
                let stale: Vec<u64> = self
                    .permissions
                    .pending
                    .read()
                    .iter()
                    .filter(|p| p.view == *view && Some(&p.origin) != origin.as_ref())
                    .map(|p| p.request)
                    .collect();
                for request in stale {
                    self.answer(request, false);
                }
                if self
                    .permissions
                    .blocked
                    .read()
                    .get(view)
                    .is_some_and(|(blocked, _)| Some(blocked) != origin.as_ref())
                {
                    self.permissions.blocked.get().remove(view);
                }
                self.sync_portal_overlays(cx);
            }
            _ => {}
        }
    }

    fn on_permission_request(
        &mut self,
        view: WebViewId,
        request: u64,
        origin: &str,
        capabilities: &[PermissionCapability],
        cx: &mut Context<Self>,
    ) {
        let Some(profile) = self.controller.read().profile_of_view(view) else {
            self.answer_engine(request, false);
            return;
        };
        let remembered = remembered_decision(
            &self.permissions.policies.read(),
            profile,
            origin,
            capabilities,
        );
        match remembered {
            Some(true) => self.answer_engine(request, true),
            Some(false) => {
                self.answer_engine(request, false);
                self.permissions.blocked.get().insert(
                    view,
                    (
                        origin.to_string(),
                        capabilities.contains(&PermissionCapability::Camera),
                    ),
                );
            }
            None => {
                if self.permissions.pending.read().is_empty() {
                    self.permissions.remember = false;
                }
                self.permissions.pending.get().push_back(PendingRequest {
                    request,
                    view,
                    profile,
                    origin: origin.to_string(),
                    capabilities: capabilities.to_vec(),
                });
            }
        }
        self.sync_portal_overlays(cx);
    }

    fn answer_engine(&mut self, request: u64, allow: bool) {
        self.engine.borrow_mut().decide_permission(request, allow);
        #[cfg(feature = "testkit")]
        self.permissions.decisions.get().push((request, allow));
    }

    /// Answer a pending request (and drop it from the queue).
    fn answer(&mut self, request: u64, allow: bool) {
        self.permissions
            .pending
            .get()
            .retain(|p| p.request != request);
        self.answer_engine(request, allow);
    }

    /// The request the prompt shows now: the first one of the active page
    /// (requests of views that are gone are dropped; the engine denied
    /// them when it destroyed the view).
    pub(crate) fn current_permission_request(&self) -> Option<PendingRequest> {
        let active = self.controller.read().snapshot().active_view?;
        self.permissions
            .pending
            .read()
            .iter()
            .find(|p| p.view == active && self.controller.read().profile_of_view(p.view).is_some())
            .cloned()
    }

    /// The prompt's buttons: Allow or Don't Allow the current request,
    /// remembering the decision for the site when asked to.
    pub(crate) fn decide_current_permission(&mut self, allow: bool, cx: &mut Context<Self>) {
        let Some(current) = self.current_permission_request() else {
            return;
        };
        if self.permissions.remember {
            let policy = if allow {
                SitePolicy::Allow
            } else {
                SitePolicy::Block
            };
            for capability in &current.capabilities {
                self.permissions.policies.get().set(
                    current.profile,
                    &current.origin,
                    *capability,
                    policy,
                );
            }
            self.save_site_permissions();
        }
        self.answer(current.request, allow);
        self.permissions.remember = false;
        self.sync_portal_overlays(cx);
    }

    /// The prompt's "Remember for this site".
    pub(crate) fn toggle_permission_remember(&mut self, cx: &mut Context<Self>) {
        if self.current_permission_request().is_some() {
            self.permissions.remember = !self.permissions.remember;
            self.sync_portal_overlays(cx);
        }
    }

    /// The prompt's model for the Portal (`None` while nothing waits for
    /// the active page).
    pub(crate) fn permission_prompt_model(
        &self,
    ) -> Option<bbr_ui::surfaces::permissions::PermissionPromptModel> {
        let current = self.current_permission_request()?;
        let profile = self
            .controller
            .read()
            .snapshot()
            .profiles
            .iter()
            .find(|p| p.id == current.profile)
            .map(|p| p.title.clone())
            .unwrap_or_default();
        Some(bbr_ui::surfaces::permissions::PermissionPromptModel {
            message: wants_line(
                site_permissions::origin_host(&current.origin),
                &current.capabilities,
            )
            .into(),
            profile: format!("Profile · {profile}").into(),
            camera: current.capabilities.contains(&PermissionCapability::Camera),
            microphone: current
                .capabilities
                .contains(&PermissionCapability::Microphone),
            remember: self.permissions.remember,
            queued: self.permissions.pending.read().len() - 1,
        })
    }

    /// The Portal hit regions of the prompt and the popover (measured).
    pub(crate) fn permission_hit_rects(&self) -> Vec<[f32; 4]> {
        let mut rects = Vec::new();
        if self.popup_notice_visible().is_some() {
            rects.extend(self.permissions.popup_bounds.get());
        }
        // Blueprint D13: the open-in-app sheet blocks the card; the no-app
        // banner captures its own box.
        if self.open_app_sheet_open() {
            rects.extend(self.transient.card);
        }
        if self.open_app.banner.is_some() {
            rects.extend(self.open_app.banner_bounds.get());
        }
        // Blueprint D13: the space chooser takes the card while it asks.
        if self.space_chooser.is_some() {
            rects.extend(self.transient.card);
        }
        // Blueprint D12: the certificate sheet's veil blocks the card.
        if self.certificate_sheet_open() {
            rects.extend(self.transient.card);
        }
        if self.permission_prompt_model().is_some() {
            rects.extend(self.permissions.prompt_bounds.get());
        }
        if self.site_settings_open() {
            rects.extend(self.permissions.popover_bounds.get());
        }
        rects
    }

    /// Where the prompt and the popover hang: under the address control,
    /// or at the card's top-left edge while the rail is hidden (window
    /// points).
    pub(crate) fn permission_anchor(&self) -> Option<(f32, f32)> {
        if self.sidebar_visible || self.rail_floating() {
            if let Some(rect) = *self.command_field_rect.borrow() {
                return Some((rect.x, rect.y + rect.height + 6.0));
            }
        }
        self.transient.card.map(|[x, y, _, _]| (x + 12.0, y + 12.0))
    }

    /// The address's leading slot for the active page.
    pub(crate) fn address_site(&self, url: &str) -> bbr_ui::components::address::AddressSite {
        use bbr_ui::components::address::{AddressSite, SiteIndicator};
        let origin = site_permissions::origin_of(url);
        let view = self.controller.read().snapshot().active_view;
        let indicator = view.and_then(|view| {
            if let Some((camera, _)) = self.permissions.capture.read().get(&view) {
                return Some(SiteIndicator::Live { camera: *camera });
            }
            self.permissions
                .blocked
                .read()
                .get(&view)
                .filter(|(blocked, _)| Some(blocked) == origin.as_ref())
                .map(|(_, camera)| SiteIndicator::Blocked { camera: *camera })
        });
        AddressSite {
            indicator,
            interactive: origin.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE: &str = "https://meet.example.com";

    #[test]
    fn remembered_policies_decide_or_ask() {
        let mut policies = SitePermissions::default();
        let both = [
            PermissionCapability::Camera,
            PermissionCapability::Microphone,
        ];
        let p = ProfileId(1);
        assert_eq!(remembered_decision(&policies, p, SITE, &both), None);
        policies.set(p, SITE, PermissionCapability::Camera, SitePolicy::Allow);
        assert_eq!(
            remembered_decision(&policies, p, SITE, &both),
            None,
            "microphone asks"
        );
        assert_eq!(
            remembered_decision(&policies, p, SITE, &[PermissionCapability::Camera]),
            Some(true)
        );
        policies.set(p, SITE, PermissionCapability::Microphone, SitePolicy::Block);
        assert_eq!(
            remembered_decision(&policies, p, SITE, &both),
            Some(false),
            "a block wins"
        );
        assert_eq!(
            remembered_decision(&policies, ProfileId(2), SITE, &both),
            None
        );
    }

    #[test]
    fn the_prompt_names_what_the_site_wants() {
        assert_eq!(
            wants_line(
                "meet.example.com",
                &[
                    PermissionCapability::Camera,
                    PermissionCapability::Microphone
                ]
            ),
            "meet.example.com wants to use your camera and microphone"
        );
        assert_eq!(
            wants_line("a.example", &[PermissionCapability::Microphone]),
            "a.example wants to use your microphone"
        );
    }
}
