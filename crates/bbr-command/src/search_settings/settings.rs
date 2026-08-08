//! Global v1 search settings: value shape, version, scope, and defaults.

use crate::search_settings::engine::SearchEngine;

/// The settings version carried by [`SearchSettingsV1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsVersion {
    /// The initial portable search-settings shape.
    V1,
}

/// The ownership scope of search settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsScope {
    /// One value shared by all profiles.
    Global,
}

/// Global search settings, version 1.
///
/// The default search engine is a **global** v1 setting (spec §4: "Default
/// Search Engine = global in v1"): every profile uses the same engine until
/// an explicit later schema decision introduces per-profile settings.
/// Version and scope are fixed by the type — there is no way to construct a
/// non-global or non-v1 value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSettingsV1 {
    engine: SearchEngine,
}

impl SearchSettingsV1 {
    /// The v1 defaults: the curated DuckDuckGo engine.
    ///
    /// DuckDuckGo is the default because the settled command-bar search
    /// fallback already renders the DuckDuckGo endpoint, keeping first-run
    /// rendering byte-identical to today's shipping core.
    pub fn defaults() -> Self {
        Self {
            engine: SearchEngine::DuckDuckGo,
        }
    }

    /// Builds a value with the given engine, keeping the global v1 scope and
    /// version.
    pub fn with_engine(engine: SearchEngine) -> Self {
        Self { engine }
    }

    /// The selected search engine.
    pub fn engine(&self) -> &SearchEngine {
        &self.engine
    }

    /// The settings version, always [`SettingsVersion::V1`].
    pub fn version(&self) -> SettingsVersion {
        SettingsVersion::V1
    }

    /// The ownership scope, always [`SettingsScope::Global`].
    pub fn scope(&self) -> SettingsScope {
        SettingsScope::Global
    }
}

impl Default for SearchSettingsV1 {
    fn default() -> Self {
        Self::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_global_v1_duckduckgo() {
        let settings = SearchSettingsV1::defaults();
        assert_eq!(settings.engine(), &SearchEngine::DuckDuckGo);
        assert_eq!(settings.version(), SettingsVersion::V1);
        assert_eq!(settings.scope(), SettingsScope::Global);
    }

    #[test]
    fn default_trait_matches_defaults() {
        assert_eq!(SearchSettingsV1::default(), SearchSettingsV1::defaults());
    }

    #[test]
    fn with_engine_replaces_engine_but_keeps_global_v1() {
        let settings = SearchSettingsV1::with_engine(SearchEngine::Google);
        assert_eq!(settings.engine(), &SearchEngine::Google);
        assert_eq!(settings.version(), SettingsVersion::V1);
        assert_eq!(settings.scope(), SettingsScope::Global);
        assert_ne!(settings, SearchSettingsV1::defaults());
    }

    #[test]
    fn custom_engine_survives_value_construction() {
        let template =
            crate::search_settings::engine::CustomUrlTemplate::new("https://example.com/?q=%s")
                .unwrap();
        let engine = SearchEngine::Custom(template.clone());
        let settings = SearchSettingsV1::with_engine(engine);
        assert_eq!(settings.engine(), &SearchEngine::Custom(template.clone()));
        let again = SearchSettingsV1::with_engine(SearchEngine::Custom(template));
        assert_eq!(settings, again);
    }

    #[test]
    fn equality_and_debug_are_typed() {
        let a = SearchSettingsV1::defaults();
        let b = SearchSettingsV1::defaults();
        assert_eq!(a, b);
        let debug = format!("{a:?}");
        assert!(debug.contains("DuckDuckGo"), "debug must name the engine");
    }
}
