//! Blueprint D02: install the floating rail's native pointer observer for
//! the browser window. The observer runs inside AppKit's event dispatch, so
//! it only sends each sample down a channel; a main-thread task hands it to
//! `BbrApp::on_rail_pointer`. The observer lives as long as the app entity.

use gpui::{App, Entity, WindowHandle};

use crate::app::rail_reveal::PointerSample;
use crate::app::BbrApp;
use crate::window_seam::with_app;

/// Install the observer and its pump.
#[cfg(target_os = "macos")]
pub(crate) fn install(
    cx: &mut App,
    window: &WindowHandle<gpui_component::Root>,
    app: &Entity<BbrApp>,
) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let view = window
        .update(
            cx,
            |_root, window, _cx| match HasWindowHandle::window_handle(window)
                .map(|handle| handle.as_raw())
            {
                Ok(RawWindowHandle::AppKit(handle)) => Some(handle.ns_view.as_ptr()),
                _ => None,
            },
        )
        .ok()
        .flatten();
    let Some(view) = view else {
        return;
    };
    let (tx, rx) = smol::channel::unbounded::<(PointerSample, bool)>();
    // SAFETY: `view` is the live GPUI root view of the browser window, on
    // the main thread; the sink only sends down a channel.
    let observer = unsafe {
        bbr_web_macos::pointer_observer::PointerObserver::install(view, move |pointer| {
            let _ = tx.try_send((
                PointerSample {
                    x: pointer.x as f32,
                    y: pointer.y as f32,
                    time_ms: pointer.time_ms,
                    pressed: pointer.pressed,
                },
                pointer.down,
            ));
        })
    };
    let Some(observer) = observer else {
        return;
    };
    let _ = with_app(window, app, cx, |view, _window, _cx| {
        view.floating_rail.observer = Some(Box::new(observer));
    });
    // Switching apps or windows ends a floating rail (no stale reveal on
    // return).
    let _ = with_app(window, app, cx, |_view, window, cx| {
        cx.observe_window_activation(window, |view, window, cx| {
            if !window.is_window_active() {
                view.end_floating_rail(window, cx);
            }
        })
        .detach();
    });
    let window = *window;
    let app = app.clone();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(sample) = rx.recv().await {
            // Coalesce a burst to its latest sample: velocity needs the
            // path, so keep every sample, but in one app update.
            let mut samples = vec![sample];
            while let Ok(next) = rx.try_recv() {
                samples.push(next);
            }
            cx.update(|cx| {
                let _ = with_app(&window, &app, cx, |view, window, cx| {
                    for (sample, down) in samples {
                        // Blueprint D11: a press closes site settings when it
                        // lands outside them; moves feed the rail reveal.
                        if down {
                            view.site_settings_pointer_down(sample.x, sample.y, cx);
                            // Blueprint D23: a press on a pane focuses it.
                            view.split_pointer_down(sample.x, sample.y, cx);
                        } else {
                            view.on_rail_pointer(sample, window, cx);
                        }
                    }
                });
            });
        }
    })
    .detach();
}
