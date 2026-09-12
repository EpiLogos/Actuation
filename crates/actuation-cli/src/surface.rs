//! Identity constants for the Actuation CLI. The command list is NOT
//! transcribed here: it is derived from the command table in `dispatch.rs`,
//! which is the single source of truth for routes, usage and handlers.
//! surface ↔ table parity is asserted by the crate tests.
use actuation_adapters::{
    HARNESS_CAPABILITY_VERSION, HARNESS_DETECTION_VERSION, INSTANTIATION_VERSION,
    LEGACY_MODEL_BEARING_SCHEMA,
};
pub use actuation_core::AGENCY_CONTRACT_VERSION;
use actuation_runtime::{AGENCY_ACTUALISATION_VERSION, REALISED_ACTUATION_VERSION};
use serde_json::{json, Value};

pub const ACTUATION_CLI_VERSION: &str = "0.2.0";
pub const ACTUATION_CLI_CONTRACT: &str = "actuation.cli/v1";

/// Frozen Wave 5 System disclosure identity, kept beside the surface it
/// discloses so both the capabilities listing and the disclosure builder
/// agree without a module cycle.
pub const SYSTEM_DISCLOSURE_VERSION: &str = "oi.product-settings-disclosure/v2";
pub const SYSTEM_DISCLOSURE_CONTRACT_REVISION: &str = "wave-5/system.1";

pub const ACTIVITY_VERSION: &str = "actuation.activity/v1";
pub const ACTUATION_STREAM_VERSION: &str = "actuation.stream/v1";
pub const MODEL_USAGE_VERSION: &str = "actuation.model-usage/v1";

pub fn native_contracts() -> Value {
    json!({
        "agency": AGENCY_CONTRACT_VERSION,
        "agency_actualisation": AGENCY_ACTUALISATION_VERSION,
        "realised": REALISED_ACTUATION_VERSION,
        "stream": ACTUATION_STREAM_VERSION,
        "activity": ACTIVITY_VERSION,
        "model_usage": MODEL_USAGE_VERSION,
        "instantiation": INSTANTIATION_VERSION,
        "model_bearing_legacy": LEGACY_MODEL_BEARING_SCHEMA,
        "harness_detection": HARNESS_DETECTION_VERSION,
        "harness_capability": HARNESS_CAPABILITY_VERSION,
        "system_disclosure": SYSTEM_DISCLOSURE_VERSION,
    })
}

pub fn cli_surface() -> Value {
    json!({
        "contract": ACTUATION_CLI_CONTRACT,
        "product": "actuation",
        "executable": "actuation",
        "version": ACTUATION_CLI_VERSION,
        "native_contracts": native_contracts(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_versions_match_the_typed_crates() {
        assert_eq!(AGENCY_CONTRACT_VERSION, "actuation.agency/v1");
        assert_eq!(
            AGENCY_ACTUALISATION_VERSION,
            "actuation.agency-actualisation/v1"
        );
        assert_eq!(REALISED_ACTUATION_VERSION, "actuation.realised/v1");
        assert_eq!(HARNESS_DETECTION_VERSION, "actuation.harness-detection/v1");
        assert_eq!(
            HARNESS_CAPABILITY_VERSION,
            "actuation.harness-capability/v1"
        );
        assert_eq!(INSTANTIATION_VERSION, "actuation.instantiation/v1");
        assert_eq!(LEGACY_MODEL_BEARING_SCHEMA, "actuation.model-bearing/v1");
    }
}
