#![forbid(unsafe_code)]

//! Portable persistence repositories, schema contracts, and recovery policy.
//!
//! Phase 7 owns the SQLite implementation in this crate. It deliberately has
//! no GPUI, native-engine, or platform dependencies; app integration follows
//! only after repository and migration contracts are independently proven.
//!
//! SQLite is supplied through the workspace-pinned `rusqlite` dependency with
//! default features disabled and bundled SQLite enabled. The crate may use the
//! safe wrapper but must not expose SQLite connection details as product state.
