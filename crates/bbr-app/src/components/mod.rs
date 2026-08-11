//! App-local Chassis components.
//!
//! After the G1a Foundations/Primitives cutover, the app keeps no component
//! module of its own: every §11 primitive (favicon tiles, tab rows, profile
//! chips, panes, sections, favorites, folders, footer, menu, address pill)
//! and the §11.3/§11.4 permanent space strip now live in `bbr-ui` and are
//! imported from there — there are no copies and no `pub use bbr_ui`
//! re-export aliases in this crate.
