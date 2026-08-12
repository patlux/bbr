//! Gallery-owned observation state for the Space-configuration story
//! (packet `space-creation-appearance-ui.md`).
//!
//! The story renders the **exact production `bbr-ui` surface**
//! (`bbr_ui::surfaces::space_config::space_config_surface`) with plain
//! fixture models and recorder callbacks — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production callbacks report
//! back so the E2E can assert real pointer interactions and the canvas can
//! reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use bbr_domain::{SpaceIdentity, SpaceTheme};
use gpui::{App, MouseDownEvent, Window};

/// What a production Space-configuration callback reports back to the
/// gallery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceConfigObservation {
    /// The production `on_select_theme` fired for a curated theme token.
    ThemeSelected(SpaceTheme),
    /// The production `on_select_identity` fired for a curated identity.
    IdentitySelected(SpaceIdentity),
    /// The production `on_done` fired (commit the rename + appearance).
    Done,
    /// The production `on_cancel` fired (keep the created Space + defaults).
    Cancel,
    /// The production `on_inside_press` fired (a press inside the surface).
    InsidePress,
}

/// The shared Space-configuration recorder: plain fixture observation
/// state, written only through the production callbacks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpaceConfigRecorder {
    /// The observations in dispatch order.
    pub events: Vec<SpaceConfigObservation>,
}

impl SpaceConfigRecorder {
    /// Apply one observed Space-configuration interaction.
    pub fn observe(&mut self, event: SpaceConfigObservation) {
        self.events.push(event);
    }
}

/// The production `on_select_theme` callback shape
/// (`SpaceConfigSelectTheme`): re-entering the app through the production
/// record path.
pub type SpaceConfigSelectThemeFn = Rc<dyn Fn(SpaceTheme, &mut Window, &mut App)>;
/// The production `on_select_identity` callback shape
/// (`SpaceConfigSelectIdentity`).
pub type SpaceConfigSelectIdentityFn = Rc<dyn Fn(SpaceIdentity, &mut Window, &mut App)>;
/// The production `on_inside_press` callback shape
/// (`SpaceConfigInsidePress`).
pub type SpaceConfigInsidePressFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;
/// The production `on_done` / `on_cancel` callback shape
/// (`SpaceConfigDone` / `SpaceConfigCancel`).
pub type SpaceConfigActionFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// The observer bundle the Space-configuration story routes the production
/// surface callbacks through. Each callback records its interaction; the
/// §13 inside-press marker stops propagation exactly like the production
/// app seam (the surface marks itself so an outside-press handler never
/// dismisses it).
pub struct SpaceConfigObserver {
    pub on_select_theme: SpaceConfigSelectThemeFn,
    pub on_select_identity: SpaceConfigSelectIdentityFn,
    pub on_inside_press: SpaceConfigInsidePressFn,
    pub on_done: SpaceConfigActionFn,
    pub on_cancel: SpaceConfigActionFn,
}

/// Build the observer bundle: every callback records its interaction
/// through `record` (which upgrades the gallery app entity, writes the
/// shared recorder, and notifies). The §13 inside-press marker stops
/// propagation exactly like the production app seam.
pub fn build_space_config_observer(
    record: impl Fn(SpaceConfigObservation, &mut App) + 'static,
) -> SpaceConfigObserver {
    let record = Rc::new(record);
    SpaceConfigObserver {
        on_select_theme: {
            let record = record.clone();
            Rc::new(move |token, _window, cx| {
                record(SpaceConfigObservation::ThemeSelected(token), cx);
            })
        },
        on_select_identity: {
            let record = record.clone();
            Rc::new(move |identity, _window, cx| {
                record(SpaceConfigObservation::IdentitySelected(identity), cx);
            })
        },
        on_inside_press: {
            let record = record.clone();
            Rc::new(move |_event, _window, cx| {
                record(SpaceConfigObservation::InsidePress, cx);
                cx.stop_propagation();
            })
        },
        on_done: {
            let record = record.clone();
            Rc::new(move |_window, cx| {
                record(SpaceConfigObservation::Done, cx);
            })
        },
        on_cancel: {
            let record = record.clone();
            Rc::new(move |_window, cx| {
                record(SpaceConfigObservation::Cancel, cx);
            })
        },
    }
}
