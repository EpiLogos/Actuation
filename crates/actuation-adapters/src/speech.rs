//! Generic speech/audio material constitution for model-bearing Agency
//! (Actuation #94).
//!
//! A speech-capable body is an ordinary constitutive condition of an
//! AgentSession, not a consumer ontology and not a Nara-specific shape. This
//! module admits the resolved material constitution of that body — the facts
//! AIKit's `aikit.model-modality/v1` resolution and model-runtime read models
//! supply — and the receipts that record the body changing.
//!
//! Three laws are enforced here structurally, not by documentation:
//!
//! 1. **Identity.** `AgentRef/AgencyRef != AgentSessionRef != ModelSurface !=
//!    provider session != transport connection` (nominal refs from
//!    `actuation-core`). A constitution is refused unless its identity roles
//!    are distinct, and a constitution change is refused unless the enduring
//!    Agent/Agency identity is preserved — a body swap records a change, it
//!    never re-identifies.
//! 2. **Authority.** A speech model's structured tool request is a request,
//!    never a canonical Action; an available capability is not an authorised
//!    one; an authorised one is not an executed one. The tool-request gate
//!    reads authority only from the supplied governing autonomy records.
//!    Nothing in the transport, provider or modality facts can grant it, so
//!    client audio transports never gain ambient root/metagency authority
//!    through this seam.
//! 3. **Honest capability.** Supported, degraded, unsupported and unknown
//!    stay four different facts ([`SpeechSupport`]), exactly as the AIKit
//!    contract states them. Absence of a capability is never silently
//!    dropped or flattened; a session's behaviour must match the recorded
//!    truth.
//!
//! Credential facts are ref/presence only ([`CREDENTIAL_CONDITIONS`], the
//! AIKit `CredentialCondition` vocabulary); secret-shaped keys are refused at
//! admission like every other public wire record.
//!
//! **Availability is a named state, not an implication.** A session whose
//! body cannot hear and speak must read as a named gap — `none-supplied`
//! (no acoustic modality was declared), `credential-gated` (the body needs a
//! credential that is not bound), `degraded` (a recorded availability
//! condition reduces it), `unavailable` (a recorded condition forbids it) —
//! never as a silently complete text agent. [`SpeechConstitution::speech_body`]
//! derives that disclosure from the carried facts alone; capability
//! ([`SpeechConstitution::speech_capable`]) and availability stay two
//! different facts and both remain on the wire. The swap path is part of the
//! same disclosure: a body can be constituted or changed later without
//! touching the Agent/Agency identity, and the change receipts prove it.
//!
//! Modalities, transforms, interaction capabilities, transports and
//! reconnect support use the same kebab vocabulary as `aikit.model-modality/
//! v1` (AIKit #317). Actuation consumes that resolution as data; it does not
//! redefine the contract, and provider spellings travel only as provenance.
//! The mirrored vocabulary is frozen in [`MIRRORED_MODALITY_VOCABULARY`] and
//! pinned by test, so an upstream rename breaks loudly here instead of
//! drifting silently.
//!
//! Ordering for upstream consumers: AIKit's raw `ModelRuntimeRelation` is not
//! an admitted constitution shape. A consumer upstream of Actuation must
//! reduce it to the Actuation-admitted `model_relation`/`access_profile`
//! documents before building a constitution — and must leave the credential
//! key the raw relation carries behind, because Actuation's secret scan
//! refuses value-shaped material at admission.

