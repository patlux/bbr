//! Space-configuration surface (DESIGN.md §11.3, packet
//! `space-creation-appearance-ui.md`).
//!
//! The BBR-owned rename/identity surface presented after Space creation and
//! from the active Space's management affordance. The surface consumes the
//! existing curated emoji/BBR-glyph appearance contract
//! ([`bbr_domain::SpaceTheme`], [`bbr_domain::SpaceEmoji`],
//! [`bbr_domain::BbrGlyph`], [`bbr_domain::SpaceIdentity`]) — no free-form
//! icon value, no free color picker. It is ordinary GPUI sidebar chrome (an
//! in-rail panel above the footer row), never a Portal presentation and
//! never a second product authority: `bbr-app` owns the bound `SpaceId`,
//! the rename/appearance mutations, and every dismissal; the surface
//! renders only.
//!
//! Gallery stories drive the exact same production surface with fixture
//! models and recorder callbacks — no fake engine, controller, or store.

pub mod callbacks;
pub mod ids;
pub mod model;
mod paper;
pub mod render;

pub use callbacks::{
    SpaceConfigCallbacks, SpaceConfigCancel, SpaceConfigDone, SpaceConfigInsidePress,
    SpaceConfigSelectIdentity, SpaceConfigSelectIntensity, SpaceConfigSelectTheme,
    SpaceConfigToggleGrain,
};
pub use ids::{
    space_config_emoji_id, space_config_glyph_id, space_config_ids, space_config_intensity_id,
    space_config_theme_id, SPACE_CONFIG_ACTIONS_ID, SPACE_CONFIG_CANCEL_ID, SPACE_CONFIG_DONE_ID,
    SPACE_CONFIG_EMOJI_LABEL_ID, SPACE_CONFIG_GLYPH_LABEL_ID, SPACE_CONFIG_GRAIN_ID,
    SPACE_CONFIG_ID, SPACE_CONFIG_IDENTITY_LABEL_ID, SPACE_CONFIG_NAME_ID,
    SPACE_CONFIG_PAPER_LABEL_ID, SPACE_CONFIG_THEME_LABEL_ID, SPACE_CONFIG_TITLE_ID,
};
pub use model::SpaceConfigModel;
pub use render::space_config_surface;
