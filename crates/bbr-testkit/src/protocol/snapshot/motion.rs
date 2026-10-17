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
    /// Animation elements the render has built since launch. Together with
    /// [`Self::transition_frames`] these are the only animation-frame
    /// requests, so a rendered frame that adds to neither requests none —
    /// the idle proof that does not depend on the display link ticking.
    pub animated_elements: u64,
    /// Animation frames the layout transitions (the §11.4 sidebar slide)
    /// have requested since launch: one per rendered frame while a slide
    /// travels, none once it rests.
    #[serde(default)]
    pub transition_frames: u64,
}

/// §11.4 sidebar slide evidence (DESIGN §6 principle 1a, Patrick
/// 2026-10-17): the committed toggle and how its presentation travelled.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SidebarMotionSnapshot {
    /// The variant the last toggle presented: `animated`, or `static`
    /// (Reduce Motion, or docking the floating rail). `None` before the
    /// first toggle.
    pub variant: Option<String>,
    /// Whether a slide is travelling at snapshot time.
    pub animating: bool,
    /// The presented share of the docked rail at snapshot time: `1`
    /// expanded, `0` collapsed.
    pub presence: f32,
    /// The presented rail edge at snapshot time (window points).
    pub presented_width: f32,
    /// Toggles presented since launch (animated or static).
    pub toggles: u64,
    /// The share the last slide started from (the share on screen at the
    /// toggle: mid-way for a reversal); `None` for a static toggle.
    #[serde(default)]
    pub start_presence: Option<f32>,
    /// How long the last slide travels: `dur:emphasis` for a full slide,
    /// the matching share of it for a reversal; `None` for a static toggle.
    #[serde(default)]
    pub duration_ms: Option<f32>,
    /// Monotonic milliseconds from the last toggle's commit to the first
    /// rendered frame that presented the committed geometry; `None` while
    /// the presentation still travels.
    pub last_settle_ms: Option<f32>,
    /// The frames the last toggle rendered, oldest first (bounded).
    pub frames: Vec<SidebarMotionFrame>,
}

/// One rendered frame of a sidebar slide.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SidebarMotionFrame {
    /// Milliseconds since the toggle committed.
    pub at_ms: f32,
    /// The presented share this frame laid out.
    pub presence: f32,
    /// The rail edge this frame laid out.
    pub rail_width: f32,
    /// The x of the focused native slot the engine held when this frame
    /// rendered (the previous frame's layout): the WKWebView travels with
    /// the rail edge instead of jumping at the end.
    pub native_x: f32,
}
