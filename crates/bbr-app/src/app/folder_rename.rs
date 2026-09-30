//! Blueprint D20 / DESIGN §11.11: inline folder rename. A folder created
//! from the plus menu opens in rename with its default title selected, so
//! the first printable character replaces it. Printable characters edit
//! the draft, Backspace deletes; Return, a press outside the field, or
//! focus leaving the folders region commits through the organization's
//! rename command; Escape keeps the committed title. The draft is
//! app-local presentation state; the folder title is owned by the
//! organization authority.
use bbr_domain::FolderId;
use gpui::{Context, KeyDownEvent, Window};

use super::BbrApp;

/// The open inline rename: the exact folder it binds (never re-derived),
/// the draft text, and whether the whole draft is selected (the first
/// printable character or Backspace replaces it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderRename {
    pub folder: FolderId,
    pub draft: String,
    pub replace_all: bool,
}

/// What one keystroke did to the open rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenameKey {
    /// Return: commit the draft.
    Commit,
    /// The draft or its selection changed (the keystroke is consumed).
    Edited,
    /// Not a rename keystroke: it continues to the region's handler.
    Ignored,
}

impl FolderRename {
    /// Apply one keystroke to the draft: Return commits; Backspace deletes
    /// the selection (or the last character); a bare arrow collapses the
    /// selection to the caret at the end (a single-line field edited at
    /// its end); a printable character replaces the selection or appends.
    /// Command shortcuts and control characters are not rename keys.
    pub(crate) fn apply_key(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        command: bool,
    ) -> RenameKey {
        if command {
            return RenameKey::Ignored;
        }
        match key {
            "enter" => return RenameKey::Commit,
            "backspace" => {
                if self.replace_all {
                    self.draft.clear();
                } else {
                    self.draft.pop();
                }
            }
            "left" | "right" | "up" | "down" => {}
            _ => {
                let Some(ch) = key_char.and_then(|chars| chars.chars().next()) else {
                    return RenameKey::Ignored;
                };
                if ch.is_control() {
                    return RenameKey::Ignored;
                }
                if self.replace_all {
                    self.draft.clear();
                }
                self.draft.push(ch);
            }
        }
        self.replace_all = false;
        RenameKey::Edited
    }
}

impl BbrApp {
    /// A new press begins (the root's capture phase, before any control's
    /// listener): forget the inside marks of the previous press. A surface
    /// and its owning control mark their own press inside and stop its
    /// propagation, so the root's bubble handler never consumes that mark;
    /// kept, it made the next press outside count as inside and left the
    /// surface open (found by the real-input E2E, reality-gaps item C).
    pub fn begin_root_press(&mut self) {
        self.clear_downloads_press_inside();
        self.u2.plus_menu_press_inside = false;
        self.space_config.press_inside = false;
        self.tab_menu.state.press_inside = false;
    }

    /// The root left-press handler: every open chrome surface that a press
    /// outside dismisses (§13 one pointer action). The render's root
    /// listener and the testkit's outside-press seam share this method.
    pub fn on_root_left_press(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_plus_menu_on_outside_press(window, cx);
        self.dismiss_downloads_on_outside_press(window, cx);
        self.dismiss_space_config_on_outside_press(window, cx);
        self.dismiss_tab_menu_on_outside_press(window, cx);
        self.dismiss_folder_rename_on_outside_press(window, cx);
    }

