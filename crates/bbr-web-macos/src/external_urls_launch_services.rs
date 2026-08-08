//! LaunchServices default-browser request (macOS only).
//!
//! The **only** place BBR claims default-handler status for its URL schemes.
//! Registration is a user action: the app never calls
//! `LSSetDefaultHandlerForURLScheme` at startup or on link delivery — only
//! the command-palette action "Set as Default Browser" routes here.
//!
//! The schemes are exactly [`DECLARED_URL_SCHEMES`]: the supported web
//! schemes of the merged contract. Claiming any other scheme would make the
//! OS dispatch links BBR is contractually bound to reject.

use std::ffi::c_void;
use std::fmt;

use objc2_core_foundation::{CFBundle, CFString};

use crate::external_urls_adapter::DECLARED_URL_SCHEMES;

/// `OSStatus` success constant of LaunchServices.
const NO_ERROR: i32 = 0;

/// Why requesting default-browser status failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultBrowserError {
    /// The app has no main bundle (a raw binary, not a `.app`); there is no
    /// bundle identifier to claim handler status for.
    NotBundled,
    /// The main bundle has no `CFBundleIdentifier` in its `Info.plist`.
    NoBundleIdentifier,
    /// `LSSetDefaultHandlerForURLScheme` returned a non-zero status.
    LaunchServices { scheme: &'static str, status: i32 },
}

impl fmt::Display for DefaultBrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBundled => write!(f, "app is not bundled (no CFBundle main bundle)"),
            Self::NoBundleIdentifier => {
                write!(f, "main bundle declares no CFBundleIdentifier")
            }
            Self::LaunchServices { scheme, status } => {
                write!(
                    f,
                    "LSSetDefaultHandlerForURLScheme({scheme}) failed with status {status}"
                )
            }
        }
    }
}

impl std::error::Error for DefaultBrowserError {}

// SAFETY: LaunchServices' `LSSetDefaultHandlerForURLScheme` takes two
// `CFStringRef`s and returns an `OSStatus` (i32). The framework symbol is
// resolved from ApplicationServices at link time.
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn LSSetDefaultHandlerForURLScheme(scheme: *const c_void, bundle_id: *const c_void) -> i32;
}

/// Request default-handler status for every declared scheme (`http`,
/// `https`), claimed under the main bundle's identifier.
///
/// Caller-visible result only: `Ok(())` when every declared scheme was
/// claimed, `Err` with the exact failing scheme/status otherwise. Never
/// called automatically by the app.
pub fn request_default_browser() -> Result<(), DefaultBrowserError> {
    let Some(bundle) = CFBundle::main_bundle() else {
        return Err(DefaultBrowserError::NotBundled);
    };
    let Some(identifier) = bundle.identifier() else {
        return Err(DefaultBrowserError::NoBundleIdentifier);
    };
    for scheme in DECLARED_URL_SCHEMES {
        let scheme_str = CFString::from_str(scheme);
        // SAFETY: both arguments are live CFString objects (the scheme is
        // created above and alive for the call; the identifier is the
        // bundle's own retained CFString); the call does not retain them.
        let status = unsafe {
            LSSetDefaultHandlerForURLScheme(
                &*scheme_str as *const _ as *const c_void,
                &*identifier as *const _ as *const c_void,
            )
        };
        if status != NO_ERROR {
            return Err(DefaultBrowserError::LaunchServices { scheme, status });
        }
    }
    Ok(())
}
