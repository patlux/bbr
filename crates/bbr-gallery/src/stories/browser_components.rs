//! The G2 browser-component stories (packet
//! `component-gallery-browser-components.md`): real `bbr-ui` components
//! with plain fixture models. The first cut is the §11.4 status line —
//! the production `bbr_ui::components::status_line` with fixture-built
//! `StatusModel` text (the §11.8 derivation branches as plain text, never
//! a controller snapshot). The Tab Row cut renders every valid matrix
//! family through the production `bbr_ui::components::tab_row` with
//! recorder callbacks, rejects impossible combinations from the one
//! matrix authority, and embeds the G1b favicon variants inline. No
//! gallery-local reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/portal.

use std::rc::Rc;

use bbr_ui::components::{drag_tick, status_line, tab_row, STATUS_ID};
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, AnyElement, SharedString};

use crate::fixtures::{status_model, STATUS_VARIANTS};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};
use crate::tab_row_matrix::{
    build_tab_row_fixture, tab_row_case_valid, DRAG_TICK_INVALID_ID, DRAG_TICK_INVALID_INDEX,
    DRAG_TICK_LEGAL_ID, DRAG_TICK_LEGAL_INDEX, PINNED_N_PIN_ID, PINNED_S_ROW_ID,
    REVEAL_CLOSE_CLOSE_ID, TAB_ROW_REJECTED, TAB_ROW_VALID,
};

/// §11.4 status line: the production `bbr-ui` component with fixture
/// models — the §11.8 derivation branches (empty, loading, title, URL
/// fallback, crash) as plain text. The stable §15 id rides the rendered
/// component; the story page shows the contract.
pub fn render_status_line(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let rows: Vec<AnyElement> = STATUS_VARIANTS
        .iter()
        .map(|text| {
            let model = status_model(text);
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(status_line(theme, &model))
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(format!("{text:?}")),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "§11.8 derivation branches (fixture text)"),
        value_row(theme, "id", STATUS_ID),
        value_row(theme, "type", "type:meta · ink_faint"),
    ];
    children.extend(rows);
    story_page(theme, "Status line", children)
}

/// §11.1/§11.2/§10/§8 Tab Row: the production `bbr-ui` `tab_row` renders
/// every valid matrix family (packet matrix contract — one declaration
/// source [`TAB_ROW_VALID`]) at its own density with recorder callbacks,
/// the §10.9 drag ticks (legal vs clamped-invalid), and the rejected
/// combinations with their reasons. The G1b favicon variants (monogram /
/// missing / loading-arc) are consumed inline through the row's favicon
/// slot — no separate favicon story.
pub fn render_tab_row(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let tab = ctx
        .tab_row
        .as_ref()
        .expect("Tab Row story renders with the recorder");
    let on_select = Rc::clone(&tab.observer.on_select);
    let on_close = Rc::clone(&tab.observer.on_close);
    let on_pin = Rc::clone(&tab.observer.on_pin);
    let on_begin_drag = Rc::clone(&tab.observer.on_begin_drag);

    let families: Vec<AnyElement> = TAB_ROW_VALID
        .iter()
        .map(|case| {
            // Each family renders at its own §8 density through the
            // production component (the canvas theme stays the caller's).
            let case_theme = Theme::new(theme.mode, theme.contrast, case.density);
            let visuals = build_tab_row_fixture(case).expect("valid matrix family builds");
            let row = tab_row(
                &case_theme,
                &visuals,
                {
                    let on = Rc::clone(&on_select);
                    move |e, w, a| on(e, w, a)
                },
                {
                    let on = Rc::clone(&on_close);
                    move |e, w, a| on(e, w, a)
                },
                {
                    let on = Rc::clone(&on_pin);
                    move |e, w, a| on(e, w, a)
                },
                {
                    let on = Rc::clone(&on_begin_drag);
                    move |e, w, a| on(e, w, a)
                },
            );
            let state = format!("{:?}", case.state).to_lowercase();
            let meta = if case.pinned {
                format!("pinned · {:?}", case.density).to_lowercase()
            } else {
                format!("{:?}", case.density).to_lowercase()
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(row)
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
                                .child(SharedString::from(format!("{state} · {meta}"))),
                        ),
                )
                .into_any()
        })
        .collect();

    // §10.9 drag ticks: the legal accent tick and the clamped-invalid
    // muted hairline tick at stable ids after the row families. The
    // standalone story renders mid-list ticks (`first: false`); the
    // slot-0 top-of-block anchor is covered by the production tabs
    // render and the organize E2E.
    let legal_tick = drag_tick(theme, DRAG_TICK_LEGAL_INDEX, false, false);
    let invalid_tick = drag_tick(theme, DRAG_TICK_INVALID_INDEX, true, false);
    let ticks = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(legal_tick)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(format!(
                            "legal ({DRAG_TICK_LEGAL_ID}) — 2 pt accent rule"
                        ))),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(invalid_tick)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(format!(
                            "clamped-invalid ({DRAG_TICK_INVALID_ID}) — muted hairline"
                        ))),
                ),
        )
        .into_any();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `tab_row_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = TAB_ROW_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !tab_row_case_valid(&r.case),
                "rejected row {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§10 states · §11.1 anatomy (production tab_row)"),
        value_row(
            theme,
            "ids",
            "bbr-tab-row/-title/-favicon/-pin/-state/-close-<n>",
        ),
        value_row(
            theme,
            "callbacks",
            "select · close · pin · begin-drag (recorder)",
        ),
    ];
    children.extend(families);
    children.push(section_heading(theme, "§10.9 drag insertion tick"));
    children.push(ticks);
    children.push(section_heading(
        theme,
        "Stable §15 interaction ids (E2E targets)",
    ));
    children.push(value_row(
        theme,
        "close (selected family)",
        REVEAL_CLOSE_CLOSE_ID,
    ));
    children.push(value_row(theme, "pin (pinned-N family)", PINNED_N_PIN_ID));
    children.push(value_row(theme, "pinned-S icon-only row", PINNED_S_ROW_ID));
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Tab Row", children)
}
