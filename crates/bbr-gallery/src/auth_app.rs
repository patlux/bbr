//! The Auth sheet app wiring (packet `component-gallery-browser-components.md`,
//! Group 8): the Auth story id discovery, the shared recorder state, and
//! the per-render observer / context assembly the gallery app entity needs
//! to render the browser-surface story that takes production
//! `bbr_ui::surfaces::auth::auth_sheet` callbacks.
//!
//! This module owns the Auth slice of `app.rs` so the gallery app entity
//! stays within the module budget. The production propagation contract
//! stays in the recorder's `build_auth_observer` (the portal listeners do
//! not stop propagation inside the card) — this module only discovers the
//! story, records what the callbacks report, and assembles the render
//! context.

use gpui::Context;

use crate::app::GalleryApp;
use crate::auth_recorder::{build_auth_observer, AuthObservation, AuthObserver, AuthRecorder};
use crate::registry::{AuthStory, CatalogRegistry, StoryId};

/// The Auth app state: the story id (found by label at construction — the
/// browser-surface story that takes production `auth_sheet` callbacks),
/// the shared recorder (what the production callbacks report back through
/// the app's production record path), and the observer rebuilt for the
/// current render pass.
pub struct AuthAppState {
    /// The credential-sheet story id.
    auth_story: StoryId,
    /// The shared Auth recorder: plain fixture observation state, written
    /// only through the app's production record path.
    recorder: AuthRecorder,
    /// The observer for the current render pass: the production
    /// `auth_sheet` callbacks re-enter the app entity through the record
    /// closure `render_context` builds.
    observer: Option<AuthObserver>,
}

impl AuthAppState {
    /// Find the Auth story and start with a fresh recorder.
    pub fn new(registry: &CatalogRegistry) -> Self {
        let auth_story = registry
            .stories
            .iter()
            .find(|s| s.label == "Auth sheet")
            .expect("auth sheet story registered")
            .id;
        Self {
            auth_story,
            recorder: AuthRecorder::default(),
            observer: None,
        }
    }

    /// The recorded Auth observation state (the snapshot reads it through
    /// the app's production record path).
    pub fn recorder(&self) -> &AuthRecorder {
        &self.recorder
    }

    /// Apply one observed Auth interaction (the app's production record
    /// path).
    pub fn observe(&mut self, event: AuthObservation) {
        self.recorder.observe(event);
    }

    /// Build the story context for the current render pass: this module
    /// owns the observe/render re-entry — the observer routes the
    /// production `auth_sheet` callbacks through a record closure that
    /// upgrades this app entity, records the interaction on the shared
    /// recorder, and notifies it. When the Auth story is selected the
    /// canvas receives the recorder snapshot plus that observer. The
    /// observer lives here so the returned story can borrow it for the
    /// whole render; `build_auth_observer` owns the propagation contract.
    pub fn render_context(
        &mut self,
        selected: StoryId,
        cx: &mut Context<GalleryApp>,
    ) -> Option<AuthStory<'_>> {
        let entity = cx.entity().downgrade();
        self.observer = Some(build_auth_observer(move |event, cx| {
            let _ = entity.update(cx, |app, cx| {
                app.auth.observe(event);
                cx.notify();
            });
        }));
        if selected == self.auth_story {
            Some(AuthStory {
                recorder: &self.recorder,
                observer: self.observer.as_ref().expect("observer built above"),
            })
        } else {
            None
        }
    }
}
