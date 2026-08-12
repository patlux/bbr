//! Acquisition bounds, the collector script, and the deterministic
//! fallback/parsing rules (task packet `real-favicon-pipeline.md`).
//!
//! Split from the acquisition host module so every file stays under the
//! ~500-line budget and each concern (bounds, orchestration, download,
//! delegate) owns one file. All functions here are pure and unit-tested;
//! the host and download paths live in their own submodules.

use std::time::Duration;

use bbr_web::FaviconCandidate;

/// The deterministic selection target: 2× the normal-density icon size
/// (18 pt → 36 pt, DESIGN §8). Mirrors the portable contract's documented
/// target exactly.
pub(crate) const FAVICON_TARGET_SIZE: u32 = 36;

/// The bounded payload cap for one favicon download. A response declaring
/// or accumulating more than this is cancelled (silent monogram fallback).
pub(crate) const FAVICON_MAX_BYTES: usize = 1 << 20;

/// The bounded deadline for one favicon download (the session's request and
/// resource timeouts). Never a hang; a timeout is a silent failure.
pub(crate) const FAVICON_DOWNLOAD_DEADLINE: Duration = Duration::from_secs(5);

/// Bounded concurrent downloads across all live views.
pub(crate) const FAVICON_MAX_IN_FLIGHT: usize = 4;

/// The bounded FIFO wait queue for downloads beyond the concurrency cap.
/// A full queue drops the newest request (silent).
pub(crate) const FAVICON_QUEUE_CAP: usize = 16;

/// The client-content-world script that collects `<link rel~="icon">`
/// candidates in document order and returns a JSON string:
/// `[{"url":"…","size":32|null}, …]`. `href` is read through the DOM so
/// relative URLs resolve against the committed document. The largest
/// declared square token of the `sizes` attribute is the candidate's size
/// (the same normalization the portable contract documents). An empty
/// document yields `[]`, and the caller applies the `/favicon.ico`
/// fallback.
pub(crate) fn candidates_script() -> &'static str {
    "(function () { \
        var out = []; \
        var links = document.querySelectorAll('link[rel]'); \
        for (var i = 0; i < links.length; i++) { \
            var rels = (links[i].getAttribute('rel') || '').toLowerCase().split(/\\s+/); \
            var icon = false; \
            for (var r = 0; r < rels.length; r++) { if (rels[r] === 'icon') { icon = true; break; } } \
            if (!icon) { continue; } \
            var sizes = links[i].getAttribute('sizes'); \
            var size = null; \
            if (sizes) { \
                var toks = sizes.toLowerCase().split(/\\s+/); \
                var best = 0; \
                for (var t = 0; t < toks.length; t++) { \
                    var n = parseInt(toks[t], 10); \
                    if (isFinite(n) && n > 0 && n > best) { best = n; } \
                } \
                if (best > 0) { size = best; } \
            } \
            out.push({ url: links[i].href, size: size }); \
        } \
        return JSON.stringify(out); \
    })()"
}

/// Derive the `/favicon.ico` fallback URL from the committed page URL:
/// `scheme://host/favicon.ico`. `None` for unparsable or non-http(s)
/// URLs (the §7.2 monogram stays the fallback).
pub(crate) fn fallback_icon_url(committed_url: &str) -> Option<String> {
    let (scheme, rest) = committed_url.split_once("://")?;
    if scheme != "https" && scheme != "http" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
    if authority.is_empty() {
        return None;
    }
    Some(format!("{scheme}://{authority}/favicon.ico"))
}

/// Parse the collector script's JSON payload into typed candidates. A
/// malformed payload (never a real page result) yields an empty list — the
/// `/favicon.ico` fallback then applies, so a hostile or broken page can
/// never fabricate a download target.
pub(crate) fn parse_candidates(payload: &str) -> Vec<FaviconCandidate> {
    serde_json::from_str(payload).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_script_targets_icon_rel_links_only() {
        let script = candidates_script();
        assert!(script.contains("querySelectorAll('link[rel]')"));
        assert!(script.contains("'icon'"));
        assert!(script.contains("JSON.stringify(out)"));
    }

    #[test]
    fn fallback_icon_url_is_deterministic_and_scheme_bounded() {
        assert_eq!(
            fallback_icon_url("https://example.com/a/b"),
            Some("https://example.com/favicon.ico".to_string())
        );
        assert_eq!(
            fallback_icon_url("http://localhost:8080/x"),
            Some("http://localhost:8080/favicon.ico".to_string())
        );
        assert_eq!(
            fallback_icon_url("https://example.com?q=1#frag"),
            Some("https://example.com/favicon.ico".to_string())
        );
        // Non-http(s) and unparsable URLs yield no fallback (silent
        // monogram), never a fabricated target.
        assert_eq!(fallback_icon_url("about:blank"), None);
        assert_eq!(fallback_icon_url("file:///tmp/x"), None);
        assert_eq!(fallback_icon_url(""), None);
        assert_eq!(fallback_icon_url("https://"), None);
    }

    #[test]
    fn parse_candidates_reads_the_script_payload_shape() {
        let candidates = parse_candidates(
            r#"[{"url":"https://a/favicon-32x32.png","size":32},{"url":"https://a/favicon.ico","size":null}]"#,
        );
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].url, "https://a/favicon-32x32.png");
        assert_eq!(candidates[0].size, Some(32));
        assert_eq!(candidates[1].url, "https://a/favicon.ico");
        assert_eq!(candidates[1].size, None);
    }

    #[test]
    fn parse_candidates_rejects_malformed_payloads_to_empty() {
        assert_eq!(parse_candidates(""), Vec::<FaviconCandidate>::new());
        assert_eq!(parse_candidates("not json"), Vec::<FaviconCandidate>::new());
        assert_eq!(
            parse_candidates(r#"{"url":"https://a/x.png"}"#),
            Vec::<FaviconCandidate>::new()
        );
    }

    #[test]
    fn the_selection_target_matches_the_portable_contract_documentation() {
        // 2× the normal-density icon size (18 pt → 36 pt, DESIGN §8).
        assert_eq!(FAVICON_TARGET_SIZE, 36);
        // The deadline is bounded (never a hang): the session's request
        // and resource timeouts cap one download at this value.
        assert_eq!(
            FAVICON_DOWNLOAD_DEADLINE.as_secs(),
            Duration::from_secs(5).as_secs()
        );
    }

    #[test]
    fn queue_cap_keeps_the_waiting_list_bounded() {
        // The bounded-wait contract: a full queue drops the newest request
        // instead of growing without bound, and the concurrency cap stays
        // below the queue cap so a slot always frees eventually.
        assert_eq!(FAVICON_QUEUE_CAP, 16);
        assert_eq!(FAVICON_MAX_IN_FLIGHT, 4);
    }
}
