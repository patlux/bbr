//! Typed rejection and divider-value tests for portable split geometry.

use crate::geometry::{
    project, DividerPosition, GeometryError, NonFinitePosition, Projection, MIN_PANE_WIDTH,
};
use bbr_web::WebRect;

const POSITIONS: &[f32] = &[0.0, 0.25, 0.5, 0.75, 1.0];

fn conserves_width(projection: &Projection, width: f32) -> bool {
    let sum = projection.primary.width + projection.divider.width + projection.secondary.width;
    (sum - width).abs() <= 8.0 * f32::EPSILON * width.max(1.0)
}

fn lands_on_content_endpoint(projection: &Projection, content: WebRect) -> bool {
    let end_x = content.x + content.width;
    let budget = 4.0 * f32::EPSILON * end_x.abs().max(1.0);
    (projection.secondary.x + projection.secondary.width - end_x).abs() <= budget
}

/// Zero and negative content dimensions are rejected with typed errors; they
/// are never silently rewritten to zero-width panes.
#[test]
fn non_positive_dimensions_rejected() {
    let position = DividerPosition::new(0.5).expect("finite");
    assert_eq!(
        project(WebRect::new(0.0, 0.0, 0.0, 60.0), position, 8.0),
        Err(GeometryError::NonPositiveWidth)
    );
    assert_eq!(
        project(WebRect::new(0.0, 0.0, -5.0, 60.0), position, 8.0),
        Err(GeometryError::NonPositiveWidth)
    );
    assert_eq!(
        project(WebRect::new(0.0, 0.0, 100.0, 0.0), position, 8.0),
        Err(GeometryError::NonPositiveHeight)
    );
    assert_eq!(
        project(WebRect::new(0.0, 0.0, 100.0, -5.0), position, 8.0),
        Err(GeometryError::NonPositiveHeight)
    );
}

/// Negative divider widths are rejected; they are never clamped to zero.
#[test]
fn negative_divider_width_rejected() {
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let position = DividerPosition::new(0.5).expect("finite");
    assert_eq!(
        project(content, position, -8.0),
        Err(GeometryError::NegativeDividerWidth)
    );
    assert_eq!(
        project(content, position, -0.001),
        Err(GeometryError::NegativeDividerWidth)
    );
}

/// Non-finite content rect fields are rejected at the projection boundary.
#[test]
fn non_finite_content_rejected() {
    let position = DividerPosition::new(0.5).expect("finite");
    for (x, y, width, height) in [
        (f32::NAN, 0.0, 100.0, 60.0),
        (0.0, f32::INFINITY, 100.0, 60.0),
        (0.0, 0.0, f32::NEG_INFINITY, 60.0),
        (0.0, 0.0, 100.0, f32::NAN),
    ] {
        assert_eq!(
            project(WebRect::new(x, y, width, height), position, 8.0),
            Err(GeometryError::NonFiniteContent),
            "non-finite content field must be rejected"
        );
    }
}

/// Non-finite divider widths are rejected at the projection boundary.
#[test]
fn non_finite_divider_width_rejected() {
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let position = DividerPosition::new(0.5).expect("finite");
    assert_eq!(
        project(content, position, f32::NAN),
        Err(GeometryError::NonFiniteDividerWidth)
    );
    assert_eq!(
        project(content, position, f32::INFINITY),
        Err(GeometryError::NonFiniteDividerWidth)
    );
    assert_eq!(
        project(content, position, f32::NEG_INFINITY),
        Err(GeometryError::NonFiniteDividerWidth)
    );
}

/// Coordinate endpoints that overflow to infinity in `f32` are rejected with
/// a typed error even though every input field is finite.
#[test]
fn overflowing_endpoints_rejected() {
    let position = DividerPosition::new(0.5).expect("finite");
    assert_eq!(
        project(WebRect::new(2.0e38, 0.0, 2.0e38, 60.0), position, 8.0),
        Err(GeometryError::OverflowingContent)
    );
    assert_eq!(
        project(WebRect::new(0.0, 3.0e38, 100.0, 3.0e38), position, 8.0),
        Err(GeometryError::OverflowingContent)
    );
}

