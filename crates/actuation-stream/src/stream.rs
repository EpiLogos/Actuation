use crate::{domain, Count, Timestamp};
use crate::{EventKind, StreamEvent};
use actuation_core::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StreamSchema {
    #[serde(rename = "actuation.stream/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StreamState {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "interrupted")]
    Interrupted,
    #[serde(rename = "cancelled")]
    Cancelled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TerminalState {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "interrupted")]
    Interrupted,
    #[serde(rename = "cancelled")]
    Cancelled,
}
impl From<TerminalState> for StreamState {
    fn from(v: TerminalState) -> Self {
        match v {
            TerminalState::Closed => Self::Closed,
            TerminalState::Interrupted => Self::Interrupted,
            TerminalState::Cancelled => Self::Cancelled,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StreamLifecycleFields {
    pub state: StreamState,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub started_at: Slot<Timestamp>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub ended_at: Slot<Timestamp>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    StreamLifecycle,
    StreamLifecycleFields,
    [state, started_at, ended_at],
    |v: &Self| {
        if (v.state == StreamState::Open) == v.ended_at.value().is_some() {
            Err(Error::new(
                "terminal lifecycle requires ended_at; open lifecycle forbids it",
            ))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StreamCursorFields {
    pub last_sequence: Count,
    pub next_sequence: Count,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    StreamCursor,
    StreamCursorFields,
    [last_sequence, next_sequence],
    |v: &Self| {
        if v.next_sequence != v.last_sequence.next()? {
            Err(Error::new("cursor must be contiguous"))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActuationStreamFields {
    pub schema: StreamSchema,
    pub stream_ref: StreamRef,
    pub actuation_ref: ActuationRef,
    pub agency_ref: AgencyRef,
    pub agent_session_ref: AgentSessionRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub world_binding_ref: Slot<WorldBindingRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub participating_loci: Slot<Vec<LocusRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub surface_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub provenance: Slot<Vec<ExternalRef>>,
    pub lifecycle: StreamLifecycle,
    pub cursor: StreamCursor,
    pub events: Vec<StreamEvent>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    ActuationStream,
    ActuationStreamFields,
    [
        schema,
        stream_ref,
        actuation_ref,
        agency_ref,
        agent_session_ref,
        world_binding_ref,
        participating_loci,
        surface_refs,
        provenance,
        lifecycle,
        cursor,
        events
    ],
    |v: &Self| {
        let identities = [
            v.stream_ref.as_str(),
            v.actuation_ref.as_str(),
            v.agency_ref.as_str(),
            v.agent_session_ref.as_str(),
        ];
        if identities.into_iter().collect::<HashSet<_>>().len() != identities.len() {
            return Err(Error::new(
                "Stream, Actuation, Agency and Session identities must remain distinct",
            ));
        }
        let mut seen = HashSet::new();
        for (i, event) in v.events.iter().enumerate() {
            event.assert_sequence(Count::new(i as u64 + 1)?)?;
            if !seen.insert(&event.fields().event_ref) {
                return Err(Error::new("duplicate event_ref in stream"));
            }
        }
        if v.cursor.fields().last_sequence.get() != v.events.len() as u64 {
            return Err(Error::new(
                "cursor must exactly describe the portable event sequence",
            ));
        }
        Ok(())
    }
);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageRequest {
    pub after_sequence: Count,
    pub limit: Option<Count>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PageCursor {
    pub after_sequence: Count,
    pub returned_through: Count,
    pub stream_last_sequence: Count,
    pub has_more: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StreamPage {
    pub schema: StreamSchema,
    pub stream_ref: StreamRef,
    pub actuation_ref: ActuationRef,
    pub agency_ref: AgencyRef,
    pub agent_session_ref: AgentSessionRef,
    pub world_binding_ref: Option<WorldBindingRef>,
    pub participating_loci: Vec<LocusRef>,
    pub surface_refs: Vec<ExternalRef>,
    pub lifecycle: StreamLifecycle,
    pub cursor: PageCursor,
    pub events: Vec<StreamEvent>,
    pub provenance: Vec<ExternalRef>,
}
impl ActuationStream {
    pub fn read(&self, request: PageRequest) -> StreamPage {
        let s = self.fields();
        let events: Vec<_> = s
            .events
            .iter()
            .filter(|e| e.fields().sequence > request.after_sequence)
            .take(request.limit.map_or(usize::MAX, |n| {
                usize::try_from(n.get()).unwrap_or(usize::MAX)
            }))
            .cloned()
            .collect();
        let last = events
            .last()
            .map_or(request.after_sequence, |e| e.fields().sequence);
        StreamPage {
            schema: s.schema,
            stream_ref: s.stream_ref.clone(),
            actuation_ref: s.actuation_ref.clone(),
            agency_ref: s.agency_ref.clone(),
            agent_session_ref: s.agent_session_ref.clone(),
            world_binding_ref: s.world_binding_ref.value().cloned(),
            participating_loci: s.participating_loci.value().cloned().unwrap_or_default(),
            surface_refs: s.surface_refs.value().cloned().unwrap_or_default(),
            lifecycle: s.lifecycle.clone(),
            events,
            provenance: s.provenance.value().cloned().unwrap_or_default(),
            cursor: PageCursor {
                after_sequence: request.after_sequence,
                returned_through: last,
                stream_last_sequence: s.cursor.fields().last_sequence,
                has_more: last < s.cursor.fields().last_sequence,
            },
        }
    }
    pub fn append(&self, event: StreamEvent) -> Result<Self> {
        let s = self.fields();
        if s.lifecycle.fields().state != StreamState::Open {
            return Err(Error::new("cannot append to a terminal ActuationStream"));
        }
        event.assert_sequence(s.cursor.fields().next_sequence)?;
        if s.events
            .iter()
            .any(|e| e.fields().event_ref == event.fields().event_ref)
        {
            return Err(Error::new("event_ref already exists"));
        }
        let mut next = s.clone();
        let mut cursor = s.cursor.fields().clone();
        cursor.last_sequence = event.fields().sequence;
        cursor.next_sequence = event.fields().sequence.next()?;
        next.cursor = StreamCursor::new(cursor)?;
        next.events.push(event);
        Self::new(next)
    }
    pub fn close(&self, state: TerminalState, ended_at: Timestamp) -> Result<Self> {
        if self.fields().lifecycle.fields().state != StreamState::Open {
            return Err(Error::new("ActuationStream is already terminal"));
        }
        let mut next = self.fields().clone();
        let mut lifecycle = next.lifecycle.into_fields();
        lifecycle.state = state.into();
        lifecycle.ended_at = Slot::Value(ended_at);
        next.lifecycle = StreamLifecycle::new(lifecycle)?;
        Self::new(next)
    }
    /// A header is not a second event journal. Event lines own sequence truth.
    pub fn header(&self) -> Value {
        let mut header = crate::wire::json(self);
        header["events"] = json!([]);
        header["cursor"] = json!({"last_sequence":0,"next_sequence":1});
        header
    }
    pub fn usage(&self, usage_ref: &ExternalRef) -> Option<&StreamEvent> {
        self.fields().events.iter().find(|e| {
            e.fields().kind == EventKind::ModelUsage
                && e.fields()
                    .model_usage
                    .value()
                    .is_some_and(|u| &u.fields().usage_ref == usage_ref)
        })
    }
}
