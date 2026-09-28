//! The G2 Group 5 Downloads panel story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::surfaces::downloads` surface (`downloads_panel_surface`)
//! renders every valid §11.12 row and panel matrix family from the one
//! matrix authority — the row families in [`crate::downloads_matrix`] and
//! the panel families in [`crate::downloads_panels`] — with recorder
//! callbacks proving which production callback fired. Rejected
//! combinations (a cancelled row without Restart+Delete, Retry off-Failed,
//! backwards progress, `NaN` fraction) refuse fixture construction and are
//! shown with their DESIGN reasons. No gallery-local reimplementation of
//! anatomy, no copied tokens, no fake engine/controller/portal.
//!
//! The production surface carries **fixed §15 ids** (`bbr-downloads-panel`,
//! `bbr-download-row-<n>`, `bbr-downloads-active`, `bbr-downloads-history`,
//! `bbr-downloads-empty`, `bbr-downloads-clear`, …) that repeat per matrix
//! family, so the gallery wraps each family in a `debug_selector`
//! container — test targets only, never product ids — and gives the panel
//! families unique row-index bases so the `bbr-download-row-<n>` ids never
//! collide (the same pattern the Core Controls stories use).

use bbr_ui::surfaces::downloads::{
    downloads_panel_surface, DownloadRowModel, DownloadsCallbacks, DownloadsPanelModel,
};
use gpui::{div, prelude::*, AnyElement, SharedString};

use crate::downloads_matrix::{
    build_download_fixture, download_case_valid, family_layout, CANCELLED_FAMILY_INDEX,
    DOWNLOAD_FAMILY_SELECTOR, DOWNLOAD_REJECTED, DOWNLOAD_VALID, DOWNLOAD_WASH_FAMILY_SELECTOR,
    DOWNLOAD_WASH_VALID, PANEL_WIDTH,
};
use crate::downloads_panels::{
    build_panel_fixture, scroll_window_start, DOWNLOAD_PANEL_FAMILY_SELECTOR, PANEL_VALID,
    SCROLL_FAMILY_ANCHOR, SCROLL_FAMILY_ROWS,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The §11.12 panel model for one family: an open, fully-faded-in panel at
/// the fixture width, borrowing the gallery's scroll/focus handles. The
/// scroll families apply their fixture scroll offset before rendering so
/// the bounded window and the wheel never disagree.
fn family_model<'a>(
    story: &crate::registry::DownloadsStory<'a>,
    rows: Vec<DownloadRowModel>,
    window: std::ops::Range<usize>,
    layout: bbr_ui::surfaces::downloads::PanelLayout,
    focused: bool,
    scroll_offset: gpui::Point<gpui::Pixels>,
) -> DownloadsPanelModel<'a> {
    story.scroll.set_offset(scroll_offset);
    DownloadsPanelModel {
        open: true,
        focused,
        alpha: 1.0,
        width: PANEL_WIDTH,
        rows,
        window,
        layout,
        scroll: story.scroll,
        focus: story.focus,
        check: None,
    }
}

