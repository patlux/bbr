//! The Tab/Favorite context-menu story (task packet
//! `tab-favorite-context-menus`): the exact production
//! `bbr_ui::surfaces::tab_menu::tab_menu_card` — the BBR-owned Tab /
//! Favorite menu (DESIGN.md §11.13/§15: placement, page/session,
//! lifecycle, system/share, ordering, close groups; submenu cards for
//! `Move to Folder` / `Move to Space`; danger rows; the §9 focus ring) —
//! renders every valid matrix family from the one matrix authority
//! [`crate::tab_menu_matrix`] with recorder callbacks proving which
//! production callback fired (`on_row_hover`, `on_row_press`,
//! `on_submenu_hover`, `on_submenu_press`). Rejected combinations (an
//! empty menu, a cursor on a disabled row, a submenu on a row without
//! one) refuse fixture construction and are shown with their contract
//! reasons. No gallery-local reimplementation of anatomy, no copied
//! tokens, no fake controller/domain authority.

use std::rc::Rc;

use bbr_ui::surfaces::tab_menu::tab_menu_card;
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};
use crate::tab_menu_matrix::{
    build_tab_menu_fixture, tab_menu_case_valid, TAB_MENU_FIXTURE_RECT,
    TAB_MENU_FIXTURE_SUBMENU_RECT, TAB_MENU_REJECTED, TAB_MENU_VALID,
};

/// The gallery test-target selector prefix for the Tab/Favorite menu
/// families (`tab-menu-family-<n>`). Test targets only — never product
/// ids.
pub const TAB_MENU_FAMILY_SELECTOR: &str = "tab-menu-family-";

/// The Tab/Favorite menu: the production `bbr-ui` `tab_menu_card` renders
/// every valid matrix family ([`TAB_MENU_VALID`]) through the plain
/// presentation model fixture (the deterministic row groups with genuine
/// enablement and the one-level folder/space submenu cards, DESIGN.md
/// §11.13/§15) and recorder callbacks (`on_row_hover`, `on_row_press`,
/// `on_submenu_hover`, `on_submenu_press`). Each family sits in its own
/// test-target container (`.relative()` containing block) so the card's
/// absolute placement never overlaps another family; the rejected
/// combinations render as value rows with their reasons.
pub fn render_tab_menu(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let menu = ctx
        .tab_menu
        .as_ref()
        .expect("Tab / Favorite menu story renders with the recorder");
    let on_row_hover = Rc::clone(&menu.observer.on_row_hover);
    let on_row_press = Rc::clone(&menu.observer.on_row_press);
    let on_submenu_hover = Rc::clone(&menu.observer.on_submenu_hover);
    let on_submenu_press = Rc::clone(&menu.observer.on_submenu_press);

    let families: Vec<AnyElement> = TAB_MENU_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_tab_menu_fixture(case).expect("valid matrix family builds");
            let has_submenu = case.open_submenu.is_some();
            let card = tab_menu_card(
                theme,
                &model,
                TAB_MENU_FIXTURE_RECT,
                has_submenu.then_some(TAB_MENU_FIXTURE_SUBMENU_RECT),
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
                {
                    let on = Rc::clone(&on_submenu_hover);
                    move |row: usize, hovered: &bool, window: &mut Window, cx: &mut App| {
                        on(row, hovered, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_submenu_press);
                    move |row: usize, event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(row, event, window, cx)
                    }
                },
            );
            // The meta line names the family's production-state dimensions.
            let meta = format!(
                "{} · {}",
                case.source_key,
                if has_submenu {
                    format!(
                        "submenu {} open (cursor {})",
                        case.open_submenu.expect("checked above"),
                        case.submenu_selected.expect("checked above")
                    )
                } else {
                    "no submenu".to_string()
                }
            );
            div()
                .debug_selector(move || format!("{TAB_MENU_FAMILY_SELECTOR}{index}"))
                .relative()
                .w(px(480.0))
                .h(px(320.0))
                .child(card)
                .into_any()
                .with_meta_label(theme, case.label, &meta)
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `tab_menu_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = TAB_MENU_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !tab_menu_case_valid(&r.case),
                "rejected menu {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.13 Tab / Favorite menu (production bbr_ui::surfaces::tab_menu)"),
        value_row(
            theme,
            "ids",
            "bbr-tab-menu · bbr-favorite-menu · bbr-tab-menu-<action> · bbr-tab-menu-folder-<n> · bbr-tab-menu-space-<n> · bbr-tab-menu-bookmark · bbr-tab-menu-close-tab",
        ),
        value_row(
            theme,
            "callbacks",
            "on_row_hover · on_row_press · on_submenu_hover · on_submenu_press (recorder) — enabled rows only",
        ),
        value_row(
            theme,
            "groups",
            "placement › · page/session · lifecycle · system/share · ordering · close (danger); unsupported rows omitted",
        ),
        value_row(
            theme,
            "surface",
            "material:overlay · radius:md · elev:2 · §9 focus ring · shared keyboard/pointer cursor · one-level submenu cards (edge-flipped)",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Tab / Favorite menu", children)
}

/// A tiny helper: append the family's meta line below its card container.
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
