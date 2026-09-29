//! Blueprint D23 (DESIGN §11.7, boards G-9/G-10): the shown split group in
//! the card.
//!
//! The group is projected onto the card's inner box with
//! `bbr_split::tiling`: one native slot per pane at its rect, a 2 pt
//! divider in each gap (`hairline` at rest, `accent` while dragged, the
//! resize cursor). Dragging a divider moves only its two neighbours, kept
//! at the 320 × 200 pt minimum; a double-click evens out its row or column.
//! A press on a pane (seen by the native pointer observer, the click still
//! reaches the page) focuses it; ⌃⌘+arrow focuses the neighbour. When the
//! card is too small for every pane the group rests on the focused pane —
//! the others are hidden natively, never destroyed — and the Portal shows
//! an "n of N" chip whose list switches panes.

use std::cell::Cell;

use bbr_browser::SplitLayout;
use bbr_domain::tiling::{Axis, Edge};
use bbr_split::tiling::{
    drag_weight, project, TiledDivider, Tiling, PANE_MIN_HEIGHT, PANE_MIN_WIDTH, TILING_DIVIDER,
};
use bbr_web::{WebRect, WebViewId};
use gpui::{div, prelude::*, px, AnyElement, Context, CursorStyle, MouseButton, Window};

use super::BbrApp;

/// The split view's app state.
#[derive(Default)]
pub(crate) struct SplitView {
    /// The divider being dragged (its projection when the drag began).
    pub(crate) drag: Option<TiledDivider>,
    /// The collapsed group's pane list is open.
    pub(crate) list_open: bool,
    /// Pane views hidden while the group rests on its focused pane.
    pub(crate) hidden: Vec<WebViewId>,
    /// The chip's rendered rect (window points) for the Portal regions.
    /// ⌃⌘S / ⌃⌥⌘S: the pane the palette's commit joins, and where.
    pub(crate) pending: Option<(bbr_domain::TabId, Edge)>,
    pub(crate) chip_bounds: std::rc::Rc<Cell<Option<[f32; 4]>>>,
    pub(crate) list_bounds: std::rc::Rc<Cell<Option<[f32; 4]>>>,
}

impl BbrApp {
    /// The card's inner box (window points), as the overlays know it.
    fn split_card(&self) -> Option<WebRect> {
        self.transient
            .card
            .map(|[x, y, w, h]| WebRect::new(x, y, w, h))
    }

    /// The shown group projected onto the card (window points).
    pub(crate) fn split_projection(&self) -> Option<(SplitLayout, Tiling)> {
        let layout = self.controller.split_layout()?;
        let card = self.split_card()?;
        let tiling = project(layout.group.root(), card, TILING_DIVIDER);
        Some((layout, tiling))
    }

    /// `(n, N)` while the shown group rests on its focused pane.
    pub(crate) fn split_collapsed(&self) -> Option<(usize, usize)> {
        let (layout, tiling) = self.split_projection()?;
        if tiling.fits() {
            return None;
        }
        let at = layout
            .views
            .iter()
            .position(|(_, view)| *view == layout.focused)?;
        Some((at + 1, layout.views.len()))
    }

    /// Hide the unfocused panes of a collapsed group natively (and show
    /// them again once it fits); runs before each frame's content.
    pub(crate) fn sync_split_collapse(&mut self, cx: &mut Context<Self>) {
        let wanted: Vec<WebViewId> = match (self.split_collapsed(), self.controller.split_layout())
        {
            (Some(_), Some(layout)) => layout
                .views
                .iter()
                .map(|(_, view)| *view)
                .filter(|view| *view != layout.focused)
                .collect(),
            _ => Vec::new(),
        };
        if wanted == self.split_view.hidden {
            return;
        }
        let mut engine = self.engine.borrow_mut();
        for view in &self.split_view.hidden {
            if !wanted.contains(view) {
                engine.show(*view, true);
            }
        }
        for view in &wanted {
            if !self.split_view.hidden.contains(view) {
                engine.show(*view, false);
            }
        }
        drop(engine);
        if wanted.is_empty() {
            self.split_view.list_open = false;
        }
        self.split_view.hidden = wanted;
        self.sync_portal_overlays(cx);
    }