/// §11.12 Downloads panel: the production `bbr-ui` surface renders every
/// valid matrix family — the seven row families, the hover/pressed wash
/// families, and the five panel families (Active+History groups,
/// History-only, empty hint, >6-row bounded scroll window, keyboard cursor
/// focus) — with recorder callbacks (`on_row_hover`, `on_row_press`,
/// `on_row_action`, `on_row_delete`, `on_clear`, `on_inside_press`). The
/// rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_downloads_panel(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let story = ctx
        .downloads
        .as_ref()
        .expect("Downloads panel story renders with the recorder");
    let callbacks = DownloadsCallbacks {
        on_key_down: story.observer.on_key_down.clone(),
        on_inside_press: story.observer.on_inside_press.clone(),
        on_row_action: story.observer.on_row_action.clone(),
        on_row_press: story.observer.on_row_press.clone(),
        on_row_hover: story.observer.on_row_hover.clone(),
        on_row_delete: story.observer.on_row_delete.clone(),
        on_clear: story.observer.on_clear.clone(),
    };

    // The seven row families: each renders as a single-row panel so the
    // machine state, the contextual action, the §10 visual, and the
    // secondary line are all visible through the production surface. The
    // row keeps the family's declaration index (so the §15 ids
    // `bbr-download-row/-action/-delete-<family>` stay distinct) and the
    // render window covers `0..index+1` — the production window/spacer
    // math then lays the single row out at the top of the panel, visible
    // below its group label, instead of pushing it below the fold.
    let families: Vec<AnyElement> = DOWNLOAD_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_download_fixture(case, index).expect("valid matrix family builds");
            let layout = family_layout(theme, case.machine);
            let window = 0..index + 1;
            let panel = downloads_panel_surface(
                theme,
                &family_model(
                    story,
                    vec![model],
                    window,
                    layout,
                    false,
                    gpui::point(gpui::px(0.0), gpui::px(0.0)),
                ),
                &callbacks,
            )
            .expect("open panel renders");
            div()
                .debug_selector(move || format!("{DOWNLOAD_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(panel)
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
                                .child(SharedString::from(format!(
                                    "{:?} · {:?}",
                                    case.machine, case.action
                                ))),
                        ),
                )
                .into_any()
        })
        .collect();

    // The §10.2 hover / §10.3 pressed wash row families. Each family keeps
    // its declaration index (so the §15 ids `bbr-download-row-<n>` stay
    // distinct) and the render window covers `0..index+1` — exactly like
    // the row families above — so the production window/spacer math lays
    // the single row out at the top of the panel, visible below its group
    // label, instead of excluding it from the production filter
    // (`model.window.contains(&row.index)`) and rendering an empty panel.
    let washes: Vec<AnyElement> = DOWNLOAD_WASH_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_download_fixture(case, index).expect("valid wash family builds");
            let layout = family_layout(theme, case.machine);
            let panel = downloads_panel_surface(
                theme,
                &family_model(
                    story,
                    vec![model],
                    0..index + 1,
                    layout,
                    false,
                    gpui::point(gpui::px(0.0), gpui::px(0.0)),
                ),
                &callbacks,
            )
            .expect("open panel renders");
            div()
                .debug_selector(move || format!("{DOWNLOAD_WASH_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(panel)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(case.label)),
                )
                .into_any()
        })
        .collect();

    // The five panel families: multi-row compositions, group labels, the
    // empty hint, the bounded scroll window, and the keyboard cursor
    // focus.
    let panels: Vec<AnyElement> = PANEL_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let fixture = build_panel_fixture(case, theme);
            let window = fixture.window.clone();
            let panel = downloads_panel_surface(
                theme,
                &family_model(
                    story,
                    fixture.rows,
                    window,
                    fixture.layout,
                    case.focused,
                    fixture.scroll_offset,
                ),
                &callbacks,
            )
            .expect("open panel renders");
            let meta = if case.focused {
                "focused (§9 ring + cursor)".to_string()
            } else if case.rows.is_empty() {
                "empty hint".to_string()
            } else if case.rows.len() > SCROLL_FAMILY_ROWS.min(6) {
                format!(
                    "bounded window {}..{} · viewport {} (anchor {})",
                    fixture.window.start,
                    fixture.window.end,
                    fixture.viewport_rows,
                    fixture.window.start
                )
            } else {
                format!(
                    "{} active · {} history",
                    fixture.layout.active_count, fixture.layout.history_count
                )
            };
            div()
                .debug_selector(move || format!("{DOWNLOAD_PANEL_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(panel)
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
    // whole matrix contract, and `download_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = DOWNLOAD_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !download_case_valid(&r.case),
                "rejected download row {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.12 row machine families (production downloads_panel_surface)"),
        value_row(
            theme,
            "ids",
            "bbr-downloads-panel · bbr-download-row/-status/-name/-meta/-progress/-action/-delete-<n>",
        ),
        value_row(
            theme,
            "callbacks",
            "on_row_hover · on_row_press · on_row_action · on_row_delete · on_clear · on_inside_press (recorder)",
        ),
        value_row(
            theme,
            "machine",
            "Waiting · Running · Succeeded · Failed · Cancelled (L6, never invented)",
        ),
    ];
    children.extend(families);
    children.push(section_heading(theme, "§11.12 panel families"));
    children.extend(panels);
    // The wash families render after the panel families so their production
    // rows (`bbr-download-row-<n>`) are the **last painted** instances of
    // the shared id space — the row families and panel families paint the
    // same ids earlier in the page, and the E2E's production-row
    // containment assertion resolves each wash row to its own wrapper only
    // when the wash panel is the last to paint it.
    children.push(section_heading(theme, "§10.2/§10.3 row washes"));
    children.extend(washes);
    children.push(section_heading(theme, "§17.7/§17.8 settled controls"));
    children.push(value_row(
        theme,
        "Clear",
        "bbr-downloads-clear (renders while any history exists)",
    ));
    children.push(value_row(
        theme,
        "Delete",
        "bbr-download-delete-<n> (only on cancelled rows)",
    ));
    children.push(section_heading(
        theme,
        "Stable §15 interaction ids (E2E targets)",
    ));
    children.push(value_row(
        theme,
        "cancelled row (Restart + Delete)",
        &format!("bbr-download-row/-action/-delete-{CANCELLED_FAMILY_INDEX}"),
    ));
    children.push(value_row(
        theme,
        "bounded scroll window",
        &format!(
            "{SCROLL_FAMILY_ROWS} rows · anchor {SCROLL_FAMILY_ANCHOR} · window {}.. (row ids bbr-download-row-<n>)",
            scroll_window_start()
        ),
    ));
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(ctx, "Downloads panel", children)
}
