//! The conventional macOS credential sheet presentation (DESIGN.md §11.4
//! command-surface family, §13 dismissal, §15 ids; extracted from
//! `bbr-app`'s portal auth sheet per the G2 Browser Components packet —
//! Group 8).
//!
//! The sheet shows the realm and host of the protection space that asked
//! ("who is asking"), a username field, a masked password field, and the
//! two conventional buttons — Cancel and Log In. It is pure presentation:
//! the caller (the app's portal probe) owns the challenge lifecycle, the
//! redacted surface, the portal geometry (absolute placement, width,
//! height) and the no-outside-dismissal backdrop policy; it builds the
//! [`AuthSheetModel`] DTO and wires the `cx.listener` closures. The
//! render consumes the redacted model; the raw credential never exists in
//! any serializable shape.
//!
//! ## Redaction invariant
//!
//! The model carries the typed username as sheet-local render data only and
//! **no password text at all**: the password render is masked length
//! (`password_entered` → bullets), never the characters. `Debug` is
//! redacted by construction and the type is deliberately never
//! `Serialize`/`Deserialize`, so no log, snapshot, gallery payload,
//! benchmark record, or error text can contain the typed values.
//!
//! ## Theme
//!
//! The surface takes the theme from the model like every other surface
//! (the app resolves the sheet's settled dark presentation when building
//! the DTO; the gallery story proves theme independence by passing the
//! gallery's own resolved theme — the hardcoded `Theme::dark()` of the
//! pre-cutover sheet is never enshrined here).

use std::fmt;

use gpui::{div, prelude::*, App, Div, MouseButton, MouseDownEvent, Stateful, Window};

use crate::palette::Palette;
use crate::theme::{Spacing, Theme};

/// Stable §15 ids of the credential sheet (DESIGN.md §15; exposed through
/// the portal snapshot's typed `ids` field and registered by the app's
/// `PortalProbeState::ids`).
pub const AUTH_SHEET_ID: &str = "bbr-auth-sheet";
pub const AUTH_HEADER_ID: &str = "bbr-auth-header";
pub const AUTH_USERNAME_ID: &str = "bbr-auth-username";
pub const AUTH_PASSWORD_ID: &str = "bbr-auth-password";
pub const AUTH_SUBMIT_ID: &str = "bbr-auth-submit";
pub const AUTH_CANCEL_ID: &str = "bbr-auth-cancel";
/// Blueprint D08 (answer 9): the `wash.active` veil over the content card
/// under the blocking credential sheet.
pub const AUTH_VEIL_ID: &str = "bbr-auth-veil";

/// Which field of the credential sheet owns the keyboard cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthField {
    Username,
    Password,
}

impl AuthField {
    /// Stable snapshot label (never the typed value).
    pub fn label(self) -> &'static str {
        match self {
            AuthField::Username => "username",
            AuthField::Password => "password",
        }
    }
}

/// The credential sheet's presentation model: redacted render data the
/// caller derives from its challenge surface. Carries the typed username
/// for the field render only (`Debug` redacts it; the type is never
/// serialized). The password is represented by the masked
/// `password_entered` flag alone — the render draws bullets, never the
/// characters (module invariant).
pub struct AuthSheetModel {
    /// The resolved theme (every surface takes the theme from its model).
    pub theme: Theme,
    /// The normalized host the challenge belongs to (authority metadata,
    /// never a credential).
    pub host: String,
    /// The realm label, when the challenge carried one.
    pub realm: Option<String>,
    /// `"origin"` or `"proxy"` — who is asking.
    pub authority: String,
    /// Which field owns the keyboard cursor.
    pub focused: AuthField,
    /// The typed username (sheet render only; redacted in `Debug`).
    pub username: String,
    /// Masked: whether the password field holds any characters.
    pub password_entered: bool,
}

impl fmt::Debug for AuthSheetModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The username is sheet-local render data; never in Debug output.
        // The password has no field at all — only the masked flag.
        write!(
            f,
            "AuthSheetModel {{ host: {:?}, realm: {:?}, authority: {:?}, focused: {:?}, username: <redacted>, password_entered: {} }}",
            self.host,
            self.realm,
            self.authority,
            self.focused,
            self.password_entered
        )
    }
}

