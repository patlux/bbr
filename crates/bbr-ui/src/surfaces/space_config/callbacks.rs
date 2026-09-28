//! Space-configuration surface callbacks (DESIGN.md §11.3): the typed
//! listener bundle `bbr-app` wires onto the `bbr-ui` surface and the
//! gallery recorder wires onto the same surface. The surface never holds a
//! controller, engine, or store — every interactive path routes through one
//! of these callbacks.

use std::rc::Rc;

use bbr_domain::{SpaceIdentity, SpaceTheme, TintIntensity};
use gpui::{App, MouseDownEvent, Window};

/// Select one curated theme token (the app's `space_config_select_theme`).
pub type SpaceConfigSelectTheme = Rc<dyn Fn(SpaceTheme, &mut Window, &mut App)>;
/// Select one validated identity — exactly one curated emoji or glyph (the
/// app's `space_config_select_identity`).
pub type SpaceConfigSelectIdentity = Rc<dyn Fn(SpaceIdentity, &mut Window, &mut App)>;
/// Select one curated tint intensity step (v2 D7; the app's
/// `space_config_select_intensity`).
pub type SpaceConfigSelectIntensity = Rc<dyn Fn(TintIntensity, &mut Window, &mut App)>;
/// Toggle the static grain in the draft (v2 D6; the app's
/// `space_config_toggle_grain`).
pub type SpaceConfigToggleGrain = Rc<dyn Fn(&mut Window, &mut App)>;
/// The §13 inside-press marker: a pointer press inside the surface marks
/// itself so the root outside-press handler never dismisses it.
pub type SpaceConfigInsidePress = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;
/// The Done control: commit the rename + appearance for the exact bound
/// SpaceId (the app's `space_config_done`).
pub type SpaceConfigDone = Rc<dyn Fn(&mut Window, &mut App)>;
/// The Cancel control / Escape: close without applying — the created Space
/// and its valid defaults stay (the app's `space_config_cancel`).
pub type SpaceConfigCancel = Rc<dyn Fn(&mut Window, &mut App)>;

/// The full production listener bundle of the §11.3 configuration surface:
/// every interactive path the surface draws routes through one of these
/// callbacks.
pub struct SpaceConfigCallbacks {
    /// Select a curated theme token (pointer press on a swatch).
    pub on_select_theme: SpaceConfigSelectTheme,
    /// Select a validated identity (pointer press on an emoji/glyph option).
    pub on_select_identity: SpaceConfigSelectIdentity,
    /// Select a curated intensity step (pointer press on a step).
    pub on_select_intensity: SpaceConfigSelectIntensity,
    /// Toggle the static grain (pointer press on the grain toggle).
    pub on_toggle_grain: SpaceConfigToggleGrain,
    /// A pointer press inside the surface (§13 one pointer action).
    pub on_inside_press: SpaceConfigInsidePress,
    /// Commit the rename + appearance for the bound SpaceId.
    pub on_done: SpaceConfigDone,
    /// Cancel without applying (the Space stays usable).
    pub on_cancel: SpaceConfigCancel,
}
