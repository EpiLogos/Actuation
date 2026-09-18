//! Canonical Nara bound as the first dialogical consumer of generic
//! speech-capable Agency (Actuation #91, on the #94 constitution).
//!
//! Nara is a consumer here, not an ontology. Nothing in this module defines
//! what speech or a modality is — that is `actuation_adapters::speech` — and
//! nothing manufactures QL meaning: the bounded dialogue context arrives
//! exactly as the caller supplies it (`ql.nara-dialogue-context/v1`), and
//! this module only checks the identity agreements Actuation owns and reads
//! the few fields interruption correlation needs. Deixis and delegation ride
//! the QL contracts (`ql.nara-deixis/v1`, `ql.nara-epii-delegation/v1`,
//! `ql.epii-enrichment/v1`); Actuation records, gates and attributes, and
//! never interprets or auto-applies.
//!
//! The laws this module enforces:
//!
//! - **One Nara across bodies.** Realtime, cascade, reconnect or text-only —
//!   the AgentRef/AgencyRef stay the same; the body change is a #94
//!   constitution change, visible in receipts.
//! - **Absence of a speech body is a named gap, not a complete text agent.**
//!   The binding's read model carries the session's `speech_body`
//!   disclosure: a Nara without a usable speech body reads `absent` with
//!   the gap named (`none-supplied`, `credential-gated`, `degraded`,
//!   `unavailable`), and `last_change` makes a body swap — "a body may be
//!   constituted or changed later, Nara stays Nara" — legible from the wire
//!   document alone.
//! - **Interruption is not destruction.** A spoken response can be
//!   cancelled where the body supports it; the receipt attributes what was
//!   cancelled and what executed; unsupported cancellation degrades
//!   honestly; the Nara, its session and its context survive.
//! - **Nara != Epii.** Delegation is structured and basis-bound; a late
//!   result is retained but never applied — `applied: false` is structural
//!   in the receipt shape, not a promise.
//! - **Co-presence is not shared Agency.** Two Naras on one projected world
//!   keep separate Agent/Agency/session/context; a shared-world guard
//!   refuses identity collisions.

use crate::speech_session::SpeechSession;
use actuation_adapters::{
    adjudicate_speech_tool_request, SpeechConstitution, SpeechConstitutionChange,
    SpeechToolDecision, SpeechToolRequest,
};
use actuation_core::{AgentRef, Error, ExternalRef, Result};
use serde_json::{json, Value};

pub const NARA_BINDING_VERSION: &str = "actuation.nara-binding/v1";
pub const NARA_INTERRUPTION_VERSION: &str = "actuation.nara-interruption/v1";
pub const NARA_DELEGATION_VERSION: &str = "actuation.nara-delegation/v1";
pub const NARA_ENRICHMENT_VERSION: &str = "actuation.nara-enrichment/v1";
pub const NARA_CONTEXT_READ_VERSION: &str = "actuation.nara-context-read/v1";

/// QL contract names. Actuation carries these documents as data; their
/// semantics stay in QL-MEF.
pub const QL_NARA_DIALOGUE_CONTEXT: &str = "ql.nara-dialogue-context/v1";
pub const QL_NARA_EPII_DELEGATION: &str = "ql.nara-epii-delegation/v1";
pub const QL_EPII_ENRICHMENT: &str = "ql.epii-enrichment/v1";

fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::new(message))
    }
}
fn text(v: &Value) -> Result<&str> {
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::new("expected non-empty text"))
}

/// The few fields Actuation reads out of the caller-supplied QL context:
/// identity agreements and the ExpressiveAct correlation handle. This is not
/// interpretation — Actuation neither scrapes UI state nor derives QL
/// meaning from these.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextReading {
    pub context_ref: String,
    pub nara_ref: String,
    pub agent_session_ref: String,
    pub expression_ref: String,
    pub expression_revision: String,
    pub expressive_act_ref: Option<String>,
    pub expressive_act_live: bool,
    pub expressive_act_speech_turn_ref: Option<String>,
}

