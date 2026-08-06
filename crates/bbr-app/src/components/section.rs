//! §11.2 section label ("Pinned"/"Tabs"), uppercase `type:badge`; the caller
//! hides the label when its section is empty. `focused` brightens the label
//! one text step (§9: the focused element brightens so focus is never
//! color-only — the §9 ring on the owning list is the structural cue, the
//! label step the text cue).

use gpui::{div, prelude::*, Div, SharedString, Stateful};

use crate::theme::{Spacing, Theme};

pub fn section_label(theme: &Theme, text: &str, focused: bool) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!(
            "bbr-section-{}",
            text.to_lowercase()
        )))
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(if focused {
            theme.palette.ink_secondary
        } else {
            theme.palette.ink_faint
        })
        .child(text.to_uppercase())
}
