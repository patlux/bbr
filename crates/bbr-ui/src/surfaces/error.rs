//! DESIGN.md §11.8 error surface — the classified web-error presentation.
//!
//! The surface is a dumb presentation component: it renders the
//! [`ErrorSurfaceModel`] (calm title/copy, the readable failing origin, and
//! the contract-derived action/dismiss labels) with the stable §15 ids.
//! The incident lifecycle, the contract taxonomy mapping, and the recovery
//! action policy stay app-owned in `bbr-app::app::error_surface`
//! (`ErrorSurfaceState` keeps the lifecycle and builds the DTO) — `bbr-ui`
//! never imports the web crate, the controller, or a snapshot, and the app
//! never redefines the panel anatomy.
//!
//! The process-crash class is presented by the DESIGN.md §11.8 crash panel
//! in [`crate::components::pane`]; this surface covers the TLS,
//! navigation, and offline classes.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::icons::Icon;
use crate::theme::{Spacing, Theme};

/// The error-surface presentation model: everything the panel renders,
/// built app-side from the classified incident (the app keeps the contract
/// lifecycle and the action policy). `title`/`copy` are calm and
/// conventional — never a raw platform error code as the primary message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorSurfaceModel {
    /// Calm, conventional title (never a raw platform code).
    pub title: SharedString,
    /// Calm explanation: what failed and what the user can do.
    pub copy: SharedString,
    /// The readable failing origin, when the incident knows one (`None`
    /// omits the URL line rather than guessing).
    pub origin: Option<SharedString>,
    /// The primary action's button label (the contract-legal recovery
    /// action; TLS never offers a bypass).
    pub action_label: SharedString,
    /// The secondary action's button label (Dismiss — a lifecycle event,
    /// never a tab mutation).
    pub dismiss_label: SharedString,
}

/// The §11.8-style error surface: a flat GPUI pane (`material:field`,
/// `elev:1`) drawn in the content area with the 32 pt lifering glyph in
/// `status.error`, the calm title/copy, the readable failing origin, and the
/// contract-derived primary action plus Dismiss. Never a portal host, never
/// composited web content. The anatomy and the `bbr-error-*` ids are
/// byte-identical to the pre-cutover inline div in `bbr-app`.
pub fn error_surface(
    theme: &Theme,
    model: &ErrorSurfaceModel,
    on_action: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id("bbr-error-panel")
        .debug_selector(|| "bbr-error-panel".to_string())
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .bg(theme.palette.surface_field)
        .child(
            div()
                .size(px(32.0))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::Lifering.canvas(theme, px(32.0))),
        )
        .child(
            div()
                .id("bbr-error-title")
                .debug_selector(|| "bbr-error-title".to_string())
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(theme.palette.ink)
                .child(model.title.clone()),
        )
        .child(
            div()
                .id("bbr-error-copy")
                .debug_selector(|| "bbr-error-copy".to_string())
                .max_w(px(460.0))
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child(model.copy.clone()),
        )
        .when(model.origin.is_some(), |surface| {
            surface.child(
                div()
                    .id("bbr-error-url")
                    .debug_selector(|| "bbr-error-url".to_string())
                    .max_w(px(460.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_size(theme.type_.mono.size)
                    .text_color(theme.palette.ink_secondary)
                    .child(model.origin.clone().expect("guarded by when")),
            )
        })
        .child(
            div()
                .flex()
                .flex_row()
                .gap(Spacing::sp3())
                .mt(Spacing::sp2())
                .child(
                    div()
                        .id("bbr-error-action")
                        .debug_selector(|| "bbr-error-action".to_string())
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .bg(theme.palette.accent)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.on_accent)
                        .on_mouse_down(MouseButton::Left, on_action)
                        .child(model.action_label.clone()),
                )
                .child(
                    div()
                        .id("bbr-error-dismiss")
                        .debug_selector(|| "bbr-error-dismiss".to_string())
                        .px(Spacing::sp4())
                        .py(Spacing::sp1())
                        .rounded(theme.radius.medium())
                        .border_1()
                        .border_color(theme.palette.border_strong)
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .on_mouse_down(MouseButton::Left, on_dismiss)
                        .child(model.dismiss_label.clone()),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(origin: Option<&str>) -> ErrorSurfaceModel {
        ErrorSurfaceModel {
            title: "This page can't be loaded".into(),
            copy: "The site's server couldn't be found. Check the address and try again.".into(),
            origin: origin.map(SharedString::from),
            action_label: "Try again".into(),
            dismiss_label: "Dismiss".into(),
        }
    }

    #[test]
    fn error_surface_models_round_trip_their_fields() {
        let with_origin = model(Some("https://example.com"));
        assert_eq!(with_origin.title, "This page can't be loaded");
        assert_eq!(with_origin.origin.as_deref(), Some("https://example.com"));
        assert_eq!(with_origin.action_label, "Try again");
        assert_eq!(with_origin.dismiss_label, "Dismiss");
        assert_eq!(with_origin, model(Some("https://example.com")));
        assert_ne!(with_origin, model(None));
        assert_ne!(with_origin.action_label, with_origin.dismiss_label);
    }

    #[test]
    fn origin_omission_is_explicit() {
        let omitted = model(None);
        assert_eq!(omitted.origin, None);
        let with_origin = model(Some("https://example.com"));
        assert!(with_origin.origin.is_some());
    }

    #[test]
    fn surface_copy_is_calm_and_never_a_raw_platform_code() {
        // The panel renders the model verbatim; the copy contract (calm,
        // no raw platform codes) is pinned here so the DTO cannot silently
        // carry a platform-code shape as the primary message.
        let with_origin = model(Some("https://example.com"));
        for text in [with_origin.title.clone(), with_origin.copy.clone()] {
            assert!(
                !raw_code_like(text.as_ref()),
                "copy is calm, no raw platform code: {text:?}"
            );
        }
    }

    fn raw_code_like(text: &str) -> bool {
        text.contains("NSURLError")
            || text.contains("WKError")
            || text.contains("CFNetwork")
            || text.contains("error -")
            || text.contains("Code ")
    }
}
