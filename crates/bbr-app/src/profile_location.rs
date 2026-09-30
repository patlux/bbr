//! Where BBR keeps its profile directory when no `--bbr-profile-dir` is
//! given: `~/Library/Application Support/<bundle id>`.
//!
//! The release app (`dev.patwoz.bbr`) and the dev bundle `just run` builds
//! (`dev.patwoz.bbr.dev`) keep separate directories, so a development
//! build never opens the store of the installed app. An unbundled
//! executable has no bundle identifier and uses the id of its build: dev
//! for a debug build, release otherwise.

use std::path::{Path, PathBuf};

/// The release app's bundle identifier.
pub(crate) const RELEASE_BUNDLE_ID: &str = "dev.patwoz.bbr";
/// The dev bundle's identifier (`scripts/package/dev_app.py`).
pub(crate) const DEV_BUNDLE_ID: &str = "dev.patwoz.bbr.dev";

/// The profile directory under `home` for the running bundle.
pub(crate) fn default_profile_dir(home: &Path, bundle_id: Option<&str>) -> PathBuf {
    let fallback = if cfg!(debug_assertions) {
        DEV_BUNDLE_ID
    } else {
        RELEASE_BUNDLE_ID
    };
    let id = bundle_id
        .filter(|id| is_directory_name(id))
        .unwrap_or(fallback);
    home.join("Library").join("Application Support").join(id)
}

/// The profile directory of this process: `$HOME` (the user's home; the
/// E2E harness points it at an isolated directory) and the main bundle.
pub(crate) fn current_default_profile_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|home| !home.is_empty())?;
    #[cfg(target_os = "macos")]
    let bundle_id = bbr_web_macos::app_identity::bundle_identifier();
    #[cfg(not(target_os = "macos"))]
    let bundle_id: Option<String> = None;
    Some(default_profile_dir(Path::new(&home), bundle_id.as_deref()))
}

/// A bundle identifier usable as one directory name.
fn is_directory_name(id: &str) -> bool {
    !id.is_empty() && id != "." && id != ".." && !id.contains('/') && !id.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundles_keep_separate_directories_under_application_support() {
        let home = Path::new("/Users/someone");
        assert_eq!(
            default_profile_dir(home, Some(RELEASE_BUNDLE_ID)),
            PathBuf::from("/Users/someone/Library/Application Support/dev.patwoz.bbr")
        );
        assert_eq!(
            default_profile_dir(home, Some(DEV_BUNDLE_ID)),
            PathBuf::from("/Users/someone/Library/Application Support/dev.patwoz.bbr.dev")
        );
    }

    #[test]
    fn an_unbundled_or_unusable_id_takes_the_build_id() {
        let home = Path::new("/h");
        let build = if cfg!(debug_assertions) {
            DEV_BUNDLE_ID
        } else {
            RELEASE_BUNDLE_ID
        };
        for id in [None, Some(""), Some(".."), Some("a/b")] {
            assert_eq!(
                default_profile_dir(home, id),
                Path::new("/h/Library/Application Support").join(build),
                "{id:?}"
            );
        }
    }
}
