//! # bbr-fixtures — deterministic loopback HTTP fixtures
//!
//! A tiny std-only HTTP/1.1 server bound to `127.0.0.1:<ephemeral>` that
//! serves static files from a fixture root plus a few dynamic, deterministic
//! routes. It exists so WebKit navigations in E2E tests hit stable,
//! content-addressed URLs (`http://127.0.0.1:<port>/<path>` — loopback is
//! ATS-exempt, so no TLS is needed).
//!
//! Keep this dependency-free: it runs in testers and the E2E harness alike.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// An HTTP/1.1 GET-only fixture server on 127.0.0.1.
pub struct FixtureServer {
    // Kept alive for the loopback binding; drop closes it.
    #[allow(dead_code)]
    listener: TcpListener,
    port: u16,
    root: PathBuf,
    hits: Arc<Mutex<HashMap<String, u32>>>,
    #[allow(dead_code)]
    thread: Option<JoinHandle<()>>,
}

impl FixtureServer {
    /// Start serving `root` on an ephemeral loopback port.
    ///
    /// # Errors
    ///
    /// Returns an error when the loopback bind fails.
    pub fn start(root: impl Into<PathBuf>) -> std::io::Result<FixtureServer> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let root = root.into();
        let hits = Arc::new(Mutex::new(HashMap::new()));
        let hits_for_thread = hits.clone();
        let root_for_thread = root.clone();
        let accept_listener = listener.try_clone()?;
        let thread = std::thread::Builder::new()
            .name("bbr-fixtures".into())
            .spawn(move || {
                for stream in accept_listener.incoming() {
                    match stream {
                        Ok(s) => {
                            let hits = hits_for_thread.clone();
                            let root = root_for_thread.clone();
                            std::thread::Builder::new()
                                .name("bbr-fixtures-conn".into())
                                .spawn(move || {
                                    let _ = handle_connection(s, &root, &hits);
                                })
                                .ok();
                        }
                        Err(_) => break,
                    }
                }
            })?
            .into();
        Ok(FixtureServer {
            listener,
            port,
            root,
            hits,
            thread,
        })
    }

    /// The base URL for fixtures, e.g. `http://127.0.0.1:54321`.
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// The full URL for a fixture path (must start with `/`).
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url(), path)
    }

    /// The bound port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Number of requests served for a given path.
    pub fn hits(&self, path: &str) -> u32 {
        self.hits.lock().unwrap().get(path).copied().unwrap_or(0)
    }

    /// The fixture root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn handle_connection(
    mut stream: TcpStream,
    root: &Path,
    hits: &Arc<Mutex<HashMap<String, u32>>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    // Read the request line only (we close each connection after one
    // response; keep-alive is unnecessary for tests).
    let _ = reader.read_line(&mut request_line)?;
    // Capture headers needed by stateful WebKit fixtures, then drain the rest.
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let Some((method, target)) = parse_request_line(&request_line) else {
        respond(&mut stream, 400, "Bad Request", b"bad request")?;
        return Ok(());
    };

    if method != "GET" {
        respond(&mut stream, 405, "Method Not Allowed", b"get only")?;
        return Ok(());
    }

    // Path-only target (strip any query string).
    let path = target.split('?').next().unwrap_or("/");
    // Deterministic download fixtures (the Phase-8 downloads E2E): the
    // download routes need response shapes the static-file path cannot
    // express — known/unknown content length, slow trickle, zero bytes,
    // and an HTTP failure — so they are dispatched before the generic
    // router. Content is content-addressed and stable per path.
    if let Some(response) = download_fixture(path) {
        record_hit(hits, path);
        response.respond(&mut stream)?;
        return Ok(());
    }
    // Deterministic error fixtures (the Phase-8 web-error E2E): the raw
    // wire shapes the generic router cannot express — an endless 302
    // redirect loop (the engine fails with `NSURLErrorHTTPTooManyRedirects`
    // → the contract's `Navigation/RedirectLoop`), a fixed 500 server
    // error, a truncated 500 (declared length never delivered → the engine
    // fails the navigation), and a flaky route that fails its first request
    // and succeeds afterwards (the recovery E2E retries it). All are
    // deterministic and content-addressed.
    if let Some(response) = error_fixture(path) {
        // The flaky route keys its behavior on the number of prior requests
        // (request 1 fails, later requests succeed), so the prior count is
        // read before the hit is recorded.
        let prior = { hits.lock().unwrap().get(path).copied().unwrap_or(0) };
        record_hit(hits, path);
        response.respond(&mut stream, prior)?;
        return Ok(());
    }
    match route(root, path, &headers)? {
        Route::File { data, content_type } => {
            let mut h = hits.lock().unwrap();
            *h.entry(path.to_string()).or_insert(0) += 1;
            drop(h);
            let data = style_fixture_html(data, &content_type);
            respond_with_headers(
                &mut stream,
                200,
                "OK",
                &[("Content-Type".into(), content_type)],
                &data,
            )?;
        }
        Route::Dynamic { body, headers } => {
            let mut h = hits.lock().unwrap();
            *h.entry(path.to_string()).or_insert(0) += 1;
            drop(h);
            let content_type = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                .map(|(_, value)| value.as_str())
                .unwrap_or_default();
            let body = style_fixture_html(body, content_type);
            respond_with_headers(&mut stream, 200, "OK", &headers, &body)?;
        }
        Route::NotFound => {
            respond(&mut stream, 404, "Not Found", b"not found")?;
        }
    }
    Ok(())
}

