//! Portable global search-engine settings and search-URL rendering.
//!
//! This is the portable contract half of the general Settings page's Search
//! section (spec §4, "Allgemeine Settings-Seite"): the default search engine
//! is a **global** v1 setting shared by the command-bar search fallback and
//! the palette/URL-field search.
//!
//! It provides:
//!
//! - [`engine::SearchEngine`]: the five curated engines (DuckDuckGo, Google,
//!   Bing, Ecosia, Kagi) plus a validated [`engine::CustomUrlTemplate`]
//!   containing exactly one `%s` search placeholder;
//! - [`render::render_search_url`]: deterministic rendering and encoding of
//!   terms into an engine URL, matching the settled command-bar DuckDuckGo
//!   semantics (`?q=<percent-encoded terms>`);
//! - [`settings::SearchSettingsV1`]: the global-v1 value shape and defaults;
//! - [`store::SettingsStore`]: an object-safe store port with typed
//!   load/store/reset outcomes and no SQLite, filesystem, or native objects.
//!
//! The module is dependency-free and deterministic: identical inputs yield
//! identical URLs and outcomes on every platform. Persistence repositories,
//! command-bar activation wiring, and the settings UI remain later serial
//! slices (see the `general-search-settings-contract` task packet).
//!
//! ```
//! use bbr_command::search_settings::{render_search_url, SearchEngine, SearchSettingsV1};
//!
//! let settings = SearchSettingsV1::defaults();
//! assert_eq!(settings.engine(), &SearchEngine::DuckDuckGo);
//! assert_eq!(
//!     render_search_url(settings.engine(), "bbr browser"),
//!     "https://duckduckgo.com/?q=bbr%20browser"
//! );
//! ```

pub mod engine;
pub mod render;
pub mod settings;
pub mod store;

pub use engine::{CustomUrlTemplate, SearchEngine, TemplateError, CURATED};
pub use render::{percent_encode, render_search_url};
pub use settings::{SearchSettingsV1, SettingsScope, SettingsVersion};
pub use store::{LoadOutcome, ResetOutcome, SettingsStore, StoreOutcome};
