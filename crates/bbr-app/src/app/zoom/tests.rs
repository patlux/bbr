//! Unit tests for the per-site zoom composition (queue item 11, part A).

use super::*;

fn url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{port}{path}")
}

fn controller_with_log() -> (ZoomController, TargetLog) {
    let (target, log) = recording_target();
    (ZoomController::new(target), log)
}

#[test]
fn site_key_resolution_ignores_path_and_normalizes_origin() {
    let a = ZoomController::resolve_key(ProfileId(1), "http://127.0.0.1:8123/page1.html");
    let b = ZoomController::resolve_key(ProfileId(1), "http://127.0.0.1:8123/other?x=1");
    assert_eq!(a, b);
    assert_eq!(
        a.as_ref().map(|k| k.to_string()),
        Some("profile=1 site=http://127.0.0.1:8123".into())
    );
    // Port is part of the identity: a different port is a different site.
    let c = ZoomController::resolve_key(ProfileId(1), "http://127.0.0.1:9999/");
    assert_ne!(a, c);
    // Profile is part of the identity.
    let d = ZoomController::resolve_key(ProfileId(2), "http://127.0.0.1:8123/");
    assert_ne!(a, d);
}

#[test]
fn unresolvable_urls_are_no_ops() {
    assert_eq!(ZoomController::resolve_key(ProfileId(1), ""), None);
    assert_eq!(
        ZoomController::resolve_key(ProfileId(1), "about:blank"),
        None
    );
    assert_eq!(ZoomController::resolve_key(ProfileId(1), "not-a-url"), None);
}

#[test]
fn zoom_in_steps_the_contract_ladder_and_applies_natively() {
    let (mut zoom, log) = controller_with_log();
    let view = WebViewId(1);
    let level = zoom.zoom_in(view, ProfileId(1), &url(8123, "/page1.html"));
    assert_eq!(level, Some(125));
    assert_eq!(zoom.snapshot(view).level, 125);
    assert_eq!(*log.borrow(), vec![(view, ZoomPercent::new(125).unwrap())]);
    // A second step reaches 150 and the override is persisted (the store
    // now holds 150 for this site).
    assert_eq!(
        zoom.zoom_in(view, ProfileId(1), &url(8123, "/page2.html")),
        Some(150)
    );
    assert_eq!(zoom.snapshot(view).level, 150);
    assert!(!zoom.snapshot(view).pending);
    assert_eq!(
        zoom.store
            .get()
            .get(&ZoomController::resolve_key(ProfileId(1), &url(8123, "/")).unwrap())
            .unwrap()
            .and_then(SiteSettingsOverrides::zoom)
            .map(ZoomPercent::as_u16),
        Some(150)
    );
}

#[test]
fn clamp_at_both_ends_is_deterministic() {
    let (mut zoom, log) = controller_with_log();
    let view = WebViewId(1);
    let mut level = 100;
    for _ in 0..13 {
        level = zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    }
    assert_eq!(level, 400);
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(400));
    assert_eq!(zoom.snapshot(view).level, 400);
    for _ in 0..15 {
        level = zoom.zoom_out(view, ProfileId(1), &url(8123, "/")).unwrap();
    }
    assert_eq!(level, 25);
    assert_eq!(zoom.zoom_out(view, ProfileId(1), &url(8123, "/")), Some(25));
    assert_eq!(zoom.snapshot(view).level, 25);
    // 400→25 is exactly 15 steps of 25 and never drifts: 13 in-calls
    // (the 13th clamps at 400) + 1 clamped in + 15 out-calls + 1
    // clamped out = 30 native applications, none redundant.
    assert_eq!(log.borrow().len(), 30);
    assert_eq!(
        log.borrow().last(),
        Some(&(view, ZoomPercent::new(25).unwrap()))
    );
}

#[test]
fn reset_returns_default_and_removes_the_override() {
    let (mut zoom, _log) = controller_with_log();
    let view = WebViewId(1);
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    assert_eq!(zoom.snapshot(view).level, 150);
    assert_eq!(
        zoom.zoom_reset(view, ProfileId(1), &url(8123, "/")),
        Some(100)
    );
    assert_eq!(zoom.snapshot(view).level, 100);
    assert_eq!(zoom.snapshot(view).applied, Some(100));
    assert!(!zoom.snapshot(view).pending);
    // The override record is gone (reset removed it).
    assert_eq!(
        zoom.store
            .get()
            .get(&ZoomController::resolve_key(ProfileId(1), &url(8123, "/")).unwrap())
            .unwrap(),
        None
    );
}

#[test]
fn per_site_levels_are_isolated() {
    let (mut zoom, _log) = controller_with_log();
    let view = WebViewId(1);
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(125));
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(150));
    // A different origin keeps its own level (the default).
    assert_eq!(zoom.zoom_out(view, ProfileId(1), &url(9999, "/")), Some(75));
    // Back to the first origin: its 150 is restored, not the other site's.
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(175));
}

