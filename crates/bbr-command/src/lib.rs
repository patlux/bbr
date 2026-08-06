#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

//! # bbr-command — portable command-bar core
//!
//! The pure, dependency-free core of the Phase-5 unified command bar: typed
//! input parsing, a provider-neutral owned corpus, a deterministic command
//! registry, and ranking across the Phase-5 priority groups.
//!
//! This crate has no GPUI, AppKit/WebKit, SQLite, filesystem, socket, or
//! platform dependency and forbids `unsafe`. It never touches a render path
//! or global state, so callers may re-run ranking freely (for example behind
//! a later debounce/cancellation seam). Identity newtypes mirror the
//! domain/engine shapes and are mapped at the app-composition boundary.
//!
//! Real-app presentation E2E, keyboard focus, and command-bar GPUI are owned
//! by the later serial app-composition packet, not by this crate.

pub mod id;
pub mod parse;

pub use id::{ActionId, HistoryId, ProfileId, SpaceId, TabId};
pub use parse::{Destination, Mode, Query, QueryContext};
