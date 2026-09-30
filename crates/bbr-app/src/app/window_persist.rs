//! Blueprint D19 (several windows, task packet `multi-window.md`): what the
//! browser windows persist. Each window publishes its presentation row
//! (content rect and rail) to the shared window registry when it draws, so
//! any window's save writes every window's row — no window overwrites
//! another's. The windows' active spaces and selections go to `windows.txt`
//! (see `crate::window_layouts`).

use bbr_persistence::{SidebarState, WindowGeometry, WindowState};

use super::BbrApp;
use crate::browser_windows::WindowRow;
use crate::window_layouts::WindowLayout;

/// `windows.txt` and the windows last written there (shared by every
/// window).
#[derive(Default)]
pub(crate) struct WindowLayoutsFile {
    path: Option<std::path::PathBuf>,
    saved: Option<Vec<WindowLayout>>,
}

impl BbrApp {
    /// This window's presentation row as it stands now.
    pub(crate) fn window_row(&self) -> WindowRow {
        WindowRow {
            geometry: *self.window_geometry.borrow(),
            sidebar: SidebarState {
                width: f64::from(self.sidebar.requested()),
                collapsed: !self.sidebar_visible,
                last_expanded_width: f64::from(self.sidebar.last_expanded()),
            },
        }
    }

    /// Publish this window's row to the registry (bookkeeping: the other
    /// windows draw none of it).
    pub(crate) fn publish_window_row(&self) {
        let key = self.window_key();
        let row = self.window_row();
        if self.windows.read().row(key) != Some(row) {
            self.windows.bookkeep().set_row(key, row);
        }
    }

    /// Publish this window's frame (screen points) for the testkit.
    pub(crate) fn publish_window_frame(&self, window: &gpui::Window) {
        let bounds = window.bounds();
        let frame = [
            f32::from(bounds.origin.x),
            f32::from(bounds.origin.y),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
        ];
        let key = self.window_key();
        if self.windows.read().frame(key) != Some(frame) {
            self.windows.bookkeep().set_frame(key, frame);
        }
    }

    /// Every open window's store row in window order, ids from `first_id`
    /// upward (above every domain id). This window's row is taken as it
    /// stands; the others as they last drew.
    pub(crate) fn persisted_windows(&self, first_id: u64) -> Vec<WindowState> {
        self.publish_window_row();
        let mut rows = self.windows.read().rows();
        if rows.is_empty() {
            rows.push(self.window_row());
        }
        rows.into_iter()
            .enumerate()
            .map(|(index, row)| WindowState {
                id: first_id + index as u64,
                geometry: row.geometry.map(|[x, y, width, height]| WindowGeometry {
                    x,
                    y,
                    width,
                    height,
                }),
                sidebar: row.sidebar,
            })
            .collect()
    }

    /// Where the windows' spaces and selections are kept; the file as it
    /// stands counts as written.
    pub(crate) fn set_window_layouts_path(&mut self, path: std::path::PathBuf) {
        let layouts = self.window_layouts_now();
        let mut file = self.window_layouts.get();
        file.path = Some(path);
        file.saved = Some(layouts);
    }

    /// Every open window's active space and selections, in window order.
    fn window_layouts_now(&self) -> Vec<WindowLayout> {
        let open = self.windows.read().keys();
        self.controller
            .read()
            .window_layouts()
            .into_iter()
            .filter(|(key, _, _)| open.is_empty() || open.contains(key))
            .map(|(_, active_space, selections)| WindowLayout {
                active_space,
                selections,
            })
            .collect()
    }

    /// Write `windows.txt` when the windows changed since the last write.
    pub(crate) fn persist_window_layouts(&self) {
        let layouts = self.window_layouts_now();
        let path = {
            let file = self.window_layouts.read();
            if file.saved.as_ref() == Some(&layouts) {
                return;
            }
            let Some(path) = file.path.clone() else {
                return;
            };
            path
        };
        if let Err(error) = crate::window_layouts::save(&path, &layouts) {
            log::warn!("window layouts not saved: {error}");
            return;
        }
        self.window_layouts.bookkeep().saved = Some(layouts);
    }
}