fn record_hit(hits: &Arc<Mutex<HashMap<String, u32>>>, path: &str) {
    let mut h = hits.lock().unwrap();
    *h.entry(path.to_string()).or_insert(0) += 1;
}

/// A deterministic download fixture response: the body writer owns the wire
/// shape (headers, chunking, delays) so WebKit observes exactly the content
/// contract each E2E scenario needs. The body is deterministic per path:
/// 64 KiB of a repeating pattern for `known.bin`, the same payload with no
/// `Content-Length` trickled in 8 KiB chunks for `unknown.bin` (indeterminate
/// `Running` stays observable), an 8×8 KiB trickle for `slow.bin`,
/// zero bytes for `zero.bin`, and a 500 for `fail.bin`. Retry reuses
/// `known.bin` (content-addressed, so a retry is byte-identical).
enum DownloadFixture {
    /// Known length: `Content-Length` header, one-shot body.
    Known(Vec<u8>),
    /// Unknown length: no `Content-Length`, body until connection close,
    /// trickled in 8 KiB chunks with a fixed 150 ms pause so the
    /// indeterminate `Running` state is observable to the polling E2E (a
    /// one-shot unknown-length body completes within milliseconds and the
    /// `Running` state never becomes observable).
    Unknown(Vec<u8>),
    /// Known length, trickled in 8 KiB chunks with a fixed 150 ms pause
    /// (deterministic slow progress and a stable cancellation window).
    Slow(Vec<u8>),
    /// Zero bytes: `Content-Length: 0` (a zero-byte download succeeds with
    /// a full bar).
    Zero,
    /// HTTP failure: 500 status (the engine fails the attempt).
    Fail,
}

/// The download fixture for `path`, or `None` when the path is not a
/// download route.
fn download_fixture(path: &str) -> Option<DownloadFixture> {
    match path {
        "/__fx/dl/known.bin" | "/__fx/dl/unknown.bin" | "/__fx/dl/slow.bin" => {
            // 64 KiB deterministic payload: 8 repetitions of an 8 KiB
            // pattern, so the bytes are content-addressed and stable.
            let chunk: Vec<u8> = (0u8..255u8).cycle().take(8192).collect();
            let body: Vec<u8> = chunk.repeat(8);
            assert_eq!(body.len(), 65536);
            match path {
                "/__fx/dl/known.bin" => Some(DownloadFixture::Known(body)),
                "/__fx/dl/unknown.bin" => Some(DownloadFixture::Unknown(body)),
                _ => Some(DownloadFixture::Slow(body)),
            }
        }
        "/__fx/dl/zero.bin" => Some(DownloadFixture::Zero),
        "/__fx/dl/fail.bin" => Some(DownloadFixture::Fail),
        _ => None,
    }
}

