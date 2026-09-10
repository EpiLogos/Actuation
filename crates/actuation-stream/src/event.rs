use crate::{domain, Count, JsonObject, Timestamp};
use actuation_core::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    #[serde(rename = "human-message")]
    HumanMessage,
    #[serde(rename = "model-message")]
    ModelMessage,
    #[serde(rename = "model-delta")]
    ModelDelta,
    #[serde(rename = "model-result")]
    ModelResult,
    #[serde(rename = "model-usage")]
    ModelUsage,
    #[serde(rename = "capability-request")]
    CapabilityRequest,
    #[serde(rename = "capability-result")]
    CapabilityResult,
    #[serde(rename = "tool-request")]
    ToolRequest,
    #[serde(rename = "tool-result")]
    ToolResult,
    #[serde(rename = "harness-event")]
    HarnessEvent,
    #[serde(rename = "world-observation")]
    WorldObservation,
    #[serde(rename = "delegation")]
    Delegation,
    #[serde(rename = "locus-event")]
    LocusEvent,
    #[serde(rename = "permission")]
    Permission,
    #[serde(rename = "refusal")]
    Refusal,
    #[serde(rename = "interruption")]
    Interruption,
    #[serde(rename = "cancellation")]
    Cancellation,
    #[serde(rename = "execution-event")]
    ExecutionEvent,
    #[serde(rename = "artifact")]
    Artifact,
    #[serde(rename = "evidence")]
    Evidence,
    #[serde(rename = "return")]
    Return,
    #[serde(rename = "custom")]
    Custom,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Disclosure {
    #[serde(rename = "portable")]
    Portable,
    #[serde(rename = "surface")]
    Surface,
    #[serde(rename = "reference-only")]
    ReferenceOnly,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributionFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub locus_ref: Slot<LocusRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agency_ref: Slot<AgencyRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_ref: Slot<AgentRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub participant_ref: Slot<ExternalRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    Attribution,
    AttributionFields,
    [locus_ref, agency_ref, agent_ref, participant_ref],
    |v: &Self| {
        if v.locus_ref.value().is_none()
            && v.agency_ref.value().is_none()
            && v.agent_ref.value().is_none()
            && v.participant_ref.value().is_none()
        {
            Err(Error::new(
                "attribution requires a supplied locus, Agency, Agent or Participant",
            ))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StreamEventFields {
    pub event_ref: EventRef,
    pub sequence: Count,
    pub kind: EventKind,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub custom_kind: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub observed_at: Slot<Timestamp>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub actor: Slot<Attribution>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub surface_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub execution_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub return_ref: Slot<ReturnRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub native_trace_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub resource_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub disclosure: Slot<Disclosure>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub content: Slot<String>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub metadata: Slot<JsonObject>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub model_usage: Slot<crate::ModelUsageObservation>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    StreamEvent,
    StreamEventFields,
    [
        event_ref,
        sequence,
        kind,
        custom_kind,
        observed_at,
        actor,
        surface_ref,
        execution_ref,
        return_ref,
        native_trace_ref,
        resource_refs,
        evidence_refs,
        disclosure,
        content,
        metadata,
        model_usage
    ],
    |v: &Self| {
        if v.sequence == Count::ZERO {
            return Err(Error::new("event sequence must be positive"));
        }
        if (v.kind == EventKind::Custom) != v.custom_kind.value().is_some() {
            return Err(Error::new("only custom events require custom_kind"));
        }
        if let Some(actor) = v.actor.value() {
            if let Some(role) = actor
                .fields()
                .extensions
                .get("role")
                .filter(|v| !v.is_null())
            {
                crate::wire::text(role, "event actor role")?;
            }
        }
        if v.disclosure.value() == Some(&Disclosure::ReferenceOnly) && v.content.value().is_some() {
            return Err(Error::new("reference-only events must not inline content"));
        }
        if (v.kind == EventKind::ModelUsage) != v.model_usage.value().is_some() {
            return Err(Error::new("only model-usage events require model_usage"));
        }
        if v.kind == EventKind::Return && v.return_ref.value().is_none() {
            return Err(Error::new("return event requires return_ref"));
        }
        if matches!(v.kind, EventKind::Artifact | EventKind::Evidence)
            && v.resource_refs.value().is_none_or(Vec::is_empty)
            && v.evidence_refs.value().is_none_or(Vec::is_empty)
        {
            return Err(Error::new(
                "artifact/evidence event requires a resource or evidence ref",
            ));
        }
        Ok(())
    }
);

impl StreamEvent {
    pub fn assert_sequence(&self, expected: Count) -> Result<()> {
        if self.fields().sequence == expected {
            Ok(())
        } else {
            Err(Error::new(format!(
                "event sequence must equal {}",
                expected.get()
            )))
        }
    }
}