/// Read and identity-check one caller-supplied `ql.nara-dialogue-context/v1`
/// document against the constitution it is bound to.
pub fn read_dialogue_context(
    context: &Value,
    constitution: &SpeechConstitution,
) -> Result<ContextReading> {
    require(
        context.is_object(),
        "the dialogue context must be an object",
    )?;
    require(
        context["schema"] == QL_NARA_DIALOGUE_CONTEXT,
        "expected ql.nara-dialogue-context/v1; Actuation consumes QL documents as supplied",
    )?;
    let nara_ref = text(&context["nara_ref"])?;
    let agent_session_ref = text(&context["agent_session_ref"])?;
    require(
        nara_ref == constitution.agent_ref().as_str(),
        "the dialogue context belongs to another Nara",
    )?;
    require(
        agent_session_ref == constitution.agent_session_ref().as_str(),
        "the dialogue context belongs to another AgentSession",
    )?;
    require(
        nara_ref != agent_session_ref,
        "NaraRef and AgentSessionRef must remain distinct identities",
    )?;
    let act = &context["expressive_act"];
    Ok(ContextReading {
        context_ref: text(&context["context_ref"])?.to_owned(),
        nara_ref: nara_ref.to_owned(),
        agent_session_ref: agent_session_ref.to_owned(),
        expression_ref: text(&context["expression_ref"])?.to_owned(),
        expression_revision: text(&context["expression_revision"])?.to_owned(),
        expressive_act_ref: act["expressive_act_ref"].as_str().map(str::to_owned),
        expressive_act_live: act["phase"]
            .as_str()
            .is_some_and(|p| p == "composing" || p == "active"),
        expressive_act_speech_turn_ref: act["speech_turn_ref"].as_str().map(str::to_owned),
    })
}

impl ContextReading {
    /// The wire read model a desktop client consumes after a context
    /// admission: the same facts this module checked, nothing more.
    pub fn read_model(&self) -> Value {
        json!({
            "schema": NARA_CONTEXT_READ_VERSION,
            "context_ref": self.context_ref,
            "nara_ref": self.nara_ref,
            "agent_session_ref": self.agent_session_ref,
            "expression_ref": self.expression_ref,
            "expression_revision": self.expression_revision,
            "expressive_act": {
                "expressive_act_ref": self.expressive_act_ref,
                "live": self.expressive_act_live,
                "speech_turn_ref": self.expressive_act_speech_turn_ref,
            },
        })
    }
}

/// Canonical Nara: one Agent, one Agency, whichever speech-capable body the
/// AIKit resolution currently supplies, and the bounded QL context the
/// caller admits.
#[derive(Clone, Debug)]
pub struct NaraBinding {
    session: SpeechSession,
    context: Value,
    context_reading: ContextReading,
    /// Explicit governance lists for tool requests. Absence is not
    /// permission: an action absent from `allowed` is refused.
    allowed_action_refs: Vec<ExternalRef>,
    denied_action_refs: Vec<ExternalRef>,
    /// Decisions recorded on this binding, in order. Authorisation and
    /// refusal both stay readable.
    decisions: Vec<SpeechToolDecision>,
    /// Delegation receipts recorded on this binding, in order.
    delegations: Vec<Value>,
}

impl NaraBinding {
    /// Constitute canonical Nara on a resolved body with the caller's bounded
    /// dialogue context. A text-only body constitutes exactly like a realtime
    /// body; Nara is not defined by any of them.
    pub fn constitute(
        constitution: SpeechConstitution,
        dialogue_context: Value,
        allowed_action_refs: Vec<ExternalRef>,
        denied_action_refs: Vec<ExternalRef>,
    ) -> Result<Self> {
        let session = SpeechSession::constitute(constitution)?;
        let context_reading = read_dialogue_context(&dialogue_context, session.constitution())?;
        Ok(Self {
            session,
            context: dialogue_context,
            context_reading,
            allowed_action_refs,
            denied_action_refs,
            decisions: Vec::new(),
            delegations: Vec::new(),
        })
    }
    pub fn agent_ref(&self) -> AgentRef {
        self.session.agent_ref()
    }
    pub fn nara_ref(&self) -> &str {
        &self.context_reading.nara_ref
    }
    pub fn session(&self) -> &SpeechSession {
        &self.session
    }
    pub fn session_mut(&mut self) -> &mut SpeechSession {
        &mut self.session
    }
    pub fn context(&self) -> &Value {
        &self.context
    }
    pub fn decisions(&self) -> &[SpeechToolDecision] {
        &self.decisions
    }
    pub fn delegations(&self) -> &[Value] {
        &self.delegations
    }

    /// Reconnect or body replacement. The #94 change receipt enforces the
    /// enduring identity; the caller's refreshed QL context must still be
    /// this Nara's. A new AgentSession ref on the same Agent is a reconnect,
    /// not a new Nara.
    pub fn reconnect(
        &mut self,
        change_ref: impl AsRef<str>,
        next_constitution: SpeechConstitution,
        next_context: Value,
        reason: impl AsRef<str>,
        evidence_refs: Vec<ExternalRef>,
        at: &str,
    ) -> Result<SpeechConstitutionChange> {
        let change = self.session.replace_body(
            change_ref,
            next_constitution.clone(),
            reason,
            evidence_refs,
            at,
        )?;
        self.context_reading = read_dialogue_context(&next_context, &next_constitution)?;
        self.context = next_context;
        Ok(change)
    }

