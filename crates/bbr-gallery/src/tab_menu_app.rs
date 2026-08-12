//! The Tab/Favorite context-menu app wiring (task packet
//! `tab-favorite-context-menus`): the story id discovery, the shared
//! recorder state, and the per-render observer / context assembly the
//! gallery app entity needs to render the one story that takes production
//! `bbr_ui::surfaces::tab_menu::tab_menu_card` row + submenu callbacks.
//!
//! This module owns the Tab-menu slice of `app.rs` so the gallery app
//! entity stays within the module budget; the propagation contract stays
//! in the recorder's `build_tab_menu_observer` (neither row callback stops
//! propagation — the caller owns the §13 dismissal).

use gpui::Context;

use crate::app::GalleryApp;
use crate::context::TabMenuStory;
use crate::registry::{CatalogRegistry, StoryId};
use crate::tab_menu_recorder::{
    build_tab_menu_observer, TabMenuObservation, TabMenuObserver, TabMenuRecorder,
};

/// The Tab/Favorite menu app state: the story id (found by label at
/// construction), the shared recorder (what the production callbacks
/// report back), and the observer rebuilt for the current render pass.
pub struct TabMenuAppState {
    /// The Tab-menu story id, found by label at construction.
    story: StoryId,
    /// The shared recorder: plain fixture observation state, written only
    /// through the app's production record path.
    recorder: TabMenuRecorder,
    /// The observer for the current render pass.
    observer: Option<TabMenuObserver>,
}

impl TabMenuAppState {
    /// Find the Tab-menu story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let story = registry
            .stories
            .iter()
            .find(|s| s.label == "Tab / Favorite menu")
            .expect("tab menu story registered")
            .id;
        Self {
            story,
            recorder: TabMenuRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Tab-menu observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &TabMenuRecorder {
        &self.recorder
    }

    /// Apply one observed Tab/Favorite menu interaction.
    pub fn observe(&mut self, event: TabMenuObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: the observer
    /// routes the production `tab_menu_card` callbacks through a record
    /// closure that upgrades this app entity, records the interaction on
    /// the shared recorder, and notifies it.
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<TabMenuStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_tab_menu_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.tab_menu.observe(event);
                cx.notify();
            });
        }));
        if selected == self.story {
            Some(TabMenuStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
