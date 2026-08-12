//! Explicit `visual-capture`-feature evidence capture.
//!
//! Captures the current frame of the running production GPUI window directly
//! from its Metal renderer. This avoids ScreenCaptureKit permission while
//! preserving the exact production chrome scene. Native WKWebView/AppKit child
//! pixels are deliberately not synthesized into the image; their ownership and
//! geometry remain separate typed snapshot evidence.

use std::path::{Component, Path};

use bbr_testkit::protocol::{Error, Request, Response};
use gpui::{App, Entity, WindowHandle};

use crate::app::BbrApp;

/// Handle the visual-capture renderer request; `None` for all others.
pub(crate) fn dispatch_capture(
    req: &Request,
    window: &WindowHandle<gpui_component::Root>,
    _app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    let Request::CaptureChrome { path } = req else {
        return None;
    };
    let path = Path::new(path);
    if !valid_capture_path(path) {
        return Some(Response::Error(Error::new(
            "invalid_capture_path",
            "CaptureChrome only accepts a direct /tmp/*.png path",
        )));
    }

    Some(
        window
            .update(cx, |_root, window, _cx| {
                window
                    .render_to_image()
                    .map_err(|error| error.to_string())?
                    .save(path)
                    .map_err(|error| error.to_string())
            })
            .map_err(|error| error.to_string())
            .and_then(|result| result)
            .map(|()| Response::Ok)
            .unwrap_or_else(|message| Response::Error(Error::new("capture_failed", message))),
    )
}

fn valid_capture_path(path: &Path) -> bool {
    let mut components = path.components();
    matches!(components.next(), Some(Component::RootDir))
        && matches!(components.next(), Some(Component::Normal(part)) if part == "tmp")
        && components.clone().count() == 1
        && path.extension().is_some_and(|extension| extension == "png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_path_is_limited_to_one_png_beneath_tmp() {
        assert!(valid_capture_path(Path::new("/tmp/bbr-visual.png")));
        for rejected in [
            "/tmp/nested/bbr.png",
            "/tmp/bbr.jpg",
            "/Users/patwoz/bbr.png",
            "relative.png",
            "/tmp/../bbr.png",
        ] {
            assert!(!valid_capture_path(Path::new(rejected)), "{rejected}");
        }
    }
}
