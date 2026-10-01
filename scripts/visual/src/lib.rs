pub mod discovery;
pub mod environment;
pub mod gallery;
pub mod payload;
pub mod prepare;
pub mod process;
pub mod provenance;
pub mod read_only;
pub mod runner;
pub mod vm_environment;
pub mod vm_supervision;

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
        .collect()
}
