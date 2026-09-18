//! Actuality of a constituted speech/audio body: turn phases, attributable
//! interruption and honest degradation (Actuation #94).
//!
//! The material constitution (resolved in `actuation-adapters::speech`) says
//! what the body can do. This module makes a session *behave* like that
//! truth:
//!
//! - a body whose interruption capability is not `supported` does not
//!   barge-in; the attempt is recorded as an honest refusal, and the
//!   in-flight response continues — degraded behaviour is recorded, never
//!   faked;
//! - interruption is attributable: the receipt names what was cancelled and
//!   which already-executed results stand;
//! - interruption is not destruction: the session, its identity and its
//!   constitution survive it;
//! - a body swap records a [`SpeechConstitutionChange`] and keeps the same
//!   Agent/Agency identity; a body that failed can fall back to a text-only
//!   constitution and the session stays usable.
//!
//! No audio packets, sample counts or transport frames appear anywhere in
//! these records: usage evidence rides refs to the usage observations, not
//! raw stream noise.

use actuation_adapters::{SpeechConstitution, SpeechConstitutionChange, SpeechSupport};
use actuation_core::{AgencyRef, AgentRef, AgentSessionRef, Error, ExternalRef, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const SPEECH_INTERRUPTION_VERSION: &str = "actuation.speech-interruption/v1";
/// The read model [`SpeechSession::read`] produces.
pub const SPEECH_SESSION_READ_VERSION: &str = "actuation.speech-session-read/v1";

/// The phase of the speech turn itself. Distinct from any stream lifecycle
/// state: a session can be open while its speech body is between turns.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpeechTurnPhase {
    /// No turn in progress. A text-only body rests here permanently.
    Idle,
    /// The body is receiving input.
    Listening,
    /// The body is producing output.
    Speaking,
    /// An in-flight response was cancelled by interruption.
    Interrupted,
    /// The last response ran to completion.
    Completed,
}

/// The live relation between one AgentSession and its constituted speech
/// body. Identity is carried, never minted: the session keeps the
/// AgentRef/AgencyRef its constitution was admitted under.
#[derive(Clone, Debug)]
pub struct SpeechSession {
    constitution: SpeechConstitution,
    phase: SpeechTurnPhase,
    /// The response currently in flight, when one is.
    in_flight: Option<ExternalRef>,
    /// Results that were actually executed before any interruption; they
    /// stand regardless of what is cancelled later.
    executed: Vec<ExternalRef>,
}

impl SpeechSession {
    /// Constitute a session on a resolved body. A text-only body constitutes
    /// exactly like a speech body: the difference is capability, not standing.
    pub fn constitute(constitution: SpeechConstitution) -> Result<Self> {
        if !constitution.body_usable() {
            return Err(Error::new(
                "a body recorded unavailable cannot constitute a live session; record the condition and resolve another body",
            ));
        }
        Ok(Self {
            constitution,
            phase: SpeechTurnPhase::Idle,
            in_flight: None,
            executed: Vec::new(),
        })
    }
    pub fn constitution(&self) -> &SpeechConstitution {
        &self.constitution
    }
    pub fn phase(&self) -> SpeechTurnPhase {
        self.phase
    }
    pub fn agent_ref(&self) -> AgentRef {
        self.constitution.agent_ref()
    }
    pub fn agency_ref(&self) -> AgencyRef {
        self.constitution.agency_ref()
    }
    pub fn agent_session_ref(&self) -> AgentSessionRef {
        self.constitution.agent_session_ref()
    }
    pub fn executed_results(&self) -> &[ExternalRef] {
        &self.executed
    }
    /// The resolved state of one interaction capability, straight from the
    /// constitution: the session behaves like this truth, not a hopeful one.
    pub fn interaction_support(&self, capability: &str) -> SpeechSupport {
        self.constitution.interaction_support(capability)
    }

