//! Deterministic HTTP authentication fixture.
//!
//! `/__fx/auth/basic` answers every request with `401 Unauthorized` and a
//! Basic challenge, so WebKit hands the real navigation delegate a native
//! authentication challenge (`webView:didReceiveAuthenticationChallenge:…`)
//! without TLS, a Keychain identity or the network. Without a credential
//! the engine shows the 401 page, whose title the E2E waits for.

use std::io;
use std::net::TcpStream;

pub(crate) const BASIC_PATH: &str = "/__fx/auth/basic";
pub(crate) const BASIC_REALM: &str = "BBR Fixture";
pub(crate) const BASIC_TITLE: &str = "BBR Auth Required";

/// Write the Basic challenge response.
pub(crate) fn respond_basic(stream: &mut TcpStream) -> io::Result<()> {
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{BASIC_TITLE}</title></head>\
         <body><main><h1>Authentication required</h1></main></body></html>"
    );
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
