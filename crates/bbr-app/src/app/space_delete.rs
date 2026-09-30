//! Blueprint D05 (DESIGN.md §11.3/§11.13): the Delete Space confirmation
//! sheet. The space menu's Delete Space… opens it bound to the exact
//! `SpaceId`; it asks once, names what closes (counts from the
//! organization authority), and deletes only on the danger button.
//!
//! The sheet renders in the one native portal (it sits over web content),
//! attached to the content card's top edge, with a `wash.active` veil over
//! the card. It owns the keyboard while open: Return triggers the focused
//! button (Cancel when it opens — a destructive default is never one
//! keystroke away), Tab moves between the two buttons, Escape cancels. No
//! outside press dismisses it.

use bbr_domain::SpaceId;
use bbr_ui::surfaces::space_delete::{
    delete_body, delete_title, SpaceDeleteButton, SpaceDeleteModel, SPACE_DELETE_SHEET_WIDTH,
};
use gpui::{Context, KeyDownEvent, Window};

use super::BbrApp;
use crate::portal::geometry::{Edges, FitMode, PortalAnchor, Pt, Rect, Sz};
use crate::portal::state::{AnchoredPlacement, DismissalPolicy, PortalPlacement, PortalRequest};

/// The sheet's stable portal presentation id.
pub const SPACE_DELETE_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0xC6);
/// The sheet's portal draw priority (the blocking-sheet slot of the auth
/// sheet).
pub const SPACE_DELETE_PORTAL_PRIORITY: usize = 2;
/// The sheet's height used for placement (title, two body lines, buttons,
/// `sp:5` padding): the card renders its natural height inside it.
const SPACE_DELETE_SHEET_HEIGHT: f32 = 188.0;

/// The open sheet: the bound space, the rendered copy, the keyboard's
/// button, the content-card rect the veil covers, and the focus domain to
/// restore on cancel.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpaceDeleteState {
    pub space: SpaceId,
    pub title: String,
    pub body: String,
    pub focused: SpaceDeleteButton,
    pub card: Rect,
    pub origin_chrome_focused: bool,
}

impl BbrApp {
    /// Open the confirmation sheet for `space` (the space menu's Delete
    /// Space… commit). The last space of a profile never opens a sheet.
    pub fn open_space_delete_sheet(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(summary) = self
            .organization
            .get()
            .space_delete_summary(&self.controller.read(), space)
        else {
            return;
        };
        if !summary.deletable {
            log::debug!("space {space:?} is the last of its profile; no delete sheet");
            return;
        }
        let origin_chrome_focused = self.chrome_focused(window);
        self.dismiss_command_portal(cx);
        self.find_discard(cx);
        self.menu_discard(cx);
        let card = self.content_card_rect(window);
        self.space_delete = Some(SpaceDeleteState {
            space,
            title: delete_title(&summary.title),
            body: delete_body(summary.tabs, summary.folders, summary.favorites),
            focused: SpaceDeleteButton::Cancel,
            card,
            origin_chrome_focused,
        });
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let request = PortalRequest::native(
            SPACE_DELETE_PRESENTATION_ID,
            sheet_placement(card),
            SPACE_DELETE_PORTAL_PRIORITY,
            // A blocking sheet: only Cancel, Delete Space and Escape exit.
            DismissalPolicy::none(),
        );
        self.portal_probe
            .borrow_mut()
            .present_space_delete(request, viewport);
        self.activate_portal_request(request, cx);
        window.focus(&self.space_delete_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        self.emit_focus(window);
        cx.notify();
    }

    /// Whether the sheet is open: its state exists AND the portal shows its
    /// presentation (a surface that replaced it — e.g. the ⌘T palette —
    /// leaves no stale sheet owning the keyboard).
    pub fn space_delete_open(&self) -> bool {
        self.space_delete.is_some()
            && self
                .portal
                .active()
                .is_some_and(|presentation| presentation.id == SPACE_DELETE_PRESENTATION_ID)
    }

    /// The sheet's presentation model and the card rect its veil covers
    /// (the portal render reads this every frame).
    pub(crate) fn space_delete_surface(&self) -> Option<(SpaceDeleteModel, Rect)> {
        self.space_delete.as_ref().map(|state| {
            (
                SpaceDeleteModel {
                    title: state.title.clone().into(),
                    body: state.body.clone().into(),
                    focused: state.focused,
                },
                state.card,
            )
        })
    }

    /// Cancel (the Cancel button, Escape, Return on Cancel): close the
    /// sheet, delete nothing, restore the origin focus domain.
    pub fn space_delete_cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.space_delete_open() {
            return;
        }
        let Some(state) = self.space_delete.take() else {
            return;
        };
        self.portal_dismiss(cx);
        if state.origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
        cx.notify();
    }