impl DownloadFixture {
    /// Write the fixture response to the stream.
    fn respond(self, stream: &mut TcpStream) -> std::io::Result<()> {
        const DISPOSITION: &str = "Content-Disposition: attachment; filename=\"download.bin\"\r\n";
        match self {
            DownloadFixture::Known(body) => {
                let mut head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{}",
                    body.len(),
                    DISPOSITION
                );
                head.push_str("Connection: close\r\n\r\n");
                stream.write_all(head.as_bytes())?;
                stream.write_all(&body)?;
                stream.flush()
            }
            DownloadFixture::Unknown(body) => {
                // No Content-Length: HTTP/1.1 signals the length by closing
                // the connection, so `expectedContentLength` is unknown and
                // the download runs indeterminate. Trickle in 8 KiB chunks
                // with a fixed pause so the indeterminate `Running` state is
                // observable (same window as `slow.bin`).
                let mut head = format!("HTTP/1.1 200 OK\r\n{}", DISPOSITION);
                head.push_str("Connection: close\r\n\r\n");
                stream.write_all(head.as_bytes())?;
                for chunk in body.chunks(8192) {
                    stream.write_all(chunk)?;
                    stream.flush()?;
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
                Ok(())
            }
            DownloadFixture::Slow(body) => {
                let mut head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{}",
                    body.len(),
                    DISPOSITION
                );
                head.push_str("Connection: close\r\n\r\n");
                stream.write_all(head.as_bytes())?;
                // Trickle in 8 KiB chunks with a fixed pause: deterministic
                // slow progress and a stable cancellation window.
                for chunk in body.chunks(8192) {
                    stream.write_all(chunk)?;
                    stream.flush()?;
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
                Ok(())
            }
            DownloadFixture::Zero => {
                // `DISPOSITION` already ends with `\r\n`, so appending one
                // `\r\n` completes the header block with the blank line —
                // exactly one, matching the Known/Slow wire shape (a doubled
                // blank line would leak a stray `\r\n` into the body).
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n{}\r\n",
                    DISPOSITION
                );
                stream.write_all(head.as_bytes())?;
                stream.flush()
            }
            DownloadFixture::Fail => respond(stream, 500, "Internal Server Error", b"boom"),
        }
    }
}

/// A deterministic error-surface fixture response: the raw wire shape the
/// engine turns into a navigation failure.
#[derive(Debug, Clone, Copy)]
enum ErrorFixture {
    /// Answer every request with a 302 to the same path, forever: WebKit
    /// aborts the navigation with `NSURLErrorHTTPTooManyRedirects`.
    RedirectLoop,
    /// A fixed 500 with a small HTML body (deterministic server error).
    ServerError,
    /// A 500 whose declared `Content-Length` is never satisfied (the
    /// connection closes early): the engine fails the navigation with an
    /// NSURLError HTTP-domain code (bad server response / connection lost).
    TruncatedError,
    /// Fail the first request (a truncated 500, so the engine reports a
    /// navigation failure) and answer every later request with a real page:
    /// the recovery E2E retries this route and must see the surface clear
    /// when the retried navigation succeeds.
    FlakyError,
}

/// The error fixture for `path`, or `None` when the path is not an error
/// route.
fn error_fixture(path: &str) -> Option<ErrorFixture> {
    match path {
        "/__fx/error/redirect-loop" => Some(ErrorFixture::RedirectLoop),
        "/__fx/error/server-error" => Some(ErrorFixture::ServerError),
        "/__fx/error/truncated-error" => Some(ErrorFixture::TruncatedError),
        "/__fx/error/flaky" => Some(ErrorFixture::FlakyError),
        _ => None,
    }
}

