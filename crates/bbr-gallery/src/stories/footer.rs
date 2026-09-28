//! The G2 Group 4 Footer story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::footer` `footer_surface` — the §11.4 footer row
//! exactly `[Downloads] [permanent space strip] [+]` — renders every
//! valid matrix family from the one matrix authority
//! [`crate::footer_matrix`] with recorder callbacks proving which
//! production callback fired (`on_downloads`, `on_plus`). The strip shown
//! in context is the **allowed strip** fixture: a presentation-only
//! single-line horizontal scroll row of §12 profile chips (no navigation
//! authority — the real permanent strip composition is G3-owned). Rejected
//! combinations (wrapped/second-line strip; a chip without space+profile
//! accessible names) refuse fixture construction and are shown with their
//! DESIGN reasons. No gallery-local reimplementation of the footer
//! anatomy, no copied tokens, no fake engine/controller/portal.

use std::rc::Rc;

use bbr_ui::components::footer::footer_ids;
use bbr_ui::components::footer_surface;
use gpui::{div, prelude::*, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::footer_matrix::{build_footer_strip, footer_case_valid, FOOTER_REJECTED, FOOTER_VALID};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the footer families
/// (`footer-family-<n>`, where `<n>` is the family's declaration index in
/// [`FOOTER_VALID`]). Test targets only — never product ids.
pub const FOOTER_FAMILY_SELECTOR: &str = "footer-family-";

/// §11.4 Footer: the production `bbr-ui` `footer_surface` renders every
/// valid matrix family — the `[Downloads]` control with/without the
/// running `tnum` count, the allowed strip in context (one chip / many
/// chips / overflow clip, active chip selected surface + outline, inactive
/// chips quiet), and the `[+]` control — with recorder callbacks
/// (`on_downloads`, `on_plus`). The rejected combinations render as value
/// rows with their reasons; the stable §15 ids are shown for the E2E
/// targets.
pub fn render_footer(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let group4 = ctx
        .group4
        .as_ref()
        .expect("Footer story renders with the recorder");
    let on_downloads = Rc::clone(&group4.observer.on_downloads);
    let on_plus = Rc::clone(&group4.observer.on_plus);

    let families: Vec<AnyElement> = FOOTER_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let strip =
                build_footer_strip(theme, case).expect("valid matrix family builds its strip");
            let footer = footer_surface(
                theme,
                group4.downloads_focus.clone(),
                false,
                case.downloads_count,
                strip,
                {
                    let on = Rc::clone(&on_downloads);
                    move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on((), window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_plus);
                    move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on((), window, cx)
                    }
                },
            );
            let meta = match (case.downloads_count, case.chips.len()) {
                (Some(count), chips) => format!("running count {count} · {chips} chip(s)"),
                (None, chips) => format!("no running count · {chips} chip(s)"),
            };
            div()
                .debug_selector(move || format!("{FOOTER_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(footer)
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
    // whole matrix contract, and `footer_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = FOOTER_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !footer_case_valid(&r.case),
                "rejected footer {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.4 footer (production footer_surface)"),
        value_row(
            theme,
            "ids",
            "bbr-footer · bbr-downloads · bbr-space-strip · bbr-plus",
        ),
        value_row(
            theme,
            "callbacks",
            "on_downloads · on_plus (recorder) — the production view listeners",
        ),
        value_row(
            theme,
            "anatomy",
            "exactly [Downloads icon] [permanent strip, flex:1] [Plus icon], hairline above — no [Spaces] toggle",
        ),
        value_row(
            theme,
            "strip fixture",
            "allowed strip in context: §12 profile chips in a single-line horizontal scroll row — presentation-only, no navigation authority (the real permanent strip is G3-owned)",
        ),
        value_row(
            theme,
            "downloads count",
            "type:meta tnum while any active attempt exists (§11.4/§17.11 — in-flight only)",
        ),
        value_row(
            theme,
            "plus name",
            "the glyph-only [ + ] keeps the text alternative (FOOTER_PLUS_NAME)",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    children.push(section_heading(theme, "§15 id derivation"));
    let ids = footer_ids();
    children.push(value_row(
        theme,
        "footer_ids",
        &format!("{} ids — {}", ids.len(), ids.join(" · ")),
    ));
    story_page(ctx, "Footer", children)
}
