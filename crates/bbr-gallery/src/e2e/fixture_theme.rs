//! Gallery fixture regressions: actual renderer/model themes, not labels.
use bbr_ui::theme::{ContrastMode, Density, MotionMode, Theme, ThemeMode};

use crate::{
    command_fixture::build_command_fixture,
    command_matrix::COMMAND_VALID,
    controls::ControlAction,
    harness::{flush, move_to_story, run_scenario},
    sidebar_fixture::{build_sidebar_fixture_with_theme, fixture_theme, DENSE_WINDOW_ROWS},
    sidebar_matrix::SIDEBAR_VALID,
};

#[test]
#[ignore]
fn fixture_appearance_and_geometry_share_the_resolved_theme() {
    run_scenario(|window, app| {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for contrast in [ContrastMode::Standard, ContrastMode::Increased] {
                for motion in [MotionMode::Standard, MotionMode::Reduced] {
                    let canvas =
                        Theme::new(mode, contrast, Density::ExtraLarge).with_motion_mode(motion);
                    for case in COMMAND_VALID {
                        let model = build_command_fixture(case, &canvas).unwrap();
                        assert_eq!(model.theme.mode, mode);
                        assert_eq!(model.theme.density, case.density);
                        assert_eq!(
                            model.theme.contrast,
                            if case.contrast == ContrastMode::Increased {
                                case.contrast
                            } else {
                                contrast
                            }
                        );
                        assert_eq!(
                            model.theme.motion_mode,
                            if case.reduced_motion {
                                MotionMode::Reduced
                            } else {
                                motion
                            }
                        );
                        assert_eq!(
                            model.theme.palette.surface_overlay,
                            Theme::new(mode, model.theme.contrast, case.density)
                                .palette
                                .surface_overlay
                        );
                    }
                    window.read(|cx| {
                        for case in SIDEBAR_VALID {
                            let theme = fixture_theme(case, &canvas);
                            let model = build_sidebar_fixture_with_theme(
                                case,
                                app.read(cx).sidebar.handles(),
                                &canvas,
                            )
                            .unwrap();
                            assert_eq!(theme.mode, mode);
                            assert_eq!(theme.density, case.density);
                            assert_eq!(
                                theme.contrast,
                                if case.contrast == ContrastMode::Increased {
                                    case.contrast
                                } else {
                                    contrast
                                }
                            );
                            assert_eq!(
                                theme.motion_mode,
                                if case.reduced_motion {
                                    MotionMode::Reduced
                                } else {
                                    motion
                                }
                            );
                            assert_eq!(model.chrome_tint, theme.palette.surface_sidebar);
                            assert_eq!(
                                model.tabs.scroll_bottom,
                                case.tab_count.saturating_sub(DENSE_WINDOW_ROWS) as f32
                                    * f32::from(theme.density.pitch())
                            );
                        }
                    });
                }
            }
        }
        // Verify the actual app's recorder model receives the light canvas,
        // rather than merely checking an independently constructed fixture.
        move_to_story(window, app, 45);
        window.update(|_, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(ControlAction::Mode(ThemeMode::Light), cx);
                app.apply_control(ControlAction::ReducedMotion(true), cx);
                app.apply_control(ControlAction::State(5), cx);
            })
        });
        flush(window);
        window.update(|_, cx| {
            app.update(cx, |app, _| {
                let theme = app.theme();
                let label = app.story_state_label(crate::registry::StoryId(45));
                let context = app
                    .command
                    .render_context(crate::registry::StoryId(45), label, &theme)
                    .unwrap();
                let model = context.model.unwrap();
                assert_eq!(model.theme.mode, ThemeMode::Light);
                assert_eq!(model.theme.motion_mode, MotionMode::Reduced);
            })
        });
    });
}
