//! The app's portal-host control surface (task packet `native-portal-host`):
//! the one native portal window seam, the general Portal API present/dismiss
//! methods, and the deterministic teardown. The portal *domain* state and the
//! native bridge live in `crate::portal`; this module owns the `BbrApp`
//! methods that drive them, so the browser root stays within its size budget.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Context, Window};

use crate::portal::geometry::Sz;
use crate::portal::probe::PortalProbeState;
use crate::portal::state::{PortalHost, PortalRequest, PortalTransition};

use super::BbrApp;

impl BbrApp {
    /// Attach the one native portal window (macOS construction seam).
    /// Exactly one per browser window; hidden and non-activating until a
    /// native presentation drives it (task packet `native-portal-host`).
    #[cfg(target_os = "macos")]
    pub fn set_native_portal(&mut self, bridge: crate::portal::native::NativePortalBridge) {
        self.native_portal = Some(bridge);
    }

    /// Narrow main-thread diagnostics of the native portal host, when
    /// attached (DESIGN §15 snapshot semantics; the testkit snapshot
    /// mirrors these fields). `None` when no portal is attached.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit snapshot only; production consumers arrive with U1
    pub fn native_portal_diagnostics(
        &self,
    ) -> Option<bbr_web_macos::portal::PortalNativeDiagnostics> {
        self.native_portal
            .as_ref()
            .and_then(crate::portal::native::NativePortalBridge::diagnostics)
    }

