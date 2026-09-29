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
    /// Blueprint D07: a cross-profile move recreated `tab`; Undo moves it
    /// back to `space` (recreating it in the source profile).
    UndoProfileMove {
        tab: bbr_domain::TabId,
        space: bbr_browser::SpaceId,
    },
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

/// Blueprint D17: the one restore notice of a launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreNotice {
    pub message: String,
    /// The selected tab's restored URL: its own load is not "the first
    /// navigation".
    pub restored_url: Option<String>,
}

/// The notice copy for a restore of `spaces` spaces and `tabs` tabs.
pub(crate) fn restore_notice_message(spaces: usize, tabs: usize) -> String {
    let spaces = if spaces == 1 {
        "1 space".to_string()
    } else {
        format!("{spaces} spaces")
    };
    let tabs = if tabs == 1 {
        "1 tab".to_string()
    } else {
        format!("{tabs} tabs")
    };
    format!("Restored {spaces} and {tabs} from your last session.")
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
    /// Blueprint D17: the restore notice ("Restored 3 spaces and 42 tabs
    /// from your last session.") and the restored page's URL — the notice
    /// leaves on the first navigation elsewhere, or on Dismiss.
    pub notice: Option<RestoreNotice>,
    /// The notice's rendered bounds (its hit region), recorded by the
    /// Portal render.
    pub notice_bounds: Rc<Cell<Option<[f32; 4]>>>,
    generation: u64,
    toast_task: Option<Task<()>>,
    readout_task: Option<Task<()>>,
}

impl TransientOverlays {
    /// Whether any overlay is on screen.
    pub fn showing(&self) -> bool {
        self.toast.is_some() || self.zoom_readout.is_some() || self.notice.is_some()
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
        if self.notice.is_some() {
            if let Some(bounds) = self.notice_bounds.get() {
                rects.push(bounds);
            }
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
        profile: Option<bbr_ui::state::ProfileCompanion>,
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
                profile,
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
            ToastAction::UndoProfileMove { tab, space } => {
                self.undo_profile_move(tab, space, cx);
            }
        }
        self.dismiss_toast(toast.model.generation, cx);
        cx.notify();
        true
    }

    /// Blueprint D17: after a restore, say what came back (once per launch).
    pub(crate) fn show_restore_notice(&mut self) {
        let browser = self.controller.snapshot();
        let tabs = browser.all_tabs.len();
        if tabs == 0 {
            return;
        }
        let restored_url = browser
            .active_index
            .and_then(|index| browser.tabs.get(index))
            .map(|tab| tab.url.clone());
        self.transient.notice = Some(RestoreNotice {
            message: restore_notice_message(browser.spaces.len(), tabs),
            restored_url,
        });
    }

    /// Dismiss the restore notice (its Dismiss action and the testkit seam).
    pub fn dismiss_restore_notice(&mut self, cx: &mut Context<Self>) -> bool {
        if self.transient.notice.take().is_none() {
            return false;
        }
        self.transient.notice_bounds.set(None);
        self.sync_portal_overlays(cx);
        true
    }

