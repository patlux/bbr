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
    /// Blueprint D06: the profile Settings › Profiles refers to.
    pub selected_profile: Option<bbr_web::ProfileId>,
    /// Blueprint D06: the open profile sheet.
    pub sheet: Option<super::settings_profiles::ProfileSheet>,
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
        self.preferences.set(crate::preferences::load(&path));
        self.preferences_path = Some(path);
        crate::preferences::set_peek_new_windows(self.preferences.read().peek_new_windows);
        cx.notify();
    }

    /// Change the preferences, write them, and re-present every window.
    pub(crate) fn update_preferences(
        &mut self,
        change: impl FnOnce(&mut Preferences),
        cx: &mut Context<Self>,
    ) {
        change(&mut self.preferences.get());
        crate::preferences::set_peek_new_windows(self.preferences.read().peek_new_windows);
        if let Some(path) = self.preferences_path.as_ref() {
            let saved = crate::preferences::save(path, &self.preferences.read());
            if let Err(error) = saved {
                log::warn!("could not write preferences: {error}");
            }
        }
        cx.notify();
    }

    /// ⌘, and Settings…: open the Settings window (or bring it forward).
    /// Deferred: the new window's first render reads this entity.
    pub fn show_settings(&mut self, cx: &mut Context<Self>) {
        let app = cx.entity();
        let existing = self.settings.read().window;
        let visible = !self.windows_hidden;
        cx.defer(move |cx| {
            let handle = crate::settings_window::open(existing, app.clone(), visible, cx);
            app.update(cx, |view, cx| {
                view.settings.get().window = handle;
                cx.notify();
            });
        });
    }

    /// The Settings window closed.
    pub(crate) fn settings_closed(&mut self) {
        self.settings.get().window = None;
        self.settings.get().open_select = None;
        self.settings.get().sheet = None;
    }

    /// The Settings model for the current pane.
    pub(crate) fn settings_model(&self) -> SettingsModel {
        let pane = self.settings.read().pane;
        let sections = match pane {
            SettingsPane::General => self.general_sections(),
            SettingsPane::Profiles => self.profile_sections(),
            SettingsPane::Search => self.search_sections(),
            SettingsPane::Privacy => self.privacy_sections(),
            SettingsPane::Downloads => self.download_sections(),
            SettingsPane::Appearance => self.appearance_sections(),
            SettingsPane::Shortcuts => super::settings_shortcuts::shortcut_sections(),
        };
        SettingsModel {
            theme: self.theme(),
            pane: self.settings.read().pane,
            sections,
            open_select: self.settings.read().open_select.clone(),
            sheet: self.profile_sheet_model(),
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
                vec![
                    SettingsRow::new(
                        "general-peek",
                        "Open links that ask for a new window as Peek",
                        SettingsControl::Toggle(self.preferences.read().peek_new_windows),
                    )
                    .detail("Preview over the current tab; promote with Open as Tab"),
                    // Blueprint D13 (answer 6): links from other apps.
                    SettingsRow::new(
                        "general-external-links",
                        "Open links from other apps in",
                        SettingsControl::Select {
                            value: usize::from(self.preferences.read().external_links_ask),
                            options: vec!["Last active space".into(), "Ask each time".into()],
                        },
                    ),
                ],
            ),
        ]
    }

    fn search_sections(&self) -> Vec<SettingsSection> {
        let engine = self.store.read().effective_settings().engine().clone();
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
                    SettingsControl::Toggle(self.preferences.read().palette_all_spaces),
                ),
            ],
        )]
    }

    fn privacy_sections(&self) -> Vec<SettingsSection> {
        // Blueprint D10: Clear Browsing Data opens its sheet over History in
        // the browser window.
        let mut sections = vec![section(
            "Browsing data",
            vec![SettingsRow::new(
                "privacy-clear-data",
                "Clear browsing data",
                SettingsControl::Button {
                    label: "Clear…".into(),
                    destructive: false,
                },
            )
            .detail("History, cookies and site data, cached files")],
        )];
        // Blueprint D11 (board L-6): the active profile's sites with a
        // remembered permission; Forget returns them to Ask.
        let profile = self.controller.read().active_profile();
        let policies = self.permissions.policies.read();
        let rows: Vec<SettingsRow> = policies
            .sites(profile)
            .iter()
            .enumerate()
            .map(|(index, origin)| {
                let detail = crate::site_permissions::DECIDABLE
                    .iter()
                    .filter_map(|capability| {
                        let policy = policies.policy(profile, origin, *capability);
                        (policy != crate::site_permissions::SitePolicy::Ask).then(|| {
                            format!(
                                "{}: {}",
                                crate::site_permissions::capability_label(*capability),
                                policy.label()
                            )
                        })
                    })
                    .chain(
                        policies
                            .popups_allowed(profile, origin)
                            .then(|| "Pop-ups: Allow".to_string()),
                    )
                    .chain(
                        policies
                            .allowed_apps(profile, origin)
                            .into_iter()
                            .map(|scheme| format!("Opens {scheme}: links")),
                    )
                    .collect::<Vec<_>>()
                    .join(" · ");
                SettingsRow::new(
                    &format!("privacy-site-{index}"),
                    crate::site_permissions::origin_host(origin).to_string(),
                    SettingsControl::Button {
                        label: "Forget".into(),
                        destructive: false,
                    },
                )
                .detail(detail)
            })
            .collect();
        // Blueprint D12: remembered client-certificate choices.
        let certificates: Vec<SettingsRow> = self
            .certificates
            .choices
            .read()
            .hosts(profile)
            .into_iter()
            .enumerate()
            .map(|(index, (host, choice))| {
                SettingsRow::new(
                    &format!("privacy-certificate-{index}"),
                    host,
                    SettingsControl::Button {
                        label: "Forget".into(),
                        destructive: false,
                    },
                )
                .detail(match choice {
                    crate::certificate_choices::CertificateChoice::Identity { name, .. } => name,
                    crate::certificate_choices::CertificateChoice::None => {
                        "Don’t send a certificate".to_string()
                    }
                })
            })
            .collect();
        if !certificates.is_empty() {
            sections.push(section("Client certificates", certificates));
        }
        sections.push(section(
            "Site permissions",
            if rows.is_empty() {
                vec![SettingsRow::new(
                    "privacy-sites-none",
                    "No site has a remembered permission",
                    SettingsControl::None,
                )]
            } else {
                rows
            },
        ));
        sections
    }

    fn download_sections(&self) -> Vec<SettingsSection> {
        let history = self.downloads.controller.read().history_ids().len();
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
        let prefs = self.preferences.read().clone();
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
        if self.on_profile_settings_event(&event, cx) {
            cx.notify();
            return;
        }
        match event {
            SettingsEvent::Pane(pane) => {
                self.settings.get().pane = pane;
                self.settings.get().open_select = None;
            }
            SettingsEvent::OpenSelect(row) => {
                let reopen = self.settings.read().open_select.as_ref() == Some(&row);
                self.settings.get().open_select = (!reopen).then_some(row);
            }
            SettingsEvent::Dismiss => self.settings.get().open_select = None,
            SettingsEvent::Key { key, .. } => {
                if key.as_ref() == "escape" {
                    self.settings.get().open_select = None;
                }
            }
            SettingsEvent::Select(_)
            | SettingsEvent::SheetField(_)
            | SettingsEvent::SheetSwatch(_)
            | SettingsEvent::SheetCancel
            | SettingsEvent::SheetConfirm => {}
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
                self.settings.get().open_select = None;
                self.settings_choose(row.as_ref(), index, cx);
            }
            SettingsEvent::Press(row) => match row.as_ref() {
                "general-default-browser" => self.request_default_browser(),
                "downloads-clear-history" => {
                    crate::app::downloads_history::downloads_clear(self, cx)
                }
                "privacy-clear-data" => self.clear_data_in_browser(cx),
                row if row.starts_with("privacy-certificate-") => {
                    let profile = self.controller.read().active_profile();
                    let index = row["privacy-certificate-".len()..].parse::<usize>().ok();
                    let hosts = self.certificates.choices.read().hosts(profile);
                    if let Some((host, _)) = index.and_then(|index| hosts.get(index)) {
                        self.certificates.choices.get().forget(profile, host);
                        self.save_certificate_choices();
                    }
                }
                row if row.starts_with("privacy-site-") => {
                    let profile = self.controller.read().active_profile();
                    let index = row["privacy-site-".len()..].parse::<usize>().ok();
                    let sites = self.permissions.policies.read().sites(profile);
                    if let Some(origin) = index.and_then(|index| sites.get(index)) {
                        self.permissions.policies.get().forget_site(profile, origin);
                        self.save_site_permissions();
                        self.publish_popup_views();
                    }
                }
                _ => {}
            },
        }
        cx.notify();
    }

    fn settings_choose(&mut self, row: &str, index: usize, cx: &mut Context<Self>) {
        match row {
            "general-external-links" => {
                self.update_preferences(|p| p.external_links_ask = index == 1, cx);
            }
            "search-engine" => {
                if let Some(engine) = ENGINES.get(index) {
                    self.store.get().set_search_engine(engine.clone());
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
