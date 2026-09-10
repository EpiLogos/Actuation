use crate::{
    ActionRef, ActivityRef, ActuationRef, AgencyIdentity, AgencyRef, AgentRef, AgentSessionRef,
    AuthorityRef, BoundsRef, DeterminationRef, Error, Extensions, ExternalRef, GrantRef, Invariant,
    InvocationRef, JourneyRef, NonEmpty, PlanRef, Record, RequestRef, Result, ReturnRef,
    ReturnRelationRef, RunRef, ScopeRef, Slot, WorldBindingRef, WorldRef,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AgencySchema {
    #[default]
    #[serde(rename = "actuation.agency/v1")]
    V1,
}

macro_rules! invariant {
    ($ty:ty, [$($field:ident),*], $check:expr) => {
        impl Invariant for $ty {
            const FIELDS: &'static [&'static str] = &[$(stringify!($field)),*];
            fn extensions(&self) -> &Extensions { &self.extensions }
            fn check(&self) -> Result<()> { ($check)(self) }
        }
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeterminationKind {
    SelfDifferentiation,
    Delegation,
    Derivation,
    Federation,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetagencyOperation {
    ActualiseAgency,
    ConfigureAgency,
    DetermineAgency,
    ReintegrateReturn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReturnMode {
    Required,
    Optional,
    AutonomousTermination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecognitionState {
    Pending,
    Recognised,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutationState {
    NotApplied,
    Applied,
}

/// Reading of actual supplied Return standing, not a Recognition decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReturnStanding {
    Offered,
    Received,
    Rejected,
    Recognised,
    Reentered,
}

/// Explicit action lists describe bounded autonomy. Absence is not permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionStanding {
    Allowed,
    Denied,
    Unspecified,
}

pub type WorldConstraints = Record<WorldConstraintsFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldConstraintsFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub human_authored_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub security_policy_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub external_reality_refs: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    WorldConstraintsFields,
    [
        human_authored_refs,
        security_policy_refs,
        evidence_refs,
        external_reality_refs
    ],
    |_: &Self| Ok(())
);

pub type WorldBinding = Record<WorldBindingFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldBindingFields {
    pub schema: AgencySchema,
    pub binding_ref: WorldBindingRef,
    pub agent_ref: AgentRef,
    pub agency_ref: AgencyRef,
    pub world_ref: WorldRef,
    pub scope_ref: ScopeRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub purpose_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub continuity_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub determining_agency_ref: Slot<AgencyRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub return_relation_ref: Slot<ReturnRelationRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub bounds_refs: Slot<Vec<BoundsRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub authority_refs: Slot<Vec<AuthorityRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub constraints: Slot<WorldConstraints>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    WorldBindingFields,
    [
        schema,
        binding_ref,
        agent_ref,
        agency_ref,
        world_ref,
        scope_ref,
        purpose_ref,
        continuity_ref,
        determining_agency_ref,
        return_relation_ref,
        bounds_refs,
        authority_refs,
        constraints
    ],
    |_: &Self| Ok(())
);

pub type RootScope = Record<RootScopeFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RootScopeFields {
    pub schema: AgencySchema,
    pub scope_ref: ScopeRef,
    pub enclosing_world_ref: WorldRef,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    RootScopeFields,
    [schema, scope_ref, enclosing_world_ref],
    |_: &Self| Ok(())
);

pub type MetagencyGrant = Record<MetagencyGrantFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetagencyGrantFields {
    pub schema: AgencySchema,
    pub grant_ref: GrantRef,
    pub agency_ref: AgencyRef,
    pub world_binding_ref: WorldBindingRef,
    pub authority_ref: AuthorityRef,
    pub operations: NonEmpty<MetagencyOperation>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub bounds_refs: Slot<Vec<BoundsRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    MetagencyGrantFields,
    [
        schema,
        grant_ref,
        agency_ref,
        world_binding_ref,
        authority_ref,
        operations,
        bounds_refs
    ],
    |_: &Self| Ok(())
);

pub type DelegatedAutonomy = Record<DelegatedAutonomyFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DelegatedAutonomyFields {
    pub may_determine_within_bounds: bool,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub allowed_action_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub denied_action_refs: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    DelegatedAutonomyFields,
    [
        may_determine_within_bounds,
        allowed_action_refs,
        denied_action_refs
    ],
    |_: &Self| Ok(())
);

pub type ReturnPolicy = Record<ReturnPolicyFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReturnPolicyFields {
    pub mode: ReturnMode,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub return_relation_ref: Slot<ReturnRelationRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    ReturnPolicyFields,
    [mode, return_relation_ref],
    |value: &Self| {
        if value.mode != ReturnMode::AutonomousTermination
            && value.return_relation_ref.value().is_none()
        {
            return Err(Error::new(
                "a required or optional Return must name its Return relation",
            ));
        }
        Ok(())
    }
);

pub type Determination = Record<DeterminationFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeterminationFields {
    pub schema: AgencySchema,
    pub determination_ref: DeterminationRef,
    pub kind: DeterminationKind,
    pub determining_agency_ref: AgencyRef,
    pub differentiated_agency_ref: AgencyRef,
    pub world_binding_ref: WorldBindingRef,
    pub bounds_refs: NonEmpty<BoundsRef>,
    pub delegated_autonomy: DelegatedAutonomy,
    pub return_policy: ReturnPolicy,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub parent_determination_ref: Slot<DeterminationRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub authority_refs: Slot<Vec<AuthorityRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    DeterminationFields,
    [
        schema,
        determination_ref,
        kind,
        determining_agency_ref,
        differentiated_agency_ref,
        world_binding_ref,
        bounds_refs,
        delegated_autonomy,
        return_policy,
        parent_determination_ref,
        authority_refs
    ],
    |value: &Self| {
        if value.kind == DeterminationKind::Federation
            && value
                .authority_refs
                .value()
                .is_some_and(|refs| !refs.is_empty())
        {
            return Err(Error::new(
                "federation does not grant determining authority; use explicit delegation",
            ));
        }
        Ok(())
    }
);

pub type ReturnProvenance = Record<ReturnProvenanceFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReturnProvenanceFields {
    pub agency_lineage_refs: NonEmpty<AgencyRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_refs: Slot<Vec<AgentRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub world_binding_refs: Slot<Vec<WorldBindingRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub authority_decision_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub authority_refs: Slot<Vec<AuthorityRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub bounds_refs: Slot<Vec<BoundsRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub activity_refs: Slot<Vec<ActivityRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub actuation_refs: Slot<Vec<ActuationRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub result_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub request_refs: Slot<Vec<RequestRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_session_refs: Slot<Vec<AgentSessionRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub action_refs: Slot<Vec<ActionRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub invocation_refs: Slot<Vec<InvocationRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub plan_refs: Slot<Vec<PlanRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub journey_refs: Slot<Vec<JourneyRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub run_refs: Slot<Vec<RunRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub provider_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub harness_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub material_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub external_source_refs: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    ReturnProvenanceFields,
    [
        agency_lineage_refs,
        agent_refs,
        world_binding_refs,
        authority_decision_refs,
        authority_refs,
        bounds_refs,
        activity_refs,
        actuation_refs,
        result_refs,
        request_refs,
        agent_session_refs,
        action_refs,
        invocation_refs,
        plan_refs,
        journey_refs,
        run_refs,
        provider_refs,
        harness_refs,
        material_refs,
        external_source_refs
    ],
    |_: &Self| Ok(())
);

pub type Return = Record<ReturnFields>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReturnFields {
    pub schema: AgencySchema,
    pub return_ref: ReturnRef,
    pub determination_ref: DeterminationRef,
    pub from_agency_ref: AgencyRef,
    pub to_agency_ref: AgencyRef,
    pub difference_refs: NonEmpty<ExternalRef>,
    pub provenance: ReturnProvenance,
    pub received: bool,
    pub recognition_state: RecognitionState,
    pub world_mutation_state: MutationState,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub artifact_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub claim_refs: Slot<Vec<ExternalRef>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
invariant!(
    ReturnFields,
    [
        schema,
        return_ref,
        determination_ref,
        from_agency_ref,
        to_agency_ref,
        difference_refs,
        provenance,
        received,
        recognition_state,
        world_mutation_state,
        artifact_refs,
        claim_refs,
        evidence_refs
    ],
    |value: &Self| {
        if !value.received && value.recognition_state != RecognitionState::Pending {
            return Err(Error::new(
                "a Return cannot be recognised or rejected before reception",
            ));
        }
        if value.world_mutation_state == MutationState::Applied
            && value.recognition_state != RecognitionState::Recognised
        {
            return Err(Error::new(
                "world mutation requires an explicitly recognised Return",
            ));
        }
        Ok(())
    }
);

impl WorldBindingFields {
    pub fn new(
        binding_ref: WorldBindingRef,
        agent_ref: AgentRef,
        agency_ref: AgencyRef,
        world_ref: WorldRef,
        scope_ref: ScopeRef,
    ) -> Self {
        Self {
            schema: AgencySchema::V1,
            binding_ref,
            agent_ref,
            agency_ref,
            world_ref,
            scope_ref,
            purpose_ref: Slot::Absent,
            continuity_ref: Slot::Absent,
            determining_agency_ref: Slot::Absent,
            return_relation_ref: Slot::Absent,
            bounds_refs: Slot::Absent,
            authority_refs: Slot::Absent,
            constraints: Slot::Absent,
            extensions: Extensions::new(),
        }
    }
}
impl WorldBinding {
    pub fn identity(&self) -> AgencyIdentity {
        AgencyIdentity {
            agent: self.fields().agent_ref.clone(),
            agency: self.fields().agency_ref.clone(),
        }
    }
    pub fn is_root_for(&self, scope: &RootScope) -> bool {
        self.fields().scope_ref == scope.fields().scope_ref
            && self.fields().world_ref == scope.fields().enclosing_world_ref
    }
    pub fn bounds(&self) -> &[BoundsRef] {
        self.fields()
            .bounds_refs
            .value()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn authorities(&self) -> &[AuthorityRef] {
        self.fields()
            .authority_refs
            .value()
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
impl RootScopeFields {
    pub fn new(scope_ref: ScopeRef, enclosing_world_ref: WorldRef) -> Self {
        Self {
            schema: AgencySchema::V1,
            scope_ref,
            enclosing_world_ref,
            extensions: Extensions::new(),
        }
    }
}
impl MetagencyGrant {
    /// This only reads the grant. Actualisation separately checks its exact
    /// WorldBinding, holder, authority source and bounds before admitting an act.
    pub fn includes(&self, operation: MetagencyOperation) -> bool {
        self.fields().operations.as_slice().contains(&operation)
    }
}
impl DelegatedAutonomy {
    pub fn action_standing(&self, action: &ExternalRef) -> ActionStanding {
        if self
            .fields()
            .denied_action_refs
            .value()
            .is_some_and(|refs| refs.contains(action))
        {
            ActionStanding::Denied
        } else if self
            .fields()
            .allowed_action_refs
            .value()
            .is_some_and(|refs| refs.contains(action))
        {
            ActionStanding::Allowed
        } else {
            ActionStanding::Unspecified
        }
    }
}
impl Return {
    pub fn standing(&self) -> ReturnStanding {
        let value = self.fields();
        if !value.received {
            ReturnStanding::Offered
        } else if value.world_mutation_state == MutationState::Applied {
            ReturnStanding::Reentered
        } else {
            match value.recognition_state {
                RecognitionState::Pending => ReturnStanding::Received,
                RecognitionState::Rejected => ReturnStanding::Rejected,
                RecognitionState::Recognised => ReturnStanding::Recognised,
            }
        }
    }
    /// Record reception only. No method here decides Recognition or applies a
    /// returned change to another owner's World.
    pub fn receive(self) -> Self {
        let mut fields = self.into_fields();
        fields.received = true;
        // Reception cannot invalidate an already admitted Return.
        Self::new(fields).expect("reception preserves the Return invariant")
    }
}

/// Validated aggregate lineage reading. As in the public v1 contract, this is
/// not a claim that its list is a unique, complete, ordered actualisation path.
/// R3's actualisation admission must require that stronger condition separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DeterminationLineage(NonEmpty<Determination>);
impl DeterminationLineage {
    pub fn new(items: NonEmpty<Determination>) -> Result<Self> {
        let by_ref: std::collections::BTreeMap<_, _> = items
            .as_slice()
            .iter()
            .map(|item| (&item.fields().determination_ref, item))
            .collect();
        for item in items.as_slice() {
            if let Some(parent_ref) = item.fields().parent_determination_ref.value() {
                let parent = by_ref.get(parent_ref).ok_or_else(|| {
                    Error::new(format!("missing parent determination {parent_ref}"))
                })?;
                if parent.fields().differentiated_agency_ref != item.fields().determining_agency_ref
                {
                    return Err(Error::new("recursive lineage must continue through its parent's differentiated Agency"));
                }
                if !parent
                    .fields()
                    .delegated_autonomy
                    .fields()
                    .may_determine_within_bounds
                {
                    return Err(Error::new(
                        "downward determination requires explicit delegated autonomy",
                    ));
                }
            }
        }
        Ok(Self(items))
    }
    pub fn as_slice(&self) -> &[Determination] {
        self.0.as_slice()
    }
}
impl<'de> Deserialize<'de> for DeterminationLineage {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        Self::new(NonEmpty::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
