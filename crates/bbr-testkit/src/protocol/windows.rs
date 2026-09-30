//! Blueprint D19: the browser windows on the wire.

use serde::{Deserialize, Serialize};

/// Blueprint D19: one open browser window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowEntry {
    /// The window's key (the first window is 0).
    pub window: u64,
    /// The snapshot's other fields describe this window.
    pub current: bool,
    pub active_space: Option<u64>,
    /// The window's selected tab in its active space: its index there
    /// (`None`: the space's empty state) and its live session.
    pub selected_index: Option<usize>,
    pub selected_view: Option<bbr_web::WebViewId>,
    /// The window whose native root hosts that session (the engine's own
    /// bookkeeping; macOS only).
    pub selected_root: Option<u64>,
    /// The window's content rect `[x, y, width, height]` as it last drew
    /// (what a save writes).
    #[serde(default)]
    pub content_rect: Option<[f64; 4]>,
    /// The window's rail: shown, and its requested width.
    #[serde(default)]
    pub sidebar_visible: bool,
    #[serde(default)]
    pub sidebar_width: f64,
    /// The window's frame `[x, y, width, height]` in screen points as it
    /// last drew (a drag between windows hit-tests against it).
    #[serde(default)]
    pub frame: Option<[f32; 4]>,
    /// What a tab dragged in from another window would do here:
    /// `rail:<slot>` or `card:<zone>`.
    #[serde(default)]
    pub incoming_drop: Option<String>,
}
