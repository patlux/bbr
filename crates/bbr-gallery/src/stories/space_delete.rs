//! Blueprint D05: the Delete Space confirmation sheet story — the exact
//! production `bbr_ui::surfaces::space_delete::space_delete_sheet` in its
//! two keyboard states (Cancel owns the keyboard when the sheet opens;
//! Tab moves it to the danger button), with the counted body copy from the
//! production `delete_body`. The app owns the portal placement, the veil
//! over the content card and the commit; the gallery shows the card only.

use bbr_ui::surfaces::space_delete::{
    delete_body, delete_title, space_delete_sheet, SpaceDeleteButton, SpaceDeleteModel,
};
use gpui::{div, prelude::*, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// The gallery test-target selector prefix for the sheet families.
pub const SPACE_DELETE_FAMILY_SELECTOR: &str = "space-delete-family-";

pub fn render_space_delete(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let families = [
        (
            "Cancel owns the keyboard",
            SpaceDeleteButton::Cancel,
            (14, 2, 1),
        ),
        (
            "Delete Space focused (Tab)",
            SpaceDeleteButton::Delete,
            (1, 0, 0),
        ),
    ];
    let children: Vec<AnyElement> = families
        .iter()
        .enumerate()
        .map(|(index, (label, focused, (tabs, folders, favorites)))| {
            let model = SpaceDeleteModel {
                title: delete_title("Research").into(),
                body: delete_body(*tabs, *folders, *favorites).into(),
                focused: *focused,
            };
            div()
                .debug_selector(move || format!("{SPACE_DELETE_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_2()
                .child(section_heading(theme, label))
                .child(space_delete_sheet(
                    theme,
                    &model,
                    |_, _, _| {},
                    |_, _, _| {},
                ))
                .into_any()
        })
        .collect();
    story_page(ctx, "Delete Space sheet", children)
}
