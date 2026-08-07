//! App-local shell presentation, space, and snapshot modules.
//!
//! Product state lives in `bbr-browser::BrowserController`; these modules own
//! presentation-only derivation (title/URL fallbacks per DESIGN.md §11.9,
//! separator-aware one-line truncation per §4, deterministic favicon
//! monograms per §7.2, per-row visual states per §10, the §11.3 space
//! grouping, and the §15 semantic ids of the current render) and the
//! app-local typed E2E snapshot adapter.
//!
//! - [`presentation`] — shared display helpers (titles, hosts, monograms,
//!   status/empty-pane lines).
//! - [`tab`] — the vertical tab row presentation (one-line budgets, row
//!   states, the pinned/regular boundary).
//! - [`space`] — the profile-grouped space surface (§11.3) and its budgets.
//! - [`semantics`] — the typed §15 semantic tree (roles/names/values/
//!   enabled/selected/focused/hierarchy).
//! - [`snapshot`] — the testkit `TestSnapshot` and `visible_ids`.
//!
//! Everything callers reference keeps the single `crate::shell::*` path
//! through the re-exports below (testkit-only items are cfg-gated like the
//! items themselves).

pub mod presentation;
#[cfg(feature = "testkit")]
pub mod semantics;
pub mod snapshot;
pub mod space;
pub mod tab;

pub use presentation::{
    display_title, monogram_hue, monogram_letter, profile_line, status_line, truncate_title,
};
#[cfg(feature = "testkit")]
pub use snapshot::TestSnapshot;
pub use space::space_groups;
pub use space::space_title_budget_chars;
#[cfg(feature = "testkit")]
pub use space::{space_states, space_titles};
pub use tab::{pinned_count, row_state, title_budget_chars};
