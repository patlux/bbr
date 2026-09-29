//! Blueprint D16 (boards L-3…L-9): the Settings window's app side — the
//! plain model built from the preferences and browser state, and every row
//! action applied at once (no Save button, macOS convention).
//!
//! Rows appear only where BBR has the behaviour behind them; board rows
//! without it (startup behaviour, link routing target, tab suspension and
//! auto-close, search suggestions and keywords, cookie policy, per-download
//! location prompts, history retention) are recorded in the packet rather
//! than drawn as controls that would do nothing. Profiles and Privacy rows
//! grow with D06, D10 and D11.

use bbr_command::search_settings::engine::SearchEngine;
use bbr_ui::surfaces::settings::{
    SettingsControl, SettingsEvent, SettingsModel, SettingsPane, SettingsRow, SettingsSection,
};
use gpui::{Context, SharedString, Window};

use super::BbrApp;
use crate::preferences::{AppearanceChoice, DensityChoice, Follow, PaperChoice, Preferences};

/// The Settings window's app-owned state.
#[derive(Default)]
pub(crate) struct SettingsState {
    pub pane: SettingsPane,
    pub open_select: Option<SharedString>,
    pub window: Option<gpui::WindowHandle<crate::settings_window::SettingsView>>,
}

/// The curated engines in Settings order.
const ENGINES: [SearchEngine; 5] = [
    SearchEngine::DuckDuckGo,
    SearchEngine::Google,
    SearchEngine::Bing,
    SearchEngine::Ecosia,
    SearchEngine::Kagi,
];

const FOLLOW: [Follow; 3] = [Follow::System, Follow::On, Follow::Off];

/// The shortcut list: `(group, title, keystroke)`, the keystroke exactly as
/// bound (a unit test checks every one against the keymap).
pub(crate) const SHORTCUTS: [(&str, &str, &str); 17] = [
    ("Tabs", "New tab", "cmd-t"),
    ("Tabs", "Close tab", "cmd-w"),
    ("Tabs", "Reopen closed tab", "cmd-shift-t"),
    ("Tabs", "Pin / unpin", "cmd-shift-p"),
    ("Spaces", "New space", "cmd-shift-n"),
    ("Spaces", "Move tab to space", "cmd-shift-m"),
    ("Navigation", "Edit address", "cmd-l"),
    ("Navigation", "Find in page", "cmd-f"),
    ("Navigation", "Back", "cmd-["),
    ("Navigation", "Forward", "cmd-]"),
    ("Navigation", "Reload", "cmd-r"),
    ("Sidebar", "Toggle sidebar", "cmd-s"),
    ("Sidebar", "Focus sidebar", "alt-cmd-s"),
    ("Library", "Save page to Library", "cmd-d"),
    ("Library", "Saved Pages Library", "cmd-shift-l"),
    ("Zoom", "Actual size", "cmd-0"),
    ("App", "Settings", "cmd-,"),
];

/// The keycaps of a bound keystroke, in the macOS modifier order.
pub(crate) fn keycaps(keystroke: &str) -> Vec<SharedString> {
    let parts: Vec<&str> = keystroke.split('-').collect();
    let (key, modifiers) = match parts.split_last() {
        Some((key, modifiers)) if !key.is_empty() => (*key, modifiers.to_vec()),
        // `cmd--` style: the key itself is "-".
        _ => ("-", parts[..parts.len().saturating_sub(2)].to_vec()),
    };
    let mut caps = Vec::new();
    for (name, glyph) in [("ctrl", "⌃"), ("alt", "⌥"), ("shift", "⇧"), ("cmd", "⌘")] {
        if modifiers.contains(&name) {
            caps.push(SharedString::from(glyph));
        }
    }
    caps.push(SharedString::from(key.to_uppercase()));
    caps
}

fn options(labels: &[&str]) -> Vec<SharedString> {
    labels
        .iter()
        .map(|label| SharedString::from(label.to_string()))
        .collect()
}

fn section(title: &str, rows: Vec<SettingsRow>) -> SettingsSection {
    SettingsSection {
        title: Some(SharedString::from(title.to_string())),
        rows,
    }
}

