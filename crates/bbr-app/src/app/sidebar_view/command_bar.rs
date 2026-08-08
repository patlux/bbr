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

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful,
};

use crate::command_bar::{
    accessory_slot_width, address_domain, address_domain_budget_chars, mid_ellipsis,
};
use crate::components;
use crate::theme::{Spacing, Theme};

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
        let p = theme.palette;
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
        div()
            .id("bbr-address")
            // The relative box is the containing block for the absolute
            // field anchor (§11.4 overlay anchor).
            .relative()
            .mx(Spacing::sp3())
            .my(Spacing::sp2())
            .px(Spacing::sp3())
            // §11.5: pill height = `density:row` (S 32 / N 36 / XL 44);
            // content centers inside the fixed-height pill.
            .h(theme.density.row())
            .rounded(theme.radius.large())
            // §11.5 fill: translucent token pair (white on dark chrome,
            // black on light chrome) following theme and space tint through
            // the material beneath; the hover wash steps one fill brighter
            // (dark) / darker (light) at `dur:fine`. No border, no ring.
            .bg(if self.address_hovered {
                p.address_fill_hover
            } else {
                p.address_fill
            })
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::sp2())
            .cursor_pointer()
            // §10.2 production hover path: the wash follows the pointer.
            .on_hover(cx.listener(|this, hovered: &bool, _window, cx| {
                this.set_address_hover(*hovered, cx);
            }))
            // §11.5 activation: pointer press opens the §11.4 overlay with
            // the full URL selected (the production pointer entry, same
            // engine as Cmd-L). Propagation stops so the sidebar's click
            // handler (which cancels the command surface) never runs after
            // it.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                    this.open_overlay(window, cx);
                    cx.stop_propagation();
                }),
            )
            .when(profile_chip.is_some(), |el| {
                let (index, spec) = profile_chip.as_ref().expect("checked above");
                let label = profile_hint.clone().expect("profile hint set");
                el.child(
                    div()
                        .id("bbr-command-profile-hint")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(Spacing::sp2())
                        .child(components::profile_chip(theme, *index, spec))
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .font_weight(theme.type_.label.weight)
                                .text_color(p.ink_secondary)
                                .child(label),
                        ),
                )
            })
            .child(
                div()
                    .id("bbr-address-domain")
                    // §11.5: the domain text yields width by ellipsis
                    // (mid-ellipsis at the char budget, GPUI `text_ellipsis`
                    // as the pixel-exact backstop); it never pushes the
                    // reserved accessory out of the pill.
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(theme.type_.title.size)
                    .font_weight(theme.type_.title.weight)
                    .text_color(p.ink)
                    .child(SharedString::from(label)),
            )
            // §11.5 trailing accessory slot: reserved geometry (one §14
            // glyph at `density:hit-min`), `flex-shrink-0`, empty in v1
            // (the §17.14 decision stays open). The slot's width never
            // changes, so later icons cause no layout change.
            .child(
                div()
                    .id("bbr-address-accessory")
                    .w(accessory_slot_width(theme.density))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center(),
            )
            // The §11.4 field anchor overlays the pill (absolute, inset-0):
            // invisible, takes no layout space, records the pill rect.
            .child(field_anchor)
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
        let p = theme.palette;
        let hint = self.command_bar_hint();
        let profile_hint = self.command_profile_hint.clone();
        let field_anchor =
            crate::app::palette::command_field_anchor(self.command_field_rect.clone());
        let profile_chip = if profile_hint.is_some() {
            crate::shell::space::active_profile_chip(&self.controller.snapshot())
        } else {
            None
        };
        div()
            .id("bbr-command-bar")
            .relative()
            .mx(Spacing::sp3())
            .my(Spacing::sp2())
            .px(Spacing::sp3())
            .h(theme.density.row())
            .rounded(theme.radius.large())
            .bg(if chrome_focused {
                p.surface_field
            } else {
                p.surface_sidebar
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                    this.open_overlay(window, cx);
                    cx.stop_propagation();
                }),
            )
            .border_2()
            .border_color(if chrome_focused { p.accent } else { p.hairline })
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::sp2())
            .when(profile_chip.is_some(), |el| {
                let (index, spec) = profile_chip.as_ref().expect("checked above");
                let label = profile_hint.clone().expect("profile hint set");
                el.child(
                    div()
                        .id("bbr-command-profile-hint")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(Spacing::sp2())
                        .child(components::profile_chip(theme, *index, spec))
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .font_weight(theme.type_.label.weight)
                                .text_color(p.ink_secondary)
                                .child(label),
                        ),
                )
            })
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child("▶"),
            )
            .child(
                div()
                    .id("bbr-command-text")
                    .flex_1()
                    .text_size(theme.type_.mono.size)
                    .text_color(p.ink)
                    .child(if self.command_text.is_empty() {
                        SharedString::from("Type a URL…")
                    } else {
                        SharedString::from(self.command_text.text.clone())
                    }),
            )
            .when(chrome_focused && self.command_bar_mode.is_active(), |el| {
                el.child(
                    div()
                        .id("bbr-command-cursor")
                        .w(px(2.0))
                        .h_full()
                        .bg(p.accent),
                )
            })
            .when(hint.is_some(), |el| {
                el.child(
                    div()
                        .id("bbr-command-hint")
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_secondary)
                        .child(hint.expect("hint is some")),
                )
            })
            .child(field_anchor)
    }
}
