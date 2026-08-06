//! Focused geometry tests: exact values, continuous clamping, typed rejection
//! of invalid geometry, representable-boundary acceptance, boundary behavior,
//! and a property-style table.

use crate::geometry::{project, DividerPosition, GeometryError, Projection, MIN_PANE_WIDTH};
use bbr_web::WebRect;

const WIDTHS: &[f32] = &[42.0, 64.0, 100.0, 1080.0];
const POSITIONS: &[f32] = &[0.0, 0.25, 0.5, 0.75, 1.0];
const DIVIDERS: &[f32] = &[0.0, 2.0, 8.0, 40.0];

/// `f32` rounding budget for total-width conservation: the panes tile the
/// content with no gap or overlap, but the three-width sum may differ from
/// the content width by a few ulps.
fn conserves_width(projection: &Projection, w: f32) -> bool {
    let sum = projection.primary.width + projection.divider.width + projection.secondary.width;
    (sum - w).abs() <= 8.0 * f32::EPSILON * w.max(1.0)
}

/// `f32` rounding budget for the final-endpoint equality: the projected far
/// edge must land on the validated content endpoint (`content.x +
/// content.width`), matching the acceptance gate in `project`.
fn lands_on_content_endpoint(projection: &Projection, content: WebRect) -> bool {
    let end_x = content.x + content.width;
    let budget = 4.0 * f32::EPSILON * end_x.abs().max(1.0);
    (projection.secondary.x + projection.secondary.width - end_x).abs() <= budget
}

/// Table/property coverage: for every combination of content width, divider
/// position, and divider width (all accepted geometry), assert minimum pane
/// widths, preserved divider width, ordered adjacency with representably
/// advancing boundaries, an exact final endpoint on the validated content
/// endpoint, total-width conservation to `f32` rounding, finite values, and
/// determinism.
#[test]
fn property_table_positive_adjacent_conserving_deterministic() {
    for &w in WIDTHS {
        for &p in POSITIONS {
            for &d in DIVIDERS {
                let position = DividerPosition::new(p).expect("finite table position");
                let content = WebRect::new(12.0, 7.0, w, 5.0);

                let first = project(content, position, d).expect("accepted table input");
                let second = project(content, position, d).expect("accepted table input");
                assert_eq!(
                    first, second,
                    "projection must be deterministic for w={w}, p={p}, d={d}"
                );

                assert!(
                    first.primary.width >= MIN_PANE_WIDTH,
                    "primary pane must keep the minimum width for w={w}, p={p}, d={d}"
                );
                assert!(
                    first.secondary.width >= MIN_PANE_WIDTH,
                    "secondary pane must keep the minimum width for w={w}, p={p}, d={d}"
                );
                assert_eq!(
                    first.divider.width, d,
                    "the requested divider width must be preserved for w={w}, p={p}, d={d}"
                );

                for rect in [first.primary, first.divider, first.secondary] {
                    assert!(
                        rect.x.is_finite()
                            && rect.y.is_finite()
                            && rect.width.is_finite()
                            && rect.height.is_finite(),
                        "all projected values must be finite for w={w}, p={p}, d={d}"
                    );
                    assert!(
                        rect.width >= 0.0 && rect.height >= 0.0,
                        "no projected pane may be negative for w={w}, p={p}, d={d}"
                    );
                    assert_eq!(rect.y, content.y, "all panes share the content y");
                    assert_eq!(
                        rect.height, content.height,
                        "all panes share the content height"
                    );
                }

                // Ordered adjacency: divider sits directly after primary,
                // secondary directly after the divider.
                assert_eq!(
                    first.divider.x,
                    first.primary.x + first.primary.width,
                    "divider must be adjacent to the primary pane for w={w}, p={p}, d={d}"
                );
                assert_eq!(
                    first.secondary.x,
                    first.divider.x + first.divider.width,
                    "secondary pane must be adjacent to the divider for w={w}, p={p}, d={d}"
                );

                // Ordered boundaries advance representably: every pane
                // interval is strictly increasing (a zero divider may keep
                // zero extent, the panes around it stay ordered).
                assert!(
                    first.divider.x > first.primary.x,
                    "primary end must advance past content.x for w={w}, p={p}, d={d}"
                );
                if d > 0.0 {
                    assert!(
                        first.secondary.x > first.divider.x,
                        "divider end must advance past the divider start for w={w}, p={p}, d={d}"
                    );
                }
                assert!(
                    first.secondary.x + first.secondary.width > first.secondary.x,
                    "secondary end must advance past the secondary start for w={w}, p={p}, d={d}"
                );

                // The final endpoint lands on the validated content endpoint
                // to `f32` rounding.
                assert!(
                    lands_on_content_endpoint(&first, content),
                    "final endpoint must land on the validated content endpoint for w={w}, p={p}, d={d}"
                );

                assert!(
                    conserves_width(&first, w),
                    "total width must be conserved to f32 rounding for w={w}, p={p}, d={d}: {:?}",
                    first
                );
            }
        }
    }
}

