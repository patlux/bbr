//! The G2 Group 8 Auth sheet story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::surfaces::auth` surface — the conventional credential sheet
//! (DESIGN §11.4 command-surface family, §13 dismissal, §15 ids) —
//! renders every valid matrix family from the one matrix authority
//! [`crate::auth_matrix`] with recorder callbacks proving which production
//! callback fired (`on_submit` for Log In, `on_cancel` for Cancel).
//! Rejected combinations (an unmasked password render, outside-press
//! dismissal, a raw/serializable credential shape) refuse fixture
//! construction and are shown with their reasons. No gallery-local
//! reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/portal.
//!
//! The production surface carries **fixed §15 ids** (`bbr-auth-sheet`,
//! `bbr-auth-header`, `bbr-auth-username`, `bbr-auth-password`,
//! `bbr-auth-submit`, `bbr-auth-cancel`) that repeat per matrix family, so
//! the gallery wraps each family in a `debug_selector` container — test
//! targets only, never product ids — for the real-pointer E2E (the same
//! pattern the Address stories use). Each family also registers a
//! `-meta` test-target on its label row so the E2E can measure the sheet's
//! bottom edge without duplicating anatomy math.
//!
//! The surface takes the theme from the model (the gallery resolves the
//! current mode/contrast/density) — the pre-cutover sheet's hardcoded
//! `Theme::dark()` is never enshrined, and the story proves the surface
//! renders in any theme through the canvas mode controls.

use std::rc::Rc;

use bbr_ui::surfaces::auth::{auth_sheet, AuthField};
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::auth_matrix::{auth_case_valid, build_auth_fixture, AUTH_REJECTED, AUTH_VALID};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the credential-sheet
/// families (`auth-family-<n>`, where `<n>` is the family's declaration
/// index in [`AUTH_VALID`]). Test targets only — never product ids.
pub const AUTH_FAMILY_SELECTOR: &str = "auth-family-";

/// The settled production sheet width (`AUTH_SHEET_WIDTH` in the app's
/// auth presentation, §11.4 compact clamp): the story pins each sheet card
/// to this width so the buttons land inside the gallery window (test
/// geometry — the real portal sizes the sheet from its resolved rect).
pub const AUTH_SHEET_WIDTH: f32 = 360.0;

/// §11.4 credential sheet: the production `bbr_ui::surfaces::auth::auth_sheet`
/// renders every valid matrix family ([`AUTH_VALID`]) through the fixture
/// model (the caller-resolved theme rides the model — never dark-only)
/// and recorder callbacks (`on_submit`, `on_cancel`). The rejected
/// combinations render as value rows with their reasons; the stable §15
/// ids are shown for the E2E targets.
pub fn render_auth_sheet(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let auth = ctx
        .auth
        .as_ref()
        .expect("Auth sheet story renders with the recorder");
    let on_submit = Rc::clone(&auth.observer.on_submit);
    let on_cancel = Rc::clone(&auth.observer.on_cancel);

    let families: Vec<AnyElement> = AUTH_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_auth_fixture(case, theme).expect("valid matrix family builds");
            let sheet = auth_sheet(
                &model,
                {
                    let on = Rc::clone(&on_submit);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
                {
                    let on = Rc::clone(&on_cancel);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
            );
            let meta = if case.password_entered {
                "password masked (bullets)"
            } else {
                "password empty"
            };
            let focus = match case.focused {
                AuthField::Username => "focus username",
                AuthField::Password => "focus password",
            };
            div()
                .debug_selector(move || format!("{AUTH_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(div().w(px(AUTH_SHEET_WIDTH)).child(sheet))
                .child(
                    div()
                        .debug_selector(move || format!("{AUTH_FAMILY_SELECTOR}{index}-meta"))
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
                                .child(SharedString::from(format!("{meta} · {focus}"))),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `auth_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = AUTH_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !auth_case_valid(&r.case),
                "rejected auth {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(
            theme,
            "§11.4 credential sheet (production bbr_ui::surfaces::auth)",
        ),
        value_row(
            theme,
            "ids",
            "bbr-auth-sheet · bbr-auth-header · bbr-auth-username · bbr-auth-password · bbr-auth-submit · bbr-auth-cancel",
        ),
        value_row(
            theme,
            "callbacks",
            "on_submit · on_cancel (recorder) — Log In / Cancel route through the production methods",
        ),
        value_row(
            theme,
            "masking",
            "the password renders bullets only; the model carries no password text and is never serialized",
        ),
        value_row(
            theme,
            "dismissal",
            "no outside-press dismissal — the only exits are Log In and Cancel (§13)",
        ),
        value_row(
            theme,
            "theme",
            "the surface takes the theme from the model (canvas mode/contrast/density controls drive the story)",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(theme, "Auth sheet", children)
}
