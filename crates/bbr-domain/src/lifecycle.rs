//! Portable tab resource lifecycle.

/// Resource state of a tab's disposable native web session.
///
/// The domain stores only policy state. The orchestration layer performs the
/// corresponding engine work (keep live, destroy, or recreate a web view).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLifecycle {
    /// Visible in the selected single pane or in either split pane.
    Active,
    /// Hidden but backed by a live native web view.
    Warm,
    /// Restorable state retained while the native web view is destroyed.
    Suspended,
    /// Recoverable navigation entry after process termination or explicit
    /// resource discard.
    Discarded,
    /// The user closed a pinned tab (DESIGN.md §10.12): the identity keeps
    /// its place in the pinned region at its pinned page, owns no session,
    /// and only an explicit selection reopens it. Entered only through
    /// [`crate::Browser::close_tab`], never through the resource policy.
    Closed,
}

impl TabLifecycle {
    /// Whether the state machine permits a direct transition to `next`.
    ///
    /// The explicit resource-policy transitions stay gradual (`Warm` must
    /// pass through `Suspended` before `Discarded`); a process crash is not a
    /// gradual move and is applied directly by [`crate::Browser::discard_tab`],
    /// which bypasses the matrix and the visibility guard. `Closed` is a
    /// user decision, not a resource step: only closing enters it, and only
    /// activation leaves it.
    pub fn can_transition_to(self, next: Self) -> bool {
        self == next
            || matches!(
                (self, next),
                (Self::Active, Self::Warm)
                    | (Self::Warm, Self::Active | Self::Suspended)
                    | (Self::Suspended, Self::Active | Self::Discarded)
                    | (Self::Discarded, Self::Active)
                    | (Self::Closed, Self::Active)
            )
    }

    /// Whether the tab owns (or should own) a live native session.
    pub fn is_live(self) -> bool {
        matches!(self, Self::Active | Self::Warm)
    }

    /// Apply one legal lifecycle transition.
    pub fn transition_to(&mut self, next: Self) -> Result<(), LifecycleError> {
        if self.can_transition_to(next) {
            *self = next;
            Ok(())
        } else {
            Err(LifecycleError {
                from: *self,
                to: next,
            })
        }
    }
}

/// An attempted lifecycle transition that skips a required intermediate state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleError {
    pub from: TabLifecycle,
    pub to: TabLifecycle,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "illegal tab lifecycle transition from {:?} to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for LifecycleError {}
