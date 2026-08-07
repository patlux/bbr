//! Portable profile-scoped per-site settings and zoom contracts.
//!
//! This module defines the stable identity, value, and lifecycle vocabulary
//! that later slices build persistence, native zoom application, and
//! settings UI on:
//!
//! - [`SiteKey`]: a validated, normalized origin (`scheme://host:port`) with
//!   exact scheme/host/port semantics that need no registrable-domain or
//!   Public Suffix List knowledge.
//! - [`ProfileSiteKey`]: a [`SiteKey`] bound to a [`crate::ProfileId`]; every
//!   stored setting is keyed by profile + site and never crosses profiles.
//! - [`ZoomPercent`]: a drift-free integer-percent zoom level with documented
//!   bounds and deterministic in/out/reset behavior.
//! - [`SiteSettingsOverrides`]: the sparse persisted record per site (only
//!   explicitly changed values); [`EffectiveSiteSettings`] resolves it
//!   against documented defaults.
//! - [`SiteZoomLifecycle`]: the deterministic zoom state machine — load,
//!   change, persist, reset, clear, supersede, and stale-ack rejection —
//!   driving the portable [`SiteSettingsStore`] port (get/put/remove).
//!
//! Persistence repositories, the WebKit zoom mapping, settings UI, testkit,
//! and real-app E2E belong to later slices and are not part of this module.

pub mod lifecycle;
pub mod overrides;
pub mod site_key;
pub mod store;
pub mod zoom;

pub use lifecycle::{AckResult, SiteZoomLifecycle};
pub use overrides::{EffectiveSiteSettings, SiteSettingsOverrides};
pub use site_key::{ProfileSiteKey, Scheme, SiteKey, SiteKeyError};
pub use store::{SiteSettingsStore, StoreError};
pub use zoom::{ZoomError, ZoomPercent};

#[cfg(all(test, feature = "serde"))]
mod serde_tests;
#[cfg(all(test, feature = "serde"))]
mod wire;
