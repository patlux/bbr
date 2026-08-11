//! The G2 Group 6 Find story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::surfaces::find` symbols — the §11.4 find-in-page bar
//! (`find_bar`) — render every valid matrix family from the one matrix
//! authority [`crate::find_matrix`] with recorder callbacks proving which
//! production callback fired (`on_previous`, `on_next`, `on_close`).
//! Rejected combinations (an ordinal without a count; a next enabled at 0
//! matches) refuse fixture construction and are shown with their reasons.
//! No gallery-local reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/portal.
//!
//! The production component carries **fixed §15 ids** (`bbr-find-bar`,
//! `bbr-find-previous`, …) that repeat per matrix family, so the gallery
//! wraps each family in a `debug_selector` container — test targets only,
//! never product ids — for the real-pointer E2E (the same pattern the
//! Core Controls and Address stories use). The gallery has no measured
//! portal-rect authority (that is `bbr-app`'s portal geometry), so each
//! family passes the fixture [`find_family_rect`] — the production
//! geometry contract without the portal measurement.

use std::rc::Rc;

use bbr_ui::surfaces::find::{
    count_label, find_bar, FIND_BAR_ID, FIND_CLOSE_ID, FIND_COUNT_ID, FIND_FIELD_ID, FIND_NEXT_ID,
    FIND_PREVIOUS_ID, FIND_TEXT_ID,
};
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::find_matrix::{
    build_find_fixture, find_case_valid, find_family_rect, FIND_FAMILY_WIDTH, FIND_REJECTED,
    FIND_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the find-bar families
/// (`find-family-<n>`, where `<n>` is the family's declaration index in
/// [`FIND_VALID`]). Test targets only — never product ids.
pub const FIND_FAMILY_SELECTOR: &str = "find-family-";

/// Register the production §15 ids as test-target keys on the first
/// family's bar container (test targets only — never product ids), so the
/// focused E2E can assert the story renders the production ids. The
/// production component carries fixed ids that repeat per matrix family,
/// so one family registers them (the debug-bounds map key is the id;
/// `debug_selector` is a no-op outside test builds).
fn with_find_id_targets(bar_container: AnyElement) -> AnyElement {
    [
        FIND_BAR_ID,
        FIND_FIELD_ID,
        FIND_TEXT_ID,
        FIND_COUNT_ID,
        FIND_PREVIOUS_ID,
        FIND_NEXT_ID,
        FIND_CLOSE_ID,
    ]
    .into_iter()
    .fold(bar_container, |acc, id| {
        div()
            .debug_selector(move || id.to_string())
            .child(acc)
            .into_any()
    })
}

/// §11.4 Find bar: the production `bbr-ui` `find_bar` renders every valid
/// matrix family ([`FIND_VALID`]) through the production `FindSurface`
/// fixture and recorder callbacks (`on_previous`, `on_next`, `on_close`).
/// The rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_find_bar(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let find = ctx
        .find
        .as_ref()
        .expect("Find bar story renders with the recorder");
    let on_previous = Rc::clone(&find.observer.on_previous);
    let on_next = Rc::clone(&find.observer.on_next);
    let on_close = Rc::clone(&find.observer.on_close);

    let families: Vec<AnyElement> = FIND_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_find_fixture(case).expect("valid matrix family builds");
            let rect = find_family_rect(theme);
            let bar = find_bar(
                theme,
                &model,
                rect,
                {
                    let on = Rc::clone(&on_previous);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_next);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_close);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
            );
            let meta = if case.focused {
                if case.query.is_empty() {
                    "focused · placeholder"
                } else {
                    "focused"
                }
            } else {
                "unfocused"
            };
            let count = count_label(&model);
            let count_meta = if count.is_empty() {
                "count — none reported".to_string()
            } else {
                format!("count — {count}")
            };
            // The test-target container is the containing block for the
            // absolutely-positioned production bar; the first family also
            // registers the production §15 ids as test-target keys.
            let bar_container = div()
                .relative()
                .w(px(FIND_FAMILY_WIDTH))
                .h(theme.density.row())
                .child(bar);
            let bar_container: AnyElement = if index == 0 {
                with_find_id_targets(bar_container.into_any())
            } else {
                bar_container.into_any()
            };
            div()
                .debug_selector(move || format!("{FIND_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(bar_container)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(format!(
                            "{} · {meta} · {count_meta}",
                            case.label
                        ))),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `find_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = FIND_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !find_case_valid(&r.case),
                "rejected find family {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.4 find bar (production bbr_ui::surfaces::find)"),
        value_row(
            theme,
            "ids",
            "bbr-find-bar · bbr-find-field · bbr-find-text · bbr-find-count · bbr-find-previous · bbr-find-next · bbr-find-close",
        ),
        value_row(
            theme,
            "callbacks",
            "on_previous · on_next · on_close (recorder) — the controls never stop propagation (the portal root's card-level §13 listener owns the stop)",
        ),
        value_row(
            theme,
            "count",
            "n of m · 0 of 0 · empty while no report (production count_label)",
        ),
        value_row(
            theme,
            "stale",
            "lifecycle state (FindState.stale), no presentation variant: the bar keeps its last report until a fresh query — the gallery never fakes a stale visual",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Find bar", children)
}
