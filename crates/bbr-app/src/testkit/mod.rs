//! Testkit glue: wires the E2E gateway into a running app's main loop.
//!
//! Only compiled under the `testkit` cargo feature. Two pieces:
//!
//! - [`start`] binds the [`GatewayServer`] (unix socket in the profile dir)
//!   and hands its handler a blocking bridge to the main thread.
//! - [`spawn_pump`] runs a main-thread task that drains (a) engine events
//!   into the shell + gateway broadcast, and (b) gateway commands into the
//!   app via the window handle.
//!
//! The gateway handler runs on socket worker threads and MUST NOT touch the
//! app directly — it forwards each request through a channel and blocks on
//! the reply produced on the main thread.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bbr_testkit::protocol::{Error, Event, Request, Response};
use bbr_testkit::GatewayServer;
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

mod dispatch;
mod dispatch_auth;
mod dispatch_benchmark;
mod dispatch_benchmark_background_links;
mod dispatch_benchmark_context_menu;
mod dispatch_benchmark_downloads;
mod dispatch_benchmark_find;
mod dispatch_benchmark_gestures;
mod dispatch_benchmark_library;
mod dispatch_benchmark_peek;
mod dispatch_benchmark_responsiveness;
mod dispatch_benchmark_swipe;
mod dispatch_benchmark_tab_menu;
mod dispatch_benchmark_u2;
mod dispatch_benchmark_zoom;
#[cfg(feature = "visual-capture")]
mod dispatch_capture;
mod dispatch_context_menu;
mod dispatch_downloads;
mod dispatch_drag;
#[cfg(target_os = "macos")]
mod dispatch_errors;
mod dispatch_find;
mod dispatch_library;
mod dispatch_portal;
mod dispatch_probe;
mod dispatch_saved_pages;
mod dispatch_tab_menu;
mod dispatch_titlebar;
mod dispatch_u2;
mod dispatch_util;
mod dispatch_zoom;
mod snapshot;
pub(crate) use snapshot::browser_change;

/// A gateway command forwarded to the main-thread pump.
pub struct GatewayCommand {
    pub req: Request,
    /// The reply channel; the pump sends exactly one response.
    pub reply_tx: smol::channel::Sender<Response>,
}

/// Start the gateway server and return the channels the pump needs.
///
/// `profile_dir` must already exist (the app creates it). `token` is the
/// per-spawn secret.
pub fn start(
    profile_dir: &Path,
    token: &str,
    app_version: &str,
    cmd_tx: smol::channel::Sender<GatewayCommand>,
) -> anyhow::Result<Arc<GatewayServer>> {
    // The handler runs on socket worker threads; it forwards to the main
    // thread and blocks (with a timeout) on the reply.
    let handler = move |req: Request| -> Response { dispatch_from_worker(req, &cmd_tx) };
    let server = GatewayServer::start(profile_dir, token, app_version, handler)?;
    Ok(Arc::new(server))
}

/// Called by socket worker threads: forward to the main pump, block for the
/// reply (bounded so a wedged app fails the test visibly instead of hanging).
fn dispatch_from_worker(req: Request, cmd_tx: &smol::channel::Sender<GatewayCommand>) -> Response {
    let (reply_tx, reply_rx) = smol::channel::unbounded::<Response>();
    let msg = GatewayCommand { req, reply_tx };
    if cmd_tx.try_send(msg).is_err() {
        return Response::Error(Error::new("app_gone", "app command channel closed"));
    }
    smol::block_on(async {
        enum WorkerReply {
            Response(Box<Result<Response, smol::channel::RecvError>>),
            Timeout,
        }
        match smol::future::race(
            async { WorkerReply::Response(Box::new(reply_rx.recv().await)) },
            async {
                smol::Timer::after(Duration::from_secs(30)).await;
                WorkerReply::Timeout
            },
        )
        .await
        {
            WorkerReply::Response(boxed) => match *boxed {
                Ok(response) => response,
                Err(_) => Response::Error(Error::new("app_gone", "app reply channel closed")),
            },
            WorkerReply::Timeout => Response::Error(Error::new(
                "timeout",
                "command timed out on the main thread",
            )),
        }
    })
}

