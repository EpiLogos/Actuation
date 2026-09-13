//! Sync client for gateway connections: the connector SDK surface and the
//! agent-session side of the vertical. One `GatewayClient` is one granted
//! connection over `actuation.gateway/v1` frames on the same host's UDS
//! carrier.

use actuation_core::{
    ActuationRef, AgencyRef, AgentSessionRef, Error, ExternalRef, Result, StreamRef,
    WorldBindingRef,
};
use actuation_stream::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::BufReader,
    os::unix::net::{SocketAddr, UnixStream},
    path::Path,
};

/// A connection attach: the canonical identity of the stream being entered.
/// The gateway checks it against the grant and the durable stream header.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttachSpec {
    pub stream_ref: StreamRef,
    pub actuation_ref: ActuationRef,
    pub agency_ref: AgencyRef,
    pub agent_session_ref: AgentSessionRef,
    #[serde(default)]
    pub world_binding_ref: Option<WorldBindingRef>,
    #[serde(default)]
    pub provenance: Option<Vec<ExternalRef>>,
    #[serde(default)]
    pub started_at: Option<Timestamp>,
    #[serde(default)]
    pub conversation: Option<String>,
}

pub struct GatewayClient {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
}

impl GatewayClient {
    /// Connect, negotiate the protocol and authenticate in one step.
    pub fn connect(socket: &Path, token: Option<&str>, subject: &str) -> Result<Self> {
        let addr = SocketAddr::from_pathname(socket)
            .map_err(|e| Error::new(format!("invalid gateway socket path: {e}")))?;
        let stream = UnixStream::connect_addr(&addr).map_err(|e| {
            Error::new(format!("cannot reach gateway at {}: {e}", socket.display()))
        })?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(130)))
            .map_err(|e| Error::new(e.to_string()))?;
        let reader = BufReader::new(stream.try_clone().map_err(|e| Error::new(e.to_string()))?);
        let mut client = Self { stream, reader };
        let reply = client.call_raw(json!({
            "op":"hello",
            "protocol": crate::wire::GATEWAY_CONTRACT,
            "token": token,
            "subject": subject,
        }))?;
        if reply.get("ok") != Some(&Value::Bool(true)) {
            return Err(Error::new(format!(
                "gateway refused the connection: {reply}"
            )));
        }
        Ok(client)
    }

    /// Send one frame and return the raw reply frame.
    pub fn call_raw(&mut self, frame: Value) -> Result<Value> {
        crate::wire::write_frame(&mut self.stream, &frame)?;
        match crate::wire::read_frame(&mut self.reader)? {
            Some(reply) => Ok(reply),
            None => Err(Error::new("gateway closed the connection")),
        }
    }

    /// Send one frame and demand an ok reply.
    pub fn call(&mut self, frame: Value) -> Result<Value> {
        let reply = self.call_raw(frame)?;
        if reply.get("ok") == Some(&Value::Bool(true)) {
            Ok(reply)
        } else {
            Err(Error::new(format!("gateway refused: {reply}")))
        }
    }

    pub fn attach(&mut self, spec: AttachSpec) -> Result<Value> {
        let mut frame = serde_json::to_value(spec)?;
        frame["op"] = json!("attach");
        self.call(frame)
    }

    pub fn send(
        &mut self,
        content: &str,
        conversation: Option<&str>,
        observed_at: Option<Timestamp>,
    ) -> Result<Value> {
        self.call(json!({
            "op":"send",
            "content":content,
            "conversation":conversation,
            "observed_at":observed_at,
        }))
    }

    pub fn post(&mut self, request: Value) -> Result<Value> {
        let mut frame = request;
        frame["op"] = json!("post");
        self.call(frame)
    }

    pub fn replay(&mut self, after: u64, limit: Option<u64>) -> Result<Value> {
        self.call(json!({"op":"replay","after":after,"limit":limit}))
    }

    pub fn wait(&mut self, after: u64, timeout_ms: u64) -> Result<Value> {
        self.call(json!({"op":"wait","after":after,"timeout_ms":timeout_ms}))
    }

    pub fn invoke(&mut self, request: Value) -> Result<Value> {
        let mut frame = request;
        frame["op"] = json!("invoke");
        self.call(frame)
    }

    pub fn invoke_raw(&mut self, request: Value) -> Result<Value> {
        let mut frame = request;
        frame["op"] = json!("invoke");
        self.call_raw(frame)
    }

    pub fn discover(&mut self) -> Result<Value> {
        self.call(json!({"op":"discover"}))
    }

    pub fn status(&mut self) -> Result<Value> {
        self.call(json!({"op":"status"}))
    }
}
