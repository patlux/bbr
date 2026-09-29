//! Blueprint D08 (DESIGN §6, §18.4): the transient feedback overlays over
//! the content card — the toast that confirms a reversible action and the
//! zoom readout that confirms each zoom step.
//!
//! Both overlap web content, so the one native Portal draws them as an
//! overlay layer above any active presentation. While no presentation is
//! active the Portal runs in its hit-region mode: it captures the pointer
//! only over the overlays' rects and lets it through to the page elsewhere,
//! so a toast never blocks the page. A modal presentation keeps capturing
//! everything. The app owns the lifecycle; `bbr_ui::surfaces::toast` owns
//! the presentation.

use std::cell::Cell;
use std::rc::Rc;

use bbr_ui::icons::Icon;
use bbr_ui::surfaces::toast::{
    zoom_readout_rect, ToastModel, ToastTone, TOAST_DURATION, ZOOM_READOUT_DURATION,
};
use gpui::{Context, SharedString, Task};

use super::BbrApp;
use bbr_command::id::BookmarkId;

/// What a toast's action undoes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastAction {
    /// ⌘D saved this page; Undo removes the record again.
    UndoSavePage(BookmarkId),
}

/// One live toast.
#[derive(Debug, Clone, PartialEq)]
pub struct ToastState {
    pub model: ToastModel,
    pub action: Option<ToastAction>,
}

/// The live zoom readout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoomReadoutState {
    /// The burst identity (its entrance plays once per burst).
    pub generation: u64,
    pub percent: u32,
}

/// App-owned overlay state (see the module docs).
#[derive(Default)]
pub struct TransientOverlays {
    pub toast: Option<ToastState>,
    pub zoom_readout: Option<ZoomReadoutState>,
    /// The content card's inner box of the last main-window frame.
    pub card: Option<[f32; 4]>,
    /// The toast's rendered bounds (window points), recorded by the Portal
    /// render: the toast hugs its content, so its hit region is measured.
    pub toast_bounds: Rc<Cell<Option<[f32; 4]>>>,
    generation: u64,
    toast_task: Option<Task<()>>,
    readout_task: Option<Task<()>>,
}

impl TransientOverlays {
    /// Whether any overlay is on screen.
    pub fn showing(&self) -> bool {
        self.toast.is_some() || self.zoom_readout.is_some()
    }

    /// The capture regions of the overlays that take input (a leaving toast
    /// no longer does).
    pub fn hit_rects(&self) -> Vec<[f32; 4]> {
        let mut rects = Vec::new();
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| !toast.model.leaving)
        {
            if let Some(bounds) = self.toast_bounds.get() {
                rects.push(bounds);
            }
        }
        if let (Some(_), Some(card)) = (self.zoom_readout, self.card) {
            rects.push(zoom_readout_rect(card));
        }
        rects
    }

    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }
}

