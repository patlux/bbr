//! Blueprint D16 (boards L-3…L-9, DESIGN §3.4 §6 §8 §9): the Settings
//! window — a macOS preferences window with a toolbar of panes and grouped
//! rows whose controls apply at once (no Save button).
//!
//! Presentation only: the app builds the plain [`SettingsModel`] from its
//! preferences and browser state and receives every interaction as one
//! [`SettingsEvent`] keyed by the row's stable id.

mod controls;
mod render;

use std::rc::Rc;

use gpui::{App, Rgba, SharedString, Window};

use crate::icons::Icon;
use crate::theme::Theme;

pub use render::settings_surface;

/// The §15 id of the Settings root.
pub const SETTINGS_ID: &str = "bbr-settings";

/// The panes, in toolbar order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsPane {
    #[default]
    General,
    Profiles,
    Search,
    Privacy,
    Downloads,
    Appearance,
    Shortcuts,
}

impl SettingsPane {
    pub const ALL: [SettingsPane; 7] = [
        SettingsPane::General,
        SettingsPane::Profiles,
        SettingsPane::Search,
        SettingsPane::Privacy,
        SettingsPane::Downloads,
        SettingsPane::Appearance,
        SettingsPane::Shortcuts,
    ];

    /// The toolbar label (also the window title).
    pub fn title(self) -> &'static str {
        match self {
            SettingsPane::General => "General",
            SettingsPane::Profiles => "Profiles",
            SettingsPane::Search => "Search",
            SettingsPane::Privacy => "Privacy",
            SettingsPane::Downloads => "Downloads",
            SettingsPane::Appearance => "Appearance",
            SettingsPane::Shortcuts => "Shortcuts",
        }
    }

    /// The stable key (`bbr-settings-tab-<key>`).
    pub fn key(self) -> &'static str {
        match self {
            SettingsPane::General => "general",
            SettingsPane::Profiles => "profiles",
            SettingsPane::Search => "search",
            SettingsPane::Privacy => "privacy",
            SettingsPane::Downloads => "downloads",
            SettingsPane::Appearance => "appearance",
            SettingsPane::Shortcuts => "shortcuts",
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|pane| pane.key() == key)
    }

    pub fn icon(self) -> Icon {
        match self {
            SettingsPane::General => Icon::Gear,
            SettingsPane::Profiles => Icon::Person,
            SettingsPane::Search => Icon::Search,
            SettingsPane::Privacy => Icon::Shield,
            SettingsPane::Downloads => Icon::Downloads,
            SettingsPane::Appearance => Icon::Eye,
            SettingsPane::Shortcuts => Icon::Keyboard,
        }
    }
}

/// One row's trailing control.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsControl {
    None,
    /// A status pill (e.g. DEFAULT).
    Badge(SharedString),
    Toggle(bool),
    /// A pop-up choice showing `options[value]`.
    Select {
        value: usize,
        options: Vec<SharedString>,
    },
    /// A segmented choice.
    Segmented {
        value: usize,
        options: Vec<SharedString>,
    },
    Button {
        label: SharedString,
        destructive: bool,
    },
    /// A plain text button (e.g. "Edit").
    Link(SharedString),
    /// Keycaps (e.g. ⌘ T).
    Keys(Vec<SharedString>),
    /// The three density cards (`None`: automatic, none pinned).
    Density(Option<usize>),
    /// A profile chip alone.
    Chip {
        letter: char,
        hue: Rgba,
    },
    /// A profile chip before a button.
    ChipButton {
        letter: char,
        hue: Rgba,
        label: SharedString,
    },
}

/// One row.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsRow {
    /// Stable id: `bbr-settings-row-<id>`.
    pub id: SharedString,
    pub title: SharedString,
    pub detail: Option<SharedString>,
    pub control: SettingsControl,
}

impl SettingsRow {
    pub fn new(id: &str, title: impl Into<SharedString>, control: SettingsControl) -> Self {
        Self {
            id: SharedString::from(id.to_string()),
            title: title.into(),
            detail: None,
            control,
        }
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A titled group of rows (one card).
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsSection {
    /// The uppercase label above the card (`None`: a card without label).
    pub title: Option<SharedString>,
    pub rows: Vec<SettingsRow>,
}

/// The whole window.
#[derive(Debug, Clone)]
pub struct SettingsModel {
    pub theme: Theme,
    pub pane: SettingsPane,
    pub sections: Vec<SettingsSection>,
    /// The row whose pop-up choice is open.
    pub open_select: Option<SharedString>,
}

/// Every interaction, keyed by the row id.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsEvent {
    Pane(SettingsPane),
    Toggle(SharedString),
    /// A choice (select option, segment, density card).
    Choose(SharedString, usize),
    /// A button or link.
    Press(SharedString),
    OpenSelect(SharedString),
    /// A press outside an open pop-up.
    Dismiss,
}

/// The one event sink.
pub type SettingsCallback = Rc<dyn Fn(SettingsEvent, &mut Window, &mut App)>;

/// The stable row id.
pub fn row_id(id: &str) -> SharedString {
    SharedString::from(format!("bbr-settings-row-{id}"))
}

/// The stable toolbar tab id.
pub fn tab_id(pane: SettingsPane) -> SharedString {
    SharedString::from(format!("bbr-settings-tab-{}", pane.key()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panes_keep_toolbar_order_and_round_trip_their_keys() {
        let keys: Vec<&str> = SettingsPane::ALL.iter().map(|pane| pane.key()).collect();
        assert_eq!(
            keys,
            [
                "general",
                "profiles",
                "search",
                "privacy",
                "downloads",
                "appearance",
                "shortcuts"
            ]
        );
        for pane in SettingsPane::ALL {
            assert_eq!(SettingsPane::parse(pane.key()), Some(pane));
        }
        assert_eq!(
            tab_id(SettingsPane::Search).as_ref(),
            "bbr-settings-tab-search"
        );
        assert_eq!(
            row_id("appearance-theme").as_ref(),
            "bbr-settings-row-appearance-theme"
        );
    }
}
