//! Blueprint D17 (board L-11, DESIGN §11.8): the startup recovery panes.
//! They take the card's empty state when the last session could not be
//! restored — never a dialog, never touching profile data: the unreadable
//! session was kept aside (Show File in Finder), a newer BBR's data was
//! left unchanged (Continue with a fresh session). Presentation only; the
//! app owns the issue and the commands.

use gpui::{div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, Stateful, Window};

use crate::icons::Icon;
use crate::theme::{Spacing, Theme};

pub const RECOVERY_PANE_ID: &str = "bbr-recovery";
pub const RECOVERY_ACTION_ID: &str = "bbr-recovery-action";

/// Which recovery the pane explains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKind {
    /// The last session could not be read; it was kept aside (`kept`).
    Unreadable { kept: bool },
    /// The session was saved by a newer BBR and left unchanged.
    Newer,
}

impl RecoveryKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Unreadable { .. } => "Your last session couldn't be opened",
            Self::Newer => "This session was saved by a newer BBR",
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            Self::Unreadable { kept: true } => {
                "BBR started with an empty space. The old session file was kept in case it can be recovered; your profiles and logins are untouched."
            }
            Self::Unreadable { kept: false } => {
                "BBR started with an empty space. Your profiles and logins are untouched."
            }
            Self::Newer => {
                "Update BBR to open it. Until then BBR runs with a fresh session and leaves the newer data unchanged."
            }
        }
    }

    /// The one action, if any.
    pub fn action(self) -> Option<&'static str> {
        match self {
            Self::Unreadable { kept: true } => Some("Show File in Finder"),
            Self::Unreadable { kept: false } => None,
            Self::Newer => Some("Continue"),
        }
    }
}

/// The recovery pane: the attention mark on its surface, the title, the
/// body and the one secondary action.
pub fn recovery_pane(
    theme: &Theme,
    kind: RecoveryKind,
    on_action: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(RECOVERY_PANE_ID)
        .debug_selector(|| RECOVERY_PANE_ID.to_string())
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(Spacing::sp3())
        .px(Spacing::sp8())
        .bg(p.surface_field)
        .child(
            div()
                .size(px(56.0))
                .rounded(theme.radius.full())
                .flex()
                .items_center()
                .justify_center()
                .bg(p.status_attention_surface)
                .child(Icon::Warning.canvas_with(theme, px(28.0), p.status_attention)),
        )
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(kind.title()),
        )
        .child(
            div()
                .max_w(px(420.0))
                .text_center()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(kind.body()),
        )
        .children(kind.action().map(|label| {
            div()
                .id(RECOVERY_ACTION_ID)
                .mt(Spacing::sp2())
                .px(Spacing::sp4())
                .py(Spacing::sp1())
                .rounded(theme.radius.medium())
                .border_1()
                .border_color(p.border_strong)
                .text_size(theme.type_.label.size)
                .text_color(p.ink)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, on_action)
                .child(label)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_never_promises_a_recovery_that_did_not_happen() {
        assert_eq!(
            RecoveryKind::Unreadable { kept: true }.action(),
            Some("Show File in Finder")
        );
        assert!(RecoveryKind::Unreadable { kept: true }
            .body()
            .contains("kept"));
        assert!(!RecoveryKind::Unreadable { kept: false }
            .body()
            .contains("kept"));
        assert_eq!(RecoveryKind::Unreadable { kept: false }.action(), None);
        assert_eq!(RecoveryKind::Newer.action(), Some("Continue"));
        assert!(RecoveryKind::Newer.body().contains("unchanged"));
    }
}
