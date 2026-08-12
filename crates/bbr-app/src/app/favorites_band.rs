//! The §11.10 favorites-band anchor (DESIGN.md §11.10, daily-driver
//! Sidebar slice): an invisible element inside the favorites region that
//! records the region's window-relative bounds every frame, so the tab-drag
//! resolver targets the real rendered drop band and the E2E snapshot
//! exposes its typed geometry — never a parallel layout model. The same
//! pattern as the §11.4 command-field anchor (`app::palette::anchor`);
//! `bbr-app` owns the recorder and passes it into the rail composition at
//! the app-owned-child boundary.

use std::rc::Rc;

use bbr_web::WebRect;
use gpui::{
    App, Bounds, Edges, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Length, Pixels, Position, Style, Window,
};

/// An invisible element inside the favorites region that records the
/// region's window-relative bounds every frame (DESIGN.md §11.10): the
/// exact drop-band span the tab-drag resolver and the E2E snapshot use, so
/// a pointer inside the rendered band toggles the dragged tab as a
/// favorite. Occupies no layout space (absolute inset-0 within the
/// `relative` region) and paints nothing.
pub struct FavoritesBandAnchor {
    publish: Rc<dyn Fn(WebRect)>,
}

/// Build the band anchor wired to the app's measured-rect cell.
pub(crate) fn favorites_band_anchor(
    rect: Rc<std::cell::RefCell<Option<WebRect>>>,
) -> FavoritesBandAnchor {
    FavoritesBandAnchor {
        publish: Rc::new(move |measured| {
            *rect.borrow_mut() = Some(measured);
        }),
    }
}

impl IntoElement for FavoritesBandAnchor {
    type Element = FavoritesBandAnchor;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for FavoritesBandAnchor {
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
