//! One live native web session: the WKWebView, its retained delegate, the
//! dark provisional-navigation cover, profile identity, and visibility kept
//! together so create/show/action/focus/destroy cannot accidentally target a
//! different tab. Also the deterministic blank-page HTML and the URL-loading
//! helper the engine uses. Split from `webview.rs` so the engine module
//! stays under the ~500-line module budget.

use objc2::rc::Retained;
use objc2::ClassType;
use objc2_app_kit::NSView;
use objc2_foundation::{NSString, NSURLRequest, NSURL};
use objc2_web_kit::WKWebView;

use bbr_web::{ProfileId, WebEvent, WebViewId};

use crate::delegate::{self, EventSink, WebViewDelegate};

/// The dark blank-page HTML the engine loads for `about:blank`, matching
/// DESIGN's dark `material:glass` behind page content so a blank session
/// never exposes WebKit's default white plane.
pub(crate) const DARK_BLANK_PAGE: &str = r#"<!doctype html>
<html style="color-scheme: dark; background: #151719">
<head><meta charset="utf-8"></head>
<body style="margin: 0; background: #151719; color: #eceef1"></body>
</html>"#;

/// All native objects and state belonging to one live web session.
pub(crate) struct NativeWebSession {
    pub(crate) profile: ProfileId,
    pub(crate) web_view: Retained<WKWebView>,
    /// Dark sibling view masking WebKit's white provisional-navigation plane.
    pub(crate) navigation_cover: Retained<NSView>,
    /// WebKit does not retain navigation/UI delegates.
    pub(crate) delegate: Retained<WebViewDelegate>,
    pub(crate) visible: bool,
}

impl NativeWebSession {
    /// Remove KVO before releasing the observed view, then detach it from AppKit.
    pub(crate) fn teardown(self) {
        unsafe {
            delegate::unobserve_title(&self.web_view, self.delegate.as_super());
        }
        self.navigation_cover.removeFromSuperview();
        self.web_view.removeFromSuperview();
    }
}

/// Load `url` into `web_view`; `about:blank` renders the deterministic dark
/// blank page instead of a network load.
pub(crate) fn load_url(web_view: &WKWebView, url: &str) {
    if url == "about:blank" {
        let html = NSString::from_str(DARK_BLANK_PAGE);
        unsafe {
            web_view.loadHTMLString_baseURL(&html, None);
        }
        return;
    }
    let ns_url = NSString::from_str(url);
    let Some(url_obj) = NSURL::URLWithString(&ns_url) else {
        log::warn!("cannot parse URL: {url}");
        return;
    };
    let request = NSURLRequest::requestWithURL(&url_obj);
    unsafe {
        web_view.loadRequest(&request);
    }
}

/// Stable UUID namespace for a persistent WebKit data store.
pub(crate) fn profile_store_uuid(profile: ProfileId) -> String {
    let upper = (profile.0 >> 48) as u16;
    let lower = profile.0 & 0x0000_ffff_ffff_ffff;
    format!("42425200-0000-4000-{upper:04x}-{lower:012x}")
}

/// Publish the current URL/title/history/loading state of a live session
/// through the event sink (used when the engine needs to re-emit state, e.g.
/// after a Stop).
pub(crate) fn emit_state(sink: &EventSink, session: &NativeWebSession, id: WebViewId) {
    let web_view = &session.web_view;
    let url = unsafe { web_view.URL() }
        .and_then(|u| u.absoluteString())
        .map(|s| s.to_string());
    let title = unsafe { web_view.title() }.map(|s| s.to_string());
    let can_back = unsafe { web_view.canGoBack() };
    let can_forward = unsafe { web_view.canGoForward() };
    if let Some(sink) = sink.borrow_mut().as_mut() {
        if let Some(url) = url {
            sink(WebEvent::UrlChanged(id, url));
        }
        if let Some(title) = title {
            sink(WebEvent::TitleChanged(id, title));
        }
        sink(WebEvent::CanGoBackChanged(id, can_back));
        sink(WebEvent::CanGoForwardChanged(id, can_forward));
        sink(WebEvent::LoadingChanged(id, false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_store_uuid_is_stable_and_unique() {
        assert_eq!(
            profile_store_uuid(ProfileId(1)),
            "42425200-0000-4000-0000-000000000001"
        );
        assert_eq!(
            profile_store_uuid(ProfileId(u64::MAX)),
            "42425200-0000-4000-ffff-ffffffffffff"
        );
        assert_ne!(
            profile_store_uuid(ProfileId(1)),
            profile_store_uuid(ProfileId(2))
        );
    }
}