    fn require_speech_input(&self) -> Result<()> {
        if !self.constitution.speech_capable() {
            return Err(Error::new(
                "the constituted body cannot hear speech; text input remains available",
            ));
        }
        Ok(())
    }
    /// Open the body's input. A text-only body refuses honestly: the refusal
    /// is the degraded behaviour, not a silent pretend-listen.
    pub fn begin_listening(&mut self) -> Result<()> {
        self.require_speech_input()?;
        require_phase(
            self.phase,
            &[
                SpeechTurnPhase::Idle,
                SpeechTurnPhase::Completed,
                SpeechTurnPhase::Interrupted,
            ],
            "listening",
        )?;
        self.phase = SpeechTurnPhase::Listening;
        Ok(())
    }
    /// Begin producing a response. `response_ref` names this response turn
    /// for later attribution.
    pub fn begin_response(&mut self, response_ref: ExternalRef) -> Result<()> {
        if self.constitution.speech_capable() {
            require_phase(
                self.phase,
                &[
                    SpeechTurnPhase::Listening,
                    SpeechTurnPhase::Idle,
                    SpeechTurnPhase::Completed,
                    SpeechTurnPhase::Interrupted,
                ],
                "a response",
            )?;
        }
        self.phase = SpeechTurnPhase::Speaking;
        self.in_flight = Some(response_ref);
        Ok(())
    }
    /// Record a result that was actually executed while a response ran.
    /// Executed work stands; a later interruption cancels what remains, not
    /// what already happened.
    pub fn commit_result(&mut self, result_ref: ExternalRef) -> Result<()> {
        require_phase(
            self.phase,
            &[SpeechTurnPhase::Speaking],
            "committing a result",
        )?;
        self.executed.push(result_ref);
        Ok(())
    }
    /// A response ran to completion.
    pub fn complete_response(&mut self) -> Result<()> {
        require_phase(self.phase, &[SpeechTurnPhase::Speaking], "completion")?;
        self.in_flight = None;
        self.phase = SpeechTurnPhase::Completed;
        Ok(())
    }

    /// Interrupt the in-flight response.
    ///
    /// - Where the body supports interruption the response is cancelled: the
    ///   receipt names the cancelled response and the executed results that
    ///   stand, and the phase becomes [`SpeechTurnPhase::Interrupted`]. The
    ///   session is not destroyed.
    /// - Where the body does not support interruption, the attempt is
    ///   recorded honestly as a refusal and nothing else changes: the
    ///   response continues, because pretending otherwise would falsify the
    ///   body.
    pub fn interrupt(
        &mut self,
        interruption_ref: impl AsRef<str>,
        reason: impl AsRef<str>,
        at: &str,
    ) -> SpeechInterruptionReceipt {
        let support = self.constitution.interruption_support();
        let in_flight = self.in_flight.clone();
        let mut base = Map::new();
        base.insert("schema".into(), json!(SPEECH_INTERRUPTION_VERSION));
        base.insert("interruption_ref".into(), json!(interruption_ref.as_ref()));
        base.insert(
            "agent_ref".into(),
            self.constitution.as_value()["agent_ref"].clone(),
        );
        base.insert(
            "agency_ref".into(),
            self.constitution.as_value()["agency_ref"].clone(),
        );
        base.insert(
            "agent_session_ref".into(),
            self.constitution.as_value()["agent_session_ref"].clone(),
        );
        base.insert(
            "constitution_ref".into(),
            json!(self.constitution.constitution_ref()),
        );
        base.insert(
            "body_ref".into(),
            self.constitution.as_value()["body_ref"].clone(),
        );
        base.insert("response_ref".into(), json!(in_flight));
        base.insert(
            "executed_refs".into(),
            json!(self.executed.iter().map(|r| r.as_str()).collect::<Vec<_>>()),
        );
        base.insert("reason".into(), json!(reason.as_ref()));
        base.insert("support".into(), support.as_value());
        base.insert("at".into(), json!(at));
        let (outcome, refusal_reason) = match (&support, &in_flight) {
            (SpeechSupport::Supported, Some(_)) => {
                self.in_flight = None;
                self.phase = SpeechTurnPhase::Interrupted;
                ("cancelled", None)
            }
            (SpeechSupport::Supported, None) => ("nothing-in-flight", None),
            (unsupported, _) => (
                "refused",
                Some(match unsupported {
                    SpeechSupport::Degraded { reason } => format!(
                        "interruption is degraded and cannot be trusted to cancel cleanly: {reason}"
                    ),
                    SpeechSupport::Unsupported { reason } => {
                        format!("the body does not support interruption: {reason}")
                    }
                    SpeechSupport::Unknown { reason } => format!(
                        "interruption is unproven on this body, and unknown never behaves as a yes: {reason}"
                    ),
                    SpeechSupport::Supported => {
                        "interruption is not supported on this body".to_string()
                    }
                }),
            ),
        };
        base.insert("outcome".into(), json!(outcome));
        base.insert(
            "refusal_reason".into(),
            refusal_reason.map(Value::String).unwrap_or(Value::Null),
        );
        base.insert("phase_after".into(), json!(self.phase));
        SpeechInterruptionReceipt::try_from(Value::Object(base))
            .expect("an interruption receipt built from an admitted session is valid")
    }

