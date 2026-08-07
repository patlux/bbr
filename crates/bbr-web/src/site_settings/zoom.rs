//! # Zoom value — bounded integer-percent zoom level
//!
//! [`ZoomPercent`] is the portable zoom state of one site. It is an integer
//! percentage with **no fractional state**, so repeated in/out steps can
//! never accumulate floating-point drift, and it is never represented by a
//! raw unchecked `f32` in public state.
//!
//! ## Documented contract
//!
//! - **Default:** [`ZoomPercent::DEFAULT`] = 100%.
//! - **Bounds:** [`ZoomPercent::MIN`] = 25% ..= [`ZoomPercent::MAX`] = 400%.
//! - **Step:** [`ZoomPercent::STEP`] = 25 percentage points per zoom in/out.
//! - **Set:** [`ZoomPercent::new`] rejects any value outside the bounds with
//!   [`ZoomError::OutOfBounds`].
//! - **In/out:** [`ZoomPercent::zoom_in`] / [`ZoomPercent::zoom_out`] move by
//!   exactly one [`STEP`](ZoomPercent::STEP), clamped at the bounds; a step
//!   at a bound is a deterministic no-op.
//! - **Reset:** [`ZoomPercent::reset`] returns the documented default.
//!
//! The native page-zoom factor mapping (WebKit `pageZoom`/`textZoom`) is a
//! later adapter slice; this module only owns the validated value.

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt;

/// A validated integer-percent zoom level in `MIN..=MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoomPercent(u16);

impl ZoomPercent {
    /// Lowest supported zoom, 25%.
    pub const MIN: u16 = 25;
    /// Default zoom, 100%.
    pub const DEFAULT: u16 = 100;
    /// Highest supported zoom, 400%.
    pub const MAX: u16 = 400;
    /// Size of one zoom in/out step, in percentage points.
    pub const STEP: u16 = 25;

    /// Set a zoom level, rejecting values outside the documented bounds.
    pub fn new(percent: u16) -> Result<Self, ZoomError> {
        if (Self::MIN..=Self::MAX).contains(&percent) {
            Ok(Self(percent))
        } else {
            Err(ZoomError::OutOfBounds {
                value: percent,
                min: Self::MIN,
                max: Self::MAX,
            })
        }
    }

    /// The integer percentage value.
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Zoom in by exactly one [`STEP`](Self::STEP), clamped at [`MAX`](Self::MAX).
    pub fn zoom_in(self) -> Self {
        Self(Self::MAX.min(self.0 + Self::STEP))
    }

    /// Zoom out by exactly one [`STEP`](Self::STEP), clamped at [`MIN`](Self::MIN).
    pub fn zoom_out(self) -> Self {
        Self(Self::MIN.max(self.0.saturating_sub(Self::STEP)))
    }

    /// Reset to the documented default level.
    pub const fn reset(self) -> Self {
        Self(Self::DEFAULT)
    }
}

impl Default for ZoomPercent {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

impl fmt::Display for ZoomPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[cfg(feature = "serde")]
impl Serialize for ZoomPercent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ZoomPercent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u16::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Typed failure of a zoom value operation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomError {
    /// The requested percentage is outside the supported range.
    OutOfBounds { value: u16, min: u16, max: u16 },
}

impl fmt::Display for ZoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { value, min, max } => {
                write!(
                    f,
                    "zoom value {value} is outside the supported range {min}..={max}"
                )
            }
        }
    }
}

