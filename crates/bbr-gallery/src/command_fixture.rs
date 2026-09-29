//! The Command fixture builder (packet `component-gallery-compositions.md`):
//! constructs the plain [`bbr_ui::surfaces::command::CommandModel`] for a
//! matrix family — **only** the model, never a visual decision and never a
//! fake activation (production activation stays app/Portal-owned). The
//! builder refuses (`None`) any case the validity authority
//! [`command_case_valid`] rejects, and the missing-key claim is additionally
//! refused through the production [`CommandModel::validate`] path so the
//! gallery exercises the exact bbr-ui validation the app's render depends
//! on.
//!
//! The gallery story and the E2E consume this single fixture authority;
//! every visual outcome is whatever the production
//! `bbr_ui::surfaces::command::command_card` renders from the model.

use bbr_ui::surfaces::command::{CommandModel, CommandResultRow};
use bbr_ui::theme::{ContrastMode, MotionMode, Theme};

use crate::command_matrix::{command_case_valid, CommandCase};

/// Matrix density is intrinsic to the family, as in the Tab Row stories.
/// Canvas appearance is inherited; an increased-contrast/reduced-motion family
/// must keep its named requirement even when the canvas controls are standard.
/// The capture inventory must record these overrides, not claim twelve unique
/// component variants when only the surrounding canvas differs.
pub fn fixture_theme(case: &CommandCase, canvas: &Theme) -> Theme {
    Theme::new(
        canvas.mode,
        if case.contrast == ContrastMode::Increased {
            case.contrast
        } else {
            canvas.contrast
        },
        case.density,
    )
    .with_motion_mode(if case.reduced_motion {
        MotionMode::Reduced
    } else {
        canvas.motion_mode
    })
}

/// Build the plain production model for one valid matrix family; `None` for
/// any rejected combination (the mutual-exclusion / missing-key contract).
pub fn build_command_fixture(case: &CommandCase, theme: &Theme) -> Option<CommandModel> {
    if !command_case_valid(case) {
        return None;
    }
    let results: Vec<CommandResultRow> = case
        .rows
        .iter()
        .map(|spec| CommandResultRow {
            key: spec.key.to_string(),
            group: spec.group.to_string(),
            title: if case.long_titles {
                // §4: the production row truncates the title at 140 pt; the
                // fixture feeds a genuinely long one-line title so the
                // truncation renders.
                "A genuinely long tab title that must truncate inside the row — this is the full one-line text".to_string()
            } else {
                spec.title.to_string()
            },
            subtitle: spec.subtitle.map(str::to_string),
            url: None,
            destructive: spec.destructive,
        })
        .collect();
    let model = CommandModel {
        presentation: case.presentation,
        theme: fixture_theme(case, theme),
        chrome_focused: case.chrome_focused,
        text: case.text.to_string(),
        edit: bbr_ui::components::EditState::end_of(case.text),
        line_sink: None,
        state: case.state,
        results,
        selected: case.selected,
        entrance: None,
        glide: None,
    };
    // The production validation authority: the model must be renderable
    // (the rejected missing-key case fails here and refuses to build).
    model.validate().ok()?;
    Some(model)
}
