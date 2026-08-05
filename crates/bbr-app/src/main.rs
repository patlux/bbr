//! `bbr` composition root.
//!
//! Owns the [`gpui::Application`], parses CLI flags, opens the window, builds
//! the native engine (attached to the GPUI root view), wires keyboard actions,
//! and — under the `testkit` feature — starts the E2E gateway.

use std::path::PathBuf;
#[cfg(feature = "testkit")]
use std::sync::Arc;

use bbr_web::{ProfileId, WebViewId};
use gpui::prelude::*;
use gpui::{px, size, App, Bounds, KeyBinding, Window, WindowBounds, WindowOptions};

mod actions;
mod app;
mod engine;
mod inject;
mod shell;
mod slot;
#[cfg(feature = "testkit")]
mod testkit;

use actions::{Back, FocusCommandBar, FocusWeb, Forward, NewTab, Reload, ToggleSidebar};
use app::BbrApp;
use engine::{build_engine, EngineConfig};
use inject::{IdGen, RandomIdGen, SequentialIdGen};
use shell::ShellState;

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
        }
    }
}

fn main() {
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
    let profile = ProfileId(idgen.next_u64());
    let view_id = WebViewId(idgen.next_u64());

    gpui::Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(cli.window_size.0), px(cli.window_size.1)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: !cli.hidden,
            show: true,
            ..Default::default()
        };

        let engine_kind = current_engine_kind();
        let shell = ShellState::new(profile, view_id, engine_kind);

        let (evt_tx, evt_rx) = smol::channel::unbounded::<bbr_web::WebEvent>();

        let window = cx
            .open_window(options, |window, cx| {
                // Capture the GPUI root NSView pointer BEFORE building the
                // root view so the engine can attach the WKWebView.
                let ns_view = extract_ns_view(&*window);
                let config = EngineConfig {
                    non_persistent: cli.test_token.as_ref().is_some(),
                };
                // SAFETY: `ns_view` is the live GPUI root view on the main thread.
                let engine = unsafe { build_engine(ns_view, &config, view_id, engine_kind) };
                cx.new(|cx| BbrApp::new(engine, shell.clone(), evt_tx.clone(), cx))
            })
            .expect("open window");

        // Create the web view with the initial URL (first prepaint will frame it).
        window
            .update(cx, |view, _w, _cx| view.create_web_view(&cli.initial_url))
            .ok();

        // Keyboard actions (fire even while the WKWebView is first responder,
        // via the macOS performKeyEquivalent path).
        install_actions(cx, &window);

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
            testkit::spawn_pump(cx, &window, evt_rx, cmd_rx, server.clone());
            if let Some(server) = &server {
                server.broadcast(bbr_testkit::protocol::Event::AppReady {
                    pid: std::process::id(),
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                });
                // Route real focus changes (Cmd-L / Escape / clicks) to the
                // gateway. The focus is queried from GPUI's actual focus tree
                // after each mutation, so the emitted event always matches the
                // snapshot's `chrome_focused`.
                let broadcaster = server.clone();
                window
                    .update(cx, |view, _w, _cx| {
                        view.set_focus_sink(Box::new(move |chrome_focused| {
                            broadcaster.broadcast(
                                bbr_testkit::protocol::Event::ChromeFocusChanged { chrome_focused },
                            );
                        }));
                    })
                    .ok();
            }
        }
        #[cfg(not(feature = "testkit"))]
        {
            // Event pump so the shell stays live even without the gateway.
            spawn_event_pump(cx, &window, evt_rx);
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

/// Register global action handlers (bubble phase) that reach the app view.
///
/// These run while the keystroke dispatch already holds the window borrowed
/// (see [`gpui::Window::dispatch_keystroke`]), so a direct `win.update` would
/// re-enter the window update and fail with "window not found". The update is
/// therefore deferred to the effect flush — the same pattern GPUI's own
/// [`gpui::Window::dispatch_action`] uses — so the view is updated once the
/// keystroke dispatch has released the window.
fn install_actions(cx: &mut App, window: &gpui::WindowHandle<BbrApp>) {
    let win = *window;
    cx.on_action(move |_: &FocusCommandBar, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, window, cx| view.activate_command_bar(window, cx));
        });
    });
    let win = *window;
    cx.on_action(move |_: &FocusWeb, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, window, cx| view.focus_web(window, cx));
        });
    });
    let win = *window;
    cx.on_action(move |_: &Back, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, _w, _cx| view.go_back());
        });
    });
    let win = *window;
    cx.on_action(move |_: &Forward, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, _w, _cx| view.go_forward());
        });
    });
    let win = *window;
    cx.on_action(move |_: &Reload, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, _w, _cx| view.reload());
        });
    });
    let win = *window;
    cx.on_action(move |_: &ToggleSidebar, cx| {
        let win = win;
        cx.defer(move |cx| {
            let _ = win.update(cx, |view, _w, cx| view.toggle_sidebar(cx));
        });
    });
    let win = *window;
    cx.on_action(move |_: &NewTab, cx| {
        let win = win;
        cx.defer(move |cx| {
            // Phase 5: command bar new-tab mode. For now, focus the command bar.
            let _ = win.update(cx, |view, window, cx| view.activate_command_bar(window, cx));
        });
    });

    cx.bind_keys([
        KeyBinding::new("cmd-l", FocusCommandBar, None),
        KeyBinding::new("escape", FocusWeb, None),
        KeyBinding::new("cmd-[", Back, None),
        KeyBinding::new("cmd-]", Forward, None),
        KeyBinding::new("cmd-r", Reload, None),
        KeyBinding::new("cmd-b", ToggleSidebar, None),
        KeyBinding::new("cmd-t", NewTab, None),
    ]);
}

/// Non-testkit builds still need the event pump to keep the shell live.
#[cfg(not(feature = "testkit"))]
fn spawn_event_pump(
    cx: &App,
    window: &gpui::WindowHandle<BbrApp>,
    evt_rx: smol::channel::Receiver<bbr_web::WebEvent>,
) {
    let win = *window;
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        let ev = match evt_rx.recv().await {
            Ok(ev) => ev,
            Err(_) => break,
        };
        async_cx
            .update(|cx| {
                let _ = win.update(cx, |view, _w, cx| view.on_web_event(ev, cx));
            })
            .ok();
    })
    .detach();
}
