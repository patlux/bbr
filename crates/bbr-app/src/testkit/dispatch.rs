//! Gateway request dispatch on the main thread.
//!
//! One `Request` → `Response` arm per capability. Pointer-path requests
//! route through the same app methods the element listeners call, so E2E
//! exercises the production listener path rather than a controller call.
//! The sidebar drag arm rebuilds the rendered frame first (untyped window
//! update, so the render can re-borrow the entity) and then dispatches
//! real mouse events through GPUI's production input pipeline.

use std::sync::Arc;

use bbr_testkit::protocol::{Error, Event, Request, Response};
use bbr_testkit::GatewayServer;
use gpui::{px, size, App, Entity, Keystroke, WindowHandle};

use crate::window_seam::with_app;

use crate::app::BbrApp;
use crate::testkit::dispatch_auth::dispatch_auth;
use crate::testkit::dispatch_benchmark::dispatch_benchmark;
use crate::testkit::dispatch_benchmark_context_menu::dispatch_benchmark_context_menu;
use crate::testkit::dispatch_benchmark_downloads::dispatch_benchmark_downloads;
use crate::testkit::dispatch_benchmark_swipe::dispatch_benchmark_swipe;
use crate::testkit::dispatch_benchmark_u2::dispatch_benchmark_u2;
use crate::testkit::dispatch_context_menu::dispatch_context_menu;
use crate::testkit::dispatch_downloads::dispatch_downloads;
use crate::testkit::dispatch_drag::dispatch_drag;
#[cfg(target_os = "macos")]
use crate::testkit::dispatch_errors::dispatch_errors;
use crate::testkit::dispatch_find::dispatch_find;
use crate::testkit::dispatch_portal::dispatch_portal;
use crate::testkit::dispatch_probe::dispatch_probe;
use crate::testkit::dispatch_saved_pages::dispatch_saved_pages;
use crate::testkit::dispatch_u2::dispatch_u2;
use crate::testkit::dispatch_util::{browser_mutation, err_response};
use crate::testkit::dispatch_zoom::dispatch_zoom;
use crate::testkit::snapshot::SnapshotExt;