/// The credential sheet card. The caller owns the portal geometry
/// (absolute placement, width/height) and the backdrop policy — a
/// credential sheet deliberately has **no** outside-press dismissal: its
/// only exits are Log In and Cancel (sensitive input). `on_submit` /
/// `on_cancel` are the production `cx.listener` closures routing through
/// the app's `auth_submit`/`auth_cancel` methods.
pub fn auth_sheet(
    model: &AuthSheetModel,
    on_submit: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let theme = &model.theme;
    let p = &theme.palette;
    let authority = match model.authority.as_str() {
        "proxy" => "proxy",
        _ => "origin server",
    };
    div()
        .id(AUTH_SHEET_ID)
        // Test-support target: the production id as the gallery E2E lookup
        // key (no-op in release builds — the same pattern `tab_row`/
        // `favicon` use so gallery scenarios can click the real controls).
        .debug_selector(|| AUTH_SHEET_ID.to_string())
        // Testkit-only: the drawn sheet rect (`sheet_rects`).
        .children(crate::rect_probe::rect_probe(AUTH_SHEET_ID))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .flex()
        .flex_col()
        .gap(Spacing::sp2())
        .child(header(model, authority, theme, p))
        .child(field(
            theme,
            p,
            AUTH_USERNAME_ID,
            "Username",
            model.username.to_string(),
            model.focused,
        ))
        .child(field(
            theme,
            p,
            AUTH_PASSWORD_ID,
            "Password",
            password_mask(model),
            model.focused,
        ))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_end()
                .gap(Spacing::sp1())
                .child(button(theme, p, AUTH_CANCEL_ID, "Cancel", false, on_cancel))
                .child(button(theme, p, AUTH_SUBMIT_ID, "Log In", true, on_submit)),
        )
}

