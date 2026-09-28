//! Opt-in windowless Gallery producer: real macOS text and offscreen Metal.
//! TestPlatform supplies only window/scheduling infrastructure, never pixels.
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{Entity, HeadlessAppContext, WindowHandle};

use crate::{
    app::GalleryApp,
    capture_cases::{self, CaptureCase},
    capture_page::{self, PageCapture},
};

struct Runner {
    // Entity handles must be released before HeadlessAppContext shuts down.
    app: Entity<GalleryApp>,
    window: WindowHandle<gpui_component::Root>,
    page: PageCapture,
    cx: HeadlessAppContext,
}

/// Return true only for a recognized capture CLI. Invalid args never launch a
/// normal interactive Gallery by accident.
pub fn cli() -> bool {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return false;
    }
    let mut cases = capture_cases::cases(&crate::stories::catalog());
    if args == ["--capture-inventory"] {
        for case in cases {
            println!("{}\t{}\t{}", case.id(), case.label, case.state_label);
        }
        return true;
    }
    assert!(
        args.len() == 2 || args.len() == 4,
        "usage: --capture-gallery <fresh-directory> [--story <id>]"
    );
    assert_eq!(args[0], "--capture-gallery");
    if args.len() == 4 {
        assert_eq!(args[2], "--story");
        let story: u64 = args[3].parse().expect("numeric story id");
        cases.retain(|case| case.story.0 == story);
        assert!(!cases.is_empty(), "story not registered");
    }
    let output = PathBuf::from(&args[1]);
    assert!(
        output.is_absolute()
            && !output.exists()
            && !output
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir)),
        "fresh absolute output directory without traversal required"
    );
    assert!(
        !output.ancestors().any(|path| path.is_symlink()),
        "symlink output forbidden"
    );
    fs::create_dir(&output).expect("create capture directory");
    let mut runner = Runner::new();
    let mut rows = Vec::new();
    for case in cases {
        runner.capture(&case, &output, &mut rows);
    }
    drop(runner);
    // Completion marker is written only after every page and shutdown succeeded.
    fs::write(output.join("coverage.tsv"), rows.join("\n") + "\n").expect("write coverage");
    true
}

impl Runner {
    fn new() -> Self {
        // font-kit is explicitly enabled in the workspace pin. Only obtain its
        // real text system: never run a MacPlatform application or open one of
        // its native windows. The test window owns the real Metal sprite atlas.
        let platform = gpui_platform::current_platform(true);
        let mut cx = HeadlessAppContext::with_platform(
            platform.text_system(),
            Arc::new(()), // Same empty asset source as the interactive Gallery.
            gpui_platform::current_headless_renderer,
        );
        let (window, app) = cx.update(|cx| {
            gpui_component::init(cx);
            crate::open_window_with_options(
                cx,
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                        gpui::point(gpui::px(0.0), gpui::px(0.0)),
                        gpui::size(
                            gpui::px(capture_page::WIDTH),
                            gpui::px(capture_page::HEIGHT),
                        ),
                    ))),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
            )
        });
        Self {
            cx,
            window,
            app,
            page: PageCapture::default(),
        }
    }

    fn capture(&mut self, case: &CaptureCase, output: &std::path::Path, rows: &mut Vec<String>) {
        self.page.reset(None);
        self.window
            .update(&mut self.cx, |_, window, cx| {
                let mode = match case.mode {
                    bbr_ui::theme::ThemeMode::Light => gpui_component::ThemeMode::Light,
                    bbr_ui::theme::ThemeMode::Dark => gpui_component::ThemeMode::Dark,
                };
                gpui_component::Theme::change(mode, Some(window), cx);
                self.app.update(cx, |app, cx| {
                    app.select_capture(case, self.page.clone(), cx)
                });
            })
            .expect("select headless case");
        self.draw();
        let heights: Vec<_> = self
            .page
            .0
            .borrow()
            .bounds
            .iter()
            .map(|b| f32::from(b.size.height))
            .collect();
        let pages = capture_page::paginate(&heights)
            .unwrap_or_else(|error| panic!("{}: {error}", case.id()));
        for (index, range) in pages.into_iter().enumerate() {
            self.page.reset(Some(range.clone()));
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                self.draw();
                let state = self.page.0.borrow();
                assert!(
                    !state.bounds.is_empty(),
                    "page did not prepaint: {}",
                    case.id()
                );
                if state.is_settled(case.settling_duration()) {
                    capture_page::validate_frame(&state, &range)
                        .unwrap_or_else(|error| panic!("{}: {error}", case.id()));
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "endpoint deadline: {}",
                    case.id()
                );
                // GPUI AnimationElement samples wall-clock Instant, not the test
                // scheduler clock. Explicit layout/paint until the production
                // finite duration has elapsed; never sleep or poll PNG pixels.
            }
            let image = self
                .cx
                .capture_screenshot(self.window.into())
                .expect("offscreen Metal readback");
            assert_eq!(image.dimensions(), (2400, 2000));
            let id = format!("{}-page-{index:02}", case.id());
            image
                .save(output.join(format!("{id}.png")))
                .expect("save Metal PNG");
            let total = self.page.0.borrow().total;
            rows.push(format!(
                "{id}\t{}\t{}\t{}\t{}\t{}\t{total}",
                case.id(),
                case.label,
                case.state_label,
                range.start,
                range.end
            ));
            eprintln!(
                "captured {id} children {}..{} / {total}",
                range.start, range.end
            );
        }
    }

    fn draw(&mut self) {
        // Drain model/asset work without advancing timers (e.g. cursor blink).
        // No DisplayLink, native event loop, presentation or activation needed.
        self.cx.run_until_parked();
        // Use the untyped app update: WindowHandle::update leases Root, which
        // would double-borrow that entity when draw renders it again.
        self.cx
            .update_window(self.window.into(), |_, window, cx| {
                self.app.update(cx, |_, cx| cx.notify());
                assert_eq!(f32::from(window.viewport_size().width), capture_page::WIDTH);
                assert_eq!(
                    f32::from(window.viewport_size().height),
                    capture_page::HEIGHT
                );
                assert_eq!(window.scale_factor(), 2.0);
                window.refresh();
                window.draw(cx).clear(); // Clear the per-App element arena every frame.
            })
            .expect("draw headless Gallery frame");
    }
}
