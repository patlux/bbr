//! The `NativeWebViewSlot` GPUI element.
//!
//! This is the load-bearing integration: a custom public GPUI [`Element`]
//! placed in the chrome layout where the web page goes. On every frame GPUI
//! hands it its window-relative `Bounds<Pixels>`; the element forwards that
//! rect (as a [`bbr_web::WebRect`]) to the engine, which frames the native
//! WKWebView. Painting a native surface means there is nothing for GPUI to
//! draw — the WKWebView shows through the Metal layer.
//!
//! No GPUI fork is required: this uses only public `Element`/`Window` APIs.

use std::rc::Rc;

use bbr_web::{WebEngine, WebRect};
use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Style, Window,
};

/// A layout node that occupies the web content area and reports its bounds
/// to the engine each frame (only when they change, so an idle window makes
/// no `setFrame:` traffic).
pub struct NativeWebViewSlot {
    /// Receives `(rect, content_height)` on the main thread when the slot's
    /// bounds change. Kept as a closure so the element stays framework-free
    /// plumbing: `prepaint` runs on the main thread during GPUI's draw.
    publish: Rc<dyn Fn(WebRect, f32)>,
    #[cfg(feature = "testkit")]
    presentation_pending: Option<Rc<dyn Fn() -> bool>>,
    #[cfg(feature = "testkit")]
    presented: Option<Rc<dyn Fn(WebRect)>>,
    /// Testkit-only: observes every engine-received rect (not only during a
    /// pending measurement) so the semantic snapshot can assert the actual
    /// native content geometry after layout changes.
    #[cfg(feature = "testkit")]
    geometry: Option<Rc<dyn Fn(WebRect)>>,
}

impl NativeWebViewSlot {
    /// Create a slot that pushes `rect`/`content_height` into `engine`.
    pub fn new<E: WebEngine + 'static + ?Sized>(engine: Rc<std::cell::RefCell<E>>) -> Self {
        let publish: Rc<dyn Fn(WebRect, f32)> = Rc::new(move |rect, content_height| {
            engine.borrow_mut().attach_slot(rect, content_height);
        });
        Self {
            publish,
            #[cfg(feature = "testkit")]
            presentation_pending: None,
            #[cfg(feature = "testkit")]
            presented: None,
            #[cfg(feature = "testkit")]
            geometry: None,
        }
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

    /// Testkit-only: observe every rect this slot hands to the engine (the
    /// WKWebView frame), so the snapshot's `content_rect` is the real native
    /// geometry, never a computed estimate.
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
        // Publish a stable id so the inspector and tests can target the slot.
        Some(ElementId::Name("native-web-slot".into()))
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
        (self.publish)(rect, content_height);
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
