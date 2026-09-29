//! The G2 Group 3 Address/Input stories (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::address` symbols — the §11.5 domain pill
//! (`address_control`) and the command staging surface (`staging_surface`)
//! — render every valid matrix family from the one matrix authority
//! [`crate::address_matrix`] with recorder callbacks proving which
//! production callback fired (`on_hover`, `on_open`). Rejected
//! combinations (pill-as-text-field, a focus ring, Copy-Link) refuse
//! fixture construction and are shown with their DESIGN reasons. No
//! gallery-local reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/portal.
//!
//! The production components carry **fixed §15 ids** (`bbr-address`,
//! `bbr-command-bar`, …) that repeat per matrix family, so the gallery
//! wraps each family in a `debug_selector` container — test targets only,
//! never product ids — for the real-pointer E2E (the same pattern the
//! Core Controls stories use).

use std::rc::Rc;

use bbr_ui::components::{address_control, staging_surface};
use bbr_ui::text::address_domain;
use gpui::{div, prelude::*, px, AnyElement, App, MouseDownEvent, SharedString, Window};

use crate::address_matrix::{
    address_case_valid, build_address_fixture, build_staging_fixture, ADDRESS_REJECTED,
    ADDRESS_VALID, STAGING_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the address-pill families
/// (`address-family-<n>`, where `<n>` is the family's declaration index in
/// [`ADDRESS_VALID`]). Test targets only — never product ids.
pub const ADDRESS_FAMILY_SELECTOR: &str = "address-family-";
/// The gallery test-target selector prefix for the staging families
/// (`staging-family-<n>`, where `<n>` is the family's declaration index in
/// [`STAGING_VALID`]). Test targets only — never product ids.
pub const STAGING_FAMILY_SELECTOR: &str = "staging-family-";

/// The gallery's §11.4 field-anchor placeholder: both `address_control`
/// and `staging_surface` take a caller-built anchor element that records
/// the pill's rect for the field-anchored overlay. The gallery has no
/// measured-rect authority (that is `bbr-app`'s `CommandFieldAnchor`), so
/// it passes a plain invisible absolute element that occupies no layout
/// space and paints nothing — the production geometry contract without the
/// overlay measurement.
fn field_anchor_placeholder() -> AnyElement {
    div()
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(0.0))
        .bottom(px(0.0))
        .into_any()
}

/// §11.5 Address control: the production `bbr-ui` `address_control` renders
/// every valid matrix family ([`ADDRESS_VALID`]) through the §11.5 domain
/// derivation (the fixture builds the `AddressModel` with the production
/// `bbr_ui::text` chain) and recorder callbacks (`on_hover`, `on_open`).
/// The rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_address_control(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let address = ctx
        .address
        .as_ref()
        .expect("Address control story renders with the recorder");
    let on_hover = Rc::clone(&address.observer.on_hover);
    let on_open = Rc::clone(&address.observer.on_open);

    let families: Vec<AnyElement> = ADDRESS_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_address_fixture(case, theme).expect("valid matrix family builds");
            let pill = address_control(
                theme,
                &model,
                field_anchor_placeholder(),
                {
                    let on = Rc::clone(&on_hover);
                    move |hovered: &bool, window: &mut Window, cx: &mut App| on(hovered, window, cx)
                },
                {
                    let on = Rc::clone(&on_open);
                    move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                        on(event, window, cx)
                    }
                },
                |_event: &MouseDownEvent, _window: &mut Window, _cx: &mut App| {},
                |_event: &MouseDownEvent, _window: &mut Window, _cx: &mut App| {},
            );
            let derived = address_domain(case.url);
            let meta = if case.hovered {
                "hovered"
            } else if case.profile_hint {
                "profile hint"
            } else {
                "at rest"
            };
            div()
                .debug_selector(move || format!("{ADDRESS_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(pill)
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
                                .child(SharedString::from(format!("{derived} · {meta}"))),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `address_case_valid` refuses every one.
    let rejected: Vec<AnyElement> = ADDRESS_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !address_case_valid(&r.case),
                "rejected pill {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.5 domain pill (production address_control)"),
        value_row(
            theme,
            "ids",
            "bbr-address · bbr-address-domain · bbr-address-accessory",
        ),
        value_row(
            theme,
            "callbacks",
            "on_hover · on_open (recorder) — no editing state, no cursor, no selection",
        ),
        value_row(
            theme,
            "fill",
            "translucent token pair · hover wash · no border, no focus ring",
        ),
    ];
    children.extend(families);
    children.push(section_heading(theme, "§11.3 transient profile hint"));
    children.push(value_row(
        theme,
        "hint chip id",
        "bbr-command-profile-hint (only while the hint is present)",
    ));
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(ctx, "Address control", children)
}

/// §11.5 Command staging: the production `bbr-ui` `staging_surface`
/// renders every valid staging family ([`STAGING_VALID`]) — New Space,
/// Move Tab, and the §11.8 empty New Tab surface — with the block-cursor
/// `show_cursor` true/false, hint present/absent, and profile-hint
/// present/absent dimensions (the profile hint rides the production
/// `StagingModel.profile_hint`/`AddressProfileHint`), wired to the
/// production `on_open` recorder callback. The profile-hint families
/// register the production `bbr-command-profile-hint` semantic id as a
/// test-target key so the focused E2E can assert the story includes it.
pub fn render_command_staging(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let address = ctx
        .address
        .as_ref()
        .expect("Command staging story renders with the recorder");
    let on_open = Rc::clone(&address.observer.on_open);

    let families: Vec<AnyElement> = STAGING_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let model = build_staging_fixture(case);
            let surface = staging_surface(theme, &model, field_anchor_placeholder(), {
                let on = Rc::clone(&on_open);
                move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    on(event, window, cx)
                }
            });
            // The profile-hint families register the production
            // `bbr-command-profile-hint` semantic id as a test-target key
            // (test targets only — never a product id), so the focused E2E
            // can assert the staging story includes the production
            // profile-hint semantic id.
            let surface: AnyElement = if case.profile_hint {
                div()
                    .debug_selector(|| "bbr-command-profile-hint".to_string())
                    .child(surface)
                    .into_any()
            } else {
                surface.into_any()
            };
            let meta = match (model.show_cursor, model.profile_hint.is_some()) {
                (true, true) => "cursor + profile hint",
                (true, false) => "cursor",
                (false, true) => "profile hint",
                (false, false) => "no cursor",
            };
            div()
                .debug_selector(move || format!("{STAGING_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(surface)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(SharedString::from(format!("{} · {meta}", case.label))),
                )
                .into_any()
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§11.5 command staging (production staging_surface)"),
        value_row(
            theme,
            "ids",
            "bbr-command-bar · bbr-command-text · bbr-command-cursor · bbr-command-hint · bbr-command-profile-hint",
        ),
        value_row(
            theme,
            "callbacks",
            "on_open (recorder) — the surface opens the §11.4 overlay on press",
        ),
        value_row(theme, "modes", "New Space · Move Tab · §11.8 empty New Tab"),
        value_row(
            theme,
            "profile hint",
            "bbr-command-profile-hint (only while the hint is present) — §12 chip + label",
        ),
    ];
    children.extend(families);
    story_page(ctx, "Command staging", children)
}
