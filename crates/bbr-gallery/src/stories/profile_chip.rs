//! The G2 Group 4 Profile chip (S/N/XL) story (packet
//! `component-gallery-browser-components.md`): the exact production
//! `bbr_ui::components::space` `profile_chip` symbol — the §12 identity
//! chip (hue tile + 1–2 character monogram at `bbr-profile-chip-<n>`) —
//! renders every valid matrix family from the one matrix authority
//! [`crate::profile_chip_matrix`]. The chip is static (no callbacks), so
//! this story proves renderability and the §12 identity contract only.
//! Rejected combinations (a chip keyed to a space index; the removed
//! legacy space-row list anatomy) refuse fixture construction and are
//! shown with their DESIGN reasons. No gallery-local reimplementation of
//! anatomy, no copied tokens, no fake engine/controller/portal.

use bbr_ui::components::profile_chip;
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, AnyElement, SharedString};

use crate::profile_chip_matrix::{
    build_profile_chip_fixture, profile_chip_case_valid, PROFILE_CHIP_REJECTED, PROFILE_CHIP_VALID,
};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector prefix for the profile-chip families
/// (`profile-chip-family-<n>`, where `<n>` is the family's declaration
/// index in [`PROFILE_CHIP_VALID`]). Test targets only — never product
/// ids.
pub const PROFILE_CHIP_FAMILY_SELECTOR: &str = "profile-chip-family-";

/// §12 Profile chip: the production `bbr-ui` `profile_chip` renders every
/// valid matrix family — the §12 chip sizes at S/N/XL density (20/24/28
/// pt), the 1- and 2-character monograms, all six §12 hue tiles, and the
/// profile-keyed id `bbr-profile-chip-<profile_index>` — with the
/// deterministic `profile_chip_id` keying (§12: never the space index).
/// The rejected combinations render as value rows with their reasons; the
/// stable §15 ids are shown for the E2E targets.
pub fn render_profile_chip(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;

    let families: Vec<AnyElement> = PROFILE_CHIP_VALID
        .iter()
        .enumerate()
        .map(|(index, case)| {
            // Each family renders at its own §8 density through the
            // production component (the canvas theme stays the caller's).
            let case_theme = Theme::new(theme.mode, theme.contrast, case.density)
                .with_motion_mode(theme.motion_mode);
            let (profile_index, spec) =
                build_profile_chip_fixture(case).expect("valid matrix family builds");
            let chip = profile_chip(&case_theme, profile_index, &spec);
            let chip_size = case_theme.density.profile_chip();
            let meta = format!(
                "{} × {} pt · bbr-profile-chip-{profile_index} · {:?}",
                f32::from(chip_size).round() as u32,
                f32::from(chip_size).round() as u32,
                case.hue,
            );
            div()
                .debug_selector(move || format!("{PROFILE_CHIP_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(chip)
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(theme.palette.ink_faint)
                                .child(SharedString::from(meta)),
                        ),
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
                                .child(SharedString::from(format!("monogram {:?}", case.monogram))),
                        ),
                )
                .into_any()
        })
        .collect();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `profile_chip_case_valid` refuses every
    // one.
    let rejected: Vec<AnyElement> = PROFILE_CHIP_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !profile_chip_case_valid(&r.case),
                "rejected profile chip {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(theme, "§12 profile chip (production profile_chip)"),
        value_row(
            theme,
            "ids",
            "bbr-profile-chip-<profile_index> — never the space index (§12)",
        ),
        value_row(theme, "callbacks", "none — the chip is static (no glow, no animation, no gradient)"),
        value_row(
            theme,
            "anatomy",
            "24 pt square (20 at S, 28 at XL), radius:sm, hue tile + 1–2 character monogram",
        ),
        value_row(
            theme,
            "identity rules",
            "color never alone; profile hue is never used for status, accent, or favicons; at most one identity is loud at a time",
        ),
    ];
    children.extend(families);
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(ctx, "Profile chip (S/N/XL)", children)
}
