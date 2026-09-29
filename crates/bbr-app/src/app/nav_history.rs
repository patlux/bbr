//! Blueprint D14 (DESIGN.md §11.14/§11.13): the Back/Forward history menu.
//!
//! A secondary click on the titlebar Back or Forward control — or holding
//! the primary button on it for [`HOLD_MS`] — lists the focused session's
//! history on that side, nearest entry first, in the §11.13 menu anatomy.
//! The list is read from the engine's own back-forward list at open (never
//! a BBR mirror); a row jumps straight to its entry through the ordinary
//! navigation seam. The menu hangs below the titlebar over web content, so
//! it renders in the one native portal like the page menu, and it keeps
//! the page menu's dismissal and keyboard rules.
//!
//! With the hold gesture, Back and Forward activate on release (a click)
//! instead of on press, so a long press never also navigates one step.

use std::time::Duration;

use bbr_ui::surfaces::menu::{ContextMenuModel, ContextMenuRow};
use bbr_ui::titlebar::TitlebarControl;
use bbr_web::WebRect;
use gpui::{Context, Pixels, Point, SharedString, Window};

use super::BbrApp;
use crate::portal::geometry::Sz;
use crate::portal::state::{DismissalPolicy, PortalRequest};

/// The press-and-hold delay that opens the history menu (the macOS
/// long-press convention for Back/Forward).
pub const HOLD_MS: u64 = 500;
/// At most this many entries are listed on either side.
pub const NAV_HISTORY_LIMIT: usize = 12;
/// The menu's stable portal presentation id.
pub const NAV_HISTORY_PRESENTATION_ID: crate::portal::state::PresentationId =
    crate::portal::state::PresentationId(0xC7);
/// Stable §15 id of the menu's portal backdrop (outside-press dismissal).
pub const NAV_HISTORY_BACKDROP_ID: &str = "bbr-nav-history-backdrop";

/// Which side of the history the menu lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    Back,
    Forward,
}

impl NavDirection {
    fn from_control(control: TitlebarControl) -> Option<Self> {
        match control {
            TitlebarControl::Back => Some(Self::Back),
            TitlebarControl::Forward => Some(Self::Forward),
            _ => None,
        }
    }

    /// The history offset of the `index`-th listed entry (nearest first).
    pub fn offset(self, index: usize) -> i32 {
        let step = index as i32 + 1;
        match self {
            Self::Back => -step,
            Self::Forward => step,
        }
    }
}

/// One listed entry: the page title (the URL when the title is empty) and
/// the URL for the snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavHistoryRow {
    pub label: String,
    pub url: String,
}

/// The open menu (`None` fields while closed).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NavHistoryState {
    pub direction: Option<NavDirection>,
    pub rows: Vec<NavHistoryRow>,
    pub selected: Option<usize>,
    pub origin_chrome_focused: bool,
}

/// A primary press on Back/Forward in flight: its hold timer generation
/// and whether the hold already opened the menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavPress {
    pub control: TitlebarControl,
    pub point: Point<Pixels>,
    pub generation: u64,
    pub held: bool,
}

/// The menu rows for one side of a session history, nearest first.
pub fn history_rows(entries: &[bbr_web::HistoryEntry]) -> Vec<NavHistoryRow> {
    entries
        .iter()
        .take(NAV_HISTORY_LIMIT)
        .map(|entry| NavHistoryRow {
            label: if entry.title.trim().is_empty() {
                entry.url.clone()
            } else {
                entry.title.clone()
            },
            url: entry.url.clone(),
        })
        .collect()
}

/// Blueprint D10: the last row opens History.
pub const NAV_HISTORY_FULL_ID: &str = "bbr-nav-history-full";

/// The stable §15 id of the `index`-th row.
pub fn nav_history_row_id(index: usize) -> String {
    format!("bbr-nav-history-{index}")
}

impl BbrApp {
    /// Whether the history menu is open (and still owns the portal).
    pub fn nav_history_open(&self) -> bool {
        self.nav_history.direction.is_some()
            && self
                .portal
                .active()
                .is_some_and(|presentation| presentation.id == NAV_HISTORY_PRESENTATION_ID)
    }