use crate::admission::{
    date, facts, one, optional_text, optional_texts, require, text, texts,
    validate_model_access_profile, validate_model_relation, INSTANTIATION_VERSION,
};
use actuation_core::{
    AgencyRef, AgentRef, AgentSessionRef, Error, ExternalRef, ModelSurfaceRef, ProviderSessionRef,
    Result, TransportConnectionRef, WorldBindingRef,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const SPEECH_CONSTITUTION_VERSION: &str = "actuation.speech-constitution/v1";
pub const SPEECH_CONSTITUTION_CHANGE_VERSION: &str = "actuation.speech-constitution-change/v1";
pub const SPEECH_TOOL_DECISION_VERSION: &str = "actuation.speech-tool-decision/v1";

/// One input or output modality, in the AIKit `aikit.model-modality/v1`
/// vocabulary.
pub const MODALITIES: &[&str] = &["text", "audio", "speech"];
/// Named conversions a body can perform, in the same vocabulary.
pub const TRANSFORMS: &[&str] = &[
    "speech-to-text",
    "text-to-speech",
    "speech-to-speech",
    "audio-understanding",
    "multimodal-text-audio",
];
/// Interaction forms, in the same vocabulary.
pub const INTERACTIONS: &[&str] = &[
    "request-response",
    "streaming-input",
    "streaming-output",
    "full-duplex-realtime",
    "structured-events",
    "tool-requests",
    "timestamps",
    "partial-transcripts",
    "final-transcripts",
    "vad-turn-detection",
    "barge-in",
];
/// Transports a surface can be reached over.
pub const TRANSPORTS: &[&str] = &[
    "in-process",
    "cli",
    "http",
    "websocket",
    "webrtc",
    "sip",
    "provider-native",
];
/// Reconnect behaviour of a connected transport.
pub const RECONNECTS: &[&str] = &[
    "not-applicable",
    "resumable",
    "reconnect-without-session",
    "unsupported",
];
/// Connection semantics kinds, in the same vocabulary.
pub const CONNECTION_KINDS: &[&str] = &["stateless", "connected"];

/// Availability conditions a body can record, in the AIKit
/// `SurfaceAvailability` vocabulary (`aikit-core/src/model_modality.rs`).
/// `available` is the absence of a condition, so it is not admitted here:
/// only the two states that are facts get recorded.
pub const AVAILABILITY_CONDITIONS: &[&str] = &["degraded", "unavailable"];

/// Credential conditions a body can declare, in the AIKit
/// `CredentialCondition` vocabulary (`aikit-core/src/model_route.rs`).
/// `required` is the pre-key fact — the body exists and no credential is
/// bound for it — which is exactly the state that must be visible before any
/// provider key exists. Refs and hints only; never secret material.
pub const CREDENTIAL_CONDITIONS: &[&str] = &["not-required", "required", "satisfied"];

/// The frozen list of every kebab term Actuation mirrors from AIKit's
/// `aikit.model-modality/v1` contract family. Upstream sources: ai-kit
/// `crates/aikit-core/src/model_modality.rs` (`ModelModality`,
/// `TransformCapability`, `InteractionCapability`, `TransportKind`,
/// `ReconnectSupport`, `ConnectionSemantics`, `SurfaceAvailability`) and
/// `crates/aikit-core/src/model_route.rs` (`CredentialCondition`).
///
/// This is a tested fixture, not decoration: `speech_vocabulary` conformance
/// asserts that the admission grammar above accepts exactly this list — no
/// more, no less — and refuses an invented term in every category. If ai-kit
/// renames or adds a term, update that source first, then this fixture and
/// the grammar lists together; the test fails until all three agree, so the
/// mirror cannot drift silently from the contract it consumes.
pub const MIRRORED_MODALITY_VOCABULARY: &[&str] = &[
    // ConnectionSemantics kinds.
    "connected",
    "stateless",
    // CredentialCondition.
    "not-required",
    "required",
    "satisfied",
    // InteractionCapability.
    "barge-in",
    "final-transcripts",
    "full-duplex-realtime",
    "partial-transcripts",
    "request-response",
    "structured-events",
    "streaming-input",
    "streaming-output",
    "timestamps",
    "tool-requests",
    "vad-turn-detection",
    // ModelModality.
    "audio",
    "speech",
    "text",
    // ReconnectSupport.
    "not-applicable",
    "reconnect-without-session",
    "resumable",
    "unsupported",
    // SurfaceAvailability (the two recorded states; "available" is the
    // absence of a condition and is not a recorded fact).
    "degraded",
    "unavailable",
    // TransformCapability.
    "audio-understanding",
    "multimodal-text-audio",
    "speech-to-speech",
    "speech-to-text",
    "text-to-speech",
    // TransportKind.
    "cli",
    "http",
    "in-process",
    "provider-native",
    "sip",
    "websocket",
    "webrtc",
];

/// The four-state answer vocabulary mirrored from the same contract:
/// proven, degraded, proven-absent and unproven stay four different facts.
pub const MIRRORED_SUPPORT_STATES: &[&str] = &["degraded", "supported", "unknown", "unsupported"];

/// The fully explicit answer to "can this body do X?". Mirrors the four-state
/// answers of the AIKit modality contract: proven, degraded, proven-absent
/// and unproven stay distinct, and `unknown` never behaves as a yes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum SpeechSupport {
    Supported,
    Degraded { reason: String },
    Unsupported { reason: String },
    Unknown { reason: String },
}

impl SpeechSupport {
    pub fn as_value(&self) -> Value {
        match self {
            Self::Supported => json!({"state": "supported"}),
            Self::Degraded { reason } => json!({"state": "degraded", "reason": reason}),
            Self::Unsupported { reason } => json!({"state": "unsupported", "reason": reason}),
            Self::Unknown { reason } => json!({"state": "unknown", "reason": reason}),
        }
    }
    pub fn from_value(v: &Value) -> Result<Self> {
        require(v.is_object(), "speech support must be an object")?;
        match v["state"].as_str() {
            Some("supported") => Ok(Self::Supported),
            Some("degraded") => Ok(Self::Degraded {
                reason: field_text(&v["reason"], "degraded reason")?,
            }),
            Some("unsupported") => Ok(Self::Unsupported {
                reason: field_text(&v["reason"], "unsupported reason")?,
            }),
            Some("unknown") => Ok(Self::Unknown {
                reason: field_text(&v["reason"], "unknown reason")?,
            }),
            _ => Err(Error::new(
                "speech support must be supported, degraded, unsupported or unknown",
            )),
        }
    }
    /// Only a proven or stated-reduced capability behaves as usable.
    /// Unsupported and unknown do not.
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Supported | Self::Degraded { .. })
    }
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// `text` with the field name carried into the error message.
fn field_text(v: &Value, label: &str) -> Result<String> {
    text(v)
        .map_err(|e| Error::new(format!("{label}: {e}")))
        .map(str::to_owned)
}

fn support_entry(v: &Value) -> Result<()> {
    SpeechSupport::from_value(v).map(|_| ())
}

