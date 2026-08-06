//! Pure, portable two-pane geometry: a bounded normalized divider position and
//! the deterministic projection of a content rectangle into
//! primary/divider/secondary rectangles using the established
//! [`bbr_web::WebRect`] host-layout type (`f32` points, top-left origin).

use bbr_web::WebRect;

/// The minimum pane width [`project`] guarantees for every accepted content
/// rect. The divider width is preserved verbatim, so the projection requires
/// `content.width >= 2 * MIN_PANE_WIDTH + divider_width`; anything narrower is
/// rejected with [`GeometryError::InsufficientWidth`].
///
/// The value is deliberately small and compatible with the current 2 pt
/// divider in [Design §11.7](../../../DESIGN.md): two 1 pt panes plus the
/// divider fit any realistic pane. It is a plain, portable layout constant —
/// no drag feel, animation, or keyboard step policy is implied.
pub const MIN_PANE_WIDTH: f32 = 1.0;

/// The initial normalized divider position carried by a validated open plan:
/// an even 50/50 split, matching the shared contract's two equal adjacent
/// panes.
pub const DEFAULT_POSITION: f32 = 0.5;

/// Rejection reason for a non-finite divider position (`NaN` or infinite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonFinitePosition;

/// A bounded normalized divider position.
///
/// The stored value is always in `[0.0, 1.0]`. Construction rejects
/// non-finite input (never silently propagating `NaN`/Infinity) and clamps
/// out-of-range finite values into the bound, so the invariant holds for
/// every constructed value. Equality is plain `f32` equality: a value that
/// clamps onto an existing position compares equal (a no-op), which is what
/// [`crate::plan::evaluate_resize`] reports as `changed: false`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DividerPosition(f32);

impl DividerPosition {
    /// Build a bounded normalized position from a raw fraction.
    ///
    /// Returns [`NonFinitePosition`] for `NaN`/infinite input; finite values
    /// outside `[0.0, 1.0]` are clamped deterministically.
    pub fn new(fraction: f32) -> Result<Self, NonFinitePosition> {
        if fraction.is_finite() {
            Ok(Self(fraction.clamp(0.0, 1.0)))
        } else {
            Err(NonFinitePosition)
        }
    }

    /// The clamped normalized fraction in `[0.0, 1.0]`.
    pub fn as_fraction(self) -> f32 {
        self.0
    }
}

/// The three rects a content rect projects into: primary pane, divider, and
/// secondary pane, in layout order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    pub primary: WebRect,
    pub divider: WebRect,
    pub secondary: WebRect,
}

/// Why [`project`] refused an input. Every variant reports a concrete,
/// unrepaired defect; malformed finite values are never silently rewritten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeometryError {
    /// A content rect field (`x`, `y`, `width`, or `height`) was `NaN` or
    /// infinite.
    NonFiniteContent,
    /// The divider width was `NaN` or infinite.
    NonFiniteDividerWidth,
    /// The content width was not strictly positive.
    NonPositiveWidth,
    /// The content height was not strictly positive.
    NonPositiveHeight,
    /// The divider width was negative.
    NegativeDividerWidth,
    /// A coordinate endpoint (`x + width` or `y + height`) overflows to
    /// infinity in `f32` even though every input field is finite.
    OverflowingContent,
    /// The content width cannot contain two positive panes at
    /// [`MIN_PANE_WIDTH`] plus the preserved divider width. `width` and
    /// `divider_width` reproduce the offending inputs; `minimum` is
    /// [`MIN_PANE_WIDTH`].
    InsufficientWidth {
        width: f32,
        divider_width: f32,
        minimum: f32,
    },
    /// Every input field is finite and the widths are sufficient, but the
    /// ordered `f32` boundaries cannot advance representably at the content
    /// offset/scale: a pane or the divider would round to zero extent (a
    /// huge `x`, a huge `width`, or a tiny divider swallowed by the local
    /// ulp), or the projected far edge cannot land on the validated content
    /// endpoint to `f32` rounding. The projection is rejected rather than
    /// returning overlapping or zero-extent rects.
    UnrepresentableProjection,
}

