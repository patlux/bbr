//! App-local Chassis components.
//!
//! After the G1a Foundations/Primitives cutover, the app keeps exactly one
//! component module: the §11.3/§11.4 permanent **space strip** composition
//! (`space_strip`), which the G3 composition row moves to `bbr-ui` in a
//! later wave. Every other §11 primitive (favicon tiles, tab rows, profile
//! chips, panes, sections, favorites, folders, footer, menu, address pill)
//! now lives in `bbr-ui` and is imported from there — there are no copies
//! and no `pub use bbr_ui` re-export aliases in this crate.

pub mod space_strip;

pub use space_strip::{space_chip, space_strip};
#[cfg(feature = "testkit")]
pub use space_strip::{space_chip_id, space_identity_id};
