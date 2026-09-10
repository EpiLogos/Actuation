use crate::{admission::*, Error, Result};
use actuation_core::{ActuationRef, AgencyRef, WorldBindingRef};
use serde_json::{json, Value};

impl InstantiationReceipt {
    /// The explicit legacy-read window changes only the three known schema
    /// labels. It does not mint identity, authority, access or observed use.
    pub fn read(mut value: Value) -> Result<Self> {
        if value["schema"] == LEGACY_MODEL_BEARING_SCHEMA {
            value["schema"] = json!(INSTANTIATION_VERSION);
            for key in ["model_relation", "access_profile"] {
                if value[key]["schema"] == LEGACY_MODEL_BEARING_SCHEMA {
                    value[key]["schema"] = json!(INSTANTIATION_VERSION);
                }
            }
        }
        Self::try_from(value)
    }
    pub fn with_detection(&self, detection: &HarnessDetection) -> Result<Self> {
        Self::try_from(attach_detection_evidence(
            self.as_value(),
            detection.as_value(),
        )?)
    }
    /// Contextual readback admission: an already-shaped receipt must still
    /// name the actual request's identities. Shape alone is not execution.
    pub fn require_correlation(
        &self,
        actuation: &ActuationRef,
        agency: &AgencyRef,
        world: &WorldBindingRef,
    ) -> Result<()> {
        require(
            self.as_value()["actuation_ref"] == actuation.as_str(),
            "receipt belongs to another actuation",
        )?;
        require(
            self.as_value()["agency_ref"] == agency.as_str(),
            "receipt belongs to another Agency",
        )?;
        require(
            self.as_value()["world_binding_ref"] == world.as_str(),
            "receipt belongs to another WorldBinding",
        )
    }
}
/// Compatibility attachment is not complete receipt admission. Production
/// callers should use InstantiationReceipt::read and with_detection. Keeping
/// this narrow existing operation does not authorise dispatch or infer caller.
pub fn attach_detection_evidence(receipt: &Value, detection: &Value) -> Result<Value> {
    object(receipt)?;
    let mut result = receipt.clone();
    let harness = receipt["harness_ref"].as_str().filter(|s| !s.is_empty());
    let Some(harness) = harness else {
        result["unattributed"] = json!(true);
        return Ok(result);
    };
    let detection = HarnessDetection::try_from(detection.clone())?;
    let slug = harness.strip_prefix("harness/").unwrap_or(harness);
    let e = detection.as_value()["harnesses"]
        .as_array()
        .expect("admitted detection")
        .iter()
        .find(|e| {
            e["slug"] == slug
                && e["state"] == "detected"
                && e["receipts"]["executable"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
        })
        .ok_or_else(|| {
            Error::new(format!(
                "instantiation refused: {harness} is not detected with receipts"
            ))
        })?;
    result["detection_ref"] = detection.as_value()["detection_ref"].clone();
    result["harness_receipts"] = e["receipts"].clone();
    Ok(result)
}
