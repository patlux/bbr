//! Blueprint D08: the transient feedback overlays — the exact production
//! `bbr_ui::surfaces::toast` toast (with and without its action, in the
//! three tones) and the zoom readout. The app renders them in the native
//! Portal's hit-region mode over the card; the gallery shows the surfaces.

use bbr_ui::icons::Icon;
use bbr_ui::surfaces::toast::{toast, zoom_readout, ToastModel, ToastTone};
use gpui::{div, prelude::*, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// The gallery test-target selector prefix for the overlay families.
pub const TOAST_FAMILY_SELECTOR: &str = "toast-family-";

pub fn render_toast(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let toasts = [
        (
            "Saved (Undo)",
            Icon::Bookmark,
            ToastTone::Accent,
            "Saved to Library",
            Some("Undo"),
        ),
        (
            "Done",
            Icon::Check,
            ToastTone::Ok,
            "Cleared 24 hours of data from Default",
            None,
        ),
        (
            "Notice (Reopen)",
            Icon::Clock,
            ToastTone::Neutral,
            "Window with 12 tabs closed",
            Some("Reopen"),
        ),
    ];
    let mut children: Vec<AnyElement> = toasts
        .iter()
        .enumerate()
        .map(|(index, (label, icon, tone, message, action))| {
            let model = ToastModel {
                generation: index as u64,
                icon: *icon,
                profile: None,
                tone: *tone,
                message: (*message).into(),
                action: action.map(Into::into),
                leaving: false,
            };
            div()
                .debug_selector(move || format!("{TOAST_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_2()
                .child(section_heading(theme, label))
                .child(div().flex().child(toast(theme, &model, |_, _, _| {})))
                .into_any()
        })
        .collect();
    children.push(
        div()
            .debug_selector(|| format!("{TOAST_FAMILY_SELECTOR}readout"))
            .flex()
            .flex_col()
            .gap_2()
            .child(section_heading(theme, "Zoom readout"))
            .child(
                div()
                    .flex()
                    .child(zoom_readout(theme, 140, 0, |_, _, _| {}, |_, _, _| {})),
            )
            .into_any(),
    );
    story_page(ctx, "Toast and zoom readout", children)
}