fn support_map(v: &Value) -> Result<()> {
    let map = v
        .as_object()
        .ok_or_else(|| Error::new("interaction support must be an object"))?;
    for (k, entry) in map {
        require(
            INTERACTIONS.contains(&k.as_str()),
            &format!("unknown interaction capability {k}"),
        )?;
        support_entry(entry)?;
    }
    Ok(())
}

fn lookup<'a>(v: &'a Value, path: &str) -> Option<&'a str> {
    let mut current = v;
    for key in path.split('.') {
        current = &current[key];
    }
    current.as_str()
}

/// Validate one resolved speech/audio material constitution.
///
/// The constitution records the body AIKit resolved for one AgentSession:
/// effective model/surface relation and revisions, input/output modalities,
/// transform role, streaming/realtime relation, inference versus
/// material-control access, provider/material binding provenance,
/// interruption capability, and the degraded/unavailable conditions of the
/// body. A text-only body is a valid constitution — a session without
/// acoustic modalities is a constituted fact, not an absence.
pub fn validate_speech_constitution(v: &Value) -> Result<()> {
    require(v.is_object(), "speech constitution must be an object")?;
    require(
        v["schema"] == SPEECH_CONSTITUTION_VERSION,
        "wrong schema; expected actuation.speech-constitution/v1",
    )?;
    for key in [
        "constitution_ref",
        "agent_ref",
        "agency_ref",
        "world_binding_ref",
        "agent_session_ref",
        "body_ref",
    ] {
        field_text(&v[key], key)?;
    }
    // Identity law: the body is none of the identities that carry it, and the
    // session is not the Agent. Typed refs keep the roles apart at compile
    // time; this check keeps their values apart at admission.
    for pair in [
        ["agent_ref", "agency_ref"],
        ["agent_ref", "agent_session_ref"],
        ["agent_ref", "body_ref"],
        ["agency_ref", "body_ref"],
        ["agent_session_ref", "body_ref"],
        ["body_ref", "provider_binding.provider_session_ref"],
        ["body_ref", "provider_binding.transport_connection_ref"],
    ] {
        if let (Some(a), Some(b)) = (v[pair[0]].as_str(), lookup(v, pair[1])) {
            require(
                a != b,
                &format!("{} and {} must remain distinct refs", pair[0], pair[1]),
            )?;
        }
    }
    optional_text(&v["body_revision"])?;
    optional_text(&v["harness_composition_ref"])?;
    optional_text(&v["modality_contract_ref"])?;
    optional_text(&v["modality_contract_revision"])?;
    validate_model_relation(&v["model_relation"])?;
    validate_model_access_profile(&v["access_profile"])?;
    for key in ["input_modalities", "output_modalities"] {
        let modalities = texts(&v[key])?;
        require(
            !modalities.is_empty(),
            "a constitution declares its modalities; absence is recorded, not omitted",
        )?;
        for m in &modalities {
            require(
                MODALITIES.contains(&m.as_str()),
                &format!("unknown modality {m}"),
            )?;
        }
        let unique: BTreeSet<_> = modalities.iter().collect();
        require(unique.len() == modalities.len(), "duplicate modality")?;
    }
    // A transform whose prerequisites the body does not carry is a
    // contradiction, and contradictions are how read models start lying.
    if let Some(transforms) = v.get("transforms").filter(|t| !t.is_null()) {
        let map = transforms
            .as_object()
            .ok_or_else(|| Error::new("transforms must be an object"))?;
        let acoustic_in = texts(&v["input_modalities"])?
            .iter()
            .any(|m| m == "audio" || m == "speech");
        let acoustic_out = texts(&v["output_modalities"])?
            .iter()
            .any(|m| m == "audio" || m == "speech");
        for (k, entry) in map {
            require(
                TRANSFORMS.contains(&k.as_str()),
                &format!("unknown transform {k}"),
            )?;
            support_entry(entry)?;
            let needs_in = matches!(
                k.as_str(),
                "speech-to-text" | "speech-to-speech" | "audio-understanding"
            );
            let needs_out = matches!(k.as_str(), "text-to-speech" | "speech-to-speech");
            require(
                !needs_in || acoustic_in,
                &format!("transform {k} requires an acoustic input modality"),
            )?;
            require(
                !needs_out || acoustic_out,
                &format!("transform {k} requires an acoustic output modality"),
            )?;
        }
    }
    optional_text(&v["transform_role"])?;
    if !v["transform_role"].is_null() {
        require(
            TRANSFORMS.contains(&field_text(&v["transform_role"], "transform role")?.as_str()),
            "unknown transform role",
        )?;
    }
    require(
        !v["interaction"].is_null(),
        "the interaction support map is required; an unqueried capability is unknown, not absent",
    )?;
    support_map(&v["interaction"])?;
    one(&v["transport"], TRANSPORTS)?;
    require(v["connection"].is_object(), "connection semantics required")?;
    require(
        v["connection"]["kind"]
            .as_str()
            .is_some_and(|k| CONNECTION_KINDS.contains(&k)),
        "connection kind must be stateless or connected",
    )?;
    match v["connection"]["kind"].as_str() {
        Some("stateless") => require(
            v["connection"]["reconnect"].is_null(),
            "stateless connection cannot declare reconnect support",
        )?,
        Some("connected") => one(&v["connection"]["reconnect"], RECONNECTS)?,
        _ => unreachable!("connection kind guarded by CONNECTION_KINDS"),
    }
    // Interruption is a resolved capability, stated in the same four states.
    support_entry(&v["interruption"])?;
    // Provider/material binding provenance: refs only, never secret material.
    require(
        v["provider_binding"].is_object(),
        "provider_binding provenance is required",
    )?;
    field_text(&v["provider_binding"]["provider_ref"], "provider_ref")?;
    optional_text(&v["provider_binding"]["provider_session_ref"])?;
    optional_text(&v["provider_binding"]["transport_connection_ref"])?;
    optional_text(&v["provider_binding"]["material_binding_ref"])?;
    facts(&v["provider_binding"]["facts"])?;
    // Degraded/unavailable conditions are named, never absorbed.
    if !v["conditions"].is_null() {
        let conditions = v["conditions"]
            .as_array()
            .ok_or_else(|| Error::new("conditions must be an array of named facts"))?;
        for c in conditions {
            require(c.is_object(), "each condition must be an object")?;
            one(&c["condition"], AVAILABILITY_CONDITIONS)?;
            field_text(&c["reason"], "condition reason")?;
        }
    }
    validate_credential_condition(&v["credential_condition"])?;
    optional_texts(&v["usage_evidence_refs"])?;
    require(v["provenance"].is_object(), "provenance is required")?;
    optional_texts(&v["provenance"]["source_refs"])?;
    date(&v["resolved_at"])?;
    no_secret_material_keys(v)
}

