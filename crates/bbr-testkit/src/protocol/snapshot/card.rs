//! DESIGN.md §2 seated content card evidence on the wire.

use serde::{Deserialize, Serialize};

/// The content card frame the render laid out for the current window state
/// and the native clip the hosting layer applied. The native slot rect
/// itself is the snapshot's `content_rect` / `pane_rects` (the card's inner
/// box).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ContentCardSnapshot {
    /// `sp:content-inset`, or the native titlebar band while the sidebar
    /// is collapsed; `0` in native full screen.
    pub inset_top: f32,
    pub inset_trailing: f32,
    pub inset_bottom: f32,
    /// `0` with the rail (flush), `sp:content-inset` while collapsed.
    pub inset_leading: f32,
    /// `radius:lg`; `0` in native full screen.
    pub radius: f32,
    /// Whether the static `elev:1.content` shadow paints.
    pub shadow: bool,
    /// The window is in native full screen (full-bleed card).
    pub full_screen: bool,
    /// The active WKWebView backing layer's corner radius — the native clip
    /// evidence; `None` without a live native view.
    pub native_radius: Option<f32>,
}