impl std::error::Error for ZoomError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_constants() {
        assert_eq!(ZoomPercent::MIN, 25);
        assert_eq!(ZoomPercent::DEFAULT, 100);
        assert_eq!(ZoomPercent::MAX, 400);
        assert_eq!(ZoomPercent::STEP, 25);
    }

    #[test]
    fn default_is_100_percent() {
        let default = ZoomPercent::default();
        assert_eq!(default.as_u16(), 100);
        assert_eq!(default, ZoomPercent::new(100).unwrap());
        assert_eq!(default.to_string(), "100%");
    }

    #[test]
    fn set_validates_bounds() {
        assert_eq!(
            ZoomPercent::new(0),
            Err(ZoomError::OutOfBounds {
                value: 0,
                min: 25,
                max: 400,
            })
        );
        assert_eq!(
            ZoomPercent::new(24),
            Err(ZoomError::OutOfBounds {
                value: 24,
                min: 25,
                max: 400,
            })
        );
        assert_eq!(
            ZoomPercent::new(401),
            Err(ZoomError::OutOfBounds {
                value: 401,
                min: 25,
                max: 400,
            })
        );
        assert_eq!(
            ZoomPercent::new(u16::MAX),
            Err(ZoomError::OutOfBounds {
                value: u16::MAX,
                min: 25,
                max: 400,
            })
        );
        assert!(ZoomPercent::new(25).is_ok());
        assert!(ZoomPercent::new(400).is_ok());
        assert_eq!(ZoomPercent::new(137).unwrap().as_u16(), 137);
        assert_eq!(ZoomPercent::new(255).unwrap().as_u16(), 255);
    }

    #[test]
    fn zoom_in_steps_by_twenty_five_from_default() {
        let mut level = ZoomPercent::default();
        let mut expected = 100;
        for _ in 0..12 {
            level = level.zoom_in();
            expected += 25;
            assert_eq!(level.as_u16(), expected);
        }
        assert_eq!(level.as_u16(), ZoomPercent::MAX);
    }

    #[test]
    fn full_in_ladder_from_min_is_exact() {
        let mut level = ZoomPercent::new(ZoomPercent::MIN).unwrap();
        let mut seen = Vec::new();
        for _ in 0..15 {
            level = level.zoom_in();
            seen.push(level.as_u16());
        }
        let expected: Vec<u16> = (2..=16).map(|k| 25 * k).collect();
        assert_eq!(seen, expected);
        assert_eq!(level.as_u16(), ZoomPercent::MAX);
    }

    #[test]
    fn full_out_ladder_from_max_is_exact() {
        let mut level = ZoomPercent::new(ZoomPercent::MAX).unwrap();
        let mut seen = Vec::new();
        for _ in 0..15 {
            level = level.zoom_out();
            seen.push(level.as_u16());
        }
        let expected: Vec<u16> = (1..=15).rev().map(|k| 25 * k).collect();
        assert_eq!(seen, expected);
        assert_eq!(level.as_u16(), ZoomPercent::MIN);
    }

    #[test]
    fn zoom_out_from_default_reaches_min_in_three_steps() {
        let mut level = ZoomPercent::default();
        level = level.zoom_out();
        assert_eq!(level.as_u16(), 75);
        level = level.zoom_out();
        assert_eq!(level.as_u16(), 50);
        level = level.zoom_out();
        assert_eq!(level.as_u16(), 25);
        assert_eq!(level.zoom_out(), level); // clamps at min
    }

    #[test]
    fn set_then_steps_stay_drift_free() {
        let base = ZoomPercent::new(137).unwrap();
        assert_eq!(base.zoom_in().as_u16(), 162);
        assert_eq!(base.zoom_in().zoom_in().zoom_out().zoom_out(), base);
    }

    #[test]
    fn bounds_clamp_without_drift() {
        let max = ZoomPercent::new(ZoomPercent::MAX).unwrap();
        assert_eq!(max.zoom_in(), max);
        let min = ZoomPercent::new(ZoomPercent::MIN).unwrap();
        assert_eq!(min.zoom_out(), min);
        let mut level = max;
        for _ in 0..64 {
            level = level.zoom_in();
        }
        assert_eq!(level, max);
    }

    #[test]
    fn reset_returns_documented_default() {
        assert_eq!(
            ZoomPercent::new(25).unwrap().reset(),
            ZoomPercent::default()
        );
        assert_eq!(
            ZoomPercent::new(400).unwrap().reset(),
            ZoomPercent::default()
        );
        assert_eq!(ZoomPercent::default().reset(), ZoomPercent::default());
    }

    #[test]
    fn integer_state_has_no_drift_over_many_steps() {
        // Integer-percent arithmetic is exact by construction: 1000 paired
        // in/out steps must land exactly on the starting level.
        let mut level = ZoomPercent::new(101).unwrap();
        for _ in 0..1000 {
            level = level.zoom_in().zoom_out();
        }
        assert_eq!(level.as_u16(), 101);
    }

    #[test]
    fn debug_shape_is_exact() {
        assert_eq!(
            format!("{:?}", ZoomPercent::new(100).unwrap()),
            "ZoomPercent(100)"
        );
    }

    #[test]
    fn error_display_is_explicit() {
        assert_eq!(
            ZoomPercent::new(401).unwrap_err().to_string(),
            "zoom value 401 is outside the supported range 25..=400"
        );
    }
}
