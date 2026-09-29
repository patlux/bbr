//! Blueprint D06 (boards P-6, L-4, PR-08; DESIGN §12): Settings › Profiles
//! — every profile with its chip, space count and Edit…, New Profile…, the
//! selected profile's Delete… and its spaces — and the profile sheets.
//!
//! New Profile names the profile, picks its colour (the next hue
//! round-robin by default) and names the first space, which is created
//! empty and activated. Edit changes the name and colour. Delete names what
//! goes (its spaces, tabs, cookies and site data), then removes the profile
//! through the organization authority and deletes its website data store.
//! The last profile cannot be deleted.

use bbr_domain::ProfileHue;
use bbr_ui::surfaces::settings::{
    SettingsControl, SettingsEvent, SettingsRow, SettingsSection, SettingsSheet, SheetField,
    SheetPreview,
};
use bbr_web::ProfileId;
use gpui::{Context, SharedString};

use super::BbrApp;

/// Which sheet is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileSheetMode {
    New,
    Edit(ProfileId),
    Delete(ProfileId),
}

/// The open profile sheet's draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSheet {
    pub mode: ProfileSheetMode,
    pub name: String,
    pub hue: usize,
    pub first_space: String,
    /// Whether the first-space name was typed (else it follows the name).
    pub space_edited: bool,
    /// The focused field (0 name, 1 first space).
    pub focus: usize,
}

const NOTE: &str = "Separate cookies, logins, history and site data";

fn hue_index(hue: ProfileHue) -> usize {
    ProfileHue::ALL
        .iter()
        .position(|candidate| *candidate == hue)
        .unwrap_or(0)
}