    /// Delete Space (the danger button, or Return while it is focused):
    /// delete the bound space through the organization authority, then
    /// present the fallback space when the deleted one was active.
    pub fn space_delete_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.space_delete_open() {
            return;
        }
        let Some(state) = self.space_delete.take() else {
            return;
        };
        self.portal_dismiss(cx);
        let outcome = self
            .organization
            .get()
            .delete_space(&mut self.controller.get(), state.space);
        match outcome {
            Ok(bbr_browser::OrganizationEvent::SpaceDeleted {
                space,
                fallback,
                closed_tabs,
                activated,
                ..
            }) => {
                self.emit_browser(&bbr_browser::ControllerEvent::SpaceDeleted {
                    space,
                    fallback,
                    closed_tabs,
                    activated,
                });
                if activated {
                    // The controller already switched; the app path runs
                    // the presentation side (tint, chip scroll, find/peek).
                    self.activate_space(fallback, window, cx);
                }
                self.persist_structure();
            }
            Ok(other) => log::error!("unexpected delete-space outcome: {other:?}"),
            Err(error) => log::error!("could not delete space {:?}: {error}", state.space),
        }
        self.focus_space_strip(window, cx);
        cx.notify();
    }

    /// Tab (the `AuthSheetFieldTab` action while the sheet is open): move
    /// the keyboard between Cancel and Delete Space.
    pub(crate) fn space_delete_toggle_focus(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.space_delete_open() {
            return false;
        }
        let Some(state) = self.space_delete.as_mut() else {
            return false;
        };
        state.focused = match state.focused {
            SpaceDeleteButton::Cancel => SpaceDeleteButton::Delete,
            SpaceDeleteButton::Delete => SpaceDeleteButton::Cancel,
        };
        cx.notify();
        true
    }

    /// The sheet's keyboard while it is open (the root routes here): Return
    /// triggers the focused button, Tab/arrows move between the buttons.
    pub(super) fn on_space_delete_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.space_delete.as_ref() else {
            return;
        };
        match event.keystroke.key.as_str() {
            "enter" | "space" => {
                cx.stop_propagation();
                match state.focused {
                    SpaceDeleteButton::Cancel => self.space_delete_cancel(window, cx),
                    SpaceDeleteButton::Delete => self.space_delete_confirm(window, cx),
                }
            }
            "tab" | "left" | "right" => {
                cx.stop_propagation();
                self.space_delete_toggle_focus(cx);
            }
            "escape" => {
                cx.stop_propagation();
                self.space_delete_cancel(window, cx);
            }
            _ => {}
        }
    }

    /// The content card's rect in window coordinates (the veil's extent).
    fn content_card_rect(&self, window: &Window) -> Rect {
        let width = f32::from(window.bounds().size.width);
        let height = f32::from(window.bounds().size.height);
        let card = self.content_card_frame(window);
        let sidebar = if self.sidebar_visible {
            self.sidebar.effective_width(width)
        } else {
            0.0
        };
        let left = sidebar + card.leading;
        Rect::new(
            Pt::new(left, card.top),
            Sz::new(
                (width - card.trailing - left).max(0.0),
                (height - card.bottom - card.top).max(0.0),
            ),
        )
    }

    /// The typed E2E snapshot of the sheet.
    #[cfg(feature = "testkit")]
    pub fn space_delete_snapshot(&self) -> Option<bbr_testkit::protocol::SpaceDeleteSnapshot> {
        if !self.space_delete_open() {
            return None;
        }
        self.space_delete
            .as_ref()
            .map(|state| bbr_testkit::protocol::SpaceDeleteSnapshot {
                space: state.space.0,
                title: state.title.clone(),
                body: state.body.clone(),
                delete_focused: state.focused == SpaceDeleteButton::Delete,
                card: [
                    state.card.x,
                    state.card.y,
                    state.card.width,
                    state.card.height,
                ],
            })
    }
}

/// The sheet attaches to the card's top edge, centred on the card, and is
/// snapped into the window with the §11.4 margin.
fn sheet_placement(card: Rect) -> PortalPlacement {
    PortalPlacement::Anchored(AnchoredPlacement {
        anchor_point: Pt::new(card.x + card.width / 2.0, card.y),
        anchor: PortalAnchor::TopCenter,
        size: Sz::new(SPACE_DELETE_SHEET_WIDTH, SPACE_DELETE_SHEET_HEIGHT),
        offset: Pt::new(0.0, 0.0),
        fit: FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
    })
}
