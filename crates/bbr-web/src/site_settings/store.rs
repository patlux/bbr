//! # Per-site settings store port
//!
//! [`SiteSettingsStore`] is the portable boundary between the per-site
//! settings lifecycle and any persistence backend. It is deliberately tiny
//! and object-safe: `get`/`put`/`remove` addressed by [`ProfileSiteKey`].
//! A repository implementation (SQLite, file store) and its schema are later
//! slices; this packet only defines the seam.
//!
//! ## Contract for implementors
//!
//! - `get` returns `Ok(None)` when no record exists; an absent record is
//!   never a failure.
//! - `put` replaces the whole sparse record for the key. The lifecycle never
//!   persists an empty record or the default zoom value, so a backend may
//!   reject those with [`StoreError::Rejected`].
//! - `remove` deletes the record; removing an absent key succeeds.
//! - Operations never cross profiles: the key always carries the mandatory
//!   [`ProfileId`](crate::ProfileId).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use crate::site_settings::{ProfileSiteKey, SiteSettingsOverrides};

/// Portable persistence boundary for per-site settings.
///
/// Object-safe by construction (no generic methods, no `Self` in argument
/// position), so a native adapter can own a `Box<dyn SiteSettingsStore>` or
/// pass `&mut dyn SiteSettingsStore` into the lifecycle.
pub trait SiteSettingsStore {
    /// Read the stored override record, or `None` when no record exists.
    fn get(&mut self, key: &ProfileSiteKey) -> Result<Option<SiteSettingsOverrides>, StoreError>;

    /// Replace the stored override record for the key.
    fn put(
        &mut self,
        key: &ProfileSiteKey,
        overrides: SiteSettingsOverrides,
    ) -> Result<(), StoreError>;

    /// Delete the stored override record for the key; succeeds when absent.
    fn remove(&mut self, key: &ProfileSiteKey) -> Result<(), StoreError>;
}

/// Typed failure of a store operation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The backend could not complete the operation (I/O, lock,
    /// transaction, unavailable repository).
    Unavailable(String),
    /// The backend rejected the record or key as invalid.
    Rejected(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "settings store unavailable: {message}"),
            Self::Rejected(message) => write!(f, "settings store rejected record: {message}"),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::collections::BTreeMap;

    /// One recorded store call, for deterministic op assertions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Op {
        Get,
        Put,
        Remove,
    }

    /// Deterministic in-memory [`SiteSettingsStore`] with an operation log
    /// and injectable per-call failures.
    #[derive(Debug, Default)]
    pub struct FakeStore {
        pub records: BTreeMap<ProfileSiteKey, SiteSettingsOverrides>,
        pub ops: Vec<(Op, ProfileSiteKey)>,
        pub fail_get: Option<StoreError>,
        pub fail_put: Option<StoreError>,
        pub fail_remove: Option<StoreError>,
    }

    impl FakeStore {
        pub fn new() -> Self {
            Self::default()
        }

        /// A store pre-seeded with one override record.
        pub fn with_record(key: ProfileSiteKey, overrides: SiteSettingsOverrides) -> Self {
            let mut store = Self::new();
            store.records.insert(key, overrides);
            store
        }

        /// The exact sequence of operations performed so far.
        pub fn op_log(&self) -> Vec<(Op, ProfileSiteKey)> {
            self.ops.clone()
        }
    }

    impl SiteSettingsStore for FakeStore {
        fn get(
            &mut self,
            key: &ProfileSiteKey,
        ) -> Result<Option<SiteSettingsOverrides>, StoreError> {
            self.ops.push((Op::Get, key.clone()));
            if let Some(error) = &self.fail_get {
                return Err(error.clone());
            }
            Ok(self.records.get(key).copied())
        }

        fn put(
            &mut self,
            key: &ProfileSiteKey,
            overrides: SiteSettingsOverrides,
        ) -> Result<(), StoreError> {
            self.ops.push((Op::Put, key.clone()));
            if let Some(error) = &self.fail_put {
                return Err(error.clone());
            }
            self.records.insert(key.clone(), overrides);
            Ok(())
        }

        fn remove(&mut self, key: &ProfileSiteKey) -> Result<(), StoreError> {
            self.ops.push((Op::Remove, key.clone()));
            if let Some(error) = &self.fail_remove {
                return Err(error.clone());
            }
            self.records.remove(key);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::site_settings::ZoomPercent;

        fn key() -> ProfileSiteKey {
            ProfileSiteKey::new(
                crate::ProfileId(1),
                crate::site_settings::SiteKey::parse("https://example.com").unwrap(),
            )
        }

        fn overrides() -> SiteSettingsOverrides {
            SiteSettingsOverrides::from_zoom(ZoomPercent::new(150).unwrap())
        }

        #[test]
        fn get_returns_none_for_absent_record() {
            let mut store = FakeStore::new();
            assert_eq!(store.get(&key()), Ok(None));
            assert_eq!(store.op_log(), vec![(Op::Get, key())]);
        }

        #[test]
        fn put_then_get_round_trips_the_record() {
            let mut store = FakeStore::new();
            store.put(&key(), overrides()).unwrap();
            assert_eq!(store.get(&key()), Ok(Some(overrides())));
            assert_eq!(store.op_log(), vec![(Op::Put, key()), (Op::Get, key())]);
        }

        #[test]
        fn remove_deletes_and_is_idempotent() {
            let mut store = FakeStore::with_record(key(), overrides());
            store.remove(&key()).unwrap();
            assert_eq!(store.get(&key()), Ok(None));
            // Removing an absent key succeeds and is recorded.
            store.remove(&key()).unwrap();
            assert_eq!(
                store.op_log(),
                vec![(Op::Remove, key()), (Op::Get, key()), (Op::Remove, key())]
            );
        }

        #[test]
        fn injected_failures_are_reported() {
            let mut store = FakeStore::new();
            store.fail_put = Some(StoreError::Unavailable("disk full".into()));
            assert_eq!(
                store.put(&key(), overrides()),
                Err(StoreError::Unavailable("disk full".into()))
            );
            assert!(store.records.is_empty(), "failed put must not store");
        }

        #[test]
        fn records_are_isolated_by_key() {
            let other = ProfileSiteKey::new(
                crate::ProfileId(2),
                crate::site_settings::SiteKey::parse("https://example.com").unwrap(),
            );
            let mut store = FakeStore::with_record(key(), overrides());
            assert_eq!(store.get(&other), Ok(None));
        }
    }
}
