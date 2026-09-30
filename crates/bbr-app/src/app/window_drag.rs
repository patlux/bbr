//! Blueprint D19 (several windows, task packet `multi-window.md`): a tab row
//! dragged out of one window onto another window's rail or card.
//!
//! The drag stays with the window it began in (the pointer events keep
//! coming to it, in its own coordinates, wherever the pointer goes). Each
//! move is turned into a screen point and hit-tested against the browser
//! windows front to back: over the source window the in-window drag
//! resolves as before; over another window, that window previews what the
//! drop would do — the insertion tick at a rail slot (against its rendered
//! rows) or a D23 split zone over its card — and the release commits there;
//! over no window nothing is targeted and the release does nothing (no
//! tear-off).

use bbr_browser::WindowKey;
use bbr_domain::TabId;
use gpui::{Context, Window};

use super::split_view::SplitDrop;
use super::BbrApp;
use crate::window_seam::with_app;

/// What a tab dragged in from another window would do here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum IncomingTarget {
    /// Land at this insertion slot of the active space's rail.
    Rail { slot: usize },
    /// Join the shown pane's group at this zone (D23).
    Card(SplitDrop),
}

/// A tab dragged in from another window, previewed here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct IncomingDrop {
    pub(crate) tab: TabId,
    pub(crate) target: IncomingTarget,
}

/// A window's part in a drag between windows.
#[derive(Debug, Default)]
pub(crate) struct WindowDrag {
    /// This window's drag is over that other window.
    pub(crate) outgoing: Option<WindowKey>,
    /// Another window's drag is over this one.
    pub(crate) incoming: Option<IncomingDrop>,
}

/// The insertion slot for window-y `y` among the rendered rows (tab index,
/// rect): the nearest row boundary; `None` above the first row (the chrome
/// above the list takes no drop). No rows: slot 0.
pub(crate) fn slot_at(rows: &[(usize, bbr_web::WebRect)], y: f32) -> Option<usize> {
    let mut rows = rows.to_vec();
    rows.sort_by_key(|(index, _)| *index);
    let Some((_, first)) = rows.first() else {
        return Some(0);
    };
    if y < first.y - first.height / 2.0 {
        return None;
    }
    rows.iter()
        .flat_map(|(index, rect)| [(*index, rect.y), (index + 1, rect.y + rect.height)])
        .min_by(|a, b| (a.1 - y).abs().total_cmp(&(b.1 - y).abs()))
        .map(|(slot, _)| slot)
}

impl BbrApp {
    /// Source side, on every drag move: hit-test the screen point under the
    /// pointer against the browser windows and preview the drop in the
    /// window it is over. Returns whether this window must drop its own
    /// target (the pointer is over another window or over none).
    pub(crate) fn route_window_drag(
        &mut self,
        x: f32,
        y: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(tab) = self
            .tab_drag
            .source()
            .and_then(|index| self.tab_id_at(index))
        else {
            return false;
        };
        let own = self.window_key();
        let bounds = window.bounds();
        let (origin_x, origin_y) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        let inside_own = x >= 0.0
            && y >= 0.0
            && x < f32::from(bounds.size.width)
            && y < f32::from(bounds.size.height);
        let (screen_x, screen_y) = (origin_x + x, origin_y + y);
        // Every other window the point is over, with its frame.
        let others: Vec<_> = self
            .windows
            .read()
            .keys()
            .into_iter()
            .filter(|key| *key != own)
            .collect();
        let mut hits = Vec::new();
        for key in others {
            let target = self.windows.read().target(key);
            let Some((handle, _)) = target else {
                continue;
            };
            let Ok(frame) = handle.update(cx, |_root, window, _cx| window.bounds()) else {
                continue;
            };
            let (left, top) = (f32::from(frame.origin.x), f32::from(frame.origin.y));
            let (right, bottom) = (
                left + f32::from(frame.size.width),
                top + f32::from(frame.size.height),
            );
            if screen_x >= left && screen_x < right && screen_y >= top && screen_y < bottom {
                hits.push((key, left, top));
            }
        }
        let mut candidates: Vec<WindowKey> = hits.iter().map(|(key, _, _)| *key).collect();
        if inside_own {
            candidates.push(own);
        }
        let stack: Vec<gpui::WindowId> = cx
            .window_stack()
            .unwrap_or_default()
            .iter()
            .map(|handle| handle.window_id())
            .collect();
        let front = self
            .windows
            .read()
            .front_to_back(&candidates, &stack)
            .first()
            .copied();
        let target = front.filter(|key| *key != own);
        if self.window_drag.outgoing != target {
            if let Some(old) = self.window_drag.outgoing.take() {
                self.with_window(old, cx, |view, _window, cx| view.clear_incoming_drop(cx));
            }
        }
        self.window_drag.outgoing = target;
        if let Some(key) = target {
            let (_, left, top) = *hits
                .iter()
                .find(|(hit, _, _)| *hit == key)
                .expect("the target was hit");
            let (local_x, local_y) = (screen_x - left, screen_y - top);
            self.with_window(key, cx, |view, window, cx| {
                view.preview_incoming_drop(tab, local_x, local_y, window, cx)
            });
        }
        !inside_own || target.is_some()
    }

