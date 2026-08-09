//! App-local shell presentation, space, and snapshot modules.
//!
//! Product state lives in `bbr-browser::BrowserController`; these modules own
//! presentation-only derivation (the §11.3 space grouping, per-row visual
//! states per §10, and the §15 semantic ids of the current render) and the
//! app-local typed E2E snapshot adapter. The shared title/URL fallbacks,
//! separator-aware truncation, and favicon monograms moved to `bbr-ui::text`
//! in the G1a cutover; the app imports them from there.
//!
//! - [`presentation`] — the status line and the empty-pane profile line.
//! - [`tab`] — the vertical tab row presentation (one-line budgets, row
//!   states, the pinned/regular boundary).
//! - [`space`] — the profile-grouped space surface (§11.3) and its budgets.
//! - [`semantics`] — the typed §15 semantic tree (roles/names/values/
//!   enabled/selected/focused/hierarchy).
//! - [`snapshot`] — the testkit `TestSnapshot` and `visible_ids`.

pub mod presentation;
#[cfg(feature = "testkit")]
pub mod semantics;
pub mod snapshot;
pub mod space;
pub mod tab;

pub use presentation::profile_line;
pub use presentation::status_line;
#[cfg(feature = "testkit")]
pub use snapshot::TestSnapshot;
pub use space::space_groups;
pub use space::space_title_budget_chars;
pub use tab::{pinned_count, row_state, title_budget_chars};
