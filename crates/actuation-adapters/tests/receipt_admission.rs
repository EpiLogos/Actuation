use actuation_adapters::*;
use actuation_core::{ActuationRef, AgencyRef, WorldBindingRef};
use serde_json::{json, Value};
fn receipt() -> Value {
    json!({"schema":INSTANTIATION_VERSION,"actuation_ref":"actuation:one","agency_ref":"agency:one","world_binding_ref":"binding:one","model_relation":{"schema":INSTANTIATION_VERSION,"model_ref":"model:opaque","inference_surface":{"contract_ref":"contract:opaque"}},"access_profile":{"schema":INSTANTIATION_VERSION,"inference":{"allowed":["invoke"]},"control":{"allowed":[]},"interior":{"depth":"opaque"}},"evidence_refs":["evidence:supplied"]})
}
#[test]
fn shaped_receipt_still_requires_exact_contextual_identity_before_readback_admission() {
    let r = InstantiationReceipt::read(receipt()).unwrap();
    let a = ActuationRef::new("actuation:one").unwrap();
    let g = AgencyRef::new("agency:one").unwrap();
    let w = WorldBindingRef::new("binding:one").unwrap();
    r.require_correlation(&a, &g, &w).unwrap();
    assert!(r
        .require_correlation(&ActuationRef::new("actuation:other").unwrap(), &g, &w)
        .is_err());
    assert!(r
        .require_correlation(&a, &AgencyRef::new("agency:other").unwrap(), &w)
        .is_err());
    assert!(r
        .require_correlation(&a, &g, &WorldBindingRef::new("binding:other").unwrap())
        .is_err());
    for k in [
        "agent_ref",
        "caller_ref",
        "recognition_ref",
        "run_ref",
        "journey_ref",
    ] {
        assert!(r.as_value().get(k).is_none());
    }
}
#[test]
fn model_access_is_not_model_control_interior_access_or_an_actual_act() {
    let r = InstantiationReceipt::read(receipt()).unwrap();
    assert_eq!(
        r.as_value()["access_profile"]["control"]["allowed"],
        json!([])
    );
    assert_eq!(
        r.as_value()["access_profile"]["interior"]["depth"],
        "opaque"
    );
    assert!(r.as_value().get("acting").is_none());
    let mut wrong = receipt();
    wrong["access_profile"]["interior"]["depth"] = json!("assumed-control");
    assert!(InstantiationReceipt::read(wrong).is_err());
}
#[test]
fn target_or_material_replacement_does_not_change_enduring_correlations() {
    let mut a = receipt();
    let mut b = receipt();
    a["harness_ref"] = json!("harness/pi");
    b["harness_ref"] = json!("harness/codex");
    a["model_relation"]["material"] = json!({"binding_ref":"workcell:first","placement":"local"});
    b["model_relation"]["material"] = json!({"binding_ref":"workcell:second","placement":"remote"});
    let a = InstantiationReceipt::read(a).unwrap();
    let b = InstantiationReceipt::read(b).unwrap();
    for k in ["actuation_ref", "agency_ref", "world_binding_ref"] {
        assert_eq!(a.as_value()[k], b.as_value()[k]);
    }
    assert_ne!(a.as_value()["harness_ref"], b.as_value()["harness_ref"]);
}
#[test]
fn receipt_null_extension_and_legacy_read_preserve_the_declared_wire_window() {
    let mut v = receipt();
    v["future_owner_data"] = json!({"x":null});
    v["return_ref"] = Value::Null;
    let r = InstantiationReceipt::read(v.clone()).unwrap();
    assert_eq!(r.into_value(), v);
    v["schema"] = json!(LEGACY_MODEL_BEARING_SCHEMA);
    v["model_relation"]["schema"] = json!(LEGACY_MODEL_BEARING_SCHEMA);
    v["access_profile"]["schema"] = json!(LEGACY_MODEL_BEARING_SCHEMA);
    let r = InstantiationReceipt::read(v).unwrap();
    assert_eq!(r.as_value()["schema"], INSTANTIATION_VERSION);
    assert!(r.as_value().get("run_ref").is_none());
}
