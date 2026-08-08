//! Testkit routing for the find-in-page presentation (task packet
//! `find-in-page-portal-bar`).
//!
//! Portal draws happen only after the mutable `BbrApp` lease is released:
//! the portal root reads that same entity during render, so drawing inside
//! `with_app` would re-enter GPUI's entity map (same rule as
//! `dispatch_portal`).

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

use super::dispatch_portal::flush_portal_frame;
use super::dispatch_util::err_response;

pub(super) fn dispatch_find(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let response = match req {
        Request::FindOpen => with_app(window, app, cx, |view, window, cx| {
            view.find_open(window, cx);
            view.portal_frame_handle()
        })
        .map(|handle| {
            flush_portal_frame(handle, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::FindClose => with_app(window, app, cx, |view, window, cx| {
            let open = view.find_state_open();
            view.find_close(window, cx);
            (open, view.portal_frame_handle())
        })
        .map(|(open, handle)| {
            flush_portal_frame(handle, cx);
            find_response(open, "no find presentation was open")
        })
        .unwrap_or_else(err_response),
        Request::FindSetQuery { query } => with_app(window, app, cx, |view, _window, cx| {
            view.find_set_query(query, cx);
            view.portal_frame_handle()
        })
        .map(|handle| {
            flush_portal_frame(handle, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::FindNext => with_app(window, app, cx, |view, _window, cx| {
            view.find_next(cx);
            view.portal_frame_handle()
        })
        .map(|handle| {
            flush_portal_frame(handle, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        Request::FindPrevious => with_app(window, app, cx, |view, _window, cx| {
            view.find_previous(cx);
            view.portal_frame_handle()
        })
        .map(|handle| {
            flush_portal_frame(handle, cx);
            Response::Ok
        })
        .unwrap_or_else(err_response),
        _ => return None,
    };
    Some(response)
}

fn find_response(open: bool, message: &str) -> Response {
    if open {
        Response::Ok
    } else {
        Response::Error(Error::new("find_inactive", message))
    }
}