    /// Replace the body. The change receipt enforces identity preservation;
    /// the session continues with the new constitution and a cleared turn.
    pub fn replace_body(
        &mut self,
        change_ref: impl AsRef<str>,
        next: SpeechConstitution,
        reason: impl AsRef<str>,
        evidence_refs: Vec<ExternalRef>,
        at: &str,
    ) -> Result<SpeechConstitutionChange> {
        let change = SpeechConstitutionChange::record(
            change_ref,
            self.constitution.clone(),
            next.clone(),
            reason,
            evidence_refs,
            at,
        )?;
        self.constitution = next;
        self.phase = SpeechTurnPhase::Idle;
        self.in_flight = None;
        Ok(change)
    }

    /// The wire state a desktop client reads: current body, phase, and the
    /// four-state capabilities that matter to a speech UI.
    pub fn read(&self) -> Value {
        json!({
            "schema": SPEECH_SESSION_READ_VERSION,
            "agent_ref": self.constitution.as_value()["agent_ref"],
            "agency_ref": self.constitution.as_value()["agency_ref"],
            "agent_session_ref": self.constitution.as_value()["agent_session_ref"],
            "constitution_ref": self.constitution.constitution_ref(),
            "body_ref": self.constitution.as_value()["body_ref"],
            "speech_capable": self.constitution.speech_capable(),
            "realtime_capable": self.constitution.realtime_capable(),
            "phase": self.phase,
            "in_flight_response_ref": self.in_flight,
            "executed_refs": self.executed.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
            "interruption": self.constitution.interruption_support().as_value(),
            "full_duplex_realtime": self.constitution.interaction_support("full-duplex-realtime").as_value(),
            "vad_turn_detection": self.constitution.interaction_support("vad-turn-detection").as_value(),
            "barge_in": self.constitution.interaction_support("barge-in").as_value(),
            "final_transcripts": self.constitution.interaction_support("final-transcripts").as_value(),
            "reconnect": self.constitution.reconnect_support(),
        })
    }
}

fn require_phase(actual: SpeechTurnPhase, allowed: &[SpeechTurnPhase], what: &str) -> Result<()> {
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err(Error::new(format!("phase {actual:?} cannot begin {what}")))
    }
}

/// The attributable receipt of one interruption attempt. Either the response
/// was cancelled (with the executed results that stand), or the attempt was
/// honestly refused because the body cannot cancel — the law that degraded
/// behaviour is recorded, never faked.
pub fn validate_speech_interruption(v: &Value) -> Result<()> {
    require(
        v.is_object(),
        "speech interruption receipt must be an object",
    )?;
    require(
        v["schema"] == SPEECH_INTERRUPTION_VERSION,
        "wrong schema; expected actuation.speech-interruption/v1",
    )?;
    for key in [
        "interruption_ref",
        "agent_ref",
        "agency_ref",
        "agent_session_ref",
        "constitution_ref",
        "body_ref",
        "reason",
        "at",
    ] {
        text(&v[key])?;
    }
    one(
        &v["outcome"],
        &["cancelled", "nothing-in-flight", "refused"],
    )?;
    if v["outcome"] == "refused" {
        text(&v["refusal_reason"])?;
    }
    require(
        v["support"].is_object(),
        "interruption support state required",
    )?;
    require(
        v["agent_ref"] != v["body_ref"] && v["agent_session_ref"] != v["body_ref"],
        "a body is not an identity: interruption attribution must keep the roles distinct",
    )?;
    wire_date(&v["at"])?;
    no_secret_keys(v)
}

fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::new(message))
    }
}
fn text(v: &Value) -> Result<()> {
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|_| ())
        .ok_or_else(|| Error::new("expected non-empty text"))
}
fn one(v: &Value, values: &[&str]) -> Result<()> {
    require(
        v.as_str().is_some_and(|s| values.contains(&s)),
        &format!("expected one of {}", values.join(", ")),
    )
}
fn wire_date(v: &Value) -> Result<()> {
    actuation_stream::Timestamp::new(
        v.as_str()
            .ok_or_else(|| Error::new("expected a timestamp"))?,
    )?;
    Ok(())
}
/// Secret-shaped keys are refused exactly as the public wire admission
/// refuses them; a body receipt carries refs and phases, never material.
fn no_secret_keys(v: &Value) -> Result<()> {
    const FORBIDDEN: &[&str] = &[
        "value",
        "material",
        "secret_value",
        "secretvalue",
        "plaintext",
        "secret",
        "password",
        "token_value",
        "api_key",
        "apikey",
        "credential",
    ];
    match v {
        Value::Object(m) => {
            for (k, v) in m {
                require(
                    !FORBIDDEN.contains(&k.to_lowercase().as_str()),
                    "receipt carries a forbidden value-shaped key",
                )?;
                no_secret_keys(v)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                no_secret_keys(v)?;
            }
        }
        _ => (),
    }
    Ok(())
}

