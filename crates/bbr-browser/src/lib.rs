//! # bbr-browser — portable browser orchestration
//!
//! The controller that owns a [`Browser`] domain model, maps every live tab
//! to exactly one disposable web session, routes navigation commands to the
//! focused tab or split pane, and applies engine [`WebEvent`]s to the owning
//! tab.
//!
//! This crate is **fully portable**: no GPUI, no AppKit/WebKit, no filesystem,
//! no SQLite, no `unsafe`. It depends only on the portable `bbr-domain` and
//! `bbr-web` crates, so every controller contract is testable without a
//! display server via the deterministic fake engine.
//!
//! ## Ownership shape
//!
//! [`BrowserController`] holds the engine as `Rc<RefCell<dyn WebEngine>>`
//! rather than a generic `E: WebEngine`. This is the "equivalent testable
//! ownership shape" the task packet allows: the composition root shares one
//! engine handle between the controller and the GPUI web-view slot, so the
//! same engine backs both the command path and layout. Contract tests drive
//! the fake engine through the very same `dyn WebEngine` seam.
//!
//! ## Session policy
//!
//! Every live tab maps 1:1 to a disposable web session ([`SessionRegistry`]).
//! Active/warm tabs own a live session; suspended/discarded tabs own none and
//! recreate one on activation; closing, suspending, crashing, or crossing a
//! profile destroys the session.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

/// The engine handle the controller drives (and the composition root shares
/// with the GPUI web-view slot): main-thread-only, interior-mutable.
pub type EngineRef = std::rc::Rc<std::cell::RefCell<dyn bbr_web::WebEngine>>;

/// Determines the next `WebViewId` for a fresh session incarnation. Injected
/// by the composition root so E2E runs can use the sequential id generator.
pub type ViewIdAllocator = Box<dyn FnMut() -> u64>;

mod controller;
mod downloads;
mod error;
#[cfg(test)]
pub(crate) mod fake;
mod favicon;
#[cfg(test)]
pub(crate) mod harness;
mod hydration;
mod nav;
mod organization;
mod outcome;
mod pane;
mod peek;
mod provenance;
mod responsiveness;
mod restore;
#[cfg(test)]
mod restore_tests;
mod session;
mod snapshot;
mod tab_menu;
#[cfg(test)]
mod testing;

#[cfg(test)]
mod controller_background_link_tests;
#[cfg(test)]
mod controller_dense_tests;
#[cfg(test)]
mod controller_event_tests;
#[cfg(test)]
mod controller_favicon_tests;
#[cfg(test)]
mod controller_gesture_tests;
#[cfg(test)]
mod controller_move_tests;
#[cfg(test)]
mod controller_pane_tests;
#[cfg(test)]
mod controller_peek_stale_tests;
#[cfg(test)]
mod controller_peek_tests;
#[cfg(test)]
mod controller_profile_tests;
#[cfg(test)]
mod controller_responsiveness_tests;
#[cfg(test)]
mod controller_tests;
#[cfg(test)]
mod hydration_tests;
#[cfg(test)]
mod organization_pin_tests;
#[cfg(test)]
mod organization_space_tests;
#[cfg(test)]
mod organization_tests;
#[cfg(test)]
mod responsiveness_tests;

pub use bbr_domain::{
    FolderId, ProfileHue, SpaceAppearance, SpaceId, SpacePaper, SpaceTheme, TabLifecycle,
    TintIntensity,
};
pub use controller::BrowserController;
pub use downloads::{
    display_filename, url_filename, ApplyOutcome, DownloadActionSink, DownloadController,
    DownloadError, DownloadRecord, RestoredTerminal,
};
pub use error::ControllerError;
pub use hydration::{
    HydrationError, HydrationPolicy, IntentClosedTab, IntentProfile, IntentSpace, IntentSplit,
    IntentTab, RestoreIntent, RestorePlan, RestoreTarget, RestoredClosedTab, RestoredProfile,
    RestoredSpace, RestoredSplit, RestoredTab, SessionRequest,
};
pub use nav::NavState;
pub use organization::{
    FavoriteSnapshot, FolderSnapshot, OrganizationController, OrganizationError, OrganizationEvent,
    OrganizationSnapshot, SpaceDeleteSummary,
};
pub use outcome::ControllerEvent;
pub use pane::PaneLayout;
pub use responsiveness::{
    Clock, RendererAvailability, ResponsivenessConfig, ResponsivenessMonitor,
};
pub use session::{SessionError, SessionRegistry};
pub use snapshot::{
    ControllerSnapshot, GlobalTabSnapshot, LifecycleCounts, ProfileSnapshot, SpaceSnapshot,
    TabFavicon, TabSnapshot,
};
pub use tab_menu::{
    FolderDestination, SpaceDestination, TabMenuAction, TabMenuAvailability, TabMenuController,
    TabMenuError, TabMenuSource,
};
