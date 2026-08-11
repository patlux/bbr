//! Narrow plain render geometry for the floating command surfaces.
//!
//! The card's resolved placement rect in host layout coordinates
//! (top-left origin, points). The app's portal probe resolves the rect
//! from its app-owned Portal geometry and passes it in, so this module
//! never imports the portal geometry types — the same boundary the find
//! and context-menu surfaces use.

/// The card's resolved placement rect in host layout coordinates
/// (top-left origin, points) — the resolved portal placement the app's
/// probe computes, passed in so the card draws at the resolved rect
/// without importing the portal geometry types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CommandRect {
    /// A fresh rect from its corner and size.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
