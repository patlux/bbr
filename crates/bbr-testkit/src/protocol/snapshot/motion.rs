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

/// D10 evidence: the drawn pane illustration on screen and its cycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IllustrationSnapshot {
    /// `empty-space`, `crashed` or `offline`.
    pub kind: String,
    /// `playing` (inside its two cycles) or `rest` (the final frame; the
    /// only state under Reduce Motion).
    pub state: String,
}

/// The motion evidence of the main window.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MotionSnapshot {
    /// The presented moments: loops in render order, then the running
    /// after-commit moments.
    pub moments: Vec<MotionMomentSnapshot>,
    /// The most recent after-commit moment starts, newest last (bounded):
    /// evidence that a moment began after its commit, independent of how
    /// long it runs. A Reduce Motion start without an animation reports
    /// `static`.
    #[serde(default)]
    pub started: Vec<MotionMomentSnapshot>,
    /// D10: the drawn pane illustration, while one is on screen.
    #[serde(default)]
    pub illustration: Option<IllustrationSnapshot>,
    /// Root-view renders since launch (proves a forced frame rendered).
    pub root_renders: u64,
    /// Animation elements the render has built since launch. Only animated
    /// moments request animation frames, so a rendered frame that builds
    /// none requests none — the idle proof that does not depend on the
    /// display link ticking.
    pub animated_elements: u64,
}
