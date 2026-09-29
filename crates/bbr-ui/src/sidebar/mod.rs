//! The G3 Sidebar composition (packet `component-gallery-compositions.md`):
//! the one `bbr-ui::sidebar` authority for the complete §11.4 rail.
//!
//! - [`geometry`] — the pure [`geometry::SidebarGeometry`] width model
//!   (min/default/max, narrow-window clamp, collapse memory, keyboard
//!   steps, drag state) plus the resize-divider component. The app keeps
//!   the input wiring and passes the listeners through the [`Callbacks`].
//! - [`model`] — the plain [`model::SidebarModel`] DTO and the one
//!   [`model::Callbacks`] set of handler closures. `bbr-app` is the
//!   assembler: it builds the model from the controller snapshot and
//!   fills every callback slot with the production app methods; the
//!   gallery fills recorder callbacks.
//! - [`callbacks`] — the concrete forwarding closures that adapt the
//!   `Rc<dyn Fn>` slots to GPUI's concrete `Fn` listener seams.
//! - [`render`] — the rail composition [`render::sidebar_surface`] that
//!   renders the model through the production `bbr-ui` components. The
//!   app-owned child elements (command field anchor, favorites band
//!   anchor, command results, downloads panel, Space configuration) are
//!   grouped in [`render::SidebarChildren`] at this boundary.
//! - [`tabs`] — the pinned block + bounded dense window of the tab region.
//! - [`regions`] — the favorites grid, folders list, plus menu, space
//!   strip, and footer row.
//! - [`strip`] — the §11.3 permanent space strip and identity chips.
//!
//! No browser/session/profile/space/ranking/persistence authority lives
//! here: the compositions receive fully-built models, and activation/
//! ranking/lifecycle stay in `bbr-app`.

pub mod callbacks;
pub mod floating;
pub mod geometry;
pub mod model;
pub mod regions;
pub mod render;
pub mod strip;
pub mod tabs;

pub use geometry::{
    sidebar_divider, SidebarGeometry, CONTENT_MIN, RESIZE_HANDLE_WIDTH, RESIZE_STEP,
    SIDEBAR_DEFAULT, SIDEBAR_DIVIDER_ID, SIDEBAR_DIVIDER_LABEL, SIDEBAR_MAX, SIDEBAR_MIN,
};
pub use model::{
    Callbacks, CommandSlot, FavoritesModel, FoldersModel, FooterModel, PlusMenuModel, SidebarModel,
    StripModel, TabsModel,
};
pub use render::{sidebar_surface, SidebarChildren};
pub use strip::{space_chip, space_chip_id, space_identity_id, space_strip, SPACE_STRIP_ID};
