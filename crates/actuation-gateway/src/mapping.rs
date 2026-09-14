//! Mapping between gateway operations and canonical `ActuationStream` events.
//!
//! The gateway mints no second event ontology. Every write lands as a portable
//! `actuation.stream/v1` event in the caller's own durable store, with exact
//! attribution taken from the connection's grant — never from caller claims.
//! Provider-native conversation identity stays opaque metadata; it is never
//! collapsed into canonical AgentSession identity.

use crate::policy::{AttachGrant, GrantRole, InvocationMode};
use actuation_core::{
    AgencyRef, AgentSessionRef, Error, EventRef, Extensions, ExternalRef, Result, ReturnRef, Slot,
};
use actuation_stream::{
    Attribution, AttributionFields, Count, Disclosure, EventKind, StreamEvent, StreamEventFields,
    Timestamp,
};
use serde::Deserialize;
use serde_json::{json, Value};

/// Event kinds an agent-locus connection may post directly. `model-usage`
/// stays with the dedicated usage adapters, `harness-event` stays
/// descriptor-driven, and `custom` needs a contract this gateway does not
/// mint. `human-message` belongs to connector subjects.
pub const POSTABLE_KINDS: &[EventKind] = &[
    EventKind::ModelMessage,
    EventKind::ModelDelta,
    EventKind::ModelResult,
    EventKind::ToolRequest,
    EventKind::ToolResult,
    EventKind::CapabilityRequest,
    EventKind::CapabilityResult,
    EventKind::WorldObservation,
    EventKind::LocusEvent,
    EventKind::Permission,
    EventKind::Refusal,
    EventKind::Interruption,
    EventKind::Cancellation,
    EventKind::ExecutionEvent,
    EventKind::Artifact,
    EventKind::Evidence,
    EventKind::Return,
];

