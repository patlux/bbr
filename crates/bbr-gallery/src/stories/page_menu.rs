//! The G3 Page-menu composition story (packets
//! `component-gallery-compositions.md` and `page-background-context-menu.md`):
//! the enriched BBR page-background menu (Back, Forward, Reload, the
//! B2-gated Bookmark Page / Remove Bookmark row, Copy Page Link) as a
//! composition-in-context coverage of the same production
//! `bbr_ui::surfaces::menu::context_menu_card` the G2 Context-menu story
//! renders. The story renders a curated bookmark-focused subset of the one
//! matrix authority [`crate::context_menu_matrix`] — the B2 gate states
//! (Bookmark Page, Remove Bookmark, omitted for a non-navigable URL) plus
//! the fresh and fully-enabled families — with the same recorder callbacks,
//! so the Gallery never rebuilds menu anatomy and the two stories cannot
//! drift.
//!
//! The G2 story owns the full matrix; this G3 story owns the enriched
//! bookmark composition evidence (StoryId 48, appended after the integrated
//! Space and Tab/Favorite menu stories). Both render the exact production card.

use std::rc::Rc;

use bbr_ui::surfaces::menu::context_menu_card;
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::context_menu_matrix::{
    build_context_menu_fixture, CONTEXT_MENU_FIXTURE_RECT, CONTEXT_MENU_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the page-menu composition
/// families (`page-menu-family-<n>`). Test targets only — never product
/// ids.
pub const PAGE_MENU_FAMILY_SELECTOR: &str = "page-menu-family-";

/// The curated composition families (indexes into [`CONTEXT_MENU_VALID`]):
/// fresh (bookmark omitted — no URL), Bookmark Page, Remove Bookmark, and
/// the fully enabled page menu. Every case is validated by the matrix
/// authority, so the composition can never render a rejected family.
pub const PAGE_MENU_COMPOSITION_FAMILIES: &[usize] = &[0, 3, 4, 5];

/// The enriched page-background menu as a composition: every curated family
/// renders through the exact production `context_menu_card` with the shared
/// recorder callbacks, each in its own test-target container; the meta line
/// names the B2 bookmark state.
pub fn render_page_menu(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let menu = ctx
        .context_menu
        .as_ref()
        .expect("Page menu story renders with the recorder");
    let on_row_hover = Rc::clone(&menu.observer.on_row_hover);
    let on_row_press = Rc::clone(&menu.observer.on_row_press);

    let families: Vec<AnyElement> = PAGE_MENU_COMPOSITION_FAMILIES
        .iter()
        .enumerate()
        .map(|(slot, &case_index)| {
            let case = &CONTEXT_MENU_VALID[case_index];
            let model = build_context_menu_fixture(case).expect("valid matrix family builds");
            let card = context_menu_card(
                theme,
                &model,
                CONTEXT_MENU_FIXTURE_RECT,
                {
                    let on = Rc::clone(&on_row_hover);
                    move |row: usize, hovered: &bool, window: &mut Window, cx: &mut App| {
                        on(row, hovered, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_row_press);
                    move |row: usize, event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(row, event, window, cx)
                    }
                },
            );
            // The meta names the B2 bookmark state and the focus ring.
            let bookmark = match case.bookmark {
                crate::context_menu_matrix::BookmarkFixture::Save => "Bookmark Page",
                crate::context_menu_matrix::BookmarkFixture::Remove => "Remove Bookmark",
                crate::context_menu_matrix::BookmarkFixture::Unavailable => "bookmark omitted",
            };
            let meta = format!(
                "{} · {bookmark}",
                if case.focused { "focused" } else { "unfocused" }
            );
            div()
                .debug_selector(move || format!("{PAGE_MENU_FAMILY_SELECTOR}{slot}"))
                .relative()
                .w(px(260.0))
                .h(px(180.0))
                .child(card)
                .into_any()
                .with_meta_label(theme, case.label, &meta)
        })
        .collect();

    story_page(
        theme,
        "Page menu",
        vec![
            section_heading(theme, "§11.4/§11.13 enriched page-background menu (production card)"),
            value_row(
                theme,
                "rows",
                "Back · Forward · Reload · Bookmark Page/Remove Bookmark (B2) · Copy Page Link",
            ),
            value_row(
                theme,
                "gate",
                "bookmark row from B2 state — omitted for a missing/non-navigable URL; never invented",
            ),
        ]
        .into_iter()
        .chain(families)
        .collect(),
    )
}

/// A tiny helper: append the family's meta line below its card container
/// (the absolute card stays inside the family box; the label flows after
/// it in the story page).
trait FamilyMeta {
    fn with_meta_label(self, theme: &bbr_ui::theme::Theme, label: &str, meta: &str) -> AnyElement;
}

impl FamilyMeta for AnyElement {
    fn with_meta_label(self, theme: &bbr_ui::theme::Theme, label: &str, meta: &str) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(self)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .text_color(theme.palette.ink)
                            .child(SharedString::from(label)),
                    )
                    .child(
                        div()
                            .text_size(theme.type_.meta.size)
                            .text_color(theme.palette.ink_faint)
                            .child(SharedString::from(meta)),
                    ),
            )
            .into_any()
    }
}