/// The "who is asking" header: the title plus the realm and host of the
/// protection space (and whether an origin server or a proxy asked).
fn header(model: &AuthSheetModel, authority: &str, theme: &Theme, p: &Palette) -> Stateful<Div> {
    let (title, detail) = match model.realm.as_deref() {
        Some(realm) => (
            "Authentication required".to_string(),
            format!("{authority} “{realm}” at {host}", host = model.host),
        ),
        None => (
            "Authentication required".to_string(),
            format!("{authority} at {host}", host = model.host),
        ),
    };
    div()
        .id(AUTH_HEADER_ID)
        .debug_selector(|| AUTH_HEADER_ID.to_string())
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
/// data — the model's `Debug` is redacted and the value never serializes).
fn field(
    theme: &Theme,
    p: &Palette,
    id: &'static str,
    label: &'static str,
    value: String,
    focused: AuthField,
) -> Stateful<Div> {
    let focused = (id == AUTH_USERNAME_ID && focused == AuthField::Username)
        || (id == AUTH_PASSWORD_ID && focused == AuthField::Password);
    div()
        .id(format!("{id}-label"))
        .debug_selector(move || format!("{id}-label"))
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
            // §9/§18.4: 1 pt hairline at rest, the 2 pt accent ring while
            // focused.
            super::floating::field_edge(div(), theme, focused)
                .id(id)
                .debug_selector(move || id.to_string())
                .h(theme.density.row())
                .px(Spacing::sp2())
                .rounded(theme.radius.medium())
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
fn password_mask(model: &AuthSheetModel) -> String {
    if !model.password_entered {
        return String::new();
    }
    // Masked length only; the model never carries the typed value.
    "••••••".to_string()
}

/// One sheet button (Cancel / Log In). Both route through the production
/// app methods via the caller's listener closures — the exact seams the
/// keyboard paths use.
fn button(
    theme: &Theme,
    p: &Palette,
    id: &'static str,
    label: &'static str,
    primary: bool,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let hit = theme.density.hit_min();
    let mut el = div()
        .id(id)
        .debug_selector(move || id.to_string())
        .h(hit)
        .px(Spacing::sp2())
        .rounded(theme.radius.medium())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_size(theme.type_.label.size)
        .child(label);
    let (fill, text) = button_colors(p, primary);
    el = el.bg(fill).text_color(text);
    el.on_mouse_down(MouseButton::Left, on_click)
}

/// The (fill, text) pair of a sheet button. The primary Log In paints
/// `on_accent` on the accent fill (§3.2); `accent_ink` is text on a neutral
/// surface and fails §3.1 on an accent fill.
fn button_colors(p: &Palette, primary: bool) -> (gpui::Rgba, gpui::Rgba) {
    if primary {
        (p.accent, p.on_accent)
    } else {
        (p.surface_field, p.ink_secondary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The settled production sheet theme (dark; the app resolves it when
    /// building the DTO — the gallery story proves theme independence by
    /// passing its own resolved theme instead).
    fn sheet_theme() -> Theme {
        Theme::dark()
    }

    fn model(
        realm: Option<&str>,
        authority: &str,
        username: &str,
        password_entered: bool,
        focused: AuthField,
    ) -> AuthSheetModel {
        AuthSheetModel {
            theme: sheet_theme(),
            host: "intranet.example".to_string(),
            realm: realm.map(str::to_string),
            authority: authority.to_string(),
            focused,
            username: username.to_string(),
            password_entered,
        }
    }

    #[test]
    fn models_round_trip_their_presentation_fields() {
        let model = model(
            Some("Finance VPN"),
            "origin",
            "alice",
            true,
            AuthField::Password,
        );
        assert_eq!(model.host, "intranet.example");
        assert_eq!(model.realm.as_deref(), Some("Finance VPN"));
        assert_eq!(model.authority, "origin");
        assert_eq!(model.username, "alice");
        assert!(model.password_entered);
        assert_eq!(model.focused, AuthField::Password);
    }

    #[test]
    fn debug_never_renders_the_typed_username() {
        let model = model(
            Some("Finance VPN"),
            "origin",
            "s3cret-user",
            true,
            AuthField::Username,
        );
        let debug = format!("{model:?}");
        assert!(
            !debug.contains("s3cret-user"),
            "Debug must not contain the typed username: {debug}"
        );
        assert!(debug.contains("username: <redacted>"));
        assert!(debug.contains("password_entered: true"));
    }

    #[test]
    fn the_model_carries_no_password_text_at_all() {
        // The redaction invariant is a type shape: the model has no
        // password field — only the masked entered flag. The render draws
        // bullets of masked length, never characters.
        let entered_model = model(None, "proxy", "", true, AuthField::Username);
        assert!(entered_model.password_entered);
        let bullets = password_mask(&entered_model);
        assert_eq!(bullets, "••••••", "masked length only, never the value");
        let empty_model = model(None, "proxy", "", false, AuthField::Username);
        assert!(
            password_mask(&empty_model).is_empty(),
            "empty password renders nothing"
        );
    }

    #[test]
    fn button_labels_meet_the_text_contrast_bar_in_every_palette() {
        // Regression: Log In painted accent.ink on accent (≈ 1.5 : 1).
        for palette in [
            Palette::light_standard(),
            Palette::light_increased(),
            Palette::dark_standard(),
            Palette::dark_increased(),
        ] {
            for primary in [true, false] {
                let (fill, text) = button_colors(&palette, primary);
                let ratio = crate::palette::contrast(text, fill);
                assert!(
                    ratio >= 4.5,
                    "button label (primary={primary}) is {ratio:.2}:1 on its fill"
                );
            }
        }
    }

    #[test]
    fn auth_field_labels_are_stable() {
        assert_eq!(AuthField::Username.label(), "username");
        assert_eq!(AuthField::Password.label(), "password");
    }

    #[test]
    fn ids_are_the_stable_s15_constants() {
        // DESIGN §15 + the portal snapshot's typed `ids` list + the
        // e2e-shell-auth AUTH_IDS support constants — one source.
        assert_eq!(AUTH_SHEET_ID, "bbr-auth-sheet");
        assert_eq!(AUTH_HEADER_ID, "bbr-auth-header");
        assert_eq!(AUTH_USERNAME_ID, "bbr-auth-username");
        assert_eq!(AUTH_PASSWORD_ID, "bbr-auth-password");
        assert_eq!(AUTH_SUBMIT_ID, "bbr-auth-submit");
        assert_eq!(AUTH_CANCEL_ID, "bbr-auth-cancel");
    }
}