impl ErrorFixture {
    /// Write the fixture response to the stream. `prior` is the number of
    /// earlier requests to this route (drives the flaky route's fail-then-
    /// succeed shape; ignored by the fixed shapes).
    fn respond(self, stream: &mut TcpStream, prior: u32) -> std::io::Result<()> {
        match self {
            ErrorFixture::RedirectLoop => {
                let head =
                    "HTTP/1.1 302 Found\r\nLocation: /__fx/error/redirect-loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                stream.write_all(head.as_bytes())?;
                stream.flush()
            }
            ErrorFixture::ServerError => respond(
                stream,
                500,
                "Internal Server Error",
                b"<!doctype html><html><head><title>Server Error</title></head><body>boom</body></html>",
            ),
            ErrorFixture::TruncatedError => {
                // 500 status, declares 64 KiB, delivers 4 bytes, closes.
                let head =
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 65536\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n";
                stream.write_all(head.as_bytes())?;
                stream.write_all(b"boom")?;
                stream.flush()
            }
            ErrorFixture::FlakyError => {
                if prior == 0 {
                    // First request: fail like the truncated route.
                    let head =
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 65536\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n";
                    stream.write_all(head.as_bytes())?;
                    stream.write_all(b"boom")?;
                    stream.flush()
                } else {
                    // Later requests: a real deterministic page.
                    respond(
                        stream,
                        200,
                        "OK",
                        b"<!doctype html><html><head><title>BBR Flaky Recovery</title></head><body>recovered</body></html>",
                    )
                }
            }
        }
    }
}

enum Route {
    File {
        data: Vec<u8>,
        content_type: String,
    },
    Dynamic {
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    },
    NotFound,
}

fn route(
    root: &Path,
    path: &str,
    request_headers: &HashMap<String, String>,
) -> std::io::Result<Route> {
    // Dynamic deterministic routes.
    match path {
        "/__fx/hello" => {
            return Ok(Route::Dynamic {
                body: b"hello from bbr-fixtures".to_vec(),
                headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())],
            });
        }
        "/__fx/robots" => {
            // Content-addressed: length always the same, content stable.
            return Ok(Route::Dynamic {
                body: b"user-agent: *\ndisallow: /\n".to_vec(),
                headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())],
            });
        }
        "/__fx/slow" => {
            // Deterministic slow page: the response is delayed server-side so
            // the shell's `loading` row state is observable in E2E without
            // test sleeps. The delay is fixed and bounded.
            std::thread::sleep(std::time::Duration::from_millis(900));
            return Ok(Route::Dynamic {
                body: br#"<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /><title>BBR Slow Page</title></head>
  <body><main><h1>Slow fixture</h1></main></body>
</html>"#
                    .to_vec(),
                headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
            });
        }
        "/__fx/cookie/set-alpha" => {
            return Ok(cookie_set_route("alpha"));
        }
        "/__fx/cookie/set-beta" => {
            return Ok(cookie_set_route("beta"));
        }
        "/__fx/cookie/read" => {
            let value = cookie_value(request_headers.get("cookie"), "bbr_profile")
                .unwrap_or_else(|| "none".into());
            return Ok(html_route(&format!(
                "<html><title>Cookie {value}</title><body>{value}</body></html>"
            )));
        }
        // Deterministic error-surface fixtures (the Phase-8 web-error E2E):
        // HTTP-level failures WebKit turns into navigation errors.
        // `/__fx/error/redirect-loop` answers every request with a 302 to
        // itself, so the engine fails the navigation with
        // `NSURLErrorHTTPTooManyRedirects` (the contract's
        // `Navigation/RedirectLoop`). `/__fx/error/server-error` answers a
        // deterministic 500 with a small body (what WebKit does with it is
        // observed by the E2E).
        _ => {}
    }

    let rel = Path::new(path.trim_start_matches('/'));
    if rel.as_os_str().is_empty()
        || rel
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Ok(Route::NotFound);
    }
    // Only normal relative components are accepted, so the candidate cannot
    // escape the fixture root through `..`, a root, or a platform prefix.
    let candidate = root.join(rel);
    if !candidate.is_file() {
        return Ok(Route::NotFound);
    }
    let data = fs::read(&candidate)?;
    let content_type = content_type_for(&candidate);
    Ok(Route::File { data, content_type })
}

