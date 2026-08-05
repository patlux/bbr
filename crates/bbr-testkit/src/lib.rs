//! # bbr-testkit — typed automation protocol for BBR
//!
//! This crate is the single source of truth for the E2E control protocol
//! (see `E2E_ARCHITECTURE.md §5`). It is **fully portable**: it only uses
//! unix sockets (`std::os::unix::net`), `serde`-derived types and
//! `serde_json` framing — there is no macOS, no GPUI, and no WebKit here.
//!
//! Two roles:
//!
//! - **Gateway server** ([`server`]) — compiled into `bbr-app` under the
//!   `testkit` cargo feature. Binds a `SOCK_STREAM` unix socket inside the
//!   per-test profile directory (mode `0700`), authenticates with a
//!   per-spawn token, length-prefixes JSON frames, keeps a seq-numbered
//!   event ring for `Attach{since}` replay, and pushes events to clients
//!   as they happen.
//! - **Client** ([`client`]) — used by the E2E harness/tests to drive a
//!   spawned `bbr` process: request/response plus `wait_for_event` /
//!   `wait_until` predicates with deadlines (no sleeps).
//!
//! ## Wire format
//!
//! One frame per message: `u32 LE` payload length followed by that many
//! bytes of a single JSON `Value`. Requests are client → server,
//! responses/events are server → client. Debuggable with `nc -U`.

pub mod client;
pub mod framing;
pub mod protocol;
pub mod server;

pub use client::Client;
pub use protocol::{Error as ProtocolError, Event, Request, Response, Snapshot};
pub use server::GatewayServer;
