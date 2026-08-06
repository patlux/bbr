//! Provider-neutral stable identities for command-bar results.
//!
//! `bbr-command` is deliberately dependency-free, so it cannot reuse
//! `bbr-domain` or `bbr-web` identity types. The newtypes below mirror the
//! same `u64` shape as the domain [`SpaceId`]/[`TabId`] and the engine
//! `ProfileId`; the app-composition seam maps between them when it builds the
//! owned corpus. The u64 payload is opaque: never interpret it beyond
//! equality/ordering in this crate.

/// Stable identifier of a browsing profile (mirrors `bbr_web::ProfileId`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId(pub u64);

/// Stable identifier of a workspace attached to one profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpaceId(pub u64);

/// Stable identifier of a persistent page identity inside one space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TabId(pub u64);

/// Stable identifier of one local history record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HistoryId(pub u64);

/// Stable identifier of a registered browser command action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub u64);
