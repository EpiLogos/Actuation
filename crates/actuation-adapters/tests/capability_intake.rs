//! The public intake law for harness capability descriptors (O:I #113 A1):
//! the gemini specimen contributed by the uncoached external author
//! validates; a fabricated event kind is refused by the closed vocabulary;
//! a descriptor that would shadow a declared capability fails coverage
//! closure; an unknown slug fails slug alignment.
use actuation_adapters::*;
use serde_json::{json, Value};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn gemini_specimen() -> Value {
    let path = format!("{ROOT}/fixtures/capability-contributions/gemini.harness-capability.json");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    serde_json::from_str(&raw).expect("the gemini specimen is JSON")
}

fn result_of(document: &Value, check: &str) -> String {
    document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check"] == json!(check))
        .expect("the check is named in the document")["result"]
        .as_str()
        .expect("admitted result")
        .to_owned()
}

#[test]
fn the_gemini_specimen_validates_against_the_intake_law() {
    let catalog = NativeCatalog::bundled().unwrap();
    let document = validate_capability_document(&catalog, &gemini_specimen());
    assert_eq!(document["valid"], json!(true), "{document}");
    assert_eq!(document["harness_slug"], json!("gemini"));
    for check in ["schema-admission", "slug-alignment", "coverage-closure"] {
        assert_eq!(result_of(&document, check), "pass", "{check} must pass");
    }
    // The receipt the contribution face mints carries the bytes' digest, the
    // unchanged descriptor, and the owner landing edit.
    let raw = serde_json::to_string(&gemini_specimen()).unwrap();
    let receipt = capability_contribution_receipt(
        &catalog,
        &gemini_specimen(),
        "fixtures/capability-contributions/gemini.harness-capability.json",
        &raw,
        1_000,
    )
    .expect("a validated specimen is received");
    assert_eq!(receipt["status"], json!("received"));
    assert_eq!(receipt["harness_slug"], json!("gemini"));
    assert!(
        receipt["contribution_ref"]
            .as_str()
            .unwrap()
            .starts_with("capability-contribution:gemini:"),
        "{}",
        receipt["contribution_ref"]
    );
    assert_eq!(
        receipt["landing"]["current_revision"],
        json!(catalog.revision())
    );
    assert_eq!(
        receipt["landing"]["landed_revision"],
        json!(catalog.revision() + 1)
    );
    assert!(
        receipt["landing"]["edit"]
            .as_str()
            .unwrap()
            .contains("capability_gaps -= gemini"),
        "{}",
        receipt["landing"]["edit"]
    );
    // The descriptor travels unchanged.
    assert_eq!(receipt["capability"], gemini_specimen());
}

#[test]
fn a_fabricated_event_kind_is_refused_by_the_closed_vocabulary() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut fabricated = gemini_specimen();
    fabricated["native_events"][0]["event"] = json!("teleport");
    fabricated["native_events"][0]["native_name"] = json!("Teleport");
    let document = validate_capability_document(&catalog, &fabricated);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "schema-admission"), "fail");
    let detail = document["checks"][0]["detail"].as_str().unwrap();
    assert!(
        detail.contains("expected one of"),
        "the refusal names the closed event vocabulary: {detail}"
    );
    assert!(capability_contribution_receipt(&catalog, &fabricated, "stdin", "{}", 0).is_err());
}

#[test]
fn a_descriptor_shadowing_a_declared_capability_fails_coverage_closure() {
    let catalog = NativeCatalog::bundled().unwrap();
    let declared = catalog.capability("pi").expect("pi is declared").as_value();
    let document = validate_capability_document(&catalog, declared);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "coverage-closure"), "fail");
    let detail = document["checks"][2]["detail"].as_str().unwrap();
    assert!(
        detail.contains("already carries a declared capability"),
        "the refusal names the shadowing: {detail}"
    );
    assert!(
        detail.contains("owner edit"),
        "the refusal points at the correction discipline: {detail}"
    );
}

#[test]
fn an_unknown_slug_fails_slug_alignment() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut stranger = gemini_specimen();
    stranger["harness_slug"] = json!("invented-harness");
    let document = validate_capability_document(&catalog, &stranger);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "slug-alignment"), "fail");
    let detail = document["checks"][1]["detail"].as_str().unwrap();
    assert!(
        detail.contains("names no detection descriptor"),
        "the refusal names the misalignment: {detail}"
    );
}
