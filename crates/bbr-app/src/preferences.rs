//! Blueprint D16 (DESIGN §3.4 §6 §8 §9): the app preferences the Settings
//! window edits — a small plain `key=value` file next to the store
//! (`preferences.txt` in the profile directory).
//!
//! The browser store is a SQLite file with a strict schema stamp and no
//! migrations; user preferences are not browser state, so they live apart
//! and a new preference never forces a schema bump (which would discard the
//! store). Unknown keys and unparsable values fall back to the defaults, so
//! an older or newer file never blocks startup. Writes replace the file
//! atomically (temporary file, then rename).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// The live "Open links that ask for a new window as Peek" choice, read by
/// the native new-window handler (installed before the app entity exists).
static PEEK_NEW_WINDOWS: AtomicBool = AtomicBool::new(true);

/// Publish the Peek choice to the new-window handler.
pub fn set_peek_new_windows(enabled: bool) {
    PEEK_NEW_WINDOWS.store(enabled, Ordering::Relaxed);
}

/// The default new-window policy with the Settings choice applied: with
/// Peek off, a request the policy would preview opens one ordinary tab
/// instead (blocking and background tabs are unchanged).
pub fn new_window_decision(
    request: &bbr_web::NewWindowRequest,
    intent: bbr_web::NewWindowIntent,
) -> bbr_web::NewWindowDecision {
    let decision = bbr_web::decision_for_intent(request, intent);
    // Blueprint D13: a site the user always allows opens its pop-ups as a
    // tab instead of the blocked notice.
    if decision == bbr_web::NewWindowDecision::Block
        && crate::site_permissions::popups_allowed_for_view(request.source)
    {
        return bbr_web::NewWindowDecision::OpenTab;
    }
    if intent == bbr_web::NewWindowIntent::Default
        && decision == bbr_web::NewWindowDecision::Preview
        && !PEEK_NEW_WINDOWS.load(Ordering::Relaxed)
    {
        bbr_web::NewWindowDecision::OpenTab
    } else {
        decision
    }
}

/// A three-way override of a macOS setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Follow {
    #[default]
    System,
    On,
    Off,
}

impl Follow {
    const fn key(self) -> &'static str {
        match self {
            Follow::System => "system",
            Follow::On => "on",
            Follow::Off => "off",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Follow::System),
            "on" => Some(Follow::On),
            "off" => Some(Follow::Off),
            _ => None,
        }
    }

    /// The effective value given the system's.
    pub fn resolve(self, system: bool) -> bool {
        match self {
            Follow::System => system,
            Follow::On => true,
            Follow::Off => false,
        }
    }
}

/// The appearance choice (System follows macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppearanceChoice {
    #[default]
    System,
    Light,
    Dark,
}

/// The density choice: automatic from the collection size (DESIGN §8), or
/// pinned by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DensityChoice {
    #[default]
    Automatic,
    Small,
    Normal,
    Large,
}

/// The paper new spaces start with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaperChoice {
    Subtle,
    #[default]
    Standard,
    Rich,
}

/// Every preference the Settings window edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preferences {
    pub appearance: AppearanceChoice,
    pub increase_contrast: Follow,
    pub reduce_motion: Follow,
    pub density: DensityChoice,
    pub default_paper: PaperChoice,
    /// Links that ask for a new window open as a Peek (else as a tab).
    pub peek_new_windows: bool,
    /// ⌘T lists tabs from every space (else only the active space's).
    pub palette_all_spaces: bool,
    /// Blueprint D13: links from other apps ask which space opens them
    /// (else they open in the last active space).
    pub external_links_ask: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            appearance: AppearanceChoice::System,
            increase_contrast: Follow::System,
            reduce_motion: Follow::System,
            density: DensityChoice::Automatic,
            default_paper: PaperChoice::Standard,
            peek_new_windows: true,
            palette_all_spaces: true,
            external_links_ask: false,
        }
    }
}

