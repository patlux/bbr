//! Stateless typed split interaction: validated open intents/plans and pure
//! focus, close, and divider-resize evaluations. This crate stores no pane,
//! focus, or divider authority; every function evaluates against
//! caller-supplied authoritative values (the owning domain split holds pane
//! identities/order and the space active tab/controller pane layout holds
//! focus) and returns plain ephemeral values.

use crate::geometry::{DividerPosition, DEFAULT_POSITION};

/// The split orientation of the Phase-6 v1 layout. Only horizontal is
/// supported: primary pane left, divider, secondary pane right. There is no
/// vertical/top-bottom behavior in this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
}

/// Typed intent to open a split: exactly two distinct ordered pane identities,
/// the pre-split selected identity (which remains focused), and the split
/// direction.
///
/// Follows the "Open a split" contract: the tab selected before opening stays
/// the focused pane and the second pane never implicitly steals command
/// focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenIntent<T> {
    pub primary: T,
    pub secondary: T,
    pub focused: T,
    pub direction: SplitDirection,
}

/// Why [`validate_open`] rejected an intent. No state is created on
/// rejection; the offending values are reported for the caller's routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenRejection<T> {
    /// Both pane identities were the same value; a split needs two distinct
    /// panes.
    IdenticalPanes { primary: T, secondary: T },
    /// The pre-split focused identity is neither pane; an outside identity
    /// cannot become split focus.
    FocusNotMember { focused: T },
}

/// The validated open request: an ephemeral plan the caller may adopt. It
/// reproduces the ordered pane identities and focus and carries the typed
/// horizontal direction plus the even 50/50 initial divider position. The
/// caller owns the authoritative state; this value has no lifecycle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenPlan<T> {
    pub primary: T,
    pub secondary: T,
    pub focused: T,
    pub direction: SplitDirection,
    pub initial_position: DividerPosition,
}

/// Validate an open request into a [`OpenPlan`].
///
/// The typed direction must be [`SplitDirection::Horizontal`] (the only
/// supported orientation), the pane identities must be distinct, and the
/// focused identity must be one of the two panes. Rejection returns the
/// offending values before any plan exists.
pub fn validate_open<T: Copy + Eq>(intent: OpenIntent<T>) -> Result<OpenPlan<T>, OpenRejection<T>> {
    let OpenIntent {
        primary,
        secondary,
        focused,
        direction: SplitDirection::Horizontal,
    } = intent;
    if primary == secondary {
        return Err(OpenRejection::IdenticalPanes { primary, secondary });
    }
    if focused != primary && focused != secondary {
        return Err(OpenRejection::FocusNotMember { focused });
    }
    Ok(OpenPlan {
        primary,
        secondary,
        focused,
        direction: SplitDirection::Horizontal,
        initial_position: DividerPosition::new(DEFAULT_POSITION).expect("default is finite"),
    })
}

/// Typed intent to make one member pane the browser command target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusIntent<T> {
    pub pane: T,
}

/// Result of [`evaluate_focus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusOutcome<T> {
    /// `pane` is already the focused pane; focus is explicitly unchanged.
    Unchanged { focused: T },
    /// `focused` is now the focused pane; the ordered pane identities are
    /// the caller's and are never reordered by this crate.
    Changed { focused: T },
    /// `pane` is not a member of the split; focus is unchanged.
    Rejected { pane: T },
    /// The caller-supplied `current` focus is not one of the two members, so
    /// the authoritative baseline is inconsistent and the request cannot be
    /// evaluated; focus is unchanged and the invalid value is reported. This
    /// is checked before any [`FocusOutcome::Unchanged`] result, so an
    /// outside `current` can never be reported as the focused pane.
    InvalidCurrent { current: T },
}

/// Evaluate a focus request against the caller's authoritative values.
///
/// `primary`/`secondary` are the ordered pane identities and `current` the
/// currently focused pane, both supplied by the caller (the space active tab
/// / controller pane layout). This function stores no focus authority
/// itself.
///
/// The authoritative `current` must itself be one of the two members: an
/// outside baseline is inconsistent and every request against it is rejected
/// ([`FocusOutcome::InvalidCurrent`]) before any other outcome, so an
/// outside value can never be reported as the focused pane.
///
/// With a member `current`, selecting a member changes focus
/// ([`FocusOutcome::Changed`]); requesting the already-focused member is an
/// explicit [`FocusOutcome::Unchanged`] no-op; an outside request target is
/// rejected ([`FocusOutcome::Rejected`]) and never becomes split focus.
pub fn evaluate_focus<T: Copy + Eq>(
    primary: T,
    secondary: T,
    current: T,
    intent: FocusIntent<T>,
) -> FocusOutcome<T> {
    let FocusIntent { pane } = intent;
    if current != primary && current != secondary {
        return FocusOutcome::InvalidCurrent { current };
    }
    if pane == current {
        FocusOutcome::Unchanged { focused: pane }
    } else if pane == primary || pane == secondary {
        FocusOutcome::Changed { focused: pane }
    } else {
        FocusOutcome::Rejected { pane }
    }
}

/// Result of [`close_outcome`]: both pane identities remain available to the
/// caller and `survivor` is the focused pane, which stays selected per the
/// "Close a split" contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseOutcome<T> {
    pub primary: T,
    pub secondary: T,
    pub survivor: T,
}

/// Produce the close outcome from the caller's authoritative pane identities
/// and focused pane: both identities are returned and the focused pane is
/// identified as the survivor that stays selected.
pub fn close_outcome<T: Copy>(primary: T, secondary: T, focused: T) -> CloseOutcome<T> {
    CloseOutcome {
        primary,
        secondary,
        survivor: focused,
    }
}

/// Typed intent to move the divider: a raw normalized fraction. Non-finite
/// values are rejected by [`evaluate_resize`]; finite values are clamped into
/// the `[0, 1]` bound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeIntent {
    pub position: f32,
}

/// Result of [`evaluate_resize`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeOutcome {
    /// The effective clamped position. `changed` is `false` exactly when the
    /// caller-supplied current position was already the effective value (a
    /// no-op under plain `f32` equality); only a real change reports
    /// `changed: true`.
    Resized {
        position: DividerPosition,
        changed: bool,
    },
    /// `position` was `NaN` or infinite; the current position is unchanged.
    Rejected { position: f32 },
}

/// Evaluate a divider resize against only the caller-supplied current divider
/// value. Non-finite input is rejected ([`ResizeOutcome::Rejected`]); finite
/// input is clamped into `[0, 1]`, and the outcome reports whether the
/// effective position actually differs from `current` (f32 no-op equality).
pub fn evaluate_resize(current: DividerPosition, intent: ResizeIntent) -> ResizeOutcome {
    let position = match DividerPosition::new(intent.position) {
        Ok(position) => position,
        Err(_) => {
            return ResizeOutcome::Rejected {
                position: intent.position,
            }
        }
    };
    ResizeOutcome::Resized {
        position,
        changed: position != current,
    }
}
