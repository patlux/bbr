//! Paper S-7b footer dock evidence on the wire: the hovered target, the
//! open states of the fixed controls, the in-flight count badge and the
//! strip's keyboard focus — exactly what the last render was given.

use serde::{Deserialize, Serialize};

/// The footer dock's live presentation state.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FooterSnapshot {
    /// The target under the pointer (`downloads` / `plus` / `space-<id>`).
    pub hovered: Option<String>,
    /// The downloads panel is open (`[Downloads]` stays pressed).
    pub downloads_open: bool,
    /// The plus menu is open (`[+]` stays pressed).
    pub plus_open: bool,
    /// The in-flight count on the `[Downloads]` badge (`None`: no badge).
    pub downloads_count: Option<usize>,
    /// §9: the space strip owns keyboard focus (ring on the active chip).
    pub strip_focused: bool,
}
