pub mod browser_output;
pub mod discovery;
pub mod environment;
pub mod gallery;
pub mod gallery_selection;
pub mod guest;
pub mod payload;
pub mod prepare;
pub mod process;
pub mod provenance;
pub mod read_only;
pub mod review;
pub mod runner;
pub mod selected_request;
pub mod semantic;
pub mod vm_capture;
pub mod vm_check;
pub mod vm_cleanup;
pub mod vm_environment;
pub mod vm_rendering;
pub mod vm_runtime;
pub mod vm_semantic;
pub mod vm_supervision;
pub mod vm_transport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite {
    Gallery,
    Browser,
}
impl Suite {
    pub fn parse(raw: &str) -> anyhow::Result<Self> {
        match raw {
            "gallery" => Ok(Self::Gallery),
            "browser" => Ok(Self::Browser),
            _ => anyhow::bail!("unknown suite: {raw}"),
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::Gallery => "gallery",
            Self::Browser => "browser",
        }
    }
    pub fn config(self) -> &'static str {
        match self {
            Self::Gallery => "visual-gallery.toml",
            Self::Browser => "visual.toml",
        }
    }
}

/// Required additional application stills, separate from the historical shell pairs.
pub const BROWSER_SURFACES: [&str; 77] = [
    "history",
    "history-clear-data",
    "library-empty",
    "library-populated",
    "settings-general",
    "settings-profiles",
    "settings-search",
    "settings-privacy",
    "settings-downloads",
    "settings-appearance",
    "settings-shortcuts",
    "library-collections",
    "library-selection-single",
    "library-selection-multiple",
    "library-move-sheet",
    "library-remove-sheet",
    "library-collection-create",
    "history-search",
    "history-search-empty",
    "history-clear-range-menu",
    "history-recently-closed",
    "settings-general-menu",
    "settings-search-menu",
    "settings-motion-menu",
    "settings-profile-create",
    "settings-profile-edit",
    "settings-shortcuts-scroll-1",
    "settings-shortcuts-scroll-2",
    "settings-shortcuts-scroll-3",
    "command-palette-open",
    "command-palette-actions-match",
    "command-palette-actions-empty",
    "find-open",
    "find-matches",
    "find-no-matches",
    "downloads-empty",
    "downloads-history",
    "downloads-missing-file",
    "footer-spaces",
    "footer-hover-downloads",
    "footer-hover-chip",
    "footer-plus-open",
    "footer-strip-focus",
    "footer-downloads-running",
    "footer-drag-target",
    "footer-overflow",
    "library-search",
    "library-search-empty",
    "library-sort-title",
    "library-row-rename",
    "library-collection-rename",
    "library-collection-remove",
    "library-long-list",
    "history-dated",
    "history-range-today",
    "history-selected",
    "history-cleared",
    "settings-contrast-menu",
    "settings-profile-create-named",
    "settings-profiles-multiple",
    "settings-profile-delete",
    "welcome",
    "welcome-import",
    "welcome-default-browser",
    "welcome-first-space",
    "about",
    "find-next",
    "command-palette-url",
    "command-palette-search",
    "peek-bar",
    "permission-prompt",
    "permission-site-settings",
    "client-certificate-sheet",
    "settings-privacy-populated",
    "open-app-no-app",
    "open-app-sheet",
    "auth-sheet",
];

/// What a browser case's PNG holds. The shell pairs keep the whole window
/// or its 304 pt rail and Settings its own window; every History, Saved
/// Pages, Downloads and footer still is read back as its element region
/// only, whose exact size the case's snapshot records as `visual_region`.
/// Command Palette and Find are their own portal windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserStill {
    Window,
    Rail,
    Settings,
    /// The About BBR window (340 x 380 pt).
    About,
    Element,
}

pub fn browser_still(case: &str) -> BrowserStill {
    let base = case.rsplit_once("--").map_or(case, |(base, _)| base);
    if base.ends_with("-rail") {
        BrowserStill::Rail
    } else if base.starts_with("settings-") {
        BrowserStill::Settings
    } else if base == "about" {
        BrowserStill::About
    } else if base == "history"
        || base == "welcome"
        || ["history-", "library-", "downloads-", "footer-", "welcome-"]
            .iter()
            .any(|prefix| base.starts_with(prefix))
    {
        BrowserStill::Element
    } else {
        BrowserStill::Window
    }
}

pub fn browser_cases() -> Vec<String> {
    ["ordinary", "representative"]
        .into_iter()
        .flat_map(|fixture| {
            ["full", "rail"].into_iter().flat_map(move |region| {
                ["light", "dark"]
                    .into_iter()
                    .map(move |mode| format!("{fixture}-{region}--{mode}"))
            })
        })
        .chain(BROWSER_SURFACES.into_iter().flat_map(|surface| {
            ["light", "dark"]
                .into_iter()
                .map(move |mode| format!("{surface}--{mode}"))
        }))
        .collect()
}
