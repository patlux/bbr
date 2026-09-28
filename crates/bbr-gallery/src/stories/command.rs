//! The G3 Command composition story (packet
//! `component-gallery-compositions.md`): the **exact production
//! `bbr_ui::surfaces::command::command_card`** renders the selected matrix
//! family from the one matrix authority [`crate::command_matrix`] with the
//! plain fixture model built by [`crate::command_fixture`]. The story's
//! state selector switches the matrix family (one surface at a time, so
//! every production id is unique and the E2E can target any element
//! directly); the rejected combinations render as value rows with their
//! reasons. No gallery-local reimplementation of anatomy, no copied tokens,
//! no fake engine/controller/portal/ranking/activation — the production
//! floating command surface takes no callbacks, and the Gallery observes
//! presentation only.

use bbr_ui::surfaces::command::{command_card, CommandRect};
use gpui::{div, prelude::*, px, AnyElement};

use crate::command_fixture::build_command_fixture;
use crate::command_matrix::{command_case_valid, COMMAND_REJECTED, COMMAND_VALID};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The story's state options: the matrix family labels in declaration
/// order (the state selector switches the rendered family). Pinned by a
/// matrix test so the options can never drift from [`COMMAND_VALID`].
pub use crate::command_matrix::COMMAND_STATE_OPTIONS;

/// The gallery test-target selector for the rendered card
/// (`command-family` — one family at a time via the story's state
/// selector). Test target only — never a product id.
pub const COMMAND_FAMILY_SELECTOR: &str = "command-family";

/// The card's resolved placement rect the story draws at (the same §11.4
/// centered-palette / anchored-overlay placement the app's portal probe
/// resolves; the gallery passes a fixed rect because it has no Portal —
/// the geometry authority stays in the app).
const CARD_RECT: CommandRect = CommandRect::new(0.0, 0.0, 600.0, 420.0);

/// §11.4 Command: the production `bbr_ui::surfaces::command::command_card`
/// renders the selected matrix family with the plain fixture model. The
/// state selector switches the family; the rejected combinations render as
/// value rows with their reasons; the stable §15 ids are shown for the E2E
/// targets.
pub fn render_command(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    // The selected family (the story's state selector; a leftover state
    // from another story falls back to the first family). The app module
    // (`command_app.rs`) records the rendered presentation summary on the
    // shared recorder through the same family selection; the story renders
    // the production card from the same fixture authority.
    let case = COMMAND_VALID
        .iter()
        .find(|c| c.label == ctx.state)
        .unwrap_or(&COMMAND_VALID[0]);
    let model = build_command_fixture(case, theme).expect("valid matrix family builds");
    // The rendered card inside a bounded surface wrapper (the card is
    // absolutely positioned at `CARD_RECT`; the wrapper is `relative` so
    // the card's absolute coordinates resolve inside it).
    let surface = div()
        .id("bbr-gallery-command-card-surface")
        .relative()
        .size_full()
        .child(command_card(&model, CARD_RECT))
        .into_any();

    // The rejected combinations with their reasons — the story shows the
    // whole matrix contract, and `command_case_valid`/`build_command_fixture`
    // refuse every one.
    let rejected: Vec<AnyElement> = COMMAND_REJECTED
        .iter()
        .map(|r| {
            assert!(
                !command_case_valid(&r.case),
                "rejected command {} builds",
                r.label
            );
            value_row(theme, r.label, r.reason)
        })
        .collect();

    let mut children = vec![
        section_heading(
            theme,
            "§11.4 floating command surfaces (production bbr_ui::surfaces::command::command_card)",
        ),
        value_row(
            theme,
            "ids",
            "bbr-palette · bbr-palette-field · bbr-palette-text · bbr-palette-mode · bbr-palette-cursor · bbr-palette-result-<k> · bbr-command-overlay · bbr-command-overlay-field · bbr-command-overlay-text · bbr-command-overlay-cursor · bbr-command-result-<k>",
        ),
        value_row(
            theme,
            "callbacks",
            "none — production activation stays app/Portal-owned; the Gallery observes presentation only",
        ),
        value_row(theme, "family", case.label),
        value_row(
            theme,
            "presentation",
            match case.presentation {
                bbr_ui::surfaces::command::CommandPresentation::Closed => "closed",
                bbr_ui::surfaces::command::CommandPresentation::CenteredPalette => {
                    "centered palette (Cmd-T, New Tab staging)"
                }
                bbr_ui::surfaces::command::CommandPresentation::AnchoredOverlay => {
                    "field-anchored overlay (Cmd-L / click, Location)"
                }
            },
        ),
        value_row(
            theme,
            "state",
            match case.state {
                bbr_ui::surfaces::command::CommandResultState::Inactive => "inactive",
                bbr_ui::surfaces::command::CommandResultState::Empty => "empty",
                bbr_ui::surfaces::command::CommandResultState::Loading => "loading",
                bbr_ui::surfaces::command::CommandResultState::Results => "results",
                bbr_ui::surfaces::command::CommandResultState::ActivationError => {
                    "activation-error"
                }
            },
        ),
        value_row(
            theme,
            "rows",
            &format!(
                "{} rendered · selected {} · chrome focus {}",
                model.results.len(),
                model.selected,
                if model.chrome_focused { "on" } else { "off" }
            ),
        ),
        value_row(
            theme,
            "mode fields",
            &format!(
                "{} density · {} contrast · reduced motion {}",
                match model.theme.density {
                    bbr_ui::theme::Density::Small => "S",
                    bbr_ui::theme::Density::Normal => "N",
                    bbr_ui::theme::Density::ExtraLarge => "XL",
                },
                match model.theme.contrast {
                    bbr_ui::theme::ContrastMode::Standard => "standard",
                    bbr_ui::theme::ContrastMode::Increased => "increased",
                },
                if model.theme.motion_mode == bbr_ui::theme::MotionMode::Reduced { "on" } else { "off" },
            ),
        ),
    ];
    // The family viewport renders the production card at its resolved rect
    // (a bounded wrapper; the story page scrolls).
    children.push(
        div()
            .debug_selector(|| COMMAND_FAMILY_SELECTOR.to_string())
            .h(px(460.0))
            .child(surface)
            .into_any(),
    );
    children.push(section_heading(
        theme,
        "Rejected combinations (matrix contract)",
    ));
    children.extend(rejected);
    story_page(ctx, "Command palette / overlay", children)
}
