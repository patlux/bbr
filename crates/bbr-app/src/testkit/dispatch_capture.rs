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
        | Request::CaptureElements { path, .. }
        | Request::CaptureAbout { path }
        | Request::CapturePortal { path }
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
    if let Request::CaptureElements { ids, margin, .. } = req {
        if ids.is_empty() || ids.len() > MAX_ELEMENTS || !(0.0..=MAX_MARGIN_PT).contains(margin) {
            return Some(Response::Error(Error::new(
                "invalid_capture_region",
                "name 1-8 element ids and a margin of 0-32 points",
            )));
        }
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
    } else if matches!(req, Request::CapturePortal { .. }) {
        // Presentations (Find, the palette, Peek) and the transient overlays
        // the portal draws without one (permission prompt, site settings,
        // sheets) share the one geometry-synced child frame; the caller
        // proves which surface it holds.
        let target = crate::window_seam::with_app(window, app, cx, |view, window, _cx| {
            let snapshot = view.snapshot(window);
            (snapshot.portal.frame_synced, view.portal_frame_handle())
        });
        match target {
            Ok((true, Some(handle))) => handle,
            Ok(_) => {
                return Some(Response::Error(Error::new(
                    "portal_closed",
                    "no geometry-synced portal frame to read back",
                )))
            }
            Err(error) => {
                return Some(Response::Error(Error::new(
                    "capture_failed",
                    error.to_string(),
                )))
            }
        }
    } else if matches!(req, Request::CaptureAbout { .. }) {
        let Some(handle) = *app.read(cx).about_window.read() else {
            return Some(Response::Error(Error::new(
                "about_closed",
                "open About BBR through its production menu item before readback",
            )));
        };
        handle.into()
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
            if matches!(
                req,
                Request::CaptureChrome { .. } | Request::CaptureElements { .. }
            ) {
                // Observe only after drawing without an app lease. Never
                // reset/clone/jump the retained production transition.
                let (open, observed) = app.update(cx, |view, cx| {
                    let observed = view
                        .downloads
                        .opacity
                        .as_ref()
                        .map(|fade| fade.rendered_sample(cx));
                    (view.downloads.open, observed)
                });
                if !downloads_capture_ready(open, observed) {
                    return Err(format!("downloads opacity not ready: {observed:?}"));
                }
            }
            let image = window
                .render_to_image()
                .map_err(|error| error.to_string())?;
            let Request::CaptureElements { ids, margin, .. } = req else {
                image.save(path).map_err(|error| error.to_string())?;
                return Ok(Response::Ok);
            };
            // The probes recorded this window's rects while the frame above
            // was drawn; only the named elements' pixels are kept.
            let rects = cx
                .try_global::<bbr_ui::rect_probe::ElementRects>()
                .map(|rects| rects.for_window(window.window_handle().window_id().as_u64()))
                .unwrap_or_default();
            let [x, y, width, height] = element_region(
                &rects,
                ids,
                *margin,
                window.scale_factor(),
                (image.width(), image.height()),
            )?;
            image::imageops::crop_imm(&image, x, y, width, height)
                .to_image()
                .save(path)
                .map_err(|error| error.to_string())?;
            Ok(Response::CapturedRegion {
                pixels: [x, y, width, height],
            })
        })
        .map_err(|error| error.to_string())
        .and_then(|result| result)
        .unwrap_or_else(|message| {
            let code = if message.starts_with(NOT_DRAWN) {
                "element_not_drawn"
            } else {
                "capture_failed"
            };
            Response::Error(Error::new(code, message))
        }),
    )
}

const MAX_ELEMENTS: usize = 8;
const MAX_MARGIN_PT: f32 = 32.0;
const NOT_DRAWN: &str = "element not drawn in this frame: ";

