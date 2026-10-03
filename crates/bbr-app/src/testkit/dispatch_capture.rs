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
    app: &Entity<BbrApp>,
    cx: &mut App,
) -> Option<Response> {
    if let Request::ScrollSettings { delta_y } = req {
        return Some(scroll_settings(*delta_y, app, cx));
    }
    let path = match req {
        Request::CaptureChrome { path }
        | Request::CaptureSettings { path }
        | Request::CaptureFindBar { path }
        | Request::CaptureCommandPalette { path } => path,
        _ => return None,
    };
    let path = Path::new(path);
    if !valid_capture_path(path) {
        return Some(Response::Error(Error::new(
            "invalid_capture_path",
            "visual readback only accepts a direct /tmp/*.png path",
        )));
    }

    let target = if matches!(req, Request::CaptureCommandPalette { .. }) {
        let target = crate::window_seam::with_app(window, app, cx, |view, window, _cx| {
            let snapshot = view.snapshot(window);
            (
                snapshot.command_surface,
                snapshot.portal.presentation,
                view.portal_frame_handle(),
            )
        });
        match target {
            Ok((bbr_testkit::protocol::CommandSurface::Palette, Some(id), Some(handle)))
                if id == crate::app::palette::PALETTE_PRESENTATION_ID.0 =>
            {
                handle
            }
            Ok(_) => {
                return Some(Response::Error(Error::new(
                    "palette_closed",
                    "open the centered Command Palette before child-window readback",
                )))
            }
            Err(error) => {
                return Some(Response::Error(Error::new(
                    "capture_failed",
                    error.to_string(),
                )))
            }
        }
    } else if matches!(req, Request::CaptureFindBar { .. }) {
        let target = crate::window_seam::with_app(window, app, cx, |view, window, _cx| {
            let snapshot = view.snapshot(window);
            (
                snapshot.find.open,
                snapshot.portal.presentation,
                view.portal_frame_handle(),
            )
        });
        match target {
            Ok((true, Some(id), Some(handle)))
                if id == crate::app::find::FIND_PRESENTATION_ID.0 =>
            {
                handle
            }
            Ok(_) => {
                return Some(Response::Error(Error::new(
                    "find_closed",
                    "open the Find bar before child-window readback",
                )))
            }
            Err(error) => {
                return Some(Response::Error(Error::new(
                    "capture_failed",
                    error.to_string(),
                )))
            }
        }
    } else if matches!(req, Request::CaptureSettings { .. }) {
        let Some(handle) = app.read(cx).settings.read().window else {
            return Some(Response::Error(Error::new(
                "settings_closed",
                "open Settings through its production action before readback",
            )));
        };
        handle.into()
    } else {
        (*window).into()
    };
    Some(
        gpui::AnyWindowHandle::update(target, cx, |_root, window, cx| {
            // Separate Settings/Portal windows are drawn without leasing their
            // roots or BbrApp. Read their actual scene, never the parent frame.
            let _clear = window.draw(cx);
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

fn scroll_settings(delta_y: i32, app: &Entity<BbrApp>, cx: &mut App) -> Response {
    if !(-2000..=2000).contains(&delta_y) {
        return Response::Error(Error::new(
            "invalid_scroll",
            "wheel delta exceeds bounded Settings input",
        ));
    }
    let Some(handle) = app.read(cx).settings.read().window else {
        return Response::Error(Error::new(
            "settings_closed",
            "open Settings before wheel input",
        ));
    };
    let result = handle
        .update(cx, |view, _, _| view.scroll.clone())
        .and_then(|scroll| {
            gpui::AnyWindowHandle::update(handle.into(), cx, |_root, window, cx| {
                // Draw without leasing SettingsView, exactly like readback above.
                let _clear = window.draw(cx);
                let bounds = scroll.bounds();
                if bounds.size.width <= gpui::px(0.0) || bounds.size.height <= gpui::px(0.0) {
                    return Response::Error(Error::new(
                        "settings_scroll_unmeasured",
                        "no rendered content viewport",
                    ));
                }
                if delta_y != 0 {
                    window.dispatch_event(
                        gpui::PlatformInput::ScrollWheel(gpui::ScrollWheelEvent {
                            position: bounds.center(),
                            delta: gpui::ScrollDelta::Pixels(gpui::point(
                                gpui::px(0.0),
                                gpui::px(delta_y as f32),
                            )),
                            modifiers: gpui::Modifiers::default(),
                            touch_phase: gpui::TouchPhase::Moved,
                        }),
                        cx,
                    );
                    let _clear = window.draw(cx);
                }
                let offset = scroll.offset();
                let max = scroll.max_offset();
                let bounds = scroll.bounds();
                Response::SettingsScroll {
                    offset: [offset.x.into(), offset.y.into()],
                    max_offset: [max.x.into(), max.y.into()],
                    viewport: [
                        bounds.origin.x.into(),
                        bounds.origin.y.into(),
                        bounds.size.width.into(),
                        bounds.size.height.into(),
                    ],
                }
            })
        });
    result.unwrap_or_else(|error| {
        Response::Error(Error::new("settings_scroll_failed", error.to_string()))
    })
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
