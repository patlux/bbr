#![forbid(unsafe_code)]

//! Portable command-bar parsing, ranking, registry, and result contracts.
//!
//! Phase 5 owns product behavior in this crate. It deliberately has no GPUI,
//! native-engine, persistence-implementation, or platform dependencies so its
//! core can evolve in an isolated worktree before app composition.
