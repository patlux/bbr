//! The §11.4 floating command presentation (DESIGN.md §11.4/§11.5/§13/
//! §15) — the centered Cmd-T palette and the field-anchored Cmd-L/click
//! overlay rendered from **one plain UI model**.
//!
//! Authority boundary (G3 Command cutover, packet
//! `component-gallery-compositions.md`): `bbr-app` remains the sole
//! command-state authority — the `CommandSurface`/`CommandResultState`/
//! `CommandResultRow` (including `Activation`), the ranking state machine
//! and debounce, text/corpus/activation/focus, the native Portal
//! host/identity/placement/dismissal/backdrop, the DTO construction, and
//! every testkit/semantics/snapshot/performance authority. This module owns
//! only:
//!
//! - the plain presentation data ([`CommandModel`], [`CommandPresentation`],
//!   [`CommandResultState`], [`CommandResultRow`]);
//! - the stable floating-command ids ([`ids`]);
//! - the narrow render geometry ([`CommandRect`]);
//! - the validation needed for the rejected combinations
//!   ([`CommandModel::validate`], [`CommandPresentation::from_raw`]);
//! - the exact render-only card anatomy ([`render::command_card`]).
//!
//! The app maps its app-owned rows onto this model at render time
//! (`app/palette.rs`) and the portal root resolves the app-owned rect and
//! invokes `command_card` (`portal/probe.rs`). No activation callback
//! belongs here or in the Gallery: production keyboard/pointer activation
//! stays app/Portal-owned, and the Gallery observes presentation only with
//! fixture-built models.

pub mod geometry;
pub mod ids;
pub mod model;
mod moments;
pub mod render;

pub use geometry::CommandRect;
pub use ids::{
    OVERLAY_CURSOR_ID, OVERLAY_FADE_ELEMENT_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_STATE_EMPTY,
    OVERLAY_STATE_ERROR, OVERLAY_STATE_LOADING, OVERLAY_TEXT_ID, PALETTE_CURSOR_ID,
    PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID, PALETTE_RESULT_PREFIX, PALETTE_STATE_EMPTY,
    PALETTE_STATE_ERROR, PALETTE_STATE_LOADING, PALETTE_TEXT_ID,
};
pub use model::{
    CommandModel, CommandModelError, CommandPresentation, CommandResultRow, CommandResultState,
};
pub use render::command_card;

#[cfg(test)]
pub(crate) use render::state_id;

#[cfg(test)]
mod tests;
