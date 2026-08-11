//! Gallery-owned observation state for the G3 Command composition story
//! (packet `component-gallery-compositions.md`).
//!
//! The Command story renders the **exact production
//! `bbr_ui::surfaces::command::command_card`** with a plain fixture-built
//! [`bbr_ui::surfaces::command::CommandModel`] — never a gallery-local
//! wrapper, reimplementation, restyle, or copy. The production floating
//! command surface has **no activation callbacks** (keyboard/pointer
//! activation stays app/Portal-owned), so the gallery's only contribution
//! is this observation state: the summary of the presentation model the
//! story actually rendered, written through the app's production record
//! path each render. The E2E asserts the observed presentation/state/rows
//! and the stable rendered ids — presentation observation only, never a
//! fake activation or product-behavior claim.

use bbr_ui::surfaces::command::{CommandModel, CommandPresentation, CommandResultState};

/// The observed presentation the Command story rendered: a plain snapshot
/// of the production model's fields (presentation discriminant, typed
/// state, focus flag, selection, and the rendered stable keys). Written
/// only through the app's production record path on each render.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRecorder {
    /// The rendered presentation discriminant (`closed` / `palette` /
    /// `overlay`).
    pub presentation: Option<String>,
    /// The rendered UI-local typed state (`inactive` / `empty` / `loading`
    /// / `results` / `activation-error`).
    pub state: Option<String>,
    /// The chrome-focused flag the model carried.
    pub chrome_focused: bool,
    /// The selected row index.
    pub selected: usize,
    /// The rendered stable row keys in render order.
    pub rows: Vec<String>,
    /// How many renders observed this family (the story re-records on each
    /// state-selector switch).
    pub renders: u32,
}

/// One observed Command render (the app's production record path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandObservation {
    pub presentation: CommandPresentation,
    pub state: CommandResultState,
    pub chrome_focused: bool,
    pub selected: usize,
    pub rows: Vec<String>,
}

impl CommandRecorder {
    /// Record one rendered model summary (the app's production record
    /// path).
    pub fn observe(&mut self, event: CommandObservation) {
        self.presentation = Some(match event.presentation {
            CommandPresentation::Closed => "closed".to_string(),
            CommandPresentation::CenteredPalette => "palette".to_string(),
            CommandPresentation::AnchoredOverlay => "overlay".to_string(),
        });
        self.state = Some(match event.state {
            CommandResultState::Inactive => "inactive".to_string(),
            CommandResultState::Empty => "empty".to_string(),
            CommandResultState::Loading => "loading".to_string(),
            CommandResultState::Results => "results".to_string(),
            CommandResultState::ActivationError => "activation-error".to_string(),
        });
        self.chrome_focused = event.chrome_focused;
        self.selected = event.selected;
        self.rows = event.rows;
        self.renders += 1;
    }

    /// The observation of one built model (the summary the app records).
    pub fn of(model: &CommandModel) -> CommandObservation {
        CommandObservation {
            presentation: model.presentation,
            state: model.state,
            chrome_focused: model.chrome_focused,
            selected: model.selected,
            rows: model.results.iter().map(|row| row.key.clone()).collect(),
        }
    }
}