/// The wire record behind [`SpeechSession::interrupt`]. Admitted through the
/// same lossless public-wire admission as every other receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechInterruptionReceipt(Value);

impl SpeechInterruptionReceipt {
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    pub fn into_value(self) -> Value {
        self.0
    }
    pub fn outcome(&self) -> &str {
        self.0["outcome"].as_str().unwrap_or_default()
    }
    pub fn cancelled_response(&self) -> Option<&str> {
        self.0["response_ref"].as_str()
    }
    pub fn executed_refs(&self) -> Vec<&str> {
        self.0["executed_refs"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }
    pub fn was_cancelled(&self) -> bool {
        self.outcome() == "cancelled"
    }
}

impl TryFrom<Value> for SpeechInterruptionReceipt {
    type Error = Error;
    fn try_from(v: Value) -> Result<Self> {
        validate_speech_interruption(&v)?;
        Ok(Self(v))
    }
}
impl Serialize for SpeechInterruptionReceipt {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
impl<'de> Deserialize<'de> for SpeechInterruptionReceipt {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        Self::try_from(Value::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

pub use actuation_adapters::SPEECH_CONSTITUTION_VERSION;

#[cfg(test)]
mod tests {
    use super::*;
    use actuation_adapters::SPEECH_CONSTITUTION_CHANGE_VERSION;
    use serde_json::json;

    fn constitution(body: &str, interruption: Value) -> SpeechConstitution {
        SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": format!("constitution:{body}"),
            "agent_ref": "agent:nara",
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
            "transforms": {"speech-to-speech": {"state": "supported"}},
            "transform_role": "speech-to-speech",
            "interaction": {
                "request-response": {"state": "supported"},
                "full-duplex-realtime": {"state": "supported"},
                "barge-in": {"state": "supported"},
                "vad-turn-detection": {"state": "supported"},
                "final-transcripts": {"state": "supported"},
                "tool-requests": {"state": "supported"}
            },
            "transport": "websocket",
            "connection": {"kind": "connected", "reconnect": "resumable"},
            "interruption": interruption,
            "provider_binding": {"provider_ref": "provider:realtime", "provider_session_ref": "provider-session:abc"},
            "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
            "resolved_at": "2026-09-17T09:00:00Z"
        }))
        .expect("fixture constitution must be valid")
    }

    #[test]
    fn a_supported_interruption_cancels_the_response_and_names_what_stands() {
        let mut session =
            SpeechSession::constitute(constitution("realtime", json!({"state": "supported"})))
                .unwrap();
        session.begin_listening().unwrap();
        session
            .begin_response(ExternalRef::new("response:1").unwrap())
            .unwrap();
        session
            .commit_result(ExternalRef::new("result:executed-1").unwrap())
            .unwrap();
        let receipt = session.interrupt("interruption:1", "user barge-in", "2026-09-17T09:01:00Z");
        assert_eq!(receipt.outcome(), "cancelled");
        assert_eq!(receipt.cancelled_response(), Some("response:1"));
        assert_eq!(receipt.executed_refs(), vec!["result:executed-1"]);
        assert_eq!(session.phase(), SpeechTurnPhase::Interrupted);
        // Interruption is not destruction: identity and body survive.
        assert_eq!(session.agent_ref().as_str(), "agent:nara");
        assert_eq!(
            session.constitution().constitution_ref(),
            "constitution:realtime"
        );
    }

    #[test]
    fn an_unsupported_interruption_is_recorded_and_the_response_continues() {
        let mut session = SpeechSession::constitute(constitution(
            "batch",
            json!({"state": "unsupported", "reason": "the provider cannot cancel mid-utterance"}),
        ))
        .unwrap();
        session.begin_listening().unwrap();
        session
            .begin_response(ExternalRef::new("response:2").unwrap())
            .unwrap();
        let receipt = session.interrupt("interruption:2", "user barge-in", "2026-09-17T09:02:00Z");
        assert_eq!(receipt.outcome(), "refused");
        assert!(receipt.as_value()["refusal_reason"]
            .as_str()
            .unwrap()
            .contains("does not support interruption"));
        // The response continues; nothing was faked.
        assert_eq!(session.phase(), SpeechTurnPhase::Speaking);
        assert_eq!(session.read()["in_flight_response_ref"], "response:2");
    }