pub(crate) fn dispatch(
    req: Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    server: Option<Arc<GatewayServer>>,
    cx: &mut App,
) -> Response {
    // The performance arms live in `dispatch_benchmark` so this router stays
    // under the ~500-line budget; every other request falls through here.
    if let Some(response) = dispatch_benchmark(&req, window, app, cx) {
        return response;
    }
    // The §11.3 swipe-commit performance arm routes through its focused
    // module (same split reason; the gesture runs entirely in the app).
    if let Some(response) = dispatch_benchmark_swipe(&req, window, app, cx) {
        return response;
    }
    // The downloads performance arms (benchmark + deterministic seed) route
    // through their focused module before the fall-through (same split
    // reason); the draw for the render-inclusive commit happens there.
    if let Some(response) = dispatch_benchmark_downloads(&req, window, app, cx) {
        return response;
    }
    if let Some(response) = dispatch_benchmark_u2(&req, window, app, cx) {
        return response;
    }
    // The tab-drag pointer arms live in `dispatch_drag` (same split reason).
    if let Some(response) = dispatch_drag(&req, window, app, cx) {
        return response;
    }
    // Error-surface incident injection + benchmark arms route through their
    // focused module (macOS-only: the classified payload is produced by the
    // native adapter's mapping).
    #[cfg(target_os = "macos")]
    if let Some(response) = dispatch_errors(&req, window, app, cx) {
        return response;
    }
    // Portal probe/teardown/flush routing lives separately because the
    // portal-window draw must happen outside the mutable app lease.
    if let Some(response) = dispatch_portal(&req, window, app, cx) {
        return response;
    }
    // Find-in-page routing shares the same portal-frame flush rule.
    if let Some(response) = dispatch_find(&req, window, app, cx) {
        return response;
    }
    // Zoom routing (queue item 11, part A): the production `Cmd-Plus`/
    // `Cmd-Minus`/`Cmd-0` seams (the benchmark arm routes through
    // `dispatch_benchmark_zoom`).
    if let Some(response) = dispatch_zoom(&req, window, app, cx) {
        return response;
    }
    // Context-menu routing (queue item 12, part B): the production pointer/
    // close seams (the benchmark arm routes through
    // `dispatch_benchmark_context_menu`).
    if let Some(response) = dispatch_context_menu(&req, window, app, cx) {
        return response;
    }
    // The context-menu benchmark arm (render-gated, like the find arms).
    if let Some(response) = dispatch_benchmark_context_menu(&req, window, app, cx) {
        return response;
    }
    // U2 footer/favorites/folders route separately to keep this router bounded.
    if let Some(response) = dispatch_u2(&req, window, app, cx) {
        return response;
    }
    // Downloads start/cancel/retry/row-action/outside-dismiss route
    // separately (same bounded-router reason).
    if let Some(response) = dispatch_downloads(&req, window, app, cx) {
        return response;
    }
    // The real WebKit content probe routes separately (same bounded-router
    // reason); non-probe requests fall through.
    if let Some(response) = dispatch_probe(&req, window, app, cx) {
        return response;
    }
    // The credential sheet (queue item 10) routes separately (same
    // bounded-router reason): the presentation + seeding + benchmark arms
    // flush the portal frame outside the mutable app lease.
    if let Some(response) = dispatch_auth(&req, window, app, cx) {
        return response;
    }
    // Saved Page save/remove/open/flush + benchmark arms route separately
    // (B2 packet `saved-pages-actions-command`; same bounded-router reason).
    if let Some(response) = dispatch_saved_pages(&req, window, app, cx) {
        return response;
    }
    let broadcast = |server: &Option<Arc<GatewayServer>>, ev: Event| {
        if let Some(s) = server {
            s.broadcast(ev);
        }
    };
    match req {
        Request::CreateProfile {
            profile_title,
            space_title,
        } => browser_mutation(with_app(window, app, cx, |view, _window, cx| {
            view.create_profile_space(&profile_title, &space_title, cx)
        })),

        Request::CreateSpace { profile, title } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.create_space(profile, &title, cx)
            }))
        }

        Request::ActivateSpace { space } => {
            browser_mutation(with_app(window, app, cx, |view, window, cx| {
                view.activate_space(bbr_browser::SpaceId(space.0), window, cx)
            }))
        }

        // Pointer-path activation of visible space rows: the same app method
        // the row mouse-down listeners call, so the asserted scenario
        // exercises the production listener path rather than a direct
        // controller call.
        Request::ClickSpace { space } => with_app(window, app, cx, |view, window, cx| {
            view.select_space_via_pointer(bbr_browser::SpaceId(space.0), window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickSpaceNew => with_app(window, app, cx, |view, window, cx| {
            view.activate_new_space(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::OpenTab { url } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.open_tab(&url, cx)
            }))
        }

        Request::NewWindow {
            source,
            source_profile,
            url,
            user_initiated,
            intent,
        } => {
            let request = bbr_web::NewWindowRequest {
                source,
                source_profile,
                url,
                user_initiated,
                target: bbr_web::NewWindowTarget::NewWindow,
                features: bbr_web::NewWindowFeatures::default(),
            };
            let decision = bbr_web::decision_for_intent(&request, intent);
            let delivery = bbr_web::NewWindowDelivery {
                outcome: bbr_web::decide(&request, decision),
                request,
            };
            browser_mutation(with_app(window, app, cx, |view, window, cx| {
                view.on_new_window_request(delivery, window, cx)
            }))
        }

        Request::SelectTab { index } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.select_tab(index, cx)
            }))
        }

        Request::CloseTab { index } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.close_tab(index, cx)
            }))
        }

        Request::ReopenTab => browser_mutation(with_app(window, app, cx, |view, window, cx| {
            view.reopen_tab(window, cx)
        })),

        // The split shared-contract seam: open/close the two-pane layout
        // through the production controller split path (the split-core lane
        // owns the user-facing creation UI). Both arms sync engine
        // visibility, notify the view, and broadcast the typed outcome.
        Request::SetSplit { primary, secondary } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.set_split(primary, secondary, cx)
            }))
        }

        Request::CloseSplit => browser_mutation(with_app(window, app, cx, |view, _window, cx| {
            view.close_split(cx)
        })),

        Request::PinTab { index } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.pin_tab(index, cx)
            }))
        }

        Request::UnpinTab { index } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.unpin_tab(index, cx)
            }))
        }

        Request::ReorderTab { from, to } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.reorder_tab_seam(from, to, cx)
            }))
        }

        // Pointer-path activation of visible rows: the same app methods the
        // row mouse-down listeners call, so the asserted scenario exercises
        // the production listener path rather than a direct controller call.
        Request::ClickTab { index } => with_app(window, app, cx, |view, window, cx| {
            view.select_row_via_pointer(index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickTabClose { index } => with_app(window, app, cx, |view, window, cx| {
            view.close_row_via_pointer(index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickTabPin { index } => with_app(window, app, cx, |view, window, cx| {
            view.toggle_row_pin(index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // Pointer hover/press state through the production listener methods
        // (the exact app handlers the row's `on_hover` and mouse-down/up
        // element listeners call): the row renders the §10.2 hover wash +
        // close reveal and the §10.3 pressed wash. Never a direct snapshot
        // mutation — the states mutate the same app fields the real pointer
        // paths drive, and the render derives the visual from them.
        Request::HoverTabRow { index } => with_app(window, app, cx, |view, _window, cx| {
            view.set_row_hover(index, true, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::UnhoverTabRow => with_app(window, app, cx, |view, _window, cx| {
            view.clear_row_hover(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        // §11.5 address-pill hover: the same production `on_hover` path the
        // pill's element listener calls — the wash follows the pointer.
        Request::HoverAddress => with_app(window, app, cx, |view, _window, cx| {
            view.set_address_hover(true, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::UnhoverAddress => with_app(window, app, cx, |view, _window, cx| {
            view.set_address_hover(false, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::PressTabRow { index } => with_app(window, app, cx, |view, _window, cx| {
            view.set_row_pressed(index, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ReleaseTabRow => with_app(window, app, cx, |view, _window, cx| {
            view.clear_row_pressed(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::Navigate { url } => {
            let r = with_app(window, app, cx, |view, window, cx| {
                view.navigate(&url, window, cx)
            })
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
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.move_tab(index, bbr_browser::SpaceId(to_space.0), cx)
            }))
        }

        Request::SuspendTab { index } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.suspend_tab(index, cx)
            }))
        }

        // Deterministic dense-list seeding (DESIGN.md §11.6): open
        // `tab_count` tabs through the production controller open path,
        // suspend every tab outside the keep set immediately (each transient
        // session is destroyed before the next open, so the concurrent
        // live-session population never approaches `tab_count`), then
        // activate the pinned tab 0. Batches keep the same final keep set
        // because the keep rule uses the global index. In-memory state;
        // persisted-restore acceptance stays open.
        Request::SeedDenseState {
            tab_count,
            pinned_count,
            recent_start,
            url,
        } => browser_mutation(with_app(window, app, cx, |view, _window, cx| {
            view.seed_dense_state(tab_count, pinned_count, recent_start, &url, cx)
        })),

        // Deterministic session-budget convergence (testkit-only setup; the
        // policy is the production lifecycle path).
        Request::EnforceSessionBudget { cap } => {
            browser_mutation(with_app(window, app, cx, |view, _window, cx| {
                view.enforce_session_budget(cap, cx)
            }))
        }

        // A real scroll of the dense regular tab region through the
        // production scroll state: the tracked `ScrollHandle` the native
        // wheel listener, the keyboard traversal, and the render share
        // (GPUI 0.2.2 exposes no public synthetic wheel dispatch —
        // `Window::dispatch_event` returns a crate-private result type). The
        // app method mutates the offset in the wheel listener's units and
        // sign; a frame is drawn afterwards so the snapshot observes the
        // bounded window the scroll produced. Never a direct snapshot
        // mutation.
        Request::ScrollDenseTabs { delta_px } => {
            let scrolled = with_app(window, app, cx, |view, _window, _cx| {
                view.scroll_dense_list(delta_px)
            })
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response);
            let _ = (**window).update(cx, |_root, window, cx| {
                let app: &mut App = cx;
                let _ = window.draw(app);
            });
            scrolled
        }

        Request::WebAction(action) => {
            with_app(window, app, cx, |view, _w, _cx| view.web_action(action))
                .map(|outcome| {
                    if outcome.is_some() {
                        Response::Ok
                    } else {
                        Response::Error(Error::new("browser_error", "web action was rejected"))
                    }
                })
                .unwrap_or_else(err_response)
        }

        Request::ActivateCommandBar => with_app(window, app, cx, |view, window, cx| {
            view.activate_command_bar(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),

        // §11.4/§11.5 address-control pointer entry: the same app method the
        // pill's mouse-down listener calls (opens the overlay that covers
        // the control in Location mode with the full URL selected).
        Request::ClickAddress => with_app(window, app, cx, |view, window, cx| {
            view.open_overlay(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),

        // §13 one-pointer-action dismissal: the same app method the
        // palette/overlay dismissal layer's mouse-down listener calls
        // (closes the open surface and restores the origin focus domain).
        Request::ClickContentOutside => with_app(window, app, cx, |view, window, cx| {
            view.close_command_surface(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),

        Request::FocusWeb => with_app(window, app, cx, |view, window, cx| {
            view.focus_web(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),

        // Transient Peek drivers (Link Preview / Peek packet): each routes
        // through the exact production app method the Escape key, the
        // preview controls, and browser-command routing call.
        Request::PeekClose => with_app(window, app, cx, |view, window, cx| {
            view.peek_close(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),
        Request::PeekPromote => with_app(window, app, cx, |view, window, cx| {
            view.peek_promote(window, cx)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),
        Request::PeekNavigate { action } => with_app(window, app, cx, |view, _window, _cx| {
            view.peek_action(action)
        })
        .map(|_| Response::Ok)
        .unwrap_or_else(err_response),

        Request::SetCommandBarText { text } => {
            let result = with_app(window, app, cx, |view, _w, cx| {
                view.command_text.set_text(text.clone());
                // The seam types like the user: every text mutation restarts
                // the ranking debounce (the deterministic flush then cancels
                // it synchronously).
                view.on_command_text_changed(cx);
                view.command_bar_mode.into()
            });
            match result {
                Ok(mode) => {
                    broadcast(&server, Event::CommandBarChanged { mode, text });
                    Response::Ok
                }
                Err(error) => err_response(error),
            }
        }

        // Deterministic command-ranking flush (testkit-only): cancels the
        // pending debounce timer and publishes the current generation's
        // ranked results synchronously, so E2E asserts burst typing (only the
        // final generation ever publishes) without waiting on real time.
        Request::FlushCommandRanking => with_app(window, app, cx, |view, _w, cx| {
            view.flush_command_ranking(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // Deterministic persistence flush (testkit-only): the synchronous
        // write of the in-memory history provider (off the render path; the
        // production background writer uses the identical repository write).
        Request::FlushPersistence => with_app(window, app, cx, |view, _w, _cx| {
            view.flush_persistence();
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // Deterministic browser-state flush (testkit-only, Phase 7 warm-
        // restore slice): the synchronous write of the current structural
        // browser model (the production background writer uses the identical
        // repository write). The deterministic seam the persistence/restart
        // E2E uses to prove committed-state recovery across restarts.
        Request::FlushBrowserState => with_app(window, app, cx, |view, _w, _cx| {
            view.flush_browser_state();
            Response::Ok
        })
        .unwrap_or_else(err_response),

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
            // Note the double deref: `window` is `&WindowHandle<gpui_component::Root>`
            // (Copy), so `*window` is the typed handle again; `**window` reaches
            // the deref target `AnyWindowHandle`.
            (**window)
                .update(cx, |_root, window, cx| {
                    window.dispatch_keystroke(ks, cx);
                })
                .map(|_| Response::Ok)
                .unwrap_or_else(|e| Response::Error(Error::new("app_error", e.to_string())))
        }

        Request::GetSnapshot => with_app(window, app, cx, |view, window, _cx| {
            view.snapshot(window).to_protocol()
        })
        .map(Response::Snapshot)
        .unwrap_or_else(err_response),

        // Bring the app to the foreground (testkit-only; the shown-harness
        // portal frontmost test). Activates the browser window, never the
        // non-activating portal.
        Request::ActivateApp => with_app(window, app, cx, |_view, _window, cx| {
            cx.activate(true);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::SetWindowSize { width, height } => {
            with_app(window, app, cx, |_view, window, _cx| {
                window.resize(size(px(width), px(height)));
            })
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response)
        }

        // Real pointer drag on the resize divider through the production
        // listener methods (see `BbrApp::drag_sidebar_divider`).
        Request::DragSidebarDivider { dx } => with_app(window, app, cx, |view, window, cx| {
            view.drag_sidebar_divider(dx, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::FocusSidebarDivider => with_app(window, app, cx, |view, window, cx| {
            view.focus_sidebar_divider(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::Ping => Response::Ok,

        Request::CloseWindow => window
            .update(cx, |_root, window, _cx| window.remove_window())
            .map(|_| Response::Ok)
            .unwrap_or_else(err_response),

        Request::Quit => {
            if let Some(server) = &server {
                server.broadcast(Event::Quitting);
            }
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

        // Routed by `dispatch_portal` before this match; reaching one here
        // is a routing bug.
        Request::PortalOpenProbe
        | Request::PortalCloseProbe
        | Request::PortalOutsideDismiss
        | Request::PortalTeardown
        | Request::PortalSetFullscreen { .. }
        | Request::PortalSetMinimized { .. }
        | Request::PortalSetAllSpaces { .. }
        | Request::PortalCloseWindow
        | Request::FlushFrame => unreachable!("portal requests route through dispatch_portal"),

        // Routed by `dispatch_find` before this match; reaching one here is
        // a routing bug.
        Request::FindOpen
        | Request::FindClose
        | Request::FindSetQuery { .. }
        | Request::FindNext
        | Request::FindPrevious => unreachable!("find requests route through dispatch_find"),

        // Routed by `dispatch_errors` before this match (macOS-only: the
        // classified payload is produced by the native adapter's mapping);
        // reaching one here is a routing bug.
        #[cfg(target_os = "macos")]
        Request::InjectErrorIncident { .. }
        | Request::ClickErrorAction
        | Request::ClickErrorDismiss
        | Request::BenchmarkErrorSurfacePresent { .. } => {
            unreachable!("error-surface requests route through dispatch_errors")
        }

        // Routed by `dispatch_zoom` before this match; reaching one here is
        // a routing bug.
        Request::ZoomIn | Request::ZoomOut | Request::ZoomReset => {
            unreachable!("zoom requests route through dispatch_zoom")
        }

        // Routed by `dispatch_context_menu` before this match; reaching one
        // here is a routing bug.
        Request::ContextMenuSelect { .. }
        | Request::ContextMenuActivate { .. }
        | Request::ContextMenuClose => {
            unreachable!("context-menu requests route through dispatch_context_menu")
        }

        // Routed by `dispatch_u2` before this match; reaching one here is
        // a routing bug.
        Request::ClickFooterDownloads
        | Request::ClickFooterPlus
        | Request::ClickPlusMenuItem { .. }
        | Request::ToggleFavoriteTab { .. }
        | Request::ClickFavoriteTile { .. }
        | Request::ClickFolderRow { .. }
        | Request::ClickFolderToggle { .. }
        | Request::ClickFolderMember { .. }
        | Request::ClickFolderMemberClose { .. }
        | Request::MoveTabIntoFolder { .. }
        | Request::SetSpaceTheme { .. } => {
            unreachable!("U2 sidebar requests route through dispatch_u2")
        }

        // Routed by `dispatch_downloads` before this match; reaching one
        // here is a routing bug.
        Request::DownloadStart { .. }
        | Request::DownloadCancel { .. }
        | Request::DownloadRetry { .. }
        | Request::ClickDownloadRowAction { .. }
        | Request::ClickDownloadRowDelete { .. }
        | Request::ClickDownloadsClear
        | Request::ClickDownloadsOutside
        | Request::ScrollDownloads { .. } => {
            unreachable!("downloads requests route through dispatch_downloads")
        }

        // Routed by `dispatch_probe` before this match; reaching one here
        // is a routing bug.
        Request::ProbeWebContent { .. } => {
            unreachable!("probe requests route through dispatch_probe")
        }
        // Routed by `dispatch_auth` before this match (queue item 10);
        // reaching one here is a routing bug.
        Request::AuthPresentChallenge { .. }
        | Request::AuthRecordChallenge { .. }
        | Request::BenchmarkAuthSheetOpen { .. } => {
            unreachable!("auth requests route through dispatch_auth")
        }

        // Routed by `dispatch_drag` before this match; reaching one here is
        // a routing bug.
        Request::BeginTabDrag { .. }
        | Request::MoveTabDrag { .. }
        | Request::EndTabDrag
        | Request::CancelTabDrag => unreachable!("tab-drag requests route through dispatch_drag"),

        // Routed by `dispatch_u2` before this match (the §11.3 sidebar swipe
        // arms); reaching one here is a routing bug.
        Request::SwipeScroll { .. } | Request::SwipeCancel => {
            unreachable!("sidebar-swipe requests route through dispatch_u2")
        }

        // Routed by `dispatch_benchmark_swipe` before this match; reaching
        // one here is a routing bug.
        Request::BenchmarkSwipeCommit { .. } => {
            unreachable!("swipe-commit benchmark requests route through dispatch_benchmark_swipe")
        }

        // Routed by `dispatch_benchmark` before this match; reaching one here
        // is a routing bug.
        Request::BenchmarkSelectTab { .. }
        | Request::BenchmarkPlusMenu { .. }
        | Request::BenchmarkFavoriteToggle { .. }
        | Request::BenchmarkFolderToggle { .. }
        | Request::BenchmarkClickTab { .. }
        | Request::BenchmarkReopenTab { .. }
        | Request::BenchmarkActivateSpace { .. }
        | Request::BenchmarkPageReady { .. }
        | Request::BenchmarkToggleSidebar { .. }
        | Request::BenchmarkDispatchKeystroke { .. }
        | Request::BenchmarkResizeSidebar { .. }
        | Request::BenchmarkRestoreSidebar { .. }
        | Request::BenchmarkPinTab { .. }
        | Request::BenchmarkReorderTab { .. }
        | Request::BenchmarkMoveTab { .. }
        | Request::BenchmarkDragMoveTab { .. }
        | Request::BenchmarkDenseScroll { .. }
        | Request::BenchmarkDenseSelect { .. }
        | Request::BenchmarkHoverTab { .. }
        | Request::BenchmarkPressTab { .. }
        | Request::BenchmarkCommandType { .. }
        | Request::BenchmarkCommandBarFirstResult { .. }
        | Request::BenchmarkCommandBarFullSearch { .. }
        | Request::BenchmarkCommandActivate { .. }
        | Request::BenchmarkCommandActivateNewTab { .. }
        | Request::BenchmarkPaletteOpen { .. }
        | Request::BenchmarkOverlayOpen { .. }
        | Request::BenchmarkFindOpen { .. }
        | Request::BenchmarkFindQueryResult { .. }
        | Request::BenchmarkFindStep { .. }
        | Request::BenchmarkPeekRequestToPresent { .. }
        | Request::BenchmarkPeekDismissToOrigin { .. }
        | Request::BenchmarkPeekPromoteToTab { .. }
        | Request::BenchmarkZoomStep { .. }
        | Request::BenchmarkContextMenuOpen { .. }
        | Request::BenchmarkDownloadsPanelOpen { .. }
        | Request::BenchmarkDownloadsProgress { .. }
        | Request::BenchmarkDownloadsRowRetry { .. }
        | Request::BenchmarkDownloadsRowRestart { .. }
        | Request::BenchmarkDownloadsRowDelete { .. }
        | Request::BenchmarkDownloadsClear { .. }
        | Request::SeedDownloadsAttempt { .. }
        | Request::SeedDownloadsCancelled { .. } => {
            unreachable!("benchmark requests route through dispatch_benchmark")
        }
        // Saved Page save/remove/open/flush + benchmark arms route through
        // `dispatch_saved_pages` before this fall-through (B2 packet
        // `saved-pages-actions-command`).
        Request::SaveSavedPage { .. }
        | Request::SavedPageAvailability { .. }
        | Request::RemoveSavedPage { .. }
        | Request::OpenSavedPage { .. }
        | Request::FlushSavedPages
        | Request::BenchmarkSavedPageSearchFirstResult { .. }
        | Request::BenchmarkSavedPageOpen { .. } => {
            unreachable!("saved-page requests route through dispatch_saved_pages")
        }
        // External-URL delivery is macOS-only; the pump intercepts
        // `DeliverExternalUrl` before dispatch on macOS, so reaching here
        // means the platform cannot deliver (queue item 13).
        Request::DeliverExternalUrl { .. } => Response::Error(Error::new(
            "unsupported",
            "external url delivery is macOS-only",
        )),
        #[cfg(not(target_os = "macos"))]
        Request::InjectErrorIncident { .. }
        | Request::ClickErrorAction
        | Request::ClickErrorDismiss
        | Request::BenchmarkErrorSurfacePresent { .. } => Response::Error(Error::new(
            "unsupported",
            "error-surface requests require the macOS engine",
        )),
    }
}
