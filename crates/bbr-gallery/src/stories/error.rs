//! The G2 Group 7 Error/Crash stories (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::surfaces::error` surface and the exact production
//! `bbr_ui::components::pane` panels (crash panel / empty pane) render
//! every valid matrix family from the one matrix authority
//! [`crate::error_matrix`] with recorder callbacks proving which
//! production callback fired (action / dismiss / reload / close /
//! new-tab). Rejected combinations (TLS bypass action, crash panel on a
//! healthy tab, raw platform error codes) refuse fixture construction and
//! are shown with their DESIGN/contract reasons. No gallery-local
//! reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/portal.
//!
//! The production components carry **fixed §15 ids** (`bbr-error-panel`,
//! `bbr-crash-panel`, …) that repeat per matrix family, so the gallery
//! wraps each family in a `debug_selector` container — test targets only,
//! never product ids — for the real-pointer E2E (the same pattern the
//! Address and Library stories use).

use std::rc::Rc;

use bbr_ui::components::{crash_panel, empty_space_pane};
use bbr_ui::surfaces::error::error_surface;
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::error_matrix::{build_error_fixture, error_case_valid, ERROR_REJECTED, ERROR_VALID};
use crate::error_panel_matrix::{
    build_panel_fixture, panel_case_valid, panel_suffix, PanelKind, PANEL_REJECTED, PANEL_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the error families
/// (`error-family-<n>`, where `<n>` is the family's declaration index in
/// [`ERROR_VALID`]). Test targets only — never product ids.
pub const ERROR_FAMILY_SELECTOR: &str = "error-family-";
/// The gallery test-target selector prefix for the pane panels
/// (`panel-crash`, `panel-empty`). Test targets only — never product ids.
pub const PANEL_SELECTOR: &str = "panel-";

/// §11.8 Error surface: the production `bbr-ui` `error_surface` renders
/// every valid matrix family ([`ERROR_VALID`]) — TLS / navigation /
/// offline with the calm title/copy, origin shown vs omitted, and the
/// contract-legal Retry action — with recorder callbacks (`on_action`,
/// `on_dismiss`). The rejected combinations render as value rows with
/// their reasons; the stable §15 ids are shown for the E2E targets.
pub fn render_error_surface(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let error = ctx
        .error
        .as_ref()
        .expect("Error surface story renders with the recorder");
    let on_action = Rc::clone(&error.observer.on_action);
    let on_dismiss = Rc::clone(&error.observer.on_dismiss);

    let families: Vec<AnyElement> = ERROR_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_error_fixture(case).expect("valid matrix family builds");
            let surface = error_surface(
                theme,
                &model,
                {
                    let on = Rc::clone(&on_action);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_dismiss);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
            );
            let meta = if case.origin.is_some() {
                "origin shown"
            } else {
                "origin omitted"
            };
            div()
                .debug_selector(move || format!("{ERROR_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    // The surface is a full-size flex column; a fixed-height
                    // frame keeps each family measurable inside the story
                    // page (the same pattern the Library composition story
                    // uses). The debug target is the Gallery's own E2E
                    // selector — the surface's §15 ids stay production ids.
                    div()
                        .debug_selector(move || format!("{ERROR_FAMILY_SELECTOR}{index}-frame"))
                        .h(px(240.0))
                        .w_full()
                        .child(surface),
                )
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
                                    "{} · {meta}",
                                    case.family.label()
                                ))),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `error_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = ERROR_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !error_case_valid(&r.case),
                "rejected error {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.8 classified error surface (production bbr_ui::surfaces::error)"),
        value_row(
            theme,
            "ids",
            "bbr-error-panel · bbr-error-title · bbr-error-copy · bbr-error-url · bbr-error-action · bbr-error-dismiss",
        ),
        value_row(
            theme,
            "callbacks",
            "action · dismiss (recorder) — no production listener stops propagation",
        ),
        value_row(
            theme,
            "action policy",
            "TLS / Navigation / Offline → Try again (Retry) — the closed contract action set has no bypass",
        ),
        value_row(
            theme,
            "copy",
            "calm, conventional — no raw platform error codes; origin shown vs omitted",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Error surface", children)
}

/// §11.8 Crash / Empty panes: the production `bbr-ui` `crash_panel` and
/// `empty_space_pane` (both `bbr_ui::components::pane`) render the valid
/// pane families ([`PANEL_VALID`]) with recorder callbacks (`on_reload`,
/// `on_close`, `on_new_tab`). The rejected combination (a crash panel on a
/// healthy tab) renders as a value row with its DESIGN reason.
pub fn render_crash_empty_panes(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let error = ctx
        .error
        .as_ref()
        .expect("Crash / Empty panes story renders with the recorder");
    let on_reload = Rc::clone(&error.observer.on_reload);
    let on_close = Rc::clone(&error.observer.on_close);
    let on_new_tab = Rc::clone(&error.observer.on_new_tab);

    let panels: Vec<AnyElement> = PANEL_VALID
        .iter()
        .map(|case| {
            let kind = build_panel_fixture(case).expect("valid panel family builds");
            let panel: AnyElement = match kind {
                PanelKind::Crash => crash_panel(
                    theme,
                    {
                        let on = Rc::clone(&on_reload);
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
                )
                .into_any(),
                PanelKind::Empty => empty_space_pane(theme, case.profile_line, {
                    let on = Rc::clone(&on_new_tab);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                })
                .into_any(),
            };
            let suffix = panel_suffix(kind);
            div()
                .debug_selector(move || format!("{PANEL_SELECTOR}{suffix}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    // The panes are full-size flex columns; a fixed-height
                    // frame keeps each panel measurable inside the story
                    // page (the same pattern the Library composition story
                    // uses).
                    div().h(px(260.0)).w_full().child(panel),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(case.label)),
                )
                .into_any()
        })
        .collect();

    // The rejected combination with its reason — the story shows the whole
    // matrix contract, and `panel_case_valid` refuses it.
    let rejected: Vec<AnyElement> = PANEL_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !panel_case_valid(&r.case),
                "rejected panel {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.8 glass panes (production bbr_ui::components::pane)"),
        value_row(
            theme,
            "ids",
            "bbr-crash-panel · bbr-crash-reload · bbr-crash-close · bbr-empty-new-tab · bbr-empty-new-tab-button",
        ),
        value_row(
            theme,
            "callbacks",
            "reload · close · new-tab (recorder) — no production listener stops propagation",
        ),
        value_row(
            theme,
            "tab states",
            "crashed tab → crash panel · no tabs → empty pane · healthy tab → native slot (no pane)",
        ),
    ];
    children.extend(panels);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Crash / Empty panes", children)
}