    /// Open the history menu for `direction` at `point` (window content
    /// coordinates). A side with no entries opens nothing. Exactly one menu
    /// presentation per window.
    pub fn open_nav_history(
        &mut self,
        direction: NavDirection,
        point: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(history) = self.controller.session_history() else {
            return;
        };
        let entries = match direction {
            NavDirection::Back => &history.back,
            NavDirection::Forward => &history.forward,
        };
        let rows = history_rows(entries);
        if rows.is_empty() {
            return;
        }
        let origin_chrome_focused = self.chrome_focused(window);
        self.menu_discard(cx);
        self.tab_menu_discard(cx);
        self.find_discard(cx);
        self.dismiss_command_portal(cx);
        // The entries plus Show Full History.
        let count = rows.len() + 1;
        self.nav_history = NavHistoryState {
            direction: Some(direction),
            rows,
            selected: Some(0),
            origin_chrome_focused,
        };
        let viewport = Sz::new(
            f32::from(window.bounds().size.width),
            f32::from(window.bounds().size.height),
        );
        let anchor = WebRect::new(f32::from(point.x), f32::from(point.y), 0.0, 0.0);
        let request = PortalRequest::native(
            NAV_HISTORY_PRESENTATION_ID,
            super::context_menu::context_menu_placement(anchor, count, &self.theme()),
            super::context_menu::CONTEXT_MENU_PRIORITY,
            DismissalPolicy::any(),
        );
        self.portal_probe
            .borrow_mut()
            .present_nav_history(request, viewport);
        self.activate_portal_request(request, cx);
        window.focus(&self.menu_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        self.emit_focus(window);
        cx.notify();
    }

    /// The secondary-click path on a titlebar control (Back/Forward only).
    pub fn open_nav_history_from_control(
        &mut self,
        control: TitlebarControl,
        point: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(direction) = NavDirection::from_control(control) {
            self.open_nav_history(direction, point, window, cx);
        }
    }

    /// The presentation model the portal renders each frame.
    pub(crate) fn nav_history_model(&self) -> ContextMenuModel {
        ContextMenuModel {
            focused: self.nav_history_open(),
            selected: self.nav_history.selected,
            items: self
                .nav_history
                .rows
                .iter()
                .enumerate()
                .map(|(index, row)| ContextMenuRow {
                    id: SharedString::from(nav_history_row_id(index)),
                    title: SharedString::from(row.label.clone()),
                    enabled: true,
                    disabled_reason: None,
                })
                .chain(std::iter::once(ContextMenuRow {
                    id: SharedString::from(NAV_HISTORY_FULL_ID),
                    title: SharedString::from("Show Full History"),
                    enabled: true,
                    disabled_reason: None,
                }))
                .collect(),
        }
    }

    /// Pointer hover on a row (the shared keyboard/pointer cursor).
    pub fn nav_history_select(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.nav_history_open() && index <= self.nav_history.rows.len() {
            self.nav_history.selected = Some(index);
            cx.notify();
        }
    }

    /// Activate the row at `index`: jump straight to its entry, then close
    /// with the origin focus restored.
    pub fn nav_history_activate(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.nav_history_open() {
            return;
        }
        let Some(direction) = self.nav_history.direction else {
            return;
        };
        if index < self.nav_history.rows.len() {
            match self.controller.go_to_history(direction.offset(index)) {
                Ok(_) => {}
                Err(error) => log::error!("history jump failed: {error}"),
            }
        }
        let full = index == self.nav_history.rows.len();
        self.nav_history_close(window, cx);
        // Blueprint D10: Show Full History opens the History page.
        if full {
            self.open_history(window, cx);
        }
    }

    /// Escape / backdrop press / close seam: close and restore the focus
    /// domain captured at open.
    pub fn nav_history_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.nav_history.direction.is_none() {
            return;
        }
        let origin_chrome_focused = self.nav_history.origin_chrome_focused;
        let presented = self.nav_history_open();
        self.nav_history = NavHistoryState::default();
        if presented {
            self.portal_dismiss(cx);
        }
        if origin_chrome_focused {
            self.focus_command_bar(window, cx);
        } else {
            self.focus_web(window, cx);
        }
        cx.notify();
    }

