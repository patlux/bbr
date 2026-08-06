#![forbid(unsafe_code)]

//! Portable persistence repositories, schema contracts, and recovery policy.
//!
//! Phase 7 owns the SQLite implementation in this crate. It deliberately has
//! no GPUI, native-engine, or platform dependencies; app integration follows
//! only after repository and migration contracts are independently proven.