impl Preferences {
    /// Parse the file body; unknown keys and bad values keep the defaults.
    pub fn parse(body: &str) -> Self {
        let mut prefs = Self::default();
        for line in body.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "appearance" => {
                    if let Some(choice) = match value {
                        "system" => Some(AppearanceChoice::System),
                        "light" => Some(AppearanceChoice::Light),
                        "dark" => Some(AppearanceChoice::Dark),
                        _ => None,
                    } {
                        prefs.appearance = choice;
                    }
                }
                "increase_contrast" => {
                    if let Some(follow) = Follow::parse(value) {
                        prefs.increase_contrast = follow;
                    }
                }
                "reduce_motion" => {
                    if let Some(follow) = Follow::parse(value) {
                        prefs.reduce_motion = follow;
                    }
                }
                "density" => {
                    if let Some(choice) = match value {
                        "automatic" => Some(DensityChoice::Automatic),
                        "small" => Some(DensityChoice::Small),
                        "normal" => Some(DensityChoice::Normal),
                        "large" => Some(DensityChoice::Large),
                        _ => None,
                    } {
                        prefs.density = choice;
                    }
                }
                "default_paper" => {
                    if let Some(choice) = match value {
                        "subtle" => Some(PaperChoice::Subtle),
                        "standard" => Some(PaperChoice::Standard),
                        "rich" => Some(PaperChoice::Rich),
                        _ => None,
                    } {
                        prefs.default_paper = choice;
                    }
                }
                "peek_new_windows" => {
                    if let Ok(flag) = value.parse() {
                        prefs.peek_new_windows = flag;
                    }
                }
                "palette_all_spaces" => {
                    if let Ok(flag) = value.parse() {
                        prefs.palette_all_spaces = flag;
                    }
                }
                "external_links" => match value {
                    "ask" => prefs.external_links_ask = true,
                    "last_space" => prefs.external_links_ask = false,
                    _ => {}
                },
                _ => {}
            }
        }
        prefs
    }

    /// The file body.
    pub fn render(&self) -> String {
        let appearance = match self.appearance {
            AppearanceChoice::System => "system",
            AppearanceChoice::Light => "light",
            AppearanceChoice::Dark => "dark",
        };
        let density = match self.density {
            DensityChoice::Automatic => "automatic",
            DensityChoice::Small => "small",
            DensityChoice::Normal => "normal",
            DensityChoice::Large => "large",
        };
        let paper = match self.default_paper {
            PaperChoice::Subtle => "subtle",
            PaperChoice::Standard => "standard",
            PaperChoice::Rich => "rich",
        };
        format!(
            "appearance={appearance}\nincrease_contrast={}\nreduce_motion={}\ndensity={density}\n\
             default_paper={paper}\npeek_new_windows={}\npalette_all_spaces={}\n\
             external_links={}\n",
            self.increase_contrast.key(),
            self.reduce_motion.key(),
            self.peek_new_windows,
            self.palette_all_spaces,
            if self.external_links_ask {
                "ask"
            } else {
                "last_space"
            },
        )
    }
}

/// The preferences file of a profile directory.
pub fn path_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("preferences.txt")
}

/// Load the preferences (defaults when the file is missing or unreadable).
pub fn load(path: &Path) -> Preferences {
    std::fs::read_to_string(path)
        .map(|body| Preferences::parse(&body))
        .unwrap_or_default()
}

/// Write the preferences atomically.
pub fn save(path: &Path, prefs: &Preferences) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("txt.tmp");
    std::fs::write(&temporary, prefs.render())?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip_and_tolerate_unknown_input() {
        let prefs = Preferences {
            appearance: AppearanceChoice::Dark,
            increase_contrast: Follow::On,
            reduce_motion: Follow::Off,
            density: DensityChoice::Large,
            default_paper: PaperChoice::Rich,
            peek_new_windows: false,
            palette_all_spaces: false,
            external_links_ask: true,
        };
        assert_eq!(Preferences::parse(&prefs.render()), prefs);
        let tolerant = Preferences::parse("appearance=purple\nfuture=1\ndensity=small\nnonsense");
        assert_eq!(tolerant.appearance, AppearanceChoice::System);
        assert_eq!(tolerant.density, DensityChoice::Small);
        assert_eq!(Preferences::parse(""), Preferences::default());
    }

    #[test]
    fn follow_resolves_against_the_system_value() {
        assert!(Follow::System.resolve(true));
        assert!(!Follow::System.resolve(false));
        assert!(Follow::On.resolve(false));
        assert!(!Follow::Off.resolve(true));
    }

    #[test]
    fn saving_replaces_the_file_atomically() {
        let dir = std::env::temp_dir().join(format!("bbr-prefs-{}", std::process::id()));
        let path = path_in(&dir);
        let prefs = Preferences {
            density: DensityChoice::Normal,
            ..Preferences::default()
        };
        save(&path, &prefs).expect("save");
        assert_eq!(load(&path), prefs);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
