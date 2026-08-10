//! The Tab Row app wiring (packet
//! `component-gallery-browser-components.md`): the Tab Row story id
//! discovery, the shared recorder state, and the per-render observer /
//! context assembly the gallery app entity needs to render the one
//! browser-component story that takes production `tab_row` callbacks.
//!
//! This module owns the Tab Row slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the recorder's `build_tab_row_observer` (only the
//! close/pin listeners stop propagation; a row-body press fires both
//! `select` and `begin_drag`, the exact production wiring
//! `bbr-app`'s `sidebar_view` uses, DESIGN.md §11.1/§11.2/§10.9) —
//! this module only discovers the story, records what the callbacks
//! report, and assembles the render context.

use gpui::Context;

use crate::app::GalleryApp;
use crate::recorder::{build_tab_row_observer, TabRowObservation, TabRowObserver, TabRowRecorder};
use crate::registry::{CatalogRegistry, StoryId, TabRowStory};

/// The Tab Row app state: the story id (found by label at construction —
/// the one browser-component story that takes production `tab_row`
/// callbacks), the shared recorder (what the production `tab_row`
/// callbacks report back through the app's production record path), and
/// the observer rebuilt for the current render pass.
pub struct TabRowAppState {
    /// The Tab Row story id, found by label at construction.
    story: StoryId,
    /// The shared Tab Row recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: TabRowRecorder,
    /// The observer for the current render pass: the production `tab_row`
    /// callbacks re-enter the app entity through the record closure
    /// `render_context` builds.
    observer: Option<TabRowObserver>,
}

impl TabRowAppState {
    /// Find the Tab Row story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let story = registry
            .stories
            .iter()
            .find(|s| s.label == "Tab Row")
            .expect("tab row story registered")
            .id;
        Self {
            story,
            recorder: TabRowRecorder::default(),
            observer: None,
        }
    }

    /// The recorded interaction counts in snapshot order (selects,
    /// closes, pins, drag-begins).
    pub fn counts(&self) -> (u32, u32, u32, u32) {
        (
            self.recorder.selects,
            self.recorder.closes,
            self.recorder.pins,
            self.recorder.drag_begins,
        )
    }

    /// Apply one observed Tab Row interaction (the app's production
    /// record path).
    pub fn observe(&mut self, event: TabRowObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `tab_row` callbacks through a record closure that
    /// upgrades this app entity, records the interaction on the shared
    /// recorder, and notifies it. When the Tab Row story is selected the
    /// canvas receives the recorder snapshot plus that observer. The
    /// observer lives here so the returned story can borrow it for the
    /// whole render; `build_tab_row_observer` owns the propagation
    /// contract (only close/pin stop, §11.1/§11.2; a row-body press fires
    /// both select and begin_drag, the production two-listener path
    /// §11.1/§10.9).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<TabRowStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_tab_row_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.tab_row.observe(event);
                cx.notify();
            });
        }));
        if selected == self.story {
            Some(TabRowStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
