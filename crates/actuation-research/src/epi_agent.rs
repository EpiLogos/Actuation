//! Epi-Logos Prime-QL constituted body.
//!
//! Actuation owns the acting #0/1 body. QL-MEF owns the explicit #0..#5
//! faculties. This module joins those owners without reminting Agent, provider,
//! SessionSpace or World identity.

use crate::{
    owner::OwnerInstrument,
    prime_run::{run_prime, PrimeRunRequest},
    toolset,
    world::World,
    Error, Result,
};
use actuation_runtime::RuntimeObserver;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const EPI_PRIME_QL_BODY_SCHEMA: &str = "actuation.epi-prime-ql-body/v1";
pub const EPI_PRIME_QL_RUN_SCHEMA: &str = "actuation.epi-prime-ql-run/v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostMode {
    Expressions,
    Techne,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpiPrimeQlRunRequest {
    pub host_mode: HostMode,
    pub subject_ref: String,
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub now_ref: Option<String>,
    #[serde(default)]
    pub prepared_context_ref: Option<String>,
    pub prime: PrimeRunRequest,
}

fn text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(Error::new(format!("{label} must be non-empty")))
    } else {
        Ok(())
    }
}

fn owner_cli(owner: &OwnerInstrument, arguments: Value) -> Result<Value> {
    Ok(owner.invoke(json!({"operation":"cli","arguments":arguments}))?["result"].clone())
}

pub fn body(owner: &OwnerInstrument) -> Result<Value> {
    let constitution = owner_cli(owner, json!(["epi-agent", "constitution"]))?;
    if constitution["schema"] != "ql.epi-logos-agent-constitution/v1"
        || constitution["whole"]["coordinate"] != "#0/1"
        || constitution["whole"]["distinct_from"] != "#0"
        || constitution["faculties"].as_array().map(Vec::len) != Some(6)
        || constitution["faculties"][4]["identity"] != "M4/M4′"
        || constitution["faculties"][5]["identity"] != "M5/M5′"
        || constitution["faculties"][4]["s_prime"] != "S4′ Anima"
        || constitution["faculties"][5]["s_prime"] != "S5′ Aletheia"
    {
        return Err(Error::new(
            "QL owner returned an incompatible Epi constitution",
        ));
    }
    let faculties = (0..6)
        .map(|position| {
            owner_cli(
                owner,
                json!(["epi-agent", "faculty", format!("#{position}")]),
            )
            .and_then(|value| {
                if value["schema"] != "ql.epi-logos-agent-faculty/v1"
                    || value["position"] != format!("#{position}")
                {
                    Err(Error::new(format!(
                        "QL faculty #{position} readback mismatched"
                    )))
                } else {
                    Ok(value)
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "schema":EPI_PRIME_QL_BODY_SCHEMA,
        "coordinate":"#0/1",
        "distinct_from":"#0",
        "prime":"native-prime-recursion",
        "ql_recurrence":"native-actuation-relational",
        "orientation_source":"crates/actuation-research/src/ql-relational-system-prompt.md",
        "operative_reference":{
            "condition":"ql-twelve",
            "supply":toolset::night_capability_supply(),
            "standing":"reference-supply; product Prime delivery remains skill/faculty mediated until the native Prime transport advertises these as first-class tools"
        },
        "faculty_owner_basis":owner.basis(),
        "domain_constitution":constitution,
        "faculties":faculties,
        "model_selection_owner":"AIKit/provider",
        "session_owner":"AIKit",
        "world_owner":"Central/AIKit composition",
        "canonical_mutation":false
    }))
}

fn position_for_operation(operation: &str) -> Option<u8> {
    match operation {
        "anuttara-read" => Some(0),
        "tda-vietoris-rips" | "kernel-apply" | "mef-lenses" | "context-frames" => Some(1),
        "bimba-neighborhood" => Some(2),
        "representation-bind" => Some(3),
        "nara-activity-validate" | "nara-elemental-map" => Some(4),
        "logos-return" => Some(5),
        _ => None,
    }
}

pub fn run(
    request: &EpiPrimeQlRunRequest,
    world: &World,
    owner: &OwnerInstrument,
    observer: &mut dyn RuntimeObserver,
) -> Result<Value> {
    text(&request.subject_ref, "subject_ref")?;
    if request.source_refs.is_empty()
        || request
            .source_refs
            .iter()
            .any(|source| source.trim().is_empty())
    {
        return Err(Error::new(
            "Epi Prime-QL run requires non-empty source_refs",
        ));
    }
    if request.prime.condition != "prime-recursive-field" {
        return Err(Error::new(
            "Epi Prime-QL default currently requires prime-recursive-field; retained research conditions remain independently invocable",
        ));
    }
    if request.prime.allow_refinement {
        return Err(Error::new(
            "mode selection does not authorise continual refinement",
        ));
    }
    if request.prime.skill_path.is_none()
        || request.prime.research_binary.is_none()
        || request.prime.faculty_config.is_none()
    {
        return Err(Error::new(
            "Epi Prime-QL requires the inherited relational skill and native faculty bridge",
        ));
    }

    let resolved_body = body(owner)?;
    let prime = run_prime(&request.prime, world, Some(owner), observer)?;
    let receipts = prime["faculty"]["receipts"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut invoked = [false; 6];
    for receipt in &receipts {
        if receipt["success"] != true {
            continue;
        }
        if let Some(operation) = receipt["operation"].as_str() {
            if let Some(position) = position_for_operation(operation) {
                invoked[usize::from(position)] = true;
            }
        }
    }

    Ok(json!({
        "schema":EPI_PRIME_QL_RUN_SCHEMA,
        "host_mode":request.host_mode,
        "subject_ref":request.subject_ref,
        "source_refs":request.source_refs,
        "now_ref":request.now_ref,
        "prepared_context_ref":request.prepared_context_ref,
        "resolved_body":resolved_body,
        "prime":prime,
        "faculty_invocation":{
            "observed":[
                {"position":"#0","invoked":invoked[0]},
                {"position":"#1","invoked":invoked[1]},
                {"position":"#2","invoked":invoked[2]},
                {"position":"#3","invoked":invoked[3]},
                {"position":"#4","invoked":invoked[4]},
                {"position":"#5","invoked":invoked[5]}
            ],
            "receipt_count":receipts.len(),
            "standing":"actual native receipts; availability is not invocation"
        },
        "claims":{
            "prime_ql_body_executed":prime["claims"]["prime_body_executed"],
            "child_loci_observed":prime["claims"]["observed_child_loci"],
            "human_acceptance":false,
            "installed_mode_default":false
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_to_faculty_mapping_keeps_zero_explicit() {
        assert_eq!(position_for_operation("anuttara-read"), Some(0));
        assert_eq!(position_for_operation("tda-vietoris-rips"), Some(1));
        assert_eq!(position_for_operation("bimba-neighborhood"), Some(2));
        assert_eq!(position_for_operation("representation-bind"), Some(3));
        assert_eq!(position_for_operation("nara-elemental-map"), Some(4));
        assert_eq!(position_for_operation("logos-return"), Some(5));
        assert_eq!(position_for_operation("capabilities"), None);
    }
}
