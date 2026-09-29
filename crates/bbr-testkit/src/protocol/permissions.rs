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
}
