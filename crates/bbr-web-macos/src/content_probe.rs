//! Real WebKit content evidence for the E2E gateway (testkit-only usage).
//!
//! The product's visibility flags, geometry, and session bookkeeping can be
//! perfectly green while the user stares at a blank white content area (the
//! schemeless Cmd-T destination bug): the view is shown, framed, and
//! reported visible — but no document ever committed. This module reads
//! what WebKit *actually* has, never our own accounting:
//!
//! - the committed document, read in BBR's own client content world
//!   ([`WKContentWorld::defaultClientWorld`], the same isolation the find
//!   strategy uses — never the page's main content world): `readyState`,
//!   `document.URL`, `document.title`, body presence, and the computed root/body
//!   background colors;
//! - the native `WKWebView` `url`/`title` properties (the URL reflects a
//!   provisional navigation immediately, so "a navigation was accepted" is
//!   observable even before the document commits — the key asymmetry that
//!   catches the schemeless-URL bug, which WebKit rejects before any
//!   provisional navigation starts).
//!
//! The probe is non-destructive: it only reads the DOM and native state,
//! installs no globals, and mutates nothing.
//!
//! # Safety
//!
//! `evaluateJavaScript` completions are delivered on the main thread, so the
//! completion block only ever runs there. The block captures a retained
//! clone of the web view and the caller's `on_result` sink; the view is
//! retained only for the bounded lifetime of the pending completion, so a
//! concurrent `destroy_web_view` can never leave the block holding a
//! deallocated view, and after the completion fires both are released.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_foundation::{NSError, NSString};
use objc2_web_kit::{WKContentWorld, WKWebView};

/// One point-in-time content observation of a live web view.
///
/// The DOM fields come from the client content world; the native fields are
/// the `WKWebView` `url`/`title` properties sampled at the same completion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentProbe {
    /// `document.readyState` ("loading" | "interactive" | "complete").
    pub ready_state: String,
    /// `document.URL` — the committed document's URL (empty on probe
    /// failure; "about:blank" for a fresh/blank view).
    pub dom_url: String,
    /// `document.title`.
    pub dom_title: String,
    /// A document body exists in the DOM.
    pub has_body: bool,
    /// Computed `background-color` of the document root.
    pub root_background_color: String,
    /// Computed `background-color` of the body, or empty when no body exists.
    pub body_background_color: String,
    /// The native `WKWebView.url` (provisional or committed; `None` when no
    /// navigation has been accepted at all).
    pub native_url: Option<String>,
    /// The native `WKWebView.title`.
    pub native_title: Option<String>,
}

/// The field separator between the six DOM values the probe script returns
/// (a unit separator never legitimately appears inside a URL or title).
const FIELD_SEPARATOR: char = '\u{1f}';

/// Issue one content probe against `web_view`. `on_result` runs on the main
/// thread with the merged DOM + native observation. A JS failure (crashed
/// web process, no document) reports the DOM fields as absent while the
/// native fields still land, so the evidence never fabricates content.
pub fn probe_web_view(web_view: &Retained<WKWebView>, on_result: impl Fn(ContentProbe) + 'static) {
    assert_main_thread();
    let mtm = MainThreadMarker::new().expect("content probe on the main thread");
    let script = "document.readyState + \"\\u001f\" + document.URL + \"\\u001f\" + \
        document.title + \"\\u001f\" + (document.body ? \"1\" : \"0\") + \"\\u001f\" + \
        getComputedStyle(document.documentElement).backgroundColor + \"\\u001f\" + \
        (document.body ? getComputedStyle(document.body).backgroundColor : \"\")";
    let ns_script = NSString::from_str(script);
    // BBR's own client content world, never the page's main content world:
    // the probe reads the shared DOM but cannot collide with or be observed
    // by page scripts (the same isolation the find strategy uses).
    let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
    let web_view = web_view.clone();
    let view_for_block = web_view.clone();
    let on_result = Box::new(on_result);
    let block = block2::RcBlock::new(move |result: *mut AnyObject, error: *mut NSError| {
        let probe = content_probe_from_result(&view_for_block, result, error);
        on_result(probe);
    });
    // SAFETY: main thread; `web_view` is retained and alive for the duration
    // of this call; `world` is the process-wide default client world
    // (retained by WebKit); the completion block only touches `web_view`
    // (retained clone) and `on_result`.
    unsafe {
        web_view.evaluateJavaScript_inFrame_inContentWorld_completionHandler(
            &ns_script,
            None,
            &world,
            Some(&block),
        );
    }
}

