use crate::wire::*;
use crate::{DetectionReport, Error, Presence, Result};
use actuation_core::{ActuationRef, AgencyRef, WorldBindingRef};
use serde_json::{json, Value};
pub const ACTUATION_INSTANTIATION_VERSION: &str = "actuation.instantiation/v1";
pub const LEGACY_MODEL_BEARING_SCHEMA: &str = "actuation.model-bearing/v1";
fn check_relation(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], ACTUATION_INSTANTIATION_VERSION, "schema")?;
    text(&v["model_ref"], "model_ref")?;
    optional_text(v, "variant_ref")?;
    if let Some(e) = present(v, "engine") {
        object(e)?;
        optional_texts(e, &["implementation_ref", "provider_ref"])?;
        facts(&e["facts"])?;
    }
    if let Some(m) = present(v, "material") {
        object(m)?;
        optional_text(m, "binding_ref")?;
        if let Some(p) = present(m, "placement") {
            choice(
                p,
                &["local", "remote", "distributed", "opaque"],
                "placement",
            )?;
        }
        facts(&m["facts"])?;
    }
    let s = &v["inference_surface"];
    object(s)?;
    text(&s["contract_ref"], "inference_surface.contract_ref")?;
    optional_text(s, "binding_ref")?;
    facts(&s["facts"])
}
document!(ModelRelation, check_relation);
fn check_access(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], ACTUATION_INSTANTIATION_VERSION, "schema")?;
    for key in ["inference", "control"] {
        let p = &v[key];
        object(p)?;
        strings(&p["allowed"], "allowed")?;
        optional_strings(p, "denied")?;
    }
    let p = &v["interior"];
    object(p)?;
    choice(
        &p["depth"],
        &[
            "opaque",
            "behavioral",
            "outputs",
            "state-read",
            "state-write",
            "causal-intervention",
            "learning",
        ],
        "interior.depth",
    )?;
    optional_strings(p, "allowed")?;
    optional_strings(p, "denied")
}
document!(ModelAccessProfile, check_access);
impl ModelAccessProfile {
    /// A receipt describes access; it does not establish an authority grant.
    pub fn records_inference(&self, operation: &str) -> bool {
        self.0["inference"]["allowed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == operation)
            && !self.0["inference"]["denied"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v == operation))
    }
}
fn check_receipt(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], ACTUATION_INSTANTIATION_VERSION, "schema")?;
    required_texts(v, &["actuation_ref", "agency_ref", "world_binding_ref"])?;
    optional_texts(
        v,
        &[
            "harness_ref",
            "harness_composition_ref",
            "agent_session_ref",
            "return_ref",
        ],
    )?;
    check_relation(&v["model_relation"])?;
    check_access(&v["access_profile"])?;
    optional_strings(v, "bounds_refs")?;
    optional_strings(v, "evidence_refs")?;
    if let Some(e) = present(v, "experiment") {
        object(e)?;
        optional_strings(e, "held_constant_refs")?;
        if let Some(vars) = present(e, "variables") {
            for var in list(vars, "experiment.variables")? {
                object(var)?;
                text(&var["name"], "variable.name")?;
                optional_text(var, "value_ref")?;
                facts(&var["facts"])?;
            }
        }
    }
    if let Some(t) = present(v, "observed_at") {
        timestamp(t, "observed_at")?;
    }
    Ok(())
}
document!(InstantiationReceipt, check_receipt);
impl InstantiationReceipt {
    /// The one explicit legacy-schema normalisation window. Validation itself
    /// never quietly rewrites a current schema or infers missing identity.
    pub fn read(mut value: Value) -> Result<Self> {
        if value["schema"] == LEGACY_MODEL_BEARING_SCHEMA {
            value["schema"] = json!(ACTUATION_INSTANTIATION_VERSION);
            for key in ["model_relation", "access_profile"] {
                if value[key]["schema"] == LEGACY_MODEL_BEARING_SCHEMA {
                    value[key]["schema"] = json!(ACTUATION_INSTANTIATION_VERSION);
                }
            }
        }
        Self::new(value)
    }
    pub fn actuation_ref(&self) -> ActuationRef {
        serde_json::from_value(self.0["actuation_ref"].clone()).expect("admitted actuation ref")
    }
    pub fn agency_ref(&self) -> AgencyRef {
        serde_json::from_value(self.0["agency_ref"].clone()).expect("admitted agency ref")
    }
    pub fn world_binding_ref(&self) -> WorldBindingRef {
        serde_json::from_value(self.0["world_binding_ref"].clone()).expect("admitted binding ref")
    }
    pub fn attach_detection(&self, detection: &DetectionReport) -> Result<Self> {
        let mut value = self.0.clone();
        if !truthy(&value["harness_ref"]) {
            value["unattributed"] = json!(true);
            return Self::new(value);
        }
        let reference = text(&value["harness_ref"], "harness_ref")?;
        let slug = reference.strip_prefix("harness/").unwrap_or(reference);
        let entry = detection
            .entry(slug)
            .filter(|_| detection.presence(slug) == Some(Presence::Detected))
            .filter(|e| truthy(&e["receipts"]["executable"]))
            .ok_or_else(|| {
                Error::new("receipt's harness is not detected with same-run receipts")
            })?;
        value["detection_ref"] = json!(detection.reference());
        value["harness_receipts"] = entry["receipts"].clone();
        Self::new(value)
    }
    /// Unknown harness attribution is explicit. It does not bypass receipt
    /// validation, and is never evidence that an invocation actually ran.
    pub fn mark_unattributed(&self) -> Result<Self> {
        if truthy(&self.0["harness_ref"]) {
            return Err(Error::new(
                "a named harness requires its detection evidence",
            ));
        }
        let mut value = self.0.clone();
        value["unattributed"] = json!(true);
        Self::new(value)
    }
}
