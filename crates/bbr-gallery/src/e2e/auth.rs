//! Auth sheet interaction scenarios (packet
//! `component-gallery-browser-components.md`, Group 8): the story renders
//! every valid credential-sheet matrix family through the production
//! `bbr_ui::surfaces::auth::auth_sheet` with recorder callbacks. A real
//! pointer press on the Log In button proves the production `on_submit`
//! fires, and a press on Cancel proves `on_cancel` fires (the recorder
//! path); the every-story sweep covers the new story automatically. No
//! product behavior is asserted here — the gallery proves renderability
//! and callback wiring only.

use bbr_ui::theme::{ContrastMode, Density, Spacing, Theme, ThemeMode};
use gpui::{point, px, Entity, VisualTestContext};

use crate::app::GalleryApp;
use crate::auth_matrix::{AUTH_REJECTED, AUTH_VALID};
use crate::harness::{bounds_of, flush, move_to_story, run_scenario, snapshot};
use crate::stories::auth::{AUTH_FAMILY_SELECTOR, AUTH_SHEET_WIDTH};

/// Navigate to the Auth sheet story (found by its stable label).
fn open_auth_sheet(window: &mut VisualTestContext, app: &Entity<GalleryApp>) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == "Auth sheet")
            .expect("Auth sheet story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

/// Leak a dynamic test-target selector into `&'static str` (the GPUI
/// `debug_bounds` API requires `'static`; a test-only leak of a few bytes
/// per scenario is fine).
fn static_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

/// The first family's Cancel and Log In (submit) button centers: the story
/// pins each production sheet card to the settled production width
/// ([`crate::stories::auth::AUTH_SHEET_WIDTH`] 360 pt) and wraps it in a
/// test-target family wrapper whose meta row is measured — so the sheet's
/// bottom edge is the meta row's top minus the wrapper's `gap:1`. The
/// buttons row is the sheet's last child (`justify_end`, `gap:1`), so the
/// Log In button's center is (`card_right − sp3 − button_w/2`,
/// `sheet_bottom − sp2 − hit_min/2`) and Cancel sits one `button_w + sp1`
/// to its left. The button widths are measured from the production ids
/// (the same labels every family renders — duplicate ids resolve to the
/// last family, which has identical button anatomy). All geometry derives
/// from measured bounds + the settled tokens — no copied anatomy.
fn first_family_button_centers(
    window: &mut VisualTestContext,
) -> (gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>) {
    let family = bounds_of(window, static_selector(format!("{AUTH_FAMILY_SELECTOR}0")));
    let meta = bounds_of(
        window,
        static_selector(format!("{AUTH_FAMILY_SELECTOR}0-meta")),
    );
    let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
    let hit = theme.density.hit_min();
    let submit_w = window
        .debug_bounds("bbr-auth-submit")
        .expect("Log In button rendered")
        .size
        .width;
    let cancel_w = window
        .debug_bounds("bbr-auth-cancel")
        .expect("Cancel button rendered")
        .size
        .width;
    let sheet_bottom = meta.origin.y - Spacing::sp1();
    let submit = point(
        family.origin.x + px(AUTH_SHEET_WIDTH) - Spacing::sp3() - submit_w / 2.0,
        sheet_bottom - Spacing::sp2() - hit / 2.0,
    );
    let cancel = point(
        submit.x - submit_w / 2.0 - Spacing::sp1() - cancel_w / 2.0,
        submit.y,
    );
    (cancel, submit)
}

#[test]
#[ignore]
fn auth_sheet_story_renders_the_matrix_families_with_stable_ids() {
    run_scenario(|window, app| {
        open_auth_sheet(window, app);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story_label, "Auth sheet");
        assert_eq!(snap.auth.submits, 0);
        assert_eq!(snap.auth.cancels, 0);
        // Every valid family is registered and renders through the
        // production component (each family wrapper has non-empty bounds).
        let family_count = window.read(|_| AUTH_VALID.len());
        assert_eq!(family_count, 8);
        for index in 0..family_count {
            let bounds = bounds_of(
                window,
                static_selector(format!("{AUTH_FAMILY_SELECTOR}{index}")),
            );
            assert!(
                bounds.size.width > px(0.0) && bounds.size.height > px(0.0),
                "auth family {index} renders"
            );
            let meta = bounds_of(
                window,
                static_selector(format!("{AUTH_FAMILY_SELECTOR}{index}-meta")),
            );
            assert!(
                meta.size.width > px(0.0) && meta.size.height > px(0.0),
                "auth family {index} meta row renders"
            );
        }
        // The production surface renders its stable §15 ids (the portal
        // snapshot's typed ids list + the e2e-shell-auth AUTH_IDS support
        // constants — one source).
        for id in [
            "bbr-auth-sheet",
            "bbr-auth-header",
            "bbr-auth-username",
            "bbr-auth-password",
            "bbr-auth-submit",
            "bbr-auth-cancel",
        ] {
            assert!(
                window.debug_bounds(id).is_some(),
                "production id {id} rendered by the story"
            );
        }
        // The rejected combinations are declared and refused.
        let rejected_count = window.read(|_| AUTH_REJECTED.len());
        assert_eq!(rejected_count, 3);
        // Mode/state fields are untouched by story selection.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_press_on_log_in_and_cancel_report_through_the_recorder() {
    run_scenario(|window, app| {
        open_auth_sheet(window, app);
        flush(window);
        let (cancel, submit) = first_family_button_centers(window);
        // Sanity: both targets sit inside the first family's bounds.
        let family = bounds_of(window, static_selector(format!("{AUTH_FAMILY_SELECTOR}0")));
        assert!(
            family.contains(&submit) && family.contains(&cancel),
            "button centers fall inside the family wrapper"
        );
        // A real pointer press on Log In: the production `on_submit`
        // callback fires (the press that routes through the app's
        // auth_submit seam).
        window.simulate_click(submit, gpui::Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.auth.submits, 1, "real Log In press records on_submit");
        assert_eq!(snap.auth.cancels, 0, "submit records no cancel");
        // A real pointer press on Cancel: the production `on_cancel`
        // callback fires (the press that routes through the app's
        // auth_cancel seam). The layout is unchanged by the recorder, so
        // the measured centers stay valid.
        window.simulate_click(cancel, gpui::Modifiers::none());
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.auth.submits, 1, "cancel records no submit");
        assert_eq!(snap.auth.cancels, 1, "real Cancel press records on_cancel");
    });
}
