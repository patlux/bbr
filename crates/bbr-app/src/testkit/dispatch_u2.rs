//! U2 sidebar request dispatch (DESIGN.md §11.4/§11.10/§11.11).
//!
//! The footer controls, plus-menu item commits, favorites toggles, and
//! folder row/member/move requests route through the same production app
//! methods the element listeners call (never a controller mutation). Split
//! from `super::dispatch` so no gateway module outgrows the ~500-line
//! budget, mirroring the `dispatch_benchmark`/`dispatch_drag` routers.

use bbr_browser::FolderId;
use bbr_testkit::protocol::{Request, Response, SwipePhaseWire};
use gpui::{point, px, App, Entity, ScrollDelta, ScrollWheelEvent, TouchPhase, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_util::err_response;

/// Handle the U2 sidebar `Request` arms; `None` for every other request so
/// `super::dispatch` falls through to its own routing.
pub(crate) fn dispatch_u2(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        // §11.4 footer controls through the production pointer paths: the
        // exact app methods the footer row's mouse-down listeners call.
        // There is no `[Spaces]` toggle — the §11.3 space strip is always
        // part of the rail and space activation routes through `ClickSpace`
        // (the chip pointer path). `[+]` marks its press inside (like the
        // render listener) so the root outside-press handler never dismisses
        // the freshly opened menu.
        Request::ClickFooterDownloads => with_app(window, app, cx, |view, window, cx| {
            view.activate_downloads(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFooterPlus => with_app(window, app, cx, |view, window, cx| {
            view.toggle_plus_menu(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.3 appearance seam: the production controller appearance
        // authority (explicit theme records are authoritative over the
        // derived profile-hue default). Rejects unknown theme names whole.
        Request::SetSpaceTheme { space, theme } => {
            with_app(window, app, cx, |view, _window, cx| {
                if let Err(message) = view.set_space_theme(bbr_browser::SpaceId(space.0), theme, cx)
                {
                    return Response::Error(bbr_testkit::protocol::Error::new(
                        "app_error",
                        format!("set-space-theme: {message}"),
                    ));
                }
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        // §11.4 plus-menu item commit through the production pointer path:
        // the same selection + dispatch sequence the item's mouse-down
        // listener runs (selection is app-owned, clamped to the v1 range).
        Request::ClickPlusMenuItem { index } => with_app(window, app, cx, |view, window, cx| {
            view.commit_plus_menu_item(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.3 Space-configuration surface through the production seams:
        // the management-affordance entry (the same `open_space_config` the
        // create paths call after creation), the draft theme/identity
        // selections, the Done commit, the Cancel close, and the exact
        // outside-press dismissal. All route through the production app
        // methods the surface listeners call — never a controller mutation.
        Request::OpenSpaceConfig { space } => with_app(window, app, cx, |view, window, cx| {
            if view.open_space_config(bbr_browser::SpaceId(space.0), window, cx) {
                Response::Ok
            } else {
                Response::Error(bbr_testkit::protocol::Error::new(
                    "unknown_space",
                    format!("open-space-config: unknown space {space:?}"),
                ))
            }
        })
        .unwrap_or_else(err_response),
        Request::SpaceConfigSelectTheme { theme } => {
            with_app(window, app, cx, |view, _window, cx| {
                let Some(token) = bbr_domain::SpaceTheme::ALL
                    .iter()
                    .find(|token| token.name() == theme)
                    .copied()
                else {
                    return Response::Error(bbr_testkit::protocol::Error::new(
                        "unknown_theme",
                        format!("space-config-select-theme: unknown theme {theme:?}"),
                    ));
                };
                view.space_config_select_theme(token, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::SpaceConfigSelectIntensity { intensity } => {
            with_app(window, app, cx, |view, _window, cx| {
                let Some(step) = bbr_domain::TintIntensity::from_name(intensity) else {
                    return Response::Error(bbr_testkit::protocol::Error::new(
                        "unknown_intensity",
                        format!("space-config-select-intensity: unknown step {intensity:?}"),
                    ));
                };
                view.space_config_select_intensity(step, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }
        Request::SpaceConfigToggleGrain => with_app(window, app, cx, |view, _window, cx| {
            view.space_config_toggle_grain(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SpaceConfigSelectIdentity { identity } => {
            with_app(window, app, cx, |view, _window, cx| {
                let resolved = bbr_domain::SpaceEmoji::new(identity.clone())
                    .map(bbr_domain::SpaceIdentity::Emoji)
                    .or_else(|_| {
                        bbr_domain::BbrGlyph::ALL
                            .iter()
                            .find(|glyph| glyph.name() == identity)
                            .copied()
                            .map(bbr_domain::SpaceIdentity::Glyph)
                            .ok_or(bbr_domain::AppearanceError::InvalidEmoji(identity.clone()))
                    });
                match resolved {
                    Ok(identity) => {
                        view.space_config_select_identity(identity, cx);
                        Response::Ok
                    }
                    Err(_) => Response::Error(bbr_testkit::protocol::Error::new(
                        "unknown_identity",
                        format!("space-config-select-identity: unknown identity {identity:?}"),
                    )),
                }
            })
            .unwrap_or_else(err_response)
        }
        Request::SpaceConfigDone => with_app(window, app, cx, |view, window, cx| {
            view.space_config_done(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SpaceConfigCancel => with_app(window, app, cx, |view, window, cx| {
            view.space_config_cancel(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::SpaceConfigPressOutside => with_app(window, app, cx, |view, window, cx| {
            view.dismiss_space_config_on_outside_press(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::ChromeBackgroundPress => with_app(window, app, cx, |view, window, cx| {
            view.on_root_left_press(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.3 sidebar swipe: drives one synthetic GPUI scroll-wheel event
        // through the exact production handler the sidebar's `on_scroll_wheel`
        // element listener calls (`BbrApp::on_sidebar_scroll_wheel`). Public
        // synthetic event dispatch is unavailable, so the testkit drives the
        // production method itself — same dominance filter, phase gating,
        // threshold, and `activate_space` authority; never a controller
        // mutation. `SwipeCancel` drives the explicit production cancel seam.
        Request::SwipeScroll { delta, phase } => with_app(window, app, cx, |view, window, cx| {
            let event = ScrollWheelEvent {
                position: point(px(0.0), px(0.0)),
                delta: ScrollDelta::Pixels(point(px(delta[0]), px(delta[1]))),
                modifiers: gpui::Modifiers::default(),
                touch_phase: match phase {
                    SwipePhaseWire::Started => TouchPhase::Started,
                    SwipePhaseWire::Moved => TouchPhase::Moved,
                    SwipePhaseWire::Ended => TouchPhase::Ended,
                },
            };
            view.on_sidebar_scroll_wheel(&event, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::SwipeCancel => with_app(window, app, cx, |view, _window, cx| {
            view.cancel_swipe(cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.10 favorites through the production paths: the toggle is the
        // exact app method the drag-into-grid drop commits through; the tile
        // click is the activation listener path.
        Request::ToggleFavoriteTab { index } => with_app(window, app, cx, |view, window, cx| {
            view.favorite_toggle_tab(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFavoriteTile { index } => with_app(window, app, cx, |view, window, cx| {
            view.activate_favorite(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.11 folder rows through the production pointer paths: row entry
        // focuses the folders region, the disclosure toggles the row, nested
        // rows select/close their member tabs (the close stops propagation
        // exactly like the render listener). The folder is keyed by its
        // stable domain id.
        Request::ClickFolderRow { id } => with_app(window, app, cx, |view, window, cx| {
            view.focus_folder_row(FolderId(*id), window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFolderToggle { id } => with_app(window, app, cx, |view, window, cx| {
            let id = FolderId(*id);
            view.focus_folder_row(id, window, cx);
            view.toggle_folder(id, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFolderMember { folder, member } => {
            with_app(window, app, cx, |view, window, cx| {
                view.select_folder_member(FolderId(*folder), *member, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        Request::ClickFolderMemberClose { folder, member } => {
            with_app(window, app, cx, |view, window, cx| {
                view.close_folder_member(FolderId(*folder), *member, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        // §11.11 move-into-folder through the L5 `MoveIntoFolder` command
        // path (grouping pins a regular tab in the same transition).
        // `folder` is the stable domain folder id.
        Request::MoveTabIntoFolder { index, folder } => {
            with_app(window, app, cx, |view, window, cx| {
                view.move_tab_into_folder(*index, FolderId(*folder), window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        // Ungroup through the same organization authority: the member
        // stays a loose pinned tab.
        Request::RemoveTabFromFolder { index, folder } => {
            with_app(window, app, cx, |view, window, cx| {
                view.remove_tab_from_folder(*index, FolderId(*folder), window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        _ => return None,
    };
    Some(response)
}
