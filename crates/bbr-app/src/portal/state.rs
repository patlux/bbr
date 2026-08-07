//! App-local portal state model (task packet `native-portal-host`, DESIGN.md
//! §11.4/§13/§15). One reusable portal per BBR browser window; a
//! [`PortalRequest`] carries a stable [`PresentationId`], a [`PortalHost`]
//! ([`PortalHost::GpuiDeferred`] escapes local GPUI layout through
//! `deferred(anchored(...)).with_priority(...)`; [`PortalHost::NativeAboveWeb`]
//! crosses the native web boundary), a [`PortalPlacement`], a draw priority,
//! and a [`DismissalPolicy`]. [`PortalState`] enforces exactly one active
//! presentation and deterministic dismissal; geometry resolution stays pure
//! in [`super::geometry`], so this module is AppKit- and GPUI-free.

use super::geometry::{anchored_rect, centered_rect, Edges, FitMode, PortalAnchor, Pt, Rect, Sz};

/// Stable identity of one portal host. The app owns exactly one host per BBR
/// browser window; the id is allocated once at window creation and never
/// changes (mirrors `bbr_web::WebViewId` stability).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortalId(pub u64);

/// Stable identity of one portal presentation. Re-presenting the same id
/// refreshes the active presentation in place ([`PortalTransition::Updated`])
/// instead of replacing it, so callers (e.g. a palette re-render) keep a
/// stable identity across frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PresentationId(pub u64);

/// Which rendering host a presentation uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalHost {
    /// Escapes local GPUI layout through `deferred(anchored(...))` with the
    /// presentation's priority; stays inside the GPUI window.
    GpuiDeferred,
    /// Crosses the native web boundary: the transparent, non-activating
    /// implementation-owned native portal window ordered above the parent
    /// window's WKWebViews.
    NativeAboveWeb,
}

impl PortalHost {
    /// Stable snapshot label (DESIGN §15; the future testkit snapshot exposes
    /// this string verbatim).
    pub fn label(self) -> &'static str {
        match self {
            PortalHost::GpuiDeferred => "gpui-deferred",
            PortalHost::NativeAboveWeb => "native-above-web",
        }
    }

    /// Whether this host places the portal over the native web content
    /// boundary (§13). Only the native host may overlap the glass.
    pub fn crosses_native_boundary(self) -> bool {
        matches!(self, PortalHost::NativeAboveWeb)
    }
}

/// Dismissal policy of a presentation. `escape` covers Escape/commit;
/// `outside_pointer` covers the DESIGN §13 one-outside-pointer-action rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DismissalPolicy {
    pub escape: bool,
    pub outside_pointer: bool,
}

impl DismissalPolicy {
    pub const fn none() -> Self {
        Self {
            escape: false,
            outside_pointer: false,
        }
    }

    pub const fn escape() -> Self {
        Self {
            escape: true,
            outside_pointer: false,
        }
    }

    pub const fn outside_pointer() -> Self {
        Self {
            escape: false,
            outside_pointer: true,
        }
    }

    /// The DESIGN §13 default for floating chrome: dismissed by Escape or one
    /// outside pointer action.
    pub const fn any() -> Self {
        Self {
            escape: true,
            outside_pointer: true,
        }
    }
}

impl Default for DismissalPolicy {
    fn default() -> Self {
        Self::none()
    }
}

/// Declarative placement of a presentation. Geometry resolution is pure
/// ([`PortalPresentation::resolve`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortalPlacement {
    /// Centered over the window content (DESIGN §11.4 centered palette),
    /// snapped into the window keeping `margin`.
    Centered { size: Sz, margin: Edges },
    /// Anchored at a point (DESIGN §11.4 field-anchored overlay), resolved by
    /// [`anchored_rect`].
    Anchored(AnchoredPlacement),
}

/// The anchored variant of [`PortalPlacement`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchoredPlacement {
    /// Anchor point in window coordinates (top-left origin, points).
    pub anchor_point: Pt,
    /// Which corner of the portal attaches to the anchor point.
    pub anchor: PortalAnchor,
    /// The portal's size in points.
    pub size: Sz,
    /// Offset applied to the anchor point before building the rect (e.g. the
    /// 4 pt gap below the command field).
    pub offset: Pt,
    /// Overflow policy ([`FitMode::SnapToWindowWithMargin`] for §11.4
    /// overlays).
    pub fit: FitMode,
}

/// A presentation request: everything the portal host needs to show one
/// surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalRequest {
    pub presentation: PresentationId,
    pub host: PortalHost,
    pub placement: PortalPlacement,
    /// Draw priority, mirroring `deferred(...).with_priority(n)`: higher
    /// values draw on top of lower ones (and order the native host above
    /// siblings).
    pub priority: usize,
    pub dismissal: DismissalPolicy,
}