const DARK_FIXTURE_STYLE: &str = r#"<style data-bbr-fixture-theme>
  :root { color-scheme: dark; background: #151719; color: #eceef1; }
  body { background: #151719; color: #eceef1; font-family: system-ui, sans-serif; }
  a { color: #3df0ce; }
</style>"#;

fn style_fixture_html(data: Vec<u8>, content_type: &str) -> Vec<u8> {
    if !content_type.starts_with("text/html") {
        return data;
    }
    let mut html = match String::from_utf8(data) {
        Ok(html) => html,
        Err(error) => return error.into_bytes(),
    };
    if html.contains("data-bbr-fixture-theme") {
        return html.into_bytes();
    }
    if let Some(index) = html.find("</head>") {
        html.insert_str(index, DARK_FIXTURE_STYLE);
    } else if let Some(index) = html.find('>') {
        html.insert_str(index + 1, DARK_FIXTURE_STYLE);
    } else {
        html.insert_str(0, DARK_FIXTURE_STYLE);
    }
    html.into_bytes()
}

fn content_type_for(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn cookie_set_route(value: &str) -> Route {
    Route::Dynamic {
        body: format!("<html><title>Cookie Set {value}</title><body>{value}</body></html>")
            .into_bytes(),
        headers: vec![
            ("Content-Type".into(), "text/html; charset=utf-8".into()),
            (
                "Set-Cookie".into(),
                format!("bbr_profile={value}; Path=/; SameSite=Lax"),
            ),
        ],
    }
}

fn html_route(html: &str) -> Route {
    Route::Dynamic {
        body: html.as_bytes().to_vec(),
        headers: vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
    }
}

fn cookie_value(header: Option<&String>, name: &str) -> Option<String> {
    header?.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    Some((method, target))
}

fn respond(stream: &mut TcpStream, status: u16, reason: &str, body: &[u8]) -> std::io::Result<()> {
    respond_with_headers(stream, status, reason, &[], body)
}

fn respond_with_headers(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> std::io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

/// Convenience for tests: read a URL synchronously.
pub fn get(url: &str) -> std::io::Result<(u16, Vec<u8>)> {
    get_with_headers(url, &[])
}

/// Convenience for tests that need request headers (for example cookies).
pub fn get_with_headers(
    url: &str,
    headers: &[(String, String)],
) -> std::io::Result<(u16, Vec<u8>)> {
    let rest = url.trim_start_matches("http://");
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let mut stream = TcpStream::connect(authority)?;
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: {authority}\r\n");
    for (name, value) in headers {
        req.push_str(name);
        req.push_str(": ");
        req.push_str(value);
        req.push_str("\r\n");
    }
    req.push_str("Connection: close\r\n\r\n");
    stream.write_all(req.as_bytes())?;
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status)?;
    let code: u16 = status
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }
    let mut body = Vec::new();
    reader.read_to_end(&mut body)?;
    Ok((code, body))
}

#[cfg(test)]
mod tests {
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
        // The truncated-error route declares a 64 KiB body but delivers 4
        // bytes: the wire shape is deterministic and the client observes the
        // premature close (the engine fails the navigation).
        let (code3, body3) = get(&server.url("/__fx/error/truncated-error")).unwrap();
        assert_eq!(code3, 500);
        assert_eq!(body3, b"boom");
        // The flaky route fails its first request and succeeds afterwards
        // (the recovery E2E retries it): request 1 is the truncated 500,
        // request 2 is a real page.
        let (code4, body4) = get(&server.url("/__fx/error/flaky")).unwrap();
        assert_eq!(code4, 500);
        assert_eq!(body4, b"boom");
        let (code5, body5) = get(&server.url("/__fx/error/flaky")).unwrap();
        assert_eq!(code5, 200);
        assert!(String::from_utf8(body5).unwrap().contains("recovered"));
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
}