/// Realistic large but fully representable geometry is accepted: every
/// ordered boundary advances and the far edge lands exactly on the validated
/// content endpoint. The former extreme finite acceptance used `1e30` widths
/// whose local ulp silently swallowed the 2 pt divider; this case stays
/// inside representable territory.
#[test]
fn large_representable_geometry_accepted() {
    let position = DividerPosition::new(0.5).expect("finite");
    // A 1,000,000 pt offset with a 1,000,000 pt width and a 2 pt divider:
    // all boundaries are multiples of the local ulp, so the projection is
    // exact and the divider keeps its requested extent.
    let content = WebRect::new(1.0e6, 0.0, 1.0e6, 600.0);
    let projection =
        project(content, position, 2.0).expect("large representable geometry is valid");
    assert_eq!(projection.primary.width, 499_999.0);
    assert_eq!(projection.divider.width, 2.0);
    assert_eq!(projection.secondary.width, 499_999.0);
    assert_eq!(projection.primary.x, 1.0e6);
    assert_eq!(projection.divider.x, 1_499_999.0);
    assert_eq!(projection.secondary.x, 1_500_001.0);
    // Ordered advancing boundaries.
    assert!(projection.divider.x > projection.primary.x);
    assert!(projection.secondary.x > projection.divider.x);
    // Final endpoint lands on the validated content endpoint (this case is
    // exact: every boundary is a multiple of the local ulp).
    assert!(lands_on_content_endpoint(&projection, content));
    assert_eq!(
        projection.secondary.x + projection.secondary.width,
        content.x + content.width
    );
    for rect in [projection.primary, projection.divider, projection.secondary] {
        assert!(rect.x.is_finite() && rect.width.is_finite() && rect.height.is_finite());
    }
    assert!(conserves_width(&projection, 1.0e6));
}

/// Geometry whose ordered `f32` boundaries cannot advance representably is
/// rejected with the typed error: a huge content offset swallows the pane
/// width at the primary end, and a huge content width swallows a small
/// divider width at the divider end. The projection is never returned with
/// zero-extent or overlapping rects.
#[test]
fn unrepresentable_boundaries_rejected() {
    let position = DividerPosition::new(0.5).expect("finite");
    // Huge offset: at x = 1e30 the local ulp is far above the minimum pane
    // width, so the primary end does not advance past content.x.
    assert_eq!(
        project(WebRect::new(1.0e30, 0.0, 1.0e6, 100.0), position, 2.0),
        Err(GeometryError::UnrepresentableProjection)
    );
    // Huge width: the primary end advances from x = 0, but the 2 pt divider
    // is swallowed by the local ulp at ~5e29, so the divider end does not
    // advance past the divider start.
    assert_eq!(
        project(WebRect::new(0.0, 0.0, 1.0e30, 100.0), position, 2.0),
        Err(GeometryError::UnrepresentableProjection)
    );
}

/// A tiny divider width at a large offset is swallowed by the local ulp: the
/// divider end equals the divider start, so the projection is rejected rather
/// than reporting a zero-extent divider between the panes.
#[test]
fn tiny_divider_swallowed_by_ulp_rejected() {
    let position = DividerPosition::new(0.5).expect("finite");
    // At x = 1e7 the local ulp is 1.0 pt; a 1e-3 pt divider cannot advance
    // the divider boundary.
    assert_eq!(
        project(WebRect::new(1.0e7, 0.0, 1000.0, 100.0), position, 1.0e-3),
        Err(GeometryError::UnrepresentableProjection)
    );
    // The same divider width at a representable offset is accepted: the
    // boundary advances and the final endpoint still lands exactly.
    let projection = project(WebRect::new(0.0, 0.0, 1000.0, 100.0), position, 1.0e-3)
        .expect("tiny divider at a representable offset is valid");
    assert_eq!(projection.divider.width, 1.0e-3);
    assert!(projection.divider.x + projection.divider.width > projection.divider.x);
    assert!(lands_on_content_endpoint(
        &projection,
        WebRect::new(0.0, 0.0, 1000.0, 100.0)
    ));
}

/// Realistic fractional content is accepted at every position: the ordered
/// boundaries advance, both panes stay at the minimum, and the far edge
/// lands on the validated content endpoint to `f32` rounding (a regression
/// against an exact-endpoint gate that would reject ordinary fractional
/// host-layout coordinates).
#[test]
fn fractional_layout_accepted() {
    let content = WebRect::new(13.37, 4.2, 100.7, 60.9);
    for &p in POSITIONS {
        let projection = project(content, DividerPosition::new(p).expect("finite"), 2.0)
            .expect("fractional layout must be accepted at p={p}");
        assert!(
            projection.divider.x > projection.primary.x,
            "primary end must advance at p={p}"
        );
        assert!(
            projection.secondary.x > projection.divider.x,
            "divider end must advance at p={p}"
        );
        assert!(
            projection.secondary.x + projection.secondary.width > projection.secondary.x,
            "secondary end must advance at p={p}"
        );
        assert!(lands_on_content_endpoint(&projection, content), "at p={p}");
        assert!(projection.primary.width >= MIN_PANE_WIDTH, "at p={p}");
        assert!(projection.secondary.width >= MIN_PANE_WIDTH, "at p={p}");
        assert_eq!(projection.divider.width, 2.0, "at p={p}");
    }
}

