//! Search engine selection: five curated engines plus a validated custom URL
//! template containing the `%s` search placeholder.

/// The five curated search engines, plus an optional validated custom URL
/// template.
///
/// Curated construction never fails; [`SearchEngine::Custom`] only exists
/// through the validated [`CustomUrlTemplate`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SearchEngine {
    /// DuckDuckGo — `https://duckduckgo.com/?q=%s`.
    DuckDuckGo,
    /// Google — `https://www.google.com/search?q=%s`.
    Google,
    /// Bing — `https://www.bing.com/search?q=%s`.
    Bing,
    /// Ecosia — `https://www.ecosia.org/search?q=%s`.
    Ecosia,
    /// Kagi — `https://kagi.com/search?q=%s`.
    Kagi,
    /// A user-supplied URL template validated at construction.
    Custom(CustomUrlTemplate),
}

/// The five curated engines in canonical settings order.
pub const CURATED: [SearchEngine; 5] = [
    SearchEngine::DuckDuckGo,
    SearchEngine::Google,
    SearchEngine::Bing,
    SearchEngine::Ecosia,
    SearchEngine::Kagi,
];

impl SearchEngine {
    /// A stable lowercase identifier for settings labels, snapshot keys, and
    /// later persistence: `duckduckgo` | `google` | `bing` | `ecosia` |
    /// `kagi` | `custom`.
    pub fn id(&self) -> &'static str {
        match self {
            SearchEngine::DuckDuckGo => "duckduckgo",
            SearchEngine::Google => "google",
            SearchEngine::Bing => "bing",
            SearchEngine::Ecosia => "ecosia",
            SearchEngine::Kagi => "kagi",
            SearchEngine::Custom(_) => "custom",
        }
    }

    /// The human-readable settings label.
    pub fn display_name(&self) -> &'static str {
        match self {
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Google => "Google",
            SearchEngine::Bing => "Bing",
            SearchEngine::Ecosia => "Ecosia",
            SearchEngine::Kagi => "Kagi",
            SearchEngine::Custom(_) => "Custom",
        }
    }

    /// True for one of the five curated engines.
    pub fn is_curated(&self) -> bool {
        !matches!(self, SearchEngine::Custom(_))
    }

    /// The search URL template with exactly one `%s` placeholder.
    ///
    /// Curated templates are compile-time constants; a custom template
    /// returns its validated, trimmed value.
    pub fn template(&self) -> &str {
        match self {
            SearchEngine::DuckDuckGo => "https://duckduckgo.com/?q=%s",
            SearchEngine::Google => "https://www.google.com/search?q=%s",
            SearchEngine::Bing => "https://www.bing.com/search?q=%s",
            SearchEngine::Ecosia => "https://www.ecosia.org/search?q=%s",
            SearchEngine::Kagi => "https://kagi.com/search?q=%s",
            SearchEngine::Custom(template) => template.as_str(),
        }
    }
}

/// A validated custom search URL template containing exactly one `%s`.
///
/// Validation is deterministic and runs in this documented order:
///
/// 1. the template must be non-empty after trimming
///    ([`TemplateError::Empty`]);
/// 2. it must be an absolute `http://` or `https://` URL with a
///    case-insensitive scheme ([`TemplateError::SchemeNotHttp`]), which
///    excludes `file:`, `javascript:`, `data:`, and relative templates;
/// 3. it must contain exactly one `%s` placeholder
///    ([`TemplateError::MissingPlaceholder`] /
///    [`TemplateError::MultiplePlaceholders`]) — `%25s` is a literal `%s`
///    and does not count;
/// 4. every other `%` must begin a valid `%XX` percent-encoding
///    ([`TemplateError::MalformedPercent`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomUrlTemplate(String);

/// The reason a custom URL template was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateError {
    /// The template is empty or only whitespace.
    Empty,
    /// The template is not an absolute `http://` or `https://` URL.
    SchemeNotHttp,
    /// The template does not contain the `%s` search placeholder.
    MissingPlaceholder,
    /// The template contains the `%s` placeholder more than once.
    MultiplePlaceholders,
    /// A `%` outside the placeholder is not a valid `%XX` encoding.
    MalformedPercent,
}

impl CustomUrlTemplate {
    /// Validates `template` and stores it trimmed.
    pub fn new(template: &str) -> Result<Self, TemplateError> {
        let trimmed = template.trim();
        if trimmed.is_empty() {
            return Err(TemplateError::Empty);
        }
        if !scheme_is_http(trimmed) {
            return Err(TemplateError::SchemeNotHttp);
        }
        let mut matches = trimmed.match_indices("%s");
        match matches.next() {
            None => Err(TemplateError::MissingPlaceholder),
            Some((placeholder, _)) if matches.next().is_none() => {
                validate_percent_sequences(trimmed, placeholder)?;
                Ok(CustomUrlTemplate(trimmed.to_owned()))
            }
            Some(_) => Err(TemplateError::MultiplePlaceholders),
        }
    }

