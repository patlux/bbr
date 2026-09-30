//! Blueprint D19 (several windows, task packet `multi-window.md`): the root
//! views of the browser windows after the first.
//!
//! The engine attaches to the first window's GPUI root view at startup;
//! every further window attaches its own root here, with its own
//! context-menu and content-gesture bridges (sharing the engine's sinks).
//! A session lives in the root of the window that shows it: moving it
//! re-parents the `WKWebView` and its navigation cover (the page, its
//! history and its process stay), re-registers it with that window's
//! bridges and forgets its cached frame, so the next slot geometry flips
//! against the new root's height.

use std::collections::{BTreeMap, HashMap};

use objc2::MainThreadMarker;

use bbr_web::WebViewId;

use crate::content_gesture::{ContentGestureHost, ContentGestureSink};
use crate::delegate::EventSink;
use crate::host::HostBridge;
use crate::menu_adapter::NativeContextMenuBridge;

/// One further window's root and bridges.
pub(crate) struct WindowRoot {
    pub(crate) host: HostBridge,
    pub(crate) menu_bridge: Option<Box<NativeContextMenuBridge>>,
    pub(crate) content_gesture: ContentGestureHost,
}

/// The further windows' roots and where each session lives.
#[derive(Default)]
pub(crate) struct WindowRoots {
    pub(crate) roots: BTreeMap<u64, WindowRoot>,
    /// A session's window (absent: the first window).
    pub(crate) session_window: HashMap<WebViewId, u64>,
    /// The window whose chrome takes the keyboard back
    /// (`focus_host_chrome`).
    pub(crate) key_window: u64,
    /// Sessions placed in a window whose root is not attached yet.
    pub(crate) pending: HashMap<WebViewId, u64>,
}

impl WindowRoots {
    /// Attach window `window`'s GPUI root view.
    ///
    /// # Safety
    ///
    /// `ns_view` must be the live GPUI root `NSView*` of that window,
    /// captured on the main thread; it is retained.
    pub(crate) unsafe fn attach(
        &mut self,
        mtm: MainThreadMarker,
        window: u64,
        ns_view: *mut std::ffi::c_void,
        sink: EventSink,
        gestures: ContentGestureSink,
    ) {
        let mut host = HostBridge::new();
        unsafe { host.attach_root_view(mtm, ns_view) };
        let menu_bridge = unsafe { NativeContextMenuBridge::attach(mtm, ns_view, sink) };
        let content_gesture = ContentGestureHost::attach(mtm, ns_view, gestures);
        self.roots.insert(
            window,
            WindowRoot {
                host,
                menu_bridge,
                content_gesture,
            },
        );
    }

    /// Forget a closed window (its sessions were moved or hidden first).
    pub(crate) fn detach(&mut self, window: u64) {
        self.roots.remove(&window);
        self.pending.retain(|_, target| *target != window);
        if self.key_window == window {
            self.key_window = 0;
        }
    }

    pub(crate) fn window_of(&self, view: WebViewId) -> u64 {
        self.session_window.get(&view).copied().unwrap_or(0)
    }

    pub(crate) fn root(&self, window: u64) -> Option<&WindowRoot> {
        self.roots.get(&window)
    }
}
