//! The `NativeWebViewSlot` GPUI element.
//!
//! This is the load-bearing integration: a custom public GPUI [`Element`]
//! placed in the chrome layout where one web page pane goes. On every frame
//! GPUI hands it its window-relative `Bounds<Pixels>`; the element forwards
//! that rect (as a [`bbr_web::WebRect`]) to the engine, which frames the
//! native WKWebView **for that view's id** — the split shell renders one
//! slot per visible pane, so each pane is framed independently. Painting a
//! native surface means there is nothing for GPUI to draw — the WKWebView
//! shows through the Metal layer.
//!
//! No GPUI fork is required: this uses only public `Element`/`Window` APIs.

use std::rc::Rc;

use bbr_web::{WebEngine, WebRect, WebViewId};
use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Style, Window,
};

/// The stable keyed slot id (DESIGN.md §15): `bbr-native-web-slot-<n>` where
/// `<n>` is the numeric [`WebViewId`].
pub fn native_slot_id(view: WebViewId) -> ElementId {
    ElementId::Name(format!("bbr-native-web-slot-{}", view.0).into())
}

/// A layout node that occupies one web-content pane and reports its bounds
/// to the engine each frame (only when they change, so an idle window makes
/// no `setFrame:` traffic).
pub struct NativeWebViewSlot {
    /// The live web view this slot frames. Geometry is keyed by this id so
    /// two split panes address their own WKWebView and never frame a sibling.
    view_id: WebViewId,
    /// Receives `(rect, content_height, corner_radius)` on the main thread
    /// every prepaint (the engine suppresses unchanged values). Kept as a
    /// closure so the element stays framework-free plumbing: `prepaint`
    /// runs on the main thread during GPUI's draw.
    publish: Rc<dyn Fn(WebRect, f32, f32)>,
    /// The seated content card's corner radius (DESIGN §2) the native layer
    /// clips to; `0` is full-bleed.
    corner_radius: f32,
    #[cfg(feature = "testkit")]
    presentation_pending: Option<Rc<dyn Fn() -> bool>>,
    #[cfg(feature = "testkit")]
    presented: Option<Rc<dyn Fn(WebRect)>>,
    /// Testkit-only: observes every engine-received rect **keyed by this
    /// slot's view id** (not only during a pending measurement) so the
    /// semantic snapshot can assert the actual native pane geometry after
    /// layout changes. Wired on every slot; the focused-slot `on_geometry`
    /// observer additionally feeds the legacy `content_rect`.
    #[cfg(feature = "testkit")]
    keyed_geometry: Option<Rc<dyn Fn(WebViewId, WebRect)>>,
    /// Testkit-only: observes the rect of this slot for the legacy
    /// `content_rect` (focused/active pane). Wired only on the focused slot
    /// so a secondary pane can never satisfy an existing benchmark.
    #[cfg(feature = "testkit")]
    geometry: Option<Rc<dyn Fn(WebRect)>>,
}

impl NativeWebViewSlot {
    /// Create a slot that publishes `rect`/`content_height` for `view_id`
    /// into `engine`.
    pub fn new<E: WebEngine + 'static + ?Sized>(
        engine: Rc<std::cell::RefCell<E>>,
        view_id: WebViewId,
    ) -> Self {
        let publish: Rc<dyn Fn(WebRect, f32, f32)> =
            Rc::new(move |rect, content_height, corner_radius| {
                let mut engine = engine.borrow_mut();
                engine.set_web_view_rect(view_id, rect, content_height);
                engine.set_web_view_corner_radius(view_id, corner_radius);
            });
        Self {
            view_id,
            publish,
            corner_radius: 0.0,
            #[cfg(feature = "testkit")]
            presentation_pending: None,
            #[cfg(feature = "testkit")]
            presented: None,
            #[cfg(feature = "testkit")]
            keyed_geometry: None,
            #[cfg(feature = "testkit")]
            geometry: None,
        }
    }

    /// Clip the native surface to the content card's corner radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    #[cfg(feature = "testkit")]
    pub fn on_presented(
        mut self,
        pending: Rc<dyn Fn() -> bool>,
        callback: Rc<dyn Fn(WebRect)>,
    ) -> Self {
        self.presentation_pending = Some(pending);
        self.presented = Some(callback);
        self
    }

    /// Testkit-only: observe every rect this slot hands to the engine
    /// (the WKWebView frame), keyed by the slot's view id, so the semantic
    /// snapshot's `pane_rects` is the real native geometry per pane, never a
    /// computed estimate.
    #[cfg(feature = "testkit")]
    pub fn on_keyed_geometry(mut self, callback: Rc<dyn Fn(WebViewId, WebRect)>) -> Self {
        self.keyed_geometry = Some(callback);
        self
    }

    /// Testkit-only: observe this slot's rect for the legacy focused/active
    /// `content_rect` (the WKWebView frame of the focused pane). Wire this
    /// only on the focused slot so a secondary pane never overwrites the
    /// content contract.
    #[cfg(feature = "testkit")]
    pub fn on_geometry(mut self, callback: Rc<dyn Fn(WebRect)>) -> Self {
        self.geometry = Some(callback);
        self
    }
}

impl IntoElement for NativeWebViewSlot {
    type Element = NativeWebViewSlot;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for NativeWebViewSlot {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        // Publish a stable keyed id so the inspector and tests can target the
        // slot and two split panes stay distinct elements. The `bbr-` prefix
        // follows DESIGN.md §15 (the semantic id shared by the testkit
        // snapshot and the future platform Accessibility bridge).
        Some(native_slot_id(self.view_id))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Fill the remaining space right of the sidebar.
        let mut style = Style {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        };
        style.size.width = gpui::relative(1.0).into();
        style.size.height = gpui::relative(1.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let rect = WebRect::new(
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        );
        let content_height = f32::from(window.bounds().size.height);
        (self.publish)(rect, content_height, self.corner_radius);
        #[cfg(feature = "testkit")]
        if let Some(keyed_geometry) = self.keyed_geometry.clone() {
            keyed_geometry(self.view_id, rect);
        }
        #[cfg(feature = "testkit")]
        if let Some(geometry) = self.geometry.clone() {
            geometry(rect);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        #[cfg_attr(not(feature = "testkit"), allow(unused_variables))] bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
        // The native WKWebView is composited above GPUI's Metal layer here;
        // nothing to paint.
        #[cfg(feature = "testkit")]
        if self
            .presentation_pending
            .as_ref()
            .is_some_and(|pending| pending())
        {
            if let Some(presented) = self.presented.clone() {
                let rect = WebRect::new(
                    f32::from(bounds.origin.x),
                    f32::from(bounds.origin.y),
                    f32::from(bounds.size.width),
                    f32::from(bounds.size.height),
                );
                presented(rect);
            }
        }
    }
}
