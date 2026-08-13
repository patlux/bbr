//! The G2 Group 4 Folders story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::folders` symbols — the §11.11 folder rows
//! (`folder_row` with the disclosure/name/count anatomy) and the nested
//! §11.1 member rows (`folder_item_row` with the monogram/title/close
//! anatomy) — render every valid matrix family from the one matrix
//! authority [`crate::folders_matrix`] with recorder callbacks proving
//! which production callback fired (`on_toggle`, `on_select`,
//! `on_close`). Rejected combinations (nested rows in a collapsed folder;
//! persisted folders) refuse fixture construction and are shown with
//! their DESIGN reasons. No gallery-local reimplementation of anatomy, no
//! copied tokens, no fake engine/controller/portal.
//!
//! The full §10 vocabulary on nested rows is a DESIGN state the current
//! `FolderMemberVisual` carries no field for (the module renders the
//! §11.1 monogram/title/close anatomy) — the story documents it rather
//! than inventing families (packet rule: the matrix derives from DESIGN
//! and the current render modules).

use std::rc::Rc;

use bbr_ui::components::folders::folders_ids;
use bbr_ui::components::{folder_item_row, folder_row};
use gpui::{div, prelude::*, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::folders_matrix::{
    build_folders_fixture, folder_case_valid, FOLDERS_REJECTED, FOLDERS_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the folders families
/// (`folders-family-<n>`, where `<n>` is the family's declaration index
/// in [`FOLDERS_VALID`]). Test targets only — never product ids.
pub const FOLDERS_FAMILY_SELECTOR: &str = "folders-family-";

/// §11.11 Folders: the production `bbr-ui` `folder_row` + `folder_item_row`
/// render every valid matrix family — collapsed / expanded, nested rows,
/// the empty collapsible folder, the count `tnum`, and the two-group
/// hairline family — with recorder callbacks (`on_toggle`, `on_select`,
/// `on_close`). Nested rows render only under their expanded folder row.
/// The rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_folders(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let group4 = ctx
        .group4
        .as_ref()
        .expect("Folders story renders with the recorder");

    let families: Vec<AnyElement> = FOLDERS_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let (rows, members) = build_folders_fixture(case).expect("valid matrix family builds");
            let mut children: Vec<AnyElement> = Vec::new();
            for row in &rows {
                let row_index = row.index;
                let toggle = {
                    let on = Rc::clone(&group4.observer.on_folder_toggle);
                    move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(row_index, window, cx)
                    }
                };
                let hover = |_hovered: &bool, _window: &mut Window, _cx: &mut App| {};
                children.push(folder_row(theme, row, hover, toggle).into_any());
                // §11.11: nested rows render only under their expanded
                // folder row (deterministic 0-based member positions).
                for member in members.iter().filter(|m| m.folder_index == row_index) {
                    let target = (member.folder_index, member.member);
                    let select = {
                        let on = Rc::clone(&group4.observer.on_folder_select);
                        move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                            on(target, window, cx)
                        }
                    };
                    let close = {
                        let on = Rc::clone(&group4.observer.on_folder_close);
                        move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                            on(target, window, cx)
                        }
                    };
                    children.push(folder_item_row(theme, member, select, close).into_any());
                }
            }
            let meta = format!(
                "{} folder row(s) · {} nested row(s){}",
                rows.len(),
                members.len(),
                if case.rows.len() > 1 {
                    " · hairlines between groups"
                } else {
                    ""
                }
            );
            div()
                .debug_selector(move || format!("{FOLDERS_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .children(children)
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
    // whole matrix contract, and `folder_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = FOLDERS_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !folder_case_valid(&r.case),
                "rejected folders {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let expanded_family = build_folders_fixture(&FOLDERS_VALID[1]).expect("expanded family");
    let expanded_id_count = folders_ids(&expanded_family.0, &expanded_family.1).len();
    let mut children = vec![
        section_heading(theme, "§11.11 folders (production folder_row + folder_item_row)"),
        value_row(
            theme,
            "ids",
            "bbr-section-folders · bbr-folders · bbr-folder-row/-name/-count/-toggle-<n> · bbr-folder-item-<n>-<m> (+ -title/-favicon/-close)",
        ),
        value_row(
            theme,
            "callbacks",
            "toggle · select · close (recorder) — close stops propagation, §11.11",
        ),
        value_row(
            theme,
            "count",
            "type:meta tnum on every row (§11.11); an empty folder stays a collapsible row",
        ),
        value_row(
            theme,
            "nested §10 vocabulary",
            "default/hover/active/focused + drag-target are DESIGN §10 states the current FolderMemberVisual carries no field for — nested rows render the §11.1 monogram/title/close anatomy (no invented families)",
        ),
        value_row(
            theme,
            "persistence",
            "session-local in v1 — folders are never persisted (§11.11; Phase 7 owns restoration)",
        ),
        value_row(
            theme,
            "id set",
            &format!(
                "folders_ids exposes {expanded_id_count} ids for the expanded family"
            ),
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Folders", children)
}