/// The window-pixel rectangle `[x, y, width, height]` that covers the union
/// of the named elements' rects (points, top-left origin) grown by
/// `margin` points, scaled to device pixels outward and clipped to the
/// readback. Every id must have drawn; nothing is resampled.
fn element_region(
    rects: &[(String, [f32; 4])],
    ids: &[String],
    margin: f32,
    scale: f32,
    (image_width, image_height): (u32, u32),
) -> Result<[u32; 4], String> {
    let mut union: Option<[f32; 4]> = None;
    for id in ids {
        let [x, y, w, h] = rects
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, rect)| *rect)
            .ok_or_else(|| format!("{NOT_DRAWN}{id}"))?;
        union = Some(match union {
            None => [x, y, x + w, y + h],
            Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x + w), y1.max(y + h)],
        });
    }
    let [x0, y0, x1, y1] = union.ok_or_else(|| format!("{NOT_DRAWN}(none named)"))?;
    let device = |value: f32, round: fn(f32) -> f32, limit: u32| {
        (round(value * scale).max(0.0) as u32).min(limit)
    };
    let left = device(x0 - margin, f32::floor, image_width);
    let top = device(y0 - margin, f32::floor, image_height);
    let right = device(x1 + margin, f32::ceil, image_width);
    let bottom = device(y1 + margin, f32::ceil, image_height);
    if right <= left || bottom <= top {
        return Err(format!("empty element region for {ids:?}"));
    }
    Ok([left, top, right - left, bottom - top])
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

fn downloads_capture_ready(open: bool, fade: Option<(Option<f32>, f32, f32)>) -> bool {
    matches!(
        (open, fade),
        (false, None) | (true, Some((Some(1.0), 1.0, 1.0))) | (false, Some((Some(0.0), 0.0, 1.0)))
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
    fn downloads_readback_requires_the_rendered_opacity_endpoint() {
        assert!(downloads_capture_ready(false, None));
        assert!(downloads_capture_ready(false, Some((Some(0.0), 0.0, 1.0))));
        assert!(downloads_capture_ready(true, Some((Some(1.0), 1.0, 1.0))));
        for fade in [
            Some((None, 0.0, 1.0)),
            Some((Some(1.0), 0.0, 1.0)),
            Some((Some(0.5), 0.0, 1.0)),
            Some((Some(0.0), 0.0, 0.5)),
            Some((Some(0.0), 1.0, 1.0)),
        ] {
            assert!(!downloads_capture_ready(false, fade), "{fade:?}");
        }
        for fade in [
            None,
            Some((None, 1.0, 1.0)),
            Some((Some(0.0), 1.0, 1.0)),
            Some((Some(0.5), 1.0, 1.0)),
            Some((Some(1.0), 1.0, 0.5)),
            Some((Some(1.0), 0.0, 1.0)),
        ] {
            assert!(!downloads_capture_ready(true, fade), "{fade:?}");
        }
    }

    #[test]
    fn element_region_is_the_grown_union_in_device_pixels() {
        let rects = vec![
            ("bbr-footer".to_string(), [0.0, 1359.0, 300.0, 48.0]),
            ("bbr-plus-menu".to_string(), [8.0, 1270.5, 284.0, 84.0]),
            ("bbr-content-card".to_string(), [300.0, 10.0, 770.0, 1387.0]),
        ];
        let ids = |list: &[&str]| list.iter().map(|id| id.to_string()).collect::<Vec<_>>();
        let window = (2160, 2814);
        // One element: its rect grown by 4 pt, clipped at the window edges.
        assert_eq!(
            element_region(&rects, &ids(&["bbr-footer"]), 4.0, 2.0, window),
            Ok([0, 2710, 608, 104])
        );
        // Several: the union; fractional edges round outward.
        assert_eq!(
            element_region(
                &rects,
                &ids(&["bbr-footer", "bbr-plus-menu"]),
                4.0,
                2.0,
                window
            ),
            Ok([0, 2533, 608, 281])
        );
        assert_eq!(
            element_region(&rects, &ids(&["bbr-content-card"]), 4.0, 2.0, window),
            Ok([592, 12, 1556, 2790])
        );
        // A missing element never falls back to a larger readback.
        assert_eq!(
            element_region(
                &rects,
                &ids(&["bbr-footer", "bbr-downloads-panel"]),
                4.0,
                2.0,
                window
            ),
            Err(format!("{NOT_DRAWN}bbr-downloads-panel"))
        );
        assert!(element_region(&rects, &[], 4.0, 2.0, window).is_err());
        assert!(
            element_region(
                &[("x".into(), [-50.0, 0.0, 10.0, 10.0])],
                &ids(&["x"]),
                0.0,
                2.0,
                window
            )
            .is_err(),
            "a rect wholly outside the window is empty"
        );
    }

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