impl BbrApp {
    /// Load the preferences file (defaults when missing).
    pub(crate) fn load_preferences(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        self.preferences = crate::preferences::load(&path);
        self.preferences_path = Some(path);
        crate::preferences::set_peek_new_windows(self.preferences.peek_new_windows);
        cx.notify();
    }

    /// Change the preferences, write them, and re-present every window.
    pub(crate) fn update_preferences(
        &mut self,
        change: impl FnOnce(&mut Preferences),
        cx: &mut Context<Self>,
    ) {
        change(&mut self.preferences);
        crate::preferences::set_peek_new_windows(self.preferences.peek_new_windows);
        if let Some(path) = self.preferences_path.as_ref() {
            if let Err(error) = crate::preferences::save(path, &self.preferences) {
                log::warn!("could not write preferences: {error}");
            }
        }
        cx.notify();
    }

    /// ⌘, and Settings…: open the Settings window (or bring it forward).
    /// Deferred: the new window's first render reads this entity.
    pub fn show_settings(&mut self, cx: &mut Context<Self>) {
        let app = cx.entity();
        let existing = self.settings.window;
        let visible = !self.windows_hidden;
        cx.defer(move |cx| {
            let handle = crate::settings_window::open(existing, app.clone(), visible, cx);
            app.update(cx, |view, cx| {
                view.settings.window = handle;
                cx.notify();
            });
        });
    }

    /// The Settings window closed.
    pub(crate) fn settings_closed(&mut self) {
        self.settings.window = None;
        self.settings.open_select = None;
    }

    /// The Settings model for the current pane.
    pub(crate) fn settings_model(&self) -> SettingsModel {
        let sections = match self.settings.pane {
            SettingsPane::General => self.general_sections(),
            SettingsPane::Profiles => self.profile_sections(),
            SettingsPane::Search => self.search_sections(),
            SettingsPane::Privacy => self.privacy_sections(),
            SettingsPane::Downloads => self.download_sections(),
            SettingsPane::Appearance => self.appearance_sections(),
            SettingsPane::Shortcuts => shortcut_sections(),
        };
        SettingsModel {
            theme: self.theme(),
            pane: self.settings.pane,
            sections,
            open_select: self.settings.open_select.clone(),
        }
    }

    fn general_sections(&self) -> Vec<SettingsSection> {
        #[cfg(target_os = "macos")]
        let default = bbr_web_macos::is_default_browser();
        #[cfg(not(target_os = "macos"))]
        let default = false;
        let browser = if default {
            SettingsRow::new(
                "general-default-browser",
                "Default browser",
                SettingsControl::Badge("Default".into()),
            )
            .detail("BBR is your default browser")
        } else {
            SettingsRow::new(
                "general-default-browser",
                "Default browser",
                SettingsControl::Button {
                    label: "Make Default".into(),
                    destructive: false,
                },
            )
            .detail("Links from other apps open in another browser")
        };
        vec![
            section("Browser", vec![browser]),
            section(
                "Tabs",
                vec![SettingsRow::new(
                    "general-peek",
                    "Open links that ask for a new window as Peek",
                    SettingsControl::Toggle(self.preferences.peek_new_windows),
                )
                .detail("Preview over the current tab; promote with Open as Tab")],
            ),
        ]
    }