/// Project `content` into adjacent primary/divider/secondary rects.
///
/// Deterministic and pure: identical finite inputs always produce identical
/// outputs. The divider width is preserved verbatim; the primary pane width
/// is clamped continuously into `[MIN_PANE_WIDTH, available - MIN_PANE_WIDTH]`
/// where `available = width - divider_width`, so both panes stay at least
/// [`MIN_PANE_WIDTH`] wide. Guarantees for every accepted input:
///
/// - the divider rect keeps the requested width exactly;
/// - both pane widths are `>= MIN_PANE_WIDTH` (hence strictly positive), and
///   the pane height equals the content height;
/// - the ordered `f32` boundaries advance representably at the content
///   offset/scale: `primary.x + primary.width > content.x`,
///   `divider.x + divider.width > divider.x` when the divider width is
///   positive, and `secondary.x + secondary.width > secondary.x`;
/// - ordered adjacency: `divider.x == primary.x + primary.width` and
///   `secondary.x == divider.x + divider.width`;
/// - the final endpoint lands on the validated content endpoint to `f32`
///   rounding: `secondary.x + secondary.width` stays within a couple of
///   ulps of `content.x + content.width` (the panes tile the content with
///   no gap or overlap);
/// - invalid inputs are rejected with a typed [`GeometryError`] and never
///   silently rewritten; geometry whose boundaries cannot advance
///   representably (huge offset, huge width, or a divider swallowed by the
///   local ulp) is rejected with
///   [`GeometryError::UnrepresentableProjection`] instead of returning
///   zero-extent or overlapping rects.
pub fn project(
    content: WebRect,
    position: DividerPosition,
    divider_width: f32,
) -> Result<Projection, GeometryError> {
    if !content.x.is_finite()
        || !content.y.is_finite()
        || !content.width.is_finite()
        || !content.height.is_finite()
    {
        return Err(GeometryError::NonFiniteContent);
    }
    if !divider_width.is_finite() {
        return Err(GeometryError::NonFiniteDividerWidth);
    }
    if content.width <= 0.0 {
        return Err(GeometryError::NonPositiveWidth);
    }
    if content.height <= 0.0 {
        return Err(GeometryError::NonPositiveHeight);
    }
    if divider_width < 0.0 {
        return Err(GeometryError::NegativeDividerWidth);
    }
    // Coordinate endpoints must stay finite. Since every pane offset below is
    // non-negative and bounded by `width`, validating the far edge here also
    // guarantees every intermediate endpoint is finite.
    if !(content.x + content.width).is_finite() || !(content.y + content.height).is_finite() {
        return Err(GeometryError::OverflowingContent);
    }

    let w = content.width;
    let h = content.height;
    let d = divider_width;

    // Two positive panes at the documented minimum plus the preserved divider
    // must fit; the requested divider width is never reduced. The guard on
    // `available` below keeps the manual clamp total even if `f32`
    // subtraction rounds a gate-accepted width below `2 * MIN_PANE_WIDTH`
    // (only possible within an ulp of the boundary).
    if w < 2.0 * MIN_PANE_WIDTH + d {
        return Err(GeometryError::InsufficientWidth {
            width: w,
            divider_width: d,
            minimum: MIN_PANE_WIDTH,
        });
    }

    let available = w - d;
    if available < 2.0 * MIN_PANE_WIDTH {
        return Err(GeometryError::InsufficientWidth {
            width: w,
            divider_width: d,
            minimum: MIN_PANE_WIDTH,
        });
    }

    let max_pane = available - MIN_PANE_WIDTH;
    let raw = position.as_fraction() * available;
    let primary_w = if raw < MIN_PANE_WIDTH {
        MIN_PANE_WIDTH
    } else if raw > max_pane {
        max_pane
    } else {
        raw
    };

    let primary_x = content.x;
    let divider_x = primary_x + primary_w;
    let secondary_x = divider_x + d;
    let secondary_w = w - primary_w - d;
    let end_x = content.x + w;
    let secondary_end = secondary_x + secondary_w;

    // Reject projections whose ordered f32 boundaries do not advance
    // representably at this offset/scale: every pane interval must be
    // strictly increasing, the divider must occupy a positive interval when
    // it has positive width (a zero divider may keep zero extent as long as
    // the panes stay ordered), and the far edge must land on the validated
    // content endpoint to `f32` rounding. All values here are validated
    // finite, so the plain comparisons are total.
    if divider_x <= primary_x {
        return Err(GeometryError::UnrepresentableProjection);
    }
    if d > 0.0 && secondary_x <= divider_x {
        return Err(GeometryError::UnrepresentableProjection);
    }
    if secondary_end <= secondary_x {
        return Err(GeometryError::UnrepresentableProjection);
    }
    let endpoint_budget = 4.0 * f32::EPSILON * end_x.abs().max(1.0);
    if (secondary_end - end_x).abs() > endpoint_budget {
        return Err(GeometryError::UnrepresentableProjection);
    }

    Ok(Projection {
        primary: WebRect::new(primary_x, content.y, primary_w, h),
        divider: WebRect::new(divider_x, content.y, d, h),
        secondary: WebRect::new(secondary_x, content.y, secondary_w, h),
    })
}
