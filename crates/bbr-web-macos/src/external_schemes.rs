//! Blueprint D13 (board M-12): links to schemes BBR does not render.
//!
//! The navigation delegate lets the page's frames load only the schemes a
//! browser renders; everything else goes to the app, which asks before
//! macOS hands the link to another app (`NSWorkspace`).

use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSFileManager, NSString, NSURL};

/// Whether WebKit renders `url` itself (web, blank, data, blob, file,
/// view-source). An empty URL (a fresh frame) counts as rendered.
pub fn renders_in_browser(url: &str) -> bool {
    let Some((scheme, _)) = url.split_once(':') else {
        return true;
    };
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "about" | "data" | "blob" | "file" | "javascript" | "view-source"
    )
}

/// The display name of the app macOS opens `url` with.
pub fn app_name_for(url: &str) -> Option<String> {
    let url = NSURL::URLWithString(&NSString::from_str(url))?;
    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.URLForApplicationToOpenURL(&url)?;
    let path = app.path()?;
    let name = NSFileManager::defaultManager()
        .displayNameAtPath(&path)
        .to_string();
    Some(name.strip_suffix(".app").unwrap_or(&name).to_string())
}

/// Hand `url` to its app. Returns whether macOS accepted it.
pub fn open(url: &str) -> bool {
    let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) else {
        return false;
    };
    NSWorkspace::sharedWorkspace().openURL(&url)
}

#[cfg(test)]
mod tests {
    use super::renders_in_browser;

    #[test]
    fn only_browser_schemes_render_in_the_page() {
        for url in [
            "https://a.example/",
            "http://127.0.0.1:1/",
            "about:blank",
            "data:text/plain,x",
            "blob:https://a/1",
            "",
        ] {
            assert!(renders_in_browser(url), "{url}");
        }
        for url in [
            "mailto:a@example.com",
            "zoommtg://zoom.us/join",
            "slack://open",
            "bbr-test-noapp:x",
        ] {
            assert!(!renders_in_browser(url), "{url}");
        }
    }
}
