//! Narrow app construction seam for the one native portal host (task packet
//! `native-portal-host`).
//!
//! Exactly one transparent, non-activating portal window per BBR browser
//! window: created hidden at window construction by
//! [`NativePortalBridge::open`], attached above the parent `NSWindow` by the
//! native bridge in `bbr-web-macos`, and retained here for app lifecycle.
//! GPUI owns portal content and state; AppKit owns only native z-order,
//! parent attachment, frame synchronization, visibility, and hit-test
//! routing. The portal window's GPUI root renders probe and product chrome
//! ([`super::probe::PortalRoot`]); the app drives it through shared portal
//! presentation state.
//!
//! On non-macOS targets the seam is an empty placeholder so the workspace
//! still evaluates; the cross-platform host is deferred to the adapter phase.

/// App-side owner of the one native portal window per browser window.
#[cfg(target_os = "macos")]
pub use macos::NativePortalBridge;

/// Portable placeholder: no native portal host exists on this platform yet
/// (the task packet defers cross-platform hosts to the adapter phase).
#[cfg(not(target_os = "macos"))]
pub struct NativePortalBridge;

#[cfg(target_os = "macos")]
mod macos {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        prelude::*, px, size, App, Bounds, Entity, Point, WindowBackgroundAppearance, WindowBounds,
        WindowHandle, WindowKind, WindowOptions,
    };
    use objc2::MainThreadMarker;

    use crate::app::BbrApp;
    use crate::portal::probe::{PortalProbeState, PortalRoot};

    /// The app-side owner of the one native portal window per browser window.
    ///
    /// Retains both the GPUI portal window (app lifecycle) and the native
    /// bridge (AppKit configuration, child attachment, frame sync,
    /// visibility, teardown). All commands are main-thread-only and
    /// idempotent.
    pub struct NativePortalBridge {
        portal_window: WindowHandle<PortalRoot>,
        native: Box<bbr_web_macos::portal::NativePortal>,
    }

    impl NativePortalBridge {
        /// Open the one portal window for the browser window and attach it
        /// above the parent. The window starts hidden, non-focusing, and
        /// non-activating; it is created once at window construction so the
        /// later open path is only state update + order-front. `app` is the
        /// browser app entity product chrome observes and routes actions
        /// through; `probe` is the shared portal render state.
        ///
        /// # Safety
        ///
        /// `parent_ns_view` must be the live GPUI root `NSView*` of the
        /// browser window, captured on the main thread; the native bridge
        /// retains it.
        pub unsafe fn open(
            cx: &mut App,
            parent_ns_view: *mut std::ffi::c_void,
            browser_window: WindowHandle<gpui_component::Root>,
            app: Entity<BbrApp>,
            probe: Rc<RefCell<PortalProbeState>>,
        ) -> anyhow::Result<Self> {
            let mtm = MainThreadMarker::new().expect("portal window opened on the main thread");
            let options = WindowOptions {
                // The native bridge pins the frame to the parent content
                // rect on attach; the initial 1×1 bounds are never visible
                // (the window starts hidden).
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    Point::new(px(0.0), px(0.0)),
                    size(px(1.0), px(1.0)),
                ))),
                titlebar: None,
                focus: false,
                show: false,
                kind: WindowKind::PopUp,
                is_movable: false,
                is_resizable: false,
                is_minimizable: false,
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            };
            let portal_window = cx
                .open_window(options, |_window, cx| {
                    cx.new(|cx| PortalRoot::new(browser_window, app.clone(), probe.clone(), cx))
                })
                .map_err(|error| anyhow::anyhow!("open portal window: {error}"))?;
            let portal_ns_view = portal_window
                .update(cx, |_root, window, _cx| crate::extract_ns_view(window))
                .map_err(|error| anyhow::anyhow!("capture portal root view: {error}"))?;
            let native = unsafe {
                bbr_web_macos::portal::NativePortal::attach(mtm, parent_ns_view, portal_ns_view)
            }
            .map_err(|error| anyhow::anyhow!("attach native portal: {error:?}"))?;
            Ok(Self {
                portal_window,
                native,
            })
        }

        /// The GPUI portal window handle (Copy), for the app to re-render or
        /// flush the probe surface.
        pub fn portal_handle(&self) -> WindowHandle<PortalRoot> {
            self.portal_window
        }

        /// Re-pin the portal window to the parent content rect (idempotent).
        pub fn sync_frame(&mut self) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "portal sync off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.sync_frame(mtm);
            }
        }

        /// Show or hide the portal window. Showing a non-activating panel
        /// never makes it key or main; hiding orders it out when no native
        /// presentation is active.
        pub fn set_visible(&mut self, visible: bool) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "portal visibility change off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.set_visible(mtm, visible);
            }
        }

        /// Blueprint D08: declare the capture regions (`None` captures every
        /// pointer event; `Some` lets it through outside the rects).
        pub fn set_hit_regions(&mut self, regions: Option<Vec<[f32; 4]>>) {
            if let Some(mtm) = MainThreadMarker::new() {
                let regions = regions.map(|rects| {
                    rects
                        .into_iter()
                        .map(|[x, y, w, h]| {
                            bbr_web_macos::portal_hit::PortalHitRect::new(
                                f64::from(x),
                                f64::from(y),
                                f64::from(w),
                                f64::from(h),
                            )
                        })
                        .collect()
                });
                self.native.set_hit_regions(mtm, regions);
            }
        }

        /// Blueprint D08 testkit seam: route a pointer at `(x, y)` (window
        /// points) through the monitor's decision; returns whether the panel
        /// captures it.
        #[cfg(feature = "testkit")]
        pub fn route_pointer_at(&mut self, x: f32, y: f32) -> bool {
            MainThreadMarker::new().is_some_and(|mtm| {
                self.native
                    .route_pointer_at(mtm, (f64::from(x), f64::from(y)))
            })
        }

        /// Narrow main-thread diagnostics snapshot (DESIGN §15 semantics;
        /// the testkit snapshot mirrors these fields). `None` when called
        /// off the main thread.
        pub fn diagnostics(&self) -> Option<bbr_web_macos::portal::PortalNativeDiagnostics> {
            let mtm = MainThreadMarker::new()?;
            Some(self.native.diagnostics(mtm))
        }

        /// Deterministically detach and close the portal window (main
        /// thread). After teardown the GPUI handle is dropped by the caller;
        /// closing natively removes the window from GPUI's arena through the
        /// platform close path. Exercised by the testkit `PortalTeardown`
        /// request (`BbrApp::portal_teardown`) and by the host's own
        /// will-close observer when the browser window closes; see the task
        /// packet's teardown evidence.
        pub fn teardown(&mut self) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "portal teardown off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.teardown(mtm);
            }
        }

        /// Drive the parent's real AppKit fullscreen transition
        /// (`toggleFullScreen:`) — the native-portal-hardening driver the
        /// testkit request uses. Idempotent.
        pub fn set_fullscreen(&mut self, fullscreen: bool) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "fullscreen drive off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.set_fullscreen(mtm, fullscreen);
            }
        }

        /// Drive the parent's real AppKit minimize/restore
        /// (`miniaturize:` / `deminiaturize:`) — the hardening driver the
        /// testkit request uses. Idempotent.
        pub fn set_minimized(&mut self, minimized: bool) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "minimize drive off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.set_minimized(mtm, minimized);
            }
        }

        /// Drive a real Window Server space-assignment transition on the
        /// parent (`CanJoinAllSpaces` collection behavior) — the hardening
        /// driver the testkit request uses. Idempotent.
        pub fn set_all_spaces(&mut self, all_spaces: bool) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "all-spaces drive off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.set_all_spaces(mtm, all_spaces);
            }
        }

        /// Close the parent browser window through the real AppKit close
        /// path (`performClose:`) — the hardening driver the testkit
        /// request uses; the host's will-close observer tears the portal
        /// down deterministically.
        pub fn close_parent(&mut self) {
            debug_assert!(
                MainThreadMarker::new().is_some(),
                "parent close off the main thread"
            );
            if let Some(mtm) = MainThreadMarker::new() {
                self.native.close_parent(mtm);
            }
        }
    }
}
