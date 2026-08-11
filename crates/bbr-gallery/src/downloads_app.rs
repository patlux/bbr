//! The Downloads panel app wiring (packet
//! `component-gallery-browser-components.md`, Group 5): the Downloads story
//! id discovery, the shared recorder state, the production `ScrollHandle` /
//! `FocusHandle` fixtures, and the per-render observer / context assembly
//! the gallery app entity needs to render the one browser-component story
//! that takes production `downloads_panel_surface` callbacks.
//!
//! This module owns the Downloads slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the surface itself (the `activate`/`delete` closures
//! stop propagation, §11.12/§13) — this module only discovers the story,
//! records what the callbacks report, holds the plain scroll/focus handles
//! the surface model requires, and assembles the render context.

use gpui::{Context, ScrollHandle};

use crate::app::GalleryApp;
use crate::downloads_recorder::{
    build_downloads_observer, DownloadObservation, DownloadsObserver, DownloadsRecorder,
};
use crate::registry::{CatalogRegistry, DownloadsStory, StoryId};

/// The Downloads app state: the story id (found by label at construction),
/// the shared recorder (what the production callbacks report back through
/// the app's production record path), the production `ScrollHandle`/focus
/// handles the surface model borrows, and the observer rebuilt for the
/// current render pass.
pub struct DownloadsAppState {
    /// The Downloads panel story id, found by label at construction.
    story: StoryId,
    /// The shared Downloads recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: DownloadsRecorder,
    /// The production scroll handle the §11.12 scroll families track (the
    /// story applies the fixture's scroll offset to it, so the bounded
    /// window and the wheel never disagree).
    scroll: ScrollHandle,
    /// The §11.12 panel focus handle (the surface tracks focus on it; the
    /// gallery never focuses it, so `focused` stays false except for the
    /// cursor-family fixture which reports its own §9 ring state).
    focus: gpui::FocusHandle,
    /// The observer for the current render pass: the production
    /// `downloads_panel_surface` callbacks re-enter the app entity through
    /// the record closure `render_context` builds.
    observer: Option<DownloadsObserver>,
}

impl DownloadsAppState {
    /// Find the Downloads story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry, cx: &mut Context<GalleryApp>) -> Self {
        let story = registry
            .stories
            .iter()
            .find(|s| s.label == "Downloads panel")
            .expect("downloads panel story registered")
            .id;
        Self {
            story,
            recorder: DownloadsRecorder::default(),
            scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
            observer: None,
        }
    }

    /// The recorded Downloads observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &DownloadsRecorder {
        &self.recorder
    }

    /// Apply one observed Downloads interaction (the app's production
    /// record path).
    pub fn observe(&mut self, event: DownloadObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `downloads_panel_surface` callbacks through a record
    /// closure that upgrades this app entity, records the interaction on
    /// the shared recorder, and notifies it. When the Downloads story is
    /// selected the canvas receives the recorder snapshot, that observer,
    /// and the plain scroll/focus handles. The observer lives here so the
    /// returned story can borrow it for the whole render; the surface owns
    /// the propagation contract (§11.12/§13).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<DownloadsStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_downloads_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.downloads.observe(event);
                cx.notify();
            });
        }));
        if selected == self.story {
            Some(DownloadsStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
                scroll: &self.scroll,
                focus: &self.focus,
            })
        } else {
            None
        }
    }
}
