//! Blueprint D11 (board M-8): site settings for the active page — Allow /
//! Ask / Block per permission (applied at once and remembered per site and
//! profile), the zoom with Reset, Clear Site Data… and All Site Settings.

use bbr_ui::surfaces::permissions::{SiteSettingsEvent, SiteSettingsModel, SiteSettingsRow};
use bbr_ui::surfaces::settings::SettingsPane;
use gpui::{Context, SharedString};

use super::SiteSettingsState;
use crate::app::BbrApp;
use crate::site_permissions::{self, capability_key, capability_label, SitePolicy, DECIDABLE};

impl BbrApp {
    /// The active page's origin and profile (`None` off http(s)).
    fn active_site(&self) -> Option<(String, bbr_web::ProfileId)> {
        let origin = site_permissions::origin_of(&self.active_url())?;
        Some((origin, self.controller.active_profile()))
    }

    /// Whether the popover shows (open, and still the active page's site).
    pub(crate) fn site_settings_open(&self) -> bool {
        match (&self.permissions.site_settings, self.active_site()) {
            (Some(state), Some((origin, profile))) => {
                state.origin == origin && state.profile == profile
            }
            _ => false,
        }
    }

    /// The address's leading slot (and the ⌘T action): open site settings
    /// for the active page, or close them.
    pub(crate) fn toggle_site_settings(&mut self, cx: &mut Context<Self>) {
        if self.site_settings_open() {
            self.close_site_settings(cx);
            return;
        }
        let Some((origin, profile)) = self.active_site() else {
            return;
        };
        self.permissions.site_settings = Some(SiteSettingsState {
            origin,
            profile,
            clear_confirm: false,
        });
        self.sync_portal_overlays(cx);
    }

    pub(crate) fn close_site_settings(&mut self, cx: &mut Context<Self>) {
        if self.permissions.site_settings.take().is_some() {
            self.permissions.popover_bounds.set(None);
            self.sync_portal_overlays(cx);
        }
    }

    /// A press outside the popover (the page, the chrome) closes it.
    pub(crate) fn site_settings_pointer_down(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        if !self.site_settings_open() {
            return;
        }
        let inside = |rect: Option<[f32; 4]>| {
            rect.is_some_and(|[rx, ry, w, h]| x >= rx && x <= rx + w && y >= ry && y <= ry + h)
        };
        if inside(self.permissions.popover_bounds.get()) {
            return;
        }
        // The address itself toggles through its own slot.
        let address = self
            .command_field_rect
            .borrow()
            .map(|r| [r.x, r.y, r.width, r.height]);
        if inside(address) {
            return;
        }
        self.close_site_settings(cx);
    }

    /// The popover's model.
    pub(crate) fn site_settings_model(&self) -> Option<SiteSettingsModel> {
        if !self.site_settings_open() {
            return None;
        }
        let state = self.permissions.site_settings.as_ref()?;
        let url = self.active_url();
        let secure = matches!(
            bbr_ui::text::address_identity(&url),
            bbr_ui::text::AddressIdentity::Site { secure: true, .. }
        );
        let profile = self
            .controller
            .snapshot()
            .profiles
            .iter()
            .find(|p| p.id == state.profile)
            .map(|p| p.title.clone())
            .unwrap_or_default();
        let connection = if secure {
            "Connection is secure"
        } else {
            "Connection is not secure"
        };
        Some(SiteSettingsModel {
            host: SharedString::from(site_permissions::origin_host(&state.origin).to_string()),
            connection: format!("{connection} · Profile {profile}").into(),
            rows: DECIDABLE
                .iter()
                .map(|capability| {
                    let values = SitePolicy::ALL
                        .iter()
                        .map(|policy| SharedString::from(policy.label()))
                        .collect();
                    let policy =
                        self.permissions
                            .policies
                            .policy(state.profile, &state.origin, *capability);
                    SiteSettingsRow {
                        key: capability_key(*capability).into(),
                        label: capability_label(*capability).into(),
                        icon: match capability {
                            bbr_web::permissions::PermissionCapability::Camera => {
                                bbr_ui::icons::Icon::Camera { blocked: false }
                            }
                            _ => bbr_ui::icons::Icon::Microphone { blocked: false },
                        },
                        values,
                        value: SitePolicy::ALL
                            .iter()
                            .position(|p| *p == policy)
                            .unwrap_or(1),
                    }
                })
                // Blueprint D13: pop-ups without a gesture — Allow, or
                // Block with the quiet notice (the default).
                .chain(std::iter::once(SiteSettingsRow {
                    key: "popups".into(),
                    label: "Pop-ups".into(),
                    icon: bbr_ui::icons::Icon::Document,
                    values: vec!["Allow".into(), "Block".into()],
                    value: if self
                        .permissions
                        .policies
                        .popups_allowed(state.profile, &state.origin)
                    {
                        0
                    } else {
                        1
                    },
                }))
                .collect(),
            zoom: bbr_ui::components::address::zoom_accessory(self.address_zoom_level()),
            clear_confirm: state.clear_confirm,
        })
    }

    /// Every popover control.
    pub(crate) fn on_site_settings_event(
        &mut self,
        event: SiteSettingsEvent,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.permissions.site_settings.clone() else {
            return;
        };
        match event {
            SiteSettingsEvent::Set(row, value) if row == DECIDABLE.len() => {
                self.permissions
                    .policies
                    .set_popups(state.profile, &state.origin, value == 0);
                self.save_site_permissions();
                self.publish_popup_views();
            }
            SiteSettingsEvent::Set(row, value) => {
                if let (Some(capability), Some(policy)) =
                    (DECIDABLE.get(row), SitePolicy::ALL.get(value))
                {
                    self.permissions.policies.set(
                        state.profile,
                        &state.origin,
                        *capability,
                        *policy,
                    );
                    self.save_site_permissions();
                    // A changed decision ends the quiet blocked mark.
                    if let Some(view) = self.controller.snapshot().active_view {
                        self.permissions.blocked.remove(&view);
                    }
                }
            }
            SiteSettingsEvent::ResetZoom => self.zoom_reset(cx),
            SiteSettingsEvent::Clear => {
                if let Some(open) = self.permissions.site_settings.as_mut() {
                    open.clear_confirm = true;
                }
            }
            SiteSettingsEvent::ClearCancel => {
                if let Some(open) = self.permissions.site_settings.as_mut() {
                    open.clear_confirm = false;
                }
            }
            SiteSettingsEvent::ClearConfirm => {
                let host = site_permissions::origin_host(&state.origin).to_string();
                self.engine
                    .borrow_mut()
                    .clear_site_data(state.profile, &host);
                self.close_site_settings(cx);
                self.show_toast(
                    bbr_ui::icons::Icon::Check,
                    None,
                    bbr_ui::surfaces::toast::ToastTone::Ok,
                    format!("Cleared the data of {host}"),
                    None,
                    cx,
                );
                return;
            }
            SiteSettingsEvent::AllSettings => {
                self.close_site_settings(cx);
                self.settings.pane = SettingsPane::Privacy;
                self.show_settings(cx);
                return;
            }
        }
        self.sync_portal_overlays(cx);
    }
}