    /// A deictic/expression context update on the same encounter. The
    /// session is never reminted: same Nara, same session, updated bounded
    /// context.
    pub fn update_context(&mut self, next_context: Value) -> Result<ContextReading> {
        let next = read_dialogue_context(&next_context, self.session.constitution())?;
        require(
            next.nara_ref == self.context_reading.nara_ref,
            "a context update cannot change the Nara identity",
        )?;
        require(
            next.agent_session_ref == self.context_reading.agent_session_ref,
            "a context update cannot change the AgentSession",
        )?;
        self.context_reading = next.clone();
        self.context = next_context;
        Ok(next)
    }

    /// Interrupt the in-flight Nara response. The generic session does the
    /// capability-honest cancellation; this adds the Nara attribution and,
    /// where the caller's context carries a live ExpressiveAct correlated to
    /// the interrupted speech turn, the O:I hold/cancel notice.
    pub fn interrupt(
        &mut self,
        interruption_ref: impl AsRef<str>,
        reason: impl AsRef<str>,
        at: &str,
    ) -> Result<NaraInterruptionReceipt> {
        let generic = self
            .session
            .interrupt(interruption_ref.as_ref(), reason.as_ref(), at);
        let correlated_act = self.context_reading.expressive_act_ref.clone().filter(|_| {
            self.context_reading.expressive_act_live
                && self
                    .context_reading
                    .expressive_act_speech_turn_ref
                    .is_some()
        });
        let receipt = NaraInterruptionReceipt::try_from(json!({
            "schema": NARA_INTERRUPTION_VERSION,
            "interruption_receipt": generic.as_value(),
            "nara_ref": self.context_reading.nara_ref,
            "context_ref": self.context_reading.context_ref,
            "session_destroyed": false,
            "expressive_act": correlated_act.map(|act| json!({
                "expressive_act_ref": act,
                // What O:I does with its reversible choreography: hold the
                // act and cancel the stale pending remainder. Actuation
                // names the disposition; O:I executes its own choreography.
                "disposition": "hold-and-cancel-pending-choreography",
            })),
        }))?;
        Ok(receipt)
    }

    /// Adjudicate a speech-model tool request against this Nara's explicit
    /// governance lists. The decision is recorded on the binding either way;
    /// authorisation never executes by itself.
    pub fn adjudicate_tool_request(
        &mut self,
        decision_ref: impl AsRef<str>,
        request: SpeechToolRequest,
        decided_by: impl AsRef<str>,
        at: &str,
    ) -> Result<SpeechToolDecision> {
        let decision = adjudicate_speech_tool_request(
            decision_ref,
            self.session.constitution(),
            request,
            &self.allowed_action_refs,
            &self.denied_action_refs,
            decided_by,
            at,
        )?;
        self.decisions.push(decision.clone());
        Ok(decision)
    }

    /// Record a structured Nara→Epii delegation. The QL document is the
    /// source of the delegation's own law (basis, admitted scope, apply
    /// gate); Actuation records the attributable fact that the delegation
    /// happened, keeps Nara foreground, and checks only what Actuation owns:
    /// identity and currentness of the basis against this binding.
    pub fn delegate_to_epii(
        &mut self,
        delegation: Value,
        delegation_receipt_ref: impl AsRef<str>,
        at: &str,
    ) -> Result<Value> {
        require(delegation.is_object(), "delegation must be an object")?;
        require(
            delegation["schema"] == QL_NARA_EPII_DELEGATION,
            "expected ql.nara-epii-delegation/v1",
        )?;
        require(
            delegation["nara_ref"] == self.context_reading.nara_ref,
            "the delegation belongs to another Nara",
        )?;
        let epii = text(&delegation["epii_session_ref"])?;
        require(
            epii != self.context_reading.nara_ref,
            "Epii must remain distinct from the delegating Nara",
        )?;
        let basis_current = delegation["basis"]["context_ref"]
            .as_str()
            .is_some_and(|c| c == self.context_reading.context_ref);
        let receipt = json!({
            "schema": NARA_DELEGATION_VERSION,
            "delegation_receipt_ref": delegation_receipt_ref.as_ref(),
            "delegation_ref": delegation["delegation_ref"],
            "nara_ref": self.context_reading.nara_ref,
            "nara_agent_ref": self.agent_ref().as_str(),
            "nara_agent_session_ref": self.context_reading.agent_session_ref,
            "epii_session_ref": epii,
            "basis_context_ref": delegation["basis"]["context_ref"],
            "basis_expression_revision": delegation["basis"]["expression_revision"],
            "scope_ref_count": delegation["scope_refs"].as_array().map(|a| a.len()).unwrap_or(0),
            "foreground_agent": self.context_reading.nara_ref,
            "basis_current_at_receipt": basis_current,
            "recorded_at": at,
        });
        self.delegations.push(receipt.clone());
        Ok(receipt)
    }

