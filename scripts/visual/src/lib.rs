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
pub const BROWSER_SURFACES: [&str; 32] = [
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
];

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
