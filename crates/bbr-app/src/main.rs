//! `bbr` composition root.
//!
//! Owns the [`gpui::Application`], parses CLI flags, opens the window, builds
//! the native engine (attached to the GPUI root view), wires keyboard actions,
//! and — under the `testkit` feature — starts the E2E gateway.

use std::path::PathBuf;
#[cfg(feature = "testkit")]
use std::sync::Arc;
#[cfg(feature = "testkit")]
use std::time::Duration;

use bbr_browser::BrowserController;
use gpui::prelude::*;
use gpui::{px, size, App, Bounds, Entity, Window, WindowBounds, WindowOptions};

mod actions;
mod app;
mod command_bar;
mod components;
mod dense;
mod engine;
mod inject;
#[allow(dead_code)] // foundation tokens; consumed by the shell slices that follow this group
mod palette;
#[cfg(feature = "testkit")]
mod performance;
// The general Portal API (deferred host, anchored placement, escape
// dismissal) awaits the U1 palette migration; the production probe consumes
// the native-host path, so the remaining surface is allowed until U1 lands.
#[allow(dead_code)]
mod portal;
mod shell;
// `SidebarGeometry::{decrease,increase,reset,last_expanded}` and the divider
// label are consumed by the unit tests and the testkit-only snapshot/keyboard
// seams; non-testkit builds need the module allowance to stay clean.
#[allow(dead_code)]
mod sidebar;
mod slot;
#[cfg(feature = "testkit")]
mod testkit;
#[allow(dead_code)] // foundation tokens; consumed by the shell slices that follow this group
mod theme;
mod window_seam;

use app::BbrApp;
use engine::{build_engine, EngineConfig};
use inject::{IdGen, RandomIdGen, SequentialIdGen};
#[cfg(not(feature = "testkit"))]
use window_seam::spawn_event_pump;
use window_seam::{install_actions, with_app};

/// Parsed command-line flags (all `--bbr-*` are test/injection knobs).
struct Cli {
    initial_url: String,
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))]
    profile_dir: PathBuf,
    test_token: Option<String>,
    hidden: bool,
    ids: IdMode,
    #[allow(dead_code)] // parsed; no-op until Phase 5 debounces
    clock: ClockMode,
    window_size: (f32, f32),
    /// Testkit-only deterministic presentation delay under test (ms). Armed
    /// only from `--bbr-bench-inject-delay-ms`; impossible without `testkit`.
    #[cfg(feature = "testkit")]
    bench_inject_delay_ms: Option<u64>,
}

#[derive(Clone, Copy, PartialEq)]
enum IdMode {
    Random,
    Sequential,
}

#[derive(Clone, Copy, PartialEq)]
enum ClockMode {
    System,
    Fake,
}

impl Cli {
    fn parse() -> Self {
        let mut initial_url = "about:blank".to_string();
        let mut profile_dir: Option<PathBuf> = None;
        let mut test_token: Option<String> = None;
        let mut hidden = false;
        let mut ids = IdMode::Random;
        let mut clock = ClockMode::System;
        let mut window_size = app::DEFAULT_WINDOW_SIZE;
        #[cfg(feature = "testkit")]
        let mut bench_inject_delay_ms = None;

        let mut args = std::env::args();
        let _bin = args.next();
        while let Some(arg) = args.next() {
            let key = arg.split('=').next().unwrap_or("");
            let inline_val = arg.split_once('=').map(|(_, v)| v.to_string());
            let mut value_opt = inline_val;
            let mut take = |value_opt: &mut Option<String>| {
                value_opt
                    .get_or_insert_with(|| args.next().unwrap())
                    .clone()
            };
            match key {
                "--bbr-initial-url" => initial_url = take(&mut value_opt),
                "--bbr-profile-dir" => {
                    profile_dir = Some(PathBuf::from(take(&mut value_opt)));
                }
                "--bbr-test-token" => test_token = Some(take(&mut value_opt)),
                "--bbr-hidden" => hidden = true,
                "--bbr-ids" => {
                    ids = match take(&mut value_opt).as_str() {
                        "sequential" => IdMode::Sequential,
                        _ => IdMode::Random,
                    };
                }
                "--bbr-clock" => {
                    clock = match take(&mut value_opt).as_str() {
                        "fake" => ClockMode::Fake,
                        _ => ClockMode::System,
                    };
                }
                "--bbr-window-size" => {
                    if let Some((w, h)) = take(&mut value_opt).split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.parse::<f32>(), h.parse::<f32>()) {
                            window_size = (w, h);
                        }
                    }
                }
                #[cfg(feature = "testkit")]
                "--bbr-bench-inject-delay-ms" => {
                    bench_inject_delay_ms = take(&mut value_opt).parse::<u64>().ok();
                }
                _ => {}
            }
        }

        let profile_dir = profile_dir.unwrap_or_else(|| {
            std::env::temp_dir().join(format!("bbr-profile-{}", std::process::id()))
        });

        Cli {
            initial_url,
            profile_dir,
            test_token,
            hidden,
            ids,
            clock,
            window_size,
            #[cfg(feature = "testkit")]
            bench_inject_delay_ms,
        }
    }
}

