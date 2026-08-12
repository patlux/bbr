//! The Context-menu app wiring (packet
//! `component-gallery-browser-components.md`, Group 9): the Context-menu
//! story id discovery, the shared recorder state, and the per-render
//! observer / context assembly the gallery app entity needs to render the
//! one browser-component story that takes production
//! `bbr_ui::surfaces::menu::context_menu_card` row callbacks.
//!
//! This module owns the Context-menu slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the recorder's `build_context_menu_observer`
//! (neither row callback stops propagation — the production portal card
//! wrapper owns the §13 stop) — this module only discovers the story,
//! records what the callbacks report, and assembles the render context.

use gpui::Context;

use crate::app::GalleryApp;
use crate::context_menu_recorder::{
    build_context_menu_observer, ContextMenuObservation, ContextMenuObserver, ContextMenuRecorder,
};
use crate::registry::{CatalogRegistry, ContextMenuStory, StoryId};

/// The Context-menu app state: the story ids (found by label at
/// construction — the G2 Context-menu story and the G3 Page-menu
/// composition, the two browser-component stories that take production
/// `context_menu_card` row callbacks), the shared recorder (what the
/// production callbacks report back through the app's production record
/// path), and the observer rebuilt for the current render pass.
pub struct ContextMenuAppState {
    /// The G2 Context-menu story id, found by label at construction.
    story: StoryId,
    /// The G3 Page-menu composition story id, found by label at
    /// construction.
    page_menu_story: StoryId,
    /// The shared Context-menu recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: ContextMenuRecorder,
    /// The observer for the current render pass: the production
    /// `context_menu_card` row callbacks re-enter the app entity through
    /// the record closure `render_context` builds.
    observer: Option<ContextMenuObserver>,
}

impl ContextMenuAppState {
    /// Find the Context-menu and Page-menu stories and start with a fresh
    /// recorder. Both stories render the exact production
    /// `context_menu_card` with the shared recorder callbacks (the G2
    /// matrix story and the G3 enriched composition), so the app state owns
    /// both story ids and one recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let stories = ["Context menu", "Page menu"];
        let mut ids = Vec::with_capacity(stories.len());
        for label in stories {
            ids.push(
                registry
                    .stories
                    .iter()
                    .find(|s| s.label == label)
                    .unwrap_or_else(|| panic!("{label} story registered"))
                    .id,
            );
        }
        Self {
            story: ids[0],
            page_menu_story: ids[1],
            recorder: ContextMenuRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Context-menu observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &ContextMenuRecorder {
        &self.recorder
    }

    /// Apply one observed Context-menu interaction (the app's production
    /// record path).
    pub fn observe(&mut self, event: ContextMenuObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `context_menu_card` row callbacks through a record
    /// closure that upgrades this app entity, records the interaction on
    /// the shared recorder, and notifies it. When the Context-menu story
    /// is selected the canvas receives the recorder snapshot plus that
    /// observer. The observer lives here so the returned story can borrow
    /// it for the whole render; `build_context_menu_observer` owns the
    /// propagation contract (neither callback stops — the portal card
    /// wrapper owns the §13 stop).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<ContextMenuStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_context_menu_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.context_menu.observe(event);
                cx.notify();
            });
        }));
        if selected == self.story || selected == self.page_menu_story {
            Some(ContextMenuStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
