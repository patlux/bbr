//! Which website-data store a profile's web views use.
//!
//! Production gives every profile its own identifier-backed persistent
//! store (`42425200-0000-4000-…`, derived from the `ProfileId`). The
//! hermetic E2E default is a non-persistent store per profile. The
//! reality-gaps packet (item F) adds a third kind for E2E runs that must
//! prove persistence: identifier-backed stores in a per-run test namespace
//! (`42425e57-<run>-4000-…`), which never collide with production
//! identifiers and which the harness removes when the run ends.

use bbr_web::ProfileId;

/// The kind of store a profile gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoreKind {
    /// The profile's persistent store (production).
    #[default]
    Persistent,
    /// A non-persistent store per profile (hermetic E2E).
    NonPersistent,
    /// A persistent store in the namespace of one test run.
    TestRun(u16),
}

/// The identifier prefix of every test-run store.
pub const TEST_STORE_PREFIX: &str = "42425e57-";

/// The store identifier of `profile` (a UUID string, lowercase), or `None`
/// for a non-persistent store.
pub fn store_identifier(kind: StoreKind, profile: ProfileId) -> Option<String> {
    let upper = (profile.0 >> 48) as u16;
    let lower = profile.0 & 0x0000_ffff_ffff_ffff;
    match kind {
        StoreKind::NonPersistent => None,
        StoreKind::Persistent => Some(format!("42425200-0000-4000-{upper:04x}-{lower:012x}")),
        StoreKind::TestRun(run) => Some(format!(
            "{TEST_STORE_PREFIX}{run:04x}-4000-{upper:04x}-{lower:012x}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_identifiers_are_stable_and_unique() {
        assert_eq!(
            store_identifier(StoreKind::Persistent, ProfileId(1)).as_deref(),
            Some("42425200-0000-4000-0000-000000000001")
        );
        assert_eq!(
            store_identifier(StoreKind::Persistent, ProfileId(u64::MAX)).as_deref(),
            Some("42425200-0000-4000-ffff-ffffffffffff")
        );
        assert_ne!(
            store_identifier(StoreKind::Persistent, ProfileId(1)),
            store_identifier(StoreKind::Persistent, ProfileId(2))
        );
    }

    #[test]
    fn a_test_run_has_its_own_namespace_and_non_persistent_has_none() {
        let id = store_identifier(StoreKind::TestRun(0xbeef), ProfileId(2)).unwrap();
        assert_eq!(id, "42425e57-beef-4000-0000-000000000002");
        assert!(id.starts_with(TEST_STORE_PREFIX));
        assert!(!store_identifier(StoreKind::Persistent, ProfileId(2))
            .unwrap()
            .starts_with(TEST_STORE_PREFIX));
        assert_eq!(
            store_identifier(StoreKind::NonPersistent, ProfileId(2)),
            None
        );
    }
}
