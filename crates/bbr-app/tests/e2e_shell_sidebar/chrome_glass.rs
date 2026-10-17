//! Public native material and accessibility policy over a real opaque WKWebView.
use super::*;
use bbr_testkit::protocol::{Snapshot, ThemeModeOverride};

fn wait_material(client: &mut Client, enabled: bool) -> Snapshot {
    let end = deadline();
    loop {
        flush_frame(client);
        let snapshot = client.snapshot().unwrap();
        if snapshot
            .content_card
            .chrome_material
            .is_some_and(|m| m.attached && m.enabled == enabled)
        {
            return snapshot;
        }
        assert!(
            Instant::now() < end,
            "material: {:?}",
            snapshot.content_card
        );
        std::thread::yield_now();
    }
}

#[test]
#[ignore]
fn desktop_glass_is_chrome_only_and_accessibility_changes_keep_web_sessions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _) = fixtures();
    let (mut harness, socket, token) = spawn_harness("chrome-glass", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).unwrap();
        client.attach(None).unwrap();
        let initial = wait_for_window_settled(&mut client, deadline());
        let view = initial.active_web_view.unwrap();
        let sessions = initial.engine_sessions;
        for mode in [ThemeModeOverride::Light, ThemeModeOverride::Dark] {
            client
                .send_ok(&dsl::set_appearance_override(Some(mode)))
                .unwrap();
            client
                .send_ok(&dsl::set_display_accessibility(
                    Some(false),
                    Some(true),
                    Some(false),
                ))
                .unwrap();
            let snapshot = wait_material(&mut client, true);
            let material = snapshot.content_card.chrome_material.unwrap();
            assert!(material.behind_window && material.follows_activation);
            assert!(!material.window_opaque);
            assert!(
                material.card_mask_valid,
                "native mask leaves content and rail intact"
            );
            assert_eq!(material.web_view_opaque, Some(true), "website stays opaque");
            // Regression: 0.92-0.96 coverage technically installed glass but
            // left the native material invisible. A third must show through.
            assert!(
                material.tint_alpha > 0.0 && material.tint_alpha <= 0.67,
                "{mode:?}: tint coverage {} hides the native material",
                material.tint_alpha
            );
            assert!(material.size[0] > 400.0 && material.size[1] > 200.0);
            assert_view_and_sessions_preserved(&snapshot, view, sessions);
            let rect = snapshot.content_rect;
            for (contrast, transparency) in [(false, true), (true, false), (true, true)] {
                client
                    .send_ok(&dsl::set_display_accessibility(
                        Some(contrast),
                        Some(true),
                        Some(transparency),
                    ))
                    .unwrap();
                let fallback = wait_material(&mut client, false);
                let material = fallback.content_card.chrome_material.unwrap();
                assert_eq!(material.tint_alpha, 1.0);
                assert!(
                    !material.card_mask_valid,
                    "opaque fallback restores GPUI corner covers"
                );
                assert_eq!(material.reduce_transparency, transparency);
                assert_eq!(material.web_view_opaque, Some(true));
                assert_eq!(fallback.content_rect, rect);
                assert_view_and_sessions_preserved(&fallback, view, sessions);
            }
        }
        client
            .send_ok(&dsl::set_display_accessibility(
                Some(false),
                Some(true),
                Some(false),
            ))
            .unwrap();
        for _ in 0..2 {
            keystroke(&mut client, "cmd-s");
            let changed = wait_material(&mut client, true);
            assert_eq!(
                changed
                    .content_card
                    .chrome_material
                    .unwrap()
                    .web_view_opaque,
                Some(true)
            );
            assert!(
                changed
                    .content_card
                    .chrome_material
                    .unwrap()
                    .card_mask_valid
            );
            assert_view_and_sessions_preserved(&changed, view, sessions);
        }
        client.quit().unwrap();
        assert!(
            wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10)).success()
        );
    }));
    finish(result, &mut harness, &socket, &token);
}