    fn profile_sections(&self) -> Vec<SettingsSection> {
        let browser = self.controller.snapshot();
        let theme = self.theme();
        let rows = browser
            .profiles
            .iter()
            .map(|profile| {
                let spaces = browser
                    .spaces
                    .iter()
                    .filter(|space| space.profile == profile.id)
                    .count();
                let letter = crate::shell::space::profile_monogram(&profile.title)
                    .chars()
                    .next()
                    .unwrap_or('P');
                SettingsRow::new(
                    &format!("profile-{}", profile.index),
                    profile.title.clone(),
                    SettingsControl::Chip {
                        letter,
                        hue: bbr_ui::palette::ProfileHueRamp::tile(profile.hue, theme.mode).tile,
                    },
                )
                .detail(if spaces == 1 {
                    "1 space".to_string()
                } else {
                    format!("{spaces} spaces")
                })
            })
            .collect();
        let active = browser
            .profiles
            .iter()
            .find(|profile| profile.id == browser.active_profile);
        let spaces = browser
            .spaces
            .iter()
            .filter(|space| space.profile == browser.active_profile)
            .map(|space| {
                let paper = self
                    .controller
                    .space_appearance(space.id)
                    .map(|appearance| appearance.paper.intensity)
                    .unwrap_or_default();
                SettingsRow::new(
                    &format!("space-{}", space.id.0),
                    space.title.clone(),
                    SettingsControl::None,
                )
                .detail(paper.label())
            })
            .collect();
        let mut sections = vec![section("Profiles", rows)];
        if let Some(active) = active {
            sections.push(section(&format!("Spaces in {}", active.title), spaces));
        }
        sections
    }

    fn search_sections(&self) -> Vec<SettingsSection> {
        let engine = self.store.effective_settings().engine();
        let mut labels: Vec<SharedString> = ENGINES
            .iter()
            .map(|engine| SharedString::from(engine.display_name()))
            .collect();
        let value = match ENGINES.iter().position(|known| known.id() == engine.id()) {
            Some(index) => index,
            None => {
                labels.push(SharedString::from(engine.display_name()));
                labels.len() - 1
            }
        };
        vec![section(
            "Default",
            vec![
                SettingsRow::new(
                    "search-engine",
                    "Search engine",
                    SettingsControl::Select {
                        value,
                        options: labels,
                    },
                ),
                SettingsRow::new(
                    "search-all-spaces",
                    "Show tabs from all spaces in ⌘T",
                    SettingsControl::Toggle(self.preferences.palette_all_spaces),
                ),
            ],
        )]
    }

    fn privacy_sections(&self) -> Vec<SettingsSection> {
        Vec::new()
    }

    fn download_sections(&self) -> Vec<SettingsSection> {
        let history = self.downloads.controller.history_ids().len();
        vec![section(
            "List",
            vec![SettingsRow::new(
                "downloads-clear-history",
                "Download history",
                SettingsControl::Button {
                    label: "Clear".into(),
                    destructive: false,
                },
            )
            .detail(match history {
                0 => "No finished downloads".to_string(),
                1 => "1 finished download".to_string(),
                count => format!("{count} finished downloads"),
            })],
        )]
    }

    fn appearance_sections(&self) -> Vec<SettingsSection> {
        let prefs = &self.preferences;
        let theme = match prefs.appearance {
            AppearanceChoice::System => 0,
            AppearanceChoice::Light => 1,
            AppearanceChoice::Dark => 2,
        };
        let follow = |value: Follow| FOLLOW.iter().position(|f| *f == value).unwrap_or(0);
        let density = match prefs.density {
            DensityChoice::Automatic => None,
            DensityChoice::Small => Some(0),
            DensityChoice::Normal => Some(1),
            DensityChoice::Large => Some(2),
        };
        let paper = match prefs.default_paper {
            PaperChoice::Subtle => 0,
            PaperChoice::Standard => 1,
            PaperChoice::Rich => 2,
        };
        vec![
            section(
                "Theme",
                vec![
                    SettingsRow::new(
                        "appearance-theme",
                        "Appearance",
                        SettingsControl::Segmented {
                            value: theme,
                            options: options(&["System", "Light", "Dark"]),
                        },
                    )
                    .detail("Follows macOS unless you choose"),
                    SettingsRow::new(
                        "appearance-contrast",
                        "Increase contrast",
                        SettingsControl::Select {
                            value: follow(prefs.increase_contrast),
                            options: options(&["Follow system", "On", "Off"]),
                        },
                    )
                    .detail("Follows the macOS accessibility setting"),
                    SettingsRow::new(
                        "appearance-motion",
                        "Reduce motion",
                        SettingsControl::Select {
                            value: follow(prefs.reduce_motion),
                            options: options(&["Follow system", "On", "Off"]),
                        },
                    )
                    .detail("Follows the macOS accessibility setting"),
                ],
            ),
            section(
                "Density",
                vec![SettingsRow::new(
                    "appearance-density",
                    "Sidebar and text size",
                    SettingsControl::Density(density),
                )
                .detail(if density.is_none() {
                    "Follows the number of tabs; pick a size to keep it"
                } else {
                    "Rows, type and the rail width step together"
                })],
            ),
            section(
                "New spaces",
                vec![SettingsRow::new(
                    "appearance-paper",
                    "Default paper",
                    SettingsControl::Segmented {
                        value: paper,
                        options: options(&["Subtle", "Standard", "Rich"]),
                    },
                )],
            ),
        ]
    }

