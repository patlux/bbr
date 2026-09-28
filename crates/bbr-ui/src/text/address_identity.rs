//! Blueprint D03 (DESIGN §11.5, resolving §17.13): what the address control
//! names and which leading mark it shows for a URL.
//!
//! - `https` sites — and plain `http` on a loopback host, which the web
//!   platform treats as a secure context — show the domain alone.
//! - Other plain `http` sites show the domain behind the not-secure mark.
//! - BBR's internal pages (`bbr://library`, `settings`, `history`,
//!   `welcome`) show a short name behind their glyph.
//! - Local files show the file name (never the path) behind the document
//!   mark.
//! - Any other scheme keeps the scheme's short name as a placeholder.

/// A BBR internal page with a name in the address control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalPage {
    Library,
    Settings,
    History,
    Welcome,
}

impl InternalPage {
    /// The short name the address control shows.
    pub fn name(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Settings => "Settings",
            Self::History => "History",
            Self::Welcome => "Welcome",
        }
    }

    fn from_host(host: &str) -> Option<Self> {
        match host {
            "library" => Some(Self::Library),
            "settings" => Some(Self::Settings),
            "history" => Some(Self::History),
            "welcome" => Some(Self::Welcome),
            _ => None,
        }
    }
}

/// The address control's identity for one URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressIdentity {
    /// A web site: its display domain and whether the connection is a
    /// secure context.
    Site { domain: String, secure: bool },
    /// A BBR internal page.
    Internal(InternalPage),
    /// A local file: its decoded file name.
    File { name: String },
    /// Another scheme: its short name (the placeholder).
    Other { scheme: String },
    /// Nothing to show.
    Empty,
}

impl AddressIdentity {
    /// The text the control shows.
    pub fn label(&self) -> String {
        match self {
            Self::Site { domain, .. } => domain.clone(),
            Self::Internal(page) => page.name().to_string(),
            Self::File { name } => name.clone(),
            Self::Other { scheme } => scheme.clone(),
            Self::Empty => String::new(),
        }
    }

    /// The stable wire name of the identity kind (typed snapshot).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Site { secure: true, .. } => "secure",
            Self::Site { secure: false, .. } => "not-secure",
            Self::Internal(_) => "internal",
            Self::File { .. } => "file",
            Self::Other { .. } => "other",
            Self::Empty => "empty",
        }
    }
}

/// Classify `url` for the address control.
pub fn address_identity(url: &str) -> AddressIdentity {
    let url = url.trim();
    let Some((scheme, rest)) = url.split_once(':') else {
        return AddressIdentity::Empty;
    };
    let scheme = scheme.trim().to_ascii_lowercase();
    match scheme.as_str() {
        "http" | "https" => {
            let domain = super::address_domain(url);
            if domain.is_empty() {
                return AddressIdentity::Empty;
            }
            let secure = scheme == "https" || is_loopback(&host_part(rest));
            AddressIdentity::Site { domain, secure }
        }
        "bbr" => match InternalPage::from_host(&host_part(rest)) {
            Some(page) => AddressIdentity::Internal(page),
            None => AddressIdentity::Other { scheme },
        },
        "file" => {
            let path = rest.trim_start_matches('/');
            let name = path
                .split(['?', '#'])
                .next()
                .unwrap_or("")
                .rsplit('/')
                .find(|segment| !segment.is_empty())
                .map(percent_decode)
                .unwrap_or_default();
            if name.is_empty() {
                AddressIdentity::Other { scheme }
            } else {
                AddressIdentity::File { name }
            }
        }
        "" => AddressIdentity::Empty,
        _ => AddressIdentity::Other { scheme },
    }
}

/// The lower-cased host of a `//host:port/path` remainder (brackets kept
/// off IPv6 literals).
fn host_part(rest: &str) -> String {
    let authority = rest.strip_prefix("//").unwrap_or(rest);
    let authority = authority.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped.split(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    };
    host.trim().to_ascii_lowercase()
}

/// Loopback hosts are potentially trustworthy origins (W3C Secure
/// Contexts): `localhost`, `*.localhost`, `127.0.0.0/8` and `::1`.
fn is_loopback(host: &str) -> bool {
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "::1"
        || host.strip_prefix("127.").is_some_and(|tail| {
            tail.split('.').count() == 3 && tail.split('.').all(|o| o.parse::<u8>().is_ok())
        })
}

/// Decode `%XX` escapes (UTF-8); malformed escapes stay as written.
fn percent_decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&segment[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(domain: &str, secure: bool) -> AddressIdentity {
        AddressIdentity::Site {
            domain: domain.into(),
            secure,
        }
    }

    #[test]
    fn https_is_secure_and_plain_http_is_not() {
        assert_eq!(address_identity("https://x.com/a"), site("x.com", true));
        assert_eq!(
            address_identity("http://router.lan/"),
            site("router.lan", false)
        );
        assert_eq!(address_identity("http://router.lan/").kind(), "not-secure");
    }

    #[test]
    fn loopback_http_is_a_secure_context() {
        assert!(matches!(
            address_identity("http://127.0.0.1:8080/x"),
            AddressIdentity::Site { secure: true, .. }
        ));
        assert!(matches!(
            address_identity("http://localhost:3000/"),
            AddressIdentity::Site { secure: true, .. }
        ));
        assert!(matches!(
            address_identity("http://app.localhost/"),
            AddressIdentity::Site { secure: true, .. }
        ));
        assert!(matches!(
            address_identity("http://[::1]:80/"),
            AddressIdentity::Site { secure: true, .. }
        ));
        assert!(matches!(
            address_identity("http://127.example.com/"),
            AddressIdentity::Site { secure: false, .. }
        ));
    }

    #[test]
    fn internal_pages_have_short_names() {
        assert_eq!(address_identity("bbr://library").label(), "Library");
        assert_eq!(
            address_identity("bbr://settings/profiles").label(),
            "Settings"
        );
        assert_eq!(address_identity("bbr://history").label(), "History");
        assert_eq!(address_identity("bbr://welcome").label(), "Welcome");
        assert_eq!(address_identity("bbr://unknown").kind(), "other");
    }

    #[test]
    fn local_files_show_the_decoded_file_name_only() {
        assert_eq!(
            address_identity("file:///Users/p/Reports/report%20q3.pdf"),
            AddressIdentity::File {
                name: "report q3.pdf".into()
            }
        );
        assert_eq!(address_identity("file:///tmp/x.html?y#z").label(), "x.html");
        assert_eq!(address_identity("file:///").kind(), "other");
    }

    #[test]
    fn other_schemes_and_empty_input() {
        assert_eq!(address_identity("about:blank").label(), "about");
        assert_eq!(address_identity("").kind(), "empty");
        assert_eq!(address_identity("https:///path").kind(), "empty");
    }
}
