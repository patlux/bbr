//! E2E client: drives a spawned `bbr` process over the testkit socket.
//!
//! Synchronous and single-threaded by design: the harness thread writes one
//! request, then reads frames until the matching response. Event frames that
//! arrive during that round trip are queued and consumed by later event waits,
//! so fast app events cannot be lost. Event waits always use deadlines.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::framing::{decode_server_frame, encode_client_frame, read_frame, ServerFrame};
use crate::protocol::{dsl, Event, Request, Response, Snapshot};

#[derive(Debug)]
pub struct WelcomeInfo {
    pub app_version: String,
    pub pid: u32,
    pub capabilities: Vec<String>,
    pub next_seq: u64,
}

pub struct Client {
    stream: UnixStream,
    pending_events: VecDeque<(u64, Event)>,
}

impl Client {
    /// Open a raw connection to `socket_path`. Authentication happens in
    /// [`Client::hello`].
    pub fn connect(socket_path: &Path) -> io::Result<Client> {
        Ok(Client {
            stream: UnixStream::connect(socket_path)?,
            pending_events: VecDeque::new(),
        })
    }

    /// Perform the token-authenticated handshake.
    pub fn hello(&mut self, token: &str) -> Result<WelcomeInfo, crate::ProtocolError> {
        match self.request(&dsl::hello(token, &["smoke"]))? {
            Response::Welcome {
                app_version,
                pid,
                capabilities,
                next_seq,
                ..
            } => Ok(WelcomeInfo {
                app_version,
                pid,
                capabilities,
                next_seq,
            }),
            Response::Error(e) => Err(e),
            other => Err(crate::ProtocolError::new(
                "unexpected_response",
                format!("expected Welcome, got {other:?}"),
            )),
        }
    }

    /// Subscribe to the event stream; returns replayed events (if any).
    pub fn attach(
        &mut self,
        since: Option<u64>,
    ) -> Result<Vec<(u64, Event)>, crate::ProtocolError> {
        match self.request(&Request::Attach { since })? {
            Response::Replay { events, .. } => Ok(events),
            Response::Error(e) => Err(e),
            other => Err(crate::ProtocolError::new(
                "unexpected_response",
                format!("expected Replay, got {other:?}"),
            )),
        }
    }

    /// Send one request and read frames until the matching response.
    pub fn request(&mut self, req: &Request) -> Result<Response, crate::protocol::Error> {
        self.stream
            .write_all(&encode_client_frame(req))
            .map_err(|e| crate::ProtocolError::new("io", e.to_string()))?;
        self.stream
            .flush()
            .map_err(|e| crate::ProtocolError::new("io", e.to_string()))?;
        loop {
            let value = read_frame(&mut self.stream)
                .map_err(|e| crate::ProtocolError::new("io", e.to_string()))?;
            match decode_server_frame(value)
                .map_err(|e| crate::ProtocolError::new("proto", e.to_string()))?
            {
                ServerFrame::Event { seq, event } => {
                    self.pending_events.push_back((seq, event));
                    continue;
                }
                ServerFrame::Response(resp) => return Ok(resp),
            }
        }
    }

    /// Convenience: `request` + expect `Response::Ok`.
    pub fn send_ok(&mut self, req: &Request) -> Result<(), crate::ProtocolError> {
        match self.request(req)? {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(e),
            other => Err(crate::ProtocolError::new(
                "unexpected_response",
                format!("expected Ok, got {other:?}"),
            )),
        }
    }

    /// Wait until a predicate matches an event, or `deadline` passes.
    /// Pure event-wait: reads frames and matches them; no blind sleeps.
    pub fn wait_for_event(
        &mut self,
        predicate: impl Fn(&Event) -> bool,
        deadline: Instant,
    ) -> Result<(u64, Event), WaitError> {
        self.wait_for_event_where(|_, event| predicate(event), deadline)
    }

