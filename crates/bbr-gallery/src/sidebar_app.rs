//! The G3 Sidebar composition app wiring (packet
//! `component-gallery-compositions.md`): the Sidebar story id, the shared
//! recorder state, the fixture handles (the GPUI focus/scroll handles the
//! plain model needs — created once here, never focused by the gallery),
//! and the per-render observer / context assembly the gallery app entity
//! needs to render the composition story that takes the production
//! `bbr_ui::sidebar::sidebar_surface` `Callbacks` seam.
//!
//! This module owns the Sidebar slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the observer's `build_sidebar_observer` (the row
//! close, pin, folder member close, plus-menu commit, plus-menu inside
//! press, downloads, plus, and divider mouse-down stop propagation —
//! exactly the app's `sidebar_callbacks` wiring); this module only
//! discovers the story, records what the callbacks report, and assembles
//! the render context.

use gpui::{Context, ScrollHandle};

use crate::app::GalleryApp;
use crate::registry::{CatalogRegistry, SidebarStory, StoryId};
use crate::sidebar_fixture::SidebarFixtureHandles;
use crate::sidebar_observer::{build_sidebar_observer, SidebarObserver};
use crate::sidebar_recorder::{SidebarObservation, SidebarRecorder};

/// The G3 Sidebar app state: the story id (found by label at
/// construction), the shared recorder (what the production `Callbacks`
/// report back through the app's production record path), the fixture
/// handles the plain model needs (created once; the gallery never focuses
/// them), and the observer rebuilt for the current render pass.
pub struct SidebarAppState {
    /// The §11.4 Sidebar composition story id.
    sidebar_story: StoryId,
    /// The shared Sidebar recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: SidebarRecorder,
    /// The observer for the current render pass: the production sidebar
    /// `Callbacks` re-enter the app entity through the record closure
    /// `render_context` builds.
    observer: Option<SidebarObserver>,
    /// The fixture handles the plain model needs (GPUI handles cannot be
    /// created without a context; created once here and shared across
    /// every matrix family).
    handles: SidebarFixtureHandles,
}

impl SidebarAppState {
    /// Find the Sidebar story and start with a fresh recorder plus the
    /// stable fixture handles.
    pub fn new(registry: &CatalogRegistry, cx: &mut Context<GalleryApp>) -> Self {
        let sidebar_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Sidebar")
            .unwrap_or_else(|| panic!("Sidebar story registered"))
            .id;
        Self {
            sidebar_story,
            recorder: SidebarRecorder::default(),
            observer: None,
            handles: SidebarFixtureHandles {
                tab_list: cx.focus_handle(),
                folders: cx.focus_handle(),
                plus_menu: cx.focus_handle(),
                space_strip: cx.focus_handle(),
                divider: cx.focus_handle(),
                downloads: cx.focus_handle(),
                dense_scroll: ScrollHandle::new(),
                space_strip_scroll: ScrollHandle::new(),
            },
        }
    }

    /// The recorded Sidebar observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &SidebarRecorder {
        &self.recorder
    }

    /// Apply one observed Sidebar interaction (the app's production record
    /// path).
    pub fn observe(&mut self, event: SidebarObservation) {
        self.recorder.observe(event);
    }

    /// Whether the selected story is the Sidebar composition story.
    fn is_sidebar_story(&self, selected: StoryId) -> bool {
        selected == self.sidebar_story
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production sidebar `Callbacks` through a record closure that
    /// upgrades this app entity, records the interaction on the shared
    /// recorder, and notifies it. When the Sidebar story is selected the
    /// canvas receives the recorder snapshot, that observer, and the
    /// fixture handles. The observer lives here so the returned story can
    /// borrow it for the whole render; `build_sidebar_observer` owns the
    /// propagation contract.
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<SidebarStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_sidebar_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.sidebar.observe(event);
                cx.notify();
            });
        }));
        if self.is_sidebar_story(selected) {
            Some(SidebarStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
                handles: &self.handles,
            })
        } else {
            None
        }
    }

    /// The fixture handles (the E2E focuses them to drive the production
    /// keyboard paths through real keystrokes).
    #[cfg(test)]
    pub(crate) fn handles(&self) -> &SidebarFixtureHandles {
        &self.handles
    }
}
