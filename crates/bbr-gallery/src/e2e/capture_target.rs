//! Guest-only semantic wiring test. No real Metal, Check SVG or input claim.
use gpui::{px, size, Entity, VisualTestContext};

use crate::{
    app::GalleryApp,
    capture_cases,
    capture_page::{self, PageCapture},
    capture_target::DrawToken,
    harness::{bounds_of, run_scenario, snapshot},
};

#[test]
#[ignore]
fn checkbox_capture_observes_its_child_in_each_appearance_and_not_other_pages() {
    let cases = capture_cases::cases(&crate::stories::catalog());
    let checkbox_cases: Vec<_> = cases.iter().filter(|case| case.story.0 == 16).collect();
    assert_eq!(
        checkbox_cases.len(),
        12,
        "both appearances at every contrast/density"
    );
    for case in checkbox_cases {
        run_scenario(|window, app| {
            window.simulate_resize(size(px(1200.0), px(1000.0)));
            let page = PageCapture::default();
            page.reset(Some(0..5));
            window.update(|window, cx| {
                let mode = match case.mode {
                    bbr_ui::theme::ThemeMode::Light => gpui_component::ThemeMode::Light,
                    bbr_ui::theme::ThemeMode::Dark => gpui_component::ThemeMode::Dark,
                };
                gpui_component::Theme::change(mode, Some(window), cx);
                app.update(cx, |app, cx| app.select_capture(case, page.clone(), cx));
            });
            window.run_until_parked();
            let before = snapshot(window, app);
            assert!(!before.checkbox_checked);
            assert_eq!(before.theme, case.mode);
            assert_eq!(before.contrast, case.contrast);
            assert_eq!(before.density, case.density);
            // Repeated draws need new observations even without a page reset.
            let mut previous = None;
            for range in [0..5, 0..5, 1..2] {
                if range == (1..2) {
                    page.reset(Some(range.clone()));
                }
                let token = draw(window, app, &page);
                assert_ne!(Some(token), previous);
                previous = Some(token);
                let state = page.0.borrow();
                capture_page::validate_frame(&state, &range).unwrap();
                assert_eq!(state.total, 5);
                let measured = state.checkbox.resolve(token, true).unwrap().unwrap();
                drop(state);
                assert_eq!(
                    measured,
                    bounds_of(window, "bbr-gallery-controls-checkbox"),
                    "hug container must equal its production child"
                );
            }
            page.reset(Some(2..5)); // Pagination must not inherit hidden control bounds.
            let token = draw(window, app, &page);
            assert_eq!(
                page.0.borrow().checkbox.resolve(token, false).unwrap(),
                None
            );
            capture_page::validate_frame(&page.0.borrow(), &(2..5)).unwrap();
            assert!(window
                .debug_bounds("bbr-gallery-controls-checkbox")
                .is_none());
            assert_eq!(
                snapshot(window, app),
                before,
                "observation must not mutate state"
            );

            // A different story must not retain a Checkbox observation either.
            let tokens = cases
                .iter()
                .find(|other| {
                    other.story.0 == 0
                        && other.mode == case.mode
                        && other.contrast == case.contrast
                        && other.density == case.density
                })
                .unwrap();
            page.reset(Some(0..10));
            window.update(|_, cx| {
                app.update(cx, |app, cx| app.select_capture(tokens, page.clone(), cx));
            });
            let token = draw(window, app, &page);
            assert_eq!(
                page.0.borrow().checkbox.resolve(token, false).unwrap(),
                None
            );
            assert!(window
                .debug_bounds("bbr-gallery-controls-checkbox")
                .is_none());
        });
    }
}

fn draw(window: &mut VisualTestContext, app: &Entity<GalleryApp>, page: &PageCapture) -> DrawToken {
    window.run_until_parked();
    let token = page.0.borrow_mut().checkbox.begin_draw();
    window.update(|window, cx| {
        app.update(cx, |_, cx| cx.notify());
        window.refresh();
        window.draw(cx).clear();
    });
    token
}
