//! The Group 4 app wiring (packet
//! `component-gallery-browser-components.md`): the five sidebar-component
//! story ids (Favorites grid, Folders, Footer, Profile chip S/N/XL, Plus
//! menu), the shared recorder state, the footer `[Downloads]` focus
//! handle, and the per-render observer / context assembly the gallery app
//! entity needs to render the browser-component stories that take
//! production `bbr-ui` callback seams.
//!
//! This module owns the Group 4 slice of `app.rs` so the gallery app
//! entity stays within the module budget. The production propagation
//! contract stays in the recorder's `build_group4_observer` (the folder
//! member close stops propagation — the app's `close_folder_member`
//! listener does the same, §11.11) — this module only discovers the
//! stories, records what the callbacks report, and assembles the render
//! context.

use gpui::{Context, FocusHandle};

use crate::app::GalleryApp;
use crate::group4_recorder::{
    build_group4_observer, Group4Observation, Group4Observer, Group4Recorder,
};
use crate::registry::{CatalogRegistry, Group4Story, StoryId};

/// The Group 4 app state: the five story ids (found by label at
/// construction), the shared recorder (what the production callbacks
/// report back through the app's production record path), the footer
/// `[Downloads]` focus handle (the `footer_surface` keyboard seam — the
/// gallery never focuses it, the handle is the production contract), and
/// the observer rebuilt for the current render pass.
pub struct Group4AppState {
    /// The §11.10 Favorites grid story id.
    favorites_story: StoryId,
    /// The §11.11 Folders story id.
    folders_story: StoryId,
    /// The §11.4 Footer story id.
    footer_story: StoryId,
    /// The §12 Profile chip (S/N/XL) story id.
    profile_chip_story: StoryId,
    /// The §11.4 Plus menu story id.
    menu_story: StoryId,
    /// The shared Group 4 recorder: plain fixture observation state,
    /// written only through the app's production record path.
    recorder: Group4Recorder,
    /// The observer for the current render pass: the production
    /// sidebar-component callbacks re-enter the app entity through the
    /// record closure `render_context` builds.
    observer: Option<Group4Observer>,
    /// The `footer_surface` `[Downloads]` keyboard-focus handle (the
    /// production contract; the gallery renders the control with the
    /// handle tracked but never keyboard-focuses it).
    downloads_focus: FocusHandle,
}

impl Group4AppState {
    /// Find the five Group 4 stories and start with a fresh recorder plus
    /// one stable footer focus handle.
    pub fn new(registry: &CatalogRegistry, cx: &mut Context<GalleryApp>) -> Self {
        let find = |label: &str| {
            registry
                .stories
                .iter()
                .find(|s| s.label == label)
                .unwrap_or_else(|| panic!("{label} story registered"))
                .id
        };
        Self {
            favorites_story: find("Favorites grid"),
            folders_story: find("Folders"),
            footer_story: find("Footer"),
            profile_chip_story: find("Profile chip (S/N/XL)"),
            menu_story: find("Plus menu"),
            recorder: Group4Recorder::default(),
            observer: None,
            downloads_focus: cx.focus_handle(),
        }
    }

    /// The recorded Group 4 observation state (the snapshot reads it
    /// through the app's production record path).
    pub fn recorder(&self) -> &Group4Recorder {
        &self.recorder
    }

    /// Apply one observed Group 4 interaction (the app's production record
    /// path).
    pub fn observe(&mut self, event: Group4Observation) {
        self.recorder.observe(event);
    }

    /// Whether the selected story is one of the five Group 4 stories.
    fn is_group4_story(&self, selected: StoryId) -> bool {
        selected == self.favorites_story
            || selected == self.folders_story
            || selected == self.footer_story
            || selected == self.profile_chip_story
            || selected == self.menu_story
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production sidebar-component callbacks through a record closure
    /// that upgrades this app entity, records the interaction on the
    /// shared recorder, and notifies it. When one of the five Group 4
    /// stories is selected the canvas receives the recorder snapshot plus
    /// that observer and the footer focus handle. The observer lives here
    /// so the returned story can borrow it for the whole render;
    /// `build_group4_observer` owns the propagation contract (§11.11: the
    /// folder member close stops propagation).
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<Group4Story<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_group4_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.group4.observe(event);
                cx.notify();
            });
        }));
        if self.is_group4_story(selected) {
            Some(Group4Story {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
                downloads_focus: self.downloads_focus.clone(),
            })
        } else {
            None
        }
    }
}