#[derive(Clone, Debug, Deserialize)]
pub struct SendRequest {
    pub content: String,
    #[serde(default)]
    pub conversation: Option<String>,
    #[serde(default)]
    pub observed_at: Option<Timestamp>,
    #[serde(default)]
    pub native_trace_ref: Option<ExternalRef>,
    #[serde(default)]
    pub metadata: Option<serde_json::Map<String, Value>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostRequest {
    pub kind: EventKind,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub return_ref: Option<ReturnRef>,
    #[serde(default)]
    pub resource_refs: Option<Vec<ExternalRef>>,
    #[serde(default)]
    pub evidence_refs: Option<Vec<ExternalRef>>,
    #[serde(default)]
    pub native_trace_ref: Option<ExternalRef>,
    #[serde(default)]
    pub execution_ref: Option<ExternalRef>,
    #[serde(default)]
    pub disclosure: Option<Disclosure>,
    #[serde(default)]
    pub observed_at: Option<Timestamp>,
    #[serde(default)]
    pub metadata: Option<serde_json::Map<String, Value>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InvokeRequest {
    pub target_agency_ref: AgencyRef,
    #[serde(default)]
    pub target_agent_session_ref: Option<AgentSessionRef>,
    pub mode: InvocationMode,
    pub invocation_ref: ExternalRef,
    #[serde(default)]
    pub return_ref: Option<ReturnRef>,
    pub payload: String,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

fn slot<T>(value: Option<T>) -> Slot<T> {
    value.map(Slot::Value).unwrap_or(Slot::Absent)
}

fn merge_metadata(
    subject: &str,
    key: &str,
    extra: Option<serde_json::Map<String, Value>>,
) -> Slot<serde_json::Map<String, Value>> {
    let mut metadata = serde_json::Map::new();
    metadata.insert(key.into(), json!(subject));
    if let Some(extra) = extra {
        for (name, value) in extra {
            metadata.insert(name, value);
        }
    }
    Slot::Value(metadata)
}

fn attribution(grant: &AttachGrant) -> Result<Attribution> {
    Attribution::new(AttributionFields {
        locus_ref: slot(grant.locus_ref.clone()),
        agency_ref: slot(grant.agency_ref.clone()),
        agent_ref: slot(grant.agent_ref.clone()),
        participant_ref: slot(grant.participant_ref.clone()),
        extensions: Extensions::new(),
    })
}

fn observed_at(supplied: Option<Timestamp>) -> Result<Slot<Timestamp>> {
    Ok(match supplied {
        Some(timestamp) => Slot::Value(timestamp),
        None => Slot::Value(Timestamp::now()?),
    })
}

fn require_connector(grant: &AttachGrant) -> Result<()> {
    if grant.role != GrantRole::Connector {
        return Err(Error::new(
            "human-message ingress belongs to connector-subject connections",
        ));
    }
    grant
        .participant_ref
        .as_ref()
        .ok_or_else(|| Error::new("connector grant must name its participant ref"))
        .map(|_| ())
}

/// Inbound connector material: one attributable `human-message` carrying the
/// granted participant and surface, with the provider conversation retained as
/// opaque metadata.
pub fn human_message(
    grant: &AttachGrant,
    event_ref: EventRef,
    sequence: Count,
    request: SendRequest,
) -> Result<StreamEvent> {
    require_connector(grant)?;
    let surface_ref = grant
        .surface_ref
        .clone()
        .ok_or_else(|| Error::new("connector grant must name its surface ref"))?;
    let mut metadata = serde_json::Map::new();
    metadata.insert("connector_subject".into(), json!(grant.subject));
    if let Some(conversation) = request.conversation {
        metadata.insert("conversation".into(), json!(conversation));
    }
    if let Some(extra) = request.metadata {
        for (name, value) in extra {
            metadata.insert(name, value);
        }
    }
    StreamEvent::new(StreamEventFields {
        event_ref,
        sequence,
        kind: EventKind::HumanMessage,
        custom_kind: Slot::Absent,
        observed_at: observed_at(request.observed_at.clone())?,
        actor: Slot::Value(attribution(grant)?),
        surface_ref: Slot::Value(surface_ref),
        execution_ref: Slot::Absent,
        return_ref: Slot::Absent,
        native_trace_ref: slot(request.native_trace_ref.clone()),
        resource_refs: Slot::Absent,
        evidence_refs: Slot::Absent,
        disclosure: Slot::Value(Disclosure::Portable),
        content: Slot::Value(request.content.clone()),
        metadata: Slot::Value(metadata),
        model_usage: Slot::Absent,
        extensions: Extensions::new(),
    })
}

/// Agent-locus material: the event is attributed to the granted locus exactly,
/// including Return correlation when the kind demands it.
pub fn agent_event(
    grant: &AttachGrant,
    event_ref: EventRef,
    sequence: Count,
    request: PostRequest,
) -> Result<StreamEvent> {
    if grant.role != GrantRole::Agent {
        return Err(Error::new(
            "posting stream material requires an agent-locus grant",
        ));
    }
    if !POSTABLE_KINDS.contains(&request.kind) {
        return Err(Error::new(format!(
            "kind {:?} is not a postable gateway event kind",
            request.kind
        )));
    }
    StreamEvent::new(StreamEventFields {
        event_ref,
        sequence,
        kind: request.kind,
        custom_kind: Slot::Absent,
        observed_at: observed_at(request.observed_at.clone())?,
        actor: Slot::Value(attribution(grant)?),
        surface_ref: slot(grant.surface_ref.clone()),
        execution_ref: slot(request.execution_ref.clone()),
        return_ref: slot(request.return_ref.clone()),
        native_trace_ref: slot(request.native_trace_ref.clone()),
        resource_refs: slot(request.resource_refs.clone()),
        evidence_refs: slot(request.evidence_refs.clone()),
        disclosure: slot(request.disclosure),
        content: slot(request.content.clone()),
        metadata: merge_metadata(&grant.subject, "gateway_subject", request.metadata),
        model_usage: Slot::Absent,
        extensions: Extensions::new(),
    })
}

fn invocation_metadata(
    request: &InvokeRequest,
    return_ref: &ReturnRef,
    reason: Option<&str>,
) -> serde_json::Map<String, Value> {
    let mut metadata = serde_json::Map::new();
    metadata.insert("mode".into(), json!(request.mode.as_str()));
    metadata.insert(
        "invocation_ref".into(),
        json!(request.invocation_ref.as_str()),
    );
    metadata.insert("return_ref".into(), json!(return_ref.as_str()));
    metadata.insert(
        "target_agency_ref".into(),
        json!(request.target_agency_ref.as_str()),
    );
    if let Some(session) = &request.target_agent_session_ref {
        metadata.insert("target_agent_session_ref".into(), json!(session.as_str()));
    }
    if let Some(reason) = reason {
        metadata.insert("reason".into(), json!(reason));
    }
    metadata
}

/// Co-internal invocation landing in the target's stream: the event is
/// attributed to the controller locus while the stream keeps its governing
/// Agency identity, and the Return is correlated in both directions.
pub fn delegation_event(
    controller: &AttachGrant,
    event_ref: EventRef,
    sequence: Count,
    request: &InvokeRequest,
    return_ref: ReturnRef,
) -> Result<StreamEvent> {
    let kind = match request.mode {
        InvocationMode::Delegation => EventKind::Delegation,
        InvocationMode::Communique | InvocationMode::SessionContribution => EventKind::LocusEvent,
    };
    StreamEvent::new(StreamEventFields {
        event_ref,
        sequence,
        kind,
        custom_kind: Slot::Absent,
        observed_at: observed_at(None)?,
        actor: Slot::Value(attribution(controller)?),
        surface_ref: slot(controller.surface_ref.clone()),
        execution_ref: Slot::Absent,
        return_ref: Slot::Value(return_ref.clone()),
        native_trace_ref: Slot::Absent,
        resource_refs: Slot::Value(vec![request.invocation_ref.clone()]),
        evidence_refs: Slot::Absent,
        disclosure: Slot::Value(Disclosure::Portable),
        content: Slot::Value(request.payload.clone()),
        metadata: Slot::Value(invocation_metadata(request, &return_ref, None)),
        model_usage: Slot::Absent,
        extensions: Extensions::new(),
    })
}

/// A refused invocation is retained where the attempt happened: attributed to
/// the controller locus, naming the invocation and the refusal reason.
pub fn invocation_refusal(
    controller: &AttachGrant,
    event_ref: EventRef,
    sequence: Count,
    request: &InvokeRequest,
    reason: &str,
) -> Result<StreamEvent> {
    let return_ref = ReturnRef::new(format!("return:refused:{}", request.invocation_ref))?;
    let mut metadata = invocation_metadata(request, &return_ref, Some(reason));
    metadata.insert("denied".into(), json!(true));
    StreamEvent::new(StreamEventFields {
        event_ref,
        sequence,
        kind: EventKind::Refusal,
        custom_kind: Slot::Absent,
        observed_at: observed_at(None)?,
        actor: Slot::Value(attribution(controller)?),
        surface_ref: slot(controller.surface_ref.clone()),
        execution_ref: Slot::Absent,
        return_ref: Slot::Absent,
        native_trace_ref: Slot::Absent,
        resource_refs: Slot::Value(vec![request.invocation_ref.clone()]),
        evidence_refs: Slot::Absent,
        disclosure: Slot::Value(Disclosure::Portable),
        content: Slot::Value(format!(
            "invocation {} to {} refused: {}",
            request.invocation_ref, request.target_agency_ref, reason
        )),
        metadata: Slot::Value(metadata),
        model_usage: Slot::Absent,
        extensions: Extensions::new(),
    })
}

/// The Return the gateway owes an invocation: a `return` event correlated by
/// its explicit `return_ref`, attributable to the locus that produced it.
pub fn find_return<'a>(
    events: &'a [StreamEvent],
    return_ref: &ReturnRef,
) -> Option<&'a StreamEvent> {
    events.iter().find(|event| {
        event.fields().kind == EventKind::Return
            && event.fields().return_ref.value() == Some(return_ref)
    })
}

fn random_hex(bytes: &mut [u8]) -> Result<String> {
    getrandom::fill(bytes).map_err(|e| Error::new(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn mint_event_ref() -> Result<EventRef> {
    let mut bytes = [0u8; 16];
    let hex = random_hex(&mut bytes)?;
    EventRef::new(format!(
        "actuation:event:{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

pub fn mint_return_ref() -> Result<ReturnRef> {
    let mut bytes = [0u8; 16];
    let hex = random_hex(&mut bytes)?;
    ReturnRef::new(format!(
        "actuation:return:{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn connector_grant() -> AttachGrant {
        serde_json::from_value(json!({
            "subject":"connector:cli","role":"connector","stream_ref":"stream:worker",
            "surface_ref":"surface:cli","participant_ref":"participant:cli-user"
        }))
        .unwrap()
    }

    fn agent_grant() -> AttachGrant {
        serde_json::from_value(json!({
            "subject":"agent:worker-1","role":"agent","stream_ref":"stream:worker",
            "agency_ref":"agency:worker","agent_ref":"agent:worker","locus_ref":"locus:worker"
        }))
        .unwrap()
    }

    fn send_request() -> SendRequest {
        serde_json::from_value(json!({
            "content":"challenge","conversation":"telegram-chat-42",
            "observed_at":"2026-09-13T09:00:00Z"
        }))
        .unwrap()
    }

    #[test]
    fn human_message_keeps_conversation_identity_out_of_session_identity() {
        let event = human_message(
            &connector_grant(),
            EventRef::new("event:in").unwrap(),
            Count::new(1).unwrap(),
            send_request(),
        )
        .unwrap();
        let fields = event.fields();
        assert_eq!(fields.kind, EventKind::HumanMessage);
        assert_eq!(
            fields.content.value().map(String::as_str),
            Some("challenge")
        );
        assert_eq!(
            fields
                .actor
                .value()
                .unwrap()
                .fields()
                .participant_ref
                .value(),
            Some(&ExternalRef::new("participant:cli-user").unwrap())
        );
        assert!(fields
            .actor
            .value()
            .unwrap()
            .fields()
            .agency_ref
            .value()
            .is_none());
        assert_eq!(
            fields.surface_ref.value().map(|r| r.as_str()),
            Some("surface:cli")
        );
        // The provider conversation id is opaque metadata only.
        assert_eq!(
            fields.metadata.value().unwrap().get("conversation"),
            Some(&json!("telegram-chat-42"))
        );
    }

    #[test]
    fn connector_grant_is_required_for_ingress() {
        let wrong = serde_json::from_value::<AttachGrant>(json!({
            "subject":"agent:worker-1","role":"agent","stream_ref":"stream:worker",
            "agency_ref":"agency:worker"
        }))
        .unwrap();
        let error = human_message(
            &wrong,
            EventRef::new("event:in").unwrap(),
            Count::new(1).unwrap(),
            send_request(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("connector-subject"));
    }

    #[test]
    fn agent_events_carry_the_granted_locus_exactly() {
        let event = agent_event(
            &agent_grant(),
            EventRef::new("event:tool").unwrap(),
            Count::new(2).unwrap(),
            serde_json::from_value(json!({
                "kind":"tool-result","content":"4","evidence_refs":["tool:adder"],
                "observed_at":"2026-09-13T09:00:01Z"
            }))
            .unwrap(),
        )
        .unwrap();
        let actor = event.fields().actor.value().unwrap().fields();
        assert_eq!(
            actor.agency_ref.value().map(|r| r.as_str()),
            Some("agency:worker")
        );
        assert_eq!(
            actor.agent_ref.value().map(|r| r.as_str()),
            Some("agent:worker")
        );
        assert_eq!(
            actor.locus_ref.value().map(|r| r.as_str()),
            Some("locus:worker")
        );
        assert_eq!(
            event
                .fields()
                .metadata
                .value()
                .unwrap()
                .get("gateway_subject"),
            Some(&json!("agent:worker-1"))
        );
    }

    #[test]
    fn non_postable_kinds_are_refused_by_name() {
        for kind in ["model-usage", "harness-event", "custom", "human-message"] {
            let error = agent_event(
                &agent_grant(),
                mint_event_ref().unwrap(),
                Count::new(1).unwrap(),
                serde_json::from_value(json!({"kind":kind,"content":"x"})).unwrap(),
            )
            .unwrap_err();
            assert!(error.to_string().contains("not a postable"), "{kind}");
        }
    }

    #[test]
    fn delegation_preserves_target_stream_governing_agency_and_correlates_return() {
        let controller = serde_json::from_value::<AttachGrant>(json!({
            "subject":"agent:ctl-1","role":"agent","stream_ref":"stream:ctl",
            "agency_ref":"agency:ctl","agent_ref":"agent:ctl"
        }))
        .unwrap();
        let request: InvokeRequest = serde_json::from_value(json!({
            "target_agency_ref":"agency:worker","mode":"delegation",
            "invocation_ref":"invocation:1","payload":"do the thing"
        }))
        .unwrap();
        let return_ref = ReturnRef::new("return:1").unwrap();
        let event = delegation_event(
            &controller,
            EventRef::new("event:delegation").unwrap(),
            Count::new(3).unwrap(),
            &request,
            return_ref.clone(),
        )
        .unwrap();
        assert_eq!(event.fields().kind, EventKind::Delegation);
        // Controller is the actor; the governing Agency of the containing
        // stream remains the target's own, never rewritten by the gateway.
        assert_eq!(
            event
                .fields()
                .actor
                .value()
                .unwrap()
                .fields()
                .agency_ref
                .value(),
            Some(&AgencyRef::new("agency:ctl").unwrap())
        );
        assert_eq!(event.fields().return_ref.value(), Some(&return_ref));
        assert_eq!(
            event.fields().resource_refs.value().unwrap(),
            &vec![ExternalRef::new("invocation:1").unwrap()]
        );
        let metadata = event.fields().metadata.value().unwrap();
        assert_eq!(metadata.get("mode"), Some(&json!("delegation")));
        assert_eq!(
            metadata.get("target_agency_ref"),
            Some(&json!("agency:worker"))
        );
        let found = find_return(std::slice::from_ref(&event), &return_ref).is_none();
        assert!(found, "a delegation is not itself the Return");
    }

    #[test]
    fn refusal_retains_invocation_evidence_and_reason() {
        let controller = serde_json::from_value::<AttachGrant>(json!({
            "subject":"agent:ctl-1","role":"agent","stream_ref":"stream:ctl",
            "agency_ref":"agency:ctl","agent_ref":"agent:ctl"
        }))
        .unwrap();
        let request: InvokeRequest = serde_json::from_value(json!({
            "target_agency_ref":"agency:intruder","mode":"delegation",
            "invocation_ref":"invocation:2","payload":"x"
        }))
        .unwrap();
        let event = invocation_refusal(
            &controller,
            mint_event_ref().unwrap(),
            Count::new(1).unwrap(),
            &request,
            "invocation from agency:ctl to agency:intruder is not granted",
        )
        .unwrap();
        assert_eq!(event.fields().kind, EventKind::Refusal);
        assert_eq!(
            event.fields().resource_refs.value().unwrap(),
            &vec![ExternalRef::new("invocation:2").unwrap()]
        );
        assert_eq!(
            event.fields().metadata.value().unwrap().get("denied"),
            Some(&json!(true))
        );
        assert!(event
            .fields()
            .content
            .value()
            .unwrap()
            .contains("not granted"));
    }

    #[test]
    fn return_events_are_findable_by_their_explicit_ref() {
        let grant = agent_grant();
        let return_ref = ReturnRef::new("return:9").unwrap();
        let event = agent_event(
            &grant,
            mint_event_ref().unwrap(),
            Count::new(1).unwrap(),
            serde_json::from_value(json!({
                "kind":"return","content":"4","return_ref":"return:9"
            }))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            find_return(&[event], &return_ref).map(|e| e.fields().content.value()),
            Some(Some(&"4".to_string()))
        );
        // A return without its ref is refused by the portable contract itself.
        assert!(agent_event(
            &grant,
            mint_event_ref().unwrap(),
            Count::new(2).unwrap(),
            serde_json::from_value::<PostRequest>(json!({"kind":"return","content":"x"})).unwrap(),
        )
        .is_err());
    }
}
