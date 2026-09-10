use actuation_core::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const AGENCY_ACTUALISATION_VERSION: &str = "actuation.agency-actualisation/v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActualisationSchema {
    #[serde(rename = "actuation.agency-actualisation/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentityStanding {
    Existing,
    Actualised,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityEvidence {
    pub standing: IdentityStanding,
    pub evidence_refs: NonEmpty<ExternalRef>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualisationProvenance {
    pub source_refs: NonEmpty<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub context_refs: Slot<Vec<ExternalRef>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualisationRequestFields {
    pub schema: ActualisationSchema,
    pub request_ref: RequestRef,
    pub requester_ref: ExternalRef,
    pub governing_binding: WorldBinding,
    pub metagency_grant: MetagencyGrant,
    pub determination: Determination,
    pub differentiated_binding: WorldBinding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub prior_determinations: Slot<Vec<Determination>>,
    pub agent_identity: IdentityEvidence,
    pub provenance: ActualisationProvenance,
}
/// A request is not yet an authorised act. Admission checks the entire relation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ActualisationRequest(ActualisationRequestFields);
impl ActualisationRequest {
    pub fn new(fields: ActualisationRequestFields) -> Self {
        Self(fields)
    }
    pub fn fields(&self) -> &ActualisationRequestFields {
        &self.0
    }
    pub fn admit(&self) -> Result<Actualisation> {
        Actualisation::admit(self.clone())
    }
}
impl<'de> Deserialize<'de> for ActualisationRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        for path in [&[][..], &["agent_identity"][..], &["provenance"][..]] {
            let mut object = &value;
            for key in path {
                object = &object[*key];
            }
            if !object.is_object() {
                return Err(serde::de::Error::custom(
                    "actualisation request and evidence must be objects",
                ));
            }
        }
        serde_json::from_value(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
fn require(condition: bool, reason: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error::new(reason))
    }
}
fn same_unique_set<T: Ord>(left: &[T], right: &[T]) -> bool {
    let left_set: BTreeSet<_> = left.iter().collect();
    let right_set: BTreeSet<_> = right.iter().collect();
    left.len() == right.len()
        && left_set.len() == left.len()
        && right_set.len() == right.len()
        && left_set == right_set
}
/// Strong ordered ancestry used by an actualisation, distinct from aggregate
/// lineage reading. An explicit null root parent is not an omitted parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActualisationLineage(NonEmpty<Determination>);
impl ActualisationLineage {
    pub fn new(prior: &[Determination], determination: Determination) -> Result<Self> {
        let mut items = prior.to_vec();
        items.push(determination);
        let unique: BTreeSet<_> = items
            .iter()
            .map(|d| &d.fields().determination_ref)
            .collect();
        require(
            unique.len() == items.len(),
            "actualisation ancestry contains duplicate refs",
        )?;
        for (index, item) in items.iter().enumerate() {
            let expected = if index == 0 {
                Slot::Absent
            } else {
                Slot::Value(items[index - 1].fields().determination_ref.clone())
            };
            require(
                item.fields().parent_determination_ref == expected,
                "actualisation requires complete contiguous ancestry",
            )?;
        }
        let items = NonEmpty::new(items)?;
        DeterminationLineage::new(items.clone())?;
        Ok(Self(items))
    }
    pub fn items(&self) -> &[Determination] {
        self.0.as_slice()
    }
}
/// The semantic relation has actually been admitted. This value has no
/// process, provider, materialisation or Recognition side effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actualisation {
    request: ActualisationRequest,
    lineage: ActualisationLineage,
    operations_used: Vec<MetagencyOperation>,
}
impl Actualisation {
    fn admit(request: ActualisationRequest) -> Result<Self> {
        let r = request.fields();
        let parent = r.governing_binding.fields();
        let child = r.differentiated_binding.fields();
        let grant = r.metagency_grant.fields();
        let d = r.determination.fields();
        let lineage = ActualisationLineage::new(
            r.prior_determinations
                .value()
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            r.determination.clone(),
        )?;
        let derives = d.kind == DeterminationKind::Derivation;
        require(
            (r.agent_identity.standing == IdentityStanding::Actualised) == derives,
            "only derivation actualises a new Agent identity",
        )?;
        require(
            !derives || child.agent_ref != parent.agent_ref,
            "derivation requires a distinct Agent identity",
        )?;
        require(
            d.kind != DeterminationKind::SelfDifferentiation || child.agent_ref == parent.agent_ref,
            "self-differentiation preserves Agent identity",
        )?;
        require(
            grant.agency_ref == parent.agency_ref && grant.world_binding_ref == parent.binding_ref,
            "grant must target exact governing Agency and WorldBinding",
        )?;
        require(
            r.governing_binding
                .authorities()
                .contains(&grant.authority_ref),
            "grant authority absent from governing binding",
        )?;
        require(
            r.metagency_grant
                .includes(MetagencyOperation::DetermineAgency),
            "grant does not authorise determine-agency",
        )?;
        let mut operations_used = vec![MetagencyOperation::DetermineAgency];
        if derives {
            require(
                r.metagency_grant
                    .includes(MetagencyOperation::ActualiseAgency),
                "derivation requires explicit actualise-agency grant",
            )?;
            operations_used.push(MetagencyOperation::ActualiseAgency);
        }
        require(
            d.determining_agency_ref == parent.agency_ref,
            "determination must originate from governing Agency",
        )?;
        require(
            d.differentiated_agency_ref == child.agency_ref
                && d.world_binding_ref == child.binding_ref,
            "determination must identify differentiated Agency and binding exactly",
        )?;
        require(
            child.determining_agency_ref.value() == Some(&parent.agency_ref),
            "child must retain determining Agency",
        )?;
        require(
            same_unique_set(r.differentiated_binding.bounds(), d.bounds_refs.as_slice()),
            "child must exactly preserve determination bounds",
        )?;
        let grant_bounds = grant
            .bounds_refs
            .value()
            .ok_or_else(|| Error::new("actualisation needs explicit grant bounds"))?;
        let parent_bounds = r.governing_binding.bounds();
        require(
            !grant_bounds.is_empty() && !parent_bounds.is_empty(),
            "actualisation bounds must not be empty",
        )?;
        require(
            grant_bounds.iter().all(|b| parent_bounds.contains(b)),
            "grant exceeds governing bounds",
        )?;
        require(
            d.bounds_refs
                .as_slice()
                .iter()
                .all(|b| grant_bounds.contains(b)),
            "determination exceeds grant bounds",
        )?;
        let authority = d.authority_refs.value().map(Vec::as_slice).unwrap_or(&[]);
        require(
            !r.governing_binding.authorities().is_empty(),
            "governing authority must be supplied",
        )?;
        require(
            authority
                .iter()
                .all(|a| r.governing_binding.authorities().contains(a)),
            "determination cannot invent authority",
        )?;
        require(
            same_unique_set(r.differentiated_binding.authorities(), authority),
            "child must exactly preserve determination authority",
        )?;
        require(
            child.return_relation_ref.value()
                == d.return_policy.fields().return_relation_ref.value(),
            "child must exactly preserve Return relation without fabrication",
        )?;
        Ok(Self {
            request,
            lineage,
            operations_used,
        })
    }
    pub fn binding(&self) -> &WorldBinding {
        &self.request.fields().differentiated_binding
    }
    pub fn lineage(&self) -> &ActualisationLineage {
        &self.lineage
    }
    pub fn request(&self) -> &ActualisationRequest {
        &self.request
    }
    pub fn receipt(&self) -> Value {
        let r = self.request.fields();
        let d = r.determination.fields();
        let grant = r.metagency_grant.fields();
        let mut return_relation = json!({"mode": d.return_policy.fields().mode});
        if let Some(reference) = d.return_policy.fields().return_relation_ref.value() {
            return_relation["return_relation_ref"] = json!(reference);
        }
        let mut agencies = vec![self.lineage.items()[0]
            .fields()
            .determining_agency_ref
            .clone()];
        agencies.extend(
            self.lineage
                .items()
                .iter()
                .map(|d| d.fields().differentiated_agency_ref.clone()),
        );
        json!({
            "schema": AGENCY_ACTUALISATION_VERSION, "receipt_ref": format!("{}:receipt", r.request_ref),
            "request_ref": r.request_ref, "requester_ref": r.requester_ref, "status": "actualised",
            "governing_binding": r.governing_binding, "differentiated_binding": r.differentiated_binding,
            "metagency": {"grant_ref": grant.grant_ref, "authority_ref": grant.authority_ref, "operations_used": self.operations_used},
            "determination": r.determination,
            "lineage": {"determination_refs": self.lineage.items().iter().map(|d| &d.fields().determination_ref).collect::<Vec<_>>(), "agency_refs": agencies},
            "bounds_refs": d.bounds_refs, "return_relation": return_relation,
            "agent_identity": {"standing": r.agent_identity.standing, "agent_ref": r.differentiated_binding.fields().agent_ref, "evidence_refs": r.agent_identity.evidence_refs},
            "effects": {"semantic_relation": "actualised", "materialisation": "not-performed", "factory_recognition": "not-performed", "source_mutation": "not-performed"},
            "provenance": {"source_refs": r.provenance.source_refs, "context_refs": r.provenance.context_refs.value().map(Vec::as_slice).unwrap_or(&[])}
        })
    }
    /// Correlate a Return to the admitted lineage before any later synthesis.
    /// This never flips reception, Recognition or applied-world state.
    pub fn admit_return(&self, returned: Return) -> Result<Return> {
        let d = self.request.fields().determination.fields();
        let value = returned.fields();
        require(
            value.determination_ref == d.determination_ref
                && value.from_agency_ref == d.differentiated_agency_ref
                && value.to_agency_ref == d.determining_agency_ref,
            "Return must preserve its exact determination and Agency relation",
        )?;
        let expected = &value.provenance.fields().agency_lineage_refs;
        require(
            expected.as_slice().contains(&d.determining_agency_ref)
                && expected.as_slice().contains(&d.differentiated_agency_ref),
            "Return cannot erase its governing or differentiated provenance",
        )?;
        Ok(returned)
    }
}
