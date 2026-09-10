use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// A World-relative constitutional composition, not a manager/worker taxonomy.
/// A single binding with no Factory ancestry is a complete ordinary reading.
/// Adding relations retains each independent identity and attributable Return.
#[derive(Clone, Debug)]
pub struct AgenticComposition {
    binding: WorldBinding,
    root_scope: Option<RootScope>,
    grants: Vec<MetagencyGrant>,
    determinations: Option<DeterminationLineage>,
    returns: Vec<Return>,
}
impl AgenticComposition {
    pub fn new(binding: WorldBinding) -> Self {
        Self {
            binding,
            root_scope: None,
            grants: vec![],
            determinations: None,
            returns: vec![],
        }
    }
    pub fn with_root_scope(mut self, scope: RootScope) -> Self {
        self.root_scope = Some(scope);
        self
    }
    pub fn with_grants(mut self, grants: Vec<MetagencyGrant>) -> Self {
        self.grants = grants;
        self
    }
    pub fn with_lineage(mut self, lineage: DeterminationLineage) -> Self {
        self.determinations = Some(lineage);
        self
    }
    pub fn with_returns(mut self, returns: Vec<Return>) -> Self {
        self.returns = returns;
        self
    }
    pub fn binding(&self) -> &WorldBinding {
        &self.binding
    }
    pub fn read(&self) -> AgencyReading {
        let binding = self.binding.fields();
        let grants: Vec<_> = self
            .grants
            .iter()
            .filter(|grant| grant.fields().agency_ref == binding.agency_ref)
            .collect();
        let operations: BTreeSet<_> = grants
            .iter()
            .flat_map(|grant| grant.fields().operations.as_slice().iter().copied())
            .collect();
        let lineage = self
            .determinations
            .as_ref()
            .map(DeterminationLineage::as_slice)
            .unwrap_or(&[]);
        let incoming: Vec<_> = self
            .returns
            .iter()
            .filter(|returned| returned.fields().to_agency_ref == binding.agency_ref)
            .collect();
        AgencyReading {
            schema: AgencySchema::V1,
            agency_ref: binding.agency_ref.clone(),
            agent_ref: binding.agent_ref.clone(),
            world_binding_ref: binding.binding_ref.clone(),
            world_ref: binding.world_ref.clone(),
            scope_ref: binding.scope_ref.clone(),
            root_for_scope: self
                .root_scope
                .as_ref()
                .is_some_and(|scope| self.binding.is_root_for(scope)),
            metagency: MetagencyReading {
                available: !grants.is_empty(),
                operations: operations.into_iter().collect(),
                grant_refs: grants
                    .iter()
                    .map(|grant| grant.fields().grant_ref.clone())
                    .collect(),
            },
            governing_determination_refs: lineage
                .iter()
                .filter(|item| item.fields().determining_agency_ref == binding.agency_ref)
                .map(|item| item.fields().determination_ref.clone())
                .collect(),
            governed_by_determination_refs: lineage
                .iter()
                .filter(|item| item.fields().differentiated_agency_ref == binding.agency_ref)
                .map(|item| item.fields().determination_ref.clone())
                .collect(),
            returns: ReturnReading {
                pending: incoming
                    .iter()
                    .filter(|item| !item.fields().received)
                    .map(|item| item.fields().return_ref.clone())
                    .collect(),
                received: incoming
                    .iter()
                    .filter(|item| item.fields().received)
                    .map(|item| item.fields().return_ref.clone())
                    .collect(),
                recognised: incoming
                    .iter()
                    .filter(|item| item.fields().recognition_state == RecognitionState::Recognised)
                    .map(|item| item.fields().return_ref.clone())
                    .collect(),
                world_mutated: incoming
                    .iter()
                    .filter(|item| item.fields().world_mutation_state == MutationState::Applied)
                    .map(|item| item.fields().return_ref.clone())
                    .collect(),
                records: self
                    .returns
                    .iter()
                    .filter(|item| {
                        item.fields().from_agency_ref == binding.agency_ref
                            || item.fields().to_agency_ref == binding.agency_ref
                    })
                    .cloned()
                    .collect(),
            },
            constraints: binding
                .constraints
                .value()
                .map(|value| serde_json::to_value(value).expect("validated constraints serialize"))
                .unwrap_or_else(|| json!({})),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AgencyReading {
    pub schema: AgencySchema,
    pub agency_ref: AgencyRef,
    pub agent_ref: AgentRef,
    pub world_binding_ref: WorldBindingRef,
    pub world_ref: WorldRef,
    pub scope_ref: ScopeRef,
    pub root_for_scope: bool,
    pub metagency: MetagencyReading,
    pub governing_determination_refs: Vec<DeterminationRef>,
    pub governed_by_determination_refs: Vec<DeterminationRef>,
    pub returns: ReturnReading,
    pub constraints: Value,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MetagencyReading {
    pub available: bool,
    pub operations: Vec<MetagencyOperation>,
    pub grant_refs: Vec<GrantRef>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReturnReading {
    pub pending: Vec<ReturnRef>,
    pub received: Vec<ReturnRef>,
    pub recognised: Vec<ReturnRef>,
    pub world_mutated: Vec<ReturnRef>,
    pub records: Vec<Return>,
}

/// The published aggregate JSON reading is an application boundary over the
/// same typed composition; no constitutional behaviour lives in a CLI parser.
pub fn agency_reading(input: Value) -> Result<AgencyReading> {
    #[derive(Deserialize)]
    struct Input {
        binding: WorldBinding,
        #[serde(default)]
        root_scope: Value,
        #[serde(default)]
        metagency_grants: Vec<MetagencyGrant>,
        #[serde(default)]
        determinations: Vec<Determination>,
        #[serde(default)]
        returns: Vec<Return>,
    }
    if !input.is_object() {
        return Err(Error::new("Agency reading input must be an object"));
    }
    let input: Input = serde_json::from_value(input)?;
    let mut composition = AgenticComposition::new(input.binding)
        .with_grants(input.metagency_grants)
        .with_returns(input.returns);
    // The old aggregate route treats a missing/null/false root reading as absent.
    let root_present = match &input.root_scope {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        _ => true,
    };
    if root_present {
        composition = composition.with_root_scope(serde_json::from_value(input.root_scope)?);
    }
    if !input.determinations.is_empty() {
        composition = composition.with_lineage(DeterminationLineage::new(NonEmpty::new(
            input.determinations,
        )?)?);
    }
    Ok(composition.read())
}
