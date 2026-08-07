//! Contract tests for the per-site zoom lifecycle against the deterministic
//! fake store: load/change/persist/ack, supersession, stale-ack rejection,
//! default-value invariants, isolation, and object safety.

use super::*;
use crate::site_settings::store::fake::{FakeStore, Op};
use crate::site_settings::{SiteKey, ZoomPercent};
use crate::ProfileId;

fn key() -> ProfileSiteKey {
    ProfileSiteKey::new(ProfileId(1), SiteKey::parse("https://example.com").unwrap())
}

fn zoom(percent: u16) -> ZoomPercent {
    ZoomPercent::new(percent).expect("fixture must be in bounds")
}

fn stored(percent: u16) -> SiteSettingsOverrides {
    SiteSettingsOverrides::from_zoom(zoom(percent))
}

#[test]
fn loading_default_creates_no_record_and_no_pending_write() {
    let mut store = FakeStore::new();
    let lifecycle = SiteZoomLifecycle::load(key(), None);
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert_eq!(lifecycle.version(), 0);
    assert!(!lifecycle.has_pending());
    assert_eq!(lifecycle.pending_version(), None);
    assert_eq!(lifecycle.acknowledged_zoom(), None);
    // Persisting without a change must not touch the store at all.
    let mut lifecycle = lifecycle;
    assert_eq!(lifecycle.persist(&mut store), Ok(None));
    assert!(store.op_log().is_empty(), "defaults never create records");
}

#[test]
fn loading_stored_override_applies_it_without_a_pending_write() {
    let lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    assert_eq!(lifecycle.effective_zoom(), zoom(150));
    assert_eq!(lifecycle.acknowledged_zoom(), Some(zoom(150)));
    assert_eq!(lifecycle.version(), 0);
    assert!(!lifecycle.has_pending());
}

#[test]
fn zoom_in_schedules_a_set_and_persist_puts_it() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in();
    assert_eq!(lifecycle.effective_zoom(), zoom(125));
    assert_eq!(lifecycle.version(), 1);
    assert_eq!(lifecycle.pending_version(), Some(1));

    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(version, 1);
    assert_eq!(store.op_log(), vec![(Op::Put, key())]);
    assert_eq!(store.get(&key()).unwrap(), Some(stored(125)));

    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert!(!lifecycle.has_pending());
    assert_eq!(lifecycle.acknowledged_zoom(), Some(zoom(125)));
}

#[test]
fn returning_to_default_schedules_a_removal_not_a_set() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(125)));
    lifecycle.zoom_out();
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert_eq!(lifecycle.version(), 1);

    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(store.get(&key()).unwrap(), None);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert_eq!(lifecycle.acknowledged_zoom(), None);
}

#[test]
fn reset_removes_an_existing_override() {
    let mut store = FakeStore::with_record(key(), stored(300));
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(300)));
    lifecycle.reset();
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert_eq!(lifecycle.version(), 1);

    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert_eq!(lifecycle.acknowledged_zoom(), None);
    assert_eq!(store.get(&key()).unwrap(), None);
}

#[test]
fn reset_without_an_override_is_a_deterministic_no_op_removal() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.reset();
    assert_eq!(lifecycle.effective_zoom(), zoom(100));

    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    // Documented behavior: reset always schedules the removal; removing an
    // absent record is an idempotent no-op for the backend.
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert_eq!(lifecycle.acknowledged_zoom(), None);
}

#[test]
fn set_zoom_persists_an_explicit_value() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.set_zoom(zoom(200));
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.get(&key()).unwrap(), Some(stored(200)));
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
}

#[test]
fn setting_the_default_value_never_issues_a_set() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    // Explicitly setting 100% is a default value: the record is removed.
    lifecycle.set_zoom(ZoomPercent::default());
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert_eq!(lifecycle.acknowledged_zoom(), None);
}

#[test]
fn later_change_supersedes_an_earlier_pending_write() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in(); // v1: Set 125
    lifecycle.zoom_in(); // v2: Set 150 supersedes v1
    assert_eq!(lifecycle.pending_version(), Some(2));
    assert_eq!(lifecycle.effective_zoom(), zoom(150));

    // Only the latest intent is written.
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(version, 2);
    assert_eq!(store.op_log(), vec![(Op::Put, key())]);
    assert_eq!(store.get(&key()).unwrap(), Some(stored(150)));
}

