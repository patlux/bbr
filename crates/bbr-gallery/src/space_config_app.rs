//! The Space-configuration app wiring (packet
//! `space-creation-appearance-ui.md`): the story id discovery, the shared
//! recorder state, and the per-render observer / context assembly the
//! gallery app entity needs to render the production
//! `bbr_ui::surfaces::space_config` surface with recorder callbacks.
//!
//! This module owns the Space-configuration slice of `app.rs` so the
//! gallery app entity stays within the module budget. The production
//! propagation contract stays in the recorder's
//! `build_space_config_observer` (the inside-press marker stops
//! propagation, exactly like the production app seam); this module only
//! discovers the story, records what the callbacks report, and assembles
//! the render context.

use gpui::{Context, FocusHandle};

use crate::app::GalleryApp;
use crate::registry::{CatalogRegistry, SpaceConfigStory, StoryId};
use crate::space_config_recorder::{
    build_space_config_observer, SpaceConfigObservation, SpaceConfigObserver, SpaceConfigRecorder,
};

/// The Space-configuration app state: the story id (found by label at
/// construction), the shared recorder, the fixture focus handle (the
/// surface's §9 keyboard domain; created once, the gallery never focuses
/// it), and the observer rebuilt for the current render pass.
pub struct SpaceConfigAppState {
    /// The Space-configuration story id, found by label at construction.
    story: StoryId,
    /// The shared recorder: plain fixture observation state, written only
    /// through the production callbacks.
    recorder: SpaceConfigRecorder,
    /// The fixture focus handle the production surface tracks (§9 ring +
    /// keyboard domain; the gallery never focuses it).
    focus: FocusHandle,
    /// The observer for the current render pass.
    observer: Option<SpaceConfigObserver>,
}

impl SpaceConfigAppState {
    /// Find the Space-configuration story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry, cx: &mut Context<GalleryApp>) -> Self {
        let story = registry
            .stories
            .iter()
            .find(|s| s.label == "Space configuration")
            .expect("space configuration story registered")
            .id;
        Self {
            story,
            recorder: SpaceConfigRecorder::default(),
            focus: cx.focus_handle(),
            observer: None,
        }
    }

    /// The recorded observation state (the snapshot reads it through the
    /// app's production record path).
    pub fn recorder(&self) -> &SpaceConfigRecorder {
        &self.recorder
    }

    /// Apply one observed interaction (the app's production record path).
    pub fn observe(&mut self, event: SpaceConfigObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: the observer
    /// routes the production surface callbacks through a record closure
    /// that upgrades this app entity, records the interaction on the
    /// shared recorder, and notifies it. When the Space-configuration
    /// story is selected the canvas receives the recorder snapshot plus
    /// that observer.
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<SpaceConfigStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_space_config_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.space_config.observe(event);
                cx.notify();
            });
        }));
        if selected == self.story {
            Some(SpaceConfigStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
                focus: &self.focus,
            })
        } else {
            None
        }
    }
}