    #[test]
    fn an_unknown_interruption_capability_never_behaves_as_a_yes() {
        let mut session = SpeechSession::constitute(constitution(
            "mystery",
            json!({"state": "unknown", "reason": "no modality contract declared interruption"}),
        ))
        .unwrap();
        session
            .begin_response(ExternalRef::new("response:3").unwrap())
            .unwrap();
        let receipt = session.interrupt("interruption:3", "stop", "2026-09-17T09:03:00Z");
        assert_eq!(receipt.outcome(), "refused");
        assert_eq!(session.phase(), SpeechTurnPhase::Speaking);
    }

    #[test]
    fn a_text_only_body_refuses_speech_and_stays_usable_for_text() {
        let text_only = SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": "constitution:text",
            "agent_ref": "agent:nara",
            "agency_ref": "agency:nara",
            "world_binding_ref": "binding:nara",
            "agent_session_ref": "session:nara-1",
            "body_ref": "model-surface:text",
            "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
                "inference_surface": {"contract_ref": "contract:opaque"}},
            "access_profile": {"schema": "actuation.instantiation/v1",
                "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
            "input_modalities": ["text"],
            "output_modalities": ["text"],
            "interaction": {"request-response": {"state": "supported"}, "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}},
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "unsupported", "reason": "no speech output exists to interrupt"},
            "provider_binding": {"provider_ref": "provider:text"},
            "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
            "resolved_at": "2026-09-17T09:00:00Z"
        }))
        .unwrap();
        assert!(!text_only.speech_capable());
        let mut session = SpeechSession::constitute(text_only).unwrap();
        assert!(session.begin_listening().is_err());
        assert_eq!(session.phase(), SpeechTurnPhase::Idle);
        // Text turns still work on the same session.
        session
            .begin_response(ExternalRef::new("response:text-1").unwrap())
            .unwrap();
        session.complete_response().unwrap();
        assert_eq!(session.phase(), SpeechTurnPhase::Completed);
    }

    #[test]
    fn a_body_swap_records_the_change_and_keeps_identity() {
        let mut session =
            SpeechSession::constitute(constitution("realtime", json!({"state": "supported"})))
                .unwrap();
        let cascade = SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": "constitution:cascade",
            "agent_ref": "agent:nara",
            "agency_ref": "agency:nara",
            "world_binding_ref": "binding:nara",
            "agent_session_ref": "session:nara-1",
            "body_ref": "model-surface:cascade",
            "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:stt",
                "inference_surface": {"contract_ref": "contract:stt"}},
            "access_profile": {"schema": "actuation.instantiation/v1",
                "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
            "input_modalities": ["speech"],
            "output_modalities": ["speech"],
            "transforms": {"speech-to-text": {"state": "supported"}, "text-to-speech": {"state": "supported"}},
            "interaction": {"request-response": {"state": "supported"}, "barge-in": {"state": "degraded", "reason": "barge-in only between cascade stages"}},
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "degraded", "reason": "barge-in only between cascade stages"},
            "provider_binding": {"provider_ref": "provider:cascade", "provider_session_ref": "provider-session:def"},
            "provenance": {"source_refs": ["aikit:model-stage-runtime:fixture@1"]},
            "resolved_at": "2026-09-17T09:05:00Z"
        }))
        .unwrap();
        let change = session
            .replace_body(
                "change:1",
                cascade,
                "realtime provider reconnect exhausted; cascade fallback",
                vec![ExternalRef::new("evidence:provider-outage").unwrap()],
                "2026-09-17T09:05:00Z",
            )
            .unwrap();
        assert_eq!(change.identity().0.as_str(), "agent:nara");
        let delta = change.delta();
        assert!(delta.body_changed);
        assert!(delta.provider_session_changed);
        assert!(delta
            .lost_interaction
            .contains(&"full-duplex-realtime".to_string()));
        assert_eq!(
            session.constitution().constitution_ref(),
            "constitution:cascade"
        );
        assert!(!session.constitution().realtime_capable());
        assert_eq!(
            change.as_value()["schema"],
            SPEECH_CONSTITUTION_CHANGE_VERSION
        );
    }
}
