//! Gallery-owned observation state for the G2 Group 9 Context-menu story
//! (packet `component-gallery-browser-components.md`).
//!
//! The Context-menu story renders the **exact production `bbr-ui`
//! surface** (`bbr_ui::surfaces::menu::context_menu_card`) with plain
//! fixture models and recorder callbacks — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production row callbacks
//! report back so the E2E can assert real pointer interactions and the
//! canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Context-menu row callback reports back to the
/// gallery: which row index the pointer hovered/pressed (the production
/// card routes its enabled-row listeners here; disabled rows carry no
/// listeners, so they never report).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuObservation {
    /// The production `on_hover` fired for the row (`menu_select` in the
    /// app — the shared keyboard/pointer cursor) with the new hovered
    /// state (`true` = the pointer entered the row, `false` = it left).
    RowHovered {
        /// The row index the pointer hovered.
        index: usize,
        /// Whether the row is now hovered (`true` entered / `false` left).
        hovered: bool,
    },
    /// The production mouse-down fired for the row (`menu_activate` in the
    /// app — select + activate + dismiss).
    RowPressed(usize),
}

/// The shared Context-menu recorder: plain fixture observation state,
/// written only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextMenuRecorder {
    /// The `(row index, hovered state)` pairs the production `on_hover`
    /// reported, in dispatch order (a `hovered: true` entry proves the
    /// pointer entered an enabled row; `false` proves it left).
    pub hovers: Vec<(usize, bool)>,
    /// The row indexes the production mouse-down reported, in dispatch
    /// order (an entry proves a pointer press on an enabled row).
    pub presses: Vec<usize>,
}

impl ContextMenuRecorder {
    /// Apply one observed Context-menu interaction.
    pub fn observe(&mut self, event: ContextMenuObservation) {
        match event {
            ContextMenuObservation::RowHovered { index, hovered } => {
                self.hovers.push((index, hovered))
            }
            ContextMenuObservation::RowPressed(index) => self.presses.push(index),
        }
    }
}

/// A hover callback carrying the production `(row index, hovered)` shape
/// the `bbr-ui` context-menu card `on_row_hover` takes, re-entering the
/// app through the production record path.
pub type ContextMenuRowHoverFn = Rc<dyn Fn(usize, &bool, &mut Window, &mut App)>;
/// A press callback carrying the production `(row index, MouseDownEvent)`
/// shape the `bbr-ui` context-menu card `on_row_press` takes, re-entering
/// the app through the production record path.
pub type ContextMenuRowPressFn = Rc<dyn Fn(usize, &MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Context-menu story routes the production
/// `context_menu_card` row callbacks through. Each callback records its
/// interaction. The propagation contract stays production-faithful: the
/// production row press does **not** stop propagation (the portal card
/// wrapper owns the §13 stop so the backdrop's one-pointer-action closes
/// the menu; in the gallery the story is the top-level container and the
/// press simply records). Disabled rows never reach either callback (the
/// production card attaches no listeners to them).
pub struct ContextMenuObserver {
    pub on_row_hover: ContextMenuRowHoverFn,
    pub on_row_press: ContextMenuRowPressFn,
}

/// Build the production `context_menu_card` observer callbacks from one
/// record action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade); this constructor owns the production
/// **propagation contract**: neither callback stops propagation (the
/// production card itself does not; the portal card wrapper owns the §13
/// stop, and in the gallery the story is the top-level container).
pub fn build_context_menu_observer(
    record: impl Fn(ContextMenuObservation, &mut App) + 'static,
) -> ContextMenuObserver {
    let record = Rc::new(record);
    let on_row_hover = {
        let record = record.clone();
        Rc::new(
            move |index: usize, hovered: &bool, _window: &mut Window, cx: &mut App| {
                record(
                    ContextMenuObservation::RowHovered {
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
                record(ContextMenuObservation::RowPressed(index), cx);
            },
        )
    };
    ContextMenuObserver {
        on_row_hover,
        on_row_press,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_hovered_and_pressed_row_indexes() {
        let mut recorder = ContextMenuRecorder::default();
        recorder.observe(ContextMenuObservation::RowHovered {
            index: 2,
            hovered: true,
        });
        recorder.observe(ContextMenuObservation::RowHovered {
            index: 0,
            hovered: false,
        });
        recorder.observe(ContextMenuObservation::RowPressed(2));
        assert_eq!(recorder.hovers, vec![(2, true), (0, false)]);
        assert_eq!(recorder.presses, vec![2]);
        // A hover never touches the presses and vice versa.
        recorder.observe(ContextMenuObservation::RowPressed(3));
        assert_eq!(recorder.hovers, vec![(2, true), (0, false)]);
        assert_eq!(recorder.presses, vec![2, 3]);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = ContextMenuRecorder::default();
        assert!(recorder.hovers.is_empty());
        assert!(recorder.presses.is_empty());
    }
}
