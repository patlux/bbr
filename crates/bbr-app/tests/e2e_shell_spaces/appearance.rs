//! Real-app §11.3 appearance-tint evidence (F2a appearance layer): the
//! active space's **effective appearance** (the one controller authority —
//! explicit theme record or derived profile-hue default) drives the
//! resolved chrome/chip tint, and production activation across spaces and
//! profiles changes that appearance while warm sessions, WebView ids,
//! native slot geometry, and chrome focus stay untouched.
//!
//! The suite proves the tint layer end to end over real WKWebViews:
//!
//! 1. **Explicit appearance is authoritative per space:** setting a curated
//!    theme through the production controller seam (`SetSpaceTheme` →
//!    `BrowserController::set_space_theme`) changes the chip's effective
//!    theme evidence without touching the active space.
//! 2. **Cross-space activation changes the active appearance:** the
//!    snapshot's typed `active_appearance` (theme/identity names + the
//!    resolved chrome/chip tint hex from the one central presentation
//!    model — never raw pixels) flips exactly with the active space, the
//!    selected chip carries the destination's theme, and the destination's
//!    warm WebView stays bit-identical across the switch.
//! 3. **Cross-profile activation changes the active appearance through the
//!    derived default:** a fresh profile's space resolves its theme from the
//!    profile's identity hue (violet) without any explicit record, proving
//!    the derived path is live in production, not only in unit tests.
//! 4. **Invariants:** warm-session counts, per-space WebView ids, the
//!    native content rect, the pane layout, and chrome focus are unchanged
//!    by every tint-activating switch.

use super::*;

