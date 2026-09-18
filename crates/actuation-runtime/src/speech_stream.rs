//! Attribution of admitted speech/Nara receipts to a persisted
//! ActuationStream (Actuation #94/#91 follow-up).
//!
//! The speech lane's receipts are admitted wire records; this module is the
//! seam that makes them durable. [`SpeechLaneReceipt`] is a typed wrapper
//! over every receipt the lane produces, and [`SpeechReceiptStream`] appends
//! one to a real [`StreamStore`] as its own event — the receipt document is
//! carried verbatim under `metadata.receipt`, byte-faithful through append,
//! fold and replay. Nothing here re-admits, reshapes or summarises a
//! receipt; the admission already happened when the receipt was produced.
//!
//! Attribution comes from the receipt itself (the Agent whose body, decision
//! or interruption it records; `decided_by` for decisions). The recorder
//! guards the stream's Actuation identity and refuses to substitute another.

use crate::{NaraInterruptionReceipt, NARA_DELEGATION_VERSION, NARA_ENRICHMENT_VERSION};
use actuation_adapters::{
    SpeechConstitution, SpeechConstitutionChange, SpeechToolDecision,
    SPEECH_CONSTITUTION_CHANGE_VERSION, SPEECH_CONSTITUTION_VERSION, SPEECH_TOOL_DECISION_VERSION,
};
use actuation_core::{
    ActuationRef, AgentRef, Error, EventRef, Extensions, ExternalRef, Result, Slot, StreamRef,
};
use actuation_stream::{
    Attribution, AttributionFields, Disclosure, EventKind, JsonObject, StreamEvent,
    StreamEventFields, StreamStore, Timestamp,
};
use serde_json::{json, Value};

