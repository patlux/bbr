//! Sidebar command slot models (DESIGN.md §11.5) and the §11.4 field anchor.
//!
//! The app resolves its `CommandBarMode` authority into the plain
//! [`CommandSlot`] presentation enum the `bbr-ui` rail renders: the §11.5
//! address pill at rest, or the `bbr-command-bar` staging input during the
//! non-address staged modes (New Space, Move Tab, the §11.8 empty New Tab
//! surface). The invisible §11.4 field anchor element (the rect recorder
//! for the field-anchored overlay) is built here and passed at the
//! composition boundary. The app keeps the mode authority, the domain
//! derivation, the controller snapshot, and the profile-hint transient;
//! the rail only renders.

use gpui::{prelude::*, AnyElement, SharedString};

use bbr_ui::components::address::{AddressModel, AddressProfileHint, StagingModel};
use bbr_ui::sidebar::model::CommandSlot;
use bbr_ui::text::{address_domain, address_domain_budget_chars, mid_ellipsis};
use bbr_ui::theme::Theme;

use super::super::BbrApp;

impl BbrApp {
    /// The §11.5 command slot model: the address pill while no command
    /// surface is open and no non-address staging mode is active; the
    /// `bbr-command-bar` staging input during New Space / Move Tab / the
    /// empty New Tab surface. The rail renders the matching `bbr-ui`
    /// component; the app keeps the mode authority.
    pub(super) fn command_slot(&self, theme: &Theme, chrome_focused: bool) -> CommandSlot {
        let staging = self.command_surface == crate::app::palette::CommandSurface::None
            && self.command_bar_mode.is_active();
        if staging {
            CommandSlot::Staging(self.staging_model(chrome_focused))
        } else {
            CommandSlot::Address(self.address_model(theme))
        }
    }

    /// The §11.4 field anchor element: an invisible element inside the
    /// command slot that records the slot's window-relative rect every
    /// frame, so the overlay that covers the control can be placed flush
    /// with it without duplicating the chrome layout math.
    pub(super) fn command_field_anchor_element(&self) -> AnyElement {
        crate::app::palette::command_field_anchor(self.command_field_rect.clone()).into_any()
    }

    /// The §11.5 address pill's presentation model: the domain only (never
    /// the command text model — the pill holds no editing state), the hover
    /// wash, and the §11.3 cross-context profile hint at the leading edge.
    fn address_model(&self, theme: &Theme) -> AddressModel {
        // The domain display derives from the live URL (never from the
        // command text model — the pill holds no editing state).
        let domain = address_domain(&self.active_url());
        // The accessory slot is reserved even while empty (§11.5), so the
        // domain budget always accounts for it.
        let budget = address_domain_budget_chars(theme, true);
        let label = mid_ellipsis(&domain, budget);
        let profile_hint = self.command_profile_hint.clone();
        // The §12 chip data derives from the active profile: the hint is
        // only ever set to the active profile and any context mutation
        // clears it, so the invariant holds by construction.
        let profile_chip = if profile_hint.is_some() {
            crate::shell::space::active_profile_chip(&self.controller.snapshot())
        } else {
            None
        };
        let profile_hint = profile_chip.map(|(index, spec)| AddressProfileHint {
            profile_index: index,
            spec,
            label: profile_hint.expect("profile hint set").clone().into(),
        });
        AddressModel {
            domain: SharedString::from(label),
            hovered: self.address_hovered,
            profile_hint,
            zoom: bbr_ui::components::address::zoom_accessory(self.address_zoom_level()),
        }
    }

    /// The §11.5 command **staging** surface's presentation model
    /// (`bbr-command-bar`): the input shown only while a non-address staged
    /// mode is active without a floating presentation — New Space, Move
    /// Tab, and the §11.8 empty New Tab surface. It collects a space name
    /// (never an address), keeps its existing field styling, block cursor,
    /// and the §11.3 hints. The address control above is never this
    /// surface; the two are mutually exclusive by `command_bar_mode`.
    fn staging_model(&self, chrome_focused: bool) -> StagingModel {
        let hint = self.command_bar_hint();
        let profile_hint = self.command_profile_hint.clone();
        let profile_chip = if profile_hint.is_some() {
            crate::shell::space::active_profile_chip(&self.controller.snapshot())
        } else {
            None
        };
        let profile_hint = profile_chip.map(|(index, spec)| AddressProfileHint {
            profile_index: index,
            spec,
            label: profile_hint.expect("profile hint set").clone().into(),
        });
        // The staging presentation resolves the `CommandBarMode` authority
        // (which stays app-owned with its testkit protocol mapping) into the
        // plain model the bbr-ui surface renders.
        StagingModel {
            focused: chrome_focused,
            text: self.command_text.text.clone(),
            hint: hint.map(Into::into),
            show_cursor: chrome_focused && self.command_bar_mode.is_active(),
            profile_hint,
        }
    }
}
