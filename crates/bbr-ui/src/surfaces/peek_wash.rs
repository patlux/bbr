//! Blueprint D09 (DESIGN §11.17): the Peek wash — the origin dims under
//! `wash.active` while the preview sits in its inset frame.
//!
//! The wash is painted by the one native portal (the only chrome allowed
//! over the glass, §13) as a single even-odd path: the card's rounded inner
//! box minus the preview's rounded frame. The preview WKWebView is never
//! covered, sampled or tinted; a hairline ring sits just outside the
//! preview's clip. Presentation only: the geometry comes from
//! [`crate::card::PeekFrame`].

use gpui::{
    canvas, div, point, prelude::*, px, AnyElement, Bounds, PathBuilder, Pixels, Point, Rgba,
};

use crate::card::{PeekFrame, PEEK_PREVIEW_RADIUS};
use crate::theme::Theme;

/// The §15 id of the wash over the covered origin.
pub const PEEK_WASH_ID: &str = "bbr-peek-wash";
/// The §15 id of the hairline ring around the preview frame.
pub const PEEK_FRAME_ID: &str = "bbr-peek-frame";

/// The cubic-Bézier handle factor for a quarter circle.
const KAPPA: f32 = 0.552_284_8;

/// Append one closed rounded rectangle `[x, y, w, h]` to `path`.
fn rounded_rect(path: &mut PathBuilder, rect: [f32; 4], radius: f32) {
    let [x, y, w, h] = rect;
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let k = r * KAPPA;
    let p = |x: f32, y: f32| -> Point<Pixels> { point(px(x), px(y)) };
    path.move_to(p(x + r, y));
    path.line_to(p(x + w - r, y));
    path.cubic_bezier_to(p(x + w, y + r), p(x + w - r + k, y), p(x + w, y + r - k));
    path.line_to(p(x + w, y + h - r));
    path.cubic_bezier_to(
        p(x + w - r, y + h),
        p(x + w, y + h - r + k),
        p(x + w - r + k, y + h),
    );
    path.line_to(p(x + r, y + h));
    path.cubic_bezier_to(p(x, y + h - r), p(x + r - k, y + h), p(x, y + h - r + k));
    path.line_to(p(x, y + r));
    path.cubic_bezier_to(p(x + r, y), p(x, y + r - k), p(x + r - k, y));
    path.close();
}

/// Offset a `[x, y, w, h]` rect by the painting element's origin.
fn offset(rect: [f32; 4], origin: Point<Pixels>) -> [f32; 4] {
    [
        rect[0] + f32::from(origin.x),
        rect[1] + f32::from(origin.y),
        rect[2],
        rect[3],
    ]
}

/// The wash over the covered origin plus the preview's hairline ring, as
/// absolute children of the surface whose origin the frame is relative to
/// (the full-window portal surface in the app, a story box in the Gallery).
pub fn peek_wash(theme: &Theme, frame: PeekFrame) -> [AnyElement; 2] {
    let p = theme.palette;
    let color: Rgba = p.wash_active;
    let wash = div()
        .id(PEEK_WASH_ID)
        .absolute()
        .inset_0()
        .child(
            canvas(
                |_bounds, _window, _cx| (),
                move |bounds: Bounds<Pixels>, (), window, _cx| {
                    // Even-odd fill (the lyon default): outer card minus
                    // the preview frame leaves the preview untouched.
                    let mut path = PathBuilder::fill();
                    let origin = bounds.origin;
                    rounded_rect(&mut path, offset(frame.card, origin), frame.card_radius);
                    rounded_rect(
                        &mut path,
                        offset(frame.preview, origin),
                        PEEK_PREVIEW_RADIUS,
                    );
                    if let Ok(path) = path.build() {
                        window.paint_path(path, color);
                    }
                },
            )
            .size_full(),
        )
        .into_any_element();
    // The ring sits one point outside the preview's clip, over the washed
    // origin only (a border paints inside its own box).
    let [x, y, w, h] = frame.preview;
    let ring = div()
        .id(PEEK_FRAME_ID)
        .absolute()
        .left(px(x - 1.0))
        .top(px(y - 1.0))
        .w(px(w + 2.0))
        .h(px(h + 2.0))
        .rounded(px(PEEK_PREVIEW_RADIUS + 1.0))
        .border_1()
        .border_color(p.hairline)
        .into_any_element();
    [wash, ring]
}