    /// The card content for the shown group: every pane's slot at its
    /// projected rect and the dividers between them, or only the focused
    /// pane when the group rests collapsed. `None` without a group.
    pub(crate) fn split_content(
        &self,
        slot: &dyn Fn(&BbrApp, WebViewId, bool) -> AnyElement,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        let (layout, tiling) = self.split_projection()?;
        let card = self.split_card()?;
        if !tiling.fits() {
            return Some(slot(self, layout.focused, true));
        }
        let theme = self.theme();
        let local = |rect: WebRect| (rect.x - card.x, rect.y - card.y, rect.width, rect.height);
        let mut container = div().id("bbr-split").relative().size_full();
        for (index, pane) in tiling.panes.iter().enumerate() {
            let Some(view) = layout.view_of(pane.tab) else {
                continue;
            };
            let (x, y, w, h) = local(pane.rect);
            container = container.child(
                div()
                    .id(("bbr-pane", index))
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .flex()
                    .child(slot(self, view, view == layout.focused)),
            );
        }
        let dragging = self.split_view.drag.clone();
        for (index, divider) in tiling.dividers.iter().enumerate() {
            let (x, y, w, h) = local(divider.rect);
            let active = dragging
                .as_ref()
                .is_some_and(|d| d.path == divider.path && d.index == divider.index);
            let color = if active {
                theme.palette.accent
            } else {
                theme.palette.hairline
            };
            let grabbed = divider.clone();
            container = container.child(
                div()
                    .id(("bbr-split-divider", index))
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .bg(color)
                    .cursor(match divider.axis {
                        Axis::Row => CursorStyle::ResizeLeftRight,
                        Axis::Column => CursorStyle::ResizeUpDown,
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &gpui::MouseDownEvent, _w, cx| {
                            cx.stop_propagation();
                            if event.click_count >= 2 {
                                this.split_equalize(&grabbed.path, cx);
                            } else {
                                this.split_view.drag = Some(grabbed.clone());
                                cx.notify();
                            }
                        }),
                    ),
            );
        }
        Some(container.into_any_element())
    }

    /// Move the dragged divider to the pointer (window points).
    pub(crate) fn split_divider_to(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        let Some(divider) = self.split_view.drag.clone() else {
            return;
        };
        let (along, min) = match divider.axis {
            Axis::Row => (x, PANE_MIN_WIDTH),
            Axis::Column => (y, PANE_MIN_HEIGHT),
        };
        let leading = drag_weight(&divider, along, min, TILING_DIVIDER);
        if self
            .controller
            .split_set_divider(&divider.path, divider.index, leading)
            .is_ok()
        {
            cx.notify();
        }
    }

