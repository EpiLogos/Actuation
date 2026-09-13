//! Shared vertical field: one started gateway, its durable store, grants and
//! a scripted resident worker agent for the integration tests.

use actuation_core::{ActuationRef, AgencyRef, AgentSessionRef, Error, Result, StreamRef};
use actuation_gateway::{
    AttachSpec, GatewayClient, GatewayConfig, GatewayHandle, GatewayPolicy,
};
use actuation_stream::JsonlStreamStore;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    thread::JoinHandle,
};

pub const TOKEN: &str = "vertical-token";

pub struct Field {
    _dir: tempfile::TempDir,
    handle: Option<GatewayHandle>,
    pub store: JsonlStreamStore,
    socket_path: PathBuf,
}
impl Field {
    pub fn start() -> Self {
        let dir = tempfile::tempdir().expect("temp field");
        let store = JsonlStreamStore::new(dir.path().join("streams")).expect("store");
        let policy: GatewayPolicy = serde_json::from_value(json!({
            "schema": "actuation.gateway-policy/v1",
            "attach": [
                {"subject":"connector:cli","role":"connector","stream_ref":"stream:worker",
                 "surface_ref":"surface:cli","participant_ref":"participant:alice"},
                {"subject":"agent:worker-1","role":"agent","stream_ref":"stream:worker",
                 "agency_ref":"agency:worker","agent_ref":"agent:worker","locus_ref":"locus:worker"},
                {"subject":"agent:ctl-1","role":"agent","stream_ref":"stream:ctl",
                 "agency_ref":"agency:ctl","agent_ref":"agent:ctl","may_invoke":true}
            ],
            "invoke": [
                {"controller_agency_ref":"agency:ctl","target_agency_ref":"agency:worker",
                 "modes":["delegation","communique","session-contribution"]}
            ]
        }))
        .expect("policy");
        let socket_path = dir.path().join("gw.sock");
        let handle = actuation_gateway::start(
            GatewayConfig {
                socket_path: socket_path.clone(),
                token: Some(TOKEN.into()),
                max_wait_ms: 30_000,
            },
            store.clone(),
            policy,
        )
        .expect("gateway starts");
        Self {
            _dir: dir,
            handle: Some(handle),
            store,
            socket_path,
        }
    }
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
    pub fn worker_attach(&self) -> AttachSpec {
        AttachSpec {
            stream_ref: StreamRef::new("stream:worker").unwrap(),
            actuation_ref: ActuationRef::new("act:vertical").unwrap(),
            agency_ref: AgencyRef::new("agency:worker").unwrap(),
            agent_session_ref: AgentSessionRef::new("session:worker-1").unwrap(),
            world_binding_ref: None,
            provenance: None,
            started_at: None,
            conversation: None,
        }
    }
    pub fn controller_attach(&self) -> AttachSpec {
        AttachSpec {
            stream_ref: StreamRef::new("stream:ctl").unwrap(),
            actuation_ref: ActuationRef::new("act:vertical").unwrap(),
            agency_ref: AgencyRef::new("agency:ctl").unwrap(),
            agent_session_ref: AgentSessionRef::new("session:ctl-1").unwrap(),
            world_binding_ref: None,
            provenance: None,
            started_at: None,
            conversation: None,
        }
    }
}
impl Drop for Field {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.stop();
        }
    }
}

/// A raw attach frame for negative tests that bypass the typed client helper.
pub fn attach_frame(spec: &AttachSpec) -> Value {
    let mut frame = serde_json::to_value(spec).expect("attach spec");
    frame["op"] = json!("attach");
    frame
}

/// The resident worker agent used by every vertical test: it attaches, waits
/// for ingress (a connector challenge or a delegation), posts real tool
/// evidence and an attributable Return, then exits. The returned receiver
/// signals a successful attach: co-internal invocation and challenge
/// delivery both require the live granted session to exist first, so tests
/// must wait for readiness before any send.
pub fn spawn_worker_agent(
    socket: PathBuf,
    subject: String,
    attach: AttachSpec,
    expected: usize,
) -> (JoinHandle<()>, std::sync::mpsc::Receiver<()>) {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        worker_loop(&socket, &subject, attach, expected, &ready_tx)
            .expect("worker agent runs cleanly")
    });
    (handle, ready_rx)
}

fn worker_loop(
    socket: &Path,
    subject: &str,
    attach: AttachSpec,
    expected: usize,
    ready: &std::sync::mpsc::Sender<()>,
) -> Result<()> {
    let mut client = GatewayClient::connect(socket, Some(TOKEN), subject)?;
    let reply = client.attach(attach)?;
    let _ = ready.send(());
    let mut cursor = reply
        .get("cursor")
        .and_then(|cursor| cursor.get("last_sequence"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let mut handled = 0;
    while handled < expected {
        let wait = client.wait(cursor, 15_000)?;
        let events = wait
            .get("events")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        cursor = wait
            .get("last_sequence")
            .and_then(Value::as_u64)
            .unwrap_or(cursor);
        for event in &events {
            let kind = event.get("kind").and_then(Value::as_str).unwrap_or("");
            if !matches!(kind, "human-message" | "delegation" | "locus-event") {
                continue;
            }
            let challenge = event
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            client.post(json!({
                "kind":"tool-request",
                "content":challenge,
                "resource_refs":["tool:echo-agent"],
            }))?;
            let answer = format!("echo: {challenge}");
            client.post(json!({
                "kind":"tool-result",
                "content":answer,
                "evidence_refs":["tool:echo-agent"],
            }))?;
            let return_ref = event
                .pointer("/metadata/return_ref")
                .and_then(Value::as_str)
                .map(str::to_string)
                .map(Ok::<String, Error>)
                .unwrap_or_else(|| {
                    actuation_gateway::mint_return_ref().map(|reference| reference.into_string())
                })?;
            client.post(json!({
                "kind":"return",
                "content":answer,
                "return_ref":return_ref,
            }))?;
            handled += 1;
        }
        if events.is_empty() && wait.get("timed_out") == Some(&Value::Bool(true)) {
            return Err(Error::new("worker saw no ingress within its wait window"));
        }
    }
    Ok(())
}
