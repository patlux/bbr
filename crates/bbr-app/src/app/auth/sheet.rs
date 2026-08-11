//! The credential sheet's edit state.
//!
//! [`AuthSheetState`] is the sheet's own memory: the username and password
//! a user types. The values are **never** serialized and `Debug` is
//! redacted by construction (module invariant), so no snapshot, log, event,
//! benchmark record, or error text can contain them. The lifecycle does not
//! read the values directly: on submit the coordinator captures them once
//! into its in-memory credential store under an opaque handle and hands the
//! redacted [`bbr_web::auth_challenges::CredentialHandle`] to the contract.
//!
//! The field-cursor vocabulary ([`bbr_ui::surfaces::auth::AuthField`]) is
//! the extracted production surface's (Group 8): the sheet's edit state
//! rides the same bbr-ui field enum the portal render and the gallery
//! story use — one focus vocabulary, no duplicate.

use std::fmt;

use bbr_ui::surfaces::auth::AuthField;

/// The editable state of one open credential sheet.
///
/// `Debug` is intentionally redacted: the shape prints the focused field and
/// masked entered flags, never the typed values. The type deliberately has
/// no serde derives, so the values cannot ride a wire shape.
#[derive(Clone, PartialEq, Eq)]
pub struct AuthSheetState {
    /// The typed username (sheet-local only).
    username: String,
    /// The typed password (sheet-local only).
    password: String,
    /// The field owning the keyboard cursor.
    pub focused: AuthField,
}

impl fmt::Debug for AuthSheetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthSheetState {{ focused: {:?}, username_entered: {}, password_entered: {} }}",
            self.focused,
            !self.username.is_empty(),
            !self.password.is_empty()
        )
    }
}

impl Default for AuthSheetState {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthSheetState {
    /// A fresh sheet: empty fields, cursor in the username field.
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            focused: AuthField::Username,
        }
    }

    /// Whether any text was typed into either field (masked evidence for the
    /// snapshot; never the text itself).
    pub fn username_entered(&self) -> bool {
        !self.username.is_empty()
    }

    /// Whether the password field holds any characters (masked evidence for
    /// the snapshot; never the text itself).
    pub fn password_entered(&self) -> bool {
        !self.password.is_empty()
    }

    /// The typed username — sheet-local render data only. Callers must not
    /// serialize or log it; the only representation that may leave the sheet
    /// is the contract's redacted handle.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// The typed password — sheet-local render data only (the render draws
    /// bullets from this, never the characters in any external shape).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Append one typed character to the focused field (the production key
    /// path; UTF-8-safe single char).
    pub fn type_char(&mut self, ch: &str) {
        let field = self.focused_field_mut();
        field.push_str(ch);
    }

    /// Remove the last character of the focused field (UTF-8-safe: `pop`
    /// removes one char).
    pub fn backspace(&mut self) {
        self.focused_field_mut().pop();
    }

    /// Move the cursor to the other field (Tab).
    pub fn focus_next(&mut self) {
        self.focused = match self.focused {
            AuthField::Username => AuthField::Password,
            AuthField::Password => AuthField::Username,
        };
    }

    /// Capture the typed values (submit path; the coordinator stores them
    /// under an opaque handle key before this state is cleared).
    pub fn capture(&self) -> (String, String) {
        (self.username.clone(), self.password.clone())
    }

    fn focused_field_mut(&mut self) -> &mut String {
        match self.focused {
            AuthField::Username => &mut self.username,
            AuthField::Password => &mut self.password,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_renders_typed_values() {
        let mut sheet = AuthSheetState::new();
        sheet.type_char("s");
        sheet.type_char("e");
        sheet.type_char("c");
        sheet.type_char("r");
        sheet.type_char("e");
        sheet.type_char("t");
        sheet.focus_next();
        sheet.type_char("p");
        sheet.type_char("w");
        let debug = format!("{sheet:?}");
        assert!(
            !debug.contains("secret") && !debug.contains("pw"),
            "Debug must not contain typed values: {debug}"
        );
        assert!(debug.contains("password_entered: true"));
        assert!(debug.contains("username_entered: true"));
    }

    #[test]
    fn typing_routes_to_the_focused_field() {
        let mut sheet = AuthSheetState::new();
        assert_eq!(sheet.focused, AuthField::Username);
        sheet.type_char("a");
        sheet.type_char("l");
        assert_eq!(sheet.username(), "al");
        assert!(!sheet.password_entered());
        sheet.focus_next();
        assert_eq!(sheet.focused, AuthField::Password);
        sheet.type_char("x");
        assert_eq!(sheet.password(), "x");
        assert!(sheet.password_entered());
        sheet.focus_next();
        assert_eq!(sheet.focused, AuthField::Username);
    }

    #[test]
    fn backspace_removes_one_character_utf8_safe() {
        let mut sheet = AuthSheetState::new();
        sheet.type_char("ß");
        sheet.backspace();
        assert_eq!(sheet.username(), "");
        sheet.type_char("a");
        sheet.backspace();
        assert!(!sheet.username_entered());
    }

    #[test]
    fn capture_returns_clones_and_fresh_sheets_start_empty() {
        let mut sheet = AuthSheetState::new();
        sheet.type_char("a");
        sheet.focus_next();
        sheet.type_char("b");
        let (username, password) = sheet.capture();
        assert_eq!((username.as_str(), password.as_str()), ("a", "b"));
        // A fresh sheet (the coordinator replaces the state with `new()` on
        // every presentation and after every decision) starts empty with the
        // cursor in the username field.
        let fresh = AuthSheetState::new();
        assert!(!fresh.username_entered());
        assert!(!fresh.password_entered());
        assert_eq!(fresh.focused, AuthField::Username);
    }
}
