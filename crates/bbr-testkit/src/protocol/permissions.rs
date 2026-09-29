//! Blueprint D11: permission prompts and site settings on the wire.

use serde::{Deserialize, Serialize};

/// The prompt shown for the active page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionPromptSnapshot {
    pub request: u64,
    pub message: String,
    pub profile: String,
    /// `camera`, `microphone`.
    pub capabilities: Vec<String>,
    pub remember: bool,
    /// Requests waiting behind it.
    pub queued: usize,
    /// The rendered rect (window points), once drawn.
    pub rect: Option<[f32; 4]>,
}

/// The open site-settings popover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteSettingsSnapshot {
    pub host: String,
    pub connection: String,
    /// `(key, value)`: `camera` → `Allow` | `Ask` | `Block`.
    pub rows: Vec<(String, String)>,
    pub zoom: Option<u16>,
    pub clear_confirm: bool,
    pub rect: Option<[f32; 4]>,
}

/// The permission state of the page and the app.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PermissionsSnapshot {
    pub prompt: Option<PermissionPromptSnapshot>,
    pub site_settings: Option<SiteSettingsSnapshot>,
    /// The address's device mark: `live-camera`, `live-microphone`,
    /// `blocked-camera`, `blocked-microphone`.
    pub address_mark: Option<String>,
    /// Every decision handed to the engine: `(request, allowed)`.
    pub decisions: Vec<(u64, bool)>,
    /// Blueprint D13: the blocked pop-up notice's message while it shows.
    #[serde(default)]
    pub popup_notice: Option<String>,
}

/// One permission interaction through the method its control calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionInput {
    /// The prompt's "Remember for this site".
    Remember,
    /// The prompt's Allow / Don't Allow.
    Decide {
        allow: bool,
    },
    /// The address's leading slot.
    ToggleSiteSettings,
    /// A site-settings row: 0 Allow, 1 Ask, 2 Block.
    SetSite {
        row: usize,
        value: usize,
    },
    ResetZoom,
    ClearData,
    ClearCancel,
    ClearConfirm,
    AllSettings,
    /// A pointer press at window point (x, y) (the native observer's path).
    PointerDown {
        x: f32,
        y: f32,
    },
    /// Blueprint D13: the blocked pop-up notice's Open / Always Allow.
    PopupOpen,
    PopupAlwaysAllow,
}

/// Blueprint D12: the client-certificate sheet shown for the active page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientCertificateSnapshot {
    pub request: u64,
    pub message: String,
    /// `(name, detail, expired)` per identity.
    pub rows: Vec<(String, String, bool)>,
    pub selected: Option<usize>,
    pub remember: bool,
    /// Requests waiting behind it.
    pub queued: usize,
}

/// Blueprint D12: the sheet and every answer handed to the engine
/// (`identity:<fingerprint>`, `none`, `cancel`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClientCertificatesSnapshot {
    pub sheet: Option<ClientCertificateSnapshot>,
    pub answers: Vec<(u64, String)>,
}

/// Blueprint D12: one sheet interaction through the method its control
/// calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientCertificateInput {
    Select { index: usize },
    Remember,
    DontSend,
    Cancel,
    Continue,
}

/// Blueprint D13: the open-in-app sheet shown for the active page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAppSheetSnapshot {
    pub title: String,
    pub message: String,
    pub remember_label: Option<String>,
    pub remember: bool,
    pub confirm: String,
}

/// Blueprint D13: links to other apps — the sheet, the no-app banner and
/// every link handed to macOS (recorded, never launched, in testkit
/// builds).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OpenAppSnapshot {
    pub sheet: Option<OpenAppSheetSnapshot>,
    pub no_app_banner: Option<String>,
    pub opened: Vec<String>,
    /// The space chooser for a link from another app: `(title, rows,
    /// selected)`.
    #[serde(default)]
    pub space_chooser: Option<(String, Vec<String>, usize)>,
}

/// Blueprint D13: one open-in-app sheet interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpenAppInput {
    Remember,
    Cancel,
    Open,
    /// The space chooser's row press.
    ChooseSpace {
        index: usize,
    },
}

/// Blueprint D15: the first run while it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnboardingSnapshot {
    /// `welcome`, `import`, `default-browser`, `first-space`.
    pub step: String,
    /// Import sources `(label, detail, checked, enabled)`; `None` while
    /// they are read.
    pub sources: Option<Vec<(String, String, bool, bool)>>,
    /// The first space's name draft.
    pub name: String,
}

/// Blueprint D15: one first-run control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WelcomeInput {
    GetStarted,
    StartImport,
    ToggleSource { index: usize },
    Skip,
    Import,
    FocusName,
    Done,
}
