//! §11.2 section label ("Pinned"/"Tabs"), uppercase `type:badge`; the caller
//! hides the label when its section is empty.

use gpui::{div, prelude::*, Div};

use crate::theme::{Spacing, Theme};

pub fn section_label(theme: &Theme, text: &str) -> Div {
    div()
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(theme.palette.ink_faint)
        .child(text.to_uppercase())
}
