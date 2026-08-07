//! Chrome-first hydration policy contract tests.
//!
//! These tests prove the portable policy's core invariants through the real
//! production `HydrationPolicy::plan`/`apply` path against a recording
//! `RestoreTarget` backed by the deterministic fake engine:
//!
//! - identities are restored first, sessions only afterwards;
//! - sessions are created **only** for the active space's visible set
//!   (selected tab, or both split panes) — never one per space, never one
//!   per tab;
//! - a dense 500-tab intent plans and executes exactly one session (the
//!   "no eager session explosion" phase-7 exit);
//! - every validation rejection is typed and deterministic.
//!
//! Layout: `support` holds the intent builders, the dense fixture, and the
//! recording target; `planning` covers plan-shape and visibility;
//! `validation` covers the typed rejection contract; `apply` covers the
//! plan-before-execute and stop-on-error execution contract.

mod apply;
mod planning;
mod support;
mod validation;