    /// Blueprint D08 testkit seam: route a pointer at `(x, y)` through the
    /// Portal's hit-region decision (no cursor movement).
    #[cfg(feature = "testkit")]
    pub fn portal_route_pointer(&mut self, x: f32, y: f32) {
        #[cfg(target_os = "macos")]
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.route_pointer_at(x, y);
        }
        let _ = (x, y);
    }

    /// The shared portal render state (the composition root passes a clone
    /// into the native bridge so the portal window renders exactly the
    /// active probe or product presentation commanded by the app).
    pub fn portal_probe_handle(&self) -> Rc<RefCell<PortalProbeState>> {
        self.portal_probe.clone()
    }

    /// Present (or refresh) a portal request through the general Portal API
    /// (task packet `native-portal-host`): updates the one-active portal
    /// state and, for a [`PortalHost::NativeAboveWeb`] presentation,
    /// resolves the placement against the window, records the active
    /// presentation, and orders the native portal window front (a non-activating
    /// panel never becomes key or main). Exactly one presentation stays
    /// active; re-presenting the same id refreshes it in place. A
    /// [`PortalHost::GpuiDeferred`] request is state-only until a production
    /// deferred caller exists.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // probe + U1 production presentation seam
    pub fn portal_present(
        &mut self,
        request: PortalRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> PortalTransition {
        if request.host == PortalHost::NativeAboveWeb {
            let viewport = Sz::new(
                f32::from(window.bounds().size.width),
                f32::from(window.bounds().size.height),
            );
            self.portal_probe.borrow_mut().present(request, viewport);
        }
        self.activate_portal_request(request, cx)
    }

    /// Activate an already-prepared presentation; render state is populated
    /// first so ordering the child in cannot expose stale content.
    pub(crate) fn activate_portal_request(
        &mut self,
        request: PortalRequest,
        cx: &mut Context<Self>,
    ) -> PortalTransition {
        let transition = self.portal.present(request);
        #[cfg(target_os = "macos")]
        if request.host == PortalHost::NativeAboveWeb {
            self.drive_native_present(cx);
        }
        transition
    }

    /// Deterministically dismiss the active presentation (the production
    /// dismissal method): the active portal content clears and the native portal
    /// window orders out. Returns whether a presentation was actually
    /// active.
    pub fn portal_dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.portal.dismiss() {
            return false;
        }
        self.portal_probe.borrow_mut().clear();
        #[cfg(target_os = "macos")]
        {
            // Blueprint D08: the Portal stays up (in its hit-region mode)
            // while a toast or the zoom readout still shows.
            let overlays = self.transient.showing();
            let regions = self.portal_hit_regions();
            if let Some(bridge) = self.native_portal.as_mut() {
                bridge.set_visible(overlays);
                bridge.set_hit_regions(regions);
            }
            // Re-render the portal root so the dismissed surface is cleared.
            // Deferred: dismissal can run inside the portal window's own
            // event dispatch (real backdrop press), where updating it again
            // would double-lease the root entity.
            if let Some(handle) = self
                .native_portal
                .as_ref()
                .map(|bridge| bridge.portal_handle())
            {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_root, _window, cx| cx.notify());
                });
            }
        }
        true
    }

    /// The exact production outside-dismiss method (DESIGN §13 one-outside-
    /// pointer-action): the portal backdrop's pointer-press listener and the
    /// testkit `PortalOutsideDismiss` request both call this. Only dismisses
    /// when the active presentation's [`DismissalPolicy`] allows
    /// outside-pointer dismissal.
    pub fn portal_outside_dismiss(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.portal.dismisses_on_outside_press() {
            return false;
        }
        self.portal_dismiss(cx)
    }

    /// Order the native portal window front and re-render its active content.
    /// Deferred to avoid re-entering another window during an app update.
    #[cfg(target_os = "macos")]
    fn drive_native_present(&mut self, cx: &mut Context<Self>) {
        // A presentation captures every pointer event (menus and sheets
        // dismiss on an outside press); the Peek captures only its washed
        // origin and bar (see `portal_hit_regions`).
        let regions = self.portal_hit_regions();
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.set_visible(true);
            bridge.set_hit_regions(regions);
        }
        if let Some(handle) = self
            .native_portal
            .as_ref()
            .map(|bridge| bridge.portal_handle())
        {
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_root, _window, cx| cx.notify());
            });
        }
    }

    /// Deterministically tear down the one native portal host (task packet
    /// teardown evidence): removes the child relation, orders the portal
    /// window out, closes it, and drops the bridge — the app owns no portal
    /// afterwards. Idempotent and safe when no host was ever attached; the
    /// probe surface and the one-active presentation state clear with it.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit teardown seam; production callers arrive with U1
    pub fn portal_teardown(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(mut bridge) = self.native_portal.take() {
            bridge.teardown();
        }
        self.portal_probe.borrow_mut().clear();
        self.portal.dismiss();
    }

    /// Native-portal-hardening driver: drive the parent's real AppKit
    /// fullscreen transition (`toggleFullScreen:`). Testkit-only seam into
    /// the one native host (task packet `native-portal-hardening`); the
    /// E2E polls `parent_fullscreen` + frame sync until the Window Server
    /// settles the transition.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // E2E lifecycle driver
    pub fn portal_set_fullscreen(&mut self, fullscreen: bool) {
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.set_fullscreen(fullscreen);
        }
    }

    /// Native-portal-hardening driver: drive the parent's real AppKit
    /// minimize/restore (`miniaturize:` / `deminiaturize:`). Testkit-only
    /// seam into the one native host.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // E2E lifecycle driver
    pub fn portal_set_minimized(&mut self, minimized: bool) {
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.set_minimized(minimized);
        }
    }

    /// Native-portal-hardening driver: drive a real Window Server
    /// space-assignment transition on the parent (`CanJoinAllSpaces`
    /// collection behavior). Testkit-only seam into the one native host.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // E2E lifecycle driver
    pub fn portal_set_all_spaces(&mut self, all_spaces: bool) {
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.set_all_spaces(all_spaces);
        }
    }

    /// Native-portal-hardening driver: close the parent browser window
    /// through the real AppKit close path (`performClose:`); the host's
    /// will-close observer tears the portal down deterministically.
    /// Testkit-only seam into the one native host.
    #[cfg(target_os = "macos")]
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // E2E lifecycle driver
    pub fn portal_close_parent(&mut self) {
        if let Some(bridge) = self.native_portal.as_mut() {
            bridge.close_parent();
        }
    }
}
