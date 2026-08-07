//! Shared gateway-dispatch helpers: the dense-frame flush and the
//! controller-mutation → `Response` conversion used by every `dispatch_*`
//! router. Kept in their own module so the main router stays under the
//! ~500-line budget and the sibling routers import one shared home instead
//! of re-implementing the conversions.

use bbr_browser::ControllerEvent;
use bbr_testkit::protocol::{Error, Response};
use gpui::{App, WindowHandle};

/// Draw one real GPUI frame synchronously through the untyped window path
/// (the draw re-borrows the root entity, so the typed `WindowHandle::update`
/// path would double-lease). Used by the dense scroll/select arms so the
/// snapshot's dense window state reflects the post-mutation render.
pub(crate) fn flush_dense_frame(window: &WindowHandle<gpui_component::Root>, cx: &mut App) {
    let _ = (**window).update(cx, |_root, window, cx| {
        let app: &mut App = cx;
        let _ = window.draw(app);
    });
}

/// Convert a controller-mutation outcome into the wire `Response`.
pub(crate) fn browser_mutation<E: std::fmt::Display>(
    update: Result<Option<ControllerEvent>, E>,
) -> Response {
    match update {
        Ok(Some(_)) => Response::Ok,
        Ok(None) => Response::Error(Error::new("browser_error", "browser mutation was rejected")),
        Err(error) => err_response(error),
    }
}

/// Convert an error into the wire `Response`.
pub(crate) fn err_response<E: std::fmt::Display>(e: E) -> Response {
    Response::Error(Error::new("app_error", e.to_string()))
}