    /// Open the inline rename for `folder` with its current title selected.
    /// The folders region takes chrome focus (anchored on the folder), so
    /// the keyboard reaches the draft; focus leaving the region commits
    /// (see [`Self::commit_folder_rename_on_focus_change`]).
    pub fn begin_folder_rename(
        &mut self,
        folder: FolderId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let organization = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read());
        let Some(title) = organization
            .folders
            .iter()
            .find(|f| f.id == folder)
            .map(|f| f.title.clone())
        else {
            return;
        };
        self.u2.folder_rename = Some(FolderRename {
            folder,
            draft: title,
            replace_all: true,
        });
        self.focus_folder_row(folder, window, cx);
    }

    /// The adapter view of the open rename: `(folder, draft, selected)`.
    pub(crate) fn folder_rename_view(&self) -> Option<(FolderId, &str, bool)> {
        self.u2
            .folder_rename
            .as_ref()
            .map(|r| (r.folder, r.draft.as_str(), r.replace_all))
    }

    /// Whether an inline rename is open.
    pub fn folder_rename_open(&self) -> bool {
        self.u2.folder_rename.is_some()
    }

    /// The rename keyboard seam (the folders region owns chrome focus while
    /// the rename is open). Returns whether the keystroke was consumed.
    pub(super) fn on_folder_rename_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(rename) = self.u2.folder_rename.as_mut() else {
            return false;
        };
        let keystroke = &event.keystroke;
        let key_char = keystroke.key_char.as_deref();
        match rename.apply_key(&keystroke.key, key_char, keystroke.modifiers.platform) {
            RenameKey::Ignored => return false,
            RenameKey::Commit => {
                cx.stop_propagation();
                self.commit_folder_rename(window, cx);
                return true;
            }
            RenameKey::Edited => {}
        }
        cx.stop_propagation();
        cx.notify();
        true
    }

    /// Commit the open rename: a changed, non-empty draft renames the folder
    /// through the organization command; an empty or unchanged draft keeps
    /// the committed title. Focus stays on the folder row.
    pub fn commit_folder_rename(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.finish_folder_rename() {
            cx.notify();
        }
    }

    /// Every production focus-domain change reports through `emit_focus`;
    /// when the folders region no longer owns focus (Cmd-T, Cmd-L, a tab
    /// row, the web view), an open rename commits like Return. The caller
    /// re-renders. GPUI's focus-out listeners are not used: they compare
    /// rendered frames and stay silent while the window is inactive.
    pub(super) fn commit_folder_rename_on_focus_change(&mut self, window: &Window) {
        if self.u2.folder_rename.is_some() && !self.u2.folders_handle.is_focused(window) {
            self.finish_folder_rename();
        }
    }

    /// Close the open rename and apply a changed, non-empty draft through
    /// the organization command. Returns whether a rename was open.
    fn finish_folder_rename(&mut self) -> bool {
        let Some(rename) = self.u2.folder_rename.take() else {
            return false;
        };
        let current = self
            .organization
            .bookkeep()
            .organization_snapshot(&self.controller.read())
            .folders
            .iter()
            .find(|f| f.id == rename.folder)
            .map(|f| f.title.clone());
        if current.as_deref() != Some(rename.draft.trim()) && !rename.draft.trim().is_empty() {
            let renamed = self
                .organization
                .get()
                .rename_folder(rename.folder, &rename.draft);
            match renamed {
                Ok(event) => log::debug!("folder renamed: {event:?}"),
                Err(error) => log::error!("could not rename folder: {error}"),
            }
        }
        true
    }

    /// Escape: close the rename and keep the committed title.
    pub fn cancel_folder_rename(&mut self, cx: &mut Context<Self>) {
        if self.u2.folder_rename.take().is_some() {
            cx.notify();
        }
    }

    /// A press anywhere outside the rename field commits it (the Finder
    /// convention). The field stops propagation of its own presses, so they
    /// never reach the root handler that calls this.
    pub fn dismiss_folder_rename_on_outside_press(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.u2.folder_rename.is_some() {
            self.commit_folder_rename(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> FolderRename {
        FolderRename {
            folder: FolderId(1),
            draft: "New Folder".into(),
            replace_all: true,
        }
    }

    #[test]
    fn the_first_character_replaces_the_selected_default_title() {
        let mut rename = fresh();
        assert_eq!(rename.apply_key("r", Some("r"), false), RenameKey::Edited);
        assert_eq!(rename.draft, "r");
        assert!(!rename.replace_all);
        rename.apply_key("space", Some(" "), false);
        rename.apply_key("a", Some("A"), false);
        assert_eq!(rename.draft, "r A");
    }

    #[test]
    fn backspace_clears_the_selection_then_deletes_one_character() {
        let mut rename = fresh();
        assert_eq!(
            rename.apply_key("backspace", None, false),
            RenameKey::Edited
        );
        assert_eq!(rename.draft, "", "the selected title is deleted whole");
        rename.apply_key("h", Some("h"), false);
        rename.apply_key("i", Some("i"), false);
        rename.apply_key("backspace", None, false);
        assert_eq!(rename.draft, "h");
    }

    #[test]
    fn arrows_collapse_the_selection_and_keep_the_draft() {
        let mut rename = fresh();
        assert_eq!(rename.apply_key("right", None, false), RenameKey::Edited);
        assert!(!rename.replace_all);
        rename.apply_key("s", Some("s"), false);
        assert_eq!(rename.draft, "New Folders", "typing appends after an arrow");
    }

    #[test]
    fn return_commits_and_command_shortcuts_pass_through() {
        let mut rename = fresh();
        assert_eq!(rename.apply_key("enter", None, false), RenameKey::Commit);
        assert_eq!(rename.apply_key("t", Some("t"), true), RenameKey::Ignored);
        assert_eq!(
            rename.apply_key("tab", Some("\t"), false),
            RenameKey::Ignored
        );
        assert_eq!(rename.apply_key("f1", None, false), RenameKey::Ignored);
        assert_eq!(rename.draft, "New Folder", "ignored keys never edit");
        assert!(rename.replace_all, "ignored keys keep the selection");
    }
}