    /// Apply one Settings interaction.
    pub fn on_settings_event(
        &mut self,
        event: SettingsEvent,
        _window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        match event {
            SettingsEvent::Pane(pane) => {
                self.settings.pane = pane;
                self.settings.open_select = None;
            }
            SettingsEvent::OpenSelect(row) => {
                self.settings.open_select = if self.settings.open_select.as_ref() == Some(&row) {
                    None
                } else {
                    Some(row)
                };
            }
            SettingsEvent::Dismiss => self.settings.open_select = None,
            SettingsEvent::Toggle(row) => match row.as_ref() {
                "general-peek" => {
                    self.update_preferences(|p| p.peek_new_windows = !p.peek_new_windows, cx)
                }
                "search-all-spaces" => {
                    self.update_preferences(|p| p.palette_all_spaces = !p.palette_all_spaces, cx)
                }
                _ => {}
            },
            SettingsEvent::Choose(row, index) => {
                self.settings.open_select = None;
                self.settings_choose(row.as_ref(), index, cx);
            }
            SettingsEvent::Press(row) => match row.as_ref() {
                "general-default-browser" => self.request_default_browser(),
                "downloads-clear-history" => {
                    crate::app::downloads_history::downloads_clear(self, cx)
                }
                _ => {}
            },
        }
        cx.notify();
    }

    fn settings_choose(&mut self, row: &str, index: usize, cx: &mut Context<Self>) {
        match row {
            "search-engine" => {
                if let Some(engine) = ENGINES.get(index) {
                    self.store.set_search_engine(engine.clone());
                }
            }
            "appearance-theme" => {
                let choice = [
                    AppearanceChoice::System,
                    AppearanceChoice::Light,
                    AppearanceChoice::Dark,
                ];
                if let Some(choice) = choice.get(index).copied() {
                    self.update_preferences(|p| p.appearance = choice, cx);
                }
            }
            "appearance-contrast" => {
                if let Some(follow) = FOLLOW.get(index).copied() {
                    self.update_preferences(|p| p.increase_contrast = follow, cx);
                }
            }
            "appearance-motion" => {
                if let Some(follow) = FOLLOW.get(index).copied() {
                    self.update_preferences(|p| p.reduce_motion = follow, cx);
                }
            }
            "appearance-density" => {
                let pinned = [
                    DensityChoice::Small,
                    DensityChoice::Normal,
                    DensityChoice::Large,
                ];
                if let Some(choice) = pinned.get(index).copied() {
                    // Choosing the pinned size again returns to automatic.
                    self.update_preferences(
                        |p| {
                            p.density = if p.density == choice {
                                DensityChoice::Automatic
                            } else {
                                choice
                            }
                        },
                        cx,
                    );
                }
            }
            "appearance-paper" => {
                let choice = [
                    PaperChoice::Subtle,
                    PaperChoice::Standard,
                    PaperChoice::Rich,
                ];
                if let Some(choice) = choice.get(index).copied() {
                    self.update_preferences(|p| p.default_paper = choice, cx);
                }
            }
            _ => {}
        }
    }
}

