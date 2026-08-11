//! The Address/Input app wiring (packet
//! `component-gallery-browser-components.md`, Group 3): the Address story
//! id discovery, the shared recorder state, and the per-render observer /
//! context assembly the gallery app entity needs to render the two
//! browser-component stories that take production `address_control` /
//! `staging_surface` callbacks.
//!
//! This module owns the Address/Input slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the recorder's `build_address_observer` (`on_open`
//! stops propagation — the app opens the §11.4 overlay on press, §11.5/
//! §13; `on_hover` never stops) — this module only discovers the stories,
//! records what the callbacks report, and assembles the render context.

use gpui::Context;

use crate::address_recorder::{
    build_address_observer, AddressObservation, AddressObserver, AddressRecorder,
};
use crate::app::GalleryApp;
use crate::registry::{AddressStory, CatalogRegistry, StoryId};

/// The Address/Input app state: the two story ids (found by label at
/// construction — the browser-component stories that take production
/// `address_control`/`staging_surface` callbacks), the shared recorder
/// (what the production callbacks report back through the app's production
/// record path), and the observer rebuilt for the current render pass.
pub struct AddressAppState {
    /// The §11.5 Address control story id.
    address_story: StoryId,
    /// The §11.5 Command staging story id.
    staging_story: StoryId,
    /// The shared Address/Staging recorder: plain fixture observation
    /// state, written only through the app's production record path.
    recorder: AddressRecorder,
    /// The observer for the current render pass: the production
    /// `address_control`/`staging_surface` callbacks re-enter the app
    /// entity through the record closure `render_context` builds.
    observer: Option<AddressObserver>,
}

impl AddressAppState {
    /// Find the two Address stories and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let address_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Address control")
            .expect("address control story registered")
            .id;
        let staging_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Command staging")
            .expect("command staging story registered")
            .id;
        Self {
            address_story,
            staging_story,
            recorder: AddressRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Address/Staging observation state (the snapshot reads
    /// it through the app's production record path).
    pub fn recorder(&self) -> &AddressRecorder {
        &self.recorder
    }

    /// Apply one observed Address/Staging interaction (the app's
    /// production record path).
    pub fn observe(&mut self, event: AddressObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `address_control`/`staging_surface` callbacks through a
    /// record closure that upgrades this app entity, records the
    /// interaction on the shared recorder, and notifies it. When one of
    /// the two Address stories is selected the canvas receives the recorder
    /// snapshot plus that observer. The observer lives here so the returned
    /// story can borrow it for the whole render; `build_address_observer`
    /// owns the propagation contract (§11.5/§13: `on_open` stops, `on_hover`
    /// never stops).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<AddressStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_address_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.address.observe(event);
                cx.notify();
            });
        }));
        if selected == self.address_story || selected == self.staging_story {
            Some(AddressStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