    /// A navigation landed: the first one away from the restored page ends
    /// the notice.
    pub(crate) fn restore_notice_navigation(&mut self, url: &str, cx: &mut Context<Self>) {
        let away = self
            .transient
            .notice
            .as_ref()
            .is_some_and(|notice| notice.restored_url.as_deref() != Some(url));
        if away {
            self.dismiss_restore_notice(cx);
        }
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

    /// The Portal's capture regions: only the overlays with no
    /// presentation; the washed origin and the bar plus the overlays while
    /// the Peek shows (its preview and the chrome outside the card take the
    /// pointer); everything (`None`) for any other presentation.
    pub(crate) fn portal_hit_regions(&self) -> Option<Vec<[f32; 4]>> {
        // Blueprint D02: a drag or divider drag that began in the floating
        // rail keeps the whole pointer until it ends.
        if self.rail_floating()
            && (self.tab_drag.is_active() || self.tab_drag.is_armed() || self.sidebar.is_dragging())
        {
            return None;
        }
        if self.portal.active().is_none() {
            let mut rects = self.transient.hit_rects();
            rects.extend(self.floating_rail_regions());
            return Some(rects);
        }
        let peek = matches!(
            self.portal_probe.borrow().content(),
            Some(crate::portal::probe::PortalContent::Peek)
        );
        let frame = self.peek_surface().and_then(|surface| surface.frame);
        match (peek, frame) {
            (true, Some(frame)) => {
                let mut rects = frame.capture_rects().to_vec();
                rects.extend(self.transient.hit_rects());
                Some(rects)
            }
            _ => None,
        }
    }

    /// Converge the native Portal with the overlay state: visible while a
    /// presentation or an overlay shows, capturing the pointer over
    /// [`Self::portal_hit_regions`].
    pub(crate) fn sync_portal_overlays(&mut self, cx: &mut Context<Self>) {
        let presentation = self.portal.active().is_some();
        let showing = self.transient.showing() || self.rail_floating();
        #[cfg(target_os = "macos")]
        {
            let regions = self.portal_hit_regions();
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

/// Blueprint D07: the cross-profile move feedback.
impl BbrApp {
    /// The §12 companion of `profile` (monogram + hue + chip index).
    fn profile_companion(
        &self,
        profile: bbr_web::ProfileId,
    ) -> Option<(String, bbr_ui::state::ProfileCompanion)> {
        let browser = self.controller.snapshot();
        browser
            .profiles
            .iter()
            .find(|candidate| candidate.id == profile)
            .map(|found| {
                (
                    found.title.clone(),
                    bbr_ui::state::ProfileCompanion {
                        profile_index: found.index,
                        letter: crate::shell::space::profile_monogram(&found.title),
                        hue: found.hue,
                    },
                )
            })
    }

    /// The drag names the destination profile before a cross-profile drop
    /// ("Move to Work · Profile Work"); `None` over a same-profile chip or
    /// with no chip target.
    pub(crate) fn drag_destination(&self) -> Option<bbr_ui::state::DragDestination> {
        let space = self.tab_drag.chip_target()?;
        let browser = self.controller.snapshot();
        let target = browser.spaces.iter().find(|s| s.id == space)?;
        if target.profile == browser.active_profile {
            return None;
        }
        let (profile_title, companion) = self.profile_companion(target.profile)?;
        Some(bbr_ui::state::DragDestination {
            text: format!("Move to {} · Profile {profile_title}", target.title),
            companion,
        })
    }

    /// A cross-profile move recreated `tab` in `to_space`: the toast names
    /// the destination profile and offers Undo back to `from_space`.
    pub(crate) fn announce_profile_move(
        &mut self,
        tab: bbr_domain::TabId,
        from_space: bbr_browser::SpaceId,
        to_space: bbr_browser::SpaceId,
        cx: &mut Context<Self>,
    ) {
        let browser = self.controller.snapshot();
        let Some(profile) = browser
            .spaces
            .iter()
            .find(|s| s.id == to_space)
            .map(|s| s.profile)
        else {
            return;
        };
        let Some((title, companion)) = self.profile_companion(profile) else {
            return;
        };
        self.show_toast(
            Icon::Check,
            Some(companion),
            ToastTone::Neutral,
            format!("Reloaded in Profile · {title} — sign-ins may differ"),
            Some((
                "Undo".into(),
                ToastAction::UndoProfileMove {
                    tab,
                    space: from_space,
                },
            )),
            cx,
        );
    }

    /// Undo a cross-profile move: move `tab` back to `space` by identity
    /// (it lives in a space that is not active), recreating it in the
    /// source profile's store. No second toast.
    fn undo_profile_move(
        &mut self,
        tab: bbr_domain::TabId,
        space: bbr_browser::SpaceId,
        cx: &mut Context<Self>,
    ) {
        match self.controller.move_tab_id(tab, space) {
            Ok(outcome) => {
                self.clear_command_profile_hint();
                self.sync_command_bar();
                self.find_discard_if_unbound(cx);
                self.emit_browser(&outcome);
                self.sync_peek_surface(cx);
                self.ensure_active_visible();
                self.persist_structure();
            }
            Err(error) => log::error!("could not undo the profile move: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_counts_what_came_back() {
        assert_eq!(
            restore_notice_message(3, 42),
            "Restored 3 spaces and 42 tabs from your last session."
        );
        assert_eq!(
            restore_notice_message(1, 1),
            "Restored 1 space and 1 tab from your last session."
        );
    }

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
                profile: None,
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