/// A zero divider width may have zero extent, but the panes stay ordered:
/// the primary and secondary intervals still advance representably and the
/// final endpoint still lands exactly on the validated content endpoint.
#[test]
fn zero_divider_width_keeps_ordered_zero_extent_divider() {
    let position = DividerPosition::new(0.5).expect("finite");
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let projection = project(content, position, 0.0).expect("zero divider is valid");
    assert_eq!(projection.divider.width, 0.0);
    assert_eq!(projection.primary.width, 50.0);
    assert_eq!(projection.secondary.width, 50.0);
    // The zero-extent divider still sits between ordered panes: primary end
    // and secondary start coincide at the divider, both strictly ordered
    // against the panes' own extents.
    assert_eq!(
        projection.divider.x,
        projection.primary.x + projection.primary.width
    );
    assert_eq!(projection.secondary.x, projection.divider.x);
    assert!(projection.divider.x > projection.primary.x);
    assert!(
        projection.secondary.x + projection.secondary.width > projection.secondary.x,
        "the secondary pane must still advance"
    );
    assert_eq!(
        projection.secondary.x + projection.secondary.width,
        content.x + content.width
    );
    assert!(conserves_width(&projection, 100.0));
}

/// Non-finite divider positions are rejected, never stored.
#[test]
fn non_finite_position_rejected() {
    assert_eq!(DividerPosition::new(f32::NAN), Err(NonFinitePosition));
    assert_eq!(DividerPosition::new(f32::INFINITY), Err(NonFinitePosition));
    assert_eq!(
        DividerPosition::new(f32::NEG_INFINITY),
        Err(NonFinitePosition)
    );
}

/// The position bound clamps deterministically: any finite value maps into
/// `[0, 1]` with exact boundary behavior.
#[test]
fn position_bound_clamps_deterministically() {
    assert_eq!(
        DividerPosition::new(-0.5).expect("finite").as_fraction(),
        0.0
    );
    assert_eq!(
        DividerPosition::new(0.0).expect("finite").as_fraction(),
        0.0
    );
    assert_eq!(
        DividerPosition::new(0.25).expect("finite").as_fraction(),
        0.25
    );
    assert_eq!(
        DividerPosition::new(1.0).expect("finite").as_fraction(),
        1.0
    );
    assert_eq!(
        DividerPosition::new(1.5).expect("finite").as_fraction(),
        1.0
    );
}

/// f32 no-op equality: a value that clamps onto an existing position compares
/// equal, which is what `evaluate_resize` reports as `changed: false`.
#[test]
fn f32_clamped_no_op_equality() {
    assert_eq!(
        DividerPosition::new(-0.5).expect("finite"),
        DividerPosition::new(0.0).expect("finite")
    );
    assert_eq!(
        DividerPosition::new(2.0).expect("finite"),
        DividerPosition::new(1.0).expect("finite")
    );
    assert_eq!(
        DividerPosition::new(0.25).expect("finite"),
        DividerPosition::new(0.25).expect("finite")
    );
}

/// Exact debug representations for the plain value types.
#[test]
fn debug_strings_are_exact() {
    assert_eq!(
        format!("{:?}", DividerPosition::new(0.5).expect("finite")),
        "DividerPosition(0.5)"
    );
    assert_eq!(
        format!("{:?}", WebRect::new(1.0, 2.0, 3.0, 4.0)),
        "WebRect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 }"
    );
    assert_eq!(
        format!(
            "{:?}",
            project(
                WebRect::new(0.0, 0.0, 100.0, 60.0),
                DividerPosition::new(0.5).expect("finite"),
                8.0
            )
            .expect("valid geometry")
        ),
        "Projection { primary: WebRect { x: 0.0, y: 0.0, width: 46.0, height: 60.0 }, \
         divider: WebRect { x: 46.0, y: 0.0, width: 8.0, height: 60.0 }, \
         secondary: WebRect { x: 54.0, y: 0.0, width: 46.0, height: 60.0 } }"
    );
}
