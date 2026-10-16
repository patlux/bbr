//! Test-owned HTTP response barrier. It holds bytes, never browser state.
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const PREFIX: &str = "/__fx/held-download/";
const DEADLINE: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(crate) struct Holds(Mutex<HashMap<usize, Arc<Barrier>>>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Pending,
    Released,
    Aborted,
}
struct State {
    requested: bool,
    outcome: Outcome,
}
struct Barrier {
    path: String,
    state: Mutex<State>,
    changed: Condvar,
    until: Instant,
}

/// One registered response. Dropping an unreleased hold aborts it rather than
/// letting the native transfer succeed after the test has failed.
pub struct DownloadHold {
    url: String,
    barrier: Arc<Barrier>,
}
impl DownloadHold {
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Prove that WebKit reached this fixture without releasing any headers.
    pub fn wait_requested(&self) -> io::Result<()> {
        let mut state = self.barrier.state.lock().unwrap();
        while !state.requested && state.outcome == Outcome::Pending {
            state = self.barrier.wait(state)?;
        }
        self.barrier.remaining(&mut state)?;
        if state.requested && state.outcome == Outcome::Pending {
            Ok(())
        } else {
            Err(io::Error::other("download hold is no longer pending"))
        }
    }

    /// Release only an observed request. Subsequent retries use the identical
    /// fixture response, without a second hold or altered response bytes.
    pub fn release(self) -> io::Result<()> {
        let mut state = self.barrier.state.lock().unwrap();
        self.barrier.remaining(&mut state)?;
        if !state.requested || state.outcome != Outcome::Pending {
            return Err(io::Error::other("download hold cannot be released"));
        }
        state.outcome = Outcome::Released;
        self.barrier.changed.notify_all();
        Ok(())
    }
}
impl Drop for DownloadHold {
    fn drop(&mut self) {
        let mut state = self.barrier.state.lock().unwrap();
        if state.outcome == Outcome::Pending {
            state.outcome = Outcome::Aborted;
            self.barrier.changed.notify_all();
        }
    }
}
impl Barrier {
    fn remaining(&self, state: &mut State) -> io::Result<Duration> {
        let remaining = self.until.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            if state.outcome == Outcome::Pending {
                state.outcome = Outcome::Aborted;
                self.changed.notify_all();
            }
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "download hold deadline",
            ));
        }
        Ok(remaining)
    }

    fn wait<'a>(
        &self,
        mut state: std::sync::MutexGuard<'a, State>,
    ) -> io::Result<std::sync::MutexGuard<'a, State>> {
        let remaining = self.remaining(&mut state)?;
        let (state, _) = self.changed.wait_timeout(state, remaining).unwrap();
        Ok(state)
    }
}
impl Holds {
    pub(crate) fn register(&self, base: &str, url: &str) -> io::Result<DownloadHold> {
        let path = url
            .strip_prefix(base)
            .ok_or_else(|| io::Error::other("foreign fixture URL"))?;
        if super::download_fixture(path).is_none() {
            return Err(io::Error::other("exact download fixture URL required"));
        }
        let mut holds = self.0.lock().unwrap();
        let id = holds.len();
        let barrier = Arc::new(Barrier {
            path: path.into(),
            state: Mutex::new(State {
                requested: false,
                outcome: Outcome::Pending,
            }),
            changed: Condvar::new(),
            until: Instant::now() + DEADLINE,
        });
        holds.insert(id, barrier.clone());
        Ok(DownloadHold {
            url: format!("{base}{PREFIX}{id}"),
            barrier,
        })
    }

