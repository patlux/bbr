//! Blueprint D23: the shown split group on the wire.

use bbr_web::WebViewId;
use serde::{Deserialize, Serialize};

/// Blueprint D23: one pane of the shown split group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitPaneSnapshot {
    pub view: WebViewId,
    /// The projected rect (window points).
    pub rect: [f32; 4],
}

/// Blueprint D23: the shown split group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitSnapshot {
    /// In reading order.
    pub panes: Vec<SplitPaneSnapshot>,
    pub dividers: Vec<[f32; 4]>,
    pub focused: WebViewId,
    /// The tree over views, e.g. `R(5000:3,5000:C(5000:4,5000:5))`.
    pub tree: String,
    /// `(n, N)` while the group rests on its focused pane.
    pub collapsed: Option<(usize, usize)>,
    pub list_open: bool,
    /// Split groups in the active space (shown or not).
    pub groups: usize,
    pub dragging: bool,
}

/// Blueprint D23: one split interaction through the method its control
/// (or the native pointer observer) calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitInput {
    /// Place the tab at `new` at `edge` (`left`, `right`, `top`, `bottom`)
    /// of the tab at `beside` (the drop and palette paths' command).
    Insert {
        beside: usize,
        new: usize,
        edge: String,
    },
    /// Press divider `index` (projection order), drag it to `to` along its
    /// axis (window points) and release.
    DragDivider { index: usize, to: f32 },
    /// Double-click divider `index`.
    DoubleClickDivider { index: usize },
    /// A press over the card seen by the native pointer observer.
    PointerDown { x: f32, y: f32 },
    /// The collapsed chip, or one row of its list.
    Chip { row: Option<usize> },
}
