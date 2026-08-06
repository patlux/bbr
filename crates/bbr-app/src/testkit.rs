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

use bbr_browser::ControllerEvent;
use bbr_testkit::protocol::{
    BrowserChange, Error, Event, Request, Response, Snapshot, TabSnapshot,
};
use bbr_testkit::GatewayServer;
use gpui::{px, size, App, Keystroke, WindowHandle};

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
        Request::CreateProfile {
            profile_title,
            space_title,
        } => browser_mutation(window.update(cx, |view, _window, cx| {
            view.create_profile_space(&profile_title, &space_title, cx)
        })),

        Request::CreateSpace { profile, title } => {
            browser_mutation(window.update(cx, |view, _window, cx| {
                view.create_space(profile, &title, cx)
            }))
        }

        Request::ActivateSpace { space } => {
            browser_mutation(window.update(cx, |view, _window, cx| {
                view.activate_space(bbr_browser::SpaceId(space.0), cx)
            }))
        }

        Request::OpenTab { url } => {
            browser_mutation(window.update(cx, |view, _window, cx| view.open_tab(&url, cx)))
        }

        Request::SelectTab { index } => {
            browser_mutation(window.update(cx, |view, _window, cx| view.select_tab(index, cx)))
        }

        Request::CloseTab { index } => {
            browser_mutation(window.update(cx, |view, _window, cx| view.close_tab(index, cx)))
        }

        Request::ReopenTab => {
            browser_mutation(window.update(cx, |view, window, cx| view.reopen_tab(window, cx)))
        }

        Request::PinTab { index } => {
            browser_mutation(window.update(cx, |view, _window, cx| view.pin_tab(index, cx)))
        }

        // Pointer-path activation of visible rows: the same app methods the
        // row mouse-down listeners call, so the asserted scenario exercises
        // the production listener path rather than a direct controller call.
        Request::ClickTab { index } => window
            .update(cx, |view, window, cx| {
                view.select_row_via_pointer(index, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

        Request::ClickTabClose { index } => window
            .update(cx, |view, window, cx| {
                view.close_row_via_pointer(index, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

        Request::Navigate { url } => {
            let r = window
                .update(cx, |view, window, cx| view.navigate(&url, window, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(err_response);
            broadcast(
                &server,
                Event::CommandBarChanged {
                    mode: bbr_testkit::protocol::CommandBarMode::Inactive,
                    text: url,
                },
            );
            r
        }

        Request::MoveTab { index, to_space } => {
            browser_mutation(window.update(cx, |view, _window, cx| {
                view.move_tab(index, bbr_browser::SpaceId(to_space.0), cx)
            }))
        }

        Request::SuspendTab { index } => {
            browser_mutation(window.update(cx, |view, _window, cx| view.suspend_tab(index, cx)))
        }

        Request::BenchmarkSelectTab {
            operation_id,
            index,
        } => browser_mutation(window.update(cx, |view, window, cx| {
            view.benchmark_select_tab(operation_id, index, window, cx)
        })),

        Request::BenchmarkClickTab {
            operation_id,
            index,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_click_tab(operation_id, index, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkReopenTab { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_reopen_tab(operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkPageReady {
            operation_id,
            expected_view,
            expected_title,
        } => window
            .update(cx, |view, _window, _cx| {
                view.benchmark_page_ready(operation_id, expected_view, expected_title)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new("benchmark_busy", "page-ready already pending"))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkToggleSidebar { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_toggle_sidebar(operation_id, window, cx)
            })
            .map(|started| {
                if started {
                    Response::Ok
                } else {
                    Response::Error(Error::new(
                        "benchmark_busy",
                        "another presentation measurement is pending",
                    ))
                }
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkDispatchKeystroke {
            operation_id,
            kind,
            keystroke,
        } => {
            let started = window
                .update(cx, |view, _window, _cx| {
                    view.begin_benchmark(operation_id, kind)
                })
                .unwrap_or(false);
            if !started {
                return Response::Error(Error::new(
                    "benchmark_busy",
                    "another presentation measurement is pending",
                ));
            }
            let ks = match Keystroke::parse(&keystroke) {
                Ok(ks) => ks,
                Err(error) => {
                    let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                    return Response::Error(Error::new("bad_keystroke", error.to_string()));
                }
            };
            (**window)
                .update(cx, |_root, window, cx| window.dispatch_keystroke(ks, cx))
                .map(|_| Response::Ok)
                .unwrap_or_else(|error| {
                    let _ = window.update(cx, |view, _window, _cx| view.cancel_benchmark());
                    Response::Error(Error::new("app_error", error.to_string()))
                })
        }

        Request::WebAction(action) => window
            .update(cx, |view, _w, _cx| view.web_action(action))
            .map(|outcome| {
                if outcome.is_some() {
                    Response::Ok
                } else {
                    Response::Error(Error::new("browser_error", "web action was rejected"))
                }
            })
            .unwrap_or_else(err_response),

        Request::ActivateCommandBar => window
            .update(cx, |view, window, cx| view.activate_command_bar(window, cx))
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response),

        Request::FocusWeb => window
            .update(cx, |view, window, cx| view.focus_web(window, cx))
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response),

        Request::SetCommandBarText { text } => {
            let result = window.update(cx, |view, _w, cx| {
                view.command_bar_text = text.clone();
                let mode = view.command_bar_mode.into();
                cx.notify();
                mode
            });
            match result {
                Ok(mode) => {
                    broadcast(&server, Event::CommandBarChanged { mode, text });
                    Response::Ok
                }
                Err(error) => err_response(error),
            }
        }

        Request::DispatchKeystroke { keystroke } => {
            let ks = match Keystroke::parse(&keystroke) {
                Ok(ks) => ks,
                Err(e) => {
                    return Response::Error(Error::new("bad_keystroke", e.to_string()));
                }
            };
            // Dispatch through the untyped `AnyWindowHandle` path, NOT
            // `WindowHandle::update`. Dispatching a keystroke synchronously
            // re-renders the root view (the keystroke can invalidate focus or
            // layout), and rendering leases the `BbrApp` entity. `WindowHandle::update`
            // already holds that entity lease for the whole closure, so the
            // nested render would double-lease and panic ("cannot update BbrApp
            // while it is already being updated"). `AnyWindowHandle::update`
            // borrows only the window, leaving the entity free for the render.
            // Note the double deref: `window` is `&WindowHandle<BbrApp>` (Copy),
            // so `*window` is the typed handle again; `**window` reaches the
            // deref target `AnyWindowHandle`.
            (**window)
                .update(cx, |_root, window, cx| {
                    window.dispatch_keystroke(ks, cx);
                })
                .map(|_| Response::Ok)
                .unwrap_or_else(|e| Response::Error(Error::new("app_error", e.to_string())))
        }

        Request::GetSnapshot => window
            .update(cx, |view, window, _cx| view.snapshot(window).to_protocol())
            .map(Response::Snapshot)
            .unwrap_or_else(err_response),

        Request::SetWindowSize { width, height } => window
            .update(cx, |_view, window, _cx| {
                window.resize(size(px(width), px(height)));
            })
            .map(|_| Response::Ok)
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

        // Handled by the pump before dispatch (rides the real event transport).
        Request::InjectWebEvent { .. } => {
            Response::Error(Error::new("pump_only", "handled by the event pump"))
        }
    }
}

fn browser_mutation<E: std::fmt::Display>(update: Result<Option<ControllerEvent>, E>) -> Response {
    match update {
        Ok(Some(_)) => Response::Ok,
        Ok(None) => Response::Error(Error::new("browser_error", "browser mutation was rejected")),
        Err(error) => err_response(error),
    }
}

pub(crate) fn browser_change(outcome: &ControllerEvent) -> Option<BrowserChange> {
    match outcome {
        ControllerEvent::ProfileCreated { profile, space } => Some(BrowserChange::ProfileCreated {
            profile: *profile,
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::SpaceCreated { profile, space } => Some(BrowserChange::SpaceCreated {
            profile: *profile,
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::SpaceActivated { space } => Some(BrowserChange::SpaceActivated {
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::TabOpened { index, .. } => {
            Some(BrowserChange::TabOpened { index: *index })
        }
        ControllerEvent::TabSelected { index } => {
            Some(BrowserChange::TabSelected { index: *index })
        }
        ControllerEvent::TabClosed { index } => Some(BrowserChange::TabClosed { index: *index }),
        ControllerEvent::TabPinned { index } => Some(BrowserChange::TabPinned { index: *index }),
        ControllerEvent::TabReopened { index, .. } => {
            Some(BrowserChange::TabReopened { index: *index })
        }
        ControllerEvent::TabMoved {
            to_space,
            recreated,
            ..
        } => Some(BrowserChange::TabMoved {
            to_space: bbr_testkit::protocol::TestSpaceId(to_space.0),
            recreated: *recreated,
        }),
        ControllerEvent::TabSuspended { index } => {
            Some(BrowserChange::TabSuspended { index: *index })
        }
        _ => None,
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
            active_profile: self.active_profile,
            active_space: self.active_space,
            tabs: self
                .tabs
                .iter()
                .map(|tab| TabSnapshot {
                    index: tab.index,
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    pinned: tab.pinned,
                    loading: tab.loading,
                    crashed: tab.crashed,
                    active: tab.active,
                    web_view: tab.web_view,
                })
                .collect(),
            active_tab_index: self.active_tab_index,
            active_web_view: self.active_web_view,
            engine_sessions: self.engine_sessions,
            engine_kind: self.engine_kind.to_string(),
            chrome_focused: self.chrome_focused,
            shell_modes: self.shell_modes.clone(),
            tab_states: self.tab_states.clone(),
            command_bar_mode: self.command_bar_mode,
            command_bar_text: self.command_bar_text.clone(),
            sidebar_visible: self.sidebar_visible,
            window_size: self.window_size,
        }
    }
}
