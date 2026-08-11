//! The Error/Crash app wiring (packet
//! `component-gallery-browser-components.md`, Group 7): the Error story id
//! discovery, the shared recorder state, and the per-render observer /
//! context assembly the gallery app entity needs to render the two
//! browser-component stories that take production `error_surface` /
//! `crash_panel` / `empty_space_pane` callbacks.
//!
//! This module owns the Error/Crash slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the recorder's `build_error_observer` (none of the
//! production error-surface or pane listeners stop propagation — the
//! surface fills the content area and the pane buttons are plain
//! mouse-down controls) — this module only discovers the stories, records
//! what the callbacks report, and assembles the render context.

use gpui::Context;

use crate::app::GalleryApp;
use crate::error_recorder::{build_error_observer, ErrorObservation, ErrorObserver, ErrorRecorder};
use crate::registry::{CatalogRegistry, ErrorStory, StoryId};

/// The Error/Crash app state: the two story ids (found by label at
/// construction — the browser-component stories that take production
/// `error_surface`/`crash_panel`/`empty_space_pane` callbacks), the shared
/// recorder (what the production callbacks report back through the app's
/// production record path), and the observer rebuilt for the current render
/// pass.
pub struct ErrorAppState {
    /// The §11.8 Error surface story id.
    error_story: StoryId,
    /// The §11.8 Crash / Empty panes story id.
    panes_story: StoryId,
    /// The shared Error/Crash recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: ErrorRecorder,
    /// The observer for the current render pass: the production
    /// `error_surface`/`crash_panel`/`empty_space_pane` callbacks re-enter
    /// the app entity through the record closure `render_context` builds.
    observer: Option<ErrorObserver>,
}

impl ErrorAppState {
    /// Find the two Error stories and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let error_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Error surface")
            .expect("error surface story registered")
            .id;
        let panes_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Crash / Empty panes")
            .expect("crash/empty panes story registered")
            .id;
        Self {
            error_story,
            panes_story,
            recorder: ErrorRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Error/Crash observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &ErrorRecorder {
        &self.recorder
    }

    /// Apply one observed Error/Crash interaction (the app's production
    /// record path).
    pub fn observe(&mut self, event: ErrorObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `error_surface`/`crash_panel`/`empty_space_pane`
    /// callbacks through a record closure that upgrades this app entity,
    /// records the interaction on the shared recorder, and notifies it.
    /// When one of the two Error stories is selected the canvas receives
    /// the recorder snapshot plus that observer. The observer lives here
    /// so the returned story can borrow it for the whole render;
    /// `build_error_observer` owns the propagation contract (no production
    /// error-surface or pane listener stops propagation).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<ErrorStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_error_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.error.observe(event);
                cx.notify();
            });
        }));
        if selected == self.error_story || selected == self.panes_story {
            Some(ErrorStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
