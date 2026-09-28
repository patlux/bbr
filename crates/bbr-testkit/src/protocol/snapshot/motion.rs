//! DESIGN.md §6/§18.5 motion evidence on the wire: which transient moments
//! the render is presenting (and in which variant), plus the render counters
//! that prove an idle shell requests no animation frames.

use serde::{Deserialize, Serialize};

/// One transient moment the render currently presents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionMomentSnapshot {
    /// The moment name (`loading-arc`, `attention-pulse`,
    /// `address-progress`, `address-progress-finish`).
    pub moment: String,
    /// The §15 id of the element the moment decorates.
    pub target: String,
    /// `animated`, `reduced` (opacity-only) or `static` (no frames).
    pub variant: String,
}

/// The motion evidence of the main window.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MotionSnapshot {
    /// The presented moments, in render order.
    pub moments: Vec<MotionMomentSnapshot>,
    /// Root-view renders since launch (proves a forced frame rendered).
    pub root_renders: u64,
    /// Animation elements the render has built since launch. Only animated
    /// moments request animation frames, so a rendered frame that builds
    /// none requests none — the idle proof that does not depend on the
    /// display link ticking.
    pub animated_elements: u64,
}
