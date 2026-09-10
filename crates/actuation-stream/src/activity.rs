use crate::{domain, Count, JsonObject, Timestamp};
use crate::{ActuationStream, Attribution, AttributionFields, StreamEvent, StreamState};
use actuation_core::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActivitySchema {
    #[serde(rename = "actuation.activity/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActivityPhase {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "interrupted")]
    Interrupted,
    #[serde(rename = "cancelled")]
    Cancelled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActivityOutcome {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "refused")]
    Refused,
    #[serde(rename = "cancelled")]
    Cancelled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Salience {
    #[serde(rename = "ambient")]
    Ambient,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "important")]
    Important,
    #[serde(rename = "critical")]
    Critical,
}
impl ActivityPhase {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Interrupted | Self::Cancelled
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityTraceFields {
    pub stream_ref: StreamRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub event_refs: Slot<Vec<EventRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub native_trace_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub from_sequence: Slot<Count>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub through_sequence: Slot<Count>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    ActivityTrace,
    ActivityTraceFields,
    [
        stream_ref,
        event_refs,
        native_trace_refs,
        from_sequence,
        through_sequence
    ],
    |v: &Self| {
        if v.from_sequence.value() == Some(&Count::ZERO)
            || v.through_sequence.value() == Some(&Count::ZERO)
        {
            return Err(Error::new("trace sequence must be positive"));
        }
        if let (Some(from), Some(through)) = (v.from_sequence.value(), v.through_sequence.value()) {
            if through < from {
                return Err(Error::new("trace end must not precede its beginning"));
            }
        }
        Ok(())
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityFields {
    pub schema: ActivitySchema,
    pub activity_ref: ActivityRef,
    pub actor: Attribution,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_session_ref: Slot<AgentSessionRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub plan_ref: Slot<PlanRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub journey_ref: Slot<JourneyRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub run_ref: Slot<RunRef>,
    pub subject_ref: ExternalRef,
    pub native_owner: ExternalRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub action_ref: Slot<ActionRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub invocation_ref: Slot<InvocationRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub actuation_ref: Slot<ActuationRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub result_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub return_ref: Slot<ReturnRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub usage_refs: Slot<Vec<ExternalRef>>,
    pub verb: ExternalRef,
    pub object: ExternalRef,
    pub summary: ExternalRef,
    pub phase: ActivityPhase,
    pub outcome: ActivityOutcome,
    pub salience: Salience,
    pub needs_attention: bool,
    pub trace: ActivityTrace,
    pub started_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub completed_at: Slot<Timestamp>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub metadata: Slot<JsonObject>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    Activity,
    ActivityFields,
    [
        schema,
        activity_ref,
        actor,
        agent_session_ref,
        plan_ref,
        journey_ref,
        run_ref,
        subject_ref,
        native_owner,
        action_ref,
        invocation_ref,
        actuation_ref,
        result_ref,
        return_ref,
        evidence_refs,
        usage_refs,
        verb,
        object,
        summary,
        phase,
        outcome,
        salience,
        needs_attention,
        trace,
        started_at,
        updated_at,
        completed_at,
        metadata
    ],
    |v: &Self| {
        if v.phase.is_terminal() != v.completed_at.value().is_some() {
            Err(Error::new(
                "terminal Activity requires completed_at; non-terminal Activity forbids it",
            ))
        } else {
            Ok(())
        }
    }
);

/// Supplied semantic descriptions of an observed act; none of these fields
/// discovers caller identity, creates an Action, or establishes authority.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDescription {
    pub activity_ref: ActivityRef,
    pub subject_ref: ExternalRef,
    pub native_owner: ExternalRef,
    pub verb: ExternalRef,
    pub object: ExternalRef,
    pub summary: ExternalRef,
    #[serde(default = "normal_salience")]
    pub salience: Salience,
    #[serde(default)]
    pub needs_attention: bool,
    #[serde(default)]
    pub action_ref: Slot<ActionRef>,
    #[serde(default)]
    pub invocation_ref: Slot<InvocationRef>,
    #[serde(default)]
    pub plan_ref: Option<PlanRef>,
    #[serde(default)]
    pub journey_ref: Option<JourneyRef>,
    #[serde(default)]
    pub run_ref: Option<RunRef>,
    #[serde(default)]
    pub result_ref: Slot<ExternalRef>,
    #[serde(default)]
    pub return_ref: Slot<ReturnRef>,
    #[serde(default)]
    pub evidence_refs: Vec<ExternalRef>,
    #[serde(default)]
    pub outcome: Slot<ActivityOutcome>,
    #[serde(default)]
    pub metadata: Slot<JsonObject>,
}
fn normal_salience() -> Salience {
    Salience::Normal
}
fn optional<T>(value: Option<T>) -> Slot<T> {
    value.map(Slot::Value).unwrap_or(Slot::Absent)
}
fn actor_for(stream: &ActuationStream, event: Option<&StreamEvent>) -> Result<Attribution> {
    let actor = event
        .and_then(|e| e.fields().actor.value())
        .map(Attribution::fields);
    Attribution::new(AttributionFields {
        agent_ref: optional(actor.and_then(|a| a.agent_ref.value()).cloned()),
        agency_ref: Slot::Value(
            actor
                .and_then(|a| a.agency_ref.value())
                .cloned()
                .unwrap_or_else(|| stream.fields().agency_ref.clone()),
        ),
        participant_ref: optional(actor.and_then(|a| a.participant_ref.value()).cloned()),
        locus_ref: optional(actor.and_then(|a| a.locus_ref.value()).cloned()),
        extensions: Extensions::new(),
    })
}
impl Activity {
    pub fn needs_attention(&self) -> bool {
        self.fields().needs_attention
    }
    pub fn from_stream(stream: &ActuationStream, d: ActivityDescription) -> Result<Self> {
        let s = stream.fields();
        let last = s.events.last();
        let phase = match s.lifecycle.fields().state {
            StreamState::Open => ActivityPhase::Running,
            StreamState::Closed => ActivityPhase::Completed,
            StreamState::Interrupted => ActivityPhase::Interrupted,
            StreamState::Cancelled => ActivityPhase::Cancelled,
        };
        let outcome = d.outcome.value().copied().unwrap_or(match phase {
            ActivityPhase::Completed => ActivityOutcome::Succeeded,
            ActivityPhase::Cancelled => ActivityOutcome::Cancelled,
            ActivityPhase::Interrupted => ActivityOutcome::Degraded,
            _ => ActivityOutcome::Pending,
        });
        let started = s
            .lifecycle
            .fields()
            .started_at
            .value()
            .or_else(|| {
                s.events
                    .first()
                    .and_then(|e| e.fields().observed_at.value())
            })
            .ok_or_else(|| {
                Error::new("Activity projection requires stream lifecycle/event timestamps")
            })?;
        let updated = s
            .lifecycle
            .fields()
            .ended_at
            .value()
            .or_else(|| last.and_then(|e| e.fields().observed_at.value()))
            .unwrap_or(started);
        let mut usages = Vec::new();
        for e in &s.events {
            if let Some(u) = e.fields().model_usage.value() {
                if !usages.contains(&u.fields().usage_ref) {
                    usages.push(u.fields().usage_ref.clone());
                }
            }
        }
        let trace = ActivityTrace::new(ActivityTraceFields {
            stream_ref: s.stream_ref.clone(),
            event_refs: Slot::Value(
                s.events
                    .iter()
                    .map(|e| e.fields().event_ref.clone())
                    .collect(),
            ),
            native_trace_refs: Slot::Value(
                s.events
                    .iter()
                    .filter_map(|e| e.fields().native_trace_ref.value().cloned())
                    .collect(),
            ),
            from_sequence: if s.events.is_empty() {
                Slot::Absent
            } else {
                Slot::Value(Count::ONE)
            },
            through_sequence: if s.events.is_empty() {
                Slot::Absent
            } else {
                Slot::Value(Count::new(s.events.len() as u64)?)
            },
            extensions: Extensions::new(),
        })?;
        let return_ref = d
            .return_ref
            .value()
            .cloned()
            .map(Slot::Value)
            .unwrap_or_else(|| {
                last.map(|e| e.fields().return_ref.clone())
                    .unwrap_or(Slot::Absent)
            });
        let metadata = optional(d.metadata.value().cloned());
        Self::new(ActivityFields {
            schema: ActivitySchema::V1,
            activity_ref: d.activity_ref,
            actor: actor_for(stream, last)?,
            agent_session_ref: Slot::Value(s.agent_session_ref.clone()),
            plan_ref: optional(d.plan_ref),
            journey_ref: optional(d.journey_ref),
            run_ref: optional(d.run_ref),
            subject_ref: d.subject_ref,
            native_owner: d.native_owner,
            action_ref: d.action_ref,
            invocation_ref: d.invocation_ref,
            actuation_ref: Slot::Value(s.actuation_ref.clone()),
            result_ref: d.result_ref,
            return_ref,
            evidence_refs: Slot::Value(d.evidence_refs),
            usage_refs: if usages.is_empty() {
                Slot::Absent
            } else {
                Slot::Value(usages)
            },
            verb: d.verb,
            object: d.object,
            summary: d.summary,
            phase,
            outcome,
            salience: d.salience,
            needs_attention: d.needs_attention,
            trace,
            started_at: started.clone(),
            updated_at: updated.clone(),
            completed_at: if phase.is_terminal() {
                Slot::Value(updated.clone())
            } else {
                Slot::Absent
            },
            metadata,
            extensions: Extensions::new(),
        })
    }
    pub fn from_event(
        stream: &ActuationStream,
        event_ref: &EventRef,
        d: ActivityDescription,
        phase: ActivityPhase,
    ) -> Result<Self> {
        let s = stream.fields();
        let event = s
            .events
            .iter()
            .find(|e| &e.fields().event_ref == event_ref)
            .ok_or_else(|| Error::new("ActuationStream contains no such event"))?;
        let e = event.fields();
        let observed = e
            .observed_at
            .value()
            .or(s.lifecycle.fields().started_at.value())
            .ok_or_else(|| Error::new("Activity event projection requires a timestamp"))?;
        let outcome = match d.outcome {
            Slot::Value(v) => v,
            Slot::Absent => ActivityOutcome::Succeeded,
            Slot::Null => return Err(Error::new("Activity outcome cannot be null")),
        };
        let mut evidence = d.evidence_refs;
        evidence.extend(e.evidence_refs.value().cloned().unwrap_or_default());
        let return_ref = d
            .return_ref
            .value()
            .cloned()
            .map(Slot::Value)
            .unwrap_or_else(|| e.return_ref.clone());
        Self::new(ActivityFields {
            schema: ActivitySchema::V1,
            activity_ref: d.activity_ref,
            actor: actor_for(stream, Some(event))?,
            agent_session_ref: Slot::Value(s.agent_session_ref.clone()),
            plan_ref: optional(d.plan_ref),
            journey_ref: optional(d.journey_ref),
            run_ref: optional(d.run_ref),
            subject_ref: d.subject_ref,
            native_owner: d.native_owner,
            action_ref: d.action_ref,
            invocation_ref: d.invocation_ref,
            actuation_ref: Slot::Value(s.actuation_ref.clone()),
            result_ref: d.result_ref,
            return_ref,
            evidence_refs: Slot::Value(evidence),
            usage_refs: optional(
                e.model_usage
                    .value()
                    .map(|u| vec![u.fields().usage_ref.clone()]),
            ),
            verb: d.verb,
            object: d.object,
            summary: d.summary,
            phase,
            outcome,
            salience: d.salience,
            needs_attention: d.needs_attention,
            trace: ActivityTrace::new(ActivityTraceFields {
                stream_ref: s.stream_ref.clone(),
                event_refs: Slot::Value(vec![e.event_ref.clone()]),
                native_trace_refs: Slot::Value(
                    e.native_trace_ref.value().cloned().into_iter().collect(),
                ),
                from_sequence: Slot::Value(e.sequence),
                through_sequence: Slot::Value(e.sequence),
                extensions: Extensions::new(),
            })?,
            started_at: observed.clone(),
            updated_at: observed.clone(),
            completed_at: if phase.is_terminal() {
                Slot::Value(observed.clone())
            } else {
                Slot::Absent
            },
            metadata: Slot::Absent,
            extensions: Extensions::new(),
        })
    }
}
