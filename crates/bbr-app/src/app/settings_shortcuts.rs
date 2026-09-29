//! Blueprint D16: the Settings › Shortcuts pane — the bound keys, grouped,
//! read against the keymap.

use bbr_ui::surfaces::settings::{SettingsControl, SettingsRow, SettingsSection};
use gpui::SharedString;

/// The shortcut list: `(group, title, keystroke)`, the keystroke exactly as
/// bound (a unit test checks every one against the keymap).
pub(crate) const SHORTCUTS: [(&str, &str, &str); 21] = [
    ("Tabs", "New tab", "cmd-t"),
    ("Tabs", "Close tab", "cmd-w"),
    ("Tabs", "Reopen closed tab", "cmd-shift-t"),
    ("Tabs", "Pin / unpin", "cmd-shift-p"),
    ("Tabs", "Next tab", "ctrl-tab"),
    ("Tabs", "Previous tab", "ctrl-shift-tab"),
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
    ("Library", "History", "cmd-y"),
    ("Library", "Clear browsing data", "cmd-shift-backspace"),
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
    caps.push(SharedString::from(match key {
        "backspace" => "⌫".to_string(),
        "tab" => "⇥".to_string(),
        key => key.to_uppercase(),
    }));
    caps
}

pub(crate) fn shortcut_sections() -> Vec<SettingsSection> {
    let mut sections: Vec<SettingsSection> = Vec::new();
    for (group, title, keystroke) in SHORTCUTS {
        let row = SettingsRow::new(
            &format!("shortcut-{keystroke}"),
            title,
            SettingsControl::Keys(keycaps(keystroke)),
        );
        match sections.last_mut() {
            Some(last) if last.title.as_deref() == Some(group) => last.rows.push(row),
            _ => sections.push(SettingsSection {
                title: Some(SharedString::from(group)),
                rows: vec![row],
            }),
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
        assert_eq!(caps("ctrl-shift-tab"), ["⌃", "⇧", "⇥"]);
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
