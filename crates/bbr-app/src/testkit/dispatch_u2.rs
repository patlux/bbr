//! U2 sidebar request dispatch (DESIGN.md §11.4/§11.10/§11.11).
//!
//! The footer controls, plus-menu item commits, favorites toggles, and
//! folder row/member/move requests route through the same production app
//! methods the element listeners call (never a controller mutation). Split
//! from `super::dispatch` so no gateway module outgrows the ~500-line
//! budget, mirroring the `dispatch_benchmark`/`dispatch_drag` routers.

use bbr_testkit::protocol::{Request, Response};
use gpui::{App, Entity, WindowHandle};

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
        // `[Spaces]` toggles the §11.3 region, `[+]` marks its press inside
        // (like the render listener) so the root outside-press handler never
        // dismisses the freshly opened menu.
        Request::ClickFooterDownloads => with_app(window, app, cx, |view, window, cx| {
            view.activate_downloads(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFooterSpaces => with_app(window, app, cx, |view, window, cx| {
            view.toggle_spaces_region(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFooterPlus => with_app(window, app, cx, |view, window, cx| {
            view.toggle_plus_menu(window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        // §11.4 plus-menu item commit through the production pointer path:
        // the same selection + dispatch sequence the item's mouse-down
        // listener runs (selection is app-owned, clamped to the v1 range).
        Request::ClickPlusMenuItem { index } => with_app(window, app, cx, |view, window, cx| {
            view.commit_plus_menu_item(*index, window, cx);
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
        // exactly like the render listener).
        Request::ClickFolderRow { index } => with_app(window, app, cx, |view, window, cx| {
            view.focus_folder_row(*index, window, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFolderToggle { index } => with_app(window, app, cx, |view, window, cx| {
            view.focus_folder_row(*index, window, cx);
            view.toggle_folder(*index, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),

        Request::ClickFolderMember { folder, member } => {
            with_app(window, app, cx, |view, window, cx| {
                view.select_folder_member(*folder, *member, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        Request::ClickFolderMemberClose { folder, member } => {
            with_app(window, app, cx, |view, window, cx| {
                view.close_folder_member(*folder, *member, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        // §11.11 move-into-folder through the L5 `MoveIntoFolder` command
        // path (the drag drop into a folder row routes through the same
        // command). `folder` is the creation-order index.
        Request::MoveTabIntoFolder { index, folder } => {
            with_app(window, app, cx, |view, window, cx| {
                view.move_tab_into_folder(*index, *folder, window, cx);
                Response::Ok
            })
            .unwrap_or_else(err_response)
        }

        _ => return None,
    };
    Some(response)
}
