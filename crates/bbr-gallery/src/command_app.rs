//! The G3 Command composition app wiring (packet
//! `component-gallery-compositions.md`): the Command story id, the shared
//! observation recorder, and the per-render context assembly the gallery
//! app entity needs to render the composition story that takes the
//! production `bbr_ui::surfaces::command::command_card`.
//!
//! This module owns the Command slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production floating command
//! surface takes no callbacks (activation stays app/Portal-owned), so this
//! module only discovers the story, builds the selected family's fixture
//! model, records what was rendered (the presentation summary — the
//! "observer-or-snapshot" evidence for a callback-free surface), and
//! assembles the render context.

use bbr_ui::surfaces::command::CommandModel;

use crate::command_fixture::build_command_fixture;
use crate::command_matrix::COMMAND_VALID;
use crate::command_recorder::{CommandObservation, CommandRecorder};
use crate::registry::{CommandStory, StoryId};

/// The G3 Command app state: the story id (found by label at
/// construction), the shared recorder (what the story rendered), the
/// current family's fixture model (rebuilt each render), and the current
/// render context.
pub struct CommandAppState {
    /// The §11.4 floating-command composition story id.
    command_story: StoryId,
    /// The shared Command recorder: the presentation summary of the model
    /// the story rendered, written only through the app's production
    /// record path.
    recorder: CommandRecorder,
    /// The current family's fixture model (rebuilt each render; the story
    /// borrows it through the render context).
    model: Option<CommandModel>,
}

impl CommandAppState {
    /// Find the Command story and start with a fresh recorder.
    pub fn new(registry: &crate::registry::CatalogRegistry) -> Self {
        let command_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Command palette / overlay")
            .unwrap_or_else(|| panic!("Command story registered"))
            .id;
        Self {
            command_story,
            recorder: CommandRecorder::default(),
            model: None,
        }
    }

    /// The recorded Command observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &CommandRecorder {
        &self.recorder
    }

    /// Record one rendered Command model summary (the app's production
    /// record path).
    pub fn observe(&mut self, event: CommandObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — it builds the selected family's
    /// fixture model, records its presentation summary on the shared
    /// recorder, and hands the canvas the recorder snapshot plus the
    /// model. When the Command story is selected the canvas receives the
    /// context (the production `command_card` renders the model at a
    /// gallery-owned rect).
    ///
    /// `family_label` is the story's current state-selector label; a
    /// leftover state from another story falls back to the first family.
    pub fn render_context(
        &mut self,
        selected: StoryId,
        family_label: &str,
    ) -> Option<CommandStory<'_>> {
        if selected != self.command_story {
            self.model = None;
            return None;
        }
        let case = COMMAND_VALID
            .iter()
            .find(|c| c.label == family_label)
            .unwrap_or(&COMMAND_VALID[0]);
        self.model = build_command_fixture(case);
        if let Some(model) = &self.model {
            self.observe(CommandRecorder::of(model));
        }
        Some(CommandStory {
            recorder: &self.recorder,
            model: self.model.as_ref(),
        })
    }
}
