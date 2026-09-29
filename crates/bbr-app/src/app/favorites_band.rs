//! Rendered-rect anchors (DESIGN.md §11.10, §11.2, §10.9): an invisible
//! element that records its parent's window-relative bounds every frame, so
//! the tab-drag resolver hits the real rendered Favorites band and §11.2
//! ghost row, and the E2E snapshot exposes their typed geometry (and the
//! §10.9 drag proxy's) — never a parallel layout model. The same pattern as
//! the §11.4 command-field anchor (`app::palette::anchor`); `bbr-app` owns
//! the recorders and passes them into the rail composition at the
//! app-owned-child boundary.

use std::rc::Rc;

use bbr_web::WebRect;
use gpui::{
    App, Bounds, Edges, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Length, Pixels, Position, Style, Window,
};

/// An invisible element that records its `relative` parent's
/// window-relative bounds every frame into an app-owned cell. Occupies no
/// layout space (absolute inset-0) and paints nothing.
pub struct RectAnchor {
    publish: Rc<dyn Fn(WebRect)>,
}

impl RectAnchor {
    /// A rect anchor that hands every measured rect to `publish`.
    pub(crate) fn new(publish: impl Fn(WebRect) + 'static) -> Self {
        Self {
            publish: Rc::new(publish),
        }
    }
}

/// Build a rect anchor wired to an app-owned measured-rect cell.
pub(crate) fn rect_anchor(rect: Rc<std::cell::RefCell<Option<WebRect>>>) -> RectAnchor {
    RectAnchor {
        publish: Rc::new(move |measured| {
            *rect.borrow_mut() = Some(measured);
        }),
    }
}

/// The §11.10 Favorites-band anchor: the exact drop-band span the tab-drag
/// resolver and the E2E snapshot use.
pub(crate) fn favorites_band_anchor(rect: Rc<std::cell::RefCell<Option<WebRect>>>) -> RectAnchor {
    rect_anchor(rect)
}

impl IntoElement for RectAnchor {
    type Element = RectAnchor;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for RectAnchor {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
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
        let style = Style {
            position: Position::Absolute,
            inset: Edges::<Length>::zero(),
            ..Default::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        (self.publish)(WebRect::new(
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        ));
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }
}
