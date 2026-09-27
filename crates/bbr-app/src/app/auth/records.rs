//! The coordinator's redacted records, in-memory credential store, and the
//! redacted wire/render shapes derived from them (queue item 10).
//!
//! Everything observable about a challenge lives here in a shape that can
//! never contain credential text: the record of a challenge and its
//! contract-owned lifecycle state, the raw credential retained in memory
//! under its opaque handle key (the store), and the snapshot/surface types
//! the portal and the testkit consume. `Debug` is redacted by construction
//! and none of these types are ever `Serialize`/`Deserialize`, so no log,
//! snapshot, testkit payload, benchmark record, or error text can contain
//! the typed values (module invariant). Split from `super::coordinator`
//! (the orchestration core) by responsibility: the coordinator decides and
//! routes, this module owns the redacted data layer. The portal surface is
//! the extracted `bbr_ui::surfaces::auth` presentation (Group 8): this
//! module keeps the redacted store/surface and builds the presentation DTO
//! ([`AuthSheetSurface::model`]) the portal consumes.

use std::fmt;

use bbr_ui::surfaces::auth::AuthField;
use bbr_web::auth_challenges::{AuthChallenge, AuthState};
use bbr_web::WebViewId;

/// One recorded challenge and its contract-owned lifecycle state.
#[derive(Debug)]
pub(crate) struct AuthRecord {
    pub(crate) challenge: AuthChallenge,
    pub(crate) state: AuthState,
}

/// One raw credential retained in memory under its opaque handle key.
///
/// The future native adapter resolves the handle back to this credential at
/// the boundary; until then it lives only here. `Debug` is redacted and the
/// type is deliberately never serialized, so no observable shape can
/// contain it (module invariant). The unit tests prove the capture path
/// through the coordinator's `stored` accessor; the fields are read by the
/// future adapter only (contract packet non-goals: native credential
/// enumeration/storage remain later adapter work).
pub(crate) struct StoredCredential {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) username: String,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) password: String,
}

impl StoredCredential {
    pub(crate) fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl fmt::Debug for StoredCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Intentionally redacted one-line shape; never the credential.
        write!(f, "StoredCredential(redacted)")
    }
}

/// The wire-visible, redacted state of the credential sheet. Mirrored on
/// the protocol as `bbr_testkit::protocol::AuthSnapshot`. By construction
/// no field can hold a credential.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthSnapshotState {
    /// A credential sheet is currently presented.
    pub open: bool,
    /// The presented challenge's stable id.
    pub challenge: Option<u64>,
    /// The owning web view of the presented challenge.
    pub web_view: Option<WebViewId>,
    /// The normalized host the challenge belongs to (authority metadata,
    /// never a credential).
    pub host: String,
    /// The realm label, when the challenge carried one.
    pub realm: Option<String>,
    /// The closed HTTP scheme name (`Basic`/`Digest`/`NTLM`/`Negotiate`).
    pub scheme: String,
    /// `origin` or `proxy`.
    pub authority: String,
    /// The presented challenge's contract lifecycle state label.
    pub state: String,
    /// The redacted decision representation of the presented challenge's
    /// terminal state when it is on screen, or of the last decided
    /// challenge after the sheet closed (the contract's redaction:
    /// `credential:#<n>`, `cancelled`, `superseded:#<n>`, …). `None`
    /// while a fresh sheet awaits a decision.
    pub outcome: Option<String>,
    /// Which sheet field owns the keyboard cursor.
    pub focused_field: Option<String>,
    /// Masked: whether the username field holds any characters.
    pub username_entered: bool,
    /// Masked: whether the password field holds any characters.
    pub password_entered: bool,
    /// Challenge ids recorded but not yet presented (FIFO awaits).
    pub awaiting: Vec<u64>,
}

/// The redacted render data the portal window consumes to draw the sheet.
/// Carries the typed username for the field render only; `Debug` redacts it
/// and the type is never serialized.
pub struct AuthSheetSurface {
    pub host: String,
    pub realm: Option<String>,
    pub scheme: String,
    pub authority: String,
    pub focused: AuthField,
    /// The typed username (sheet render only; redacted in `Debug`).
    /// `pub(crate)`: the coordinator's `surface` builder sets it.
    pub(crate) username: String,
    pub password_entered: bool,
}

impl fmt::Debug for AuthSheetSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The username is sheet-local render data; never in Debug output.
        write!(
            f,
            "AuthSheetSurface {{ host: {:?}, realm: {:?}, scheme: {:?}, authority: {:?}, focused: {:?}, username: <redacted>, password_entered: {} }}",
            self.host,
            self.realm,
            self.scheme,
            self.authority,
            self.focused,
            self.password_entered
        )
    }
}

impl AuthSheetSurface {
    /// Masked: whether the username field holds any characters.
    pub fn username_entered(&self) -> bool {
        !self.username.is_empty()
    }

    /// The `bbr-ui` presentation model the portal consumes to render the
    /// sheet (the anchor's "records.rs builds the DTO" migration
    /// contract, Group 8): the app derives the DTO from the redacted
    /// surface and passes the shell's resolved `theme`, so the sheet follows
    /// the system appearance like every other portal surface (the former
    /// fixed `Theme::dark()` rendered a dark sheet over light chrome).
    pub fn model(&self, theme: bbr_ui::theme::Theme) -> bbr_ui::surfaces::auth::AuthSheetModel {
        bbr_ui::surfaces::auth::AuthSheetModel {
            theme,
            host: self.host.clone(),
            realm: self.realm.clone(),
            authority: self.authority.clone(),
            focused: self.focused,
            username: self.username.clone(),
            password_entered: self.password_entered,
        }
    }
}
