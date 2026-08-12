//! The G2 Group 9 Context-menu story (packet
//! `component-gallery-browser-components.md`, Group 9; enriched by
//! `page-background-context-menu.md`): the exact production
//! `bbr_ui::surfaces::menu::context_menu_card` — the BBR-owned page menu
//! (Back, Forward, Reload, the B2-gated Bookmark Page / Remove Bookmark
//! row, Copy Page Link, DESIGN.md §11.4/§11.13/§13/§15) — renders every
//! valid matrix family from the one matrix authority
//! [`crate::context_menu_matrix`] with recorder callbacks proving which
//! production callback fired (`on_row_hover`, `on_row_press`). Rejected
//! combinations (a zero-item menu, a selection on a disabled row,
//! source-less Tab/Favorite/Folder menus, Gallery-owned WebKit element
//! menus) refuse fixture construction and are shown with their DESIGN
//! reasons. No gallery-local reimplementation of anatomy, no copied
//! tokens, no fake engine/controller/portal.
//!
//! The production card carries **fixed §15 ids** (`bbr-page-menu` and the
//! per-action row ids `bbr-page-menu-*`) that repeat per matrix family, so
//! the gallery wraps each family in a `debug_selector` container — test
//! targets only, never product ids — for the real-pointer E2E (the same
//! pattern the Address stories use).

use std::rc::Rc;

use bbr_ui::surfaces::menu::context_menu_card;
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::context_menu_matrix::{
    build_context_menu_fixture, context_menu_case_valid, CONTEXT_MENU_FIXTURE_RECT,
    CONTEXT_MENU_REJECTED, CONTEXT_MENU_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the page-menu families
/// (`context-menu-family-<n>`, where `<n>` is the family's declaration
/// index in [`CONTEXT_MENU_VALID`]). Test targets only — never product
/// ids.
pub const CONTEXT_MENU_FAMILY_SELECTOR: &str = "context-menu-family-";

/// The §11.4 page menu: the production `bbr-ui` `context_menu_card`
/// renders every valid matrix family ([`CONTEXT_MENU_VALID`]) through the
/// presentation model fixture (the five fixed rows with genuine Back/
/// Forward/link enablement, DESIGN.md §11.13/§15) and recorder callbacks
/// (`on_row_hover`, `on_row_press`). Each family sits in its own
/// test-target container (`.relative()` containing block) so the card's
/// absolute placement at the fixture rect never overlaps another family;
/// the rejected combinations render as value rows with their reasons.
pub fn render_context_menu(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let menu = ctx
        .context_menu
        .as_ref()
        .expect("Context menu story renders with the recorder");
    let on_row_hover = Rc::clone(&menu.observer.on_row_hover);
    let on_row_press = Rc::clone(&menu.observer.on_row_press);

    let families: Vec<AnyElement> = CONTEXT_MENU_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
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
            // The meta line names the family's production-state dimensions.
            let meta = format!(
                "{} · {}",
                if case.focused { "focused" } else { "unfocused" },
                match (case.can_go_back, case.can_go_forward, case.url) {
                    (true, true, Some(_)) => "Back+Forward+URL",
                    (true, false, Some(_)) => "Back+URL",
                    (false, false, Some(_)) => "URL",
                    (true, true, None) => "Back+Forward",
                    (true, false, None) => "Back",
                    _ => "fresh",
                }
            );
            div()
                .debug_selector(move || format!("{CONTEXT_MENU_FAMILY_SELECTOR}{index}"))
                .relative()
                .w(px(260.0))
                .h(px(180.0))
                .child(card)
                .into_any()
                .with_meta_label(theme, case.label, &meta)
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `context_menu_case_valid` refuses every
    // one.
    let rejected: Vec<AnyElement> = CONTEXT_MENU_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !context_menu_case_valid(&r.case),
                "rejected menu {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.4/§11.13 page menu (production bbr_ui::surfaces::menu)"),
        value_row(
            theme,
            "ids",
            "bbr-page-menu · bbr-page-menu-back/-forward/-reload/-bookmark/-copy-link",
        ),
        value_row(
            theme,
            "callbacks",
            "on_row_hover · on_row_press (recorder) — enabled rows only, disabled rows carry no listeners",
        ),
        value_row(
            theme,
            "rows",
            "Back · Forward · Reload · Bookmark Page/Remove Bookmark (B2) · Copy Page Link — unowned capabilities never render",
        ),
        value_row(
            theme,
            "surface",
            "material:overlay · radius:md · elev:2 · §9 focus ring · keyboard/pointer cursor share one selection",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Context menu", children)
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
