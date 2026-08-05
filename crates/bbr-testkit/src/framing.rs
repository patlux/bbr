//! Length-prefixed JSON framing shared by the gateway server and client.
//!
//! Frame layout: `u32 LE` payload length, then exactly that many bytes of a
//! single JSON `Value`. A socket is a sequence of such frames.
//!
//! Wire envelopes (symmetric `type`/`content` tagging):
//!
//! ```text
//! client → server: {"type":"request",  "payload": <Request>}
//! server → client: {"type":"response", "payload": <Response>}
//! server → client: {"type":"event",    "payload": {"seq": <u64>, "event": <Event>}}
//! ```

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::{Event, Request, Response};

const MAX_FRAME: usize = 8 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub(crate) enum ClientFrame {
    Request(Request),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub(crate) enum ServerFrame {
    Response(Response),
    Event { seq: u64, event: Event },
}

/// Write one JSON frame to `w`.
pub fn write_frame(w: &mut impl Write, value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame too large: {} bytes", bytes.len()),
        ));
    }
    let len = (bytes.len() as u32).to_le_bytes();
    w.write_all(&len)?;
    w.write_all(&bytes)?;
    w.flush()
}

/// Read one JSON frame from `r` (blocking until a full frame is available).
pub fn read_frame(r: &mut impl Read) -> io::Result<Value> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length too large: {len}"),
        ));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf)?)
}

pub(crate) fn encode_client_frame(request: &Request) -> Vec<u8> {
    let value = serde_json::to_value(ClientFrame::Request(request.clone()))
        .expect("Request is serializable");
    let mut buf = Vec::new();
    write_frame(&mut buf, &value).expect("framing cannot fail on Vec<u8>");
    buf
}

pub(crate) fn encode_server_frame(frame: &ServerFrame) -> Vec<u8> {
    let value = serde_json::to_value(frame).expect("ServerFrame is serializable");
    let mut buf = Vec::new();
    write_frame(&mut buf, &value).expect("framing cannot fail on Vec<u8>");
    buf
}

pub(crate) fn decode_server_frame(value: Value) -> io::Result<ServerFrame> {
    serde_json::from_value(value).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::dsl;
    use std::io::Cursor;

    #[test]
    fn framing_round_trip() {
        let values = [
            serde_json::json!({"type": "request", "payload": {"Ping": {}}}),
            serde_json::json!({"nested": [1, 2, 3], "s": "x"}),
            serde_json::json!(null),
        ];
        let mut buf = Vec::new();
        for v in &values {
            write_frame(&mut buf, v).unwrap();
        }
        let mut cur = Cursor::new(buf);
        for expected in &values {
            let got = read_frame(&mut cur).unwrap();
            assert_eq!(&got, expected);
        }
    }

    #[test]
    fn framing_rejects_oversized() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(u32::MAX).to_le_bytes());
        let mut cur = Cursor::new(buf);
        let err = read_frame(&mut cur).unwrap_err();
        assert!(err.to_string().contains("too large"));
    }

    #[test]
    fn request_envelope_round_trips() {
        let req = dsl::navigate("https://example.com");
        let bytes = encode_client_frame(&req);
        let mut cur = Cursor::new(bytes);
        let value = read_frame(&mut cur).unwrap();
        let frame: ClientFrame = serde_json::from_value(value).unwrap();
        assert_eq!(frame, ClientFrame::Request(req));
    }

    #[test]
    fn server_frame_envelope_is_tagged() {
        let value = serde_json::to_value(ServerFrame::Event {
            seq: 3,
            event: Event::AppReady {
                pid: 42,
                app_version: "0.1.0".into(),
            },
        })
        .unwrap();
        assert_eq!(value["type"], "event");
        assert_eq!(value["payload"]["seq"], 3);
        let back = serde_json::from_value::<ServerFrame>(value).unwrap();
        assert!(matches!(
            back,
            ServerFrame::Event {
                seq: 3,
                event: Event::AppReady { .. }
            }
        ));
    }
}
