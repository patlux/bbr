//! The testkit gateway server: a unix socket that exposes the running app
//! to E2E harnesses.
//!
//! Security posture (see E2E_ARCHITECTURE.md §12):
//! - Only compiled under `bbr-app`'s `testkit` cargo feature (release builds
//!   exclude it; the feature is never applied to release profiles).
//! - The socket lives **inside** the per-test profile directory (mode
//!   `0700`), which is only writable by the test runner and the app.
//! - Every connection must complete a token-authenticated `Hello` before any
//!   other request is honoured; the token is spawned per-test via arg/env,
//!   never embedded in the product.
//! - Loopback/unix transport only — nothing externally reachable.

use std::fs;
use std::io::{self, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use super::framing::{encode_server_frame, ClientFrame, ServerFrame};
use super::protocol::{Error, Event, Request, Response, PROTOCOL_VERSION};

/// Number of most-recent events kept for `Attach{since}` replay.
const RING_CAPACITY: usize = 4096;

/// One output message queued for a connection's writer thread.
enum OutMsg {
    Response(Vec<u8>),
    Event { json: Vec<u8>, _seq: u64 },
}

/// cap-first insertion ring (VecDeque with a max length).
struct Ring {
    events: std::collections::VecDeque<(u64, Event)>,
    cap: usize,
}

impl Ring {
    fn new(cap: usize) -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(cap),
            cap,
        }
    }
    fn push(&mut self, seq: u64, event: Event) {
        if self.events.len() == self.cap {
            self.events.pop_front();
        }
        self.events.push_back((seq, event));
    }
    fn replay(&self, since: Option<u64>) -> Vec<(u64, Event)> {
        match since {
            None => self.events.iter().cloned().collect(),
            Some(s) => self
                .events
                .iter()
                .filter(|(seq, _)| *seq > s)
                .cloned()
                .collect(),
        }
    }
}

struct Shared {
    token: String,
    app_version: String,
    ring: Mutex<Ring>,
    clients: Mutex<Vec<(u64, Sender<OutMsg>)>>,
    next_client_id: AtomicU64,
    next_event_seq: AtomicU64,
}