#[test]
fn navigation_within_same_site_keeps_the_level() {
    let (mut zoom, log) = controller_with_log();
    let view = WebViewId(1);
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/page1.html"))
        .unwrap();
    zoom.on_navigation(view, ProfileId(1), &url(8123, "/page2.html"));
    assert_eq!(zoom.snapshot(view).level, 125);
    assert_eq!(
        zoom.snapshot(view).site.as_deref(),
        Some("profile=1 site=http://127.0.0.1:8123")
    );
    // Same-site navigation applied the level once (on the command); the
    // navigation itself is a no-op re-apply.
    assert_eq!(log.borrow().len(), 1);
}

#[test]
fn navigation_to_another_site_uses_its_own_level() {
    let (mut zoom, log) = controller_with_log();
    let view = WebViewId(1);
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    // Navigate to the other origin: its level is the default 100.
    zoom.on_navigation(view, ProfileId(1), &url(9999, "/page1.html"));
    assert_eq!(zoom.snapshot(view).level, 100);
    assert_eq!(zoom.snapshot(view).applied, Some(100));
    // Navigate back: the stored 150 is restored and applied to the view.
    zoom.on_navigation(view, ProfileId(1), &url(8123, "/page1.html"));
    assert_eq!(zoom.snapshot(view).level, 150);
    assert_eq!(zoom.snapshot(view).applied, Some(150));
    assert_eq!(log.borrow().len(), 4);
}

#[test]
fn fresh_view_restores_the_stored_level_on_load_and_activation() {
    let (mut zoom, log) = controller_with_log();
    zoom.zoom_in(WebViewId(1), ProfileId(1), &url(8123, "/"))
        .unwrap();
    // A brand-new view for the same site (tab recreation): its load
    // restores the stored level.
    zoom.on_navigation(WebViewId(2), ProfileId(1), &url(8123, "/page1.html"));
    assert_eq!(zoom.snapshot(WebViewId(2)).applied, Some(125));
    // Activation of a hidden view that missed a later change re-applies
    // the current level.
    zoom.zoom_in(WebViewId(2), ProfileId(1), &url(8123, "/"))
        .unwrap();
    zoom.on_activated(WebViewId(3), ProfileId(1), &url(8123, "/"));
    assert_eq!(zoom.snapshot(WebViewId(3)).applied, Some(150));
    assert_eq!(log.borrow().len(), 4);
}

#[test]
fn defaults_never_create_records() {
    let (mut zoom, _log) = controller_with_log();
    let view = WebViewId(1);
    // Loading a site with no record performs no store write.
    zoom.on_navigation(view, ProfileId(1), &url(8123, "/"));
    assert!(zoom.store.get().records.is_empty());
    assert_eq!(zoom.snapshot(view).level, 100);
    // Zoom in then back to the default via reset: the record is removed,
    // never set to the default value.
    zoom.zoom_in(view, ProfileId(1), &url(8123, "/")).unwrap();
    assert!(!zoom.store.get().records.is_empty());
    zoom.zoom_reset(view, ProfileId(1), &url(8123, "/"))
        .unwrap();
    assert!(zoom.store.get().records.is_empty());
}

#[test]
fn profiles_are_isolated() {
    let (mut zoom, _log) = controller_with_log();
    let view = WebViewId(1);
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(125));
    // The same origin in another profile keeps the default.
    assert_eq!(zoom.zoom_out(view, ProfileId(2), &url(8123, "/")), Some(75));
    // The first profile's level is untouched.
    assert_eq!(zoom.zoom_in(view, ProfileId(1), &url(8123, "/")), Some(150));
}

#[test]
fn memory_store_port_contract() {
    let mut store = MemorySiteSettingsStore::default();
    let key = ZoomController::resolve_key(ProfileId(1), "http://example.com:443").unwrap();
    let overrides = SiteSettingsOverrides::from_zoom(ZoomPercent::new(175).unwrap());
    assert_eq!(store.get(&key), Ok(None));
    store.put(&key, overrides).unwrap();
    assert_eq!(store.get(&key), Ok(Some(overrides)));
    // A record with no override field is rejected by the port (the
    // lifecycle never persists an empty record).
    let empty = SiteSettingsOverrides::from_zoom(ZoomPercent::new(100).unwrap());
    assert!(!empty.is_empty(), "from_zoom(100) is a non-default record");
    store.remove(&key).unwrap();
    assert_eq!(store.get(&key), Ok(None));
    // Removing an absent key succeeds.
    store.remove(&key).unwrap();
    // Profile isolation: the same origin under another profile is absent.
    let other = ZoomController::resolve_key(ProfileId(2), "http://example.com:443").unwrap();
    store.put(&key, overrides).unwrap();
    assert_eq!(store.get(&other), Ok(None));
}

#[test]
fn unresolvable_active_url_means_no_step_and_default_snapshot() {
    let (mut zoom, log) = controller_with_log();
    let view = WebViewId(1);
    assert_eq!(zoom.zoom_in(view, ProfileId(1), "about:blank"), None);
    assert_eq!(zoom.snapshot(view).level, 100);
    assert!(zoom.snapshot(view).site.is_none());
    assert_eq!(log.borrow().len(), 0);
}
