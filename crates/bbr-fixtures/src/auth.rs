//! Deterministic HTTP authentication fixture.
//!
//! `/__fx/auth/basic` asks for HTTP Basic credentials in realm
//! `BBR Fixture`. A request carrying `bbr` / `secret` gets the protected
//! page; every other request gets `401 Unauthorized` with the challenge,
//! so WebKit hands the real navigation delegate a native authentication
//! challenge without TLS, a Keychain or the network. Without a credential
//! the engine shows the 401 page, whose title the E2E waits for.

use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

pub(crate) const BASIC_PATH: &str = "/__fx/auth/basic";
pub(crate) const BASIC_REALM: &str = "BBR Fixture";
pub(crate) const BASIC_TITLE: &str = "BBR Auth Required";
pub(crate) const SIGNED_IN_TITLE: &str = "BBR Auth Signed In";
/// `Basic base64("bbr:secret")`.
const ACCEPTED: &str = "Basic YmJyOnNlY3JldA==";

/// Answer one request to the Basic route.
pub(crate) fn respond_basic(
    stream: &mut TcpStream,
    headers: &HashMap<String, String>,
) -> io::Result<()> {
    if headers
        .get("authorization")
        .is_some_and(|value| value.trim() == ACCEPTED)
    {
        let body = page(SIGNED_IN_TITLE, "Signed in");
        return super::respond_with_headers(
            stream,
            200,
            "OK",
            &[("Content-Type".into(), "text/html; charset=utf-8".into())],
            body.as_bytes(),
        );
    }
    let body = page(BASIC_TITLE, "Authentication required");
    super::respond_with_headers(
        stream,
        401,
        "Unauthorized",
        &[
            (
                "WWW-Authenticate".into(),
                format!("Basic realm=\"{BASIC_REALM}\""),
            ),
            ("Content-Type".into(), "text/html; charset=utf-8".into()),
        ],
        body.as_bytes(),
    )
}

fn page(title: &str, heading: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head>\
         <body><main><h1>{heading}</h1></main></body></html>"
    )
}