    /// Receive an Epii result. A late result — one whose basis revision no
    /// longer matches the live encounter — is retained as returned material
    /// and never applied; even a current one is recorded as *proposed only*,
    /// because application belongs to the owning human/Nara, never to this
    /// receipt path. `applied: false` is structural in the receipt.
    pub fn receive_enrichment(
        &self,
        enrichment: Value,
        enrichment_receipt_ref: impl AsRef<str>,
        at: &str,
    ) -> Result<Value> {
        require(enrichment.is_object(), "enrichment must be an object")?;
        require(
            enrichment["schema"] == QL_EPII_ENRICHMENT,
            "expected ql.epii-enrichment/v1",
        )?;
        let delegation_ref = text(&enrichment["delegation_ref"])?;
        require(
            self.delegations
                .iter()
                .any(|d| d["delegation_ref"] == enrichment["delegation_ref"]),
            &format!("enrichment answers unknown delegation {delegation_ref}"),
        )?;
        let basis_revision = text(&enrichment["basis_expression_revision"])?;
        let live_revision = self.context_reading.expression_revision.as_str();
        let current = basis_revision == live_revision;
        Ok(json!({
            "schema": NARA_ENRICHMENT_VERSION,
            "enrichment_receipt_ref": enrichment_receipt_ref.as_ref(),
            "enrichment_ref": enrichment["enrichment_ref"],
            "delegation_ref": delegation_ref,
            "nara_ref": self.context_reading.nara_ref,
            "standing": if current { "proposed-only" } else { "retained-not-applied" },
            "currentness": if current {
                json!({"current": true})
            } else {
                json!({"current": false,
                       "reason": format!("produced against Expression revision {basis_revision} while the live encounter is at {live_revision}; the QL apply gate refuses stale application")})
            },
            // The never-auto-apply law, in the shape itself.
            "applied": false,
            "recorded_at": at,
        }))
    }

    /// The wire read a desktop client consumes: one Nara, its current body,
    /// turn phase, expressive-act correlation, governance decisions and
    /// delegation receipts. The `speech` block is the session read model:
    /// it carries `speech_body` — the named availability of the speech body
    /// (present, or absent with the gap named) — and `last_change` when a
    /// body was swapped, so "text-capable now, speech body absent: <reason>,
    /// a body may be constituted or swapped later, Nara stayed Nara" reads
    /// from this document alone.
    pub fn read(&self) -> Value {
        json!({
            "schema": NARA_BINDING_VERSION,
            "nara_ref": self.context_reading.nara_ref,
            "agent_ref": self.agent_ref().as_str(),
            "agency_ref": self.session.agency_ref().as_str(),
            "agent_session_ref": self.context_reading.agent_session_ref,
            "context_ref": self.context_reading.context_ref,
            "expression_revision": self.context_reading.expression_revision,
            "speech": self.session.read(),
            "expressive_act": {
                "expressive_act_ref": self.context_reading.expressive_act_ref,
                "live": self.context_reading.expressive_act_live,
                "speech_turn_ref": self.context_reading.expressive_act_speech_turn_ref,
            },
            "decision_refs": self.decisions.iter().map(|d| d.decision_ref()).collect::<Vec<_>>(),
            "delegation_receipts": self.delegations,
        })
    }
}

/// Two Naras that share a projected world still keep everything that makes
/// them separate: Agent, Agency, session and bounded context. This guard
/// refuses the collision; it never merges or discloses on their behalf.
pub fn shared_world_guard(a: &NaraBinding, b: &NaraBinding) -> Result<()> {
    let same_world = a.session.constitution().world_binding_ref()
        == b.session.constitution().world_binding_ref();
    if !same_world {
        return Ok(());
    }
    let distinct = a.agent_ref() != b.agent_ref()
        && a.session.agency_ref() != b.session.agency_ref()
        && a.session.agent_session_ref() != b.session.agent_session_ref()
        && a.context_reading.context_ref != b.context_reading.context_ref;
    require(
        distinct,
        "two Naras sharing one world must not share Agency, session or bounded context",
    )
}

/// The Nara-level interruption receipt: the generic cancellation plus who it
/// happened to, which bounded context was live, the ExpressiveAct
/// hold/cancel notice for O:I, and the structural fact that this was not
/// session destruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaraInterruptionReceipt(Value);

impl NaraInterruptionReceipt {
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    pub fn cancelled(&self) -> bool {
        self.0["interruption_receipt"]["outcome"] == "cancelled"
    }
    pub fn expressive_act_ref(&self) -> Option<&str> {
        self.0["expressive_act"]["expressive_act_ref"].as_str()
    }
}

