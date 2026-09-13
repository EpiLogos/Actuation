//! The Agency Gateway service.
//!
//! Workcell hosts the material conditions for this service; the gateway owns
//! the encounter plane over canonical Agency / AgentSession / ActuationStream
//! relations. It accepts same-host UDS connections (the first carrier of the
//! transport-independent `actuation.gateway/v1` frames), maps every connection
//! to an explicit grant, and writes canonical stream events through the
//! caller's own durable `JsonlStreamStore` with exact attribution.
//!
//! The gateway mints no session registry of its own: its connection table only
//! tracks which granted subject is presently live on which canonical stream.
//! Canonical identity lives in the streams, never in the socket.

use crate::mapping::{self, InvokeRequest, PostRequest, SendRequest};
use crate::policy::{AttachGrant, GatewayPolicy, GrantRole};
use crate::wire::{error_frame, HelloFrame, Reply, GATEWAY_CONTRACT, GATEWAY_IDENTITY};
use actuation_core::{
    ActuationRef, AgencyRef, AgentSessionRef, Error, ExternalRef, Result, StreamRef,
    WorldBindingRef,
};
use actuation_stream::{
    Count, JsonlStreamStore, OpenStream, PageRequest, StreamEvent, StreamStore, Timestamp,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(25);
const DEFAULT_WAIT_MS: u64 = 10_000;
const DEFAULT_INVOKE_TIMEOUT_MS: u64 = 30_000;

pub struct GatewayConfig {
    pub socket_path: PathBuf,
    pub token: Option<String>,
    pub max_wait_ms: u64,
}
impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/actuation-gateway.sock"),
            token: None,
            max_wait_ms: 120_000,
        }
    }
}

/// One live granted connection: the subject, its exact grant, and the
/// canonical session it is attached to. This is presence, not identity.
#[derive(Clone, Debug)]
pub struct Binding {
    pub connection: u64,
    pub grant: AttachGrant,
    pub agent_session_ref: AgentSessionRef,
    pub conversation: Option<String>,
}

#[derive(Deserialize)]
struct AttachFrame {
    stream_ref: StreamRef,
    actuation_ref: ActuationRef,
    agency_ref: AgencyRef,
    agent_session_ref: AgentSessionRef,
    #[serde(default)]
    world_binding_ref: Option<WorldBindingRef>,
    #[serde(default)]
    provenance: Option<Vec<ExternalRef>>,
    #[serde(default)]
    started_at: Option<Timestamp>,
    #[serde(default)]
    conversation: Option<String>,
}