fn count(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

impl BbrApp {
    /// The profile Delete…, Spaces in and the selection refer to.
    fn selected_profile(&self) -> Option<ProfileId> {
        let browser = self.controller.snapshot();
        self.settings
            .selected_profile
            .filter(|id| browser.profiles.iter().any(|profile| profile.id == *id))
            .or(Some(browser.active_profile))
    }

    /// Open the New Profile sheet on Settings › Profiles.
    pub(crate) fn open_profile_sheet_new(&mut self, cx: &mut Context<Self>) {
        let profiles = self.controller.snapshot().profiles.len();
        self.settings.pane = bbr_ui::surfaces::settings::SettingsPane::Profiles;
        self.settings.open_select = None;
        self.settings.sheet = Some(ProfileSheet {
            mode: ProfileSheetMode::New,
            name: String::new(),
            hue: profiles % ProfileHue::ALL.len(),
            first_space: String::new(),
            space_edited: false,
            focus: 0,
        });
        cx.notify();
    }

    pub(crate) fn profile_sections(&self) -> Vec<SettingsSection> {
        let browser = self.controller.snapshot();
        let theme = self.theme();
        let selected = self.selected_profile();
        let rows = browser
            .profiles
            .iter()
            .map(|profile| {
                let spaces = browser
                    .spaces
                    .iter()
                    .filter(|space| space.profile == profile.id)
                    .count();
                SettingsRow::new(
                    &format!("profile-{}", profile.index),
                    profile.title.clone(),
                    SettingsControl::ChipButton {
                        letter: crate::shell::space::profile_monogram(&profile.title)
                            .chars()
                            .next()
                            .unwrap_or('P'),
                        hue: bbr_ui::palette::ProfileHueRamp::tile(profile.hue, theme.mode).tile,
                        label: "Edit…".into(),
                    },
                )
                .detail(count(spaces, "space", "spaces"))
            })
            .collect();
        let mut manage = vec![SettingsRow::new(
            "profile-new",
            "Add a profile",
            SettingsControl::Button {
                label: "New Profile…".into(),
                destructive: false,
            },
        )
        .detail(NOTE)];
        let selected_profile =
            selected.and_then(|id| browser.profiles.iter().find(|profile| profile.id == id));
        if let Some(profile) = selected_profile.filter(|p| self.controller.can_delete_profile(p.id))
        {
            manage.push(
                SettingsRow::new(
                    "profile-delete",
                    format!("Delete “{}”…", profile.title),
                    SettingsControl::Button {
                        label: "Delete…".into(),
                        destructive: true,
                    },
                )
                .detail("Removes its spaces, tabs, cookies and site data"),
            );
        }
        let mut sections = vec![
            SettingsSection {
                title: Some("Profiles".into()),
                rows,
            },
            SettingsSection {
                title: None,
                rows: manage,
            },
        ];
        if let Some(profile) = selected_profile {
            let spaces = browser
                .spaces
                .iter()
                .filter(|space| space.profile == profile.id)
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
            sections.push(SettingsSection {
                title: Some(SharedString::from(format!("Spaces in {}", profile.title))),
                rows: spaces,
            });
        }
        sections
    }

    /// The open sheet as the window draws it.
    pub(crate) fn profile_sheet_model(&self) -> Option<SettingsSheet> {
        let sheet = self.settings.sheet.as_ref()?;
        let theme = self.theme();
        let browser = self.controller.snapshot();
        let hues: Vec<gpui::Rgba> = ProfileHue::ALL
            .iter()
            .map(|hue| bbr_ui::palette::ProfileHueRamp::tile(*hue, theme.mode).tile)
            .collect();
        let letter = |name: &str| {
            crate::shell::space::profile_monogram(if name.trim().is_empty() { "P" } else { name })
                .chars()
                .next()
                .unwrap_or('P')
        };
        let field = |label: &str, value: &str, focused: bool| SheetField {
            label: SharedString::from(label.to_string()),
            value: SharedString::from(value.to_string()),
            focused,
        };
        Some(match sheet.mode {
            ProfileSheetMode::New | ProfileSheetMode::Edit(_) => {
                let new = sheet.mode == ProfileSheetMode::New;
                let mut fields = vec![field("Name", &sheet.name, sheet.focus == 0)];
                if new {
                    fields.push(field("First space", &sheet.first_space, sheet.focus == 1));
                }
                SettingsSheet {
                    title: if new { "New Profile" } else { "Edit Profile" }.into(),
                    preview: Some(SheetPreview {
                        letter: letter(&sheet.name),
                        hue: hues[sheet.hue],
                        name: SharedString::from(if sheet.name.trim().is_empty() {
                            "New profile".to_string()
                        } else {
                            sheet.name.clone()
                        }),
                        note: NOTE.into(),
                    }),
                    fields,
                    swatches: Some((hues, sheet.hue)),
                    message: None,
                    cancel: "Cancel".into(),
                    confirm: if new { "Create Profile" } else { "Save" }.into(),
                    destructive: false,
                    confirm_enabled: !sheet.name.trim().is_empty(),
                }
            }
            ProfileSheetMode::Delete(profile) => {
                let title = browser
                    .profiles
                    .iter()
                    .find(|p| p.id == profile)
                    .map(|p| p.title.clone())
                    .unwrap_or_default();
                let spaces: Vec<_> = browser
                    .spaces
                    .iter()
                    .filter(|space| space.profile == profile)
                    .map(|space| space.id)
                    .collect();
                let tabs = browser
                    .all_tabs
                    .iter()
                    .filter(|tab| spaces.contains(&tab.space))
                    .count();
                SettingsSheet {
                    title: SharedString::from(format!("Delete “{title}”?")),
                    preview: None,
                    fields: Vec::new(),
                    swatches: None,
                    message: Some(SharedString::from(format!(
                        "Removes {}, {}, and its cookies and site data. This can’t be undone.",
                        count(spaces.len(), "space", "spaces"),
                        count(tabs, "tab", "tabs")
                    ))),
                    cancel: "Cancel".into(),
                    confirm: "Delete Profile".into(),
                    destructive: true,
                    confirm_enabled: true,
                }
            }
        })
    }

    /// Profile rows, buttons and sheet events; `false` for other events.
    pub(crate) fn on_profile_settings_event(
        &mut self,
        event: &SettingsEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if let Some(sheet) = self.settings.sheet.as_mut() {
            match event {
                SettingsEvent::SheetField(index) => sheet.focus = *index,
                SettingsEvent::SheetSwatch(index) => sheet.hue = *index % ProfileHue::ALL.len(),
                SettingsEvent::SheetCancel => self.settings.sheet = None,
                SettingsEvent::SheetConfirm => self.confirm_profile_sheet(cx),
                SettingsEvent::Key { key, text } => match key.as_ref() {
                    "escape" => self.settings.sheet = None,
                    "enter" => self.confirm_profile_sheet(cx),
                    "tab" => {
                        if sheet.mode == ProfileSheetMode::New {
                            sheet.focus = 1 - sheet.focus.min(1);
                        }
                    }
                    "backspace" => {
                        if sheet.focus == 0 {
                            sheet.name.pop();
                        } else {
                            sheet.first_space.pop();
                            sheet.space_edited = true;
                        }
                    }
                    _ => {
                        if let Some(text) = text.as_ref().filter(|t| !t.is_empty()) {
                            if sheet.focus == 0 {
                                sheet.name.push_str(text);
                            } else {
                                sheet.first_space.push_str(text);
                                sheet.space_edited = true;
                            }
                        }
                    }
                },
                // A sheet is modal: nothing behind it reacts.
                _ => {}
            }
            if let Some(sheet) = self.settings.sheet.as_mut() {
                if !sheet.space_edited {
                    sheet.first_space = sheet.name.clone();
                }
            }
            return true;
        }
        match event {
            SettingsEvent::Select(row) => {
                if let Some(index) = row
                    .strip_prefix("profile-")
                    .and_then(|i| i.parse::<usize>().ok())
                {
                    self.settings.selected_profile = self
                        .controller
                        .snapshot()
                        .profiles
                        .iter()
                        .find(|profile| profile.index == index)
                        .map(|profile| profile.id);
                    return true;
                }
                false
            }
            SettingsEvent::Press(row) => {
                if row.as_ref() == "profile-new" {
                    self.open_profile_sheet_new(cx);
                    return true;
                }
                if row.as_ref() == "profile-delete" {
                    if let Some(profile) = self.selected_profile() {
                        self.settings.sheet = Some(ProfileSheet {
                            mode: ProfileSheetMode::Delete(profile),
                            name: String::new(),
                            hue: 0,
                            first_space: String::new(),
                            space_edited: true,
                            focus: 0,
                        });
                    }
                    return true;
                }
                let Some(index) = row
                    .strip_prefix("profile-")
                    .and_then(|i| i.parse::<usize>().ok())
                else {
                    return false;
                };
                let browser = self.controller.snapshot();
                if let Some(profile) = browser.profiles.iter().find(|p| p.index == index) {
                    self.settings.selected_profile = Some(profile.id);
                    self.settings.sheet = Some(ProfileSheet {
                        mode: ProfileSheetMode::Edit(profile.id),
                        name: profile.title.clone(),
                        hue: hue_index(profile.hue),
                        first_space: String::new(),
                        space_edited: true,
                        focus: 0,
                    });
                }
                true
            }
            _ => false,
        }
    }

    fn confirm_profile_sheet(&mut self, cx: &mut Context<Self>) {
        let Some(sheet) = self.settings.sheet.clone() else {
            return;
        };
        let hue = ProfileHue::ALL[sheet.hue % ProfileHue::ALL.len()];
        match sheet.mode {
            ProfileSheetMode::New => {
                let name = sheet.name.trim().to_string();
                if name.is_empty() {
                    return;
                }
                let space_name = match sheet.first_space.trim() {
                    "" => name.clone(),
                    space => space.to_string(),
                };
                match self.controller.create_profile_space(name, space_name) {
                    Ok(event) => {
                        if let bbr_browser::ControllerEvent::ProfileCreated { profile, space } =
                            event
                        {
                            let _ = self.controller.set_profile_hue(profile, hue);
                            self.settings.selected_profile = Some(profile);
                            self.emit_browser(&event);
                            self.activate_space_in_browser(space, cx);
                        }
                    }
                    Err(error) => log::error!("could not create the profile: {error}"),
                }
            }
            ProfileSheetMode::Edit(profile) => {
                if sheet.name.trim().is_empty() {
                    return;
                }
                let _ = self.controller.rename_profile(profile, &sheet.name);
                let _ = self.controller.set_profile_hue(profile, hue);
            }
            ProfileSheetMode::Delete(profile) => {
                match self
                    .organization
                    .delete_profile(&mut self.controller, profile)
                {
                    Ok(event) => {
                        self.engine.borrow_mut().delete_profile_store(profile);
                        self.emit_browser(&event);
                        if let bbr_browser::ControllerEvent::ProfileDeleted {
                            fallback,
                            activated: true,
                            ..
                        } = event
                        {
                            self.activate_space_in_browser(fallback, cx);
                        }
                        self.settings.selected_profile = None;
                    }
                    Err(error) => log::error!("could not delete the profile: {error}"),
                }
            }
        }
        self.settings.sheet = None;
        self.persist_structure();
        cx.notify();
    }

    /// Blueprint D10: Settings › Privacy › Clear… brings the browser
    /// window forward with History's Clear Browsing Data sheet.
    pub(crate) fn clear_data_in_browser(&mut self, cx: &mut Context<Self>) {
        let Some(browser) = self.browser_window else {
            return;
        };
        let app = cx.entity();
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser, &app, cx, |view, window, cx| {
                window.activate_window();
                view.open_clear_browsing_data(window, cx);
            });
        });
    }

    /// Activate `space` against the browser window (a Settings action runs
    /// in the Settings window; the space's presentation belongs to the
    /// browser).
    fn activate_space_in_browser(&mut self, space: bbr_browser::SpaceId, cx: &mut Context<Self>) {
        let Some(browser) = self.browser_window else {
            return;
        };
        let app = cx.entity();
        cx.defer(move |cx| {
            let _ = crate::window_seam::with_app(&browser, &app, cx, |view, window, cx| {
                let _ = view.activate_space(space, window, cx);
            });
        });
    }
}