/// Merge the JS completion (DOM fields) with the native url/title read at
/// completion time.
fn content_probe_from_result(
    web_view: &WKWebView,
    result: *mut AnyObject,
    error: *mut NSError,
) -> ContentProbe {
    let mut probe = ContentProbe {
        native_url: native_url(web_view),
        native_title: native_title(web_view),
        ..ContentProbe::default()
    };
    if !error.is_null() {
        log::warn!("content probe: evaluateJavaScript failed with an error");
        return probe;
    }
    // SAFETY: `result` is the JS return value (an NSString for our string
    // script) or null; downcast_ref verifies the class before dereferencing.
    let Some(object) = (unsafe { result.as_ref() }) else {
        return probe;
    };
    let Some(ns_string) = object.downcast_ref::<NSString>() else {
        log::warn!("content probe: evaluateJavaScript returned a non-string value");
        return probe;
    };
    apply_dom_fields(&mut probe, &ns_string.to_string());
    probe
}

/// Fill the DOM fields from the script's six unit-separator-delimited
/// values. A payload that is not exactly six fields (a malformed script
/// result, never a real page value) leaves the DOM fields absent — the
/// probe never fabricates content.
fn apply_dom_fields(probe: &mut ContentProbe, payload: &str) {
    let fields: Vec<&str> = payload.split(FIELD_SEPARATOR).collect();
    if fields.len() != 6 {
        log::warn!("content probe: malformed payload ({} fields)", fields.len());
        return;
    }
    probe.ready_state = fields[0].to_owned();
    probe.dom_url = fields[1].to_owned();
    probe.dom_title = fields[2].to_owned();
    probe.has_body = fields[3] == "1";
    probe.root_background_color = fields[4].to_owned();
    probe.body_background_color = fields[5].to_owned();
}

fn native_url(web_view: &WKWebView) -> Option<String> {
    // SAFETY: main thread, valid view.
    (unsafe { web_view.URL() })
        .and_then(|url| url.absoluteString())
        .map(|s| s.to_string())
}

fn native_title(web_view: &WKWebView) -> Option<String> {
    // SAFETY: main thread, valid view.
    (unsafe { web_view.title() }).map(|s| s.to_string())
}

fn assert_main_thread() {
    debug_assert!(
        MainThreadMarker::new().is_some(),
        "content probe used off the main thread"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom_fields_parse_four_unit_separated_values() {
        let mut probe = ContentProbe::default();
        apply_dom_fields(
            &mut probe,
            "complete\u{1f}https://example.com/\u{1f}Example\u{1f}1\u{1f}rgb(21, 23, 25)\u{1f}rgb(21, 23, 25)",
        );
        assert_eq!(probe.ready_state, "complete");
        assert_eq!(probe.dom_url, "https://example.com/");
        assert_eq!(probe.dom_title, "Example");
        assert!(probe.has_body);
        assert_eq!(probe.root_background_color, "rgb(21, 23, 25)");
        assert_eq!(probe.body_background_color, "rgb(21, 23, 25)");
    }

    #[test]
    fn dom_fields_parse_blank_view() {
        let mut probe = ContentProbe::default();
        apply_dom_fields(
            &mut probe,
            "complete\u{1f}about:blank\u{1f}\u{1f}1\u{1f}rgb(21, 23, 25)\u{1f}rgb(21, 23, 25)",
        );
        assert_eq!(probe.dom_url, "about:blank");
        assert_eq!(probe.dom_title, "");
        assert!(probe.has_body);
        assert_eq!(probe.root_background_color, "rgb(21, 23, 25)");
        assert_eq!(probe.body_background_color, "rgb(21, 23, 25)");
    }

    #[test]
    fn malformed_payload_keeps_dom_fields_absent() {
        let mut probe = ContentProbe::default();
        apply_dom_fields(&mut probe, "only-one-field");
        assert_eq!(probe.ready_state, "");
        assert_eq!(probe.dom_url, "");
    }
}
