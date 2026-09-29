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
                            ["Allow", "Ask", "Block"][row.value.min(2)].to_string(),
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
            prompt,
            site_settings,
            address_mark,
            decisions: self.permissions.decisions.clone(),
        }
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
        }
        cx.notify();
    }
}