#[test]
fn stale_ack_never_mutates_newer_state() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in(); // v1
    lifecycle.zoom_in(); // v2 supersedes v1
    assert_eq!(lifecycle.version(), 2);

    // Ack for the superseded v1 write arrives late.
    assert_eq!(lifecycle.ack(1), AckResult::Stale { version: 1 });
    assert_eq!(lifecycle.effective_zoom(), zoom(150), "effective untouched");
    assert_eq!(lifecycle.acknowledged_zoom(), None, "acked value untouched");
    assert_eq!(lifecycle.pending_version(), Some(2), "pending write kept");

    // The store must still receive and acknowledge the latest write.
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(version, 2);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 2 });
    assert_eq!(lifecycle.acknowledged_zoom(), Some(zoom(150)));
}

#[test]
fn ack_without_outstanding_write_is_mismatched() {
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    assert_eq!(lifecycle.ack(0), AckResult::Mismatched { version: 0 });
    assert_eq!(lifecycle.ack(1), AckResult::Mismatched { version: 1 });
    assert!(!lifecycle.has_pending());
}

#[test]
fn ack_of_a_future_version_is_mismatched_and_keeps_pending() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in(); // v1
    assert_eq!(lifecycle.ack(2), AckResult::Mismatched { version: 2 });
    assert_eq!(lifecycle.pending_version(), Some(1));
    assert_eq!(lifecycle.acknowledged_zoom(), None);

    // The real write is still applied afterwards.
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(version, 1);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
}

#[test]
fn ack_of_an_already_applied_version_is_mismatched() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in();
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    // The write is no longer outstanding: a duplicate ack is unmatched.
    assert_eq!(lifecycle.ack(1), AckResult::Mismatched { version: 1 });
}

#[test]
fn persist_failure_keeps_state_and_allows_retry() {
    let mut store = FakeStore::new();
    store.fail_put = Some(StoreError::Unavailable("disk full".into()));
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in();

    assert_eq!(
        lifecycle.persist(&mut store),
        Err(StoreError::Unavailable("disk full".into()))
    );
    assert_eq!(lifecycle.effective_zoom(), zoom(125), "effective kept");
    assert_eq!(
        lifecycle.pending_version(),
        Some(1),
        "pending kept for retry"
    );

    // The backend recovers; the same pending write succeeds.
    store.fail_put = None;
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(version, 1);
    assert_eq!(store.get(&key()).unwrap(), Some(stored(125)));
}

#[test]
fn clear_override_removes_now_and_cancels_pending() {
    let mut store = FakeStore::with_record(key(), stored(150));
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    lifecycle.zoom_in(); // pending Set 175
    assert_eq!(lifecycle.pending_version(), Some(1));

    lifecycle.clear_override(&mut store).unwrap();
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(store.get(&key()).unwrap(), None);
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert_eq!(lifecycle.acknowledged_zoom(), None);
    assert!(!lifecycle.has_pending());
    // Nothing remains to write or acknowledge.
    assert_eq!(lifecycle.persist(&mut store), Ok(None));
    assert_eq!(lifecycle.ack(1), AckResult::Mismatched { version: 1 });
}

#[test]
fn clear_override_failure_leaves_everything_untouched() {
    let mut store = FakeStore::with_record(key(), stored(150));
    store.fail_remove = Some(StoreError::Unavailable("locked".into()));
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    lifecycle.zoom_in();

    assert_eq!(
        lifecycle.clear_override(&mut store),
        Err(StoreError::Unavailable("locked".into()))
    );
    assert_eq!(lifecycle.effective_zoom(), zoom(175), "effective untouched");
    assert_eq!(lifecycle.pending_version(), Some(1), "pending untouched");
    assert_eq!(lifecycle.acknowledged_zoom(), Some(zoom(150)));
    assert_eq!(store.get(&key()).unwrap(), Some(stored(150)), "record kept");
}

#[test]
fn clear_discards_pending_and_reverts_to_acknowledged_state() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    lifecycle.zoom_out(); // pending Set 125
    assert_eq!(lifecycle.effective_zoom(), zoom(125));

    lifecycle.clear();
    assert_eq!(lifecycle.effective_zoom(), zoom(150), "reverts to acked");
    assert!(!lifecycle.has_pending());
    assert_eq!(lifecycle.persist(&mut store), Ok(None), "no store write");
    assert!(store.op_log().is_empty());
}

#[test]
fn clear_without_any_override_reverts_to_default() {
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in();
    lifecycle.clear();
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert!(!lifecycle.has_pending());
}

#[test]
fn bound_clamping_still_schedules_deterministic_writes() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(400)));
    lifecycle.zoom_in(); // clamped at max: value unchanged, write still scheduled
    assert_eq!(lifecycle.effective_zoom(), zoom(400));
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Put, key())]);
    assert_eq!(store.get(&key()).unwrap(), Some(stored(400)));
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });

    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(25)));
    lifecycle.zoom_out(); // clamped at min
    assert_eq!(lifecycle.effective_zoom(), zoom(25));
    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Put, key())]);
    assert_eq!(store.get(&key()).unwrap(), Some(stored(25)));
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
}

