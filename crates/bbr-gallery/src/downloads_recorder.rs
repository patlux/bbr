//! Gallery-owned observation state for the G2 Group 5 Downloads panel story
//! (packet `component-gallery-browser-components.md`).
//!
//! The Downloads story renders the **exact production `bbr-ui` surface**
//! (`bbr_ui::surfaces::downloads::downloads_panel_surface`) with plain
//! fixture models and recorder callbacks — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production callbacks report
//! back so the E2E can assert real pointer interactions and the canvas can
//! reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use bbr_ui::surfaces::downloads::{
    DownloadsClear, DownloadsInsidePress, DownloadsKeyDown, DownloadsRowAction, DownloadsRowDelete,
    DownloadsRowState,
};
use gpui::{App, MouseDownEvent, Window};

/// What a production Downloads callback reports back to the gallery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadObservation {
    /// The §11.12 panel keyboard listener fired (the app's
    /// `on_downloads_key_down`; only reachable while the panel owns
    /// chrome focus).
    KeyDown,
    /// A pointer press inside the panel (§13 one-pointer-action marker —
    /// the root outside-press handler must never dismiss the panel).
    InsidePress,
    /// The §11.12 contextual row action by deterministic panel index.
    RowAction(usize),
    /// The §10.3 row pressed wash by panel index.
    RowPress(usize, bool),
    /// The §10.2 row hover wash by panel index.
    RowHover(usize, bool),
    /// The §17.8 row Delete control by attempt id.
    RowDelete(u64),
    /// The §17.7 Clear panel control.
    Clear,
}

/// The shared Downloads recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DownloadsRecorder {
    /// The keyboard events the production `on_key_down` reported.
    pub key_downs: u32,
    /// The pointer presses the production `on_inside_press` reported.
    pub inside_presses: u32,
    /// The panel indices whose contextual action fired (Cancel / Retry /
    /// Restart / Reveal-in-Finder).
    pub actions: Vec<usize>,
    /// The `(index, pressed)` wash transitions (§10.3).
    pub presses: Vec<(usize, bool)>,
    /// The `(index, hovered)` wash transitions (§10.2).
    pub hovers: Vec<(usize, bool)>,
    /// The attempt ids the §17.8 Delete control removed.
    pub deletes: Vec<u64>,
    /// The number of §17.7 Clear activations.
    pub clears: u32,
}

impl DownloadsRecorder {
    /// Apply one observed Downloads interaction.
    pub fn observe(&mut self, event: DownloadObservation) {
        match event {
            DownloadObservation::KeyDown => self.key_downs += 1,
            DownloadObservation::InsidePress => self.inside_presses += 1,
            DownloadObservation::RowAction(index) => self.actions.push(index),
            DownloadObservation::RowPress(index, pressed) => self.presses.push((index, pressed)),
            DownloadObservation::RowHover(index, hovered) => self.hovers.push((index, hovered)),
            DownloadObservation::RowDelete(id) => self.deletes.push(id),
            DownloadObservation::Clear => self.clears += 1,
        }
    }
}

/// The callback bundle the Downloads story routes the production
/// `downloads_panel_surface` callbacks through — the exact
/// [`bbr_ui::surfaces::downloads::DownloadsCallbacks`] shapes. Each
/// callback records its interaction; the propagation contract stays where
/// production owns it (the surface's `activate`/`delete` closures stop
/// propagation, exactly like `bbr-app`'s production wiring, §11.12/§13),
/// so the recorder observes what a real binary would see.
pub struct DownloadsObserver {
    pub on_key_down: DownloadsKeyDown,
    pub on_inside_press: DownloadsInsidePress,
    pub on_row_action: DownloadsRowAction,
    pub on_row_press: DownloadsRowState,
    pub on_row_hover: DownloadsRowState,
    pub on_row_delete: DownloadsRowDelete,
    pub on_clear: DownloadsClear,
}

/// Build the production `downloads_panel_surface` observer callbacks from
/// one record action. `record` re-enters the app's production record path
/// (the app-owned entity upgrade); this constructor mirrors the app's
/// listener bundle exactly — the surface's own closures own the §13
/// stop-propagation contract, the observer only records.
pub fn build_downloads_observer(
    record: impl Fn(DownloadObservation, &mut App) + 'static,
) -> DownloadsObserver {
    let record = Rc::new(record);
    let on_key_down = {
        let record = record.clone();
        Rc::new(
            move |_e: &gpui::KeyDownEvent, _window: &mut Window, cx: &mut App| {
                record(DownloadObservation::KeyDown, cx);
            },
        )
    };
    let on_inside_press = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(DownloadObservation::InsidePress, cx);
            },
        )
    };
    let on_row_action = {
        let record = record.clone();
        Rc::new(move |index: usize, _window: &mut Window, cx: &mut App| {
            record(DownloadObservation::RowAction(index), cx);
        })
    };
    let on_row_press = {
        let record = record.clone();
        Rc::new(
            move |index: usize, pressed: bool, _window: &mut Window, cx: &mut App| {
                record(DownloadObservation::RowPress(index, pressed), cx);
            },
        )
    };
    let on_row_hover = {
        let record = record.clone();
        Rc::new(
            move |index: usize, hovered: bool, _window: &mut Window, cx: &mut App| {
                record(DownloadObservation::RowHover(index, hovered), cx);
            },
        )
    };
    let on_row_delete = {
        let record = record.clone();
        Rc::new(move |id: u64, _window: &mut Window, cx: &mut App| {
            record(DownloadObservation::RowDelete(id), cx);
        })
    };
    let on_clear = {
        let record = record.clone();
        Rc::new(move |_window: &mut Window, cx: &mut App| {
            record(DownloadObservation::Clear, cx);
        })
    };
    DownloadsObserver {
        on_key_down,
        on_inside_press,
        on_row_action,
        on_row_press,
        on_row_hover,
        on_row_delete,
        on_clear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_updates_the_expected_field_only() {
        let mut recorder = DownloadsRecorder::default();
        recorder.observe(DownloadObservation::RowHover(3, true));
        recorder.observe(DownloadObservation::RowHover(3, false));
        recorder.observe(DownloadObservation::RowPress(3, true));
        recorder.observe(DownloadObservation::RowPress(3, false));
        recorder.observe(DownloadObservation::RowAction(3));
        recorder.observe(DownloadObservation::RowDelete(42));
        recorder.observe(DownloadObservation::Clear);
        recorder.observe(DownloadObservation::InsidePress);
        recorder.observe(DownloadObservation::KeyDown);
        assert_eq!(recorder.hovers, vec![(3, true), (3, false)]);
        assert_eq!(recorder.presses, vec![(3, true), (3, false)]);
        assert_eq!(recorder.actions, vec![3]);
        assert_eq!(recorder.deletes, vec![42]);
        assert_eq!(recorder.clears, 1);
        assert_eq!(recorder.inside_presses, 1);
        assert_eq!(recorder.key_downs, 1);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = DownloadsRecorder::default();
        assert!(recorder.hovers.is_empty());
        assert!(recorder.presses.is_empty());
        assert!(recorder.actions.is_empty());
        assert!(recorder.deletes.is_empty());
        assert_eq!(recorder.clears, 0);
        assert_eq!(recorder.inside_presses, 0);
        assert_eq!(recorder.key_downs, 0);
    }
}
