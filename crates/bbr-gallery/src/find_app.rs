//! The Find app wiring (packet
//! `component-gallery-browser-components.md`, Group 6): the Find story id
//! discovery, the shared recorder state, and the per-render observer /
//! context assembly the gallery app entity needs to render the
//! browser-component story that takes production `find_bar` callbacks.
//!
//! This module owns the Find slice of `app.rs` so the gallery app entity
//! stays within the module budget. The production propagation contract
//! stays in the recorder's `build_find_observer` (the find controls never
//! stop propagation — the portal root's card-level §13 listener owns the
//! stop; the gallery has no backdrop) — this module only discovers the
//! story, records what the callbacks report, and assembles the render
//! context.

use gpui::Context;

use crate::app::GalleryApp;
use crate::find_recorder::{build_find_observer, FindObserver, FindRecorder};
use crate::registry::{CatalogRegistry, FindStory, StoryId};

/// The Find app state: the Find story id (found by label at construction
/// — the browser-component story that takes production `find_bar`
/// callbacks), the shared recorder (what the production callbacks report
/// back through the app's production record path), and the observer
/// rebuilt for the current render pass.
pub struct FindAppState {
    /// The §11.4 Find bar story id.
    find_story: StoryId,
    /// The shared Find recorder: plain fixture observation state, written
    /// only through the app's production record path.
    recorder: FindRecorder,
    /// The observer for the current render pass: the production
    /// `find_bar` callbacks re-enter the app entity through the record
    /// closure `render_context` builds.
    observer: Option<FindObserver>,
}

impl FindAppState {
    /// Find the Find story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let find_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Find bar")
            .expect("find bar story registered")
            .id;
        Self {
            find_story,
            recorder: FindRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Find observation state (the snapshot reads it through
    /// the app's production record path).
    pub fn recorder(&self) -> &FindRecorder {
        &self.recorder
    }

    /// Apply one observed Find interaction (the app's production record
    /// path).
    pub fn observe(&mut self, event: crate::find_recorder::FindObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `find_bar` callbacks through a record closure that
    /// upgrades this app entity, records the interaction on the shared
    /// recorder, and notifies it. When the Find story is selected the
    /// canvas receives the recorder snapshot plus that observer. The
    /// observer lives here so the returned story can borrow it for the
    /// whole render; `build_find_observer` owns the propagation contract
    /// (the find controls never stop — the portal root's card-level §13
    /// listener owns the stop).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<FindStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_find_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.find.observe(event);
                cx.notify();
            });
        }));
        if selected == self.find_story {
            Some(FindStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