    /// Like [`Client::wait_for_event`], but only matches events with
    /// `seq > since` (a consumed-event cursor). Use this to assert a *fresh*
    /// emission of an event kind that may also have occurred earlier in the
    /// run (e.g. reload's `LoadingChanged` after a forward already produced
    /// one) — stale queued events are ignored, so the assertion cannot be
    /// satisfied by a leftover from a previous step.
    pub fn wait_for_event_since(
        &mut self,
        predicate: impl Fn(&Event) -> bool,
        since: u64,
        deadline: Instant,
    ) -> Result<(u64, Event), WaitError> {
        self.wait_for_event_where(|seq, event| seq > since && predicate(event), deadline)
    }

    fn wait_for_event_where(
        &mut self,
        matches: impl Fn(u64, &Event) -> bool,
        deadline: Instant,
    ) -> Result<(u64, Event), WaitError> {
        loop {
            if let Some(index) = self
                .pending_events
                .iter()
                .position(|(seq, event)| matches(*seq, event))
            {
                return Ok(self
                    .pending_events
                    .remove(index)
                    .expect("queued event index must exist"));
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(WaitError::Timeout);
            }
            let timeout = deadline.saturating_duration_since(now);
            self.stream
                .set_read_timeout(Some(timeout))
                .map_err(WaitError::Io)?;
            let value = match read_frame(&mut self.stream) {
                Ok(v) => v,
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    continue; // re-check deadline
                }
                Err(e) => return Err(WaitError::Io(e)),
            };
            match decode_server_frame(value) {
                Ok(ServerFrame::Event { seq, event }) if matches(seq, &event) => {
                    return Ok((seq, event));
                }
                Ok(ServerFrame::Event { seq, event }) => {
                    self.pending_events.push_back((seq, event));
                }
                Ok(ServerFrame::Response(_)) => continue,
                Err(e) => return Err(WaitError::Io(e)),
            }
        }
    }

    /// Wait until no event arrives for `quiet` (stability probe).
    pub fn wait_until_stable(&mut self, quiet: Duration) -> Result<(), WaitError> {
        let deadline = Instant::now() + quiet;
        loop {
            if Instant::now() >= deadline {
                return Ok(());
            }
            self.bounded_poll(deadline)?;
        }
    }

    fn bounded_poll(&mut self, deadline: Instant) -> Result<bool, WaitError> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        if timeout.is_zero() {
            return Ok(true);
        }
        self.stream
            .set_read_timeout(Some(timeout))
            .map_err(WaitError::Io)?;
        match read_frame(&mut self.stream) {
            Ok(_) => Ok(false), // an event arrived; keep polling
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                Ok(true)
            }
            Err(e) => Err(WaitError::Io(e)),
        }
    }

    /// Request a full app snapshot.
    pub fn snapshot(&mut self) -> Result<Snapshot, crate::ProtocolError> {
        match self.request(&Request::GetSnapshot)? {
            Response::Snapshot(s) => Ok(s),
            Response::Error(e) => Err(e),
            other => Err(crate::ProtocolError::new(
                "unexpected_response",
                format!("expected Snapshot, got {other:?}"),
            )),
        }
    }

    /// Request graceful shutdown.
    pub fn quit(&mut self) -> Result<(), crate::ProtocolError> {
        self.send_ok(&Request::Quit)
    }
}

#[derive(Debug)]
pub enum WaitError {
    Timeout,
    Io(io::Error),
}

impl std::fmt::Display for WaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitError::Timeout => write!(f, "timed out waiting for event"),
            WaitError::Io(e) => write!(f, "io error while waiting: {e}"),
        }
    }
}

