use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SANDBOX: AtomicU64 = AtomicU64::new(1);

struct Sandbox {
    dir: std::path::PathBuf,
}
impl Sandbox {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "bbr-fixtures-test-{}-{}",
            std::process::id(),
            NEXT_SANDBOX.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Sandbox { dir }
    }
    fn write(&self, rel: &str, content: &str) {
        let p = self.dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn serves_static_files_and_counts_hits() {
    let sb = Sandbox::new();
    sb.write("page1.html", "<html><title>One</title></html>");
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    let (code, body) = get(&server.url("/page1.html")).unwrap();
    assert_eq!(code, 200);
    let body = String::from_utf8(body).unwrap();
    assert!(body.contains("<title>One</title>"));
    assert!(body.contains("data-bbr-fixture-theme"));
    assert!(body.contains("background: #151719"));
    assert_eq!(server.hits("/page1.html"), 1);
    let (_code2, _) = get(&server.url("/page1.html")).unwrap();
    assert_eq!(server.hits("/page1.html"), 2);
}

#[test]
fn missing_files_are_404_and_traversal_is_blocked() {
    let sb = Sandbox::new();
    sb.write("ok.html", "ok");
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    let (code, _) = get(&server.url("/nope.html")).unwrap();
    assert_eq!(code, 404);
    let (code2, _) = get(&server.url("/../../etc/hosts")).unwrap();
    assert_eq!(code2, 404);
    let (code3, _) = get(&server.url("/")).unwrap();
    assert_eq!(code3, 404);
}

/// Read a URL's complete raw response bytes (status line, headers, and
/// body) with no HTTP parsing, so a test can pin the exact wire shape
/// the engine observes.
fn raw_get(url: &str) -> std::io::Result<Vec<u8>> {
    let rest = url.trim_start_matches("http://");
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let mut stream = TcpStream::connect(authority)?;
    let req = format!("GET {path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes())?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[test]
fn error_fixtures_are_deterministic() {
    let sb = Sandbox::new();
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    // The 500 route answers a fixed server error.
    let (code, body) = get(&server.url("/__fx/error/server-error")).unwrap();
    assert_eq!(code, 500);
    assert!(String::from_utf8(body).unwrap().contains("boom"));
    assert_eq!(server.hits("/__fx/error/server-error"), 1);
    // The redirect-loop route keeps redirecting to itself (302 → same
    // location) — a client that follows redirects never terminates.
    let (code2, _) = get(&server.url("/__fx/error/redirect-loop")).unwrap();
    assert_eq!(code2, 302);
    let body2 = get(&server.url("/__fx/error/redirect-loop")).unwrap().1;
    assert!(body2.is_empty());
    assert_eq!(server.hits("/__fx/error/redirect-loop"), 2);
    // The truncated-error route serves a 500 chunked whose first chunk
    // declares 64 KiB (hex `10000`) but delivers only 4 bytes before
    // the connection closes: the raw client observes the premature
    // close in the middle of the chunk — the chunked body never reaches
    // its terminating zero chunk, so the transfer is observably
    // incomplete. This framing (not a `Content-Length` truncation,
    // which WebKit accepts as an empty completion) is what makes the
    // engine fail the navigation; the wire is pinned byte-for-byte so
    // the shape cannot drift.
    let (code3, body3) = get(&server.url("/__fx/error/truncated-error")).unwrap();
    assert_eq!(code3, 500);
    assert_eq!(body3, b"10000\r\nboom");
    let raw3 = raw_get(&server.url("/__fx/error/truncated-error")).unwrap();
    assert!(
        raw3.starts_with(b"HTTP/1.1 500 Internal Server Error\r\n"),
        "the truncated fixture stays a 500: {raw3:?}"
    );
    assert!(
        String::from_utf8_lossy(&raw3).contains("Transfer-Encoding: chunked"),
        "the truncated fixture must use chunked framing: {raw3:?}"
    );
    assert!(
        raw3.windows(7).any(|w| w == b"10000\r\n"),
        "the truncated fixture declares the 64 KiB chunk size: {raw3:?}"
    );
    assert!(
        !raw3.windows(5).any(|w| w == b"0\r\n\r\n"),
        "the truncated fixture must never terminate the chunked body: {raw3:?}"
    );
    assert_eq!(
        raw3.windows(4).filter(|w| *w == b"boom").count(),
        1,
        "exactly the 4-byte chunk payload is delivered: {raw3:?}"
    );
    // The flaky route fails its first request (the same truncated
    // chunked 500) and succeeds afterwards (the recovery E2E retries
    // it): request 1 is the truncated chunked 500, request 2 is a real
    // page. Hit counting is pinned so the fail-then-succeed decision
    // cannot drift.
    let (code4, body4) = get(&server.url("/__fx/error/flaky")).unwrap();
    assert_eq!(code4, 500);
    assert_eq!(body4, b"10000\r\nboom");
    let (code5, body5) = get(&server.url("/__fx/error/flaky")).unwrap();
    assert_eq!(code5, 200);
    assert!(String::from_utf8(body5).unwrap().contains("recovered"));
    assert_eq!(
        server.hits("/__fx/error/flaky"),
        2,
        "request 1 fails, request 2 succeeds"
    );
}

#[test]
fn cookie_routes_reflect_only_the_supplied_cookie_jar() {
    let sb = Sandbox::new();
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    let (none_code, none) = get(&server.url("/__fx/cookie/read")).unwrap();
    assert_eq!(none_code, 200);
    assert!(String::from_utf8(none).unwrap().contains("Cookie none"));

    let (alpha_code, alpha) = get_with_headers(
        &server.url("/__fx/cookie/read"),
        &[("Cookie".into(), "bbr_profile=alpha".into())],
    )
    .unwrap();
    assert_eq!(alpha_code, 200);
    assert!(String::from_utf8(alpha).unwrap().contains("Cookie alpha"));
}

#[test]
fn dynamic_routes_are_deterministic() {
    let sb = Sandbox::new();
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    let (code, body) = get(&server.url("/__fx/hello")).unwrap();
    assert_eq!(code, 200);
    assert_eq!(body, b"hello from bbr-fixtures");
    let (code2, _) = get(&server.url("/__fx/robots")).unwrap();
    assert_eq!(code2, 200);
}

#[test]
fn hang_routes_serve_content_and_count_hits() {
    let sb = Sandbox::new();
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    for (path, marker) in [
        (hang::BUSY_PATH, "BBR Hang Busy"),
        (hang::HEAVY_PATH, "__bbrHeavyState"),
        (hang::DIALOG_PATH, "__bbrRunDialogs"),
        (hang::BEFOREUNLOAD_PATH, "__bbrBeforeUnloadReady"),
    ] {
        let (code, body) = get(&server.url(path)).unwrap();
        assert_eq!(code, 200, "{path}");
        let body = String::from_utf8(body).unwrap();
        assert!(body.contains(marker), "{path}: {marker}");
        assert!(body.contains("data-bbr-fixture-theme"), "{path}");
        assert_eq!(server.hits(path), 1, "{path}");
    }
}

#[test]
fn download_fixtures_are_deterministic() {
    let sb = Sandbox::new();
    let server = FixtureServer::start(sb.dir.clone()).unwrap();
    // Known length: 64 KiB payload, counted hits.
    let (code, body) = get(&server.url("/__fx/dl/known.bin")).unwrap();
    assert_eq!(code, 200);
    assert_eq!(body.len(), 65536);
    assert_eq!(server.hits("/__fx/dl/known.bin"), 1);
    let (_c2, body2) = get(&server.url("/__fx/dl/known.bin")).unwrap();
    assert_eq!(body, body2, "content-addressed: retries are byte-identical");
    // Unknown length: same payload, no Content-Length (body until close).
    let (code3, body3) = get(&server.url("/__fx/dl/unknown.bin")).unwrap();
    assert_eq!(code3, 200);
    assert_eq!(body3.len(), 65536);
    // Zero bytes succeeds with an empty body.
    let (code4, body4) = get(&server.url("/__fx/dl/zero.bin")).unwrap();
    assert_eq!(code4, 200);
    assert!(body4.is_empty());
    // Failure is an HTTP 500.
    let (code5, _) = get(&server.url("/__fx/dl/fail.bin")).unwrap();
    assert_eq!(code5, 500);
}