    /// The validated template exactly as stored (trimmed).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// True when the template begins with `http://` or `https://`
/// (case-insensitive).
fn scheme_is_http(template: &str) -> bool {
    let lower = template.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// Validates every `%` byte that is not the single placeholder start.
fn validate_percent_sequences(template: &str, placeholder: usize) -> Result<(), TemplateError> {
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }
        if i == placeholder {
            // The placeholder `%s`; its `s` is the following byte.
            i += 2;
            continue;
        }
        if i + 2 >= bytes.len()
            || !bytes[i + 1].is_ascii_hexdigit()
            || !bytes[i + 2].is_ascii_hexdigit()
        {
            return Err(TemplateError::MalformedPercent);
        }
        i += 3;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_table_is_stable() {
        let expected: &[(&str, &str, &str)] = &[
            ("duckduckgo", "DuckDuckGo", "https://duckduckgo.com/?q=%s"),
            ("google", "Google", "https://www.google.com/search?q=%s"),
            ("bing", "Bing", "https://www.bing.com/search?q=%s"),
            ("ecosia", "Ecosia", "https://www.ecosia.org/search?q=%s"),
            ("kagi", "Kagi", "https://kagi.com/search?q=%s"),
        ];
        assert_eq!(CURATED.len(), expected.len());
        for (engine, (id, name, template)) in CURATED.iter().zip(expected) {
            assert!(engine.is_curated(), "{id} must be curated");
            assert_eq!(engine.id(), *id);
            assert_eq!(engine.display_name(), *name);
            assert_eq!(engine.template(), *template);
            assert_eq!(engine.template().matches("%s").count(), 1);
        }
        // The canonical settings order is fixed for the future UI.
        let ids: Vec<&str> = CURATED.iter().map(SearchEngine::id).collect();
        assert_eq!(ids, vec!["duckduckgo", "google", "bing", "ecosia", "kagi"]);
    }

    #[test]
    fn custom_engine_delegates_identity_and_template() {
        let template = CustomUrlTemplate::new("https://example.com/search?q=%s&lang=en").unwrap();
        let engine = SearchEngine::Custom(template.clone());
        assert_eq!(engine.id(), "custom");
        assert_eq!(engine.display_name(), "Custom");
        assert!(!engine.is_curated());
        assert_eq!(engine.template(), template.as_str());
        assert_eq!(engine.template(), "https://example.com/search?q=%s&lang=en");
    }

    #[test]
    fn custom_template_accepts_wellformed_templates() {
        for template in [
            "https://example.com/search?q=%s",
            "http://example.com/search?q=%s",
            "https://example.com/%s",
            "https://example.com/search?q=%s&lang=en&safe=on",
            "https://example.com/search?q=%s&ref=%25literal",
            "HTTPS://Example.com/search?q=%s",
            "https://localhost:8080/?q=%s",
        ] {
            let parsed = CustomUrlTemplate::new(template).unwrap_or_else(|error| {
                panic!("expected valid template {template:?}, got {error:?}")
            });
            assert_eq!(parsed.as_str(), template.trim());
        }
    }

    #[test]
    fn custom_template_trims_input() {
        let parsed = CustomUrlTemplate::new("  https://example.com/?q=%s  ").unwrap();
        assert_eq!(parsed.as_str(), "https://example.com/?q=%s");
    }

    #[test]
    fn custom_template_rejects_invalid_templates() {
        let cases: &[(&str, TemplateError)] = &[
            ("", TemplateError::Empty),
            ("   ", TemplateError::Empty),
            ("example.com?q=%s", TemplateError::SchemeNotHttp),
            ("//example.com?q=%s", TemplateError::SchemeNotHttp),
            ("javascript:alert(1)?q=%s", TemplateError::SchemeNotHttp),
            ("file:///tmp/search?q=%s", TemplateError::SchemeNotHttp),
            ("data:text/plain,%s", TemplateError::SchemeNotHttp),
            ("https://example.com?q=", TemplateError::MissingPlaceholder),
            (
                "https://example.com?q=%25s",
                TemplateError::MissingPlaceholder,
            ),
            (
                "https://example.com?q=%s&x=%s",
                TemplateError::MultiplePlaceholders,
            ),
            (
                "https://example.com?q=%s%zz",
                TemplateError::MalformedPercent,
            ),
            ("https://example.com?q=%s%", TemplateError::MalformedPercent),
            ("https://example.com?q=%%s", TemplateError::MalformedPercent),
        ];
        for (template, expected) in cases {
            assert_eq!(
                CustomUrlTemplate::new(template),
                Err(*expected),
                "unexpected validation outcome for {template:?}"
            );
        }
    }

    #[test]
    fn template_equality_is_structural() {
        let a = CustomUrlTemplate::new("https://example.com/?q=%s").unwrap();
        let b = CustomUrlTemplate::new("https://example.com/?q=%s").unwrap();
        let c = CustomUrlTemplate::new("https://other.example/?q=%s").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(SearchEngine::DuckDuckGo, SearchEngine::DuckDuckGo);
        assert_ne!(SearchEngine::DuckDuckGo, SearchEngine::Google);
        assert_eq!(SearchEngine::Custom(a), SearchEngine::Custom(b));
    }
}