/// Admit the optional `credential_condition` fact: the AIKit
/// `CredentialCondition` shape, refs and hints only. Each state carries
/// exactly its own fields — `required` names the missing credential in its
/// hint, `satisfied` names the hint and the binding ref, `not-required`
/// carries neither — so a half-stated credential fact is refused rather
/// than guessed at.
fn validate_credential_condition(v: &Value) -> Result<()> {
    if v.is_null() {
        return Ok(());
    }
    require(
        v.is_object(),
        "credential_condition must be an object when carried",
    )?;
    match v["condition"].as_str() {
        Some("not-required") => {
            require(
                v["hint"].is_null(),
                "a credential-free body cannot carry a credential hint",
            )?;
            require(
                v["binding_ref"].is_null(),
                "a credential-free body cannot carry a binding ref",
            )?;
        }
        Some("required") => {
            field_text(&v["hint"], "credential hint")?;
            require(
                v["binding_ref"].is_null(),
                "an unbound credential cannot carry a binding ref",
            )?;
        }
        Some("satisfied") => {
            field_text(&v["hint"], "credential hint")?;
            field_text(&v["binding_ref"], "credential binding_ref")?;
        }
        _ => {
            return Err(Error::new(
                "credential condition must be not-required, required or satisfied",
            ))
        }
    }
    Ok(())
}

/// A constitution carries refs, phases and scalar provider facts — never
/// secret material. The same value-shaped key names the secret scan refuses
/// are refused here, except `material`: a model relation's material binding
/// provenance is a legitimate, admitted constitution field, not a secret.
fn no_secret_material_keys(v: &Value) -> Result<()> {
    const FORBIDDEN: &[&str] = &[
        "value",
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
                    "constitution carries a forbidden value-shaped key",
                )?;
                no_secret_material_keys(v)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                no_secret_material_keys(v)?;
            }
        }
        _ => (),
    }
    Ok(())
}