#[test]
fn profile_and_site_isolation_is_exact_per_key() {
    let same_site_other_profile =
        ProfileSiteKey::new(ProfileId(2), SiteKey::parse("https://example.com").unwrap());
    let same_profile_other_site = ProfileSiteKey::new(
        ProfileId(1),
        SiteKey::parse("https://other.example").unwrap(),
    );

    let mut store = FakeStore::new();
    let mut a = SiteZoomLifecycle::load(key(), None);
    let mut b = SiteZoomLifecycle::load(same_site_other_profile.clone(), None);
    let mut c = SiteZoomLifecycle::load(same_profile_other_site.clone(), None);

    a.zoom_in(); // v1 Set 125 for key A
    b.set_zoom(zoom(300));
    c.reset();

    let a_version = a.persist(&mut store).unwrap().unwrap();
    let b_version = b.persist(&mut store).unwrap().unwrap();
    let c_version = c.persist(&mut store).unwrap().unwrap();

    // Every command carries its own profile+site key.
    assert_eq!(
        store.op_log(),
        vec![
            (Op::Put, key()),
            (Op::Put, same_site_other_profile.clone()),
            (Op::Remove, same_profile_other_site.clone()),
        ]
    );
    assert_eq!(a.ack(a_version), AckResult::Applied { version: a_version });
    assert_eq!(b.ack(b_version), AckResult::Applied { version: b_version });
    assert_eq!(c.ack(c_version), AckResult::Applied { version: c_version });

    // Records stay isolated: each lifecycle only ever sees its own key.
    assert_eq!(store.get(&key()).unwrap(), Some(stored(125)));
    assert_eq!(
        store.get(&same_site_other_profile).unwrap(),
        Some(stored(300))
    );
    assert_eq!(store.get(&same_profile_other_site).unwrap(), None);
}

/// Persist through an object-safe `&mut dyn` handle (the seam a native
/// adapter would use).
fn persist_through_dyn(
    store: &mut dyn SiteSettingsStore,
    lifecycle: &mut SiteZoomLifecycle,
) -> Result<Option<u64>, StoreError> {
    lifecycle.persist(store)
}

#[test]
fn store_and_lifecycle_are_object_safe() {
    let mut store: Box<dyn SiteSettingsStore> = Box::new(FakeStore::new());
    let mut lifecycle = SiteZoomLifecycle::load(key(), None);
    lifecycle.zoom_in();

    // Both the store and the lifecycle can be driven through trait objects.
    let version = persist_through_dyn(store.as_mut(), &mut lifecycle)
        .unwrap()
        .unwrap();
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 1 });
    assert_eq!(store.get(&key()).unwrap(), Some(stored(125)));

    // Reset and clear through the boxed handle too.
    lifecycle.zoom_out();
    let version = persist_through_dyn(store.as_mut(), &mut lifecycle)
        .unwrap()
        .unwrap();
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 2 });
    assert_eq!(store.get(&key()).unwrap(), None);
}

#[test]
fn zoom_out_from_loaded_override_then_reset_chain_is_deterministic() {
    let mut store = FakeStore::new();
    let mut lifecycle = SiteZoomLifecycle::load(key(), Some(stored(300)));
    lifecycle.zoom_out(); // 275
    lifecycle.zoom_out(); // 250
    lifecycle.reset(); // 100 → Remove, supersedes both writes
    assert_eq!(lifecycle.effective_zoom(), zoom(100));
    assert_eq!(lifecycle.version(), 3);
    assert_eq!(lifecycle.pending_version(), Some(3));

    let version = lifecycle.persist(&mut store).unwrap().unwrap();
    assert_eq!(store.op_log(), vec![(Op::Remove, key())]);
    assert_eq!(lifecycle.ack(version), AckResult::Applied { version: 3 });
    assert_eq!(lifecycle.acknowledged_zoom(), None);
}

#[test]
fn debug_shape_is_exact() {
    let lifecycle = SiteZoomLifecycle::load(key(), Some(stored(150)));
    assert_eq!(
        format!("{lifecycle:?}"),
        "SiteZoomLifecycle { key: ProfileSiteKey { profile: ProfileId(1), site: SiteKey { scheme: Https, host: \"example.com\", port: 443 } }, effective: ZoomPercent(150), acknowledged: Some(ZoomPercent(150)), version: 0, pending: None }"
    );
}
