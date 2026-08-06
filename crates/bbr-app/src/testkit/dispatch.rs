//! Gateway request dispatch on the main thread.
//!
//! One `Request` → `Response` arm per capability. Pointer-path requests
//! route through the same app methods the element listeners call, so E2E
//! exercises the production listener path rather than a controller call.
//! The sidebar drag arm rebuilds the rendered frame first (untyped window
//! update, so the render can re-borrow the entity) and then dispatches
//! real mouse events through GPUI's production input pipeline.

use std::sync::Arc;

use bbr_browser::ControllerEvent;
use bbr_testkit::protocol::{Error, Event, Request, Response};
use bbr_testkit::GatewayServer;
use gpui::{px, size, App, Keystroke, WindowHandle};

use crate::app::BbrApp;
use crate::testkit::snapshot::SnapshotExt;

pub(crate) fn dispatch(
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
            browser_mutation(window.update(cx, |view, window, cx| {
                view.activate_space(bbr_browser::SpaceId(space.0), window, cx)
            }))
        }

        // Pointer-path activation of visible space rows: the same app method
        // the row mouse-down listeners call, so the asserted scenario
        // exercises the production listener path rather than a direct
        // controller call.
        Request::ClickSpace { space } => window
            .update(cx, |view, window, cx| {
                view.select_space_via_pointer(bbr_browser::SpaceId(space.0), window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

        Request::ClickSpaceNew => window
            .update(cx, |view, window, cx| {
                view.activate_new_space(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

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

        Request::BenchmarkActivateSpace {
            operation_id,
            space,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_activate_space(
                    operation_id,
                    bbr_browser::SpaceId(space.0),
                    window,
                    cx,
                )
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

        // Real pointer drag on the resize divider through the production
        // listener methods (see `BbrApp::drag_sidebar_divider`).
        Request::DragSidebarDivider { dx } => window
            .update(cx, |view, window, cx| {
                view.drag_sidebar_divider(dx, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

        Request::FocusSidebarDivider => window
            .update(cx, |view, window, cx| {
                view.focus_sidebar_divider(window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response),

        Request::BenchmarkResizeSidebar {
            operation_id,
            width,
        } => window
            .update(cx, |view, window, cx| {
                view.benchmark_resize_sidebar(operation_id, width, window, cx)
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

        Request::BenchmarkRestoreSidebar { operation_id } => window
            .update(cx, |view, window, cx| {
                view.benchmark_restore_sidebar(operation_id, window, cx)
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

fn err_response<E: std::fmt::Display>(e: E) -> Response {
    Response::Error(Error::new("app_error", e.to_string()))
}