/// A lossless admitted wire record: foreign fields, null and omission all
/// survive admission unchanged, and the record cannot be mutated afterwards.
macro_rules! wire_record {
    ($name:ident, $validate:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(Value);
        impl $name {
            pub fn as_value(&self) -> &Value {
                &self.0
            }
            pub fn into_value(self) -> Value {
                self.0
            }
        }
        impl TryFrom<Value> for $name {
            type Error = Error;
            fn try_from(v: Value) -> Result<Self> {
                $validate(&v)?;
                Ok(Self(v))
            }
        }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                self.0.serialize(s)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
                Self::try_from(Value::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

wire_record!(SpeechConstitution, validate_speech_constitution);

impl SpeechConstitution {
    fn str_field(&self, key: &str) -> &str {
        self.0[key].as_str().unwrap_or_default()
    }
    pub fn constitution_ref(&self) -> &str {
        self.str_field("constitution_ref")
    }
    pub fn agent_ref(&self) -> AgentRef {
        AgentRef::new(self.str_field("agent_ref")).expect("admitted constitution has an agent ref")
    }
    pub fn agency_ref(&self) -> AgencyRef {
        AgencyRef::new(self.str_field("agency_ref"))
            .expect("admitted constitution has an agency ref")
    }
    pub fn world_binding_ref(&self) -> WorldBindingRef {
        WorldBindingRef::new(self.str_field("world_binding_ref"))
            .expect("admitted constitution has a world binding ref")
    }
    pub fn agent_session_ref(&self) -> AgentSessionRef {
        AgentSessionRef::new(self.str_field("agent_session_ref"))
            .expect("admitted constitution has a session ref")
    }
    /// The resolved body: a ModelSurface identity, never an Agent, Agency or
    /// session.
    pub fn body_ref(&self) -> ModelSurfaceRef {
        ModelSurfaceRef::new(self.str_field("body_ref"))
            .expect("admitted constitution has a body ref")
    }
    pub fn body_revision(&self) -> Option<&str> {
        self.0["body_revision"].as_str()
    }
    pub fn provider_session_ref(&self) -> Option<ProviderSessionRef> {
        self.0["provider_binding"]["provider_session_ref"]
            .as_str()
            .map(|s| ProviderSessionRef::new(s).expect("admitted provider session ref"))
    }
    pub fn transport_connection_ref(&self) -> Option<TransportConnectionRef> {
        self.0["provider_binding"]["transport_connection_ref"]
            .as_str()
            .map(|s| TransportConnectionRef::new(s).expect("admitted transport connection ref"))
    }
    /// Whether the constituted body can actually hear and speak. Derived from
    /// the declared acoustic modalities and usable transforms, never from a
    /// declaration alone.
    pub fn speech_capable(&self) -> bool {
        let acoustic_in = self
            .modalities("input_modalities")
            .iter()
            .any(|m| m == "audio" || m == "speech");
        let acoustic_out = self
            .modalities("output_modalities")
            .iter()
            .any(|m| m == "audio" || m == "speech");
        acoustic_in && acoustic_out && self.body_usable()
    }
    /// Whether the body carries interactive speech in the realtime direction:
    /// acoustic both ways plus a usable full-duplex interaction.
    pub fn realtime_capable(&self) -> bool {
        self.speech_capable()
            && self
                .interaction_support("full-duplex-realtime")
                .is_supported()
    }
    fn modalities(&self, key: &str) -> Vec<String> {
        self.0[key]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn input_modalities(&self) -> Vec<String> {
        self.modalities("input_modalities")
    }
    pub fn output_modalities(&self) -> Vec<String> {
        self.modalities("output_modalities")
    }
    pub fn interaction_support(&self, capability: &str) -> SpeechSupport {
        SpeechSupport::from_value(&self.0["interaction"][capability]).unwrap_or(
            SpeechSupport::Unknown {
                reason: format!("capability {capability} was not adjudicated in this constitution"),
            },
        )
    }
    /// The structured tool-request channel. A request channel only: this
    /// grants no authority of its own.
    pub fn tool_request_channel(&self) -> SpeechSupport {
        self.interaction_support("tool-requests")
    }
    pub fn interruption_support(&self) -> SpeechSupport {
        SpeechSupport::from_value(&self.0["interruption"])
            .expect("admitted constitution carries a four-state interruption fact")
    }
    /// Only a supported interruption fact permits barge-in behaviour.
    pub fn supports_interruption(&self) -> bool {
        self.interruption_support().is_supported()
    }
    pub fn reconnect_support(&self) -> Option<&str> {
        self.0["connection"]["reconnect"].as_str()
    }
    fn has_unavailable_condition(&self) -> bool {
        self.0["conditions"]
            .as_array()
            .is_some_and(|cs| cs.iter().any(|c| c["condition"] == "unavailable"))
    }
    /// A body recorded unavailable cannot act; degraded bodies act with the
    /// recorded reduction.
    pub fn body_usable(&self) -> bool {
        !self.has_unavailable_condition()
    }
    /// The declared credential condition, when the body carries one.
    pub fn credential_condition(&self) -> Option<&str> {
        self.0["credential_condition"]["condition"].as_str()
    }
    /// The hint a `required` or `satisfied` credential condition names.
    pub fn credential_hint(&self) -> Option<&str> {
        self.0["credential_condition"]["hint"].as_str()
    }
    /// Whether the constituted body can read and write text.
    pub fn text_capable(&self) -> bool {
        let text_in = self
            .modalities("input_modalities")
            .iter()
            .any(|m| m == "text");
        let text_out = self
            .modalities("output_modalities")
            .iter()
            .any(|m| m == "text");
        text_in && text_out
    }

    /// The named availability of the speech body, derived from the carried
    /// facts alone.
    ///
    /// This is the disclosure law a downstream UI renders: a session whose
    /// body cannot hear and speak reads as **absent with a named gap** —
    /// never as a silently complete text agent — and the gap names itself:
    ///
    /// - `unavailable` — a recorded condition forbids the body outright
    ///   (cannot appear on a live session: constituting one is refused);
    /// - `none-supplied` — no acoustic modality was declared: no speech body
    ///   exists;
    /// - `credential-gated` — a speech body is declared but its credential
    ///   is required and unbound: the visible pre-key state;
    /// - `degraded` — a recorded availability condition reduces the body.
    ///
    /// A body that carries none of these reads `present`. Capability and
    /// availability stay two different facts: `speech_capable`,
    /// `realtime_capable` and `text_capable` ride beside this disclosure, and
    /// the conditions and credential facts stay unflattened. The swap path is
    /// disclosed by the session read model's `last_change`: a body can be
    /// constituted or changed later without touching the Agent/Agency
    /// identity.
    pub fn speech_body(&self) -> Value {
        let acoustic = |key: &str| {
            self.modalities(key)
                .iter()
                .any(|m| m == "audio" || m == "speech")
        };
        let conditions = self.0["conditions"].as_array().cloned().unwrap_or_default();
        let condition_reason = |kind: &str| {
            conditions
                .iter()
                .find(|c| c["condition"] == kind)
                .and_then(|c| c["reason"].as_str())
                .map(str::to_owned)
        };
        let gap = if let Some(detail) = condition_reason("unavailable") {
            Some(("unavailable", Some(detail)))
        } else if !acoustic("input_modalities") || !acoustic("output_modalities") {
            Some(("none-supplied", None))
        } else if self.credential_condition() == Some("required") {
            Some((
                "credential-gated",
                self.credential_hint().map(str::to_owned),
            ))
        } else {
            condition_reason("degraded").map(|detail| ("degraded", Some(detail)))
        };
        let state = if gap.is_some() { "absent" } else { "present" };
        let mut body = json!({
            "state": state,
            "body_ref": self.as_value()["body_ref"],
            "body_revision": self.as_value()["body_revision"],
            "input_modalities": self.as_value()["input_modalities"],
            "output_modalities": self.as_value()["output_modalities"],
            "text_capable": self.text_capable(),
            "speech_capable": self.speech_capable(),
            "realtime_capable": self.realtime_capable(),
            "conditions": conditions,
            "credential_condition": self.as_value()["credential_condition"],
        });
        if let Some((reason, detail)) = gap {
            body["reason"] = json!(reason);
            body["detail"] = detail.map(Value::String).unwrap_or(Value::Null);
        }
        body
    }
    pub fn resolved_at(&self) -> &str {
        self.str_field("resolved_at")
    }
}

/// The recorded fact that one AgentSession's material body changed, with the
/// enduring Agent/Agency identity preserved by admission — not inferred.
///
/// This is the instantiation/history receipt the identity law demands: a
/// provider, model or connection swap produces one of these, and the same
/// `AgentRef`/`AgencyRef` stands on both sides of it.
pub fn validate_speech_constitution_change(v: &Value) -> Result<()> {
    require(
        v.is_object(),
        "speech constitution change must be an object",
    )?;
    require(
        v["schema"] == SPEECH_CONSTITUTION_CHANGE_VERSION,
        "wrong schema; expected actuation.speech-constitution-change/v1",
    )?;
    for key in ["change_ref", "reason"] {
        field_text(&v[key], key)?;
    }
    let before = SpeechConstitution::try_from(v["before"].clone())
        .map_err(|e| Error::new(format!("before constitution: {e}")))?;
    let after = SpeechConstitution::try_from(v["after"].clone())
        .map_err(|e| Error::new(format!("after constitution: {e}")))?;
    // The law this receipt exists for: a body change must not remint identity.
    require(
        before.agent_ref() == after.agent_ref() && before.agency_ref() == after.agency_ref(),
        "a speech body change must not change Agent or Agency identity",
    )?;
    require(
        before.constitution_ref() != after.constitution_ref(),
        "a change receipt must record two distinct constitutions",
    )?;
    texts(&v["evidence_refs"])?;
    require(
        !v["evidence_refs"].as_array().is_some_and(Vec::is_empty),
        "a body change requires evidence of the change",
    )?;
    date(&v["changed_at"])?;
    no_secret_material_keys(v)
}
wire_record!(
    SpeechConstitutionChange,
    validate_speech_constitution_change
);

/// What actually moved between two constitutions. Losses are facts, not
/// judgements; the delta never softens them.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConstitutionDelta {
    pub body_changed: bool,
    pub model_changed: bool,
    pub provider_changed: bool,
    pub provider_session_changed: bool,
    pub transport_connection_changed: bool,
    pub connection_changed: bool,
    pub gained_interaction: Vec<String>,
    pub lost_interaction: Vec<String>,
    pub interruption_before: SpeechSupport,
    pub interruption_after: SpeechSupport,
    pub speech_capable_before: bool,
    pub speech_capable_after: bool,
}

fn delta(before: &SpeechConstitution, after: &SpeechConstitution) -> ConstitutionDelta {
    let interactions = |c: &SpeechConstitution| {
        c.0["interaction"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter(|(_, v)| SpeechSupport::from_value(v).is_ok_and(|s| s.is_usable()))
                    .map(|(k, _)| k.clone())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default()
    };
    let (before_i, after_i) = (interactions(before), interactions(after));
    let ref_of = |c: &SpeechConstitution, path: &str| lookup(c.as_value(), path).map(str::to_owned);
    let changed = |path: &str| ref_of(before, path) != ref_of(after, path);
    ConstitutionDelta {
        body_changed: before.body_ref().as_str() != after.body_ref().as_str(),
        model_changed: before.as_value()["model_relation"]["model_ref"]
            != after.as_value()["model_relation"]["model_ref"],
        provider_changed: changed("provider_binding.provider_ref"),
        provider_session_changed: changed("provider_binding.provider_session_ref"),
        transport_connection_changed: changed("provider_binding.transport_connection_ref"),
        connection_changed: before.as_value()["connection"] != after.as_value()["connection"],
        gained_interaction: after_i.difference(&before_i).cloned().collect(),
        lost_interaction: before_i.difference(&after_i).cloned().collect(),
        interruption_before: before.interruption_support(),
        interruption_after: after.interruption_support(),
        speech_capable_before: before.speech_capable(),
        speech_capable_after: after.speech_capable(),
    }
}

impl SpeechConstitutionChange {
    /// Record a body change between two admitted constitutions of one
    /// enduring Agent/Agency. Refuses when the identity changed — that is a
    /// different Agent, and this receipt must never paper over it.
    pub fn record(
        change_ref: impl AsRef<str>,
        before: SpeechConstitution,
        after: SpeechConstitution,
        reason: impl AsRef<str>,
        evidence_refs: Vec<ExternalRef>,
        changed_at: &str,
    ) -> Result<Self> {
        let delta = delta(&before, &after);
        Self::try_from(json!({
            "schema": SPEECH_CONSTITUTION_CHANGE_VERSION,
            "change_ref": change_ref.as_ref(),
            "agent_ref": before.as_value()["agent_ref"],
            "agency_ref": before.as_value()["agency_ref"],
            "world_binding_ref": before.as_value()["world_binding_ref"],
            "agent_session_before": before.as_value()["agent_session_ref"],
            "agent_session_after": after.as_value()["agent_session_ref"],
            "before": before.as_value(),
            "after": after.as_value(),
            "delta": delta,
            "reason": reason.as_ref(),
            "evidence_refs": evidence_refs.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
            "changed_at": changed_at,
        }))
    }
    pub fn delta(&self) -> ConstitutionDelta {
        serde_json::from_value(self.0["delta"].clone())
            .expect("admitted change carries its own delta")
    }
    /// The enduring identity this change preserved.
    pub fn identity(&self) -> (AgentRef, AgencyRef) {
        (
            AgentRef::new(self.str_field("agent_ref")).expect("admitted change has an agent ref"),
            AgencyRef::new(self.str_field("agency_ref"))
                .expect("admitted change has an agency ref"),
        )
    }
    fn str_field(&self, key: &str) -> &str {
        self.0[key].as_str().unwrap_or_default()
    }
    pub fn change_ref(&self) -> &str {
        self.str_field("change_ref")
    }
}

/// A structured tool request emitted by a speech/realtime model.
///
/// This wire record is a request, nothing more. It does not name a canonical
/// Action into existence, and carrying it never executes anything.
pub fn validate_speech_tool_request(v: &Value) -> Result<()> {
    require(v.is_object(), "speech tool request must be an object")?;
    require(
        v["schema"] == SPEECH_TOOL_DECISION_VERSION,
        "wrong schema; expected actuation.speech-tool-decision/v1",
    )?;
    for key in [
        "request_ref",
        "constitution_ref",
        "agent_session_ref",
        "requested_at",
    ] {
        field_text(&v[key], key)?;
    }
    optional_text(&v["proposed_action_ref"])?;
    // Payload refs ride the non-Action side of the surface: a model's tool
    // proposal is never recorded as a canonical Action projection.
    let payloads = texts(&v["payload_refs"])?;
    require(!payloads.is_empty(), "a tool request names what it wants")?;
    date(&v["requested_at"])?;
    no_secret_material_keys(v)
}
wire_record!(SpeechToolRequest, validate_speech_tool_request);

/// How the authority gate resolved one speech-model tool request. An
/// authorised decision is still not an execution; [`SpeechExecution`] exists
/// because the two must never be conflated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "resolution", rename_all = "kebab-case")]
pub enum SpeechToolResolution {
    Authorised,
    Refused,
}

wire_record!(SpeechToolDecision, validate_speech_tool_decision);

pub fn validate_speech_tool_decision(v: &Value) -> Result<()> {
    require(v.is_object(), "speech tool decision must be an object")?;
    require(
        v["schema"] == SPEECH_TOOL_DECISION_VERSION,
        "wrong schema; expected actuation.speech-tool-decision/v1",
    )?;
    for key in ["decision_ref", "decided_by", "decided_at"] {
        field_text(&v[key], key)?;
    }
    let request = SpeechToolRequest::try_from(v["request"].clone())
        .map_err(|e| Error::new(format!("decided request: {e}")))?;
    require(
        request.as_value()["constitution_ref"] == v["constitution_ref"],
        "decision must name the same constitution as its request",
    )?;
    match v["resolution"]["resolution"].as_str() {
        Some("authorised") => {
            field_text(&v["resolution"]["action_ref"], "authorised action ref")?;
            require(
                v["execution"].is_null(),
                "an authorisation is not an execution; record execution separately",
            )?;
        }
        Some("refused") => {
            field_text(&v["resolution"]["reason"], "refusal reason")?;
            one(
                &v["resolution"]["stage"],
                &["channel", "denied", "unauthorised"],
            )?;
        }
        _ => {
            return Err(Error::new(
                "decision resolution must be authorised or refused",
            ))
        }
    }
    date(&v["decided_at"])?;
    no_secret_material_keys(v)
}

impl SpeechToolDecision {
    fn str_field(&self, key: &str) -> &str {
        self.0[key].as_str().unwrap_or_default()
    }
    pub fn decision_ref(&self) -> &str {
        self.str_field("decision_ref")
    }
    pub fn is_authorised(&self) -> bool {
        self.0["resolution"]["resolution"] == "authorised"
    }
    /// Record the actual execution of an authorised decision. A refused or
    /// absent decision cannot be executed, and the authorisation itself stays
    /// immutable: execution is a separate receipt that cites it.
    pub fn record_execution(
        &self,
        execution_ref: impl AsRef<str>,
        evidence_refs: Vec<ExternalRef>,
        executed_at: &str,
    ) -> Result<SpeechExecution> {
        require(
            self.is_authorised(),
            "a refused tool request cannot be executed",
        )?;
        require(!evidence_refs.is_empty(), "execution requires evidence")?;
        SpeechExecution::try_from(json!({
            "schema": SPEECH_TOOL_DECISION_VERSION,
            "execution_ref": execution_ref.as_ref(),
            "decision_ref": self.decision_ref(),
            "action_ref": self.0["resolution"]["action_ref"],
            "constitution_ref": self.str_field("constitution_ref"),
            "agent_session_ref": self.0["request"]["agent_session_ref"],
            "evidence_refs": evidence_refs.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
            "executed_at": executed_at,
        }))
    }
}

wire_record!(SpeechExecution, validate_speech_execution);

pub fn validate_speech_execution(v: &Value) -> Result<()> {
    require(v.is_object(), "speech execution must be an object")?;
    require(
        v["schema"] == SPEECH_TOOL_DECISION_VERSION,
        "wrong schema; expected actuation.speech-tool-decision/v1",
    )?;
    for key in [
        "execution_ref",
        "decision_ref",
        "action_ref",
        "constitution_ref",
        "agent_session_ref",
        "executed_at",
    ] {
        field_text(&v[key], key)?;
    }
    texts(&v["evidence_refs"])?;
    require(
        !v["evidence_refs"].as_array().is_some_and(Vec::is_empty),
        "execution requires evidence",
    )?;
    date(&v["executed_at"])?;
    no_secret_material_keys(v)
}

/// Adjudicate one speech-model tool request against the body's declared
/// tool-request channel and the governing autonomy the caller supplies.
///
/// The gate enforces the authority law in order:
///
/// ```text
/// model request != canonical Action   (the request is only a proposal)
/// available      != authorised        (a supported channel is not permission)
/// authorised     != executed          (execution is a separate receipt)
/// ```
///
/// Authority comes only from the supplied allowed/denied action refs — the
/// same explicit-list grammar the core `DelegatedAutonomy` uses (absence is
/// not permission). The transport, provider, modality facts and the audio
/// channel itself appear nowhere in the authority inputs.
pub fn adjudicate_speech_tool_request(
    decision_ref: impl AsRef<str>,
    constitution: &SpeechConstitution,
    request: SpeechToolRequest,
    allowed_action_refs: &[ExternalRef],
    denied_action_refs: &[ExternalRef],
    decided_by: impl AsRef<str>,
    decided_at: &str,
) -> Result<SpeechToolDecision> {
    require(
        request.as_value()["constitution_ref"] == constitution.constitution_ref(),
        "tool request belongs to another constitution",
    )?;
    let proposed = request.as_value()["proposed_action_ref"].as_str();
    let resolution = if !constitution.tool_request_channel().is_usable() {
        json!({
            "resolution": "refused", "stage": "channel",
            "reason": "the constituted body does not carry a usable structured tool-request channel",
        })
    } else if let Some(action) = proposed {
        let action = ExternalRef::new(action)?;
        if denied_action_refs.contains(&action) {
            json!({"resolution": "refused", "stage": "denied",
                   "reason": "the proposed action is explicitly denied by the governing autonomy"})
        } else if allowed_action_refs.contains(&action) {
            json!({"resolution": "authorised", "action_ref": action.as_str()})
        } else {
            json!({"resolution": "refused", "stage": "unauthorised",
                   "reason": "available capability is not authorised authority: the action is absent from the explicit allowed list"})
        }
    } else {
        json!({"resolution": "refused", "stage": "unauthorised",
               "reason": "the request proposes no canonical action; there is nothing to authorise"})
    };
    SpeechToolDecision::try_from(json!({
        "schema": SPEECH_TOOL_DECISION_VERSION,
        "decision_ref": decision_ref.as_ref(),
        "request": request.as_value(),
        "constitution_ref": constitution.constitution_ref(),
        "resolution": resolution,
        "decided_by": decided_by.as_ref(),
        "decided_at": decided_at,
    }))
}

/// The provenance line a constitution cites for the AIKit resolution it
/// carries: the read-model document and its revision. Provider spellings stay
/// inside these refs.
pub fn aikit_resolution_ref(read_model_ref: impl AsRef<str>, revision: impl AsRef<str>) -> String {
    format!(
        "aikit:model-runtime:{}@{}",
        read_model_ref.as_ref(),
        revision.as_ref()
    )
}

/// Attach an admitted speech constitution to an instantiation receipt value,
/// correlating identity exactly: a receipt cannot carry another session's
/// body. Shape alone is not correlation.
pub fn attach_speech_constitution(
    receipt: &Value,
    constitution: &SpeechConstitution,
) -> Result<Value> {
    require(
        receipt["schema"] == INSTANTIATION_VERSION,
        "expected an actuation.instantiation/v1 receipt",
    )?;
    require(
        receipt["agency_ref"] == constitution.as_value()["agency_ref"],
        "constitution belongs to another Agency",
    )?;
    require(
        receipt["world_binding_ref"] == constitution.as_value()["world_binding_ref"],
        "constitution belongs to another WorldBinding",
    )?;
    if !receipt["agent_session_ref"].is_null() {
        require(
            receipt["agent_session_ref"] == constitution.as_value()["agent_session_ref"],
            "constitution belongs to another AgentSession",
        )?;
    }
    let mut attached = receipt.clone();
    attached["speech_constitution"] = constitution.as_value().clone();
    Ok(attached)
}
