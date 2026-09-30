//! The running app's bundle identity.

use objc2_foundation::NSBundle;

/// The main bundle's `CFBundleIdentifier`: `dev.patwoz.bbr` for the release
/// app, `dev.patwoz.bbr.dev` for the dev bundle `just run` builds, and
/// `None` for an unbundled executable (the E2E harness's binary).
pub fn bundle_identifier() -> Option<String> {
    NSBundle::mainBundle()
        .bundleIdentifier()
        .map(|identifier| identifier.to_string())
}