/// The running gateway server. Drop to stop accepting (existing clients are
/// left to close on their own; the app unlinks the socket on exit).
pub struct GatewayServer {
    // Kept alive for the socket binding; drop closes the listener.
    #[allow(dead_code)]
    listener: UnixListener,
    path: PathBuf,
    shared: Arc<Shared>,
    #[allow(dead_code)]
    accept_thread: Option<JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

impl GatewayServer {
    /// Bind a socket at `profile_dir/bbr-test.sock` (creating `profile_dir`
    /// with mode `0700`) and start the accept loop.
    ///
    /// `handler` runs on per-connection worker threads and returns the
    /// [`Response`] for every non-handshake/[`Request::Attach`] request.
    /// In `bbr-app` the glue implementation forwards to the main thread and
    /// blocks until the response is ready.
    pub fn start<H>(
        profile_dir: &Path,
        token: impl Into<String>,
        app_version: impl Into<String>,
        handler: H,
    ) -> io::Result<Self>
    where
        H: Fn(Request) -> Response + Send + Sync + 'static,
    {
        fs::create_dir_all(profile_dir)?;
        fs::set_permissions(profile_dir, fs::Permissions::from_mode(0o700))?;

        let path = profile_dir.join("bbr-test.sock");
        if path.exists() {
            // Stale socket from a crashed run; safe to remove (each test owns
            // its profile dir exclusively).
            fs::remove_file(&path)?;
        }

        let listener = UnixListener::bind(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;

        let shared = Arc::new(Shared {
            token: token.into(),
            app_version: app_version.into(),
            ring: Mutex::new(Ring::new(RING_CAPACITY)),
            clients: Mutex::new(Vec::new()),
            next_client_id: AtomicU64::new(1),
            next_event_seq: AtomicU64::new(1),
        });
        let stopped = Arc::new(AtomicBool::new(false));

        let shared_for_thread = shared.clone();
        let handler = Arc::new(handler);
        let stopped_for_thread = stopped.clone();
        let listener_owned = listener.try_clone()?;
        let accept_thread = std::thread::Builder::new()
            .name("bbr-testkit-accept".into())
            .spawn(move || {
                while !stopped_for_thread.load(Ordering::Relaxed) {
                    match listener_owned.accept() {
                        Ok((stream, _addr)) => {
                            let client_id = shared_for_thread
                                .next_client_id
                                .fetch_add(1, Ordering::Relaxed);
                            let (tx, rx) = mpsc::channel::<OutMsg>();
                            shared_for_thread
                                .clients
                                .lock()
                                .unwrap()
                                .push((client_id, tx.clone()));
                            let shared = shared_for_thread.clone();
                            let handler = handler.clone();
                            std::thread::Builder::new()
                                .name("bbr-testkit-conn".into())
                                .spawn(move || {
                                    connection_loop(client_id, stream, tx, rx, shared, handler);
                                })
                                .ok();
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
            })?;

        Ok(GatewayServer {
            listener,
            path,
            shared,
            accept_thread: Some(accept_thread),
            stopped,
        })
    }

    /// Get the bound socket path (useful for diagnostics).
    pub fn socket_path(&self) -> &Path {
        &self.path
    }

    /// Publish an event to every connected client and append it to the ring.
    pub fn broadcast(&self, event: Event) {
        let seq = self.shared.next_event_seq.fetch_add(1, Ordering::SeqCst);
        self.shared.ring.lock().unwrap().push(seq, event.clone());
        let json = encode_server_frame(&ServerFrame::Event { seq, event });
        // Best-effort push; slow clients drop events (state lives in
        // GetSnapshot anyway).
        let clients = self.shared.clients.lock().unwrap();
        for (_, client) in clients.iter() {
            let _ = client.send(OutMsg::Event {
                json: json.clone(),
                _seq: seq,
            });
        }
    }

    /// Stop the accept loop and unlink the socket path.
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        let _ = fs::remove_file(&self.path);
    }
}

/// Per-connection loop: authenticate (Hello), answer Attach inline, forward
/// everything else to the app handler, and stream events out.
fn connection_loop(
    client_id: u64,
    stream: UnixStream,
    out_tx: Sender<OutMsg>,
    out_rx: Receiver<OutMsg>,
    shared: Arc<Shared>,
    handler: Arc<dyn Fn(Request) -> Response + Send + Sync>,
) {
    let write_half = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };

    // The writer thread owns the write half exclusively, serializing event
    // pushes and responses per connection.
    let writer_thread = std::thread::Builder::new()
        .name("bbr-testkit-writer".into())
        .spawn(move || {
            let mut writer = BufWriter::new(write_half);
            while let Ok(msg) = out_rx.recv() {
                let bytes = match msg {
                    OutMsg::Response(bytes) => bytes,
                    OutMsg::Event { json, .. } => json,
                };
                if writer.write_all(&bytes).is_err() || writer.flush().is_err() {
                    return;
                }
            }
        })
        .expect("spawn gateway writer thread");

    let mut reader = BufReader::new(stream);
    _connection_requests(&mut reader, &out_tx, &shared, &handler);

    // Remove every sender clone before joining the writer. Keeping the copy
    // in `Shared::clients` alive would leave `recv` blocked forever after a
    // client disconnects and deadlock this connection thread.
    shared
        .clients
        .lock()
        .unwrap()
        .retain(|(id, _)| *id != client_id);
    drop(out_tx);
    let _ = writer_thread.join();
}

/// Real per-connection request loop: authenticate (Hello), answer Attach
/// inline, forward everything else to the app handler.
fn _connection_requests(
    reader: &mut BufReader<UnixStream>,
    out_tx: &Sender<OutMsg>,
    shared: &Arc<Shared>,
    handler: &Arc<dyn Fn(Request) -> Response + Send + Sync>,
) {
    let mut authenticated = false;
    loop {
        let value = match super::framing::read_frame(reader) {
            Ok(v) => v,
            Err(_) => return,
        };
        let frame: ClientFrame = match serde_json::from_value(value) {
            Ok(f) => f,
            Err(_) => return,
        };
        let ClientFrame::Request(req) = frame;

        let response = match req {
            Request::Hello {
                protocol_version,
                token,
                capabilities,
            } => {
                if protocol_version != PROTOCOL_VERSION {
                    Response::Error(Error::new(
                        "protocol_mismatch",
                        format!("client protocol {protocol_version}, server {PROTOCOL_VERSION}"),
                    ))
                } else if token != shared.token {
                    // Wrong token: reject and drop the connection.
                    return;
                } else {
                    authenticated = true;
                    Response::Welcome {
                        protocol_version,
                        app_version: shared.app_version.clone(),
                        pid: std::process::id(),
                        capabilities,
                        next_seq: shared.next_event_seq.load(Ordering::SeqCst),
                    }
                }
            }
            Request::Attach { since } => {
                if !authenticated {
                    return;
                }
                let events = shared.ring.lock().unwrap().replay(since);
                Response::Replay {
                    events,
                    next_seq: shared.next_event_seq.load(Ordering::SeqCst),
                }
            }
            _other if !authenticated => return,
            other => handler(other),
        };

        let json = encode_server_frame(&super::framing::ServerFrame::Response(response));
        if out_tx.send(OutMsg::Response(json)).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_profile() -> PathBuf {
        std::env::temp_dir().join(format!(
            "bbr-testkit-server-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn socket_has_restrictive_permissions() {
        let dir = tmp_profile();
        let server =
            GatewayServer::start(&dir, "tok-1", "0.1.0", |_| Response::Ok).expect("start server");
        let meta = std::fs::metadata(server.socket_path()).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o700);
        let dir_meta = std::fs::metadata(&dir).unwrap();
        assert_eq!(dir_meta.permissions().mode() & 0o777, 0o700);
    }

    #[test]
    fn hello_round_trip_and_bad_token_rejected() {
        let dir = tmp_profile();
        let server = GatewayServer::start(&dir, "tok-1", "0.1.0", |req| match req {
            Request::Ping => Response::Ok,
            _ => Response::Error(Error::new("unexpected", "only Ping expected")),
        })
        .expect("start server");
        server.broadcast(Event::AppReady {
            pid: 1,
            app_version: "0.1.0".into(),
        });
        let path = server.socket_path().to_path_buf();
        let handle = std::thread::spawn(move || {
            let mut client = crate::client::Client::connect(&path).expect("connect");
            let welcome = client.hello("tok-1").expect("hello");
            assert_eq!(welcome.pid, std::process::id());
            let replay = client.attach(None).expect("attach");
            assert_eq!(replay.len(), 1);
            assert!(matches!(replay[0].1, Event::AppReady { .. }));
            // Wrong token must fail the handshake (connection dropped).
            let err = client.hello("nope").expect_err("wrong token");
            assert!(!err.to_string().is_empty());
        });
        handle.join().unwrap();
    }
}