impl TryFrom<Value> for NaraInterruptionReceipt {
    type Error = Error;
    fn try_from(v: Value) -> Result<Self> {
        validate_nara_interruption(&v)?;
        Ok(Self(v))
    }
}

pub fn validate_nara_interruption(v: &Value) -> Result<()> {
    require(v.is_object(), "Nara interruption receipt must be an object")?;
    require(
        v["schema"] == NARA_INTERRUPTION_VERSION,
        "wrong schema; expected actuation.nara-interruption/v1",
    )?;
    let generic = &v["interruption_receipt"];
    require(
        generic.is_object(),
        "the generic interruption receipt is required",
    )?;
    crate::speech_session::validate_speech_interruption(generic)?;
    require(
        generic["schema"] != NARA_INTERRUPTION_VERSION,
        "the generic receipt must remain the generic schema",
    )?;
    text(&v["nara_ref"])?;
    text(&v["context_ref"])?;
    require(
        v["session_destroyed"] == false,
        "interruption is not session destruction",
    )?;
    require(
        v["nara_ref"] == generic["agent_ref"],
        "the Nara attribution must agree with the session's Agent identity",
    )?;
    if !v["expressive_act"].is_null() {
        text(&v["expressive_act"]["expressive_act_ref"])?;
        require(
            v["expressive_act"]["disposition"] == "hold-and-cancel-pending-choreography",
            "unknown ExpressiveAct disposition",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speech_session::SpeechTurnPhase;
    use actuation_adapters::{
        SPEECH_CONSTITUTION_CHANGE_VERSION, SPEECH_CONSTITUTION_VERSION,
        SPEECH_TOOL_DECISION_VERSION,
    };
    use serde_json::json;

    fn constitution(
        body: &str,
        agent_session: &str,
        speech: bool,
        interruption: Value,
    ) -> SpeechConstitution {
        let (input, output, interaction) = if speech {
            (
                json!(["text", "speech"]),
                json!(["text", "speech"]),
                json!({
                    "request-response": {"state": "supported"},
                    "full-duplex-realtime": {"state": "supported"},
                    "barge-in": {"state": "supported"},
                    "vad-turn-detection": {"state": "supported"},
                    "final-transcripts": {"state": "supported"},
                    "tool-requests": {"state": "supported"}
                }),
            )
        } else {
            (
                json!(["text"]),
                json!(["text"]),
                json!({
                    "request-response": {"state": "supported"},
                    "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}
                }),
            )
        };
        SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": format!("constitution:{body}"),
            "agent_ref": "nara:canonical",
            "agency_ref": "agency:nara",
            "world_binding_ref": "binding:shared-world",
            "agent_session_ref": agent_session,
            "body_ref": format!("model-surface:{body}"),
            "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
                "inference_surface": {"contract_ref": "contract:opaque"}},
            "access_profile": {"schema": "actuation.instantiation/v1",
                "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
            "input_modalities": input,
            "output_modalities": output,
            "interaction": interaction,
            "transport": if speech { "websocket" } else { "http" },
            "connection": if speech {
                json!({"kind": "connected", "reconnect": "resumable"})
            } else {
                json!({"kind": "stateless", "reconnect": null})
            },
            "interruption": interruption,
            "provider_binding": {"provider_ref": format!("provider:{body}"),
                "provider_session_ref": format!("provider-session:{body}")},
            "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
            "resolved_at": "2026-09-17T09:00:00Z"
        }))
        .expect("fixture constitution must be valid")
    }

    fn context(nara: &str, session: &str) -> Value {
        json!({
            "schema": QL_NARA_DIALOGUE_CONTEXT,
            "context_ref": "context:encounter-1",
            "nara_ref": nara,
            "subject_ref": "subject:frank",
            "agent_session_ref": session,
            "coordinate_ref": "M2-5-9:C0",
            "expression_ref": "expression:1",
            "expression_revision": "rev-7",
            "profile_ref": "profile:1",
            "profile_revision": "rev-2",
            "active_m_focus": "m4",
            "disclosed": [],
            "available_action_refs": [],
            "expressive_act": {
                "expressive_act_ref": "expressive-act:9",
                "phase": "active",
                "basis_expression_revision": "rev-7",
                "speech_turn_ref": "response:1"
            }
        })
    }

    fn realtime_binding() -> NaraBinding {
        NaraBinding::constitute(
            constitution(
                "realtime",
                "session:nara-1",
                true,
                json!({"state": "supported"}),
            ),
            context("nara:canonical", "session:nara-1"),
            vec![ExternalRef::new("action:read-coordinate").unwrap()],
            vec![ExternalRef::new("action:delete-world").unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn the_same_nara_inhabits_realtime_cascade_and_text_bodies() {
        let realtime = realtime_binding();
        assert_eq!(realtime.nara_ref(), "nara:canonical");
        assert_eq!(realtime.agent_ref().as_str(), "nara:canonical");
        assert!(realtime.session().constitution().realtime_capable());

        // Body swap: realtime -> cascade, same Agent/Agency/session. The
        // cascade carries speech but not the full-duplex interaction.
        let mut cascade = realtime.clone();
        let cascade_constitution = SpeechConstitution::try_from(json!({
            "schema": SPEECH_CONSTITUTION_VERSION,
            "constitution_ref": "constitution:cascade",
            "agent_ref": "nara:canonical",
            "agency_ref": "agency:nara",
            "world_binding_ref": "binding:shared-world",
            "agent_session_ref": "session:nara-1",
            "body_ref": "model-surface:cascade",
            "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:stt",
                "inference_surface": {"contract_ref": "contract:stt"}},
            "access_profile": {"schema": "actuation.instantiation/v1",
                "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
            "input_modalities": ["speech"],
            "output_modalities": ["speech"],
            "transforms": {"speech-to-text": {"state": "supported"}, "text-to-speech": {"state": "supported"}},
            "interaction": {
                "request-response": {"state": "supported"},
                "barge-in": {"state": "degraded", "reason": "barge-in only between cascade stages"},
                "tool-requests": {"state": "supported"}
            },
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "degraded", "reason": "barge-in only between cascade stages"},
            "provider_binding": {"provider_ref": "provider:cascade", "provider_session_ref": "provider-session:cascade"},
            "provenance": {"source_refs": ["aikit:model-stage-runtime:fixture@1"]},
            "resolved_at": "2026-09-17T09:06:00Z"
        }))
        .unwrap();
        let change = cascade
            .reconnect(
                "change:rt-to-cascade",
                cascade_constitution.clone(),
                context("nara:canonical", "session:nara-1"),
                "realtime provider lost; cascade fallback",
                vec![ExternalRef::new("evidence:outage").unwrap()],
                "2026-09-17T09:06:00Z",
            )
            .unwrap();
        assert_eq!(change.identity().0.as_str(), "nara:canonical");
        assert_eq!(
            change.as_value()["schema"],
            SPEECH_CONSTITUTION_CHANGE_VERSION
        );
        assert_eq!(cascade.nara_ref(), "nara:canonical");
        assert!(!cascade.session().constitution().realtime_capable());
        assert_eq!(cascade.context()["context_ref"], "context:encounter-1");

        // Speech failure -> text-only body: Nara stays usable.
        let mut text_only = cascade.clone();
        text_only
            .reconnect(
                "change:cascade-to-text",
                constitution(
                    "text",
                    "session:nara-1",
                    false,
                    json!({"state": "unsupported", "reason": "no speech output"}),
                ),
                context("nara:canonical", "session:nara-1"),
                "speech body failed; text fallback",
                vec![ExternalRef::new("evidence:speech-failure").unwrap()],
                "2026-09-17T09:07:00Z",
            )
            .unwrap();
        assert!(!text_only.session().constitution().speech_capable());
        assert!(text_only.session_mut().begin_listening().is_err());
        text_only
            .session_mut()
            .begin_response(ExternalRef::new("response:text").unwrap())
            .unwrap();
        assert_eq!(text_only.session().phase(), SpeechTurnPhase::Speaking);
    }

    #[test]
    fn a_reconnect_to_a_new_session_still_keeps_one_nara() {
        let mut nara = realtime_binding();
        // A reconnect may open a new AgentSession on the same Agent.
        let change = nara
            .reconnect(
                "change:reconnect",
                constitution(
                    "realtime-2",
                    "session:nara-2",
                    true,
                    json!({"state": "supported"}),
                ),
                context("nara:canonical", "session:nara-2"),
                "transport connection rebuilt",
                vec![ExternalRef::new("evidence:reconnect").unwrap()],
                "2026-09-17T09:08:00Z",
            )
            .unwrap();
        assert_eq!(
            change.identity(),
            (
                AgentRef::new("nara:canonical").unwrap(),
                actuation_core::AgencyRef::new("agency:nara").unwrap()
            )
        );
        assert_eq!(
            nara.session().agent_session_ref().as_str(),
            "session:nara-2"
        );
        assert_eq!(nara.nara_ref(), "nara:canonical");
    }

    #[test]
    fn a_context_update_never_remints_the_session() {
        let mut nara = realtime_binding();
        let mut next = context("nara:canonical", "session:nara-1");
        next["expression_revision"] = json!("rev-8");
        next["context_ref"] = json!("context:encounter-1");
        let reading = nara.update_context(next).unwrap();
        assert_eq!(reading.expression_revision, "rev-8");
        assert_eq!(
            nara.session().agent_session_ref().as_str(),
            "session:nara-1"
        );
        assert_eq!(nara.agent_ref().as_str(), "nara:canonical");
        // A foreign context is refused, not absorbed.
        let foreign = context("nara:other", "session:nara-1");
        assert!(nara.update_context(foreign).is_err());
    }

    #[test]
    fn interruption_cancels_attributably_and_correlates_the_expressive_act() {
        let mut nara = realtime_binding();
        nara.session_mut()
            .begin_response(ExternalRef::new("response:1").unwrap())
            .unwrap();
        nara.session_mut()
            .commit_result(ExternalRef::new("result:done").unwrap())
            .unwrap();
        let receipt = nara
            .interrupt(
                "interruption:9",
                "user spoke over the response",
                "2026-09-17T09:09:00Z",
            )
            .unwrap();
        assert!(receipt.cancelled());
        assert_eq!(
            receipt.as_value()["interruption_receipt"]["response_ref"],
            "response:1"
        );
        assert_eq!(
            receipt.as_value()["interruption_receipt"]["executed_refs"],
            json!(["result:done"])
        );
        assert_eq!(receipt.expressive_act_ref(), Some("expressive-act:9"));
        assert_eq!(
            receipt.as_value()["expressive_act"]["disposition"],
            "hold-and-cancel-pending-choreography"
        );
        assert_eq!(receipt.as_value()["session_destroyed"], false);
        // The session, context and identity all survive.
        assert_eq!(nara.session().phase(), SpeechTurnPhase::Interrupted);
        assert_eq!(nara.nara_ref(), "nara:canonical");
    }

    #[test]
    fn unsupported_interruption_degrades_honestly_and_the_body_swap_matches_it() {
        let mut nara = NaraBinding::constitute(
            constitution(
                "cascade",
                "session:nara-1",
                true,
                json!({"state": "unsupported", "reason": "the cascade cannot cancel mid-stage"}),
            ),
            context("nara:canonical", "session:nara-1"),
            vec![],
            vec![],
        )
        .unwrap();
        nara.session_mut()
            .begin_response(ExternalRef::new("response:2").unwrap())
            .unwrap();
        let receipt = nara
            .interrupt("interruption:10", "stop", "2026-09-17T09:10:00Z")
            .unwrap();
        assert!(!receipt.cancelled());
        assert_eq!(
            receipt.as_value()["interruption_receipt"]["outcome"],
            "refused"
        );
        // Where the AIKit truth says unsupported, the behaviour matches it.
        assert_eq!(nara.session().phase(), SpeechTurnPhase::Speaking);
    }

    #[test]
    fn a_speech_tool_request_is_refused_before_effect_and_authorised_only_explicitly() {
        let mut nara = realtime_binding();
        let request = |action: Option<&str>| {
            SpeechToolRequest::try_from(json!({
                "schema": SPEECH_TOOL_DECISION_VERSION,
                "request_ref": "request:model-1",
                "constitution_ref": "constitution:realtime",
                "agent_session_ref": "session:nara-1",
                "proposed_action_ref": action,
                "payload_refs": ["ref:spoken-target"],
                "requested_at": "2026-09-17T09:11:00Z"
            }))
            .unwrap()
        };
        // Available channel is not authority: an unlisted action is refused.
        let refused = nara
            .adjudicate_tool_request(
                "decision:unlisted",
                request(Some("action:not-in-the-list")),
                "owner",
                "2026-09-17T09:11:01Z",
            )
            .unwrap();
        assert!(!refused.is_authorised());
        // Explicitly denied is refused even before the allowed list matters.
        let denied = nara
            .adjudicate_tool_request(
                "decision:denied",
                request(Some("action:delete-world")),
                "owner",
                "2026-09-17T09:11:02Z",
            )
            .unwrap();
        assert!(!denied.is_authorised());
        // An explicitly allowed action is authorised — and still not executed.
        let allowed = nara
            .adjudicate_tool_request(
                "decision:allowed",
                request(Some("action:read-coordinate")),
                "owner",
                "2026-09-17T09:11:03Z",
            )
            .unwrap();
        assert!(allowed.is_authorised());
        let execution = allowed
            .record_execution(
                "execution:1",
                vec![ExternalRef::new("evidence:execution").unwrap()],
                "2026-09-17T09:11:04Z",
            )
            .unwrap();
        assert_eq!(execution.as_value()["decision_ref"], "decision:allowed");
        // A refused decision cannot be executed at all.
        assert!(refused
            .record_execution(
                "execution:2",
                vec![ExternalRef::new("evidence:x").unwrap()],
                "2026-09-17T09:11:05Z"
            )
            .is_err());
        assert_eq!(nara.decisions().len(), 3);
    }

    #[test]
    fn delegation_stays_structured_and_a_late_result_is_retained_not_applied() {
        let mut nara = realtime_binding();
        let delegation = json!({
            "schema": QL_NARA_EPII_DELEGATION,
            "delegation_ref": "delegation:1",
            "nara_ref": "nara:canonical",
            "epii_session_ref": "session:epii-1",
            "basis": {
                "context_ref": "context:encounter-1",
                "expression_ref": "expression:1",
                "expression_revision": "rev-7",
                "profile_ref": "profile:1",
                "profile_revision": "rev-2",
                "coordinate_ref": "M2-5-9:C0",
                "bimba_registry_revision": "registry-3"
            },
            "brief": "resolve the relation under focus",
            "scope_refs": ["source:1"],
            "delegated_at_unix_ms": 1,
            "state": "delegated"
        });
        let receipt = nara
            .delegate_to_epii(delegation, "delegation-receipt:1", "2026-09-17T09:12:00Z")
            .unwrap();
        assert_eq!(receipt["foreground_agent"], "nara:canonical");
        assert_eq!(receipt["epii_session_ref"], "session:epii-1");
        assert_eq!(receipt["basis_current_at_receipt"], true);

        // The enrichment arrives after the encounter moved to rev-9.
        let mut nara_moved = nara.clone();
        let mut next = context("nara:canonical", "session:nara-1");
        next["expression_revision"] = json!("rev-9");
        nara_moved.update_context(next).unwrap();
        let enrichment = json!({
            "schema": QL_EPII_ENRICHMENT,
            "enrichment_ref": "enrichment:1",
            "delegation_ref": "delegation:1",
            "basis_context_ref": "context:encounter-1",
            "basis_expression_revision": "rev-7",
            "proposed_focus_refs": ["source:1"],
            "coordinate_refs": [],
            "summary": "the relation resolves as ..."
        });
        let late = nara_moved
            .receive_enrichment(
                enrichment.clone(),
                "enrichment-receipt:1",
                "2026-09-17T09:13:00Z",
            )
            .unwrap();
        assert_eq!(late["standing"], "retained-not-applied");
        assert_eq!(late["applied"], false);
        assert!(late["currentness"]["current"] == false);
        // Even a current result is proposed only; nothing auto-applies.
        let fresh = nara
            .receive_enrichment(enrichment, "enrichment-receipt:2", "2026-09-17T09:14:00Z")
            .unwrap();
        assert_eq!(fresh["standing"], "proposed-only");
        assert_eq!(fresh["applied"], false);
        // An enrichment for an unknown delegation is refused outright.
        assert!(nara
            .receive_enrichment(
                json!({
                    "schema": QL_EPII_ENRICHMENT,
                    "enrichment_ref": "enrichment:2",
                    "delegation_ref": "delegation:unknown",
                    "basis_context_ref": "context:encounter-1",
                    "basis_expression_revision": "rev-7",
                    "proposed_focus_refs": [],
                    "coordinate_refs": []
                }),
                "enrichment-receipt:3",
                "2026-09-17T09:15:00Z",
            )
            .is_err());
    }

    #[test]
    fn two_naras_on_one_world_keep_separate_identity_and_context() {
        let a = realtime_binding();
        let mut other_context = context("nara:canonical", "session:nara-1");
        other_context["context_ref"] = json!("context:encounter-2");
        // Same Agent and session as `a` would be a collision even with a
        // different context instance.
        let collision = NaraBinding::constitute(
            constitution(
                "realtime",
                "session:nara-1",
                true,
                json!({"state": "supported"}),
            ),
            other_context,
            vec![],
            vec![],
        )
        .unwrap();
        assert!(shared_world_guard(&a, &collision).is_err());

        // A genuinely separate participant on the same projected world.
        let b_constitution = constitution(
            "realtime",
            "session:nara-b",
            true,
            json!({"state": "supported"}),
        );
        let mut b_value = b_constitution.into_value();
        b_value["agent_ref"] = json!("nara:b");
        b_value["agency_ref"] = json!("agency:nara-b");
        b_value["constitution_ref"] = json!("constitution:nara-b");
        let b_constitution = SpeechConstitution::try_from(b_value).unwrap();
        let mut b_context = context("nara:b", "session:nara-b");
        b_context["context_ref"] = json!("context:b-1");
        let b = NaraBinding::constitute(b_constitution, b_context, vec![], vec![]).unwrap();
        shared_world_guard(&a, &b).unwrap();
    }
}