/// Exact default projection: 100 pt content, 8 pt divider, 50/50 position.
#[test]
fn exact_default_projection() {
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let projection =
        project(content, DividerPosition::new(0.5).expect("finite"), 8.0).expect("valid geometry");
    assert_eq!(
        projection,
        Projection {
            primary: WebRect::new(0.0, 0.0, 46.0, 60.0),
            divider: WebRect::new(46.0, 0.0, 8.0, 60.0),
            secondary: WebRect::new(54.0, 0.0, 46.0, 60.0),
        }
    );
}

/// Exact clamped-low projection: position 0 still leaves the primary pane at
/// the minimum pane width and the secondary pane positive.
#[test]
fn exact_clamped_low() {
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let projection =
        project(content, DividerPosition::new(0.0).expect("finite"), 8.0).expect("valid geometry");
    assert_eq!(projection.primary.width, MIN_PANE_WIDTH);
    assert_eq!(projection.secondary.width, 91.0);
    assert_eq!(projection.divider.width, 8.0);
    assert!(projection.secondary.width > 0.0);
}

/// Exact clamped-high projection: position 1 still leaves the secondary pane
/// at the minimum pane width and the primary pane positive.
#[test]
fn exact_clamped_high() {
    let content = WebRect::new(0.0, 0.0, 100.0, 60.0);
    let projection =
        project(content, DividerPosition::new(1.0).expect("finite"), 8.0).expect("valid geometry");
    assert_eq!(projection.secondary.width, MIN_PANE_WIDTH);
    assert_eq!(projection.primary.width, 91.0);
    assert_eq!(projection.divider.width, 8.0);
    assert!(projection.primary.width > 0.0);
}

/// The exact acceptance boundary: `width == 2 * MIN_PANE_WIDTH + divider`
/// yields two panes at exactly the minimum, independent of the position.
#[test]
fn exact_boundary_yields_minimum_panes() {
    let content = WebRect::new(0.0, 0.0, 10.0, 20.0); // 2 * 1.0 + 8.0
    for &p in POSITIONS {
        let projection = project(content, DividerPosition::new(p).expect("finite"), 8.0)
            .expect("valid geometry");
        assert_eq!(projection.primary.width, MIN_PANE_WIDTH, "at p={p}");
        assert_eq!(projection.secondary.width, MIN_PANE_WIDTH, "at p={p}");
        assert_eq!(projection.divider.width, 8.0, "at p={p}");
        assert!(
            conserves_width(&projection, 10.0),
            "boundary must conserve width at p={p}"
        );
    }
}

/// Continuous near-boundary behavior: just past the acceptance boundary the
/// primary width varies continuously with position and width (no jump back
/// to an invented compact layout).
#[test]
fn continuous_near_boundary_behavior() {
    let content = WebRect::new(0.0, 0.0, 10.5, 20.0); // 2 * 1.0 + 8.0 + 0.5
    let at_zero =
        project(content, DividerPosition::new(0.0).expect("finite"), 8.0).expect("valid geometry");
    let at_half =
        project(content, DividerPosition::new(0.5).expect("finite"), 8.0).expect("valid geometry");
    let at_one =
        project(content, DividerPosition::new(1.0).expect("finite"), 8.0).expect("valid geometry");

    assert_eq!(at_zero.primary.width, MIN_PANE_WIDTH);
    assert_eq!(at_one.secondary.width, MIN_PANE_WIDTH);
    assert_eq!(at_half.primary.width, 1.25);
    assert_eq!(at_half.secondary.width, 1.25);
    assert!(
        at_half.primary.width > MIN_PANE_WIDTH
            && at_half.primary.width < at_zero.primary.width + 1.0,
        "near-boundary widths move continuously, got {:?}",
        at_half
    );
    assert_eq!(at_half.divider.width, 8.0, "divider width is preserved");
    assert!(conserves_width(&at_half, 10.5));
}

/// A width one ulp below the acceptance boundary is rejected; the boundary
/// itself is accepted. The narrowest accepted width is
/// `2 * MIN_PANE_WIDTH + divider_width`.
#[test]
fn insufficient_compact_width_rejected() {
    let content = WebRect::new(0.0, 0.0, 9.5, 20.0);
    let error = project(content, DividerPosition::new(0.5).expect("finite"), 8.0)
        .expect_err("9.5 < 2 * 1.0 + 8.0 must be rejected");
    assert_eq!(
        error,
        GeometryError::InsufficientWidth {
            width: 9.5,
            divider_width: 8.0,
            minimum: MIN_PANE_WIDTH,
        }
    );

    // The requested divider width is never silently reduced.
    let narrow_divider = WebRect::new(0.0, 0.0, 2.0, 20.0);
    assert_eq!(
        project(
            narrow_divider,
            DividerPosition::new(0.5).expect("finite"),
            8.0,
        ),
        Err(GeometryError::InsufficientWidth {
            width: 2.0,
            divider_width: 8.0,
            minimum: MIN_PANE_WIDTH,
        })
    );
}