/// One admitted receipt of the speech/Nara lane, ready to be recorded.
#[derive(Clone, Debug)]
pub enum SpeechLaneReceipt<'a> {
    Constitution(&'a SpeechConstitution),
    ConstitutionChange(&'a SpeechConstitutionChange),
    ToolDecision(&'a SpeechToolDecision),
    Interruption(&'a crate::SpeechInterruptionReceipt),
    NaraInterruption(&'a NaraInterruptionReceipt),
    /// The `actuation.nara-delegation/v1` receipt value
    /// [`crate::NaraBinding::delegate_to_epii`] produced.
    NaraDelegation(&'a Value),
    /// The `actuation.nara-enrichment/v1` receipt value
    /// [`crate::NaraBinding::receive_enrichment`] produced.
    NaraEnrichment(&'a Value),
}

impl SpeechLaneReceipt<'_> {
    /// The exact schema version the receipt was admitted under. This is the
    /// event's custom kind on the stream: the wire type of the receipt is
    /// never lost by recording it.
    pub fn schema(&self) -> Result<&'static str> {
        Ok(match self {
            Self::Constitution(_) => SPEECH_CONSTITUTION_VERSION,
            Self::ConstitutionChange(_) => SPEECH_CONSTITUTION_CHANGE_VERSION,
            Self::ToolDecision(_) => SPEECH_TOOL_DECISION_VERSION,
            Self::Interruption(_) => crate::SPEECH_INTERRUPTION_VERSION,
            Self::NaraInterruption(_) => crate::NARA_INTERRUPTION_VERSION,
            Self::NaraDelegation(_) => NARA_DELEGATION_VERSION,
            Self::NaraEnrichment(_) => NARA_ENRICHMENT_VERSION,
        })
    }
    /// The admitted receipt document, verbatim.
    pub fn document(&self) -> Result<Value> {
        let value = match self {
            Self::Constitution(r) => r.as_value().clone(),
            Self::ConstitutionChange(r) => r.as_value().clone(),
            Self::ToolDecision(r) => r.as_value().clone(),
            Self::Interruption(r) => r.as_value().clone(),
            Self::NaraInterruption(r) => r.as_value().clone(),
            Self::NaraDelegation(v) | Self::NaraEnrichment(v) => {
                let expected = self.schema()?;
                if !v.is_object() || v["schema"] != json!(expected) {
                    return Err(Error::new(format!(
                        "receipt document is not an admitted {expected}"
                    )));
                }
                (*v).clone()
            }
        };
        Ok(value)
    }
    /// The Agent the receipt is about, read from the receipt itself — never
    /// guessed from shape or supplied separately where the receipt already
    /// names it.
    fn agent_ref(&self) -> Result<Option<AgentRef>> {
        let named = |v: &Value, path: &[&str]| {
            let mut current = v;
            for key in path {
                current = &current[key];
            }
            current.as_str().map(AgentRef::new).transpose()
        };
        Ok(match self {
            Self::Constitution(r) => named(r.as_value(), &["agent_ref"])?,
            Self::ConstitutionChange(r) => named(r.as_value(), &["agent_ref"])?,
            Self::Interruption(r) => named(r.as_value(), &["agent_ref"])?,
            Self::NaraInterruption(r) => {
                named(r.as_value(), &["interruption_receipt", "agent_ref"])?
            }
            Self::NaraDelegation(v) => named(v, &["nara_agent_ref"])?.or(named(v, &["nara_ref"])?),
            Self::NaraEnrichment(v) => named(v, &["nara_ref"])?,
            // A decision's actor is its adjudicator: the receipt names who
            // decided, not only which session the request arrived on.
            Self::ToolDecision(_) => None,
        })
    }
    fn decided_by(&self) -> Option<ExternalRef> {
        match self {
            Self::ToolDecision(d) => d
                .as_value()
                .get("decided_by")
                .and_then(Value::as_str)
                .and_then(|s| ExternalRef::new(s).ok()),
            _ => None,
        }
    }
    fn attribution(&self) -> Result<Attribution> {
        let agent = self.agent_ref()?;
        let participant = self.decided_by();
        if agent.is_none() && participant.is_none() {
            return Err(Error::new(
                "the receipt names no Agent or adjudicator; it cannot be attributed",
            ));
        }
        Attribution::new(AttributionFields {
            locus_ref: Slot::Absent,
            agency_ref: Slot::Absent,
            agent_ref: match agent {
                Some(agent) => Slot::Value(agent),
                None => Slot::Absent,
            },
            participant_ref: match participant {
                Some(participant) => Slot::Value(participant),
                None => Slot::Absent,
            },
            extensions: Extensions::new(),
        })
    }
}

/// The stream side of the speech lane: appends admitted receipts to a real
/// stream store as attributable, portable events.
#[derive(Clone, Debug)]
pub struct SpeechReceiptStream<S, C = fn() -> Result<Timestamp>> {
    store: S,
    stream_ref: StreamRef,
    actuation_ref: ActuationRef,
    clock: C,
}

impl<S: StreamStore> SpeechReceiptStream<S> {
    pub fn new(store: S, stream_ref: StreamRef, actuation_ref: ActuationRef) -> Self {
        Self {
            store,
            stream_ref,
            actuation_ref,
            clock: Timestamp::now,
        }
    }
}

impl<S: StreamStore, C: FnMut() -> Result<Timestamp> + Send> SpeechReceiptStream<S, C> {
    /// Bind a specific clock: it witnesses reception of the receipt, not the
    /// receipt's own event time — the receipt carries its timestamps inside,
    /// verbatim.
    pub fn with_clock(
        store: S,
        stream_ref: StreamRef,
        actuation_ref: ActuationRef,
        clock: C,
    ) -> Self {
        Self {
            store,
            stream_ref,
            actuation_ref,
            clock,
        }
    }
    pub fn store(&self) -> &S {
        &self.store
    }
    /// Append one admitted receipt as its own stream event and return the
    /// event ref that now cites it. The receipt document rides
    /// `metadata.receipt` verbatim; the event's custom kind is the receipt's
    /// schema version.
    pub fn record(
        &mut self,
        event_ref: impl AsRef<str>,
        receipt: SpeechLaneReceipt,
    ) -> Result<EventRef> {
        let event_ref = EventRef::new(event_ref.as_ref())?;
        let stream = self.store.load(&self.stream_ref)?;
        if stream.fields().actuation_ref != self.actuation_ref {
            return Err(Error::new(
                "speech receipt stream cannot substitute Actuation identity",
            ));
        }
        let schema = receipt.schema()?;
        let document = receipt.document()?;
        let mut metadata = JsonObject::new();
        metadata.insert("receipt_schema".into(), json!(schema));
        metadata.insert("receipt".into(), document);
        let event = StreamEvent::new(StreamEventFields {
            event_ref: event_ref.clone(),
            sequence: stream.fields().cursor.fields().next_sequence,
            kind: EventKind::Custom,
            custom_kind: Slot::Value(ExternalRef::new(schema)?),
            observed_at: Slot::Value((self.clock)()?),
            actor: Slot::Value(receipt.attribution()?),
            surface_ref: Slot::Absent,
            execution_ref: Slot::Absent,
            return_ref: Slot::Absent,
            native_trace_ref: Slot::Absent,
            resource_refs: Slot::Absent,
            evidence_refs: Slot::Absent,
            disclosure: Slot::Value(Disclosure::Portable),
            content: Slot::Absent,
            metadata: Slot::Value(metadata),
            model_usage: Slot::Absent,
            extensions: Extensions::new(),
        })?;
        self.store.append(&self.stream_ref, event)?;
        Ok(event_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actuation_adapters::{
        adjudicate_speech_tool_request, SpeechToolRequest, SPEECH_CONSTITUTION_VERSION,
        SPEECH_TOOL_DECISION_VERSION,
    };
    use actuation_core::{AgencyRef, AgentSessionRef};
    use actuation_stream::{JsonlStreamStore, OpenStream};
    use serde_json::{json, Value};

    fn constitution(body: &str, interruption: Value) -> SpeechConstitution {
        SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": format!("constitution:{body}"),
            "agent_ref": "nara:canonical",
            "agency_ref": "agency:nara",
            "world_binding_ref": "binding:nara",
            "agent_session_ref": "session:nara-1",
            "body_ref": format!("model-surface:{body}"),
            "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
                "inference_surface": {"contract_ref": "contract:opaque"}},
            "access_profile": {"schema": "actuation.instantiation/v1",
                "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
            "input_modalities": ["text", "speech"],
            "output_modalities": ["text", "speech"],
            "interaction": {"request-response": {"state": "supported"},
                "tool-requests": {"state": "supported"}},
            "transport": "websocket",
            "connection": {"kind": "connected", "reconnect": "resumable"},
            "interruption": interruption,
            "provider_binding": {"provider_ref": "provider:realtime",
                "provider_session_ref": "provider-session:abc"},
            "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
            "resolved_at": "2026-09-18T09:00:00Z"
        }))
        .expect("fixture constitution must be valid")
    }

    #[test]
    fn a_refused_decision_is_attributed_to_its_adjudicator() {
        let constitution = constitution("realtime", json!({"state": "supported"}));
        let request = SpeechToolRequest::try_from(json!({
            "schema": SPEECH_TOOL_DECISION_VERSION,
            "request_ref": "request:1",
            "constitution_ref": "constitution:realtime",
            "agent_session_ref": "session:nara-1",
            "proposed_action_ref": "action:not-listed",
            "payload_refs": ["ref:1"],
            "requested_at": "2026-09-18T09:00:01Z"
        }))
        .unwrap();
        let decision = adjudicate_speech_tool_request(
            "decision:1",
            &constitution,
            request,
            &[],
            &[],
            "human:owner",
            "2026-09-18T09:00:02Z",
        )
        .unwrap();
        let receipt = SpeechLaneReceipt::ToolDecision(&decision);
        assert_eq!(receipt.schema().unwrap(), SPEECH_TOOL_DECISION_VERSION);
        assert!(receipt.agent_ref().unwrap().is_none());
        let attribution = receipt.attribution().unwrap();
        assert_eq!(
            attribution
                .fields()
                .participant_ref
                .value()
                .unwrap()
                .as_str(),
            "human:owner"
        );
        assert_eq!(receipt.document().unwrap(), *decision.as_value());
    }

    #[test]
    fn a_non_admitted_value_is_refused_at_record_input() {
        let constitution = constitution("realtime", json!({"state": "supported"}));
        let ok = SpeechLaneReceipt::Constitution(&constitution);
        assert_eq!(ok.document().unwrap(), *constitution.as_value());
        let foreign = json!({
            "schema": "actuation.speech-constitution/v9",
            "constitution_ref": "constitution:fake"
        });
        let bad = SpeechLaneReceipt::NaraDelegation(&foreign);
        assert!(bad.document().is_err());
        let bad = SpeechLaneReceipt::NaraEnrichment(&foreign);
        assert!(bad.document().is_err());
    }

    #[test]
    fn recording_refuses_to_substitute_actuation_identity() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonlStreamStore::new(dir.path()).unwrap();
        let stream_ref = StreamRef::new("stream:speech").unwrap();
        store
            .open(&OpenStream {
                stream_ref: stream_ref.clone(),
                actuation_ref: ActuationRef::new("actuation:nara-1").unwrap(),
                agency_ref: AgencyRef::new("agency:nara").unwrap(),
                agent_session_ref: AgentSessionRef::new("session:nara-1").unwrap(),
                world_binding_ref: None,
                provenance: None,
                started_at: None,
            })
            .unwrap();
        let mut other_identity = SpeechReceiptStream::new(
            store,
            stream_ref,
            ActuationRef::new("actuation:someone-else").unwrap(),
        );
        let constitution = constitution("realtime", json!({"state": "supported"}));
        assert!(other_identity
            .record(
                "actuation:event:refused",
                SpeechLaneReceipt::Constitution(&constitution)
            )
            .is_err());
    }
}