impl std::error::Error for WaitError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Error as ProtoError;
    use std::path::PathBuf;

    fn tmp_profile() -> PathBuf {
        // A per-process counter guarantees uniqueness across parallel tests;
        // a wall-clock timestamp alone can collide when tests start in the
        // same clock tick (the fixture suite grew to >9 tests).
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        std::env::temp_dir().join(format!("bbr-testkit-client-{}-{n}", std::process::id()))
    }

    fn server_with(ping_reply: Response) -> (crate::server::GatewayServer, PathBuf) {
        let dir = tmp_profile();
        let server =
            crate::server::GatewayServer::start(&dir, "tok", "0.1.0", move |req| match req {
                Request::Ping => ping_reply.clone(),
                _ => Response::Error(ProtoError::new("unexpected", "only Ping expected")),
            })
            .expect("start");
        let path = server.socket_path().to_path_buf();
        (server, path)
    }

    #[test]
    fn request_ping_round_trip() {
        let (_server, path) = server_with(Response::Ok);
        let mut client = Client::connect(&path).unwrap();
        client.hello("tok").unwrap();
        assert!(client.send_ok(&Request::Ping).is_ok());
        assert!(client.send_ok(&Request::GetSnapshot).is_err());
    }

    #[test]
    fn request_queues_events_that_arrive_before_its_response() {
        let (server, path) = server_with(Response::Ok);
        let mut client = Client::connect(&path).unwrap();
        client.hello("tok").unwrap();

        server.broadcast(Event::ChromeFocusChanged {
            chrome_focused: true,
        });
        client.send_ok(&Request::Ping).unwrap();

        let (seq, event) = client
            .wait_for_event(
                |event| {
                    matches!(
                        event,
                        Event::ChromeFocusChanged {
                            chrome_focused: true
                        }
                    )
                },
                Instant::now() + Duration::from_millis(100),
            )
            .unwrap();
        assert_eq!(seq, 1);
        assert!(matches!(event, Event::ChromeFocusChanged { .. }));
    }

    #[test]
    fn wait_for_event_works_and_times_out() {
        let (server, path) = server_with(Response::Ok);
        let mut client = Client::connect(&path).unwrap();
        client.hello("tok").unwrap();

        server.broadcast(Event::ChromeFocusChanged {
            chrome_focused: true,
        });

        let (seq, ev) = client
            .wait_for_event(
                |e| {
                    matches!(
                        e,
                        Event::ChromeFocusChanged {
                            chrome_focused: true
                        }
                    )
                },
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(seq, 1);
        assert!(matches!(ev, Event::ChromeFocusChanged { .. }));

        // Non-matching predicate → deadline expires.
        let err = client
            .wait_for_event(
                |e| matches!(e, Event::AppReady { .. }),
                Instant::now() + Duration::from_millis(300),
            )
            .unwrap_err();
        assert!(matches!(err, WaitError::Timeout));
    }

    #[test]
    fn wait_for_event_since_requires_a_fresh_event() {
        let (server, path) = server_with(Response::Ok);
        let mut client = Client::connect(&path).unwrap();
        client.hello("tok").unwrap();

        // A stale event (seq 1) matching the predicate is queued during the
        // Ping round trip, before any cursor is recorded.
        server.broadcast(Event::ChromeFocusChanged {
            chrome_focused: true,
        });
        client.send_ok(&Request::Ping).unwrap();
        let since = 1; // cursor after the stale event

        // A *fresh* event with the same payload must still be matched.
        server.broadcast(Event::ChromeFocusChanged {
            chrome_focused: true,
        });
        server.broadcast(Event::ChromeFocusChanged {
            chrome_focused: false,
        });
        let (seq, ev) = client
            .wait_for_event_since(
                |e| {
                    matches!(
                        e,
                        Event::ChromeFocusChanged {
                            chrome_focused: true
                        }
                    )
                },
                since,
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(seq, 2, "must ignore the stale seq-1 event");
        assert!(matches!(ev, Event::ChromeFocusChanged { .. }));

        // Advancing the cursor past seq 2 means the earlier matching event is
        // no longer eligible, so the wait times out.
        let last = client
            .wait_for_event_since(
                |e| {
                    matches!(
                        e,
                        Event::ChromeFocusChanged {
                            chrome_focused: true
                        }
                    )
                },
                2,
                Instant::now() + Duration::from_millis(300),
            )
            .unwrap_err();
        assert!(matches!(last, WaitError::Timeout));
    }
}
