use crate::invariant;
use actuation_core::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const REALISED_ACTUATION_VERSION: &str = "actuation.realised/v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RealisedSchema {
    #[serde(rename = "actuation.realised/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Recurrence {
    SingleShot,
    TurnBased,
    EventDriven,
    Continuous,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationState {
    Observed,
    Partial,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoopFacts(Record<LoopFactsFields>);
impl LoopFacts {
    pub fn new(fields: LoopFactsFields) -> Result<Self> {
        Ok(Self(Record::new(fields)?))
    }
    pub fn fields(&self) -> &LoopFactsFields {
        self.0.fields()
    }
    pub fn into_fields(self) -> LoopFactsFields {
        self.0.into_fields()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoopFactsFields {
    pub recurrence: Recurrence,
    pub acting: bool,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub entrypoint_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub observed_faculties: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    LoopFactsFields,
    [recurrence, acting, entrypoint_ref, observed_faculties],
    |v: &Self| {
        if !v.acting {
            Err(Error::new(
                "inert model availability is not realised Actuation",
            ))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BodyRelation(Record<BodyRelationFields>);
impl BodyRelation {
    pub fn new(fields: BodyRelationFields) -> Result<Self> {
        Ok(Self(Record::new(fields)?))
    }
    pub fn fields(&self) -> &BodyRelationFields {
        self.0.fields()
    }
    pub fn into_fields(self) -> BodyRelationFields {
        self.0.into_fields()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BodyRelationFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub harness_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub session_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub process_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub model_condition_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub material_binding_ref: Slot<ExternalRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    BodyRelationFields,
    [
        harness_ref,
        session_ref,
        process_ref,
        model_condition_ref,
        material_binding_ref
    ],
    |_: &Self| { Ok(()) }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Observation(Record<ObservationFields>);
impl Observation {
    pub fn new(fields: ObservationFields) -> Result<Self> {
        Ok(Self(Record::new(fields)?))
    }
    pub fn fields(&self) -> &ObservationFields {
        self.0.fields()
    }
    pub fn into_fields(self) -> ObservationFields {
        self.0.into_fields()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservationFields {
    pub state: ObservationState,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub unsupported_faculties: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub degraded_faculties: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    ObservationFields,
    [
        state,
        evidence_refs,
        unsupported_faculties,
        degraded_faculties
    ],
    |v: &Self| {
        if v.state == ObservationState::Observed
            && v.evidence_refs.value().is_none_or(Vec::is_empty)
        {
            Err(Error::new("observed actuality must carry evidence"))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LifecycleSupport(Record<LifecycleSupportFields>);
impl LifecycleSupport {
    pub fn new(fields: LifecycleSupportFields) -> Result<Self> {
        Ok(Self(Record::new(fields)?))
    }
    pub fn fields(&self) -> &LifecycleSupportFields {
        self.0.fields()
    }
    pub fn into_fields(self) -> LifecycleSupportFields {
        self.0.into_fields()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LifecycleSupportFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub interrupt: Slot<bool>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub cancel: Slot<bool>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub terminate: Slot<bool>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    LifecycleSupportFields,
    [interrupt, cancel, terminate],
    |_: &Self| { Ok(()) }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RealisedActuation(Record<RealisedActuationFields>);
impl RealisedActuation {
    pub fn new(fields: RealisedActuationFields) -> Result<Self> {
        Ok(Self(Record::new(fields)?))
    }
    pub fn fields(&self) -> &RealisedActuationFields {
        self.0.fields()
    }
    pub fn into_fields(self) -> RealisedActuationFields {
        self.0.into_fields()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RealisedActuationFields {
    pub schema: RealisedSchema,
    pub realised_ref: RealisedRef,
    pub actuation_ref: ActuationRef,
    pub agent_ref: AgentRef,
    pub agency_ref: AgencyRef,
    pub world_binding_ref: WorldBindingRef,
    #[serde(rename = "loop")]
    pub loop_facts: LoopFacts,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub body: Slot<BodyRelation>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub participating_loci: Slot<Vec<LocusRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub stream_ref: Slot<StreamRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub return_ref: Slot<ReturnRef>,
    pub observation: Observation,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub lifecycle: Slot<LifecycleSupport>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
impl Invariant for RealisedActuationFields {
    const FIELDS: &'static [&'static str] = &[
        "schema",
        "realised_ref",
        "actuation_ref",
        "agent_ref",
        "agency_ref",
        "world_binding_ref",
        "loop",
        "body",
        "participating_loci",
        "stream_ref",
        "return_ref",
        "observation",
        "lifecycle",
    ];
    fn extensions(&self) -> &Extensions {
        &self.extensions
    }
    fn check(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for BodyRelationFields {
    fn default() -> Self {
        Self {
            harness_ref: Slot::Absent,
            session_ref: Slot::Absent,
            process_ref: Slot::Absent,
            model_condition_ref: Slot::Absent,
            material_binding_ref: Slot::Absent,
            extensions: Extensions::new(),
        }
    }
}
impl RealisedActuation {
    pub fn identity(&self) -> AgencyIdentity {
        AgencyIdentity {
            agent: self.fields().agent_ref.clone(),
            agency: self.fields().agency_ref.clone(),
        }
    }
    /// A receipt is attributable to precisely the supplied binding. Material
    /// or harness refs cannot stand in for any of these identity roles.
    pub fn belongs_to(&self, binding: &WorldBinding) -> bool {
        self.identity() == binding.identity()
            && self.fields().world_binding_ref == binding.fields().binding_ref
    }
    pub fn reading(&self) -> Value {
        let v = self.fields();
        let body = v.body.value().map(BodyRelation::fields);
        let observation = v.observation.fields();
        json!({
            "schema": REALISED_ACTUATION_VERSION, "realised_ref": v.realised_ref,
            "actuation_ref": v.actuation_ref, "agent_ref": v.agent_ref, "agency_ref": v.agency_ref,
            "world_binding_ref": v.world_binding_ref, "recurrence": v.loop_facts.fields().recurrence,
            "harness_ref": body.and_then(|b| b.harness_ref.value()),
            "session_ref": body.and_then(|b| b.session_ref.value()),
            "model_condition_ref": body.and_then(|b| b.model_condition_ref.value()),
            "material_binding_ref": body.and_then(|b| b.material_binding_ref.value()),
            "stream_ref": v.stream_ref.value(), "return_ref": v.return_ref.value(),
            "participating_loci": v.participating_loci.value().map(Vec::as_slice).unwrap_or(&[]),
            "observation": {
                "state": observation.state,
                "evidence_refs": observation.evidence_refs.value().map(Vec::as_slice).unwrap_or(&[]),
                "unsupported_faculties": observation.unsupported_faculties.value().map(Vec::as_slice).unwrap_or(&[]),
                "degraded_faculties": observation.degraded_faculties.value().map(Vec::as_slice).unwrap_or(&[])
            }
        })
    }
    pub fn continuity_to(&self, next: &Self) -> ContinuityDelta {
        let previous = self.fields();
        let next = next.fields();
        let old_body = previous
            .body
            .value()
            .map(BodyRelation::fields)
            .cloned()
            .unwrap_or_default();
        let new_body = next
            .body
            .value()
            .map(BodyRelation::fields)
            .cloned()
            .unwrap_or_default();
        ContinuityDelta {
            same_agent: previous.agent_ref == next.agent_ref,
            same_agency: previous.agency_ref == next.agency_ref,
            same_world_binding: previous.world_binding_ref == next.world_binding_ref,
            same_actuation: previous.actuation_ref == next.actuation_ref,
            harness_changed: old_body.harness_ref != new_body.harness_ref,
            session_changed: old_body.session_ref != new_body.session_ref,
            process_changed: old_body.process_ref != new_body.process_ref,
            model_condition_changed: old_body.model_condition_ref != new_body.model_condition_ref,
            material_binding_changed: old_body.material_binding_ref
                != new_body.material_binding_ref,
            recurrence_changed: previous.loop_facts.fields().recurrence
                != next.loop_facts.fields().recurrence,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContinuityDelta {
    pub same_agent: bool,
    pub same_agency: bool,
    pub same_world_binding: bool,
    pub same_actuation: bool,
    pub harness_changed: bool,
    pub session_changed: bool,
    pub process_changed: bool,
    pub model_condition_changed: bool,
    pub material_binding_changed: bool,
    pub recurrence_changed: bool,
}