    pub(crate) fn resolve(&self, path: &str) -> io::Result<Option<String>> {
        let Some(id) = path.strip_prefix(PREFIX) else {
            return Ok(None);
        };
        let id: usize = id
            .parse()
            .map_err(|_| io::Error::other("invalid hold ID"))?;
        let barrier = self
            .0
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| io::Error::other("unknown hold ID"))?;
        let mut state = barrier.state.lock().unwrap();
        if state.requested && state.outcome == Outcome::Pending {
            return Err(io::Error::other("duplicate pending held request"));
        }
        state.requested = true;
        barrier.changed.notify_all();
        while state.outcome == Outcome::Pending {
            state = barrier.wait(state)?;
        }
        match state.outcome {
            Outcome::Released => Ok(Some(barrier.path.clone())),
            _ => Err(io::Error::other("download hold aborted")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn request(url: &str) -> TcpStream {
        let (host, path) = url
            .strip_prefix("http://")
            .unwrap()
            .split_once('/')
            .unwrap();
        let mut stream = TcpStream::connect(host).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            stream,
            "GET /{path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        stream
    }

    #[test]
    fn holds_headers_until_observed_then_preserves_every_fixture_byte_and_retry() {
        let server = crate::FixtureServer::start(std::env::temp_dir()).unwrap();
        for name in ["known", "unknown", "slow", "zero", "fail"] {
            let url = server.url(&format!("/__fx/dl/{name}.bin"));
            let hold = server.hold_download(&url).unwrap();
            let held_url = hold.url().to_owned();
            let mut stream = request(&held_url);
            hold.wait_requested().unwrap();
            stream.set_nonblocking(true).unwrap();
            assert_eq!(
                stream.peek(&mut [0]).unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
            stream.set_nonblocking(false).unwrap();
            hold.release().unwrap();
            let mut actual = Vec::new();
            stream.read_to_end(&mut actual).unwrap();
            let mut expected = Vec::new();
            request(&url).read_to_end(&mut expected).unwrap();
            assert_eq!(actual, expected);
            let mut repeated = Vec::new();
            request(&held_url).read_to_end(&mut repeated).unwrap();
            assert_eq!(actual, repeated);
        }
    }

    #[test]
    fn rejects_foreign_unknown_unobserved_duplicate_and_aborted_holds() {
        let holds = Arc::new(Holds::default());
        assert!(holds
            .register("http://a", "http://b/__fx/dl/known.bin")
            .is_err());
        assert!(holds.register("http://a", "http://a/page1.html").is_err());
        assert!(holds.resolve("/__fx/held-download/99").is_err());
        assert!(holds.resolve("/__fx/held-download/nope").is_err());
        assert_eq!(holds.resolve("/ordinary").unwrap(), None);
        assert!(holds
            .register("http://a", "http://a/__fx/dl/known.bin")
            .unwrap()
            .release()
            .is_err());
        let hold = holds
            .register("http://a", "http://a/__fx/dl/zero.bin")
            .unwrap();
        let path = hold.url().strip_prefix("http://a").unwrap().to_owned();
        let child_holds = holds.clone();
        let child_path = path.clone();
        let child = std::thread::spawn(move || child_holds.resolve(&child_path));
        hold.wait_requested().unwrap();
        assert!(holds.resolve(&path).is_err());
        drop(hold);
        assert!(child.join().unwrap().is_err());
        assert!(holds.resolve(&path).is_err());
    }

    #[test]
    fn expired_observed_request_cannot_be_released_or_reported_ready() {
        for release_directly in [false, true] {
            let holds = Holds::default();
            let mut hold = holds
                .register("http://a", "http://a/__fx/dl/zero.bin")
                .unwrap();
            holds.0.lock().unwrap().clear();
            let barrier = Arc::get_mut(&mut hold.barrier).unwrap();
            barrier.until = Instant::now();
            barrier.state.get_mut().unwrap().requested = true;
            if !release_directly {
                assert_eq!(
                    hold.wait_requested().unwrap_err().kind(),
                    io::ErrorKind::TimedOut
                );
            }
            assert!(hold.release().is_err());
        }
    }

    #[test]
    fn expiry_never_releases_a_response() {
        let holds = Holds::default();
        let mut hold = holds
            .register("http://a", "http://a/__fx/dl/zero.bin")
            .unwrap();
        holds.0.lock().unwrap().clear();
        Arc::get_mut(&mut hold.barrier).unwrap().until = Instant::now();
        assert_eq!(
            hold.wait_requested().unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(hold.release().is_err());
    }
}