    pub(crate) fn on_split_divider_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.split_view.drag.is_some() {
            let position = event.position;
            self.split_divider_to(f32::from(position.x), f32::from(position.y), cx);
        }
    }

    pub(crate) fn on_split_divider_end(
        &mut self,
        _event: &gpui::MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.split_view.drag.take().is_some() {
            self.persist_structure();
            cx.notify();
        }
    }

    /// Even out the row or column at `path`.
    pub(crate) fn split_equalize(&mut self, path: &[usize], cx: &mut Context<Self>) {
        if self.controller.split_equalize(path).is_ok() {
            self.persist_structure();
            cx.notify();
        }
    }

    /// A press over the card (window points): the pane under it takes the
    /// focus. The press itself still reaches the page.
    pub(crate) fn split_pointer_down(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        let Some((layout, tiling)) = self.split_projection() else {
            return;
        };
        if !tiling.fits() {
            return;
        }
        let Some(pane) = tiling.panes.iter().find(|pane| {
            x >= pane.rect.x
                && x < pane.rect.x + pane.rect.width
                && y >= pane.rect.y
                && y < pane.rect.y + pane.rect.height
        }) else {
            return;
        };
        if layout.view_of(pane.tab) != Some(layout.focused) {
            self.split_focus_tab(pane.tab, cx);
        }
    }

    /// ⌃⌘+arrow: focus the pane next to the focused one.
    pub(crate) fn split_focus_neighbour(&mut self, edge: Edge, cx: &mut Context<Self>) -> bool {
        let Some((layout, tiling)) = self.split_projection() else {
            return false;
        };
        let Some(from) = layout.tab_of(layout.focused) else {
            return false;
        };
        match tiling.neighbour(from, edge) {
            Some(tab) => {
                self.split_focus_tab(tab, cx);
                true
            }
            None => false,
        }
    }

    /// The collapsed group's chip (and list) — `None` while it fits.
    pub(crate) fn split_chip_model(&self) -> Option<bbr_ui::surfaces::split_chip::SplitChipModel> {
        let (at, total) = self.split_collapsed()?;
        let list = self.split_view.list_open.then(|| {
            let layout = self.controller.split_layout();
            let tabs = self.controller.snapshot().all_tabs;
            layout
                .map(|layout| {
                    layout
                        .views
                        .iter()
                        .map(|(tab, _)| {
                            let title = tabs
                                .iter()
                                .find(|entry| entry.id == *tab)
                                .map(|entry| {
                                    if entry.title.is_empty() {
                                        entry.url.clone()
                                    } else {
                                        entry.title.clone()
                                    }
                                })
                                .unwrap_or_default();
                            gpui::SharedString::from(title)
                        })
                        .collect()
                })
                .unwrap_or_default()
        });
        Some(bbr_ui::surfaces::split_chip::SplitChipModel { at, total, list })
    }

    /// The chip toggles the list; a list row focuses its pane.
    pub(crate) fn split_chip_press(&mut self, row: Option<usize>, cx: &mut Context<Self>) {
        match row {
            None => self.split_view.list_open = !self.split_view.list_open,
            Some(index) => {
                self.split_view.list_open = false;
                let tab = self
                    .controller
                    .split_layout()
                    .and_then(|layout| layout.views.get(index).map(|(tab, _)| *tab));
                if let Some(tab) = tab {
                    self.split_focus_tab(tab, cx);
                }
            }
        }
        if !self.split_view.list_open {
            self.split_view.list_bounds.set(None);
        }
        self.sync_portal_overlays(cx);
        cx.notify();
    }

    /// The Portal regions of the chip and the list.
    pub(crate) fn split_hit_rects(&self) -> Vec<[f32; 4]> {
        if self.split_collapsed().is_none() {
            return Vec::new();
        }
        let mut rects: Vec<[f32; 4]> = self.split_view.chip_bounds.get().into_iter().collect();
        if self.split_view.list_open {
            rects.extend(self.split_view.list_bounds.get());
        }
        rects
    }

    /// ⌃⌘S (right) / ⌃⌥⌘S (below): the ⌘T palette opens in split mode;
    /// its commit becomes a pane at that edge of the focused tab.
    pub(crate) fn begin_split_palette(
        &mut self,
        edge: Edge,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.controller.snapshot();
        let Some(beside) = snapshot.active_index.and_then(|index| {
            snapshot
                .all_tabs
                .iter()
                .find(|tab| tab.space == snapshot.active_space && tab.index == index)
                .map(|tab| tab.id)
        }) else {
            return;
        };
        self.activate_new_tab(window, cx);
        // After the palette opened (its open clears a stale target).
        self.split_view.pending = Some((beside, edge));
    }

    /// The palette committed a tab (opened or selected): it joins the
    /// pending split.
    pub(crate) fn join_pending_split(&mut self, cx: &mut Context<Self>) {
        let Some((beside, edge)) = self.split_view.pending.take() else {
            return;
        };
        let snapshot = self.controller.snapshot();
        let index_of = |tab: bbr_domain::TabId| {
            snapshot
                .all_tabs
                .iter()
                .find(|entry| entry.id == tab && entry.space == snapshot.active_space)
                .map(|entry| entry.index)
        };
        let (Some(beside_index), Some(new_index)) = (index_of(beside), snapshot.active_index)
        else {
            return;
        };
        if beside_index == new_index {
            return;
        }
        match self.controller.split_insert(beside_index, new_index, edge) {
            Ok(event) => {
                self.emit_browser(&event);
                self.persist_structure();
                cx.notify();
            }
            Err(error) => log::warn!("the committed tab could not join the split: {error}"),
        }
    }

    /// ⇧⌃⌘S: dissolve the shown group (every tab stays open).
    pub(crate) fn dissolve_shown_split(&mut self, cx: &mut Context<Self>) -> bool {
        if self.controller.split_layout().is_none() {
            return false;
        }
        match self.controller.close_split() {
            Ok(event) => {
                self.emit_browser(&event);
                self.persist_structure();
                cx.notify();
                true
            }
            Err(_) => false,
        }
    }

    /// Focus the pane of `tab` (select it in the space).
    pub(crate) fn split_focus_tab(&mut self, tab: bbr_domain::TabId, cx: &mut Context<Self>) {
        let index = self
            .controller
            .snapshot()
            .all_tabs
            .iter()
            .find(|entry| entry.id == tab)
            .map(|entry| entry.index);
        if let Some(index) = index {
            self.select_tab(index, cx);
        }
    }
}
