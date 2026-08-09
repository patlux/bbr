//! DESIGN.md §11.4/§11.8 status line — the rail's status row.
//!
//! The status line is a dumb presentation component: it renders the
//! [`StatusModel`] text with the stable §15 id. The text derivation (crash /
//! loading / title / URL fallback / "No page") stays app-owned in
//! `bbr-app::shell::presentation::status_model` — `bbr-ui` never imports the
//! browser controller or a snapshot, and the app never redefines the row
//! anatomy.

use gpui::{div, prelude::*, Div, SharedString, Stateful, Styled};

use crate::theme::{Spacing, Theme};

/// The §15 id of the status line (DESIGN.md §11.4 assert ids).
pub const STATUS_ID: &str = "bbr-status";

/// The status-line presentation model: the plain text the line renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusModel {
    pub text: SharedString,
}

/// The §11.4 status line: `type:meta` text at `ink_faint` with the stable
/// §15 id. The anatomy is byte-identical to the pre-cutover inline div in
/// the app's sidebar composition.
pub fn status_line(theme: &Theme, model: &StatusModel) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(STATUS_ID)
        .px(Spacing::sp4())
        .py(Spacing::sp2())
        .text_size(theme.type_.meta.size)
        .text_color(p.ink_faint)
        .child(model.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_id_is_the_stable_section_15_id() {
        assert_eq!(STATUS_ID, "bbr-status");
    }

    #[test]
    fn status_models_round_trip_their_text() {
        let model = StatusModel {
            text: "Example".into(),
        };
        assert_eq!(model.text, "Example");
        assert_eq!(
            model,
            StatusModel {
                text: "Example".into()
            }
        );
        assert_ne!(
            model,
            StatusModel {
                text: "No page".into()
            }
        );
    }
}