impl PortalRequest {
    /// A deferred-GPUI request that stays inside the GPUI window.
    pub fn deferred(
        presentation: PresentationId,
        placement: PortalPlacement,
        priority: usize,
        dismissal: DismissalPolicy,
    ) -> Self {
        Self {
            presentation,
            host: PortalHost::GpuiDeferred,
            placement,
            priority,
            dismissal,
        }
    }

    /// A native-above-web request that crosses the native boundary.
    pub fn native(
        presentation: PresentationId,
        placement: PortalPlacement,
        priority: usize,
        dismissal: DismissalPolicy,
    ) -> Self {
        Self {
            presentation,
            host: PortalHost::NativeAboveWeb,
            placement,
            priority,
            dismissal,
        }
    }
}

/// The active presentation: the frozen state of the most recent request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalPresentation {
    pub id: PresentationId,
    pub host: PortalHost,
    pub placement: PortalPlacement,
    pub priority: usize,
    pub dismissal: DismissalPolicy,
}

impl From<PortalRequest> for PortalPresentation {
    fn from(request: PortalRequest) -> Self {
        Self {
            id: request.presentation,
            host: request.host,
            placement: request.placement,
            priority: request.priority,
            dismissal: request.dismissal,
        }
    }
}

impl PortalPresentation {
    /// Resolves the placement to a concrete viewport rect in window
    /// coordinates (top-left origin, points), pure and framework-free.
    pub fn resolve(&self, viewport: Sz, client_inset: Edges) -> Rect {
        match self.placement {
            PortalPlacement::Centered { size, margin } => {
                centered_rect(viewport, size, margin, client_inset)
            }
            PortalPlacement::Anchored(placement) => anchored_rect(
                placement.anchor_point,
                placement.anchor,
                placement.size,
                placement.offset,
                placement.fit,
                viewport,
                client_inset,
            ),
        }
    }
}

/// The deterministic outcome of a [`PortalState`] mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalTransition {
    /// A new presentation became active.
    Presented,
    /// The active presentation id was refreshed in place.
    Updated,
    /// The active presentation was replaced by a different id.
    Replaced,
    /// The active presentation was dismissed.
    Dismissed,
}

/// One-active-presentation portal state for a single window host.
///
/// The invariant is structural: `active` is an `Option`, so there is exactly
/// one active presentation or none. `dismissed` records the last lifecycle
/// action so the presentation layer can distinguish "never shown" from
/// "shown then ordered out" (the native host starts hidden and must never
/// re-show a dismissed presentation without a fresh request).
#[derive(Debug, Clone, PartialEq)]
pub struct PortalState {
    pub id: PortalId,
    active: Option<PortalPresentation>,
    dismissed: bool,
}

impl PortalState {
    pub fn new(id: PortalId) -> Self {
        Self {
            id,
            active: None,
            dismissed: false,
        }
    }

    pub fn active(&self) -> Option<&PortalPresentation> {
        self.active.as_ref()
    }

    /// The host of the active presentation, if any.
    pub fn host(&self) -> Option<PortalHost> {
        self.active.map(|presentation| presentation.host)
    }

    pub fn is_dismissed(&self) -> bool {
        self.dismissed
    }

    /// Presents (or refreshes) a request. Exactly one presentation is active
    /// afterwards: a new id replaces the previous one, the same id updates it
    /// in place, and dismissal state is cleared.
    pub fn present(&mut self, request: PortalRequest) -> PortalTransition {
        self.dismissed = false;
        let transition = match &self.active {
            Some(active) if active.id == request.presentation => PortalTransition::Updated,
            Some(_) => PortalTransition::Replaced,
            None => PortalTransition::Presented,
        };
        self.active = Some(request.into());
        transition
    }

    /// Deterministically dismisses the active presentation. Returns whether a
    /// presentation was actually active; calling again is a no-op.
    pub fn dismiss(&mut self) -> bool {
        if self.active.is_some() {
            self.active = None;
            self.dismissed = true;
            true
        } else {
            false
        }
    }

    /// Whether the active presentation's [`DismissalPolicy`] allows the
    /// DESIGN §13 one-outside-pointer-action dismissal. The portal backdrop's
    /// pointer press and the testkit `PortalOutsideDismiss` request both gate
    /// on this before calling [`Self::dismiss`], so a presentation that opted
    /// out of outside-pointer dismissal is never closed by an outside press.
    pub fn dismisses_on_outside_press(&self) -> bool {
        self.active
            .is_some_and(|presentation| presentation.dismissal.outside_pointer)
    }
}

/// Unit tests live in the sibling `state/tests.rs` module file (the file-size
/// contract; same module path, so `use super::*` reaches the private state).
#[cfg(test)]
mod tests;
