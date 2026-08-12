//! Gallery-owned observation state for the Tab/Favorite context-menu story
//! (task packet `tab-favorite-context-menus`, DESIGN.md §11.13/§15).
//!
//! The story renders the **exact production `bbr-ui` surface**
//! (`bbr_ui::surfaces::tab_menu::tab_menu_card`) with plain fixture
//! models and recorder callbacks — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production row/submenu
//! callbacks report back so the E2E can assert real pointer interactions
//! and the canvas can reflect them.

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Tab/Favorite menu callback reports back to the
/// gallery: which top-level row (or open-submenu row) the pointer
/// hovered/pressed. The production card routes its enabled-row listeners
/// here; disabled rows carry no listeners, so they never report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMenuObservation {
    /// The production top-level `on_hover` fired with the new hovered
    /// state (`true` = the pointer entered an enabled row).
    RowHovered { index: usize, hovered: bool },
    /// The production top-level mouse-down fired for an enabled row.
    RowPressed(usize),
    /// The production submenu-row `on_hover` fired (`true` = entered).
    SubmenuHovered { index: usize, hovered: bool },
    /// The production submenu-row mouse-down fired for an enabled row.
    SubmenuPressed(usize),
}

/// The shared Tab-menu recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabMenuRecorder {
    /// The `(row index, hovered state)` pairs the production top-level
    /// `on_hover` reported, in dispatch order.
    pub hovers: Vec<(usize, bool)>,
    /// The row indexes the production top-level mouse-down reported.
    pub presses: Vec<usize>,
    /// The `(row index, hovered state)` pairs the production submenu
    /// `on_hover` reported, in dispatch order.
    pub submenu_hovers: Vec<(usize, bool)>,
    /// The row indexes the production submenu mouse-down reported.
    pub submenu_presses: Vec<usize>,
}

impl TabMenuRecorder {
    /// Apply one observed Tab/Favorite menu interaction.
    pub fn observe(&mut self, event: TabMenuObservation) {
        match event {
            TabMenuObservation::RowHovered { index, hovered } => self.hovers.push((index, hovered)),
            TabMenuObservation::RowPressed(index) => self.presses.push(index),
            TabMenuObservation::SubmenuHovered { index, hovered } => {
                self.submenu_hovers.push((index, hovered))
            }
            TabMenuObservation::SubmenuPressed(index) => self.submenu_presses.push(index),
        }
    }
}

/// A hover callback carrying the production `(row index, hovered)` shape
/// the `bbr-ui` tab-menu card `on_row_hover`/`on_submenu_hover` take.
pub type TabMenuHoverFn = Rc<dyn Fn(usize, &bool, &mut Window, &mut App)>;
/// A press callback carrying the production `(row index, MouseDownEvent)`
/// shape the `bbr-ui` tab-menu card `on_row_press`/`on_submenu_press` take.
pub type TabMenuPressFn = Rc<dyn Fn(usize, &MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the story routes the production `tab_menu_card`
/// callbacks through. Each callback records its interaction. Disabled rows
/// never reach either callback (the production card attaches no listeners
/// to them).
pub struct TabMenuObserver {
    pub on_row_hover: TabMenuHoverFn,
    pub on_row_press: TabMenuPressFn,
    pub on_submenu_hover: TabMenuHoverFn,
    pub on_submenu_press: TabMenuPressFn,
}

/// Build the production `tab_menu_card` observer callbacks from one record
/// action.
pub fn build_tab_menu_observer(
    record: impl Fn(TabMenuObservation, &mut App) + 'static,
) -> TabMenuObserver {
    let record = Rc::new(record);
    let on_row_hover = {
        let record = record.clone();
        Rc::new(
            move |index: usize, hovered: &bool, _window: &mut Window, cx: &mut App| {
                record(
                    TabMenuObservation::RowHovered {
                        index,
                        hovered: *hovered,
                    },
                    cx,
                );
            },
        )
    };
    let on_row_press = {
        let record = record.clone();
        Rc::new(
            move |index: usize, _e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(TabMenuObservation::RowPressed(index), cx);
            },
        )
    };
    let on_submenu_hover = {
        let record = record.clone();
        Rc::new(
            move |index: usize, hovered: &bool, _window: &mut Window, cx: &mut App| {
                record(
                    TabMenuObservation::SubmenuHovered {
                        index,
                        hovered: *hovered,
                    },
                    cx,
                );
            },
        )
    };
    let on_submenu_press = {
        let record = record.clone();
        Rc::new(
            move |index: usize, _e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(TabMenuObservation::SubmenuPressed(index), cx);
            },
        )
    };
    TabMenuObserver {
        on_row_hover,
        on_row_press,
        on_submenu_hover,
        on_submenu_press,
    }
}
