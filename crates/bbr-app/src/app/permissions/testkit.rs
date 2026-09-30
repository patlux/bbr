//! Blueprint D11 testkit seams: the permission state on the wire and one
//! interaction through the method its control calls.

use bbr_testkit::protocol::{
    PermissionInput, PermissionPromptSnapshot, PermissionsSnapshot, SiteSettingsSnapshot,
};
use bbr_ui::components::address::SiteIndicator;
use bbr_ui::surfaces::permissions::SiteSettingsEvent;
use gpui::Context;

use crate::app::BbrApp;
use crate::site_permissions::capability_key;

impl BbrApp {
    pub(crate) fn permissions_snapshot(&self) -> PermissionsSnapshot {
        let prompt = self.current_permission_request().and_then(|current| {
            let model = self.permission_prompt_model()?;
            Some(PermissionPromptSnapshot {
                request: current.request,
                message: model.message.to_string(),
                profile: model.profile.to_string(),
                capabilities: current
                    .capabilities
                    .iter()
                    .map(|c| capability_key(*c).to_string())
                    .collect(),
                remember: model.remember,
                queued: model.queued,
                rect: self.permissions.prompt_bounds.get(),
            })
        });
        let site_settings = self
            .site_settings_model()
            .map(|model| SiteSettingsSnapshot {
                host: model.host.to_string(),
                connection: model.connection.to_string(),
                rows: model
                    .rows
                    .iter()
                    .map(|row| {
                        (
                            row.key.to_string(),
                            row.values
                                .get(row.value)
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
                zoom: model.zoom,
                clear_confirm: model.clear_confirm,
                rect: self.permissions.popover_bounds.get(),
            });
        let address_mark = self
            .address_site(&self.active_url())
            .indicator
            .map(|indicator| {
                match indicator {
                    SiteIndicator::Live { camera: true } => "live-camera",
                    SiteIndicator::Live { camera: false } => "live-microphone",
                    SiteIndicator::Blocked { camera: true } => "blocked-camera",
                    SiteIndicator::Blocked { camera: false } => "blocked-microphone",
                }
                .to_string()
            });
        PermissionsSnapshot {
            popup_notice: self.popup_notice_message(),
            prompt,
            site_settings,
            address_mark,
            decisions: self.permissions.decisions.read().clone(),
        }
    }

    /// The §15 ids of the prompt, the site settings and the certificate
    /// sheet while they show.
    pub(crate) fn permission_visible_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if let Some(model) = self.client_certificate_model() {
            ids.push(bbr_ui::surfaces::client_certificate::CLIENT_CERTIFICATE_ID.to_string());
            ids.extend((0..model.rows.len()).map(|i| format!("bbr-client-certificate-row-{i}")));
            ids.extend(
                ["remember", "none", "cancel", "continue"]
                    .map(|control| format!("bbr-client-certificate-{control}")),
            );
        }
        if self.open_app_sheet_open() {
            ids.extend(
                ["", "-remember", "-cancel", "-confirm"]
                    .map(|part| format!("{}{part}", bbr_ui::surfaces::open_app::OPEN_APP_ID)),
            );
        }
        if self.open_app.banner.is_some() {
            ids.push("bbr-external-no-app".into());
        }
        if let Some(model) = self.space_chooser_model() {
            ids.push(bbr_ui::surfaces::space_chooser::SPACE_CHOOSER_ID.to_string());
            ids.extend((0..model.rows.len()).map(|i| format!("bbr-space-chooser-row-{i}")));
        }
        if self.popup_notice_visible().is_some() {
            ids.extend(
                ["", "-message", "-open", "-allow"].map(|part| format!("bbr-popup-blocked{part}")),
            );
        }
        if self.permission_prompt_model().is_some() {
            ids.push(bbr_ui::surfaces::permissions::PERMISSION_PROMPT_ID.to_string());
            ids.extend(
                ["remember", "deny", "allow"].map(|control| format!("bbr-permission-{control}")),
            );
        } else if let Some(model) = self.site_settings_model() {
            ids.push(bbr_ui::surfaces::permissions::SITE_SETTINGS_ID.to_string());
            for row in &model.rows {
                ids.push(format!("bbr-site-settings-{}", row.key));
                ids.extend(
                    (0..row.values.len())
                        .map(|value| format!("bbr-site-settings-{}-{value}", row.key)),
                );
            }
            if model.zoom.is_some() {
                ids.push("bbr-site-settings-zoom".into());
                ids.push("bbr-site-settings-zoom-reset".into());
            }
            if model.clear_confirm {
                ids.push("bbr-site-settings-clear-cancel".into());
                ids.push("bbr-site-settings-clear-confirm".into());
            } else {
                ids.push("bbr-site-settings-clear".into());
                ids.push("bbr-site-settings-all".into());
            }
        }
        ids
    }

    pub(crate) fn permission_input(&mut self, input: PermissionInput, cx: &mut Context<Self>) {
        match input {
            PermissionInput::Remember => self.toggle_permission_remember(cx),
            PermissionInput::Decide { allow } => self.decide_current_permission(allow, cx),
            PermissionInput::ToggleSiteSettings => self.toggle_site_settings(cx),
            PermissionInput::SetSite { row, value } => {
                self.on_site_settings_event(SiteSettingsEvent::Set(row, value), cx)
            }
            PermissionInput::ResetZoom => {
                self.on_site_settings_event(SiteSettingsEvent::ResetZoom, cx)
            }
            PermissionInput::ClearData => self.on_site_settings_event(SiteSettingsEvent::Clear, cx),
            PermissionInput::ClearCancel => {
                self.on_site_settings_event(SiteSettingsEvent::ClearCancel, cx)
            }
            PermissionInput::ClearConfirm => {
                self.on_site_settings_event(SiteSettingsEvent::ClearConfirm, cx)
            }
            PermissionInput::AllSettings => {
                self.on_site_settings_event(SiteSettingsEvent::AllSettings, cx)
            }
            PermissionInput::PointerDown { x, y } => self.site_settings_pointer_down(x, y, cx),
            // Routed with the window by the dispatcher.
            PermissionInput::PopupOpen | PermissionInput::PopupAlwaysAllow => {}
        }
        cx.notify();
    }
}
