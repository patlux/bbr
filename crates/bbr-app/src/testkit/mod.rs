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
use gpui::{App, WindowHandle};

use crate::app::BbrApp;

mod dispatch;
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
pub fn spawn_pump(
    cx: &App,
    window: &WindowHandle<BbrApp>,
    evt_rx: smol::channel::Receiver<bbr_web::WebEvent>,
    inject_tx: smol::channel::Sender<bbr_web::WebEvent>,
    cmd_rx: smol::channel::Receiver<GatewayCommand>,
    server: Option<Arc<GatewayServer>>,
) {
    let window = *window;
    let server = server.clone();
    cx.spawn(async move |async_cx: &mut gpui::AsyncApp| loop {
        enum Incoming {
            Command(Result<GatewayCommand, smol::channel::RecvError>),
            Event(Result<bbr_web::WebEvent, smol::channel::RecvError>),
        }

        let incoming =
            smol::future::race(async { Incoming::Command(cmd_rx.recv().await) }, async {
                Incoming::Event(evt_rx.recv().await)
            })
            .await;
        match incoming {
            Incoming::Command(Ok(msg)) => {
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
                    _ => handle_command(msg.req, &window, &server, async_cx).await,
                };
                let _ = msg.reply_tx.send(resp).await;
            }
            Incoming::Event(Ok(ev)) => {
                async_cx
                    .update(|cx| {
                        if let Some(server) = &server {
                            server.broadcast(Event::Web(ev.clone()));
                        }
                        let _ = window.update(cx, |view, _w, cx| view.on_web_event(ev, cx));
                    })
                    .ok();
            }
            Incoming::Command(Err(_)) | Incoming::Event(Err(_)) => break,
        }
    })
    .detach();
}

/// Handle one gateway command on the main thread and return its response.
async fn handle_command(
    req: Request,
    window: &WindowHandle<BbrApp>,
    server: &Option<Arc<GatewayServer>>,
    async_cx: &mut gpui::AsyncApp,
) -> Response {
    let server = server.clone();
    async_cx
        .update(|cx| dispatch::dispatch(req, window, server, cx))
        .unwrap_or_else(|_| Response::Error(Error::new("app_closed", "app is shutting down")))
}
