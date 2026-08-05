//! Two-pane split layout referencing tabs of one space.

use crate::TabId;

/// A side-by-side layout of exactly two tabs inside one space.
///
/// v1 deliberately supports two panes only (PLAN.md Phase 6): a general
/// tiling tree is deferred until real use requires it. A space holds at most
/// one split at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub primary: TabId,
    pub secondary: TabId,
}

impl Split {
    pub fn new(primary: TabId, secondary: TabId) -> Self {
        Self { primary, secondary }
    }

    pub fn contains(&self, tab: TabId) -> bool {
        self.primary == tab || self.secondary == tab
    }
}