    /// Run `f` in browser window `key` (from inside another window's
    /// update).
    fn with_window(
        &self,
        key: WindowKey,
        cx: &mut Context<Self>,
        f: impl FnOnce(&mut BbrApp, &mut Window, &mut Context<BbrApp>),
    ) {
        let target = self.windows.read().target(key);
        if let Some((handle, app)) = target {
            let _ = with_app(&handle, &app, cx, f);
        }
    }

    /// Source side, on release: the drop happens in the window the pointer
    /// is over. Returns `None` when the drag is not over another window.
    pub(crate) fn finish_window_drag(&mut self, cx: &mut Context<Self>) -> Option<bool> {
        let key = self.window_drag.outgoing.take()?;
        let mut committed = false;
        self.with_window(key, cx, |view, window, cx| {
            committed = view.commit_incoming_drop(window, cx);
        });
        Some(committed)
    }

    /// Source side, on Escape: the other window's preview goes.
    pub(crate) fn cancel_window_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(key) = self.window_drag.outgoing.take() {
            self.with_window(key, cx, |view, _window, cx| view.clear_incoming_drop(cx));
        }
    }

    /// Target side: what dropping `tab` at window point `(x, y)` would do —
    /// a rail slot, a card zone, or nothing — previewed here.
    pub(crate) fn preview_incoming_drop(
        &mut self,
        tab: TabId,
        x: f32,
        y: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (rail_left, rail_right) = self.rail_span(window);
        let target = if x >= rail_left && x < rail_right {
            let rows = self.drag_offer_rects.rows.borrow().clone();
            slot_at(&rows, y).map(|slot| IncomingTarget::Rail { slot })
        } else {
            self.split_drop_at(Some(tab), x, y)
                .map(IncomingTarget::Card)
        };
        let incoming = target.map(|target| IncomingDrop { tab, target });
        if self.window_drag.incoming == incoming {
            return;
        }
        self.window_drag.incoming = incoming;
        self.show_incoming_drop(cx);
    }

    /// Target side: the drag left this window (or was cancelled).
    pub(crate) fn clear_incoming_drop(&mut self, cx: &mut Context<Self>) {
        if self.window_drag.incoming.take().is_some() {
            self.show_incoming_drop(cx);
        }
    }

    /// Present the incoming preview: the rail tick (see `tabs_model`) or the
    /// card zone through this window's Portal.
    fn show_incoming_drop(&mut self, cx: &mut Context<Self>) {
        let card = match self.window_drag.incoming.map(|drop| drop.target) {
            Some(IncomingTarget::Card(drop)) => Some(drop),
            _ => None,
        };
        if self.split_view.drop != card {
            self.split_view.drop = card;
            self.sync_portal_overlays(cx);
        }
        let preview = self.window_drag.incoming.map(|drop| match drop.target {
            IncomingTarget::Rail { slot } => format!("rail:{slot}"),
            IncomingTarget::Card(drop) => {
                format!("card:{}", super::split_view::zone_name(drop.zone))
            }
        });
        let key = self.window_key();
        self.windows.bookkeep().set_incoming(key, preview);
        cx.notify();
    }

    /// The rail tick an incoming drag shows (the slot it would land at).
    pub(crate) fn incoming_rail_slot(&self) -> Option<usize> {
        match self.window_drag.incoming?.target {
            IncomingTarget::Rail { slot } => Some(slot),
            IncomingTarget::Card(_) => None,
        }
    }

    /// Target side, on release: move the tab here — into this window's
    /// active space at the previewed slot, or beside the previewed pane —
    /// and show it; this window takes over.
    pub(crate) fn commit_incoming_drop(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(drop) = self.window_drag.incoming.take() else {
            return false;
        };
        self.show_incoming_drop(cx);
        let committed = match drop.target {
            IncomingTarget::Rail { slot } => self.land_dropped_tab(drop.tab, Some(slot), cx),
            IncomingTarget::Card(split) => {
                self.land_dropped_tab(drop.tab, None, cx)
                    && self
                        .index_of_tab(drop.tab)
                        .is_some_and(|index| self.commit_split_drop(index, split, cx))
            }
        };
        if committed {
            if !self.windows_hidden {
                window.activate_window();
            }
            self.window_activated(cx);
            self.focus_tab_list(window, cx);
        }
        committed
    }

    /// Move `tab` into this window's active space (at `slot`) and show it,
    /// through the controller's one drop authority; a cross-profile move
    /// says so with the D07 toast.
    fn land_dropped_tab(
        &mut self,
        tab: TabId,
        slot: Option<usize>,
        cx: &mut Context<Self>,
    ) -> bool {
        let from_space = self.controller.read().space_of_tab(tab);
        let result = self.controller.get().drop_tab_here(tab, slot);
        match result {
            Ok(event) => {
                if let (
                    bbr_browser::ControllerEvent::TabMoved {
                        tab,
                        to_space,
                        recreated: true,
                    },
                    Some(from_space),
                ) = (&event, from_space)
                {
                    self.announce_profile_move(*tab, from_space, *to_space, cx);
                }
                self.clear_command_profile_hint();
                self.sync_command_bar();
                self.find_discard_if_unbound(cx);
                self.emit_browser(&event);
                self.sync_peek_surface(cx);
                self.ensure_active_visible();
                self.persist_structure();
                cx.notify();
                true
            }
            Err(error) => {
                log::warn!("a tab from another window was refused: {error}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_web::WebRect;

    fn rows() -> Vec<(usize, WebRect)> {
        vec![
            (0, WebRect::new(0.0, 100.0, 300.0, 36.0)),
            (1, WebRect::new(0.0, 140.0, 300.0, 36.0)),
            (3, WebRect::new(0.0, 220.0, 300.0, 36.0)),
        ]
    }

    #[test]
    fn a_point_lands_at_the_nearest_row_boundary() {
        assert_eq!(slot_at(&rows(), 101.0), Some(0));
        assert_eq!(slot_at(&rows(), 130.0), Some(1));
        assert_eq!(slot_at(&rows(), 170.0), Some(2));
        assert_eq!(slot_at(&rows(), 250.0), Some(4));
        assert_eq!(slot_at(&rows(), 900.0), Some(4), "below the list: the end");
        assert_eq!(slot_at(&rows(), 40.0), None, "the chrome above the list");
        assert_eq!(slot_at(&[], 40.0), Some(0), "an empty space");
    }
}