#[derive(Deserialize)]
struct WaitFrame {
    after: Count,
    #[serde(default)]
    limit: Option<Count>,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
struct ReplayFrame {
    #[serde(default)]
    after: Option<Count>,
    #[serde(default)]
    limit: Option<Count>,
}

pub struct Gateway {
    config: GatewayConfig,
    store: JsonlStreamStore,
    policy: GatewayPolicy,
    bindings: Mutex<HashMap<u64, Binding>>,
    next_connection: AtomicU64,
    pub shutdown: AtomicBool,
}

impl Gateway {
    pub fn bind(
        config: GatewayConfig,
        store: JsonlStreamStore,
        policy: GatewayPolicy,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            config,
            store,
            policy,
            bindings: Mutex::new(HashMap::new()),
            next_connection: AtomicU64::new(1),
            shutdown: AtomicBool::new(false),
        }))
    }

    pub fn store(&self) -> &JsonlStreamStore {
        &self.store
    }

    pub fn policy(&self) -> &GatewayPolicy {
        &self.policy
    }

    /// Serve until `shutdown`. Each connection is one thread; the gateway is a
    /// small persistent service, not a broker ontology.
    pub fn run(self: &Arc<Self>, listener: UnixListener) {
        listener.set_nonblocking(true).ok();
        while !self.shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let gateway = Arc::clone(self);
                    thread::spawn(move || gateway.handle_connection(stream));
                }
                Err(_) => thread::sleep(POLL_INTERVAL),
            }
        }
        let _ = std::fs::remove_file(&self.config.socket_path);
    }

    fn receipt(&self, stream_ref: &StreamRef, event: StreamEvent) -> Result<Value> {
        let stream = self.store.load(stream_ref)?;
        Ok(json!({
            "stream_ref": stream.fields().stream_ref,
            "event": event,
            "cursor": stream.fields().cursor,
            "lifecycle": stream.fields().lifecycle,
        }))
    }

    /// Load fresh under the store's law, map, then append: a competing writer
    /// yields a visible sequence refusal, never a silent overwrite.
    fn append_mapped<F>(&self, stream_ref: &StreamRef, map: F) -> Result<(StreamEvent, Value)>
    where
        F: FnOnce(Count) -> Result<StreamEvent>,
    {
        let stream = self.store.load(stream_ref)?;
        let sequence = stream.fields().cursor.fields().next_sequence;
        let event = map(sequence)?;
        self.store.append(stream_ref, event.clone())?;
        let receipt = self.receipt(stream_ref, event.clone())?;
        Ok((event, receipt))
    }

    fn live_target(&self, request: &InvokeRequest) -> Option<Binding> {
        let bindings = self.bindings.lock().expect("binding table");
        let mut live: Vec<_> = bindings
            .values()
            .filter(|binding| {
                binding.grant.role == GrantRole::Agent
                    && binding.grant.agency_ref.as_ref() == Some(&request.target_agency_ref)
                    && request
                        .target_agent_session_ref
                        .as_ref()
                        .map_or(true, |session| &binding.agent_session_ref == session)
            })
            .cloned()
            .collect();
        live.sort_by_key(|binding| binding.connection);
        live.into_iter().next()
    }

    fn wait_for_return(
        &self,
        stream_ref: &StreamRef,
        return_ref: &actuation_core::ReturnRef,
        timeout: Duration,
    ) -> Result<Option<StreamEvent>> {
        let deadline = Instant::now() + timeout;
        loop {
            let stream = self.store.load(stream_ref)?;
            if let Some(found) =
                mapping::find_return(stream.fields().events.as_slice(), return_ref)
            {
                return Ok(Some(found.clone()));
            }
            if Instant::now() >= deadline || self.shutdown.load(Ordering::SeqCst) {
                return Ok(None);
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Co-internal invocation: policy admission first, a refusal retained where
    /// the attempt happened when refused, a delegation event in the target's
    /// own stream when allowed, then the correlated attributable Return.
    fn invoke(self: &Arc<Self>, binding: &Binding, request: &InvokeRequest) -> Reply {
        let controller_agency = match binding.grant.agency_ref.as_ref() {
            Some(agency) => agency,
            None => return Reply::Denied("invoking requires a granted controller Agency".into()),
        };
        if !binding.grant.may_invoke {
            return Reply::Denied(format!(
                "subject {} is not granted invocation authority",
                binding.grant.subject
            ));
        }
        let refusal = |reason: String| -> Reply {
            match self.append_mapped(&binding.grant.stream_ref, |sequence| {
                mapping::invocation_refusal(
                    &binding.grant,
                    mapping::mint_event_ref()?,
                    sequence,
                    request,
                    &reason,
                )
            }) {
                Ok((_, receipt)) => Reply::Ok(
                    json!({"denied": true, "reason": reason, "refusal_receipt": receipt})
                        .as_object()
                        .expect("object")
                        .clone(),
                ),
                Err(error) => Reply::Error(error.to_string()),
            }
        };
        if let Err(error) =
            self.policy
                .invocation(controller_agency, &request.target_agency_ref, request.mode)
        {
            return refusal(error.to_string());
        }
        let target = match self.live_target(request) {
            Some(target) => target,
            None => {
                return refusal(format!(
                    "no live resident session for {} on this gateway",
                    request.target_agency_ref
                ))
            }
        };
        let return_ref = match request.return_ref.clone() {
            Some(return_ref) => return_ref,
            None => match mapping::mint_return_ref() {
                Ok(return_ref) => return_ref,
                Err(error) => return Reply::Error(error.to_string()),
            },
        };
        let delegation_receipt = match self.append_mapped(&target.grant.stream_ref, |sequence| {
            mapping::delegation_event(
                &binding.grant,
                mapping::mint_event_ref()?,
                sequence,
                request,
                return_ref.clone(),
            )
        }) {
            Ok((_, receipt)) => receipt,
            Err(error) => return Reply::Error(error.to_string()),
        };
        let timeout = Duration::from_millis(
            request
                .timeout_ms
                .unwrap_or(DEFAULT_INVOKE_TIMEOUT_MS)
                .min(self.config.max_wait_ms),
        );
        match self.wait_for_return(&target.grant.stream_ref, &return_ref, timeout) {
            Ok(Some(return_event)) => Reply::Ok(
                json!({
                    "invocation_ref": request.invocation_ref,
                    "mode": request.mode.as_str(),
                    "target_stream_ref": target.grant.stream_ref,
                    "target_agent_session_ref": target.agent_session_ref,
                    "delegation_receipt": delegation_receipt,
                    "return_event": return_event,
                })
                .as_object()
                .expect("object")
                .clone(),
            ),
            Ok(None) => Reply::Error(
                json!({
                    "timed_out": true,
                    "invocation_ref": request.invocation_ref,
                    "return_ref": return_ref,
                    "error": format!(
                        "invocation {} timed out awaiting Return on {}",
                        request.invocation_ref, target.grant.stream_ref
                    ),
                })
                .to_string(),
            ),
            Err(error) => Reply::Error(error.to_string()),
        }
    }

    fn handle_attach(
        &self,
        binding: &mut Option<Binding>,
        subject: &str,
        frame: AttachFrame,
    ) -> Reply {
        if binding.is_some() {
            return Reply::Error("connection is already attached".into());
        }
        let grant = match self.policy.attach_grant(subject, &frame.stream_ref) {
            Some(grant) => grant.clone(),
            None => {
                return Reply::Denied(format!(
                    "subject {subject} is not granted attach on {}",
                    frame.stream_ref
                ))
            }
        };
        if grant.role == GrantRole::Agent {
            if let Err(error) = self.policy.agent_grant(subject, &frame.stream_ref) {
                return Reply::Denied(error.to_string());
            }
        }
        // Canonical identity is supplied here and checked by the store against
        // the stream's own durable header. The gateway never mints a session.
        let opening = OpenStream {
            stream_ref: frame.stream_ref.clone(),
            actuation_ref: frame.actuation_ref,
            agency_ref: frame.agency_ref,
            agent_session_ref: frame.agent_session_ref.clone(),
            world_binding_ref: frame.world_binding_ref,
            provenance: frame.provenance,
            started_at: frame.started_at,
        };
        let stream = match self.store.open(&opening) {
            Ok(stream) => stream,
            Err(error) => return Reply::Error(error.to_string()),
        };
        let fields = stream.fields();
        let connection = self.next_connection.fetch_add(1, Ordering::SeqCst);
        let record = Binding {
            connection,
            grant,
            agent_session_ref: frame.agent_session_ref,
            conversation: frame.conversation,
        };
        let role = record.grant.role;
        let surface = record.grant.surface_ref.clone();
        let participant = record.grant.participant_ref.clone();
        let agency = record.grant.agency_ref.clone();
        let agent = record.grant.agent_ref.clone();
        let locus = record.grant.locus_ref.clone();
        self.bindings
            .lock()
            .expect("binding table")
            .insert(connection, record.clone());
        *binding = Some(record);
        Reply::Ok(
            json!({
                "role": match role {
                    GrantRole::Connector => "connector",
                    GrantRole::Agent => "agent",
                },
                "stream_ref": fields.stream_ref,
                "actuation_ref": fields.actuation_ref,
                "agency_ref": fields.agency_ref,
                "agent_session_ref": fields.agent_session_ref,
                "lifecycle": fields.lifecycle,
                "cursor": fields.cursor,
                "surface_ref": surface,
                "participant_ref": participant,
                "agency_granted": agency,
                "agent_granted": agent,
                "locus_granted": locus,
            })
            .as_object()
            .expect("object")
            .clone(),
        )
    }

    fn handle_connection(self: &Arc<Self>, stream: UnixStream) {
        // Accepted sockets inherit the listener's non-blocking flag on some
        // platforms (BSD/macOS); this connection is synchronous blocking I/O.
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(Duration::from_secs(130)));
        let mut writer = match stream.try_clone() {
            Ok(writer) => writer,
            Err(_) => return,
        };
        let mut reader = std::io::BufReader::new(stream);
        let mut subject: Option<String> = None;
        let mut binding: Option<Binding> = None;
        loop {
            let frame = match crate::wire::read_frame(&mut reader) {
                Ok(Some(frame)) => frame,
                Ok(None) | Err(_) => break,
            };
            let op = match crate::wire::frame_op(&frame) {
                Ok(op) => op.to_string(),
                Err(error) => {
                    let _ = crate::wire::write_frame(&mut writer, &error_frame(&error.to_string()));
                    break;
                }
            };
            if op == "hello" {
                if subject.is_some() {
                    let _ = crate::wire::write_frame(&mut writer, &error_frame("already greeted"));
                    break;
                }
                match serde_json::from_value::<HelloFrame>(frame) {
                    Ok(hello) if hello.protocol != GATEWAY_CONTRACT => {
                        let _ = crate::wire::write_frame(
                            &mut writer,
                            &Reply::UnsupportedProtocol(hello.protocol).frame(),
                        );
                        break;
                    }
                    Ok(hello) => {
                        if let Some(expected) = &self.config.token {
                            if hello.token.as_deref() != Some(expected.as_str()) {
                                let _ = crate::wire::write_frame(
                                    &mut writer,
                                    &Reply::Denied("authentication failed".into()).frame(),
                                );
                                break;
                            }
                        }
                        let _ = crate::wire::write_frame(
                            &mut writer,
                            &Reply::Ok(
                                json!({
                                    "gateway": GATEWAY_IDENTITY,
                                    "contract": GATEWAY_CONTRACT,
                                    "subject": hello.subject,
                                    "store": self.store.root().display().to_string(),
                                })
                                .as_object()
                                .expect("object")
                                .clone(),
                            )
                            .frame(),
                        );
                        subject = Some(hello.subject);
                    }
                    Err(error) => {
                        let _ =
                            crate::wire::write_frame(&mut writer, &error_frame(&error.to_string()));
                        break;
                    }
                }
                continue;
            }
            let subject = match &subject {
                Some(subject) => subject.clone(),
                None => {
                    let _ = crate::wire::write_frame(
                        &mut writer,
                        &Reply::Denied("hello required before any operation".into()).frame(),
                    );
                    break;
                }
            };
            let reply = self.dispatch(&mut binding, &subject, op, frame);
            if crate::wire::write_frame(&mut writer, &reply.frame()).is_err() {
                break;
            }
        }
        if let Some(bound) = binding.take() {
            self.bindings
                .lock()
                .expect("binding table")
                .remove(&bound.connection);
        }
    }

    fn dispatch(
        self: &Arc<Self>,
        binding: &mut Option<Binding>,
        subject: &str,
        op: String,
        frame: Value,
    ) -> Reply {
        match op.as_str() {
            "status" => Reply::Ok(
                json!({
                    "gateway": GATEWAY_IDENTITY,
                    "contract": GATEWAY_CONTRACT,
                    "live_connections": self.bindings.lock().expect("binding table").len(),
                })
                .as_object()
                .expect("object")
                .clone(),
            ),
            "ping" => Reply::Ok(json!({"pong": true}).as_object().expect("object").clone()),
            "attach" => match serde_json::from_value::<AttachFrame>(frame) {
                Ok(frame) => self.handle_attach(binding, subject, frame),
                Err(error) => Reply::Error(error.to_string()),
            },
            "send" => match (binding.as_ref(), serde_json::from_value::<SendRequest>(frame)) {
                (Some(bound), Ok(request)) => {
                    match self.append_mapped(&bound.grant.stream_ref, |sequence| {
                        mapping::human_message(
                            &bound.grant,
                            mapping::mint_event_ref()?,
                            sequence,
                            request,
                        )
                    }) {
                        Ok((_, receipt)) => {
                            Reply::Ok(receipt.as_object().expect("object").clone())
                        }
                        Err(error) => Reply::Error(error.to_string()),
                    }
                }
                (None, _) => Reply::Denied("attach before sending".into()),
                (Some(_), Err(error)) => Reply::Error(error.to_string()),
            },
            "post" => match (binding.as_ref(), serde_json::from_value::<PostRequest>(frame)) {
                (Some(bound), Ok(request)) => {
                    match self.append_mapped(&bound.grant.stream_ref, |sequence| {
                        mapping::agent_event(
                            &bound.grant,
                            mapping::mint_event_ref()?,
                            sequence,
                            request,
                        )
                    }) {
                        Ok((_, receipt)) => {
                            Reply::Ok(receipt.as_object().expect("object").clone())
                        }
                        Err(error) => Reply::Error(error.to_string()),
                    }
                }
                (None, _) => Reply::Denied("attach before posting".into()),
                (Some(_), Err(error)) => Reply::Error(error.to_string()),
            },
            "replay" => match serde_json::from_value::<ReplayFrame>(frame) {
                Ok(frame) => match binding.as_ref() {
                    Some(bound) => {
                        match self.store.replay(
                            &bound.grant.stream_ref,
                            PageRequest {
                                after_sequence: frame.after.unwrap_or(Count::ZERO),
                                limit: frame.limit,
                            },
                        ) {
                            Ok(page) => Reply::Ok(
                                json!({"page": page})
                                    .as_object()
                                    .expect("object")
                                    .clone(),
                            ),
                            Err(error) => Reply::Error(error.to_string()),
                        }
                    }
                    None => Reply::Denied("attach before replay".into()),
                },
                Err(error) => Reply::Error(error.to_string()),
            },
            "wait" => match serde_json::from_value::<WaitFrame>(frame) {
                Ok(frame) => match binding.as_ref() {
                    Some(bound) => {
                        let timeout = Duration::from_millis(
                            frame
                                .timeout_ms
                                .unwrap_or(DEFAULT_WAIT_MS)
                                .min(self.config.max_wait_ms),
                        );
                        match self.wait_until(
                            &bound.grant.stream_ref,
                            frame.after,
                            frame.limit,
                            timeout,
                        ) {
                            Ok((timed_out, events, last_sequence)) => Reply::Ok(
                                json!({
                                    "timed_out": timed_out,
                                    "events": events,
                                    "last_sequence": last_sequence,
                                })
                                .as_object()
                                .expect("object")
                                .clone(),
                            ),
                            Err(error) => Reply::Error(error.to_string()),
                        }
                    }
                    None => Reply::Denied("attach before waiting".into()),
                },
                Err(error) => Reply::Error(error.to_string()),
            },
            "invoke" => match serde_json::from_value::<InvokeRequest>(frame) {
                Ok(request) => match binding.as_ref() {
                    Some(bound) => self.invoke(bound, &request),
                    None => Reply::Denied("attach before invoking".into()),
                },
                Err(error) => Reply::Error(error.to_string()),
            },
            "discover" => match binding.as_ref() {
                Some(_) => {
                    let streams: Vec<Value> = self
                        .policy
                        .visible_streams(subject)
                        .iter()
                        .map(|stream_ref| match self.store.load(stream_ref) {
                            Ok(stream) => json!({
                                "stream_ref": stream.fields().stream_ref,
                                "actuation_ref": stream.fields().actuation_ref,
                                "agency_ref": stream.fields().agency_ref,
                                "agent_session_ref": stream.fields().agent_session_ref,
                                "lifecycle": stream.fields().lifecycle,
                                "last_sequence": stream.fields().cursor.fields().last_sequence,
                            }),
                            Err(error) => json!({
                                "stream_ref": stream_ref,
                                "absent": true,
                                "error": error.to_string(),
                            }),
                        })
                        .collect();
                    Reply::Ok(json!({"streams": streams}).as_object().expect("object").clone())
                }
                None => Reply::Denied("attach before discovering".into()),
            },
            other => Reply::Error(format!("unknown op {other:?}")),
        }
    }

    fn wait_until(
        &self,
        stream_ref: &StreamRef,
        after: Count,
        limit: Option<Count>,
        timeout: Duration,
    ) -> Result<(bool, Vec<StreamEvent>, Count)> {
        let deadline = Instant::now() + timeout;
        loop {
            let stream = self.store.load(stream_ref)?;
            let last_sequence = stream.fields().cursor.fields().last_sequence;
            if last_sequence > after {
                let cap = limit
                    .map(|limit| usize::try_from(limit.get()).unwrap_or(usize::MAX))
                    .unwrap_or(usize::MAX);
                let events: Vec<_> = stream
                    .fields()
                    .events
                    .iter()
                    .filter(|event| event.fields().sequence > after)
                    .take(cap)
                    .cloned()
                    .collect();
                return Ok((false, events, last_sequence));
            }
            if Instant::now() >= deadline || self.shutdown.load(Ordering::SeqCst) {
                return Ok((true, Vec::new(), last_sequence));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }
}

/// A started gateway plus its accept thread, for tests and embedding.
pub struct GatewayHandle {
    pub gateway: Arc<Gateway>,
    thread: Option<JoinHandle<()>>,
}
impl GatewayHandle {
    pub fn stop(mut self) {
        self.gateway.shutdown.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Bind the socket, then serve on a background thread. The socket path is
/// owned by the operator (or the Workcell-managed service lifecycle); a stale
/// socket file from a dead process is removed before binding.
pub fn start(
    config: GatewayConfig,
    store: JsonlStreamStore,
    policy: GatewayPolicy,
) -> Result<GatewayHandle> {
    let _ = std::fs::remove_file(&config.socket_path);
    if let Some(parent) = config.socket_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::new(e.to_string()))?;
    }
    let listener = UnixListener::bind(&config.socket_path).map_err(|e| Error::new(e.to_string()))?;
    let gateway = Gateway::bind(config, store, policy)?;
    let thread_gateway = Arc::clone(&gateway);
    let thread = thread::spawn(move || thread_gateway.run(listener));
    Ok(GatewayHandle {
        gateway,
        thread: Some(thread),
    })
}
