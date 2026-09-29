//! The §11.4 command-field anchor (DESIGN.md §11.4): an invisible element
//! inside the command field that records the field's window-relative bounds
//! every frame, so the field-anchored overlay can be placed flush with the
//! field without duplicating the chrome layout math.

use std::rc::Rc;

use bbr_web::WebRect;
use gpui::{
    App, Bounds, Edges, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Length, Pixels, Position, Style, Window,
};

/// An invisible element inside the command field that records the field's
/// window-relative bounds every frame (DESIGN.md §11.4): the exact anchor
/// for the field-anchored overlay (top/leading edge flush, width from the
/// field), so the overlay placement never duplicates the chrome layout math.
/// Occupies no layout space (absolute inset-0 within the `relative` command
/// bar) and paints nothing.
/// Hands the measured field rect to the app (with the app context, so a
/// moved field can re-anchor an open overlay).
type Publish = Rc<dyn Fn(WebRect, &mut App)>;

pub struct CommandFieldAnchor {
    publish: Publish,
}

/// Build the field anchor wired to the app's measured-rect cell.
pub(crate) fn command_field_anchor(
    rect: std::rc::Rc<std::cell::RefCell<Option<WebRect>>>,
) -> CommandFieldAnchor {
    CommandFieldAnchor {
        publish: Rc::new(move |measured, _cx| {
            *rect.borrow_mut() = Some(measured);
        }),
    }
}

/// Blueprint D02: the anchor inside the floating rail — a field that moved
/// (the rail just floated in for ⌘L) re-anchors an open overlay once.
pub(crate) fn command_field_anchor_following(
    rect: std::rc::Rc<std::cell::RefCell<Option<WebRect>>>,
    app: gpui::Entity<crate::app::BbrApp>,
    browser: gpui::WindowHandle<gpui_component::Root>,
) -> CommandFieldAnchor {
    CommandFieldAnchor {
        publish: Rc::new(move |measured, cx| {
            let moved = *rect.borrow() != Some(measured);
            *rect.borrow_mut() = Some(measured);
            if moved {
                let app = app.clone();
                cx.defer(move |cx| {
                    let _ = crate::window_seam::with_app(&browser, &app, cx, |view, window, cx| {
                        view.reanchor_command_overlay(window, cx)
                    });
                });
            }
        }),
    }
}

impl IntoElement for CommandFieldAnchor {
    type Element = CommandFieldAnchor;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for CommandFieldAnchor {
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
        cx: &mut App,
    ) -> Self::PrepaintState {
        (self.publish)(
            WebRect::new(
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
            ),
            cx,
        );
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