/// Spawn the main-thread pump: drains engine events and gateway commands.
///
/// `window` lets the pump mutate the app on the main thread; `server` (when
/// given) receives event broadcasts. The event pump always runs so the shell
/// stays live even without the testkit feature.
#[allow(clippy::too_many_arguments)] // the pump wiring: channels, handles, server
pub fn spawn_pump(
    cx: &App,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    evt_rx: smol::channel::Receiver<bbr_web::WebEvent>,
    inject_tx: smol::channel::Sender<bbr_web::WebEvent>,
    #[cfg(target_os = "macos")] find_rx: smol::channel::Receiver<
        bbr_web_macos::find_mapping::FindCompletion,
    >,
    #[cfg(target_os = "macos")] downloads_rx: smol::channel::Receiver<
        bbr_web::downloads::DownloadEvent,
    >,
    // External-URL open deliveries (queue item 13): buffered URLs from the
    // AppKit delegate path or the testkit seam, drained on the main thread
    // into `BbrApp::on_external_url`. The unbounded queue doubles as the
    // cold-start buffer: a link arriving before the app finished launching
    // waits here until the pump starts.
    #[cfg(target_os = "macos")] external_url_rx: smol::channel::Receiver<String>,
    // The external-URL delivery queue's sender: the testkit
    // `DeliverExternalUrl` request pushes into the exact channel the AppKit
    // delegate path uses, so the seam and the production path share one
    // transport (queue item 13).
    #[cfg(target_os = "macos")] external_url_tx: smol::channel::Sender<String>,
    #[cfg(target_os = "macos")] incidents_rx: smol::channel::Receiver<
        bbr_web_macos::error_mapping::ClassifiedIncident,
    >,
    #[cfg(target_os = "macos")] new_windows_rx: smol::channel::Receiver<bbr_web::NewWindowDelivery>,
    #[cfg(target_os = "macos")] gestures_rx: smol::channel::Receiver<bbr_web::ContentGestureCommit>,
    // The content-gesture delivery queue's sender: the testkit
    // `ContentGestureCommit` request pushes into the exact channel the
    // native content-gesture bridge reporter uses, so the seam and the
    // production path share one transport (N1).
    #[cfg(target_os = "macos")] gestures_tx: smol::channel::Sender<bbr_web::ContentGestureCommit>,
    cmd_rx: smol::channel::Receiver<GatewayCommand>,
    server: Option<Arc<GatewayServer>>,
) {
    let window = *window;
    let app = app.clone();
    let server = server.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        enum Incoming {
            Command(Result<GatewayCommand, smol::channel::RecvError>),
            Event(Result<bbr_web::WebEvent, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Find(Result<bbr_web_macos::find_mapping::FindCompletion, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Download(Result<bbr_web::downloads::DownloadEvent, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            ExternalUrl(Result<String, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            Incident(
                Result<bbr_web_macos::error_mapping::ClassifiedIncident, smol::channel::RecvError>,
            ),
            #[cfg(target_os = "macos")]
            NewWindow(Result<bbr_web::NewWindowDelivery, smol::channel::RecvError>),
            #[cfg(target_os = "macos")]
            ContentGesture(Result<bbr_web::ContentGestureCommit, smol::channel::RecvError>),
        }

        let incoming = smol::future::race(
            async { Incoming::Command(cmd_rx.recv().await) },
            smol::future::race(
                async { Incoming::Event(evt_rx.recv().await) },
                smol::future::race(
                    #[cfg(target_os = "macos")]
                    async {
                        Incoming::Find(find_rx.recv().await)
                    },
                    smol::future::race(
                        #[cfg(target_os = "macos")]
                        async {
                            Incoming::Download(downloads_rx.recv().await)
                        },
                        smol::future::race(
                            #[cfg(target_os = "macos")]
                            async {
                                Incoming::ExternalUrl(external_url_rx.recv().await)
                            },
                            smol::future::race(
                                #[cfg(target_os = "macos")]
                                async {
                                    Incoming::Incident(incidents_rx.recv().await)
                                },
                                smol::future::race(
                                    #[cfg(target_os = "macos")]
                                    async {
                                        Incoming::NewWindow(new_windows_rx.recv().await)
                                    },
                                    #[cfg(target_os = "macos")]
                                    async {
                                        Incoming::ContentGesture(gestures_rx.recv().await)
                                    },
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        )
        .await;
        match incoming {
            Incoming::Command(Ok(msg)) => {
                let quitting = matches!(msg.req, Request::Quit);
                // Ordering contract for injected content-gesture commits: a
                // gateway command observes every commit pushed before it.
                // The typed `ContentGestureCommit` request rides the exact
                // channel the native bridge reporter uses; the pump drains
                // it here before answering the command so an immediate
                // snapshot after an injection can never race a partial
                // drain (the observed no-op matrix failure saw 1 of 3
                // commits: the snapshot command won the pump race over the
                // pending gesture queue). Testkit-only; the production
                // event pump in `window_seam` has no command plane.
                #[cfg(target_os = "macos")]
                while let Ok(commit) = gestures_rx.try_recv() {
                    async_cx.update(|cx| {
                        let _ = with_app(&window, &app, cx, |view, _w, cx| {
                            view.on_content_gesture_commit(commit, cx)
                        });
                    });
                }
                // Deterministic event injection rides the same channel the
                // real engine sink uses: the pump broadcasts it to E2E and
                // applies it through `on_web_event`, exactly like a native
                // delegate event. Testkit-only request; production builds
                // never see it.
                let resp = match &msg.req {
                    Request::InjectWebEvent { event } => {
                        let _ = inject_tx.try_send(event.clone());
                        Response::Ok
                    }
                    // The external-URL delivery seam pushes into the exact
                    // queue the AppKit delegate path uses; the pump's own
                    // `ExternalUrl` arm drains it into the app through
                    // `on_external_url` — one production transport.
                    #[cfg(target_os = "macos")]
                    Request::DeliverExternalUrl { url } => {
                        let _ = external_url_tx.try_send(url.clone());
                        Response::Ok
                    }
                    // The content-gesture seam pushes into the exact queue
                    // the native content-gesture bridge reporter uses; the
                    // pump's own `ContentGesture` arm drains it into the app
                    // through `on_content_gesture_commit` — one production
                    // transport, exactly like the new-window deliveries.
                    #[cfg(target_os = "macos")]
                    Request::ContentGestureCommit { view, direction } => {
                        let _ = gestures_tx.try_send(bbr_web::ContentGestureCommit {
                            view: *view,
                            direction: *direction,
                        });
                        Response::Ok
                    }
                    _ => handle_command(msg.req, &window, &app, &server, async_cx).await,
                };
                let _ = msg.reply_tx.send(resp).await;
                // `Quit` has already broadcast `Quitting`, called the real
                // `cx.quit()` path and flushed its response to the client.
                // Stop only this testkit observation loop so a dense run does
                // not serialize hundreds of already-buffered WebKit callbacks
                // ahead of GPUI/native teardown. Product session destruction,
                // persistence and process exit remain owned by the app.
                if quitting {
                    return;
                }
            }
            Incoming::Event(Ok(ev)) => {
                // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
                async_cx.update(|cx| {
                    if let Some(server) = &server {
                        server.broadcast(Event::Web(ev.clone()));
                    }
                    let _ = with_app(&window, &app, cx, |view, window, cx| {
                        view.on_web_event(ev, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Find(Ok(completion)) => {
                // Find completions arrive from the WebKit completion block;
                // route them through the same main-thread app update path
                // as engine events (`BbrApp::on_find_completion`).
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_find_completion(completion, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Download(Ok(event)) => {
                // L6 download events from the native adapter: route through
                // the same main-thread app update path as engine events
                // (the portable controller applies them — the first terminal
                // event wins).
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        crate::app::downloads_behavior::on_download_event(view, event, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::ExternalUrl(Ok(url)) => {
                // External-URL open from the AppKit delegate path (or the
                // testkit seam): route through the same main-thread app
                // update path as engine events. URLs buffered before the app
                // finished launching are drained here, in order.
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_external_url(url, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::Incident(Ok(incident)) => {
                // Classified web-error incidents from the native delegate's
                // incident sink: route through the same main-thread app
                // update path as engine/find/downloads events (the surface
                // presents the classified incident without touching tab,
                // session, profile, or space identity).
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_incident(incident, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::NewWindow(Ok(delivery)) => {
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, window, cx| {
                        view.on_new_window_request(delivery, window, cx)
                    });
                });
            }
            #[cfg(target_os = "macos")]
            Incoming::ContentGesture(Ok(commit)) => {
                async_cx.update(|cx| {
                    let _ = with_app(&window, &app, cx, |view, _w, cx| {
                        view.on_content_gesture_commit(commit, cx)
                    });
                });
            }
            Incoming::Command(Err(_)) | Incoming::Event(Err(_)) => break,
            #[cfg(target_os = "macos")]
            Incoming::Find(Err(_))
            | Incoming::Download(Err(_))
            | Incoming::ExternalUrl(Err(_))
            | Incoming::Incident(Err(_))
            | Incoming::NewWindow(Err(_))
            | Incoming::ContentGesture(Err(_)) => break,
        }
    })
    .detach();
}

/// Handle one gateway command on the main thread and return its response.
async fn handle_command(
    req: Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    server: &Option<Arc<GatewayServer>>,
    async_cx: &mut gpui::AsyncApp,
) -> Response {
    let server = server.clone();
    // GPUI-CE `AsyncApp::update` is infallible (no `.unwrap_or_else`); the
    // dispatch itself returns the `Response` for every request.
    async_cx.update(|cx| dispatch::dispatch(req, window, app, server, cx))
}