fn main() {
    #[cfg(feature = "testkit")]
    let process_started = std::time::Instant::now();
    let cli = Cli::parse();
    // `--bbr-clock` is parsed but intentionally a no-op in Phase 1: there is
    // no time-based logic yet (debounces land in Phase 5/7). Documented in
    // ARCHITECTURE.md.
    let _clock = cli.clock;

    stderr_logger::init();

    let mut idgen: Box<dyn IdGen> = match cli.ids {
        IdMode::Sequential => Box::new(SequentialIdGen::default()),
        IdMode::Random => Box::new(RandomIdGen::default()),
    };
    // WebViewId allocation is injected into the portable controller so E2E
    // can assert stable native-session identity across tab switches.
    let allocate_view: bbr_browser::ViewIdAllocator = Box::new(move || idgen.next_u64());

    // GPUI-CE application start; gpui-component must be initialized before any
    // component (including `Root`) is used.
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(cli.window_size.0), px(cli.window_size.1)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: !cli.hidden,
            // `--bbr-hidden` (every E2E harness) must never order the window
            // front: a shown window is visible to the host's window manager
            // (yabai BSP on this host tiles it to the current space's tile
            // frame — 1213x1063 here — so the frame is no longer the
            // requested 1080x700 and geometry suites flake depending on the
            // manager's state). A never-shown window keeps the requested
            // bounds deterministically; WebKit and GPUI rendering work
            // without display (frames are driven by the testkit FlushFrame
            // path).
            show: !cli.hidden,
            ..Default::default()
        };

        let engine_kind = current_engine_kind();
        let (evt_tx, evt_rx) = smol::channel::unbounded::<bbr_web::WebEvent>();
        #[cfg(feature = "testkit")]
        let performance = performance::PerformanceProbe::new(
            process_started,
            cli.bench_inject_delay_ms.map(Duration::from_millis),
        );
        #[cfg(feature = "testkit")]
        let view_performance = performance.clone();

        // Root-window + app-entity seam: the browser window's root view is
        // `gpui_component::Root`; the `BbrApp` entity is built first and
        // retained separately so actions, pumps, dispatch, and presentation
        // hooks address the app directly (see `with_app`).
        let mut app_entity: Option<Entity<BbrApp>> = None;
        let window = cx
            .open_window(options, |window, cx| {
                // Capture the GPUI root NSView pointer BEFORE building the
                // root view so the engine can attach the WKWebView.
                let ns_view = extract_ns_view(&*window);
                let config = EngineConfig {
                    non_persistent: cli.test_token.as_ref().is_some(),
                };
                // SAFETY: `ns_view` is the live GPUI root view on the main thread.
                let engine = unsafe { build_engine(ns_view, &config, engine_kind) };
                let controller = BrowserController::new(engine.clone(), allocate_view);
                let app = cx.new(|cx| {
                    BbrApp::new(
                        engine,
                        controller,
                        engine_kind,
                        evt_tx.clone(),
                        #[cfg(feature = "testkit")]
                        view_performance,
                        cx,
                    )
                });
                app_entity = Some(app.clone());
                cx.new(|cx| gpui_component::Root::new(app, window, cx))
            })
            .expect("open window");
        let app = app_entity.expect("app entity built with the window root");

        // Create the web view with the initial URL (first prepaint will frame it).
        let _ = with_app(&window, &app, cx, |view, _w, _cx| {
            view.open_initial_tab(&cli.initial_url)
        });

        // Native portal host (task packet native-portal-host): exactly one
        // transparent, non-activating portal window per browser window,
        // created hidden at window construction and attached above the
        // parent NSWindow by the bridge in `bbr-web-macos`. Retained by the
        // app entity; AppKit owns z-order/attachment/frame/visibility. The
        // app entity and the shared probe render state are handed to the
        // portal window's root so the probe surface renders exactly what the
        // app commands and routes outside presses through the production
        // outside-dismiss method.
        #[cfg(target_os = "macos")]
        {
            let parent_ns_view = window
                .update(cx, |_root, window, _cx| extract_ns_view(window))
                .expect("capture browser root view");
            let probe = with_app(&window, &app, cx, |view, _w, _cx| {
                view.portal_probe_handle()
            })
            .expect("capture portal probe handle");
            // SAFETY: `parent_ns_view` is the live GPUI root view of the
            // browser window on the main thread; the bridge retains it.
            let bridge = unsafe {
                crate::portal::native::NativePortalBridge::open(
                    cx,
                    parent_ns_view,
                    app.clone(),
                    probe,
                )
            }
            .expect("open native portal host");
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.set_native_portal(bridge);
            });
        }
        // Keyboard actions (fire even while the WKWebView is first responder,
        // via the macOS performKeyEquivalent path).
        install_actions(cx, window, app.clone());

        // Test gateway (only under the `testkit` feature).
        #[cfg(feature = "testkit")]
        {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let server: Option<Arc<bbr_testkit::GatewayServer>> =
                if let Some(token) = &cli.test_token {
                    std::fs::create_dir_all(&cli.profile_dir).ok();
                    let server = testkit::start(
                        &cli.profile_dir,
                        token,
                        env!("CARGO_PKG_VERSION"),
                        cmd_tx.clone(),
                    )
                    .expect("start testkit gateway");
                    Some(server)
                } else {
                    None
                };
            testkit::spawn_pump(
                cx,
                &window,
                &app,
                evt_rx,
                evt_tx.clone(),
                cmd_rx,
                server.clone(),
            );
            if let Some(server) = &server {
                let broadcaster = server.clone();
                performance.set_sink(Box::new(move |measurement| {
                    broadcaster.broadcast(bbr_testkit::protocol::Event::PresentationCommitted(
                        measurement,
                    ));
                }));
                server.broadcast(bbr_testkit::protocol::Event::AppReady {
                    pid: std::process::id(),
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                });
                // Route real focus changes (Cmd-L / Escape / clicks) to the
                // gateway. The focus is queried from GPUI's actual focus tree
                // after each mutation, so the emitted event always matches the
                // snapshot's `chrome_focused`.
                let broadcaster = server.clone();
                let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                    view.set_focus_sink(Box::new(move |chrome_focused| {
                        broadcaster.broadcast(bbr_testkit::protocol::Event::ChromeFocusChanged {
                            chrome_focused,
                        });
                    }));
                    let broadcaster = server.clone();
                    view.set_browser_sink(Box::new(move |outcome| {
                        if let Some(kind) = testkit::browser_change(outcome) {
                            broadcaster
                                .broadcast(bbr_testkit::protocol::Event::BrowserChanged { kind });
                        }
                    }));
                });
            }
        }
        #[cfg(not(feature = "testkit"))]
        {
            // Event pump so the shell stays live even without the gateway.
            spawn_event_pump(cx, &window, &app, evt_rx);
        }

        if !cli.hidden {
            cx.activate(true);
        }
    });
}

/// A tiny logger that forwards `log` records to stderr.
mod stderr_logger {
    use std::io::Write;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());
    static LOGGER: Logger = Logger;

    struct Logger;

    pub fn init() {
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(log::LevelFilter::Info);
    }

    impl log::Log for Logger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            let _guard = LOCK.lock().unwrap();
            let _ = writeln!(std::io::stderr(), "[{}] {}", record.level(), record.args());
        }
        fn flush(&self) {}
    }
}

/// Extract the GPUI root `NSView*` from the window handle (macOS only).
#[cfg(target_os = "macos")]
fn extract_ns_view(window: &Window) -> *mut std::ffi::c_void {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = match HasWindowHandle::window_handle(window) {
        Ok(handle) => handle,
        Err(_) => return std::ptr::null_mut(),
    };
    match handle.as_raw() {
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr(),
        _ => std::ptr::null_mut(),
    }
}

#[cfg(not(target_os = "macos"))]
fn extract_ns_view(_window: &Window) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

#[cfg(target_os = "macos")]
fn current_engine_kind() -> &'static str {
    "native-wkwebview"
}

#[cfg(not(target_os = "macos"))]
fn current_engine_kind() -> &'static str {
    "null"
}
