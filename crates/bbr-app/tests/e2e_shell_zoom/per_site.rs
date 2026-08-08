//! Per-site isolation and activation scenarios (queue item 11, part A).
//!
//! Proves the persistence semantics of the merged contract on real distinct
//! origins (two fixture servers on different loopback ports): a navigation
//! within the same origin keeps the level; a navigation to a different
//! origin uses its own stored level; returning restores the stored level.
//! Tab switching between sites on different origins re-applies each site's
//! level to the freshly shown view. Identity (view/session/profile/space)
//! is asserted at every step; zooming never composites or samples web
//! content (the level is applied through the native `pageZoom` property).

use crate::e2e_common::*;
use crate::support::*;

/// Same-origin navigation keeps the level; a different origin uses its own;
/// returning restores the stored level — all through the production
/// navigation path with the same live WebViewId and session.
#[test]
#[ignore]
fn navigation_keeps_same_site_level_and_isolates_other_sites() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("zoom-sites", &fixtures.a1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let (baseline, _view) = start(&mut client, PAGE_ONE_TITLE, since);
        let site_a = origin_site(&fixtures.server_a.base_url());
        let site_b = origin_site(&fixtures.server_b.base_url());

        // Origin A: zoom to 150 (two Cmd-Plus steps).
        zoom_in_key(&mut client, 125, &site_a, &baseline);
        zoom_in_key(&mut client, 150, &site_a, &baseline);

        // Same-origin navigation (A page 1 → A page 2): the level survives.
        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: fixtures.a2.clone(),
            })
            .expect("navigate within site A");
        let after = wait_url(&mut client, &fixtures.a2, deadline());
        assert_zoom_at(&after, 150, &site_a, &baseline);
        assert_identity(&after, &baseline);

        // Cross-origin navigation (A → B): B uses its own level (default).
        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: fixtures.b1.clone(),
            })
            .expect("navigate to site B");
        let after_b = wait_url(&mut client, &fixtures.b1, deadline());
        assert_zoom_at(&after_b, ZOOM_DEFAULT, &site_b, &baseline);

        // B keeps its own level after a Cmd-Plus (125), independent of A.
        zoom_in_key(&mut client, 125, &site_b, &baseline);

        // Back to A: the stored 150 is restored and applied to the same view.
        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: fixtures.a1.clone(),
            })
            .expect("navigate back to site A");
        let back = wait_url(&mut client, &fixtures.a1, deadline());
        assert_zoom_at(&back, 150, &site_a, &baseline);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Tab switching between two sites on different origins: each site's stored
/// level is applied to the freshly shown view on activation, sessions stay
/// live, and the WebViewId/profile/space identity is preserved per tab.
#[test]
#[ignore]
fn tab_switch_reactivates_each_sites_level_and_preserves_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("zoom-tabs", &fixtures.a1);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let (baseline, first_view) = start(&mut client, PAGE_ONE_TITLE, since);
        let site_a = origin_site(&fixtures.server_a.base_url());
        let site_b = origin_site(&fixtures.server_b.base_url());

        // Tab 0 on origin A: zoom to 150.
        zoom_in_key(&mut client, 125, &site_a, &baseline);
        zoom_in_key(&mut client, 150, &site_a, &baseline);

        // Tab 1 on origin B: its own level (125 after one step).
        client
            .send_ok(&dsl::open_tab(&fixtures.b1))
            .expect("open tab on site B");
        let opened = client.snapshot().expect("tab B snapshot");
        let second_view = opened.active_web_view.expect("second view");
        assert_ne!(second_view, first_view, "a fresh session backs tab B");
        let _ = wait_fixture_loaded(&mut client, second_view, PAGE_ONE_TITLE, since);
        let b_baseline = identity_baseline(&opened);
        assert_eq!(b_baseline.sessions, baseline.sessions + 1);
        assert_eq!(b_baseline.profile, baseline.profile, "same profile");
        assert_eq!(b_baseline.space, baseline.space, "same space");
        // Tab A's identity after tab B opened: the same live WebViewId and
        // space, now alongside B's session (2 sessions total).
        let a_baseline = IdentityBaseline {
            sessions: b_baseline.sessions,
            ..baseline
        };
        zoom_in_key(&mut client, 125, &site_b, &b_baseline);

        // Switch back to tab 0: the view re-applies origin A's 150 (the
        // hidden view never presents a stale level), and the same WebViewId
        // is still live (tab B's session stays open).
        client.send_ok(&dsl::select_tab(0)).expect("select tab A");
        let back = client.snapshot().expect("tab A snapshot");
        assert_eq!(back.active_web_view, Some(first_view), "A's view is live");
        assert_zoom_at(&back, 150, &site_a, &a_baseline);

        // Switch to tab 1: origin B's 125 is applied to B's view.
        client.send_ok(&dsl::select_tab(1)).expect("select tab B");
        let b2 = client.snapshot().expect("tab B snapshot");
        assert_eq!(b2.active_web_view, Some(second_view), "B's view is live");
        assert_zoom_at(&b2, 125, &site_b, &b_baseline);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
