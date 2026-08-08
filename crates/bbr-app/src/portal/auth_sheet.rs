//! The conventional macOS credential sheet rendered inside the one native
//! portal window (queue item 10, P8c Auth-Adapter).
//!
//! The sheet is attached to its browser window through the one
//! `NativeAboveWeb` host: a child window ordered above the parent's
//! WKWebViews, exactly like the find bar. It shows the realm and host of
//! the protection space that asked ("who is asking"), a username field, a
//! masked password field, and the two conventional buttons — Cancel and
//! Log In. The buttons route through the production app methods
//! (`auth_submit`/`auth_cancel`) via the portal root's deferred update,
//! and those methods route every decision through the merged portable
//! contract; the sheet UI itself holds no authority over the challenge
//! lifecycle.
//!
//! Unlike the command surfaces, the backdrop deliberately has **no**
//! outside-press dismissal: a credential sheet's only exits are Cancel and
//! Log In (sensitive input). The render consumes the redacted
//! [`crate::app::auth::AuthSheetSurface`]; the raw credential never exists
//! in any serializable shape.

use gpui::{div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, Stateful};

use super::geometry::Rect;
use super::probe::PortalRoot;
use crate::app::auth::{
    AuthField, AuthSheetSurface, AUTH_CANCEL_ID, AUTH_HEADER_ID, AUTH_PASSWORD_ID, AUTH_SHEET_ID,
    AUTH_SUBMIT_ID, AUTH_USERNAME_ID,
};
use crate::theme::Spacing;

/// Build the credential sheet at the resolved portal rect. The caller owns
/// the full-window backdrop and pointer propagation policy (the backdrop
/// stops all presses — no outside dismissal for credentials).
pub fn auth_sheet(
    surface: &AuthSheetSurface,
    rect: Rect,
    cx: &mut Context<PortalRoot>,
) -> Stateful<Div> {
    let theme = crate::theme::Theme::dark();
    let p = theme.palette;
    let authority = match surface.authority.as_str() {
        "proxy" => "proxy",
        _ => "origin server",
    };
    div()
        .id(AUTH_SHEET_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .flex()
        .flex_col()
        .gap(Spacing::sp2())
        .child(header(surface, authority, &theme, &p))
        .child(field(
            AUTH_USERNAME_ID,
            "Username",
            surface.username().to_string(),
            surface,
        ))
        .child(field(
            AUTH_PASSWORD_ID,
            "Password",
            password_mask(surface),
            surface,
        ))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_end()
                .gap(Spacing::sp1())
                .child(button(AUTH_CANCEL_ID, "Cancel", false, cx))
                .child(button(AUTH_SUBMIT_ID, "Log In", true, cx)),
        )
}

/// The "who is asking" header: the title plus the realm and host of the
/// protection space (and whether an origin server or a proxy asked).
fn header(
    surface: &AuthSheetSurface,
    authority: &str,
    theme: &crate::theme::Theme,
    p: &crate::palette::Palette,
) -> Stateful<Div> {
    let (title, detail) = match surface.realm.as_deref() {
        Some(realm) => (
            "Authentication required".to_string(),
            format!("{authority} “{realm}” at {host}", host = surface.host),
        ),
        None => (
            "Authentication required".to_string(),
            format!("{authority} at {host}", host = surface.host),
        ),
    };
    div()
        .id(AUTH_HEADER_ID)
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .text_size(theme.type_.title.size)
                .text_color(p.ink)
                .child(title),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(detail),
        )
}

/// One sheet field: label + the field box. The password field renders
/// bullets; the username field renders the typed text (sheet-local render
/// data — the surface's Debug is redacted and the value never serializes).
fn field(
    id: &'static str,
    label: &'static str,
    value: String,
    surface: &AuthSheetSurface,
) -> Stateful<Div> {
    let theme = crate::theme::Theme::dark();
    let p = theme.palette;
    let focused = (id == AUTH_USERNAME_ID && surface.focused == AuthField::Username)
        || (id == AUTH_PASSWORD_ID && surface.focused == AuthField::Password);
    div()
        .id(format!("{id}-label"))
        .flex()
        .flex_col()
        .gap(Spacing::sp1())
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(label),
        )
        .child(
            div()
                .id(id)
                .h(theme.density.row())
                .px(Spacing::sp2())
                .rounded(theme.radius.medium())
                .border_2()
                .border_color(if focused { p.accent } else { p.hairline })
                .bg(p.surface_field)
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .text_size(theme.type_.mono.size)
                        .text_color(if value.is_empty() { p.ink_faint } else { p.ink })
                        .child(value),
                ),
        )
}

/// The password render: bullets, never the characters (a conventional
/// credential sheet masks the password even inside its own UI).
fn password_mask(surface: &AuthSheetSurface) -> String {
    if !surface.password_entered {
        return String::new();
    }
    // Masked length only; the render never receives the typed value.
    "••••••".to_string()
}

/// One sheet button (Cancel / Log In). Both route through the production
/// app methods via the portal root's deferred update — the exact seams the
/// keyboard paths use. The listeners are inlined per button (the find bar's
/// pattern) so each arm's closure type stays concrete.
fn button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    cx: &mut Context<PortalRoot>,
) -> Stateful<Div> {
    let theme = crate::theme::Theme::dark();
    let p = theme.palette;
    let hit = theme.density.hit_min();
    let mut el = div()
        .id(id)
        .h(hit)
        .px(Spacing::sp2())
        .rounded(theme.radius.medium())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_size(theme.type_.label.size)
        .child(label);
    if primary {
        el = el.bg(p.accent).text_color(p.accent_ink);
    } else {
        el = el.bg(p.surface_field).text_color(p.ink_secondary);
    }
    if id == AUTH_SUBMIT_ID {
        el = el.on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                this.auth_submit_clicked(cx);
            }),
        );
    } else {
        el = el.on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                this.auth_cancel_clicked(cx);
            }),
        );
    }
    el
}
