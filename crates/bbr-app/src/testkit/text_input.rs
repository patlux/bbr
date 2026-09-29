//! Blueprint D02: the `CommandTextInput` seam. Sends one input-method
//! message to the browser window's GPUI root view through its real
//! `NSTextInputClient` conformance — the AppKit → GPUI → registered input
//! handler path an input method takes — then records the client's readback
//! for the snapshot. The native call must run outside any app borrow (the
//! view re-enters the app through the input handler), so this runs on the
//! pump between updates rather than inside `dispatch`.

use bbr_testkit::protocol::{Error, Response, TextInputOp, TextInputReadback};
use gpui::{AsyncApp, Entity, WindowHandle};

use crate::app::BbrApp;
use crate::window_seam::with_app;

pub(super) fn drive(
    op: TextInputOp,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
    async_cx: &mut AsyncApp,
) -> Response {
    #[cfg(target_os = "macos")]
    {
        let view = async_cx.update(|cx| {
            (**window)
                .update(cx, |_root, window, _cx| root_view(window))
                .ok()
                .flatten()
        });
        let Some(view) = view else {
            return Response::Error(Error::new("no_view", "the browser window has no root view"));
        };
        // SAFETY: `view` is the live GPUI root NSView of the browser window;
        // the pump runs on the main thread and holds no app borrow here.
        let readback = unsafe {
            use bbr_web_macos::text_input_client as client;
            match &op {
                TextInputOp::SetMarked { text, selected } => {
                    client::set_marked_text(view, text, *selected)
                }
                TextInputOp::Insert { text } => client::insert_text(view, text),
                TextInputOp::Unmark => client::unmark_text(view),
                TextInputOp::Probe => {}
            }
            client::read_back(view)
        };
        async_cx.update(|cx| {
            let _ = with_app(window, app, cx, |view, _window, _cx| {
                view.text_input_readback = Some(TextInputReadback {
                    marked: readback.marked,
                    selected: readback.selected,
                    first_rect: readback.first_rect,
                    window_frame: readback.window_frame,
                });
            });
        });
        Response::Ok
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (op, window, app, async_cx);
        Response::Error(Error::new("unsupported", "no native text-input client"))
    }
}

#[cfg(target_os = "macos")]
fn root_view(window: &gpui::Window) -> Option<*mut std::ffi::c_void> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match HasWindowHandle::window_handle(window).ok()?.as_raw() {
        RawWindowHandle::AppKit(handle) => Some(handle.ns_view.as_ptr()),
        _ => None,
    }
}