/// §11.3 appearance-tint layer: cross-space and cross-profile activation
/// through the production chip listener changes the active appearance
/// (explicit per-space theme and derived profile-hue theme) while the
/// destination's warm session, WebView identity, native slot geometry, and
/// chrome focus stay untouched.
#[test]
#[ignore]
fn activation_changes_the_active_appearance_without_touching_sessions_or_geometry() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-appearance", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // Launch appearance: the default profile's first §12 hue (Cobalt)
        // derives the active theme deterministically; the snapshot carries
        // the resolved chrome/chip tint hex from the central light/standard
        // presentation model, so the E2E asserts the exact rendered state —
        // no pixel reads, copied palette literals or parallel render store.
        let launch = client.snapshot().expect("launch appearance snapshot");
        let appearance = launch
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.space, default_space);
        assert_eq!(
            appearance.theme, "cobalt",
            "derived from the Cobalt profile hue"
        );
        assert_eq!(
            appearance.identity, "code",
            "default curated identity glyph"
        );
        let (cobalt_chrome, cobalt_chip) = light_standard_tints(bbr_domain::SpaceTheme::Cobalt);
        assert_eq!(
            appearance.chrome_tint, cobalt_chrome,
            "cobalt light chrome wash"
        );
        assert_eq!(
            appearance.chip_tint, cobalt_chip,
            "cobalt light chip surface"
        );
        assert_eq!(launch.space_chips.len(), 1);
        assert_eq!(launch.space_chips[0].theme, "cobalt");
        assert!(launch.space_chips[0].selected);

        // A second space in the default profile (id 4), then an explicit
        // curated theme through the production controller seam: the chip's
        // effective theme flips to pine while the active appearance stays
        // cobalt (a theme record is per-space, not an activation).
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::set_space_theme(reading, "pine"))
            .expect("set pine theme");
        let themed = client.snapshot().expect("themed chip snapshot");
        let reading_chip = themed
            .space_chips
            .iter()
            .find(|c| c.space == reading)
            .expect("reading chip");
        assert_eq!(
            reading_chip.theme, "pine",
            "explicit record is authoritative"
        );
        assert_eq!(
            themed.active_appearance.as_ref().unwrap().theme,
            "cobalt",
            "the active space is untouched by a theme record elsewhere"
        );
        assert_eq!(themed.active_web_view, Some(first_view));
        assert_eq!(
            themed.engine_sessions, 1,
            "theme records never touch sessions"
        );

        // Cross-space activation (production chip listener → the same
        // controller activation the keyboard path uses): the active
        // appearance flips to pine, the selected chip carries pine, and the
        // derived chrome/chip tints change with it. The source space's warm
        // session survives hidden (engine_sessions stays 1).
        client
            .send_ok(&dsl::click_space(reading))
            .expect("activate reading");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after reading activation");
        let reading_active = client.snapshot().expect("reading active snapshot");
        let appearance = reading_active
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.space, reading);
        assert_eq!(appearance.theme, "pine", "explicit theme is authoritative");
        assert_eq!(appearance.identity, "code");
        let (pine_chrome, pine_chip) = light_standard_tints(bbr_domain::SpaceTheme::Pine);
        assert_eq!(
            appearance.chrome_tint, pine_chrome,
            "pine light chrome wash"
        );
        assert_eq!(appearance.chip_tint, pine_chip, "pine light chip surface");
        let selected = reading_active
            .space_chips
            .iter()
            .find(|c| c.selected)
            .expect("selected chip");
        assert_eq!(selected.space, reading);
        assert_eq!(
            selected.theme, "pine",
            "the selected chip carries the theme"
        );
        assert_eq!(
            reading_active.engine_sessions, 1,
            "a space switch never recreates sessions"
        );
        assert_eq!(
            reading_active
                .spaces
                .iter()
                .find(|s| s.space == default_space)
                .unwrap()
                .active_view,
            Some(first_view),
            "the source space keeps its warm WebView hidden"
        );
        assert!(
            reading_active.chrome_focused,
            "chrome focus stays in the strip"
        );

        // Give Reading a real tab: its live WebView + slot geometry become
        // the identity/geometry evidence the tint activation must preserve.
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after reading tab");
        let reading_rect = client.snapshot().expect("reading tab rect");
        assert_eq!(reading_rect.active_web_view, Some(reading_view));
        assert_eq!(reading_rect.engine_sessions, 2);
        let reading_content_rect = reading_rect.content_rect;
        let reading_pane = reading_rect.pane_layout;
        let reading_pane_rects = reading_rect.pane_rects.clone();

        // Back to Default (cobalt): the tinted chrome flips back, the warm
        // first view restores with the same WebViewId, and the native slot
        // geometry is unchanged.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("activate default");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after default activation");
        let default_active = client.snapshot().expect("default active snapshot");
        assert_eq!(
            default_active.active_appearance.as_ref().unwrap().theme,
            "cobalt"
        );
        assert_eq!(
            default_active.active_web_view,
            Some(first_view),
            "warm first view restores"
        );
        assert_eq!(default_active.engine_sessions, 2, "no session recreated");
        assert_eq!(
            default_active.content_rect, reading_content_rect,
            "the native slot geometry never moves with the tint"
        );
        // The pane layout is the focused native view's identity: after the
        // warm restore it is exactly the first view in the same single-pane
        // shape (same geometry, same view identity — never recreated).
        assert_eq!(
            default_active.pane_layout,
            Some(bbr_testkit::protocol::PaneLayoutSnapshot {
                primary: first_view,
                secondary: None,
                focused: first_view,
            }),
            "the first view stays the focused single pane"
        );
        // Keyed geometry: the default space's single-pane rect is keyed by
        // the first view's id (the native slot frame is a property of the
        // window, never of the tint).
        assert_eq!(default_active.pane_rects.len(), 1, "single-pane keyed rect");
        assert_eq!(
            default_active.pane_rects[0].view, first_view,
            "the keyed rect belongs to the warm first view"
        );
        assert!(default_active.chrome_focused);

        // Activate Reading again (pine): the same warm WebView id returns
        // and the slot geometry is byte-identical to the first visit — the
        // tint activation never touches the web/native slot.
        client
            .send_ok(&dsl::click_space(reading))
            .expect("reactivate reading");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after reading reactivation");
        let reading_again = client.snapshot().expect("reading reactivated snapshot");
        assert_eq!(
            reading_again.active_appearance.as_ref().unwrap().theme,
            "pine"
        );
        assert_eq!(
            reading_again.active_web_view,
            Some(reading_view),
            "the same WebView id returns — no recreation"
        );
        assert_eq!(reading_again.engine_sessions, 2);
        assert_eq!(
            reading_again.content_rect, reading_content_rect,
            "slot geometry identical across tint activations"
        );
        assert_eq!(reading_again.pane_layout, reading_pane);
        assert_eq!(reading_again.pane_rects, reading_pane_rects);

        // Cross-profile activation: a fresh profile's space derives its
        // theme from the profile identity hue (violet, the second §12 hue)
        // with no explicit record — the derived path is live in production.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_active = client.snapshot().expect("private active snapshot");
        let appearance = private_active
            .active_appearance
            .as_ref()
            .expect("active appearance resolves");
        assert_eq!(appearance.space, private_space);
        assert_eq!(
            appearance.theme, "violet",
            "the derived profile-hue theme is the production default"
        );
        let (violet_chrome, violet_chip) = light_standard_tints(bbr_domain::SpaceTheme::Violet);
        assert_eq!(
            appearance.chrome_tint, violet_chrome,
            "violet light chrome wash"
        );
        assert_eq!(
            appearance.chip_tint, violet_chip,
            "violet light chip surface"
        );
        assert_eq!(private_active.active_profile, private_profile);
        assert_eq!(
            private_active
                .space_chips
                .iter()
                .find(|c| c.selected)
                .unwrap()
                .theme,
            "violet",
            "the derived chip carries the profile hue theme"
        );
        assert_eq!(
            private_active.engine_sessions, 2,
            "cross-profile stays session-stable"
        );
        assert_eq!(
            private_active
                .spaces
                .iter()
                .find(|s| s.space == reading)
                .unwrap()
                .active_view,
            Some(reading_view),
            "Reading keeps its warm WebView across the profile switch"
        );
        assert!(
            private_active.chrome_focused,
            "cross-profile keeps chrome focus"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