    /// The menu's keyboard (the root routes here while it is open).
    pub(super) fn on_nav_history_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let count = self.nav_history.rows.len() + 1;
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                self.nav_history_close(window, cx);
            }
            "enter" => {
                cx.stop_propagation();
                let index = self.nav_history.selected.unwrap_or(0);
                self.nav_history_activate(index, window, cx);
            }
            "up" | "down" if count > 0 => {
                cx.stop_propagation();
                let current = self.nav_history.selected.unwrap_or(0) as isize;
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let next = (current + delta).rem_euclid(count as isize) as usize;
                self.nav_history.selected = Some(next);
                cx.notify();
            }
            _ => {}
        }
    }

    /// The Back/Forward primary press (the control's activation slot): arm
    /// the hold timer. The step itself waits for the release.
    pub fn begin_nav_press(
        &mut self,
        control: TitlebarControl,
        point: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if NavDirection::from_control(control).is_none() {
            return;
        }
        self.nav_press_generation += 1;
        let generation = self.nav_press_generation;
        self.nav_press = Some(NavPress {
            control,
            point,
            generation,
            held: false,
        });
        self.nav_hold_timer = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(HOLD_MS))
                .await;
            let _ = this.update_in(cx, |view, window, cx| {
                view.on_nav_hold(generation, window, cx);
            });
        }));
    }

    /// The hold elapsed while the same press is still down: open the menu
    /// (the release then does not navigate).
    fn on_nav_hold(&mut self, generation: u64, window: &mut Window, cx: &mut Context<Self>) {
        let Some(press) = self.nav_press.as_mut() else {
            return;
        };
        if press.generation != generation || press.held {
            return;
        }
        press.held = true;
        let (control, point) = (press.control, press.point);
        self.open_nav_history_from_control(control, point, window, cx);
    }

    /// The primary button went up anywhere (the root's release path): a
    /// short press released over its control steps Back/Forward once; a
    /// held press (the menu opened) or a release elsewhere does nothing.
    pub fn end_nav_press(&mut self, cx: &mut Context<Self>) {
        self.nav_hold_timer = None;
        let Some(press) = self.nav_press.take() else {
            return;
        };
        if press.held || self.titlebar_hovered != Some(press.control) {
            return;
        }
        match press.control {
            TitlebarControl::Back => self.go_back(),
            TitlebarControl::Forward => self.go_forward(),
            _ => {}
        }
        cx.notify();
    }

    /// The typed E2E snapshot of the menu.
    #[cfg(feature = "testkit")]
    pub fn nav_history_snapshot(&self) -> Option<bbr_testkit::protocol::NavHistorySnapshot> {
        if !self.nav_history_open() {
            return None;
        }
        let direction = self.nav_history.direction?;
        Some(bbr_testkit::protocol::NavHistorySnapshot {
            direction: match direction {
                NavDirection::Back => "back".into(),
                NavDirection::Forward => "forward".into(),
            },
            labels: self
                .nav_history
                .rows
                .iter()
                .map(|row| row.label.clone())
                .collect(),
            urls: self
                .nav_history
                .rows
                .iter()
                .map(|row| row.url.clone())
                .collect(),
            selected: self.nav_history.selected,
            show_full_history: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(url: &str, title: &str) -> bbr_web::HistoryEntry {
        bbr_web::HistoryEntry {
            url: url.into(),
            title: title.into(),
        }
    }

    #[test]
    fn rows_list_nearest_first_with_the_url_standing_in_for_a_missing_title() {
        let rows = history_rows(&[
            entry("https://b.example/", "B"),
            entry("https://a.example/", " "),
        ]);
        assert_eq!(rows[0].label, "B");
        assert_eq!(rows[1].label, "https://a.example/");
        assert_eq!(rows[1].url, "https://a.example/");
    }

    #[test]
    fn the_list_is_bounded() {
        let many: Vec<_> = (0..30)
            .map(|n| entry(&format!("https://{n}.example/"), ""))
            .collect();
        assert_eq!(history_rows(&many).len(), NAV_HISTORY_LIMIT);
    }

    #[test]
    fn offsets_count_away_from_the_current_entry() {
        assert_eq!(NavDirection::Back.offset(0), -1);
        assert_eq!(NavDirection::Back.offset(2), -3);
        assert_eq!(NavDirection::Forward.offset(0), 1);
        assert_eq!(nav_history_row_id(3), "bbr-nav-history-3");
    }
}
