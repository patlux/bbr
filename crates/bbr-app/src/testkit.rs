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

use bbr_testkit::protocol::{Error, Event, Request, Response, Snapshot};
use bbr_testkit::GatewayServer;
use gpui::{App, Keystroke, WindowHandle};

use crate::app::BbrApp;

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
            Response(Result<Response, smol::channel::RecvError>),
            Timeout,
        }
        match smol::future::race(
            async { WorkerReply::Response(reply_rx.recv().await) },
            async {
                smol::Timer::after(Duration::from_secs(30)).await;
                WorkerReply::Timeout
            },
        )
        .await
        {
            WorkerReply::Response(Ok(response)) => response,
            WorkerReply::Response(Err(_)) => {
                Response::Error(Error::new("app_gone", "app reply channel closed"))
            }
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
    cmd_rx: smol::channel::Receiver<GatewayCommand>,
    server: Option<Arc<GatewayServer>>,
) {
    let window = window.clone();
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
                let resp = handle_command(msg.req, &window, &server, async_cx).await;
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
        .update(|cx| dispatch(req, window, server, cx))
        .unwrap_or_else(|_| Response::Error(Error::new("app_closed", "app is shutting down")))
}

fn dispatch(
    req: Request,
    window: &WindowHandle<BbrApp>,
    server: Option<Arc<GatewayServer>>,
    cx: &mut App,
) -> Response {
    let broadcast = |server: &Option<Arc<GatewayServer>>, ev: Event| {
        if let Some(s) = server {
            s.broadcast(ev);
        }
    };
    match req {
        Request::Navigate { url } => {
            let r = window
                .update(cx, |view, window, cx| view.navigate(&url, window, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            broadcast(&server, Event::CommandBarChanged { text: url });
            r
        }

        Request::WebAction(action) => window
            .update(cx, |view, _w, _cx| view.web_action(action))
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response),

        Request::ActivateCommandBar => {
            let r = window
                .update(cx, |view, window, cx| view.activate_command_bar(window, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            broadcast(
                &server,
                Event::ChromeFocusChanged {
                    chrome_focused: true,
                },
            );
            r
        }

        Request::FocusWeb => {
            let r = window
                .update(cx, |view, window, cx| view.focus_web(window, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            broadcast(
                &server,
                Event::ChromeFocusChanged {
                    chrome_focused: false,
                },
            );
            r
        }

        Request::SetCommandBarText { text } => {
            let r = window
                .update(cx, |view, _w, cx| {
                    view.command_bar_text = text.clone();
                    cx.notify();
                })
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            broadcast(&server, Event::CommandBarChanged { text });
            r
        }

        Request::DispatchKeystroke { keystroke } => {
            let ks = match Keystroke::parse(&keystroke) {
                Ok(ks) => ks,
                Err(e) => {
                    return Response::Error(Error::new("bad_keystroke", e.to_string()));
                }
            };
            window
                .update(cx, |_view, window, cx| {
                    window.dispatch_keystroke(ks, cx);
                })
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response)
        }

        Request::GetSnapshot => window
            .update(cx, |view, window, _cx| view.snapshot(window).to_protocol())
            .map(Response::Snapshot)
            .unwrap_or_else(err_response),

        Request::Ping => Response::Ok,

        Request::Quit => {
            cx.quit();
            Response::Ok
        }

        // Handled by the server itself.
        Request::Hello { .. } | Request::Attach { .. } => {
            Response::Error(Error::new("server_only", "handled by gateway server"))
        }
    }
}

fn err_response<E: std::fmt::Display>(e: E) -> Response {
    Response::Error(Error::new("app_error", e.to_string()))
}

/// Conversion from the app's local snapshot type to the wire type.
pub(crate) trait SnapshotExt {
    fn to_protocol(&self) -> Snapshot;
}

impl SnapshotExt for crate::shell::TestSnapshot {
    fn to_protocol(&self) -> Snapshot {
        Snapshot {
            url: self.url.clone(),
            title: self.title.clone(),
            loading: self.loading,
            can_go_back: self.can_go_back,
            can_go_forward: self.can_go_forward,
            web_view_hosted: self.web_view_hosted,
            engine_kind: self.engine_kind.to_string(),
            chrome_focused: self.chrome_focused,
            command_bar_text: self.command_bar_text.clone(),
            sidebar_visible: self.sidebar_visible,
            window_size: self.window_size,
        }
    }
}
