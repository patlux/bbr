//! The G2 Group 4 Plus-menu story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::menu` symbols — `plus_menu_items`,
//! `PlusMenuState`, `plus_menu_ids`, `PLUS_MENU_ID` — render every valid
//! §11.4 plus-menu state family from the one matrix authority
//! [`crate::menu_matrix`] with recorder callbacks proving which production
//! item id a pointer press commits. The menu surface itself (overlay
//! geometry, elevation, focus wiring) is app-owned until the plus-menu
//! behavior step (the `bbr-ui` menu module owns the item identity
//! contract and the app-local state), so this story proves the production
//! item identities, the open/closed/selection state matrix, and the
//! recorder wiring — never a gallery-local copy of the app's overlay.
//! Rejected combinations (a third item; an item while the menu is closed)
//! refuse fixture construction and are shown with their DESIGN reasons.

use std::rc::Rc;

use bbr_ui::components::menu::{plus_menu_ids, plus_menu_items, PLUS_MENU_ID};
use bbr_ui::theme::Spacing;
use gpui::{div, prelude::*, AnyElement, App, MouseButton, MouseDownEvent, SharedString, Window};

use crate::menu_matrix::{
    build_plus_menu_fixture, plus_menu_case_valid, PLUS_MENU_REJECTED, PLUS_MENU_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the plus-menu families
/// (`menu-family-<n>`, where `<n>` is the family's declaration index in
/// [`PLUS_MENU_VALID`]). Test targets only — never product ids.
pub const MENU_FAMILY_SELECTOR: &str = "menu-family-";

/// §11.4 Plus menu: the production `bbr_ui` menu symbols render every
/// valid state family — closed (ids hidden), open with New Space
/// selected, open with New Folder selected — with the two v1 items
/// (production ids/labels/glyphs) and a recorder commit callback proving
/// which production item id a pointer press commits. The keyboard
/// `↑`/`↓`/Enter/Escape state machine rides the production
/// `PlusMenuState` (asserted in the matrix tests; the surface focus
/// wiring is app-owned). The rejected combinations render as value rows
/// with their reasons.
pub fn render_plus_menu(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let group4 = ctx
        .group4
        .as_ref()
        .expect("Plus menu story renders with the recorder");
    let on_commit = Rc::clone(&group4.observer.on_menu_commit);

    let families: Vec<AnyElement> = PLUS_MENU_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let state = build_plus_menu_fixture(case).expect("valid matrix family builds");
            let body: AnyElement = if state.open {
                // The §11.4 menu content: the two production v1 items at
                // their production §15 ids, the keyboard selection
                // highlighted, each press committing through the recorder.
                let items: Vec<AnyElement> = plus_menu_items()
                    .iter()
                    .enumerate()
                    .map(|(item_index, item)| {
                        let is_selected = item_index == state.selection;
                        // `item.id` is `&'static str`; capture the copy so
                        // the commit closure never borrows the temporary
                        // `plus_menu_items()` array.
                        let item_id = item.id;
                        let commit = {
                            let on = Rc::clone(&on_commit);
                            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                on(item_id, window, cx)
                            }
                        };
                        div()
                            .id(item.id)
                            // The production §15 item id is the E2E target
                            // (the tab-row pattern: production ids ride the
                            // debug-selector map).
                            .debug_selector(move || item.id.to_string())
                            .px(Spacing::sp3())
                            .py(Spacing::sp2())
                            .gap(Spacing::sp2())
                            .items_center()
                            .flex()
                            .flex_row()
                            .cursor_pointer()
                            .when(is_selected, |el| {
                                el.bg(theme.palette.tint_accent)
                                    .border_l_2()
                                    .border_color(theme.palette.accent)
                            })
                            .child(
                                div()
                                    .size(theme.density.icon())
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(item.glyph.canvas(theme, theme.density.icon())),
                            )
                            .child(
                                div()
                                    .text_size(theme.type_.label.size)
                                    .font_weight(theme.type_.label.weight)
                                    .text_color(if is_selected {
                                        theme.palette.ink
                                    } else {
                                        theme.palette.ink_secondary
                                    })
                                    .child(item.label),
                            )
                            .on_mouse_down(MouseButton::Left, commit)
                            .into_any()
                    })
                    .collect();
                // The §11.4 menu surface id (the overlay geometry/elevation/
                // focus wiring is app-owned; this story shows the content
                // contract in the gallery).
                div()
                    .id(PLUS_MENU_ID)
                    .debug_selector(|| PLUS_MENU_ID.to_string())
                    .rounded(theme.radius.medium())
                    .bg(theme.palette.surface_overlay)
                    .border_1()
                    .border_color(theme.palette.hairline)
                    .py(Spacing::sp1())
                    .children(items)
                    .into_any()
            } else {
                // §11.4: a closed menu renders no items — the ids expose
                // only while open (`plus_menu_ids` returns nothing).
                value_row(
                    theme,
                    "closed",
                    "ids hidden — plus_menu_ids returns nothing",
                )
            };
            let meta = if state.open {
                format!(
                    "selection {} · {}",
                    state.selection,
                    state.selected_item().label
                )
            } else {
                "no selection".to_string()
            };
            div()
                .debug_selector(move || format!("{MENU_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(body)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(theme.palette.ink)
                                .child(SharedString::from(case.label)),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(theme.palette.ink_faint)
                                .child(SharedString::from(meta)),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `plus_menu_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = PLUS_MENU_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !plus_menu_case_valid(&r.case),
                "rejected plus menu {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let items = plus_menu_items();
    let mut children = vec![
        section_heading(theme, "§11.4 plus menu (production menu symbols)"),
        value_row(
            theme,
            "ids",
            "bbr-plus-menu · bbr-plus-new-space · bbr-plus-new-folder",
        ),
        value_row(
            theme,
            "callbacks",
            "item commit (recorder) — which production item id a press commits",
        ),
        value_row(
            theme,
            "items",
            &format!(
                "exactly {} v1 items: {} (New Space) · {} (New Folder)",
                items.len(),
                items[0].id,
                items[1].id,
            ),
        ),
        value_row(
            theme,
            "keyboard",
            "↑/↓ wrap selection (production PlusMenuState::move_selection) · Enter commits · Escape dismisses without action",
        ),
        value_row(
            theme,
            "surface",
            "overlay geometry/elevation/focus wiring is app-owned until the plus-menu behavior step — this story proves the item identities, the state matrix, and the recorder wiring",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    children.push(section_heading(theme, "§15 id derivation"));
    let open_ids =
        plus_menu_ids(&build_plus_menu_fixture(&PLUS_MENU_VALID[1]).expect("open family"));
    children.push(value_row(
        theme,
        "plus_menu_ids (open)",
        &open_ids.join(" · "),
    ));
    story_page(ctx, "Plus menu", children)
}