impl BbrApp {
    /// Show `message` as the one toast (a newer toast replaces an older
    /// one). It leaves after [`TOAST_DURATION`].
    pub(crate) fn show_toast(
        &mut self,
        icon: Icon,
        tone: ToastTone,
        message: impl Into<SharedString>,
        action: Option<(SharedString, ToastAction)>,
        cx: &mut Context<Self>,
    ) {
        let generation = self.transient.next_generation();
        self.transient.toast_bounds.set(None);
        self.transient.toast = Some(ToastState {
            model: ToastModel {
                generation,
                icon,
                tone,
                message: message.into(),
                action: action.as_ref().map(|(label, _)| label.clone()),
                leaving: false,
            },
            action: action.map(|(_, action)| action),
        });
        self.transient.toast_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(TOAST_DURATION).await;
            let _ = this.update(cx, |app, cx| app.toast_leave(generation, cx));
        }));
        self.sync_portal_overlays(cx);
    }

    /// The toast's time is up: play the exit fade, then remove it.
    fn toast_leave(&mut self, generation: u64, cx: &mut Context<Self>) {
        let Some(toast) = self.transient.toast.as_mut() else {
            return;
        };
        if toast.model.generation != generation {
            return;
        }
        toast.model.leaving = true;
        self.transient.toast_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(bbr_ui::theme::Motion::fine())
                .await;
            let _ = this.update(cx, |app, cx| app.dismiss_toast(generation, cx));
        }));
        self.sync_portal_overlays(cx);
    }

    /// Remove the toast of `generation` (a newer toast stays).
    pub(crate) fn dismiss_toast(&mut self, generation: u64, cx: &mut Context<Self>) {
        if self
            .transient
            .toast
            .as_ref()
            .is_some_and(|toast| toast.model.generation == generation)
        {
            self.transient.toast = None;
            self.transient.toast_bounds.set(None);
            self.transient.toast_task = None;
            self.sync_portal_overlays(cx);
        }
    }

    /// The toast's action (the production pointer path and the testkit
    /// seam): undo the confirmed action, then remove the toast at once.
    pub fn toast_action(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(toast) = self.transient.toast.clone() else {
            return false;
        };
        let Some(action) = toast.action else {
            return false;
        };
        match action {
            ToastAction::UndoSavePage(id) => {
                let command = self.saved_pages;
                command.remove(self, id);
            }
        }
        self.dismiss_toast(toast.model.generation, cx);
        cx.notify();
        true
    }

    /// Show (or refresh) the zoom readout at `percent`; it leaves
    /// [`ZOOM_READOUT_DURATION`] after the last step of a burst.
    pub(crate) fn show_zoom_readout(&mut self, percent: u32, cx: &mut Context<Self>) {
        let generation = match self.transient.zoom_readout {
            Some(readout) => readout.generation,
            None => self.transient.next_generation(),
        };
        self.transient.zoom_readout = Some(ZoomReadoutState {
            generation,
            percent,
        });
        self.transient.readout_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(ZOOM_READOUT_DURATION).await;
            let _ = this.update(cx, |app, cx| {
                if app
                    .transient
                    .zoom_readout
                    .is_some_and(|readout| readout.generation == generation)
                {
                    app.transient.zoom_readout = None;
                    app.sync_portal_overlays(cx);
                }
            });
        }));
        self.sync_portal_overlays(cx);
    }

    /// Record the card's inner box for the overlays; a changed box while
    /// an overlay shows re-syncs the Portal regions.
    pub(super) fn record_overlay_card(&mut self, inner: [f32; 4], cx: &mut Context<Self>) {
        if self.transient.card != Some(inner) {
            self.transient.card = Some(inner);
            if self.transient.showing() {
                self.sync_portal_overlays(cx);
            }
        }
    }

    /// Converge the native Portal with the overlay state: visible while a
    /// presentation or an overlay shows; with no presentation active the
    /// Portal captures the pointer only over the overlays.
    pub(crate) fn sync_portal_overlays(&mut self, cx: &mut Context<Self>) {
        let presentation = self.portal.active().is_some();
        let showing = self.transient.showing();
        #[cfg(target_os = "macos")]
        {
            let regions = (!presentation).then(|| self.transient.hit_rects());
            if let Some(bridge) = self.native_portal.as_mut() {
                bridge.set_visible(presentation || showing);
                bridge.set_hit_regions(regions);
            }
            if let Some(handle) = self
                .native_portal
                .as_ref()
                .map(|bridge| bridge.portal_handle())
            {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_root, _window, cx| cx.notify());
                });
            }
        }
        let _ = (presentation, showing);
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_rects_cover_the_measured_toast_and_the_readout_only() {
        let mut overlays = TransientOverlays::default();
        assert!(!overlays.showing());
        assert!(overlays.hit_rects().is_empty());
        overlays.card = Some([260.0, 10.0, 900.0, 700.0]);
        overlays.toast = Some(ToastState {
            model: ToastModel {
                generation: 1,
                icon: Icon::Bookmark,
                tone: ToastTone::Accent,
                message: "Saved to Library".into(),
                action: Some("Undo".into()),
                leaving: false,
            },
            action: None,
        });
        // Not measured yet: no region.
        assert!(overlays.hit_rects().is_empty());
        overlays.toast_bounds.set(Some([560.0, 646.0, 300.0, 40.0]));
        assert_eq!(overlays.hit_rects(), vec![[560.0, 646.0, 300.0, 40.0]]);
        overlays.zoom_readout = Some(ZoomReadoutState {
            generation: 2,
            percent: 110,
        });
        assert_eq!(overlays.hit_rects().len(), 2);
        // A leaving toast no longer takes input.
        overlays.toast.as_mut().unwrap().model.leaving = true;
        assert_eq!(overlays.hit_rects(), vec![[650.0, 34.0, 120.0, 32.0]]);
    }
}
