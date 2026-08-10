//! Portable contract for committed web-content navigation gestures.
//!
//! This is the narrow seam the native macOS adapter (bbr-web-macos) and the
//! portable controller (bbr-browser) share:
//!
//! - [`ContentGesturePhase`] is the logical gesture phase the native
//!   `scrollWheel:` tracker reports (started/moved/ended/cancelled). It is
//!   deliberately *logical*: the adapter maps AppKit's `NSEventPhase` onto
//!   it and portable code never sees device event objects.
//! - [`ContentGestureDirection`] is the **normalized** navigation intent:
//!   the macOS adapter converts raw platform deltas (including the
//!   natural-scroll preference) into `Back`/`Forward`. Portable crates never
//!   infer direction from raw device deltas.
//! - [`ContentGestureCommit`] is the typed commit the adapter reports once a
//!   gesture passes the native commit threshold; the controller routes it
//!   through the one history authority.
//!
//! No navigation authority lives here. The controller owns the decision
//! matrix (bbr-browser), WebKit owns the real history state, and this crate
//! only carries plain logical values across the platform boundary.

use crate::WebViewId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The logical phase of one in-flight content gesture, as reported by the
/// platform adapter. `Ended` and `Cancelled` are terminal: a tracker returns
/// to Idle after either and a repeated terminal event can never commit twice.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentGesturePhase {
    /// The gesture began (AppKit `NSEventPhaseBegan`/`MayBegin`).
    Started,
    /// The gesture continued (AppKit `NSEventPhaseChanged`).
    Moved,
    /// The gesture released (AppKit `NSEventPhaseEnded`). May commit.
    Ended,
    /// The gesture was interrupted (AppKit `NSEventPhaseCancelled`). Never
    /// commits.
    Cancelled,
}

/// The logical navigation intent of a committed gesture, normalized by the
/// platform adapter from raw device deltas and the natural-scroll
/// preference. Portable code never infers direction from raw deltas.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentGestureDirection {
    /// Navigate the bound view backward through its real WebKit history —
    /// or, at a settled child-tab root with valid opener provenance, return
    /// to the opener (controller decision, never adapter policy).
    Back,
    /// Navigate the bound view forward through its real WebKit history.
    Forward,
}

/// One committed logical content gesture on exactly one web view.
///
/// The adapter reports this **once** per gesture, only after the native
/// commit threshold and only on `Ended`; cancellation, vertical dominance,
/// boundary rubber-banding, and repeated terminal events never produce a
/// second commit. The controller routes it through the single history
/// authority — never a second navigation-state copy.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentGestureCommit {
    /// The exact `WebViewId`/pane the gesture began over.
    pub view: WebViewId,
    /// The normalized logical direction of the committed gesture.
    pub direction: ContentGestureDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_and_phase_are_plain_values() {
        assert_eq!(ContentGestureDirection::Back, ContentGestureDirection::Back);
        assert_ne!(
            ContentGestureDirection::Back,
            ContentGestureDirection::Forward
        );
        assert_eq!(ContentGesturePhase::Ended, ContentGesturePhase::Ended);
        assert_ne!(ContentGesturePhase::Ended, ContentGesturePhase::Moved);
    }

    #[test]
    fn commit_carries_exact_view_and_direction() {
        let commit = ContentGestureCommit {
            view: WebViewId(7),
            direction: ContentGestureDirection::Back,
        };
        assert_eq!(commit.view, WebViewId(7));
        assert_eq!(commit.direction, ContentGestureDirection::Back);
        assert_eq!(commit.clone(), commit);
    }

    /// The perf harness keys committed gestures by `WebViewId` in ordered
    /// collections, so identifiers must stay plain `Copy` values.
    #[test]
    fn commit_ids_are_copy_and_comparable() {
        let a = ContentGestureCommit {
            view: WebViewId(1),
            direction: ContentGestureDirection::Forward,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
