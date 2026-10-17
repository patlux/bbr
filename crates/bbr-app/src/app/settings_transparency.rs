//! Settings › Appearance › Window › Sidebar transparency: the row, the
//! slider's press/drag value and its arrow-key steps. The one preference is
//! shared, so a change re-presents every browser window at once.

use bbr_ui::surfaces::settings::{SettingsControl, SettingsRow, SettingsSection};
use gpui::{Context, SharedString};

use super::BbrApp;
use crate::preferences::DEFAULT_CHROME_TRANSPARENCY;

/// The Settings transparency row and its keyboard step.
pub(crate) const TRANSPARENCY_ROW: &str = "appearance-transparency";
const TRANSPARENCY_STEP: u8 = 5;

impl BbrApp {
    /// The "Window" section with the slider and a detail naming what wins.
    pub(super) fn transparency_section(&self) -> SettingsSection {
        let transparency = self.preferences.read().chrome_transparency;
        let system = self.effective_accessibility();
        let detail = if system.reduce_transparency {
            "Off while Reduce transparency is on in macOS"
        } else if system.increase_contrast {
            "Off while Increase contrast is on"
        } else if transparency == 0 {
            "Opaque sidebar and window frame"
        } else if transparency > DEFAULT_CHROME_TRANSPARENCY {
            "More desktop shows through; text contrast depends on the wallpaper"
        } else {
            "The desktop shows through the sidebar and window frame"
        };
        let focused = self.settings.read().active_slider.as_deref() == Some(TRANSPARENCY_ROW);
        SettingsSection {
            title: Some("Window".into()),
            rows: vec![SettingsRow::new(
                TRANSPARENCY_ROW,
                "Sidebar transparency",
                SettingsControl::Slider {
                    value: transparency,
                    step: TRANSPARENCY_STEP,
                    label: SharedString::from(format!("{transparency}%")),
                    focused,
                },
            )
            .detail(detail)],
        }
    }

    /// A slider value from a press, a drag or a key. Unchanged values write
    /// nothing, so a drag does not rewrite the file on every pointer move.
    pub(super) fn set_slider(&mut self, row: &str, value: u8, cx: &mut Context<Self>) {
        if row == TRANSPARENCY_ROW {
            let value = value.min(100);
            if self.preferences.read().chrome_transparency != value {
                self.update_preferences(|p| p.chrome_transparency = value, cx);
            }
        }
    }

    /// The arrow keys step the active slider; Home/End jump to its ends.
    pub(super) fn step_active_slider(&mut self, key: &str, cx: &mut Context<Self>) {
        let Some(row) = self.settings.read().active_slider.clone() else {
            return;
        };
        if row.as_ref() != TRANSPARENCY_ROW {
            return;
        }
        let current = self.preferences.read().chrome_transparency;
        let value = match key {
            "left" | "down" => current.saturating_sub(TRANSPARENCY_STEP),
            "right" | "up" => current.saturating_add(TRANSPARENCY_STEP).min(100),
            "home" => 0,
            "end" => 100,
            _ => return,
        };
        self.set_slider(&row, value, cx);
    }
}
