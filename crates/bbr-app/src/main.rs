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
mod store;
#[cfg(feature = "testkit")]
mod testkit;
mod window_seam;

use app::BbrApp;
use engine::{build_engine, EngineConfig};
use inject::{IdGen, RandomIdGen, SequentialIdGen};
#[cfg(not(feature = "testkit"))]
use window_seam::spawn_event_pump;
use window_seam::{
    install_actions, install_sigint_shutdown, quit_when_browser_window_closes, with_app,
};

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

    // External-URL open delivery (queue item 13): the unbounded queue the
    // AppKit delegate path and the testkit seam share. Created before the
    // run loop so every URL — including one delivered before the app has
    // finished launching — is buffered and drained once the pump starts.
    #[cfg(target_os = "macos")]
    let (external_url_tx, external_url_rx) = smol::channel::unbounded::<String>();

    stderr_logger::init();

    let idgen = std::rc::Rc::new(std::cell::RefCell::new(match cli.ids {
        IdMode::Sequential => Box::new(SequentialIdGen::default()) as Box<dyn IdGen>,
        IdMode::Random => Box::new(RandomIdGen::default()) as Box<dyn IdGen>,
    }));
    // WebViewId allocation is injected into the portable controller so E2E
    // can assert stable native-session identity across tab switches. Built
    // on demand per startup path (restore and fresh-fallback are mutually
    // exclusive code paths, each owning one allocator), sharing the one
    // underlying id generator so sequential ids stay deterministic.
    let build_allocate_view = move || -> bbr_browser::ViewIdAllocator {
        let idgen = idgen.clone();
        Box::new(move || idgen.borrow_mut().next_u64())
    };

    // GPUI-CE application start; gpui-component must be initialized before any
    // component (including `Root`) is used. The external-URL open delivery
    // (queue item 13) registers on the `Application` builder: GPUI's AppKit
    // delegate (`application:openURLs:` on its `GPUIApplicationDelegate`)
    // forwards OS-dispatched URLs here, and the callback pushes each URL
    // into the unbounded queue the pump drains once the app entity exists —
    // a link arriving while the app is still launching is buffered, never
    // dropped (a dropped cold-start link is a real user-visible failure).
    #[cfg(target_os = "macos")]
    let application = {
        let app = gpui_platform::application();
        let tx = external_url_tx.clone();
        app.on_open_urls(move |urls: Vec<String>| {
            for url in urls {
                if let Err(error) = tx.try_send(url) {
                    log::warn!("could not queue external url: {error}");
                }
            }
        });
        app
    };
    #[cfg(not(target_os = "macos"))]
    let application = gpui_platform::application();
    application.run(move |cx: &mut App| {
        gpui_component::init(cx);
        install_sigint_shutdown(cx);

        // App-owned persistence composition (Phase 7, command-history
        // provider slice): the one `bbr-app -> bbr-persistence` bootstrap.
        // The store restores the global search settings and the
        // command-history provider at startup and logs the truthful
        // `StoreLoad` disposition (a discarded legacy/mismatched store is a
        // normal, non-fatal startup path). The bounded nonblocking history
        // writer runs on the background executor; ranking and render never
        // touch SQLite.
        let mut store = match store::AppStore::open(&cli.profile_dir) {
            Ok(store) => store,
            Err(error) => {
                // A fatal open failure (the store cannot even be created)
                // is logged truthfully and the app continues ephemeral:
                // history/settings stay in memory for the session, every
                // persist is refused with a warning, and the snapshot
                // reports the store as unavailable — never a fabricated
                // disposition.
                log::error!("persistence store open failed ({error:?}); continuing ephemeral");
                store::AppStore::ephemeral()
            }
        };
        // The two bounded nonblocking background writers (command-history
        // provider + browser-state) run on the background executor; ranking
        // and render never touch SQLite. Exactly one AppStore/Repository
        // authority owns both persist paths.
        store.spawn_writer(cx.background_executor());
        store.spawn_browser_writer(cx.background_executor());

        // Phase-7 chrome-first warm-restore plan: the composition reads the
        // repository-restored browser snapshot ONCE here (before the store
        // moves into the app), maps it through the app-owned portable
        // mapping, and selects the deterministic default active space (the
        // policy default: first space with an active tab, else the first
        // space — the pre-quit showing space is not persisted). The window
        // presentation rows (geometry + sidebar) compose through the
        // production window options and the app's restore seam below.
        let restored_browser = store.restored_browser().clone();
        let restore_intent = if restored_browser.profiles.is_empty() {
            None
        } else {
            Some(store::browser::persisted_to_intent(
                &restored_browser.profiles,
            ))
        };
        let restore_active_space = restore_intent
            .as_ref()
            .and_then(store::browser::default_active_space);
        let window_row = restored_browser.windows.first();
        let window_geometry = window_row.and_then(|window| window.geometry);
        let sidebar_state = window_row.map(|window| window.sidebar);
        // Clamp the persisted window size to the positivity/finite contract
        // (the repository already recovered invalid optional values with a
        // typed warning; this is the app-layer safety net before the bounds
        // reach the window options). A window too large for the screen is
        // left to the window server, which clamps the visible frame.
        let requested_bounds = match window_geometry {
            Some(geometry)
                if geometry.width.is_finite()
                    && geometry.width > 0.0
                    && geometry.height.is_finite()
                    && geometry.height > 0.0 =>
            {
                Some(Bounds {
                    origin: gpui::point(px(geometry.x as f32), px(geometry.y as f32)),
                    size: size(px(geometry.width as f32), px(geometry.height as f32)),
                })
            }
            _ => None,
        };
        let bounds = requested_bounds.unwrap_or_else(|| {
            Bounds::centered(None, size(px(cli.window_size.0), px(cli.window_size.1)), cx)
        });
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
        // Find completions ride their own channel (the WebEvent channel is
        // typed for engine navigation events): the find adapter's sink sends
        // `FindCompletion`s from the WebKit completion block, and the pump
        // drains them into `BbrApp::on_find_completion` on the main thread.
        #[cfg(target_os = "macos")]
        let (find_tx, find_rx) =
            smol::channel::unbounded::<bbr_web_macos::find_mapping::FindCompletion>();
        // L6 download events ride their own channel too: the native download
        // adapter's sink sends `DownloadEvent`s from the WKDownload delegate
        // callbacks, and the pump drains them into `BbrApp::on_download_event`
        // (the portable controller applies them — the first terminal event
        // wins).
        #[cfg(target_os = "macos")]
        let (downloads_tx, downloads_rx) =
            smol::channel::unbounded::<bbr_web::downloads::DownloadEvent>();
        // Classified web-error incidents ride their own channel too (task
        // packet `error-surface-contract`): the native delegate's incident
        // sink sends `ClassifiedIncident`s from the WKWebView delegate
        // callbacks, and the pump drains them into `BbrApp::on_incident` on
        // the main thread — the same pattern as the find/downloads
        // adapters. The classification itself (NSError domain+code → the
        // merged contract taxonomy) happens in `bbr-web-macos`; the app
        // only presents the classified result.
        #[cfg(target_os = "macos")]
        let (incidents_tx, incidents_rx) =
            smol::channel::unbounded::<bbr_web_macos::error_mapping::ClassifiedIncident>();
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
        let mut restore_evidence = app::RestoreEvidence::Fresh;
        #[cfg(target_os = "macos")]
        let mut native_engine: Option<crate::engine::NativeEngineRef> = None;
        let window = cx
            .open_window(options, |window, cx| {
                // Capture the GPUI root NSView pointer BEFORE building the
                // root view so the engine can attach the WKWebView.
                let ns_view = extract_ns_view(&*window);
                let config = EngineConfig {
                    non_persistent: cli.test_token.as_ref().is_some(),
                };
                // SAFETY: `ns_view` is the live GPUI root view on the main thread.
                let built = unsafe { build_engine(ns_view, &config, engine_kind) };
                let engine = built.engine;
                #[cfg(target_os = "macos")]
                {
                    native_engine = Some(built.native);
                }
                let controller = match (&restore_intent, restore_active_space) {
                    // Persisted browser model: restore chrome-first through
                    // the restored facade. Only the active space's visible
                    // set hydrates a session; every other tab stays
                    // session-less until activation. A typed failure rolls
                    // the controller back to a fresh default model (the
                    // facade's pinned non-transactional recovery contract)
                    // and reports [`RestoreEvidence::Failed`] — never a
                    // partial restored window presented as valid.
                    (Some(intent), Some(active_space)) => {
                        match BrowserController::restored(
                            engine.clone(),
                            build_allocate_view(),
                            intent,
                            active_space,
                        ) {
                            Ok((controller, plan)) => {
                                restore_evidence = app::RestoreEvidence::Restored {
                                    deferred_tabs: plan.deferred.len(),
                                };
                                controller
                            }
                            Err(error) => {
                                log::error!(
                                    "browser-state restore rejected ({error}); starting fresh"
                                );
                                restore_evidence = app::RestoreEvidence::Failed;
                                BrowserController::new(engine.clone(), build_allocate_view())
                            }
                        }
                    }
                    // No persisted browser model: the fresh path opens the
                    // initial tab as before.
                    _ => {
                        restore_evidence = app::RestoreEvidence::Fresh;
                        BrowserController::new(engine.clone(), build_allocate_view())
                    }
                };
                let organization = bbr_browser::OrganizationController::new();
                let app = cx.new(|cx| {
                    let mut app = BbrApp::new(
                        engine,
                        controller,
                        organization,
                        engine_kind,
                        evt_tx.clone(),
                        #[cfg(target_os = "macos")]
                        find_tx.clone(),
                        #[cfg(feature = "testkit")]
                        view_performance,
                        store,
                        cx,
                    );
                    app.set_restore_evidence(restore_evidence);
                    app
                });
                app_entity = Some(app.clone());
                cx.new(|cx| gpui_component::Root::new(app, window, cx))
            })
            .expect("open window");
        let app = app_entity.expect("app entity built with the window root");
        // Normal-quit persistence correctness: flush the latest structural
        // browser model through the existing owning shutdown seam when the
        // browser window closes (the background writer uses the identical
        // repository write). Narrow seam only — no general deterministic
        // process-shutdown behavior is implemented here (queue item 20).
        quit_when_browser_window_closes(cx, window, app.clone());

        // macOS-only native seams: the concrete engine handle (the find
        // adapter binds a live WKWebView without creating/moving sessions)
        // is handed to the app after construction, like the portal host.
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine.clone() {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.set_find_engine(native);
            });
        }

        // Error-surface composition (task packet `error-surface-contract`):
        // the concrete engine's classified-incident sink forwards every
        // native failure into the incidents channel the pump drains into
        // `BbrApp::on_incident`. Installed before the download composition
        // consumes the native handle, like the find seam.
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine.clone() {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.install_error_surface(native, incidents_tx.clone());
            });
        }

        // macOS-only per-site zoom composition: the concrete engine handle
        // binds the zoom target (WebKitZoomAdapter) to the active view's
        // WKWebView without creating/moving sessions.
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine.clone() {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.set_zoom_engine(native);
            });
        }

        // macOS-only context-menu composition: the concrete engine handle
        // exposes the native bridge's ownership diagnostics (the right-click
        // monitor and the page-vs-element classification counters); the
        // menu itself routes actions through the portable engine seam.
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine.clone() {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.set_menu_engine(native);
            });
        }

        // macOS-only window-geometry composition: the concrete engine handle
        // lets the persist source read the live session's window content
        // rect (contentLayoutRect) so the persisted geometry round-trips
        // exactly through the content-semantics window options.
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine.clone() {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.set_geometry_engine(native);
            });
        }

        // macOS-only download composition: the concrete native adapter
        // (bound to the real WKDownload lifecycle) plus the engine handle
        // for the active-view capture. Under the testkit feature the
        // suggested-destination folder is the profile's own `downloads`
        // directory (deterministic E2E destination; the v1 destination
        // policy — silent suggested folder vs. a save panel — stays open,
        // DESIGN §17.9); production uses the engine default (system
        // downloads).
        #[cfg(target_os = "macos")]
        if let Some(native) = native_engine {
            let mtm =
                objc2::MainThreadMarker::new().expect("download adapter built on main thread");
            let options = bbr_web_macos::downloads_web_kit::DownloadAdapterOptions {
                #[cfg(feature = "testkit")]
                download_folder: Some(
                    cli.profile_dir
                        .join("downloads")
                        .to_string_lossy()
                        .into_owned(),
                ),
                #[cfg(not(feature = "testkit"))]
                download_folder: None,
            };
            let strategy =
                bbr_web_macos::downloads_web_kit::WebKitDownloadStrategy::new(mtm, options);
            let adapter = std::rc::Rc::new(
                bbr_web_macos::downloads_adapter::NativeDownloadAdapter::with_strategy(strategy),
            );
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.install_downloads(adapter, native, downloads_tx.clone());
            });
        }

        // Phase-7 window/sidebar presentation restore: the persisted sidebar
        // geometry (width, collapse memory, collapsed flag) composes through
        // the production restore seam with the same clamping every input
        // path uses. The window bounds were already applied through the
        // window options above. Invalid optional values were already
        // recovered by the repository (typed warnings); the app clamps again
        // as a safety net.
        if let Some(sidebar) = sidebar_state {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.restore_window_state(&sidebar);
            });
        }

        // Create the web view with the initial URL (first prepaint will
        // frame it) — only on the fresh/failed path. A chrome-first restore
        // already owns a hydrated model: the active space's visible set has
        // its session and every other tab is deferred (session-less until
        // activation), so no initial tab may be added on top.
        if !matches!(restore_evidence, app::RestoreEvidence::Restored { .. }) {
            let _ = with_app(&window, &app, cx, |view, _w, _cx| {
                view.open_initial_tab(&cli.initial_url)
            });
        }

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
                    window,
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
                find_rx,
                downloads_rx,
                #[cfg(target_os = "macos")]
                external_url_rx,
                #[cfg(target_os = "macos")]
                external_url_tx,
                incidents_rx,
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
            spawn_event_pump(
                cx,
                &window,
                &app,
                window_seam::EventPumpChannels {
                    events: evt_rx,
                    find: find_rx,
                    downloads: downloads_rx,
                    external_urls: external_url_rx,
                    incidents: incidents_rx,
                },
            );
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
