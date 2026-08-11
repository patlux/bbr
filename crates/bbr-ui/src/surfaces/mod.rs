//! BBR browser surfaces (DESIGN.md §11.4 and the G2 browser-component
//! extraction table in `docs/roadmap/tasks/ui-library-anchor.md`): the
//! larger presentation surfaces rendered inside the app's native portal
//! window — the find-in-page bar first, with Downloads / Error / Auth /
//! Context menu following in their own G2 groups.
//!
//! Like [`crate::components`], every surface is a plain function taking
//! `(model: &Model, theme: &Theme, callbacks…)` with a plain `bbr-ui` DTO
//! and the same GPUI handler closures the app passes today. `bbr-ui` holds
//! no browser/session/lifecycle/portal authority: the app keeps the
//! controller, the session, the native Portal host, and the DTO building.

pub mod find;

pub use find::{count_label, find_bar, FindBarRect, FindSurface};
