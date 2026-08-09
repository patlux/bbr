//! Sidebar address control (DESIGN.md §11.5) and the non-address command
//! staging surface.
//!
//! The sidebar address is a **display control, not a text field**: at rest
//! (and behind an open palette/overlay presentation) the slot renders the
//! domain pill `bbr-address` — borderless, translucent (token pair driven by
//! theme and space tint), hover wash, button semantics, a reserved trailing
//! accessory slot, and no border or focus ring in any state. All address
//! text entry lives in the §11.4 palette presentations; the pill holds no
//! editing state, cursor or selection and never accepts keyboard text.
//!
//! The same slot renders a separate **command staging surface**
//! (`bbr-command-bar`) only for the non-address staged modes — New Space,
//! Move Tab, and the §11.8 empty New Tab surface — which collect a space
//! name (never an address) and keep their existing field styling, cursor,
//! and hints. The address control itself is never that surface.

use gpui::{prelude::*, Context, Div, MouseDownEvent, SharedString, Stateful};

use bbr_ui::components::{
    address_control, staging_surface, AddressModel, AddressProfileHint, StagingModel,
};
use bbr_ui::text::{address_domain, address_domain_budget_chars, mid_ellipsis};
use bbr_ui::theme::Theme;

use super::super::BbrApp;

impl BbrApp {
    /// The sidebar command slot: the §11.5 address pill while no command
    /// surface is open and no non-address staging mode is active; the
    /// `bbr-command-bar` staging input during New Space / Move Tab / the
    /// empty New Tab surface. While a §11.4 palette/overlay presentation is
    /// open the pill stays rendered underneath it (the overlay covers it in
    /// a separate native portal window; the address control never becomes
    /// an editing surface behind it).
    pub(super) fn command_bar(
        &self,
        theme: &Theme,
        chrome_focused: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let staging = self.command_surface == crate::app::palette::CommandSurface::None
            && self.command_bar_mode.is_active();
        if staging {
            self.staging_surface(theme, chrome_focused, cx)
        } else {
            self.address_control(theme, cx)
        }
    }

    /// The §11.5 address control: a borderless translucent pill carrying the
    /// domain only, with a reserved trailing accessory slot. `type:title`
    /// domain text (`bbr-address-domain`, mid-ellipsis), the slot
    /// (`bbr-address-accessory`) reserving one §14 glyph's `density:hit-min`
    /// width so a later icon changes no layout, hover wash on both chrome
    /// polarities, button semantics (pointer press opens the §11.4 overlay
    /// over the control), and — per §11.5 — **no border and no focus ring
    /// at any time**. The §11.3 cross-context profile hint renders at the
    /// leading edge for its one-interaction lifetime (it follows whichever
    /// command surface is actually visible). The invisible §11.4 field
    /// anchor overlays the pill to record its window-relative rect.
    fn address_control(&self, theme: &Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        // The domain display derives from the live URL (never from the
        // command text model — the pill holds no editing state).
        let domain = address_domain(&self.active_url());
        // The accessory slot is reserved even while empty (§11.5), so the
        // domain budget always accounts for it.
        let budget = address_domain_budget_chars(theme, true);
        let label = mid_ellipsis(&domain, budget);
        // The §11.4 field anchor: an invisible element inside the pill that
        // records the pill's window-relative rect every frame, so the
        // overlay that covers the control can be placed flush with it
        // without duplicating the chrome layout math.
        let field_anchor =
            crate::app::palette::command_field_anchor(self.command_field_rect.clone());
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
        let model = AddressModel {
            domain: SharedString::from(label),
            hovered: self.address_hovered,
            profile_hint,
        };
        let on_hover = cx.listener(|this, hovered: &bool, _window, cx| {
            this.set_address_hover(*hovered, cx);
        });
        let on_open = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.open_overlay(window, cx);
            cx.stop_propagation();
        });
        address_control(theme, &model, field_anchor.into_any(), on_hover, on_open)
    }

    /// The §11.5 command **staging** surface (`bbr-command-bar`): the input
    /// shown only while a non-address staged mode is active without a
    /// floating presentation — New Space, Move Tab, and the §11.8 empty New
    /// Tab surface. It collects a space name (never an address), keeps its
    /// existing field styling, block cursor, and the §11.3 hints, and opens
    /// the §11.4 overlay on pointer press like the legacy field. The address
    /// control above is never this surface; the two are mutually exclusive
    /// by `command_bar_mode`.
    fn staging_surface(
        &self,
        theme: &Theme,
        chrome_focused: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let hint = self.command_bar_hint();
        let profile_hint = self.command_profile_hint.clone();
        let field_anchor =
            crate::app::palette::command_field_anchor(self.command_field_rect.clone());
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
        let model = StagingModel {
            focused: chrome_focused,
            text: self.command_text.text.clone(),
            hint: hint.map(Into::into),
            show_cursor: chrome_focused && self.command_bar_mode.is_active(),
            profile_hint,
        };
        let on_open = cx.listener(|this, _e: &MouseDownEvent, window, cx| {
            this.open_overlay(window, cx);
            cx.stop_propagation();
        });
        staging_surface(theme, &model, field_anchor.into_any(), on_open)
    }
}