/// The testkit view of the open Settings window.
#[cfg(feature = "testkit")]
impl BbrApp {
    pub(crate) fn settings_snapshot(&self) -> Option<bbr_testkit::protocol::SettingsSnapshot> {
        self.settings.window?;
        let model = self.settings_model();
        let rows = model
            .sections
            .iter()
            .flat_map(|section| section.rows.iter())
            .map(|row| bbr_testkit::protocol::SettingsRowSnapshot {
                id: row.id.to_string(),
                title: row.title.to_string(),
                detail: row.detail.as_ref().map(|detail| detail.to_string()),
                value: control_value(&row.control),
            })
            .collect();
        Some(bbr_testkit::protocol::SettingsSnapshot {
            pane: model.pane.key().to_string(),
            rows,
            open_select: model.open_select.map(|row| row.to_string()),
        })
    }

    /// One testkit Settings input through the controls' method.
    pub(crate) fn settings_input(
        &mut self,
        input: bbr_testkit::protocol::SettingsInput,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        use bbr_testkit::protocol::SettingsInput;
        let event = match input {
            SettingsInput::Pane { pane } => match SettingsPane::parse(&pane) {
                Some(pane) => SettingsEvent::Pane(pane),
                None => return false,
            },
            SettingsInput::Toggle { row } => SettingsEvent::Toggle(row.into()),
            SettingsInput::Choose { row, index } => SettingsEvent::Choose(row.into(), index),
            SettingsInput::Press { row } => SettingsEvent::Press(row.into()),
            SettingsInput::OpenSelect { row } => SettingsEvent::OpenSelect(row.into()),
            SettingsInput::Dismiss => SettingsEvent::Dismiss,
        };
        self.on_settings_event(event, Some(window), cx);
        true
    }
}

/// What a control shows, on the wire.
#[cfg(feature = "testkit")]
fn control_value(control: &SettingsControl) -> String {
    match control {
        SettingsControl::None => String::new(),
        SettingsControl::Badge(text) => text.to_string(),
        SettingsControl::Toggle(on) => if *on { "on" } else { "off" }.to_string(),
        SettingsControl::Select { value, options }
        | SettingsControl::Segmented { value, options } => options
            .get(*value)
            .map(|o| o.to_string())
            .unwrap_or_default(),
        SettingsControl::Button { label, .. } | SettingsControl::Link(label) => label.to_string(),
        SettingsControl::Keys(keys) => keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(" "),
        SettingsControl::Density(value) => match value {
            None => "automatic".to_string(),
            Some(0) => "small".to_string(),
            Some(1) => "normal".to_string(),
            Some(_) => "large".to_string(),
        },
        SettingsControl::Chip { letter, .. } => letter.to_string(),
        SettingsControl::ChipButton { letter, label, .. } => format!("{letter} {label}"),
    }
}

fn shortcut_sections() -> Vec<SettingsSection> {
    let mut sections: Vec<SettingsSection> = Vec::new();
    for (group, title, keystroke) in SHORTCUTS {
        let row = SettingsRow::new(
            &format!("shortcut-{keystroke}"),
            title,
            SettingsControl::Keys(keycaps(keystroke)),
        );
        match sections.last_mut() {
            Some(last) if last.title.as_deref() == Some(group) => last.rows.push(row),
            _ => sections.push(section(group, vec![row])),
        }
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycaps_follow_the_macos_modifier_order() {
        let caps = |keystroke| {
            keycaps(keystroke)
                .iter()
                .map(|cap| cap.to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(caps("cmd-shift-t"), ["⇧", "⌘", "T"]);
        assert_eq!(caps("alt-cmd-s"), ["⌥", "⌘", "S"]);
        assert_eq!(caps("cmd-["), ["⌘", "["]);
        assert_eq!(caps("cmd-,"), ["⌘", ","]);
    }

    #[test]
    fn every_listed_shortcut_is_bound() {
        let bound: Vec<gpui::Keystroke> = crate::window_seam::keybindings()
            .iter()
            .flat_map(|binding| binding.keystrokes().iter().map(|k| k.inner().clone()))
            .collect();
        for (_, title, keystroke) in SHORTCUTS {
            let parsed = gpui::Keystroke::parse(keystroke).expect("parses");
            assert!(
                bound
                    .iter()
                    .any(|b| b.key == parsed.key && b.modifiers == parsed.modifiers),
                "{title}: {keystroke} is not bound"
            );
        }
    }
}
