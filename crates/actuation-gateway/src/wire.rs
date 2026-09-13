//! Gateway wire contract: transport-independent frames, one JSON object per
//! line. The same frames can ride UDS today and an authenticated WebSocket or
//! sidecar stdio carrier later without changing gateway/session semantics.
//!
//! A frame is admitted only as an object with an `op` string; replies keep the
//! same one-object-per-line discipline. Authentication and role live in the
//! session, never in the frame codec.

use actuation_core::{Error, Result};
use serde::Deserialize;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};

pub const GATEWAY_CONTRACT: &str = "actuation.gateway/v1";
pub const GATEWAY_IDENTITY: &str = "actuation-gateway";
pub const POLICY_SCHEMA: &str = "actuation.gateway-policy/v1";

/// One admitted frame. The first line that is not a JSON object with a
/// non-empty string `op` is refused; nothing after it is interpreted.
pub fn encode_frame(frame: &Value) -> Result<String> {
    if !frame.is_object() {
        return Err(Error::new("gateway frames must be JSON objects"));
    }
    let mut line = serde_json::to_string(frame)?;
    line.push('\n');
    Ok(line)
}

/// One admitted frame: a single JSON object per line. Requests must name an
/// `op`; replies need not. The server refuses op-less requests explicitly.
pub fn decode_frame(line: &str) -> Result<Value> {
    let frame: Value = serde_json::from_str(line)
        .map_err(|e| Error::new(format!("invalid gateway frame: {e}")))?;
    if !frame.is_object() {
        return Err(Error::new("gateway frames must be JSON objects"));
    }
    Ok(frame)
}

/// The requested operation of an inbound frame, if it names one.
pub fn frame_op(frame: &Value) -> Result<&str> {
    frame
        .get("op")
        .and_then(Value::as_str)
        .filter(|op| !op.is_empty())
        .ok_or_else(|| Error::new("gateway request requires a non-empty string op"))
}

pub fn read_frame<R: Read>(reader: &mut BufReader<R>) -> Result<Option<Value>> {
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|e| Error::new(e.to_string()))?;
    if read == 0 {
        return Ok(None);
    }
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }
    decode_frame(&line).map(Some)
}

pub fn write_frame<W: Write>(writer: &mut W, frame: &Value) -> Result<()> {
    writer
        .write_all(encode_frame(frame)?.as_bytes())
        .map_err(|e| Error::new(e.to_string()))?;
    writer.flush().map_err(|e| Error::new(e.to_string()))
}

pub fn ok_frame(fields: serde_json::Map<String, Value>) -> Value {
    let mut frame = serde_json::Map::new();
    frame.insert("ok".into(), Value::Bool(true));
    for (key, value) in fields {
        frame.insert(key, value);
    }
    Value::Object(frame)
}

pub fn error_frame(message: &str) -> Value {
    serde_json::json!({"ok":false,"error":message})
}

pub fn denied_frame(message: &str) -> Value {
    serde_json::json!({"ok":false,"denied":true,"error":message})
}

#[derive(Clone, Debug, Deserialize)]
pub struct HelloFrame {
    pub protocol: String,
    #[serde(default)]
    pub token: Option<String>,
    pub subject: String,
}

/// A reply that is never silence: every refused operation names its standing.
#[derive(Clone, Debug)]
pub enum Reply {
    /// Ok payload fields; the codec stamps `"ok":true` on the way out.
    Ok(serde_json::Map<String, Value>),
    Denied(String),
    Error(String),
    UnsupportedProtocol(String),
}
impl Reply {
    pub fn frame(self) -> Value {
        match self {
            Self::Ok(mut fields) => {
                fields.insert("ok".into(), Value::Bool(true));
                Value::Object(fields)
            }
            Self::Denied(message) => denied_frame(&message),
            Self::Error(message) => error_frame(&message),
            Self::UnsupportedProtocol(supplied) => serde_json::json!(
                {"ok":false,"unsupported_protocol":true,"supported":GATEWAY_CONTRACT,"error":format!("unsupported gateway protocol {supplied}; this gateway speaks {GATEWAY_CONTRACT}")}
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn roundtrip(frame: &str) -> Option<Value> {
        let mut reader = BufReader::new(Cursor::new(frame.as_bytes().to_vec()));
        read_frame(&mut reader).unwrap()
    }

    #[test]
    fn frames_roundtrip_as_single_json_objects_per_line() {
        let frame = serde_json::json!({"op":"hello","protocol":GATEWAY_CONTRACT,"subject":"connector:cli"});
        let mut writer: Vec<u8> = Vec::new();
        write_frame(&mut writer, &frame).unwrap();
        assert_eq!(
            roundtrip(&String::from_utf8(writer).unwrap()).unwrap(),
            frame
        );
    }

    #[test]
    fn non_object_frames_are_refused_and_ops_are_requested_from_requests() {
        assert!(decode_frame("[1,2]").is_err());
        assert!(decode_frame("not json").is_err());
        assert!(frame_op(&serde_json::json!({"op":"hello"})).unwrap() == "hello");
        assert!(frame_op(&serde_json::json!({"nope":true})).is_err());
        assert!(frame_op(&serde_json::json!({"op":""})).is_err());
        assert!(roundtrip("").is_none(), "clean EOF is an empty frame");
    }

    #[test]
    fn protocol_mismatch_names_the_supported_contract() {
        let reply = Reply::UnsupportedProtocol("actuation.gateway/v9".into());
        let frame = reply.frame();
        assert_eq!(frame["supported"], GATEWAY_CONTRACT);
        assert_eq!(frame["unsupported_protocol"], true);
    }
}
