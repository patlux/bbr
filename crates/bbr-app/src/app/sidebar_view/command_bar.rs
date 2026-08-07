//! Sidebar command-field renderer and U1 field-anchor entry seam.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful,
};

use crate::components;
use crate::theme::{Spacing, Theme};

use super::super::BbrApp;

impl BbrApp {
    /// The command bar component (mirrors the live URL / staged destination).
    /// While a move is staged (`Cmd-Shift-M`), the §11.3 destination hint
    /// renders after the text so the destination profile/space is visible
    /// before Enter commits — the cross-context identity the interaction
    /// contract requires. After a cross-profile space activation, the §11.3
    /// cross-context transient renders the destination profile chip at the
    /// top (leading edge) of the command field for one interaction: the §12
    /// chip plus a `type:label` "in Profile · <title>" hint. The chip data
    /// derives from the active profile — the hint is only ever set to the
    /// active profile and any context mutation clears it, so the invariant
    /// holds by construction.
    pub(super) fn command_bar(
        &self,
        theme: &Theme,
        chrome_focused: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let hint = self.command_bar_hint();
        let profile_hint = self.command_profile_hint.clone();
        // The §11.4 field anchor: an invisible element inside the field that
        // records the field's window-relative rect every frame, so the
        // field-anchored overlay (Cmd-L / field click) can be placed flush
        // with the field without duplicating the chrome layout math.
        let field_anchor =
            crate::app::palette::command_field_anchor(self.command_field_rect.clone());
        // The §12 chip data derives from the active profile: the hint is only
        // ever set to the active profile and any context mutation clears it,
        // so the invariant holds by construction.
        let profile_chip = if profile_hint.is_some() {
            crate::shell::space::active_profile_chip(&self.controller.snapshot())
        } else {
            None
        };
        div()
            .id("bbr-command-bar")
            // The relative box is the containing block for the absolute
            // field anchor (§11.4 overlay anchor).
            .relative()
            .mx(Spacing::sp3())
            .my(Spacing::sp2())
            .px(Spacing::sp3())
            .py(Spacing::sp2())
            .rounded(theme.radius.large())
            .bg(if chrome_focused {
                p.surface_field
            } else {
                p.surface_sidebar
            })
            // §11.4 entry routing: a pointer click on the command field
            // opens the field-anchored overlay in Location mode with the
            // full URL selected (the production pointer entry, same engine
            // as Cmd-L). Propagation stops so the sidebar's click handler
            // (which cancels the command surface) never runs after it.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                    this.open_overlay(window, cx);
                    cx.stop_propagation();
                }),
            )
            // §9 ring: the focused command field gets the accent outline at
            // ring thickness (2 pt), never color-only — the §10 text step
            // brightens the field's own content.
            .border_2()
            .border_color(if chrome_focused { p.accent } else { p.hairline })
            .flex()
            .flex_row()
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
            // The §11.4 field anchor overlays the bar (absolute, inset-0):
            // invisible, takes no layout space, records the field rect.
            .child(field_anchor)
    }
}
