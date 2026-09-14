//! The connector seam: what every first-party Surface connector implements,
//! proved here by the fully functional local CLI connector. Telegram, Discord,
//! Slack and later platforms implement the same trait against their native
//! body; capability differences stay explicit instead of being forced into
//! false parity.

use crate::client::GatewayClient;
use actuation_core::{Error, Result};
use serde_json::{json, Value};

/// What this connector can honestly carry. Declared, not assumed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectorCapabilities {
    pub text_inbound: bool,
    pub text_outbound: bool,
    pub media: bool,
    pub edit: bool,
    pub react: bool,
    pub typing: bool,
    pub threads: bool,
}

#[derive(Clone, Debug)]
pub struct InboundEvent {
    /// Provider-native conversation identity. Opaque to the gateway; it is
    /// never collapsed into canonical AgentSession identity.
    pub conversation: String,
    pub text: String,
}

pub struct Health {
    pub healthy: bool,
    pub detail: String,
}

/// A connector admitted by an explicit gateway grant.
pub trait SurfaceConnector {
    fn capabilities(&self) -> ConnectorCapabilities;
    /// Provider provenance: which native body this connector fronts.
    fn provenance(&self) -> Value;
    fn health(&mut self) -> Health;
    /// Admit one platform-native inbound event into the canonical stream with
    /// exact gateway-side attribution. Returns the durable receipt.
    fn admit(&mut self, event: InboundEvent) -> Result<Value>;
    /// Await the attributable Return for what this surface sent in.
    fn await_return(&mut self, after: u64, timeout_ms: u64) -> Result<Value>;
}

/// The local CLI connector: same host, UDS carrier, one conversation.
pub struct LocalConnector {
    client: GatewayClient,
    conversation: String,
    last_health: Option<Health>,
}
impl LocalConnector {
    pub fn new(client: GatewayClient, conversation: impl Into<String>) -> Self {
        Self {
            client,
            conversation: conversation.into(),
            last_health: None,
        }
    }
    pub fn client(&mut self) -> &mut GatewayClient {
        &mut self.client
    }
}

impl SurfaceConnector for LocalConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            text_inbound: true,
            text_outbound: true,
            media: false,
            edit: false,
            react: false,
            typing: false,
            threads: false,
        }
    }

    fn provenance(&self) -> Value {
        json!({
            "connector": "local-cli",
            "carrier": "uds",
            "contract": crate::wire::GATEWAY_CONTRACT,
        })
    }

    fn health(&mut self) -> Health {
        let health = match self.client.call(json!({"op":"ping"})) {
            Ok(reply) => Health {
                healthy: reply.get("pong") == Some(&Value::Bool(true)),
                detail: "gateway ping".into(),
            },
            Err(error) => Health {
                healthy: false,
                detail: error.to_string(),
            },
        };
        self.last_health = Some(Health {
            healthy: health.healthy,
            detail: health.detail.clone(),
        });
        health
    }

    fn admit(&mut self, event: InboundEvent) -> Result<Value> {
        self.client.send(
            &event.text,
            Some(&if event.conversation.is_empty() {
                self.conversation.clone()
            } else {
                event.conversation
            }),
            None,
        )
    }

    fn await_return(&mut self, after: u64, timeout_ms: u64) -> Result<Value> {
        // Stream progress is not a Return: keep reading forward from our
        // cursor until the attributable Return arrives or the deadline passes.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut cursor = after;
        loop {
            let remaining = deadline
                .saturating_duration_since(std::time::Instant::now())
                .as_millis() as u64;
            let reply = self.client.wait(cursor, remaining)?;
            let events = reply
                .get("events")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for event in &events {
                if event.get("kind").and_then(Value::as_str) == Some("return")
                    && event.pointer("/actor/agent_ref").is_some()
                {
                    return Ok(event.clone());
                }
            }
            cursor = reply
                .get("last_sequence")
                .and_then(Value::as_u64)
                .unwrap_or(cursor);
            if cursor <= after
                || reply.get("timed_out") == Some(&Value::Bool(true))
                || std::time::Instant::now() >= deadline
            {
                return Err(Error::new(format!(
                    "no attributable return arrived within {timeout_ms}ms; last cursor {cursor}"
                )));
            }
        }
    }
}

/// The cursor a connector keeps between encounters: where it has read to.
pub fn last_sequence(reply: &Value) -> u64 {
    reply
        .get("last_sequence")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}
