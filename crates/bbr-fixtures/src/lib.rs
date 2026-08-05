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
    match route(root, path, &headers)? {
        Route::File { data, content_type } => {
            let mut h = hits.lock().unwrap();
            *h.entry(path.to_string()).or_insert(0) += 1;
            drop(h);
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
            respond_with_headers(&mut stream, 200, "OK", &headers, &body)?;
        }
        Route::NotFound => {
            respond(&mut stream, 404, "Not Found", b"not found")?;
        }
    }
    Ok(())
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
        assert_eq!(body, b"<html><title>One</title></html>");
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
}
